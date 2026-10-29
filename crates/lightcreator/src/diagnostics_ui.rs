//! Help > Export diagnostics (zip)…: one zip file to attach to a bug report. It holds the version and the system, the
//! settings (without the access key of the web server and the list of recent files), the device, the project and the
//! console of the machine, and nothing else.
use crate::app::{App, APP_VERSION};
use crate::i18n::trf;
use std::io::Write;
use std::path::Path;

/// Settings that are left out: a key and paths of files.
const LEFT_OUT: [&str; 2] = ["api_access", "recent_files"];

/// `user:password@` in an address is hidden: `http://user:secret@host/` → `http://***@host/`.
pub fn hide_passwords(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find("://") {
        out.push_str(&rest[..i + 3]);
        rest = &rest[i + 3..];
        let end = rest.find(|c: char| c == '/' || c == '"' || c == '\'' || c.is_whitespace()).unwrap_or(rest.len());
        match rest[..end].rfind('@') {
            Some(at) => {
                out.push_str("***");
                out.push_str(&rest[at..end]);
            }
            None => out.push_str(&rest[..end]),
        }
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

impl App {
    /// Ask where to put the file and write it.
    pub fn export_diagnostics(&mut self) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let name = format!("lightcreator-diagnostics-{}.zip", crate::backup_ui::ymd(now));
        let Some(dest) = rfd::FileDialog::new().add_filter("Zip", &["zip"]).set_file_name(&name).save_file() else { return };
        match self.write_diagnostics(&dest) {
            Ok(()) => {
                let file = dest.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                self.status = trf("Diagnostics saved: {}", &[&dest.display()]);
                self.toast(trf("Diagnostics saved: {}", &[&file]));
            }
            Err(e) => self.status = trf("Could not save the diagnostics: {}", &[&e]),
        }
    }

    /// The files of the zip, as (name, content).
    pub fn diagnostics_files(&self) -> Vec<(&'static str, String)> {
        let d = &self.doc.device;
        let mut info = String::new();
        let mut line = |k: &str, v: String| info.push_str(&format!("{k}: {v}\n"));
        line("Program", format!("LightCreator {APP_VERSION}"));
        line("System", format!("{} {}", std::env::consts::OS, std::env::consts::ARCH));
        line("Language", self.lang.code().to_owned());
        line("Theme", self.scheme.id().to_owned());
        line("Device", d.name.clone());
        line("Controller", format!("{:?}", d.controller));
        line("Work area", format!("{} x {} mm", d.work_size().0, d.work_size().1));
        line("Connected", self.connected.to_string());
        line("Machine", self.machine.0.to_string());
        line("Project", self.path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "(not saved)".into()));
        line("Layers", self.doc.layers.len().to_string());
        let settings: String = self.settings_pairs().into_iter().filter(|(k, _)| !LEFT_OUT.contains(k)).map(|(k, v)| format!("{k}={v}\n")).collect();
        let console: String = self
            .console
            .iter()
            .filter(|l| !l.poll)
            .map(|l| {
                let mark = match l.dir {
                    base_ui::link::Dir::Tx => ">",
                    base_ui::link::Dir::Rx => "<",
                    base_ui::link::Dir::Info => "#",
                };
                format!("{mark} {}\n", l.text)
            })
            .collect();
        vec![
            ("info.txt", info),
            ("device.json", hide_passwords(&serde_json::to_string_pretty(d).unwrap_or_default())),
            ("settings.txt", hide_passwords(&settings)),
            ("console.txt", console),
            ("project.cre", self.doc.to_json()),
        ]
    }

    pub fn write_diagnostics(&self, dest: &Path) -> Result<(), String> {
        let file = std::fs::File::create(dest).map_err(|e| e.to_string())?;
        let mut zip = zip::ZipWriter::new(file);
        let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        for (name, text) in self.diagnostics_files() {
            zip.start_file(name, opts).map_err(|e| e.to_string())?;
            zip.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
        }
        zip.finish().map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn passwords_in_addresses_are_hidden() {
        assert_eq!(hide_passwords("\"rtsp://user:secret@192.168.1.5/live\" and http://host:80/x"), "\"rtsp://***@192.168.1.5/live\" and http://host:80/x");
        assert_eq!(hide_passwords("no address"), "no address");
    }

    #[test]
    fn the_zip_holds_the_files_and_no_access_key() {
        let ctx = eframe::egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.api_access = "TOPSECRET".into();
        a.doc.device.cameras = vec![lc_core::CameraSource::url("A", "http://admin:hunter2@10.0.0.2/video", 0)];
        let dest = std::env::temp_dir().join(format!("lc-diag-{}.zip", std::process::id()));
        a.write_diagnostics(&dest).unwrap();
        let mut z = zip::ZipArchive::new(std::fs::File::open(&dest).unwrap()).unwrap();
        let mut all = String::new();
        let mut names = vec![];
        for i in 0..z.len() {
            let mut f = z.by_index(i).unwrap();
            names.push(f.name().unwrap_or_default().to_string());
            f.read_to_string(&mut all).unwrap();
        }
        let _ = std::fs::remove_file(&dest);
        assert_eq!(names, ["info.txt", "device.json", "settings.txt", "console.txt", "project.cre"]);
        assert!(all.contains("LightCreator v") && !all.contains("TOPSECRET") && !all.contains("hunter2"));
    }
}
