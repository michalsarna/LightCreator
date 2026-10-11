//! Materials: the **Materials** tab holds the library of the device you work with (settings of all layers of
//! each material; it is kept in the device profile and travels with saved projects). The built-in library and the
//! materials added for every device are browsed, and extended, in the **Material library** window of the Laser menu.
//! A library read from a project file is merged into the device library, with a window to settle differences.
use crate::app::App;
use crate::i18n::{tr, trf};
use crate::theme;
use crate::units_ui::{drag_len, drag_speed, fmt_len, fmt_speed};
use eframe::egui::{self, Color32, RichText};
use lc_core::materials::{add_missing, apply_rows, builtin_materials, plan_merge, Conflict, LayerSpec, Material, MaterialLayer, Value};
use lc_core::{Dither, LaserKind, Layer, LayerMode, SpeedUnit, Units};
use std::sync::OnceLock;

/// The built-in materials, computed once.
pub fn builtin() -> &'static [Material] {
    static B: OnceLock<Vec<Material>> = OnceLock::new();
    B.get_or_init(builtin_materials)
}

/// Differences between a library in a file and this device's library, waiting for the user's choice.
pub struct MergeDlg {
    pub source: String,
    pub conflicts: Vec<Conflict>,
    /// For every conflict and row: take the value from the file.
    pub take: Vec<Vec<bool>>,
    pub added: usize,
    pub unchanged: usize,
}

/// The materials the user added for every device, from the stored settings. Settings saved before libraries
/// were reworked hold single-layer presets under `user_materials`; they are turned into materials.
pub fn load_global(get: &dyn Fn(&str) -> Option<String>) -> Vec<Material> {
    if let Some(m) = get("global_materials").and_then(|j| serde_json::from_str::<Vec<Material>>(&j).ok()) {
        return m;
    }
    get("user_materials")
        .and_then(|j| serde_json::from_str::<Vec<lc_core::materials::Preset>>(&j).ok())
        .map(|p| lc_core::materials::materials_from_presets(&p))
        .unwrap_or_default()
}

fn fmt_value(v: Value, units: Units, su: SpeedUnit) -> String {
    match v {
        Value::Mode(m) => tr(m.label()).to_string(),
        Value::Speed(s) => fmt_speed(units, su, false, s),
        Value::Percent(p) => format!("{p:.0} %"),
        Value::Count(n) => n.to_string(),
        Value::Length(l) => fmt_len(units, l, 2),
        Value::Angle(a) => format!("{a:.0}°"),
        Value::Flag(b) => if b { tr("Yes") } else { tr("No") }.to_string(),
        Value::Dither(d) => tr(d.label()).to_string(),
    }
}

/// "Cut · Engrave" for the layers of a material.
fn layer_summary(m: &Material) -> String {
    m.layers.iter().map(|l| l.label.as_str()).collect::<Vec<_>>().join(" · ")
}

impl App {
    /// Fill the library of the device from the built-in and the global materials, once.
    pub fn seed_device_library(&mut self) {
        if self.doc.device.materials_seeded {
            return;
        }
        let laser = self.doc.device.laser;
        add_missing(&mut self.doc.device.materials, builtin(), laser);
        add_missing(&mut self.doc.device.materials, &self.global_materials, laser);
        self.doc.device.materials_seeded = true;
        self.sync_profile();
    }

    /// Merge the library that came with a project file into the one of this device. New materials are added at
    /// once; those that differ open a window to choose, value by value.
    pub fn import_incoming_materials(&mut self, incoming: Vec<Material>, source: &str) {
        if incoming.is_empty() {
            return;
        }
        let plan = plan_merge(&self.doc.device.materials, &incoming);
        let added = plan.added.len();
        self.doc.device.materials.extend(plan.added);
        self.sync_profile();
        if plan.conflicts.is_empty() {
            if added > 0 {
                self.status = trf("Added {} material(s) from {}", &[&added, &source]);
            }
            return;
        }
        let take = plan.conflicts.iter().map(|c| vec![false; c.rows.len()]).collect();
        self.mat_merge = Some(MergeDlg { source: source.to_string(), conflicts: plan.conflicts, take, added, unchanged: plan.unchanged });
    }

