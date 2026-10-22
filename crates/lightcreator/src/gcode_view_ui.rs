//! The G-code viewer itself lives in baseCreator (`base_ui::gcode_view_ui`); this is the glue to the application: the menu
//! command, a dropped file, and the view in the place of the work area.
use crate::app::App;
use std::path::PathBuf;

pub use base_ui::gcode_view_ui::is_gcode;

impl App {
    pub fn view_gcode_dialog(&mut self) {
        if let Some(p) = base_ui::gcode_view_ui::pick_file() {
            // From the start screen the viewer takes the whole window; closing it returns to the start screen.
            self.open_gcode(p);
        }
    }

    pub fn open_gcode(&mut self, p: PathBuf) -> bool {
        match base_ui::gcode_view_ui::open(&mut self.gcode_view, p) {
            Ok(()) => true,
            Err(e) => {
                self.status = e;
                false
            }
        }
    }

    /// The viewer in the place of the work area; false when there is none (the editor is drawn then).
    pub fn gcode_view_panel(&mut self, ui: &mut eframe::egui::Ui) -> bool {
        base_ui::gcode_view_ui::panel(&mut self.gcode_view, ui)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_the_start_screen_the_viewer_opens_without_entering_the_editor() {
        let ctx = eframe::egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        assert_eq!(a.screen, crate::app::Screen::Start);
        let dir = std::env::temp_dir().join(format!("lc-gview-start-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("job.nc");
        std::fs::write(&p, "G21 G90\nG0 X0 Y0\nG1 X10 Y0 F500\nG1 X10 Y10\n").unwrap();
        assert!(a.open_gcode(p));
        assert!(a.gcode_view.is_some());
        assert_eq!(a.screen, crate::app::Screen::Start);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[derive(Debug)]
    struct Dropped(PathBuf);
    impl eframe::egui::DroppedFile for Dropped {
        fn path(&self) -> &std::path::Path {
            &self.0
        }
        fn bytes(&self) -> Result<Vec<u8>, String> {
            std::fs::read(&self.0).map_err(|e| e.to_string())
        }
    }

    #[test]
    fn a_g_code_file_dropped_on_the_start_screen_opens_the_viewer() {
        let ctx = eframe::egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        let dir = std::env::temp_dir().join(format!("lc-gview-drop-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("job.nc");
        std::fs::write(&p, "G21 G90\nG0 X0 Y0\nG1 X10 Y0 F500\nG1 X10 Y10\n").unwrap();
        let mut raw = eframe::egui::RawInput::default();
        raw.dropped_files.push(std::sync::Arc::new(Dropped(p)));
        let mut out = ctx.run_ui(raw, |ui| a.handle_drops(ui.ctx()));
        out.textures_delta.clear();
        assert!(a.gcode_view.is_some());
        assert_eq!(a.screen, crate::app::Screen::Start);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
