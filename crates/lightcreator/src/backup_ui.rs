//! Help > Download backup…: one zip file with everything the program keeps on this computer, the settings
//! (device profiles, language and so on) and the shared clipart gallery. The zip is made by `base_core::backup`.
use crate::app::{App, APP_VERSION};
use crate::i18n::trf;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub use lc_core::backup::ymd;

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
                text: crate::i18n::tr("Your current settings will be replaced by the backup. Its pictures are added to the shared gallery. Continue?").to_string(),
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

    /// Put the settings of a backup in place and add its pictures to the shared gallery. Returns the number of
    /// pictures added plus the settings file.
    pub fn restore_backup(&mut self, ctx: &eframe::egui::Context, src: &Path) -> Result<usize, String> {
        let r = lc_core::backup::restore(src, "LightCreator", self.clip.lib.dir())?;
        self.apply_settings(ctx, &r.settings);
        self.clip.lib.refresh();
        self.clip.reload();
        Ok(r.pictures + 1)
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
        if let Some(v) = get("api_on") {
            self.api_on = v == "1";
        }
        if let Some(v) = get("api_port").and_then(|v| v.parse::<u16>().ok()).filter(|p| *p >= 1024) {
            self.api_port = v;
        }
        if let Some(v) = get("api_lan") {
            self.api_lan = v == "1";
        }
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
            self.ed.grid_prefs = g;
        }
        if let Some(v) = get("grid_main_on") {
            self.ed.show_grid = v != "0";
        }
        if let Some(v) = get("snap_objects") {
            self.ed.snap_objects = v == "1";
        }
        if let Some(v) = get("zones_visible") {
            self.zones_visible = v != "0";
        }
        if let Some(v) = get("grid_main_mm").and_then(|v| v.parse::<f64>().ok()) {
            self.ed.grid = v;
        }
        if let Some(o) = get("tab_order") {
            self.tab_order = crate::app::SideTab::arranged(o);
        }
        self.cfg = None;
        if self.screen == crate::app::Screen::Editor && !self.profiles.is_empty() {
            self.enter_editor(self.active);
        }
    }

    /// Write the zip: the settings as they are now, the whole shared gallery and a short README.
    /// Returns the number of files in it.
    pub fn write_backup(&self, dest: &Path) -> Result<usize, String> {
        lc_core::backup::write(dest, "LightCreator", APP_VERSION, &self.settings_pairs(), self.clip.lib.dir())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    const SVG: &str = "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 10 10'><rect width='10' height='10'/></svg>";

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
        let mut lib = lc_core::clipart::Library::open(clip);
        lib.add(SVG, "star", "Mine", "file", "", "").unwrap();
        a.clip = crate::clipart_ui::new_state(lib);
        a.profiles = vec![lc_core::Device::default()];
        let zip_path = dir.join("backup.zip");
        let n = a.write_backup(&zip_path).expect("backup written");
        assert_eq!(n, 4);
        let mut z = zip::ZipArchive::new(std::fs::File::open(&zip_path).unwrap()).unwrap();
        let names: Vec<String> = (0..z.len()).map(|i| z.by_index(i).unwrap().name().unwrap_or_default().to_string()).collect();
        for want in ["app.ron", "clipart/index.json", "clipart/star.svg", "README.txt"] {
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
        let mut lib = lc_core::clipart::Library::open(clip);
        lib.add(SVG, "star", "Mine", "file", "", "").unwrap();
        a.clip = crate::clipart_ui::new_state(lib);
        let mut dev = lc_core::Device::default();
        dev.name = "My laser".into();
        a.profiles = vec![dev];
        a.lang = crate::i18n::Lang::Pl;
        let zip_path = dir.join("b.zip");
        a.write_backup(&zip_path).unwrap();
        // A fresh program with nothing: no profiles, empty clipart folder.
        let mut b = App::build(&ctx, None, false);
        let fresh = dir.join("new/clipart");
        b.clip = crate::clipart_ui::new_state(lc_core::clipart::Library::open(fresh.clone()));
        b.profiles.clear();
        assert!(b.restore_backup(&ctx, &zip_path).is_ok());
        assert_eq!(b.profiles.len(), 1);
        assert_eq!(b.profiles[0].name, "My laser");
        assert_eq!(b.lang, crate::i18n::Lang::Pl);
        assert_eq!(b.clip.lib.items.len(), 1, "the picture comes back");
        assert_eq!(b.clip.lib.items[0].name, "star");
        // A picture another program saved in the meantime stays, and restoring twice does not double anything.
        b.clip.lib.add(&SVG.replace("<rect", "<circle r='4'"), "newer", "Mine", "file", "", "").unwrap();
        assert!(b.restore_backup(&ctx, &zip_path).is_ok());
        assert_eq!(b.clip.lib.items.len(), 2, "{:?}", b.clip.lib.items);
        crate::i18n::set_lang(crate::i18n::Lang::En);
        // Something that is not a backup is refused.
        let junk = dir.join("junk.zip");
        std::fs::write(&junk, "not a zip").unwrap();
        assert!(b.restore_backup(&ctx, &junk).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
