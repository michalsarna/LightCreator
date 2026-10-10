//! Material library window: browse presets, apply one to the active layer, save your own.
use crate::app::App;
use crate::i18n::{tr, trf};
use crate::theme;
use crate::units_ui::fmt_speed;
use eframe::egui::{self, RichText};
use lc_core::materials::{builtin, Preset};
use lc_core::LaserKind;

impl App {
    fn all_presets(&self) -> Vec<Preset> {
        let mut v = builtin();
        v.extend(self.user_presets.iter().cloned());
        v
    }

    pub fn materials_window(&mut self, ctx: &egui::Context) {
        if !self.show_materials {
            return;
        }
        let mut open = true;
        let units = self.doc.device.units;
        let all = self.all_presets();
        let layer_name = self.doc.layers[self.active_layer].name.clone();
        let mut apply: Option<Preset> = None;
        let mut delete: Option<Preset> = None;
        let mut save_new: Option<Preset> = None;
        egui::Window::new(tr("Material library")).open(&mut open).default_size([600.0, 520.0]).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(tr("Laser type"));
                let label = |l: Option<LaserKind>| l.map(|k| k.label().to_string()).unwrap_or_else(|| tr("All").to_string());
                egui::ComboBox::from_id_salt("mat_laser").selected_text(label(self.mat_laser)).show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.mat_laser, None, tr("All"));
                    for k in LaserKind::ALL {
                        ui.selectable_value(&mut self.mat_laser, Some(k), k.label());
                    }
                });
                ui.add(egui::TextEdit::singleline(&mut self.mat_filter).hint_text(tr("Search")).desired_width(180.0));
            });
            ui.label(RichText::new(tr("Starting values for typical 10 W diode and 40 W CO2 lasers. Always test on scrap.")).color(theme::text_dim()));
            ui.separator();
            let q = self.mat_filter.to_lowercase();
            let rows: Vec<(usize, &Preset)> = all
                .iter()
                .enumerate()
                .filter(|(_, p)| self.mat_laser.map_or(true, |k| p.laser == k))
                .filter(|(_, p)| q.is_empty() || p.material.to_lowercase().contains(&q) || p.operation.to_lowercase().contains(&q))
                .collect();
            egui::ScrollArea::vertical().max_height(300.0).auto_shrink([false, false]).show(ui, |ui| {
                egui::Grid::new("mat_grid").num_columns(7).striped(true).spacing([14.0, 4.0]).show(ui, |ui| {
                    for h in ["Material", "Operation", "Laser type", "Mode", "Speed", "Power", "Passes"] {
                        ui.label(RichText::new(tr(h)).strong());
                    }
                    ui.end_row();
                    for (i, p) in rows {
                        ui.horizontal(|ui| {
                            // Your own presets carry a star.
                            if p.user {
                                if let Some(img) = crate::icons::image("star", 13.0, theme::accent()) {
                                    ui.add(img);
                                }
                            }
                            if ui.selectable_label(self.mat_sel == Some(i), &p.material).clicked() {
                                self.mat_sel = Some(i);
                            }
                        });
                        ui.label(tr_op(&p.operation));
                        ui.label(p.laser.label());
                        ui.label(tr(p.mode.label()));
                        ui.label(fmt_speed(units, self.doc.device.speed_unit, false, p.speed));
                        ui.label(format!("{:.0} %", p.power));
                        ui.label(p.passes.to_string());
                        ui.end_row();
                    }
                });
            });
            ui.separator();
            let sel = self.mat_sel.and_then(|i| all.get(i)).cloned();
            ui.horizontal(|ui| {
                if ui.add_enabled(sel.is_some(), egui::Button::new(trf("Apply to layer {}", &[&layer_name]))).clicked() {
                    apply = sel.clone();
                }
                if ui.add_enabled(sel.as_ref().is_some_and(|p| p.user), egui::Button::new(tr("Delete"))).clicked() {
                    delete = sel.clone();
                }
            });
            ui.add_space(6.0);
            ui.label(RichText::new(tr("Save the active layer as a preset")).strong());
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.mat_name).hint_text(tr("Material name")).desired_width(180.0));
                egui::ComboBox::from_id_salt("mat_op").selected_text(tr_op(&self.mat_op)).show_ui(ui, |ui| {
                    for op in ["Cut", "Engrave", "Score"] {
                        ui.selectable_value(&mut self.mat_op, op.to_string(), tr_op(op));
                    }
                });
                if ui.add_enabled(!self.mat_name.trim().is_empty(), egui::Button::new(tr("Save preset"))).clicked() {
                    save_new = Some(Preset::from_layer(self.mat_name.trim(), &self.mat_op, self.doc.device.laser, &self.doc.layers[self.active_layer]));
                }
            });
        });
        if let Some(p) = apply {
            self.checkpoint();
            let l = self.active_layer;
            p.apply(&mut self.doc.layers[l]);
            self.touch();
            self.status = trf("Applied {} ({}) to layer {}", &[&p.material, &tr_op(&p.operation), &layer_name]);
        }
        if let Some(p) = delete {
            self.user_presets.retain(|u| *u != p);
            self.mat_sel = None;
        }
        if let Some(p) = save_new {
            self.user_presets.push(p);
            self.mat_name.clear();
        }
        self.show_materials = open;
    }
}

fn tr_op(op: &str) -> &'static str {
    match op {
        "Cut" => tr("Cut"),
        "Engrave" => tr("Engrave"),
        "Score" => tr("Score"),
        _ => "",
    }
}
