//! The G-code viewer itself lives in baseCreator (`base_ui::gcode_view_ui`); this is the glue to the application: the menu
//! command, a dropped file, and the view in the place of the work area.
use crate::app::App;
use std::path::PathBuf;

pub use base_ui::gcode_view_ui::is_gcode;

impl App {
    pub fn view_gcode_dialog(&mut self) {
        if let Some(p) = base_ui::gcode_view_ui::pick_file() {
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