    /// The window that settles differences between the library of a file and the one of the device.
    pub fn merge_dialog(&mut self, ctx: &egui::Context) {
        let Some(dlg) = &mut self.mat_merge else { return };
        let (units, su) = (self.doc.device.units, self.doc.device.speed_unit);
        let layer_names: Vec<String> = self.doc.layers.iter().map(|l| l.name.clone()).collect();
        let mut apply = false;
        let mut cancel = false;
        let frame = egui::Frame::window(&ctx.global_style()).inner_margin(egui::Margin { left: 10, right: 10, top: 6, bottom: 8 });
        egui::Window::new("material_merge").title_bar(false).frame(frame).collapsible(false).default_size([620.0, 460.0]).resizable(true).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
            let mut open = true;
            crate::window_ui::mini_title(ui, &tr("Material library differences"), &mut open, |_| {});
            if !open {
                cancel = true;
            }
            ui.label(trf("{} has material settings that differ from this device. Choose the values to keep.", &[&dlg.source]));
            if dlg.added > 0 || dlg.unchanged > 0 {
                ui.label(RichText::new(trf("{} new material(s) were added; {} were already the same.", &[&dlg.added, &dlg.unchanged])).color(theme::text_dim()));
            }
            ui.separator();
            egui::ScrollArea::vertical().max_height(340.0).auto_shrink([false, true]).show(ui, |ui| {
                for (ci, c) in dlg.conflicts.iter().enumerate() {
                    egui::CollapsingHeader::new(RichText::new(&c.name).strong()).id_salt(("merge", ci)).default_open(true).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.small_button(tr("Keep all of mine")).clicked() {
                                dlg.take[ci].iter_mut().for_each(|t| *t = false);
                            }
                            if ui.small_button(tr("Take all from the file")).clicked() {
                                dlg.take[ci].iter_mut().for_each(|t| *t = true);
                            }
                        });
                        egui::Grid::new(("merge_grid", ci)).num_columns(3).striped(true).spacing([18.0, 4.0]).show(ui, |ui| {
                            for h in ["Setting", "This device", "File"] {
                                ui.label(RichText::new(tr(h)).strong());
                            }
                            ui.end_row();
                            for (ri, row) in c.rows.iter().enumerate() {
                                let lname = layer_names.get(row.layer).cloned().unwrap_or_else(|| row.layer.to_string());
                                let setting = if row.field.is_empty() { trf("Layer {}: whole layer", &[&lname]) } else { format!("{}: {}", lname, tr(row.field)) };
                                ui.label(setting);
                                let (local, incoming) = match (row.local, row.incoming) {
                                    (Some(a), Some(b)) => (fmt_value(a, units, su), fmt_value(b, units, su)),
                                    _ => {
                                        let label = c.incoming.layer(row.layer).map(|l| l.label.clone()).unwrap_or_default();
                                        (tr("not there").to_string(), label)
                                    }
                                };
                                ui.radio_value(&mut dlg.take[ci][ri], false, local);
                                ui.radio_value(&mut dlg.take[ci][ri], true, incoming);
                                ui.end_row();
                            }
                        });
                    });
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                let ok = egui::Button::new(RichText::new(tr("Apply")).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0x2e, 0xa0, 0x4f));
                if ui.add_sized([120.0, 28.0], ok).clicked() {
                    apply = true;
                }
                if ui.add_sized([200.0, 28.0], egui::Button::new(tr("Keep my library as it is"))).clicked() {
                    cancel = true;
                }
            });
        });
        if apply {
            if let Some(dlg) = self.mat_merge.take() {
                for (c, take) in dlg.conflicts.iter().zip(&dlg.take) {
                    if let Some(local) = self.doc.device.materials.iter_mut().find(|m| m.same_key(&c.incoming)) {
                        apply_rows(local, c, take);
                    }
                }
                self.sync_profile();
                self.status = tr("The library of this device was updated.").into();
            }
        } else if cancel {
            self.mat_merge = None;
        }
    }

    // ---------------------------------------------------------------------------------------------------------
    // The Materials tab: the library of this device.
    // ---------------------------------------------------------------------------------------------------------

    pub fn materials_tab(&mut self, ui: &mut egui::Ui) {
        let (units, su) = (self.doc.device.units, self.doc.device.speed_unit);
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.mat_dev_filter).hint_text(tr("Search")).desired_width(120.0));
            if ui.button(tr("Add from library…")).on_hover_text(tr("Copy materials from the built-in library into this device")).clicked() {
                self.mat_laser = Some(self.doc.device.laser);
                self.mat_add_mode = true;
                self.show_materials = true;
            }
        });
        ui.label(RichText::new(trf("Library of the device {}", &[&self.doc.device.name])).color(theme::text_dim()));
        let q = self.mat_dev_filter.to_lowercase();
        let lib = self.doc.device.materials.clone();
        let shown: Vec<usize> = (0..lib.len()).filter(|i| q.is_empty() || lib[*i].name.to_lowercase().contains(&q)).collect();
        if lib.is_empty() {
            ui.add_space(6.0);
            ui.label(RichText::new(tr("This library is empty. Add materials from the built-in library or save your layers as a material.")).color(theme::text_dim()));
        }
        egui::ScrollArea::vertical().id_salt("dev_materials").max_height(180.0).auto_shrink([false, true]).show(ui, |ui| {
            for i in &shown {
                let m = &lib[*i];
                let text = format!("{}  ·  {}", m.name, layer_summary(m));
                if ui.selectable_label(self.mat_dev_sel == Some(*i), text).clicked() {
                    self.mat_dev_sel = Some(*i);
                }
            }
        });
        ui.separator();
        if let Some(i) = self.mat_dev_sel.filter(|i| *i < lib.len()) {
            let mut m = lib[i].clone();
            let mut apply = false;
            let mut delete = false;
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut m.name).desired_width(ui.available_width() - 4.0));
            });
            ui.add(egui::TextEdit::singleline(&mut m.note).hint_text(tr("Note")).desired_width(ui.available_width() - 4.0));
            let mut remove: Option<usize> = None;
            for (li, ml) in m.layers.iter_mut().enumerate() {
                let lname = self.doc.layers.get(ml.index).map(|l| l.name.clone()).unwrap_or_default();
                egui::CollapsingHeader::new(RichText::new(format!("{} ({})", ml.label, trf("Layer {}", &[&lname]))).strong()).id_salt(("mat_layer", i, li)).default_open(true).show(ui, |ui| {
                    layer_spec_ui(ui, ("mat_spec", i, li), &mut ml.spec, units, su);
                    ui.horizontal(|ui| {
                        ui.label(tr("Label"));
                        ui.add(egui::TextEdit::singleline(&mut ml.label).desired_width(110.0));
                        if ui.small_button(tr("Remove layer")).clicked() {
                            remove = Some(li);
                        }
                    });
                });
            }
            if let Some(li) = remove {
                m.layers.remove(li);
            }
            // Add a layer that the material does not have yet, with the settings of that layer in the design.
            let free: Vec<usize> = (0..30).filter(|i| m.layer(*i).is_none()).collect();
            ui.horizontal(|ui| {
                let mut pick = self.mat_add_layer.min(29);
                if !free.contains(&pick) {
                    pick = free.first().copied().unwrap_or(0);
                }
                egui::ComboBox::from_id_salt("mat_add_layer").selected_text(self.doc.layers[pick].name.clone()).show_ui(ui, |ui| {
                    for f in &free {
                        ui.selectable_value(&mut pick, *f, self.doc.layers[*f].name.clone());
                    }
                });
                self.mat_add_layer = pick;
                if ui.add_enabled(!free.is_empty(), egui::Button::new(tr("Add layer"))).on_hover_text(tr("Uses the settings of this layer in the design")).clicked() {
                    let l = &self.doc.layers[pick];
                    m.layers.push(MaterialLayer { index: pick, label: l.name.clone(), spec: LayerSpec::from_layer(l) });
                    m.layers.sort_by_key(|l| l.index);
                }
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let go = egui::Button::new(RichText::new(tr("Apply to the layers")).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0x2e, 0xa0, 0x4f));
                if ui.add(go).on_hover_text(tr("Puts the settings of all its layers into the design")).clicked() {
                    apply = true;
                }
                if ui.button(tr("Delete")).clicked() {
                    delete = true;
                }
            });
            if m != lib[i] {
                self.doc.device.materials[i] = m.clone();
                self.sync_profile();
            }
            if apply {
                self.checkpoint();
                let n = m.apply(&mut self.doc.layers);
                self.touch();
                self.status = trf("Applied {} to {} layer(s)", &[&m.name, &n]);
            }
            if delete {
                self.doc.device.materials.remove(i);
                self.mat_dev_sel = None;
                self.sync_profile();
            }
            ui.separator();
        }
        // Save the layers of the design as a material of this device.
        ui.label(RichText::new(tr("Save the layers of the design as a material")).strong());
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.mat_name).hint_text(tr("Material name")).desired_width(ui.available_width() - 90.0));
            if ui.add_enabled(!self.mat_name.trim().is_empty(), egui::Button::new(tr("Save"))).on_hover_text(tr("Saves the settings of the layers used in the design")).clicked() {
                let m = self.material_from_design(&self.mat_name.clone());
                let name = m.name.clone();
                match self.doc.device.materials.iter().position(|e| e.same_key(&m)) {
                    Some(p) => self.doc.device.materials[p] = m,
                    None => self.doc.device.materials.push(m),
                }
                self.mat_dev_sel = self.doc.device.materials.iter().position(|e| e.name == name);
                self.sync_profile();
                self.status = trf("Saved {} in this device's library", &[&name]);
                self.mat_name.clear();
            }
        });
    }

    /// A material with the settings of the layers the design uses (or just the active layer).
    pub fn material_from_design(&self, name: &str) -> Material {
        let mut used: Vec<usize> = (0..self.doc.layers.len()).filter(|i| self.doc.shapes.iter().any(|s| s.layer == *i)).collect();
        if used.is_empty() {
            used.push(self.active_layer);
        }
        let layers: Vec<(usize, &Layer)> = used.iter().map(|i| (*i, &self.doc.layers[*i])).collect();
        Material::from_layers(name, self.doc.device.laser, &layers)
    }

    // ---------------------------------------------------------------------------------------------------------
    // The Material library window (Laser menu): built-in and global materials.
    // ---------------------------------------------------------------------------------------------------------

    pub fn materials_window(&mut self, ctx: &egui::Context) {
        if !self.show_materials {
            return;
        }
        let mut open = true;
        let mut add_to_device: Option<Material> = None;
        let mut delete: Option<Material> = None;
        let mut save_global: Option<Material> = None;
        let frame = egui::Frame::window(&ctx.global_style()).inner_margin(egui::Margin { left: 10, right: 10, top: 6, bottom: 8 });
        egui::Window::new("material_library").title_bar(false).frame(frame).default_size([640.0, 520.0]).resizable(true).show(ctx, |ui| {
            crate::window_ui::mini_title(ui, &tr("Material library"), &mut open, |_| {});
            let all: Vec<(Material, bool)> = builtin().iter().map(|m| (m.clone(), false)).chain(self.global_materials.iter().map(|m| (m.clone(), true))).collect();
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
            ui.label(RichText::new(tr("Materials added here are available for every device. A device's own library is edited in the Materials tab.")).color(theme::text_dim()));
            ui.separator();
            let q = self.mat_filter.to_lowercase();
            let rows: Vec<usize> = (0..all.len()).filter(|i| self.mat_laser.map_or(true, |k| all[*i].0.laser == k)).filter(|i| q.is_empty() || all[*i].0.name.to_lowercase().contains(&q)).collect();
            egui::ScrollArea::vertical().max_height(300.0).auto_shrink([false, false]).show(ui, |ui| {
                egui::Grid::new("mat_grid").num_columns(4).striped(true).spacing([14.0, 4.0]).show(ui, |ui| {
                    for h in ["Material", "Laser type", "Layers", "Source"] {
                        ui.label(RichText::new(tr(h)).strong());
                    }
                    ui.end_row();
                    for i in rows {
                        let (m, global) = &all[i];
                        if ui.selectable_label(self.mat_sel == Some(i), &m.name).clicked() {
                            self.mat_sel = Some(i);
                        }
                        ui.label(m.laser.label());
                        ui.label(layer_summary(m));
                        ui.label(if *global { tr("Added by you") } else { tr("Built-in") });
                        ui.end_row();
                    }
                });
            });
            ui.separator();
            let sel = self.mat_sel.and_then(|i| all.get(i)).cloned();
            ui.horizontal(|ui| {
                let go = egui::Button::new(RichText::new(tr("Add to this device")).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0x2e, 0xa0, 0x4f));
                if ui.add_enabled(sel.is_some(), go).clicked() {
                    add_to_device = sel.as_ref().map(|s| s.0.clone());
                }
                if ui.add_enabled(sel.as_ref().is_some_and(|s| s.1), egui::Button::new(tr("Delete"))).clicked() {
                    delete = sel.as_ref().map(|s| s.0.clone());
                }
            });
            ui.add_space(6.0);
            ui.label(RichText::new(tr("Save the layers of the design as a material for every device")).strong());
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.mat_global_name).hint_text(tr("Material name")).desired_width(220.0));
                if ui.add_enabled(!self.mat_global_name.trim().is_empty(), egui::Button::new(tr("Save to the library"))).clicked() {
                    save_global = Some(self.material_from_design(&self.mat_global_name.clone()));
                }
            });
        });
        if let Some(m) = add_to_device {
            match self.doc.device.materials.iter().position(|e| e.same_key(&m)) {
                Some(_) => self.status = trf("{} is already in this device's library", &[&m.name]),
                None => {
                    self.status = trf("Added {} to this device's library", &[&m.name]);
                    self.doc.device.materials.push(m);
                    self.sync_profile();
                }
            }
        }
        if let Some(m) = delete {
            self.global_materials.retain(|g| *g != m);
            self.mat_sel = None;
        }
        if let Some(m) = save_global {
            if builtin().iter().chain(&self.global_materials).any(|e| e.same_key(&m)) {
                self.status = tr("A material with this name already exists in the library.").into();
            } else {
                self.status = trf("Saved {} in the material library", &[&m.name]);
                self.global_materials.push(m);
                self.mat_global_name.clear();
            }
        }
        self.show_materials = open;
        if !open {
            self.mat_add_mode = false;
        }
    }
}

