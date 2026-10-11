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

    /// Choose a backup zip and restore from it. `confirm` asks first, because it replaces what is there now.
    pub fn restore_backup_dialog(&mut self, ctx: &eframe::egui::Context, confirm: bool) {
        let Some(src) = rfd::FileDialog::new().add_filter("Zip", &["zip"]).pick_file() else { return };
        if confirm {
            self.confirm = Some(crate::session_ui::Confirm {
                title: crate::i18n::tr("Restore from backup").to_string(),
                text: crate::i18n::tr("Your current settings and clipart will be replaced by the backup. Continue?").to_string(),
                act: crate::session_ui::ConfirmAct::RestoreBackup(src),
            });
            return;
        }
        self.restore_backup_from(ctx, &src);
    }

    /// Restore from `src` and report the result.
    pub fn restore_backup_from(&mut self, ctx: &eframe::egui::Context, src: &std::path::Path) {
        match self.restore_backup(ctx, src) {
            Ok(n) => {
                self.status = trf("Backup restored: {}", &[&src.display()]);
                self.toast(trf("Backup restored: {}", &[&src.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()]));
                let _ = n;
            }
            Err(e) => self.status = trf("Restore failed: {}", &[&e]),
        }
    }

    /// Put the settings and the clipart of a backup in place. Returns the number of files read.
    pub fn restore_backup(&mut self, ctx: &eframe::egui::Context, src: &Path) -> Result<usize, String> {
        let file = std::fs::File::open(src).map_err(|e| e.to_string())?;
        let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        // Settings first: a backup without them is not one of ours.
        let settings: BTreeMap<String, String> = {
            let mut f = zip.by_name("app.ron").map_err(|_| "app.ron not found in the file".to_string())?;
            let mut text = String::new();
            std::io::Read::read_to_string(&mut f, &mut text).map_err(|e| e.to_string())?;
            ron::from_str(&text).map_err(|e| e.to_string())?
        };
        let dir = self.clip.lib.dir().to_path_buf();
        let mut count = 1usize;
        for i in 0..zip.len() {
            let mut f = zip.by_index(i).map_err(|e| e.to_string())?;
            if f.is_dir() {
                continue;
            }
            // Only files below `clipart/`, and never outside the folder.
            let Some(rel) = f.enclosed_name().and_then(|p| p.strip_prefix("clipart").ok().map(Path::to_path_buf)) else { continue };
            if rel.as_os_str().is_empty() {
                continue;
            }
            let target = dir.join(rel);
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut out = std::fs::File::create(&target).map_err(|e| format!("{}: {}", target.display(), e))?;
            std::io::copy(&mut f, &mut out).map_err(|e| e.to_string())?;
            count += 1;
        }
        self.apply_settings(ctx, &settings);
        self.clip.lib = lc_core::clipart::Library::open(dir);
        self.clip.reload();
        Ok(count)
    }

    /// Take the settings of a backup (the strings of `settings_pairs`) into the running program.
    pub fn apply_settings(&mut self, ctx: &eframe::egui::Context, m: &BTreeMap<String, String>) {
        let get = |k: &str| m.get(k).map(String::as_str);
        if let Some(l) = get("lang").and_then(crate::i18n::Lang::from_code) {
            self.lang = l;
            crate::i18n::set_lang(l);
        }
        if let Some(s) = get("scheme").and_then(crate::theme::Scheme::from_id) {
            self.scheme = s;
            crate::theme::apply(ctx, s);
        }
        if let Some(p) = get("profiles").and_then(|j| serde_json::from_str::<Vec<lc_core::Device>>(j).ok()) {
            self.profiles = p;
        }
        self.active = get("active_profile").and_then(|v| v.parse::<usize>().ok()).filter(|i| *i < self.profiles.len()).unwrap_or(0);
        self.start_sel = self.active;
        if let Some(v) = get("autosave_on") {
            self.autosave_on = v != "0";
        }
        if let Some(v) = get("autosave_secs").and_then(|v| v.parse::<u32>().ok()).filter(|v| (10..=3600).contains(v)) {
            self.autosave_secs = v;
        }
        if let Some(v) = get("check_updates") {
            self.check_updates = v != "0";
        }
        if let Some(r) = get("recent_files").and_then(|j| serde_json::from_str::<Vec<PathBuf>>(j).ok()) {
            self.recent = r.into_iter().take(crate::menu::MAX_RECENT).collect();
        }
        if let Some(g) = get("grid_prefs").and_then(|j| serde_json::from_str(j).ok()) {
            self.grid_prefs = g;
        }
        if let Some(v) = get("grid_main_on") {
            self.show_grid = v != "0";
        }
        if let Some(v) = get("snap_objects") {
            self.snap_objects = v == "1";
        }
        if let Some(v) = get("zones_visible") {
            self.zones_visible = v != "0";
        }
        if let Some(v) = get("grid_main_mm").and_then(|v| v.parse::<f64>().ok()) {
            self.grid = v;
        }
        if get("global_materials").is_some() || get("user_materials").is_some() {
            self.global_materials = crate::materials_ui::load_global(&|k| m.get(k).cloned());
        }
        if let Some(l) = get("layer_labels").and_then(|j| serde_json::from_str(&j).ok()) {
            self.layer_labels = l;
        }
        if let Some(h) = get("mat_list_h").and_then(|v| v.parse::<f32>().ok()).filter(|h| (60.0..=700.0).contains(h)) {
            self.mat_list_h = h;
        }
        if let Some(o) = get("tab_order") {
            self.tab_order = crate::app::SideTab::arranged(o);
        }
        self.cfg = None;
        if self.screen == crate::app::Screen::Editor && !self.profiles.is_empty() {
            self.enter_editor(self.active);
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

    #[test]
    fn a_backup_restores_profiles_and_clipart_into_a_fresh_program() {
        let ctx = eframe::egui::Context::default();
        let dir = std::env::temp_dir().join(format!("lc-restore-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        // The program that makes the backup.
        let mut a = App::build(&ctx, None, false);
        let clip = dir.join("old/clipart");
        std::fs::create_dir_all(clip.join("Mine")).unwrap();
        std::fs::write(clip.join("index.json"), "{\"items\":[]}").unwrap();
        std::fs::write(clip.join("Mine/star.svg"), "<svg/>").unwrap();
        a.clip = crate::clipart_ui::ClipState::new(lc_core::clipart::Library::open(clip));
        let mut dev = lc_core::Device::default();
        dev.name = "My laser".into();
        a.profiles = vec![dev];
        a.lang = crate::i18n::Lang::Pl;
        let zip_path = dir.join("b.zip");
        a.write_backup(&zip_path).unwrap();
        // A fresh program with nothing: no profiles, empty clipart folder.
        let mut b = App::build(&ctx, None, false);
        let fresh = dir.join("new/clipart");
        b.clip = crate::clipart_ui::ClipState::new(lc_core::clipart::Library::open(fresh.clone()));
        b.profiles.clear();
        assert!(b.restore_backup(&ctx, &zip_path).is_ok());
        assert_eq!(b.profiles.len(), 1);
        assert_eq!(b.profiles[0].name, "My laser");
        assert_eq!(b.lang, crate::i18n::Lang::Pl);
        assert!(fresh.join("Mine/star.svg").is_file(), "clipart files come back");
        crate::i18n::set_lang(crate::i18n::Lang::En);
        // Something that is not a backup is refused.
        let junk = dir.join("junk.zip");
        std::fs::write(&junk, "not a zip").unwrap();
        assert!(b.restore_backup(&ctx, &junk).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
