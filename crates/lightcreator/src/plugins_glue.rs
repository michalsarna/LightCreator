//! Plugins in LightCreator: the windows, the importers, the generators and the postprocessor of the G-code export
//! (the machinery is in baseCreator: `base_core::plugins`, `base_ui::plugins_ui`).
use crate::app::{App, Screen};
use eframe::egui;
use lc_core::plugin_script::Shapes;
use base_ui::i18n::{tr, trf};
use base_ui::plugins_ui;
use std::path::{Path, PathBuf};

impl App {
    /// The windows of the plugins and the generators; what a generator makes is added to the drawing.
    pub fn plugin_windows(&mut self, ctx: &egui::Context) {
        if let Some(say) = plugins_ui::window(ctx, &mut self.plugins) {
            self.status = say;
            self.after_plugins_changed();
        }
        let area = self.doc.device.work_size();
        if let Some(s) = plugins_ui::generator_window(ctx, &mut self.plugins, area) {
            self.add_plugin_shapes(s, &self.plugins.last_generator_name());
        }
    }

    /// Pictures that plugins bring join the shared gallery (once).
    pub fn after_plugins_changed(&mut self) {
        self.plugins.sync_clipart(&crate::data_paths().gallery);
        self.clip.lib.refresh();
    }

    /// Put shapes made by a plugin into the drawing: selected, and one undo takes them back.
    fn add_plugin_shapes(&mut self, s: Shapes, what: &str) {
        self.checkpoint();
        let n = s.scene.shapes.len();
        let last = self.doc.layers.len().saturating_sub(1);
        self.ed.sel = self.doc.scene.append(s.scene, last);
        self.status = trf("Imported {} paths from {}", &[&n, &what]);
        self.touch();
        self.ed.view.need_fit = true;
        if !s.warnings.is_empty() {
            self.toast(s.warnings.join("; "));
        }
    }

    /// File > Import > Import with a plugin…
    pub fn import_with_plugin_dialog(&mut self) {
        let area = self.doc.device.work_size();
        match self.plugins.import_dialog(area) {
            Some((p, r)) => {
                self.finish_plugin_import(&p, r);
            }
            None => self.status = tr("No plugin reads files: install one in the Plugins window").to_string(),
        }
    }

    /// A file a plugin reads (by its extension): imported into the drawing. `None` when no plugin reads it.
    pub fn try_plugin_import(&mut self, path: &Path) -> Option<bool> {
        let area = self.doc.device.work_size();
        let r = self.plugins.import_file(path, area)?;
        Some(self.finish_plugin_import(path, r))
    }

    fn finish_plugin_import(&mut self, path: &Path, r: Result<Shapes, String>) -> bool {
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        match r {
            Ok(s) => {
                self.add_plugin_shapes(s, &name);
                true
            }
            Err(e) => {
                self.status = trf("Import failed: {}", &[&e]);
                false
            }
        }
    }

    /// Does a plugin import files with this extension?
    pub fn plugin_reads(&self, path: &Path) -> bool {
        let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
        self.plugins.import_extensions().iter().any(|e| e.eq_ignore_ascii_case(&ext))
    }

    /// The text of a G-code file after the postprocessor the person chose (as it is when none). `None` after an error.
    pub fn postprocessed(&mut self, text: String) -> Option<String> {
        let area = self.doc.device.work_size();
        match self.plugins.export_text(&text, area) {
            Ok((t, _said)) => Some(t),
            Err(e) => {
                self.status = trf("The postprocessor failed, nothing was saved: {}", &[&e]);
                None
            }
        }
    }

    /// Open a file that is for a plugin from the start screen: the editor opens with an empty drawing first.
    pub fn plugin_import_from_start(&mut self, p: PathBuf) {
        if self.screen == Screen::Start {
            self.start_project(|app| app.try_plugin_import(&p).unwrap_or(false));
        } else {
            self.try_plugin_import(&p);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str, post: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!("lc-plugins-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let p = base.join("plugins/my-tools");
        std::fs::create_dir_all(&p).unwrap();
        std::fs::write(p.join("plugin.json"), r#"{"id":"my-tools","name":"My tools","importers":[{"id":"xy","name":"XY","extensions":["xy"],"script":"read.rhai"}],"postprocessors":[{"id":"n","name":"Numbers","script":"post.rhai"}]}"#).unwrap();
        std::fs::write(p.join("read.rhai"), "fn read(text, ctx) { [rect(5, 5, 10, 20), circle(30, 30, 4)] }").unwrap();
        std::fs::write(p.join("post.rhai"), post).unwrap();
        base
    }

    #[test]
    fn a_plugin_reads_its_files_into_the_drawing_and_postprocesses_the_gcode() {
        let base = scratch("all", r#"fn post(lines, ctx) { let out = []; let n = 1; for l in lines { out.push("N" + n + " " + l); n += 1; } out }"#);
        let mut a = App::build(&egui::Context::default(), None, false);
        a.plugins = base_ui::plugins_ui::PluginsUi::new("lightcreator", &base);
        a.doc.add(0, lc_core::Kind::Rect { w: 5.0, h: 5.0 }, lc_core::Xf::IDENTITY);
        let file = base.join("a.xy");
        std::fs::write(&file, "x").unwrap();
        assert!(a.plugin_reads(&file));
        let recent = a.recent.clone();
        assert_eq!(a.try_plugin_import(&file), Some(true));
        assert_eq!(a.doc.scene.shapes.len(), 3);
        assert_eq!(a.ed.sel.len(), 2);
        assert_eq!(a.recent, recent);
        a.do_undo();
        assert_eq!(a.doc.scene.shapes.len(), 1);
        a.plugins.post = Some("my-tools/n".into());
        assert_eq!(a.postprocessed("G21\nM5\n".into()).unwrap(), "N1 G21\nN2 M5\n");
        std::fs::remove_dir_all(base).ok();
    }

    #[test]
    fn a_postprocessor_with_controller_commands_saves_nothing() {
        let base = scratch("bad", r#"fn post(lines, ctx) { ["$H"] }"#);
        let mut a = App::build(&egui::Context::default(), None, false);
        a.plugins = base_ui::plugins_ui::PluginsUi::new("lightcreator", &base);
        a.plugins.post = Some("my-tools/n".into());
        assert!(a.postprocessed("G21\n".into()).is_none());
        assert!(a.status.contains("nothing was saved"), "{}", a.status);
        std::fs::remove_dir_all(base).ok();
    }
}
