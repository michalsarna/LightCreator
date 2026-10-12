//! Global preferences: grids (the window is in base-ui).
use crate::app::App;
use eframe::egui;

pub use base_ui::editor::GridPrefs;

impl App {
    /// The window with the options of the grids and of the colours of the work area.
    pub fn prefs_window(&mut self, ctx: &egui::Context) {
        let mut show = self.show_prefs;
        base_ui::editor::view_options_window(self, ctx, &mut show);
        self.show_prefs = show;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base_ui::editor::EditorHost;
    use crate::theme;
    use eframe::egui::Color32;

    #[test]
    fn the_workspace_colour_is_the_schemes_until_chosen_and_old_settings_still_load() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        assert_eq!(a.ed.grid_prefs.workspace_color, None);
        assert_eq!(a.workspace_fill(), theme::workspace());
        a.ed.grid_prefs.workspace_color = Some([10, 20, 30, 255]);
        assert_eq!(a.workspace_fill(), Color32::from_rgb(10, 20, 30));
        // Settings written before this option existed.
        let mut v = serde_json::to_value(GridPrefs::default()).unwrap();
        v.as_object_mut().unwrap().remove("workspace_color");
        let old: GridPrefs = serde_json::from_value(v).unwrap();
        assert_eq!(old.workspace_color, None);
        // The choice is kept with the settings.
        let back: GridPrefs = serde_json::from_str(&serde_json::to_string(&a.ed.grid_prefs).unwrap()).unwrap();
        assert_eq!(back.workspace_color, Some([10, 20, 30, 255]));
    }
}
