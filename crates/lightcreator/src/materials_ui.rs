//! Materials: the **Materials** tab holds the library of the device you work with (settings of all layers of
//! each material; it is kept in the device profile and travels with saved projects). The built-in library and the
//! materials added for every device are browsed, and extended, in the **Material library** window of the Laser menu.
//! A library read from a project file is merged into the device library, with a window to settle differences.
use crate::app::App;
use crate::i18n::{tr, trf};
use crate::theme;
use crate::units_ui::{drag_len, drag_speed, fmt_len, fmt_speed};
use eframe::egui::{self, Color32, RichText};
use lc_core::materials::{add_missing, apply_rows, builtin_materials, guess_category, plan_merge, Conflict, LayerSpec, LibraryFile, Material, MaterialLayer, Value, CATEGORIES, NO_COLOUR};
use lc_core::{Dither, LaserKind, Layer, LayerMode, SpeedUnit, Units, PALETTE};
use std::path::PathBuf;
use std::sync::OnceLock;

/// The built-in materials, computed once.
pub fn builtin() -> &'static [Material] {
    static B: OnceLock<Vec<Material>> = OnceLock::new();
    B.get_or_init(builtin_materials)
}

/// Which library a window of differences changes.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MergeTarget {
    /// The library of the device in use.
    Current,
    /// The materials added for every device.
    Global,
    /// The library of a device profile.
    Profile(usize),
}

/// What goes into a library file (export) or comes out of it (import): the choice is made in a window.
pub struct LibDlg {
    pub export: bool,
    pub file: LibraryFile,
    /// Name of the file an import comes from.
    pub source: String,
    pub global_on: bool,
    pub global: Vec<bool>,
    /// Per device: take it, and which of its materials.
    pub devices: Vec<bool>,
    pub mats: Vec<Vec<bool>>,
}

impl LibDlg {
    pub fn new(export: bool, file: LibraryFile, source: &str) -> LibDlg {
        LibDlg {
            export,
            source: source.to_string(),
            global_on: true,
            global: vec![true; file.global.len()],
            devices: vec![true; file.devices.len()],
            mats: file.devices.iter().map(|d| vec![true; d.materials.len()]).collect(),
            file,
        }
    }

    /// The part of the library that is ticked.
    pub fn selection(&self) -> LibraryFile {
        let global = self.file.global.iter().zip(&self.global).filter(|(_, on)| **on).map(|(m, _)| m.clone()).collect();
        let mut devices = vec![];
        for (i, d) in self.file.devices.iter().enumerate() {
            if self.devices[i] {
                let mut d = d.clone();
                d.materials = d.materials.iter().zip(&self.mats[i]).filter(|(_, on)| **on).map(|(m, _)| m.clone()).collect();
                devices.push(d);
            }
        }
        LibraryFile::new(global, devices)
    }

    fn nothing(&self) -> bool {
        let s = self.selection();
        s.global.is_empty() && s.devices.is_empty()
    }
}

/// Differences between a library in a file and this device's library, waiting for the user's choice.
pub struct MergeDlg {
    pub source: String,
    /// The library the chosen values go to.
    pub target: MergeTarget,
    pub target_name: String,
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
        Value::Colour(i) => if i < PALETTE.len() { format!("#{:02x}{:02x}{:02x}", PALETTE[i][0], PALETTE[i][1], PALETTE[i][2]) } else { "–".to_string() },
    }
}

/// The name of a category in the language of the interface (the names the program ships with are translated).
pub fn cat_label(name: &str) -> String {
    match name {
        "Wood" => tr("Wood"),
        "Metal" => tr("Metal"),
        "Plastic" => tr("Plastic"),
        "Paper and leather" => tr("Paper and leather"),
        "Stone and glass" => tr("Stone and glass"),
        "Other" => tr("Other"),
        other => return other.to_string(),
    }
    .to_string()
}

fn colour32(i: usize) -> Color32 {
    let c = PALETTE[i.min(PALETTE.len() - 1)];
    Color32::from_rgb(c[0], c[1], c[2])
}

