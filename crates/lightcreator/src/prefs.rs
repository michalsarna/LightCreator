//! Global preferences: grids.
use crate::app::App;
use crate::i18n::tr;
use crate::theme;
use crate::units_ui::drag_len;
use eframe::egui::{self, Color32, RichText};

pub use base_ui::editor::GridPrefs;

fn c32(c: [u8; 4]) -> Color32 {
    Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
}

impl App {
    /// Keep the secondary grid finer than the main grid.
    pub fn clamp_grid(&mut self) {
        self.ed.grid = self.ed.grid.clamp(0.1, 1000.0);
        let max = (self.ed.grid * 0.9).max(0.05);
        self.ed.grid_prefs.minor_mm = self.ed.grid_prefs.minor_mm.clamp(0.05, max);
    }

    /// Fill colour of the work area: your choice, or very light grey in the dark schemes and white in the light ones.
    pub fn bed_fill(&self) -> Color32 {
        match self.ed.grid_prefs.bed_color {
            Some(c) => c32(c),
            None if self.scheme.is_dark() => Color32::from_rgb(0xea, 0xea, 0xed),
            None => Color32::WHITE,
        }
    }

    /// Colour of the space around the work area: your choice, or the one of the colour scheme.
    pub fn workspace_fill(&self) -> Color32 {
        self.ed.grid_prefs.workspace_color.map(c32).unwrap_or_else(theme::workspace)
    }

    pub fn prefs_window(&mut self, ctx: &egui::Context) {
        if !self.show_prefs {
            return;
        }
        self.clamp_grid();
        let units = self.doc.device.units;
        let mut open = true;
        crate::window_ui::popup(ctx, "w_prefs_2", &tr("View options"), [340.0, 220.0], &mut open, |ui| {
            ui.label(RichText::new(tr("Main grid")).strong());
            egui::Grid::new("grid_main").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                ui.label(tr("Show"));
                ui.checkbox(&mut self.ed.show_grid, "");
                ui.end_row();
                ui.label(tr("Distance between lines"));
                drag_len(ui, units, &mut self.ed.grid, 0.5, Some((0.1, 1000.0)));
                ui.end_row();
                ui.label(tr("Colour"));
                let mut col = c32(self.ed.grid_prefs.main_color);
                if egui::color_picker::color_edit_button_srgba(ui, &mut col, egui::color_picker::Alpha::OnlyBlend).changed() {
                    self.ed.grid_prefs.main_color = col.to_srgba_unmultiplied();
                }
                ui.end_row();
            });
            ui.add_space(8.0);
            ui.label(RichText::new(tr("Secondary grid")).strong());
            let max = (self.ed.grid * 0.9).max(0.05);
            egui::Grid::new("grid_minor").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                ui.label(tr("Show"));
                ui.checkbox(&mut self.ed.grid_prefs.minor_on, "");
                ui.end_row();
                ui.label(tr("Distance between lines"));
                drag_len(ui, units, &mut self.ed.grid_prefs.minor_mm, 0.1, Some((0.05, max)));
                ui.end_row();
                ui.label(tr("Colour"));
                let mut col = c32(self.ed.grid_prefs.minor_color);
                if egui::color_picker::color_edit_button_srgba(ui, &mut col, egui::color_picker::Alpha::OnlyBlend).changed() {
                    self.ed.grid_prefs.minor_color = col.to_srgba_unmultiplied();
                }
                ui.end_row();
            });
            ui.label(RichText::new(tr("The secondary grid must be finer than the main grid.")).color(theme::text_dim()));
            ui.add_space(8.0);
            ui.label(RichText::new(tr("Work area")).strong());
            egui::Grid::new("bed_bg").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                ui.label(tr("Background colour"));
                ui.horizontal(|ui| {
                    let mut auto = self.ed.grid_prefs.bed_color.is_none();
                    let current = self.bed_fill().to_srgba_unmultiplied();
                    if ui.checkbox(&mut auto, tr("Automatic")).changed() {
                        self.ed.grid_prefs.bed_color = if auto { None } else { Some(current) };
                    }
                    if let Some(c) = self.ed.grid_prefs.bed_color {
                        let mut col = c32(c);
                        if egui::color_picker::color_edit_button_srgba(ui, &mut col, egui::color_picker::Alpha::Opaque).changed() {
                            self.ed.grid_prefs.bed_color = Some(col.to_srgba_unmultiplied());
                        }
                    }
                });
                ui.end_row();
                ui.label(tr("Workspace colour")).on_hover_text(tr("The space around the work area"));
                ui.horizontal(|ui| {
                    let mut auto = self.ed.grid_prefs.workspace_color.is_none();
                    let current = self.workspace_fill().to_srgba_unmultiplied();
                    if ui.checkbox(&mut auto, tr("Automatic")).changed() {
                        self.ed.grid_prefs.workspace_color = if auto { None } else { Some(current) };
                    }
                    if let Some(c) = self.ed.grid_prefs.workspace_color {
                        let mut col = c32(c);
                        if egui::color_picker::color_edit_button_srgba(ui, &mut col, egui::color_picker::Alpha::Opaque).changed() {
                            self.ed.grid_prefs.workspace_color = Some(col.to_srgba_unmultiplied());
                        }
                    }
                });
                ui.end_row();
                ui.label(tr("Line thickness"));
                ui.add(egui::Slider::new(&mut self.ed.grid_prefs.line_width, 0.5..=3.0).step_by(0.1));
                ui.end_row();
            });
            if ui.button(tr("Reset grid")).clicked() {
                self.ed.grid_prefs = GridPrefs::default();
                self.ed.grid = 10.0;
                self.ed.show_grid = true;
            }
        });
        self.show_prefs = open;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
