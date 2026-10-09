//! Global preferences: grids.
use crate::app::App;
use crate::i18n::tr;
use crate::theme;
use crate::units_ui::drag_len;
use eframe::egui::{self, Color32, RichText};
use serde::{Deserialize, Serialize};

/// Appearance of the work-area grids. The main grid's switch and spacing live in `App::show_grid` / `App::grid`
/// because snapping uses them too.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GridPrefs {
    pub main_color: [u8; 4],
    pub minor_on: bool,
    /// Spacing of the secondary grid in mm. Always smaller than the main spacing.
    pub minor_mm: f64,
    pub minor_color: [u8; 4],
    /// Colour of the work-area sheet; `None` picks one that suits the colour scheme.
    #[serde(default)]
    pub bed_color: Option<[u8; 4]>,
    /// Thickness of the outlines drawn in the editor, in points. Previews use a little less.
    #[serde(default = "default_line_width")]
    pub line_width: f32,
}

fn default_line_width() -> f32 {
    1.0
}

impl Default for GridPrefs {
    fn default() -> Self {
        GridPrefs { main_color: [0xc4, 0xc4, 0xcc, 0xff], minor_on: false, minor_mm: 2.0, minor_color: [0xe6, 0xe6, 0xea, 0xff], bed_color: None, line_width: 1.0 }
    }
}

fn c32(c: [u8; 4]) -> Color32 {
    Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
}

impl App {
    /// Keep the secondary grid finer than the main grid.
    pub fn clamp_grid(&mut self) {
        self.grid = self.grid.clamp(0.1, 1000.0);
        let max = (self.grid * 0.9).max(0.05);
        self.grid_prefs.minor_mm = self.grid_prefs.minor_mm.clamp(0.05, max);
    }

    /// Fill colour of the work area: your choice, or very light grey in the dark schemes and white in the light ones.
    pub fn bed_fill(&self) -> Color32 {
        match self.grid_prefs.bed_color {
            Some(c) => c32(c),
            None if self.scheme.is_dark() => Color32::from_rgb(0xea, 0xea, 0xed),
            None => Color32::WHITE,
        }
    }

    pub fn prefs_window(&mut self, ctx: &egui::Context) {
        if !self.show_prefs {
            return;
        }
        self.clamp_grid();
        let units = self.doc.device.units;
        let mut open = true;
        egui::Window::new(tr("View options")).open(&mut open).collapsible(false).resizable(false).show(ctx, |ui| {
            ui.label(RichText::new(tr("Main grid")).strong());
            egui::Grid::new("grid_main").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                ui.label(tr("Show"));
                ui.checkbox(&mut self.show_grid, "");
                ui.end_row();
                ui.label(tr("Distance between lines"));
                drag_len(ui, units, &mut self.grid, 0.5, Some((0.1, 1000.0)));
                ui.end_row();
                ui.label(tr("Colour"));
                let mut col = c32(self.grid_prefs.main_color);
                if egui::color_picker::color_edit_button_srgba(ui, &mut col, egui::color_picker::Alpha::OnlyBlend).changed() {
                    self.grid_prefs.main_color = col.to_srgba_unmultiplied();
                }
                ui.end_row();
            });
            ui.add_space(8.0);
            ui.label(RichText::new(tr("Secondary grid")).strong());
            let max = (self.grid * 0.9).max(0.05);
            egui::Grid::new("grid_minor").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                ui.label(tr("Show"));
                ui.checkbox(&mut self.grid_prefs.minor_on, "");
                ui.end_row();
                ui.label(tr("Distance between lines"));
                drag_len(ui, units, &mut self.grid_prefs.minor_mm, 0.1, Some((0.05, max)));
                ui.end_row();
                ui.label(tr("Colour"));
                let mut col = c32(self.grid_prefs.minor_color);
                if egui::color_picker::color_edit_button_srgba(ui, &mut col, egui::color_picker::Alpha::OnlyBlend).changed() {
                    self.grid_prefs.minor_color = col.to_srgba_unmultiplied();
                }
                ui.end_row();
            });
            ui.label(RichText::new(tr("The secondary grid must be finer than the main grid.")).color(theme::text_dim()));
            ui.add_space(8.0);
            ui.label(RichText::new(tr("Work area")).strong());
            egui::Grid::new("bed_bg").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                ui.label(tr("Background colour"));
                ui.horizontal(|ui| {
                    let mut auto = self.grid_prefs.bed_color.is_none();
                    let current = self.bed_fill().to_srgba_unmultiplied();
                    if ui.checkbox(&mut auto, tr("Automatic")).changed() {
                        self.grid_prefs.bed_color = if auto { None } else { Some(current) };
                    }
                    if let Some(c) = self.grid_prefs.bed_color {
                        let mut col = c32(c);
                        if egui::color_picker::color_edit_button_srgba(ui, &mut col, egui::color_picker::Alpha::Opaque).changed() {
                            self.grid_prefs.bed_color = Some(col.to_srgba_unmultiplied());
                        }
                    }
                });
                ui.end_row();
                ui.label(tr("Line thickness"));
                ui.add(egui::Slider::new(&mut self.grid_prefs.line_width, 0.5..=3.0).step_by(0.1));
                ui.end_row();
            });
            if ui.button(tr("Reset grid")).clicked() {
                self.grid_prefs = GridPrefs::default();
                self.grid = 10.0;
                self.show_grid = true;
            }
        });
        self.show_prefs = open;
    }
}
