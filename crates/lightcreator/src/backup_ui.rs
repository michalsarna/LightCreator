//! Help > Download backup…: one zip file with everything the program keeps on this computer, the settings
//! (device profiles, language, material presets and so on) and the clipart you saved.
use crate::app::{App, APP_VERSION};
use crate::i18n::trf;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

/// `yyyymmdd` of a Unix time, for the suggested file name.
pub fn ymd(secs: u64) -> String {
    // Days since 1970-01-01 to a calendar date (Howard Hinnant's algorithm).
    let z = (secs / 86400) as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    format!("{:04}{:02}{:02}", if m <= 2 { y + 1 } else { y }, m, d)
}

fn files_in(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = rd.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let p = e.path();
        if p.is_dir() {
            files_in(&p, out);
        } else if p.is_file() {
            out.push(p);
        }
    }
}

impl App {
    /// Ask where to put the backup and write it.
    pub fn download_backup(&mut self) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let name = format!("lightcreator-backup-{}.zip", ymd(now));
        let Some(dest) = rfd::FileDialog::new().add_filter("Zip", &["zip"]).set_file_name(&name).save_file() else { return };
        match self.write_backup(&dest) {
            Ok(n) => {
                let file = dest.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                self.status = trf("Backup saved: {}", &[&dest.display()]);
                self.toast(trf("Backup saved: {}", &[&file]));
                let _ = n;
            }
            Err(e) => self.status = trf("Backup failed: {}", &[&e]),
        }
    }

    /// Write the zip: `app.ron` (the settings as they are now), the whole `clipart` folder and a short README.
    /// Returns the number of files in it.
    pub fn write_backup(&self, dest: &Path) -> Result<usize, String> {
        let file = std::fs::File::create(dest).map_err(|e| e.to_string())?;
        let mut zip = zip::ZipWriter::new(file);
        let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        let mut count = 0usize;
        let mut add = |zip: &mut zip::ZipWriter<std::fs::File>, name: &str, data: &[u8]| -> Result<(), String> {
            zip.start_file(name, opts).map_err(|e| e.to_string())?;
            zip.write_all(data).map_err(|e| e.to_string())?;
            count += 1;
            Ok(())
        };
        // The same key / value strings the program stores itself, in the format of its settings file.
        let map: BTreeMap<&str, String> = self.settings_pairs().into_iter().collect();
        let ron_text = ron::ser::to_string_pretty(&map, ron::ser::PrettyConfig::default()).map_err(|e| e.to_string())?;
        add(&mut zip, "app.ron", ron_text.as_bytes())?;
        let dir = self.clip.lib.dir().to_path_buf();
        let mut files = vec![];
        files_in(&dir, &mut files);
        for f in &files {
            let rel = f.strip_prefix(&dir).unwrap_or(f).to_string_lossy().replace('\\', "/");
            let data = std::fs::read(f).map_err(|e| format!("{}: {}", f.display(), e))?;
            add(&mut zip, &format!("clipart/{rel}"), &data)?;
        }
        let readme = format!(
            "LightCreator {APP_VERSION} backup\n\n\
             app.ron   your settings: device profiles, language, colour scheme, material presets, recent files\n\
             clipart/  the pictures you saved or downloaded, with index.json\n\n\
             To restore, close LightCreator and copy app.ron and the clipart folder into its data folder\n\
             (macOS: ~/Library/Application Support/lightcreator, Windows: %APPDATA%\\lightcreator\\data,\n\
             Linux: ~/.local/share/lightcreator).\n"
        );
        add(&mut zip, "README.txt", readme.as_bytes())?;
        zip.finish().map_err(|e| e.to_string())?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn dates_come_out_right() {
        assert_eq!(ymd(0), "19700101");
        assert_eq!(ymd(951_782_400), "20000229");
        assert_eq!(ymd(1_791_676_800), "20261011");
    }

    #[test]
    fn the_backup_holds_settings_and_clipart() {
        let ctx = eframe::egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        let dir = std::env::temp_dir().join(format!("lc-backup-{}", std::process::id()));
        let clip = dir.join("clipart");
        std::fs::create_dir_all(clip.join("Mine")).unwrap();
        std::fs::write(clip.join("index.json"), "{}").unwrap();
        std::fs::write(clip.join("Mine/star.svg"), "<svg/>").unwrap();
        a.clip = crate::clipart_ui::ClipState::new(lc_core::clipart::Library::open(clip));
        a.profiles = vec![lc_core::Device::default()];
        let zip_path = dir.join("backup.zip");
        let n = a.write_backup(&zip_path).expect("backup written");
        assert_eq!(n, 4);
        let mut z = zip::ZipArchive::new(std::fs::File::open(&zip_path).unwrap()).unwrap();
        let names: Vec<String> = (0..z.len()).map(|i| z.by_index(i).unwrap().name().unwrap_or_default().to_string()).collect();
        for want in ["app.ron", "clipart/index.json", "clipart/Mine/star.svg", "README.txt"] {
            assert!(names.iter().any(|n| n == want), "{want} in {names:?}");
        }
        let mut ron_text = String::new();
        z.by_name("app.ron").unwrap().read_to_string(&mut ron_text).unwrap();
        // It reads back as the settings map, with the profile in it.
        let map: BTreeMap<String, String> = ron::from_str(&ron_text).unwrap();
        assert!(map.get("profiles").is_some_and(|p| p.contains("Diode")), "{map:?}");
        assert_eq!(map.get("lang").map(String::as_str), Some("en"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