/// The colour a layer of a material has (the colour of its layer when the material does not say).
fn layer_colour(ml: &MaterialLayer) -> Color32 {
    colour32(if ml.spec.colour == NO_COLOUR { ml.index } else { ml.spec.colour })
}

fn swatch(ui: &mut egui::Ui, c: Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(11.0, 11.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 2.0, c);
    ui.painter().rect_stroke(rect, 2.0, egui::Stroke::new(1.0, theme::border()), egui::StrokeKind::Inside);
}

/// A drop-down of the layer colours.
fn colour_picker(ui: &mut egui::Ui, id: impl std::hash::Hash + std::fmt::Debug, colour: &mut usize, fallback: usize) {
    let shown = if *colour == NO_COLOUR { fallback } else { *colour };
    egui::ComboBox::from_id_salt(id).width(46.0).selected_text(RichText::new("■■").color(colour32(shown))).show_ui(ui, |ui| {
        egui::Grid::new("colour_grid").spacing([3.0, 3.0]).show(ui, |ui| {
            for i in 0..PALETTE.len() {
                let (rect, r) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::click());
                ui.painter().rect_filled(rect, 2.0, colour32(i));
                if shown == i {
                    ui.painter().rect_stroke(rect, 2.0, egui::Stroke::new(2.0, theme::text()), egui::StrokeKind::Inside);
                }
                if r.clicked() {
                    *colour = i;
                }
                if i % 6 == 5 {
                    ui.end_row();
                }
            }
        });
    });
}

/// "Wood", "Metal", ... and the categories of the user, each once.
fn categories_of<'a>(libs: impl Iterator<Item = &'a Material>) -> Vec<String> {
    let mut v: Vec<String> = CATEGORIES.iter().map(|s| s.to_string()).collect();
    let mut extra: Vec<String> = vec![];
    for m in libs {
        let c = m.category();
        if !v.iter().chain(&extra).any(|x| x.eq_ignore_ascii_case(&c)) {
            extra.push(c);
        }
    }
    extra.sort_by_key(|c| c.to_lowercase());
    v.splice(v.len() - 1..v.len() - 1, extra);
    v
}

/// A filter drop-down for a list of materials: "All categories" and the categories that occur in it.
fn category_filter(ui: &mut egui::Ui, id: &str, current: &mut Option<String>, libs: &[&Material]) {
    let mut used: Vec<String> = categories_of(libs.iter().copied()).into_iter().filter(|c| libs.iter().any(|m| m.category().eq_ignore_ascii_case(c))).collect();
    used.dedup();
    if current.as_ref().is_some_and(|c| !used.contains(c)) {
        *current = None;
    }
    let text = current.as_deref().map(cat_label).unwrap_or_else(|| tr("All categories").to_string());
    egui::ComboBox::from_id_salt(id).width(130.0).selected_text(text).show_ui(ui, |ui| {
        ui.selectable_value(current, None, tr("All categories"));
        for c in used {
            let label = cat_label(&c);
            ui.selectable_value(current, Some(c), label);
        }
    });
}

/// One entry of a list of materials: name and category, then every layer with its colour, label and main settings.
fn material_card(ui: &mut egui::Ui, m: &Material, selected: bool, units: Units, su: SpeedUnit) -> egui::Response {
    let fill = if selected { theme::accent().gamma_multiply(0.18) } else { Color32::TRANSPARENT };
    let out = egui::Frame::new().fill(fill).corner_radius(3).inner_margin(egui::Margin::symmetric(4, 3)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(RichText::new(&m.name).strong());
            ui.label(RichText::new(cat_label(&m.category())).small().color(theme::text_dim()));
        });
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            for l in &m.layers {
                swatch(ui, layer_colour(l));
                ui.label(RichText::new(format!("{} · {} · {:.0} % · {}×", l.label, fmt_speed(units, su, false, l.spec.speed), l.spec.power, l.spec.passes)).small().color(theme::text_dim()));
                ui.add_space(6.0);
            }
        });
    });
    let r = ui.interact(out.response.rect, ui.id().with(("material_card", &m.name, m.laser as u8)), egui::Sense::click());
    r.on_hover_cursor(egui::CursorIcon::PointingHand)
}

