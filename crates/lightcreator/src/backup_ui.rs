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
            // The tests run side by side and look for the texts of their windows in English.
            if !cfg!(test) {
                crate::i18n::set_lang(l);
            }
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
        // A backup from before the presets were in it leaves them alone.
        if let Some(text) = get(base_ui::layer_presets_ui::BACKUP_KEY) {
            let valid = &crate::layer_presets::valid;
            self.presets.get_or_insert_with(|| base_ui::layer_presets_ui::Library::load(crate::layer_presets::FILE, valid)).replace_text(text, valid);
        }
        // A backup from before the devices were in it leaves them alone.
        if let Some(text) = get(lc_core::control_devices::BACKUP_KEY) {
            self.control_devices.restore(text);
        }
        self.tabs.restore(get("tab_order").unwrap_or(""), get("side_collapsed").is_some_and(|v| v == "1"));
        self.cfg = None;
        if self.screen == crate::app::Screen::Editor && !self.profiles.is_empty() {
            self.enter_editor(self.active);
        }
    }

    /// Write the zip: the settings as they are now, the whole shared gallery and a short README.
    /// Returns the number of files in it.
    pub fn write_backup(&self, dest: &Path) -> Result<usize, String> {
        lc_core::backup::write(dest, "LightCreator", APP_VERSION, &self.backup_pairs(), self.clip.lib.dir())
    }

    /// What goes into the settings of a backup: the settings of the program and the layer presets, which have a file of their own.
    fn backup_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = self.settings_pairs();
        pairs.push(base_ui::layer_presets_ui::backup_pair(self.presets.as_ref(), crate::layer_presets::FILE, &crate::layer_presets::valid));
        pairs.push(self.control_devices.backup_pair());
        pairs
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

    #[test]
    fn the_layer_presets_are_in_the_backup_and_an_older_backup_leaves_them_alone() {
        let ctx = eframe::egui::Context::default();
        let dir = std::env::temp_dir().join(format!("lc-backup-presets-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut a = App::build(&ctx, None, false);
        let mut lib = base_ui::layer_presets_ui::Library::from_path(None, &crate::layer_presets::valid);
        lib.put("birch 3 mm", lc_core::Layer::new(0));
        a.presets = Some(lib);
        let gallery = dir.join("gallery");
        let zip = dir.join("b.zip");
        lc_core::backup::write(&zip, "LightCreator", "test", &a.backup_pairs(), &gallery).unwrap();
        assert!(!a.settings_pairs().iter().any(|(k, _)| *k == base_ui::layer_presets_ui::BACKUP_KEY), "the presets stay out of the saved settings");
        // A program with a preset of its own: the list of the backup takes its place, as the other settings do.
        let mut b = App::build(&ctx, None, false);
        b.presets.as_mut().unwrap().put("mine", lc_core::Layer::new(0));
        let r = lc_core::backup::restore(&zip, "LightCreator", &dir.join("gallery2")).unwrap();
        b.apply_settings(&ctx, &r.settings);
        let names: Vec<_> = b.presets.as_ref().unwrap().items.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["birch 3 mm"]);
        // Without the entry nothing happens to them.
        let mut older = r.settings.clone();
        older.remove(base_ui::layer_presets_ui::BACKUP_KEY);
        b.apply_settings(&ctx, &older);
        assert_eq!(b.presets.as_ref().unwrap().items.len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_control_devices_are_in_the_backup_and_an_older_backup_leaves_them_alone() {
        use lc_core::control_devices::{ControlDevice, KEYBOARD_ID};
        let ctx = eframe::egui::Context::default();
        let dir = std::env::temp_dir().join(format!("lc-backup-devices-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut a = App::build(&ctx, None, false);
        a.control_devices.lib.add(ControlDevice::keyboard("Keyboard"));
        let zip = dir.join("b.zip");
        lc_core::backup::write(&zip, "LightCreator", "test", &a.backup_pairs(), &dir.join("gallery")).unwrap();
        assert!(!a.settings_pairs().iter().any(|(k, _)| *k == lc_core::control_devices::BACKUP_KEY), "they have a file of their own");
        let mut b = App::build(&ctx, None, false);
        let r = lc_core::backup::restore(&zip, "LightCreator", &dir.join("gallery2")).unwrap();
        b.apply_settings(&ctx, &r.settings);
        assert_eq!(b.control_devices.lib.default.as_deref(), Some(KEYBOARD_ID));
        let mut older = r.settings.clone();
        older.remove(lc_core::control_devices::BACKUP_KEY);
        b.control_devices.lib.remove(KEYBOARD_ID);
        b.control_devices.lib.add(ControlDevice::pad("abc", "Pad"));
        b.apply_settings(&ctx, &older);
        assert_eq!(b.control_devices.lib.default.as_deref(), Some("pad-abc"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_order_of_the_tabs_and_the_collapsed_panel_are_in_the_backup() {
        let ctx = eframe::egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        let dir = std::env::temp_dir().join(format!("lc-backup-tabs-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        a.clip = crate::clipart_ui::new_state(lc_core::clipart::Library::open(dir.join("clipart")));
        a.profiles = vec![lc_core::Device::default()];
        let mut order = a.tabs.order().to_vec();
        order.swap(0, 3);
        a.tabs.restore(&order.join(","), true);
        let zip_path = dir.join("backup.zip");
        a.write_backup(&zip_path).unwrap();
        let mut b = App::build(&ctx, None, false);
        b.clip = crate::clipart_ui::new_state(lc_core::clipart::Library::open(dir.join("clipart2")));
        assert!(b.restore_backup(&ctx, &zip_path).is_ok());
        assert_eq!(b.tabs.order(), a.tabs.order());
        assert!(b.tabs.collapsed);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