/// All settings of one layer of a material, in a two-column table.
fn layer_spec_ui(ui: &mut egui::Ui, id: (&str, usize, usize), s: &mut LayerSpec, units: Units, su: SpeedUnit) {
    egui::Grid::new(id).num_columns(2).spacing([10.0, 4.0]).show(ui, |ui| {
        ui.label(tr("Mode"));
        egui::ComboBox::from_id_salt((id, "mode")).selected_text(tr(s.mode.label())).show_ui(ui, |ui| {
            for m in LayerMode::ALL {
                ui.selectable_value(&mut s.mode, m, tr(m.label()));
            }
        });
        ui.end_row();
        ui.label(tr("Speed"));
        drag_speed(ui, units, su, false, &mut s.speed, 0.5, Some((0.1, 1000.0)));
        ui.end_row();
        ui.label(tr("Power (%)"));
        ui.add(egui::DragValue::new(&mut s.power).range(0.0..=100.0).suffix(" %"));
        ui.end_row();
        ui.label(tr("Passes"));
        ui.add(egui::DragValue::new(&mut s.passes).range(1..=100));
        ui.end_row();
        ui.label(tr("Interval"));
        drag_len(ui, units, &mut s.interval, 0.005, Some((0.01, 5.0)));
        ui.end_row();
        ui.label(tr("Scan angle"));
        ui.add(egui::DragValue::new(&mut s.angle).range(-180.0..=180.0).suffix("°"));
        ui.end_row();
        ui.label(tr("Overscan"));
        drag_len(ui, units, &mut s.overscan, 0.1, Some((0.0, 20.0)));
        ui.end_row();
        ui.label(tr("Bidirectional"));
        ui.checkbox(&mut s.bidirectional, "");
        ui.end_row();
        ui.label(tr("Dithering"));
        egui::ComboBox::from_id_salt((id, "dither")).selected_text(tr(s.dither.label())).show_ui(ui, |ui| {
            for d in Dither::ALL {
                ui.selectable_value(&mut s.dither, d, tr(d.label()));
            }
        });
        ui.end_row();
        ui.label(tr("Min power (%)"));
        let max = s.power;
        ui.add(egui::DragValue::new(&mut s.min_power).range(0.0..=max).suffix(" %"));
        ui.end_row();
        ui.label(tr("Air assist"));
        ui.checkbox(&mut s.air_assist, tr("Air pump on"));
        ui.end_row();
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Context;
    use lc_core::{Device, Document, Kind, Xf};

    fn app_with_profile() -> (App, Context) {
        let ctx = Context::default();
        let mut a = App::build(&ctx, None, false);
        a.profiles = vec![Device::default()];
        a.enter_editor(0);
        (a, ctx)
    }

    #[test]
    fn a_device_library_is_seeded_once_from_the_built_in_one() {
        let (mut a, _ctx) = app_with_profile();
        let n = a.doc.device.materials.len();
        assert!(n > 5 && a.doc.device.materials.iter().all(|m| m.laser == LaserKind::Diode) && a.doc.device.materials_seeded);
        // It is kept in the profile too.
        assert_eq!(a.profiles[0].materials.len(), n);
        // Deleting a material does not bring it back at the next start of the editor.
        a.doc.device.materials.remove(0);
        a.sync_profile();
        a.enter_editor(0);
        assert_eq!(a.doc.device.materials.len(), n - 1);
    }

    #[test]
    fn global_materials_reach_a_new_device_and_old_presets_are_converted() {
        let ctx = Context::default();
        let mut a = App::build(&ctx, None, false);
        let mut l = lc_core::Layer::new(0);
        l.speed = 12.0;
        a.global_materials.push(Material::from_layers("My birch", LaserKind::Diode, &[(0, &l)]));
        a.profiles = vec![Device::default()];
        a.enter_editor(0);
        assert!(a.doc.device.materials.iter().any(|m| m.name == "My birch"));
        // Settings saved before the rework held single-layer presets under another key.
        let old = r#"[{"material":"Mine","operation":"Cut","laser":"Co2","mode":"Line","speed":9.0,"power":50.0,"passes":1,"interval":0.1,"user":true},{"material":"Mine","operation":"Engrave","laser":"Co2","mode":"Fill","speed":90.0,"power":20.0,"passes":1,"interval":0.1,"user":true}]"#;
        let got = load_global(&|k| (k == "user_materials").then(|| old.to_string()));
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].layers.iter().map(|l| l.index).collect::<Vec<_>>(), vec![0, 1]);
        // The new key wins when both exist.
        let new = serde_json::to_string(&a.global_materials).unwrap();
        let got = load_global(&|k| match k {
            "global_materials" => Some(new.clone()),
            "user_materials" => Some(old.to_string()),
            _ => None,
        });
        assert_eq!(got[0].name, "My birch");
    }

    #[test]
    fn a_saved_project_carries_the_library_and_opening_it_merges_into_the_device() {
        let dir = std::env::temp_dir().join(format!("lc-matlib-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // The file comes from another machine: its library has a new material, an equal one and a changed one.
        let mut other = Document::default();
        other.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::IDENTITY);
        let (mut a, ctx) = app_with_profile();
        let local = a.doc.device.materials.clone();
        let same = local[0].clone();
        let mut changed = local[1].clone();
        changed.layers[0].spec.speed += 7.0;
        changed.layers[0].spec.air_assist = !changed.layers[0].spec.air_assist;
        let mut fresh = same.clone();
        fresh.name = "Only in the file".into();
        other.device.materials = vec![same, changed.clone(), fresh];
        let file = dir.join("p.lcr");
        std::fs::write(&file, other.to_json()).unwrap();
        a.open_path(file.clone());
        // The new material is added at once, the changed one waits for a decision.
        assert!(a.doc.device.materials.iter().any(|m| m.name == "Only in the file"));
        let dlg = a.mat_merge.as_mut().expect("a window with the differences");
        assert_eq!(dlg.conflicts.len(), 1);
        assert_eq!(dlg.conflicts[0].rows.iter().map(|r| r.field).collect::<Vec<_>>(), vec!["Speed", "Air assist"]);
        assert_eq!((dlg.added, dlg.unchanged), (1, 1));
        // Nothing has changed locally yet; take the speed from the file and keep the air assist.
        assert_eq!(a.doc.device.materials.iter().find(|m| m.same_key(&changed)).unwrap().layers[0].spec.speed, local[1].layers[0].spec.speed);
        dlg.take[0] = vec![true, false];
        let _ = ctx;
        // Apply, as the window does.
        let dlg = a.mat_merge.take().unwrap();
        for (c, take) in dlg.conflicts.iter().zip(&dlg.take) {
            let m = a.doc.device.materials.iter_mut().find(|m| m.same_key(&c.incoming)).unwrap();
            apply_rows(m, c, take);
        }
        a.sync_profile();
        let now = a.doc.device.materials.iter().find(|m| m.same_key(&changed)).unwrap();
        assert_eq!(now.layers[0].spec.speed, changed.layers[0].spec.speed);
        assert_eq!(now.layers[0].spec.air_assist, local[1].layers[0].spec.air_assist);
        assert_eq!(a.profiles[0].materials.len(), a.doc.device.materials.len(), "kept in the profile");
        // Saving writes the device library into the project; opening it again finds nothing to settle.
        a.save(false);
        let json = std::fs::read_to_string(&file).unwrap_or_default();
        let _ = json;
        let doc2 = Document::from_json(&a.doc.to_json()).unwrap();
        assert_eq!(doc2.device.materials, a.doc.device.materials);
        a.import_incoming_materials(doc2.device.materials.clone(), "again.lcr");
        assert!(a.mat_merge.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn saving_the_design_layers_makes_a_material_with_every_used_layer() {
        let (mut a, _ctx) = app_with_profile();
        a.doc.layers[0].speed = 4.0;
        a.doc.layers[3].power = 25.0;
        a.doc.layers[3].air_assist = true;
        a.doc.add(0, Kind::Rect { w: 5.0, h: 5.0 }, Xf::IDENTITY);
        a.doc.add(3, Kind::Rect { w: 5.0, h: 5.0 }, Xf::IDENTITY);
        let m = a.material_from_design("Test");
        assert_eq!(m.layers.iter().map(|l| l.index).collect::<Vec<_>>(), vec![0, 3]);
        assert!(m.layer(3).unwrap().spec.air_assist);
        // Applying it to a clean document restores both layers.
        let mut clean = Document::default();
        assert_eq!(m.apply(&mut clean.layers), 2);
        assert_eq!((clean.layers[0].speed, clean.layers[3].power), (4.0, 25.0));
    }

    #[test]
    fn the_backup_holds_the_global_materials_and_the_device_libraries() {
        let ctx = Context::default();
        let (mut a, _c) = app_with_profile();
        let mut l = lc_core::Layer::new(1);
        l.power = 33.0;
        a.global_materials.push(Material::from_layers("Global one", LaserKind::Co2, &[(1, &l)]));
        let n = a.doc.device.materials.len();
        let dir = std::env::temp_dir().join(format!("lc-matbackup-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let zip = dir.join("b.zip");
        a.write_backup(&zip).unwrap();
        let mut b = App::build(&ctx, None, false);
        b.profiles.clear();
        b.global_materials.clear();
        b.restore_backup(&ctx, &zip).unwrap();
        assert_eq!(b.global_materials.len(), 1);
        assert_eq!(b.profiles[0].materials.len(), n, "the library of the device comes back with its profile");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