impl App {
    /// The list of materials a target stands for.
    fn target_materials(&mut self, t: MergeTarget) -> Option<&mut Vec<Material>> {
        match t {
            MergeTarget::Current => Some(&mut self.doc.device.materials),
            MergeTarget::Global => Some(&mut self.global_materials),
            MergeTarget::Profile(i) if self.screen == crate::app::Screen::Editor && i == self.active => Some(&mut self.doc.device.materials),
            MergeTarget::Profile(i) => self.profiles.get_mut(i).map(|p| &mut p.materials),
        }
    }

    /// Show a window of differences now, or after the ones that are waiting.
    fn queue_merge(&mut self, d: MergeDlg) {
        if self.mat_merge.is_none() {
            self.mat_merge = Some(d);
        } else {
            self.mat_merge_next.push(d);
        }
    }

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
        let name = self.doc.device.name.clone();
        self.queue_merge(MergeDlg { source: source.to_string(), target: MergeTarget::Current, target_name: name, conflicts: plan.conflicts, take, added, unchanged: plan.unchanged });
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
            if dlg.target == MergeTarget::Current {
                ui.label(trf("{} has material settings that differ from this device. Choose the values to keep.", &[&dlg.source]));
            } else {
                ui.label(trf("{} has material settings that differ from {}. Choose the values to keep.", &[&dlg.source, &dlg.target_name]));
            }
            let mine = if dlg.target == MergeTarget::Global { tr("Material library") } else { tr("This device") };
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
                            for h in [tr("Setting"), mine, tr("File")] {
                                ui.label(RichText::new(h).strong());
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
                if let Some(lib) = self.target_materials(dlg.target) {
                    for (c, take) in dlg.conflicts.iter().zip(&dlg.take) {
                        if let Some(local) = lib.iter_mut().find(|m| m.same_key(&c.incoming)) {
                            apply_rows(local, c, take);
                        }
                    }
                }
                self.sync_profile();
                self.status = tr("The library of this device was updated.").into();
            }
        } else if cancel {
            self.mat_merge = None;
        }
        if self.mat_merge.is_none() && !self.mat_merge_next.is_empty() {
            self.mat_merge = Some(self.mat_merge_next.remove(0));
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
        ui.horizontal(|ui| {
            let refs: Vec<&Material> = lib.iter().collect();
            category_filter(ui, "mat_dev_cat", &mut self.mat_dev_cat, &refs);
        });
        let cat = self.mat_dev_cat.clone();
        let shown: Vec<usize> = (0..lib.len()).filter(|i| q.is_empty() || lib[*i].name.to_lowercase().contains(&q)).filter(|i| cat.as_ref().map_or(true, |c| lib[*i].category().eq_ignore_ascii_case(c))).collect();
        if lib.is_empty() {
            ui.add_space(6.0);
            ui.label(RichText::new(tr("This library is empty. Add materials from the built-in library or save your layers as a material.")).color(theme::text_dim()));
        }
        egui::ScrollArea::vertical().id_salt("dev_materials").max_height(260.0).auto_shrink([false, true]).show(ui, |ui| {
            let mut last = String::new();
            let mut order = shown.clone();
            order.sort_by_key(|i| (cat_label(&lib[*i].category()).to_lowercase(), lib[*i].name.to_lowercase()));
            for i in order {
                let c = lib[i].category();
                if !c.eq_ignore_ascii_case(&last) {
                    ui.label(RichText::new(cat_label(&c)).strong().color(theme::accent()));
                    last = c;
                }
                if material_card(ui, &lib[i], self.mat_dev_sel == Some(i), units, su).clicked() {
                    self.mat_dev_sel = Some(i);
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
            let cats = categories_of(self.doc.device.materials.iter().chain(&self.global_materials).chain(self.profiles.iter().flat_map(|p| p.materials.iter())));
            ui.horizontal(|ui| {
                ui.label(tr("Category"));
                let own = m.category();
                egui::ComboBox::from_id_salt("mat_category").width(120.0).selected_text(cat_label(&own)).show_ui(ui, |ui| {
                    for c in &cats {
                        if ui.selectable_label(own.eq_ignore_ascii_case(c), cat_label(c)).clicked() {
                            m.category = c.clone();
                        }
                    }
                });
                ui.add(egui::TextEdit::singleline(&mut self.mat_new_cat).hint_text(tr("or a new category")).desired_width(110.0));
                if ui.add_enabled(!self.mat_new_cat.trim().is_empty(), egui::Button::new(tr("Save"))).clicked() {
                    m.category = self.mat_new_cat.trim().to_string();
                    self.mat_new_cat.clear();
                }
            });
            let mut remove: Option<usize> = None;
            for (li, ml) in m.layers.iter_mut().enumerate() {
                let lname = self.doc.layers.get(ml.index).map(|l| l.name.clone()).unwrap_or_default();
                egui::CollapsingHeader::new(RichText::new(format!("{} ({})", ml.label, trf("Layer {}", &[&lname]))).strong()).id_salt(("mat_layer", i, li)).default_open(true).show(ui, |ui| {
                    layer_spec_ui(ui, ("mat_spec", i, li), &mut ml.spec, units, su);
                    ui.horizontal(|ui| {
                        ui.label(tr("Colour"));
                        colour_picker(ui, ("mat_colour", i, li), &mut ml.spec.colour, ml.index);
                        ui.label(tr("Label"));
                        ui.add(egui::TextEdit::singleline(&mut ml.label).desired_width(90.0));
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
            let name = self.mat_name.clone();
            let cat = self.save_category(ui, "tab_save_cat", &name);
            if ui.add_enabled(!name.trim().is_empty(), egui::Button::new(tr("Save"))).on_hover_text(tr("Saves the settings of the layers used in the design")).clicked() {
                let mut m = self.material_from_design(&name);
                m.category = cat;
                let name = m.name.clone();
                match self.doc.device.materials.iter().position(|e| e.same_key(&m)) {
                    Some(p) => self.doc.device.materials[p] = m,
                    None => self.doc.device.materials.push(m),
                }
                self.mat_dev_sel = self.doc.device.materials.iter().position(|e| e.name == name);
                self.sync_profile();
                self.status = trf("Saved {} in this device's library", &[&name]);
                self.mat_name.clear();
                self.mat_save_cat = None;
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(tr("Export library…")).clicked() {
                self.start_library_export();
            }
            if ui.button(tr("Import library…")).clicked() {
                self.start_library_import();
            }
        });
    }

    /// A category drop-down for a material that is going to be saved: it starts with the one the name suggests.
    fn save_category(&mut self, ui: &mut egui::Ui, id: &str, name: &str) -> String {
        let cats = categories_of(self.doc.device.materials.iter().chain(&self.global_materials).chain(self.profiles.iter().flat_map(|p| p.materials.iter())));
        let cur = self.mat_save_cat.clone().unwrap_or_else(|| guess_category(name).to_string());
        egui::ComboBox::from_id_salt(id).width(110.0).selected_text(cat_label(&cur)).show_ui(ui, |ui| {
            for c in &cats {
                if ui.selectable_label(cur.eq_ignore_ascii_case(c), cat_label(c)).clicked() {
                    self.mat_save_cat = Some(c.clone());
                }
            }
        });
        cur
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
        let mut export = false;
        let mut import = false;
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
                ui.add(egui::TextEdit::singleline(&mut self.mat_filter).hint_text(tr("Search")).desired_width(140.0));
                let refs: Vec<&Material> = all.iter().map(|(m, _)| m).collect();
                category_filter(ui, "mat_win_cat", &mut self.mat_cat, &refs);
            });
            ui.label(RichText::new(tr("Starting values for typical 10 W diode and 40 W CO2 lasers. Always test on scrap.")).color(theme::text_dim()));
            ui.label(RichText::new(tr("Materials added here are available for every device. A device's own library is edited in the Materials tab.")).color(theme::text_dim()));
            ui.separator();
            let q = self.mat_filter.to_lowercase();
            let cat = self.mat_cat.clone();
            let mut rows: Vec<usize> = (0..all.len()).filter(|i| self.mat_laser.map_or(true, |k| all[*i].0.laser == k)).filter(|i| q.is_empty() || all[*i].0.name.to_lowercase().contains(&q)).filter(|i| cat.as_ref().map_or(true, |c| all[*i].0.category().eq_ignore_ascii_case(c))).collect();
            rows.sort_by_key(|i| (cat_label(&all[*i].0.category()).to_lowercase(), all[*i].0.name.to_lowercase()));
            let (units, su) = (self.doc.device.units, self.doc.device.speed_unit);
            egui::ScrollArea::vertical().max_height(300.0).auto_shrink([false, false]).show(ui, |ui| {
                let mut last = String::new();
                for i in rows {
                    let (m, global) = &all[i];
                    let c = m.category();
                    if !c.eq_ignore_ascii_case(&last) {
                        ui.label(RichText::new(cat_label(&c)).strong().color(theme::accent()));
                        last = c;
                    }
                    if material_card(ui, m, self.mat_sel == Some(i), units, su).on_hover_text(format!("{} · {}", m.laser.label(), if *global { tr("Added by you") } else { tr("Built-in") })).clicked() {
                        self.mat_sel = Some(i);
                    }
                }
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
                ui.add(egui::TextEdit::singleline(&mut self.mat_global_name).hint_text(tr("Material name")).desired_width(180.0));
                let name = self.mat_global_name.clone();
                let cat = self.save_category(ui, "win_save_cat", &name);
                if ui.add_enabled(!name.trim().is_empty(), egui::Button::new(tr("Save to the library"))).clicked() {
                    let mut m = self.material_from_design(&name);
                    m.category = cat;
                    save_global = Some(m);
                }
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.button(tr("Export library…")).clicked() {
                    export = true;
                }
                if ui.button(tr("Import library…")).clicked() {
                    import = true;
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
                self.mat_save_cat = None;
            }
        }
        if export {
            self.start_library_export();
        }
        if import {
            self.start_library_import();
        }
        self.show_materials = open;
        if !open {
            self.mat_add_mode = false;
        }
    }
}


// -------------------------------------------------------------------------------------------------------------
// Library files (.lcb): the private library of the user in one file.
// -------------------------------------------------------------------------------------------------------------

/// A group of ticks: the box of the group, and one per entry. Ticking the group ticks all, and any entry ticks the group.
fn tick_group(ui: &mut egui::Ui, id: impl std::hash::Hash + std::fmt::Debug, title: &str, master: &mut bool, names: &[String], picks: &mut [bool]) {
    let state = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), ui.make_persistent_id(id), false);
    state
        .show_header(ui, |ui| {
            if ui.checkbox(master, RichText::new(title).strong()).changed() {
                picks.iter_mut().for_each(|p| *p = *master);
            }
        })
        .body(|ui| {
            let before = picks.to_vec();
            for (n, p) in names.iter().zip(picks.iter_mut()) {
                ui.checkbox(p, n);
            }
            if picks != before.as_slice() {
                *master = picks.iter().any(|p| *p);
            }
        });
}

impl App {
    /// Ask what to put into a library file.
    pub fn start_library_export(&mut self) {
        self.sync_profile();
        let file = LibraryFile::new(self.global_materials.clone(), self.profiles.clone());
        self.lib_dlg = Some(LibDlg::new(true, file, ""));
    }

    /// Pick a library file and ask what to take from it.
    pub fn start_library_import(&mut self) {
        let Some(path) = rfd::FileDialog::new().add_filter("LightCreator library", &["lcb"]).pick_file() else { return };
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        match LibraryFile::from_json(&text) {
            Some(file) => {
                let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                self.lib_dlg = Some(LibDlg::new(false, file, &name));
            }
            None => self.status = tr("This is not a library file of LightCreator.").into(),
        }
    }

    /// Write the chosen part of the library to a file.
    fn write_library(&mut self, sel: &LibraryFile, path: &PathBuf) {
        match std::fs::write(path, sel.to_json()) {
            Ok(()) => {
                let n = sel.global.len() + sel.devices.iter().map(|d| d.materials.len()).sum::<usize>();
                self.status = trf("Saved the library: {} material(s) and {} device(s)", &[&n, &sel.devices.len()]);
            }
            Err(e) => self.status = format!("{}: {e}", path.display()),
        }
    }

    /// Take a library read from a file into this one. New materials and devices are added; materials that differ
    /// open windows to choose the values. Returns how many materials and devices were added at once.
    pub fn import_library(&mut self, file: LibraryFile, source: &str) -> (usize, usize) {
        self.sync_profile();
        let (mut mats, mut devs) = (0, 0);
        let mut windows = vec![];
        let mut merge = |app: &mut App, target: MergeTarget, name: String, incoming: &[Material]| -> usize {
            let local = app.target_materials(target).map(|v| v.clone()).unwrap_or_default();
            let plan = plan_merge(&local, incoming);
            let added = plan.added.len();
            if let Some(lib) = app.target_materials(target) {
                lib.extend(plan.added);
            }
            if !plan.conflicts.is_empty() {
                let take = plan.conflicts.iter().map(|c| vec![false; c.rows.len()]).collect();
                windows.push(MergeDlg { source: source.to_string(), target, target_name: name, conflicts: plan.conflicts, take, added, unchanged: plan.unchanged });
            }
            added
        };
        mats += merge(self, MergeTarget::Global, tr("Material library").to_string(), &file.global);
        for d in file.devices {
            let found = self.profiles.iter().position(|p| p.name.trim().eq_ignore_ascii_case(d.name.trim()) && p.laser == d.laser);
            match found {
                Some(i) => mats += merge(self, MergeTarget::Profile(i), d.name.clone(), &d.materials),
                None => {
                    let mut d = d;
                    d.materials_seeded = true;
                    mats += d.materials.len();
                    devs += 1;
                    self.profiles.push(d);
                }
            }
        }
        self.sync_profile();
        for w in windows {
            self.queue_merge(w);
        }
        self.status = trf("Imported {} material(s) and {} device(s)", &[&mats, &devs]);
        (mats, devs)
    }

    /// The window in which the user ticks what to export or import.
    pub fn library_dialog(&mut self, ctx: &egui::Context) {
        let Some(dlg) = &mut self.lib_dlg else { return };
        let mut open = true;
        let mut go = false;
        let title = if dlg.export { tr("Export library…") } else { tr("Import library…") };
        let title = title.trim_end_matches('…').to_string();
        let frame = egui::Frame::window(&ctx.global_style()).inner_margin(egui::Margin { left: 10, right: 10, top: 6, bottom: 8 });
        egui::Window::new("library_file").title_bar(false).frame(frame).collapsible(false).default_size([440.0, 460.0]).resizable(true).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
            crate::window_ui::mini_title(ui, &title, &mut open, |_| {});
            ui.label(if dlg.export { tr("Tick what goes into the file.") } else { tr("Tick what to take from the file.") });
            if !dlg.export {
                ui.label(RichText::new(&dlg.source).color(theme::text_dim()));
            }
            ui.separator();
            egui::ScrollArea::vertical().max_height(340.0).auto_shrink([false, true]).show(ui, |ui| {
                let names: Vec<String> = dlg.file.global.iter().map(|m| format!("{}  ({})", m.name, m.laser.label())).collect();
                tick_group(ui, "lib_global", &format!("{} ({})", tr("Global materials"), names.len()), &mut dlg.global_on, &names, &mut dlg.global);
                ui.add_space(4.0);
                ui.label(RichText::new(tr("Devices")).strong());
                for (i, d) in dlg.file.devices.iter().enumerate() {
                    let names: Vec<String> = d.materials.iter().map(|m| m.name.clone()).collect();
                    tick_group(ui, ("lib_device", i), &format!("{} ({})", d.name, names.len()), &mut dlg.devices[i], &names, &mut dlg.mats[i]);
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button(tr("Select all")).clicked() {
                    dlg.global_on = true;
                    dlg.global.iter_mut().for_each(|p| *p = true);
                    dlg.devices.iter_mut().for_each(|p| *p = true);
                    dlg.mats.iter_mut().for_each(|v| v.iter_mut().for_each(|p| *p = true));
                }
                let text = if dlg.export { tr("Export") } else { tr("Import") };
                let ok = egui::Button::new(RichText::new(text).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0x2e, 0xa0, 0x4f));
                if ui.add_enabled(!dlg.nothing(), ok).clicked() {
                    go = true;
                }
            });
        });
        if go {
            if let Some(dlg) = self.lib_dlg.take() {
                let sel = dlg.selection();
                if dlg.export {
                    if let Some(path) = rfd::FileDialog::new().add_filter("LightCreator library", &["lcb"]).set_file_name("library.lcb").save_file() {
                        self.write_library(&sel, &path);
                    } else {
                        self.lib_dlg = Some(dlg);
                    }
                } else {
                    self.import_library(sel, &dlg.source);
                }
            }
        } else if !open {
            self.lib_dlg = None;
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

    #[test]
    fn a_library_file_is_chosen_piece_by_piece_and_imported_into_devices() {
        let (mut a, _ctx) = app_with_profile();
        a.global_materials.push(Material::from_layers("Mine", LaserKind::Diode, &[(0, &lc_core::Layer::new(0))]));
        let mut other = Device::default();
        other.name = "Second".into();
        other.materials = a.doc.device.materials[..2].to_vec();
        a.profiles.push(other);
        a.start_library_export();
        let dlg = a.lib_dlg.as_mut().unwrap();
        assert!(dlg.export && dlg.nothing() == false);
        // Untick the first device, one material of the second and the global ones.
        dlg.devices[0] = false;
        dlg.mats[1][0] = false;
        dlg.global.iter_mut().for_each(|p| *p = false);
        let sel = dlg.selection();
        assert_eq!((sel.global.len(), sel.devices.len(), sel.devices[0].name.as_str(), sel.devices[0].materials.len()), (0, 1, "Second", 1));
        // Import into a fresh program: the known device gets what it lacks, an unknown one is added whole.
        let mut file = sel;
        let mut third = Device::default();
        third.name = "Third".into();
        third.materials = a.doc.device.materials[..3].to_vec();
        file.devices.push(third);
        file.global.push(Material::from_layers("From file", LaserKind::Co2, &[(1, &lc_core::Layer::new(1))]));
        let n = a.profiles.len();
        let (mats, devs) = a.import_library(file, "x.lcb");
        assert_eq!((devs, a.profiles.len()), (1, n + 1));
        assert_eq!(mats, 1 + 3, "one global material and the three of the new device; the known one had it already");
        assert!(a.global_materials.iter().any(|m| m.name == "From file"));
        assert!(a.mat_merge.is_none());
        // A material that differs opens a window naming its target.
        let mut changed = a.profiles[1].materials[0].clone();
        changed.layers[0].spec.power -= 10.0;
        let mut file = lc_core::materials::LibraryFile::default();
        let mut d = Device::default();
        d.name = "Second".into();
        d.materials = vec![changed];
        file.devices.push(d);
        a.import_library(file, "y.lcb");
        let dlg = a.mat_merge.as_ref().expect("differences");
        assert_eq!((dlg.target, dlg.target_name.as_str()), (MergeTarget::Profile(1), "Second"));
    }

    #[test]
    fn the_order_of_the_tabs_is_kept_and_new_tabs_join_the_end() {
        use crate::app::SideTab;
        let order = SideTab::arranged("materials,clipart,bogus,materials");
        assert_eq!(order[0].0, SideTab::Materials);
        assert_eq!(order[1].0, SideTab::Clipart);
        assert_eq!(order.len(), SideTab::ALL.len());
        let ids = order.iter().map(|(t, _)| t.id()).collect::<Vec<_>>().join(",");
        assert_eq!(SideTab::arranged(&ids), order);
        // The default order is the one of `ALL`.
        assert_eq!(SideTab::arranged(""), SideTab::ALL.to_vec());
    }
}
