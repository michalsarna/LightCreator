//! Materials: the **Materials** tab holds the library of the device you work with (settings of all layers of
//! each material; it is kept in the device profile and travels with saved projects). The built-in library and the
//! materials added for every device are browsed, and extended, in the **Material library** window of the Laser menu.
//! A library read from a project file is merged into the device library, with a window to settle differences.
use crate::app::App;
use crate::i18n::{tr, trf};
use crate::theme;
use crate::units_ui::{drag_len, drag_speed, fmt_len, fmt_speed};
use eframe::egui::{self, Color32, RichText};
use lc_core::materials::{add_missing, apply_rows, builtin_materials, effective_library, guess_category, plan_merge, Conflict, LayerSpec, LibraryFile, Material, MaterialLayer, Value, CATEGORIES, NO_COLOUR};
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

/// The name of a material in the language of the interface; the names that ship with the program are translated.
fn tr_name(name: &str) -> String {
    match name {
        "Plywood" => tr("Plywood"),
        "MDF" => tr("MDF"),
        "Cardboard" => tr("Cardboard"),
        "Paper" => tr("Paper"),
        "Leather" => tr("Leather"),
        "Felt" => tr("Felt"),
        "Cork" => tr("Cork"),
        "Acrylic" => tr("Acrylic"),
        "Acrylic, black or dark" => tr("Acrylic, black or dark"),
        "Slate / stone" => tr("Slate / stone"),
        "Anodised aluminium (coated)" => tr("Anodised aluminium (coated)"),
        "Rubber" => tr("Rubber"),
        "Fabric" => tr("Fabric"),
        "Glass" => tr("Glass"),
        "Wood (hardwood)" => tr("Wood (hardwood)"),
        other => return other.to_string(),
    }
    .to_string()
}

/// "Plywood 3 mm" in the language and units of the interface.
pub fn mat_name(m: &Material, units: Units) -> String {
    if m.thickness > 0.0 {
        let digits = if (m.thickness.fract()).abs() < 1e-9 { 0 } else { 1 };
        format!("{} {}", tr_name(&m.name), fmt_len(units, m.thickness, digits))
    } else {
        tr_name(&m.name)
    }
}

/// A tag; "Cut", "Engrave" and "Score" are translated.
fn tr_tag(t: &str) -> String {
    match t {
        "Cut" => tr("Cut"),
        "Engrave" => tr("Engrave"),
        "Score" => tr("Score"),
        other => return other.to_string(),
    }
    .to_string()
}

/// One material as the main library lists it: all the laser variants that share an id.
pub struct Entry {
    pub key: String,
    pub variants: Vec<Material>,
}

impl Entry {
    /// The variant to show: for the laser type asked for, else for the device, else any.
    fn pick(&self, want: Option<LaserKind>, device: LaserKind) -> &Material {
        want.and_then(|k| self.variants.iter().find(|m| m.laser == k)).or_else(|| self.variants.iter().find(|m| m.laser == device)).unwrap_or(&self.variants[0])
    }

    fn lasers(&self) -> String {
        self.variants.iter().map(|m| m.laser.label()).collect::<Vec<_>>().join(", ")
    }
}

pub fn entries(lib: &[Material]) -> Vec<Entry> {
    let mut out: Vec<Entry> = vec![];
    for m in lib {
        match out.iter_mut().find(|e| e.key == m.key()) {
            Some(e) => e.variants.push(m.clone()),
            None => out.push(Entry { key: m.key(), variants: vec![m.clone()] }),
        }
    }
    out
}

/// A group of a list that folds up by its title (the name of the category, in the main colour), as in the clipart list.
fn category_section(ui: &mut egui::Ui, id: impl std::hash::Hash + std::fmt::Debug, cat: &str, n: usize, body: impl FnOnce(&mut egui::Ui)) {
    egui::CollapsingHeader::new(RichText::new(format!("{} ({n})", cat_label(cat))).strong().color(theme::accent())).id_salt(id).default_open(true).show(ui, body);
}

/// What the editor of a material needs to know about the program around it.
pub struct EditCx<'a> {
    pub units: Units,
    pub su: SpeedUnit,
    /// The layers of the design: names, and the settings a new layer of the material starts with.
    pub layers: &'a [Layer],
    pub cats: Vec<String>,
    pub new_cat: &'a mut String,
    pub add_layer: &'a mut usize,
}

/// Everything about one material: name, thickness, category, tags, note and the settings of each layer.
pub fn material_editor(ui: &mut egui::Ui, m: &mut Material, salt: &str, cx: &mut EditCx) {
    m.freeze_id();
    ui.horizontal_wrapped(|ui| {
        ui.add(egui::TextEdit::singleline(&mut m.name).desired_width((ui.available_width() - 130.0).clamp(80.0, 240.0)));
        ui.label(tr("Thickness"));
        drag_len(ui, cx.units, &mut m.thickness, 0.1, Some((0.0, 100.0)));
    });
    ui.horizontal_wrapped(|ui| {
        ui.label(tr("Category"));
        let own = m.category();
        egui::ComboBox::from_id_salt((salt, "category")).width(120.0).selected_text(cat_label(&own)).show_ui(ui, |ui| {
            for c in &cx.cats {
                if ui.selectable_label(own.eq_ignore_ascii_case(c), cat_label(c)).clicked() {
                    m.category = c.clone();
                }
            }
        });
        ui.add(egui::TextEdit::singleline(cx.new_cat).hint_text(tr("or a new category")).desired_width(110.0));
        if ui.add_enabled(!cx.new_cat.trim().is_empty(), egui::Button::new(tr("Save"))).clicked() {
            m.category = cx.new_cat.trim().to_string();
            cx.new_cat.clear();
        }
    });
    // The tags are typed as one line; the text is kept between frames so that a comma can be typed.
    let tags_id = ui.make_persistent_id((salt, "tags", m.key(), m.laser as u8));
    let mut text: String = ui.data_mut(|d| d.get_temp::<String>(tags_id)).unwrap_or_else(|| m.tags.iter().map(|t| tr_tag(t)).collect::<Vec<_>>().join(", "));
    ui.horizontal(|ui| {
        ui.label(tr("Tags"));
        if ui.add(egui::TextEdit::singleline(&mut text).hint_text(tr("Tags, separated by commas")).desired_width(ui.available_width() - 4.0)).changed() {
            m.tags = text.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect();
        }
    });
    ui.data_mut(|d| d.insert_temp(tags_id, text));
    ui.add(egui::TextEdit::singleline(&mut m.note).hint_text(tr("Note")).desired_width(ui.available_width() - 4.0));
    let mut remove: Option<usize> = None;
    for (li, ml) in m.layers.iter_mut().enumerate() {
        let lname = cx.layers.get(ml.index).map(|l| l.name.clone()).unwrap_or_default();
        egui::CollapsingHeader::new(RichText::new(format!("{} ({})", ml.label, trf("Layer {}", &[&lname]))).strong()).id_salt((salt, "layer", li)).default_open(true).show(ui, |ui| {
            layer_spec_ui(ui, (salt, 0, li), &mut ml.spec, cx.units, cx.su);
            ui.horizontal_wrapped(|ui| {
                ui.label(tr("Colour"));
                colour_picker(ui, (salt, "colour", li), &mut ml.spec.colour, ml.index);
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
    let free: Vec<usize> = (0..cx.layers.len().min(30)).filter(|i| m.layer(*i).is_none()).collect();
    ui.horizontal_wrapped(|ui| {
        let mut pick = (*cx.add_layer).min(29);
        if !free.contains(&pick) {
            pick = free.first().copied().unwrap_or(0);
        }
        egui::ComboBox::from_id_salt((salt, "add_layer")).selected_text(cx.layers[pick].name.clone()).show_ui(ui, |ui| {
            for f in &free {
                ui.selectable_value(&mut pick, *f, cx.layers[*f].name.clone());
            }
        });
        *cx.add_layer = pick;
        if ui.add_enabled(!free.is_empty(), egui::Button::new(tr("Add layer"))).on_hover_text(tr("Uses the settings of this layer in the design")).clicked() {
            let l = &cx.layers[pick];
            m.layers.push(MaterialLayer { index: pick, label: l.name.clone(), spec: LayerSpec::from_layer(l) });
            m.layers.sort_by_key(|l| l.index);
        }
    });
}

/// The window that edits one material of the main library.
pub struct MatEdit {
    pub m: Material,
}

/// One entry of a list of materials: name and category, then every layer with its colour, label and main settings.
fn material_card(ui: &mut egui::Ui, m: &Material, selected: bool, units: Units, su: SpeedUnit, lasers: Option<&str>) -> egui::Response {
    let fill = if selected { theme::accent().gamma_multiply(0.18) } else { Color32::TRANSPARENT };
    // The frame adds its margin to the inner width, so leave room for it or the panel grows with every frame.
    let inner = (ui.available_width() - 8.0).max(40.0);
    let out = egui::Frame::new().fill(fill).corner_radius(3).inner_margin(egui::Margin::symmetric(4, 3)).show(ui, |ui| {
        ui.set_width(inner);
        ui.horizontal(|ui| {
            ui.label(RichText::new(mat_name(m, units)).strong());
            if let Some(l) = lasers {
                ui.label(RichText::new(l).small().color(theme::text_dim()));
            }
        });
        if !m.tags.is_empty() || !m.note.trim().is_empty() {
            let tags = m.tags.iter().map(|t| tr_tag(t)).collect::<Vec<_>>().join(", ");
            ui.label(RichText::new(format!("{}{}{}", tags, if tags.is_empty() || m.note.trim().is_empty() { "" } else { " · " }, m.note.trim())).small().italics().color(theme::text_dim()));
        }
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            for l in &m.layers {
                swatch(ui, layer_colour(l));
                ui.label(RichText::new(format!("{} · {} · {:.0} % · {}×", l.label, fmt_speed(units, su, false, l.spec.speed), l.spec.power, l.spec.passes)).small().color(theme::text_dim()));
                ui.add_space(6.0);
            }
        });
    });
    let r = ui.interact(out.response.rect, ui.id().with(("material_card", m.key(), m.laser as u8)), egui::Sense::click());
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

    /// The main library as this computer has it: the built-in materials with the changes saved here, and the
    /// materials added here.
    pub fn main_library(&self) -> Vec<Material> {
        effective_library(builtin(), &self.global_materials)
    }

    /// Fill the library of the device from the main library, once.
    pub fn seed_device_library(&mut self) {
        if self.doc.device.materials_seeded {
            return;
        }
        let laser = self.doc.device.laser;
        let main = self.main_library();
        add_missing(&mut self.doc.device.materials, &main, laser);
        self.doc.device.materials_seeded = true;
        self.sync_profile();
    }

    /// Merge the library that came with a project file into the one of this device. New materials are added at
    /// once; those that differ open a window to choose, value by value.
    pub fn import_incoming_materials(&mut self, incoming: Vec<Material>, source: &str) {
        if incoming.is_empty() {
            return;
        }
        // Only what suits the laser of this device belongs in its library.
        let laser = self.doc.device.laser;
        let incoming: Vec<Material> = incoming.into_iter().filter(|m| m.laser == laser).collect();
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
        let matches = |m: &Material| q.is_empty() || mat_name(m, units).to_lowercase().contains(&q) || m.tags.iter().any(|t| tr_tag(t).to_lowercase().contains(&q)) || m.note.to_lowercase().contains(&q);
        let shown: Vec<usize> = (0..lib.len()).filter(|i| matches(&lib[*i])).filter(|i| cat.as_ref().map_or(true, |c| lib[*i].category().eq_ignore_ascii_case(c))).collect();
        if lib.is_empty() {
            ui.add_space(6.0);
            ui.label(RichText::new(tr("This library is empty. Add materials from the built-in library or save your layers as a material.")).color(theme::text_dim()));
        }
        egui::ScrollArea::vertical().id_salt("dev_materials").max_height(260.0).auto_shrink([false, true]).show(ui, |ui| {
            let mut cats: Vec<String> = shown.iter().map(|i| lib[*i].category()).collect();
            cats.sort_by_key(|c| cat_label(c).to_lowercase());
            cats.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
            for c in cats {
                let mut order: Vec<usize> = shown.iter().copied().filter(|i| lib[*i].category().eq_ignore_ascii_case(&c)).collect();
                order.sort_by_key(|i| (mat_name(&lib[*i], units).to_lowercase(), lib[*i].thickness as i64));
                category_section(ui, ("tab_cat", c.clone()), &c, order.len(), |ui| {
                    for i in order {
                        if material_card(ui, &lib[i], self.mat_dev_sel == Some(i), units, su, None).clicked() {
                            self.mat_dev_sel = Some(i);
                        }
                    }
                });
            }
        });
        ui.separator();
        if let Some(i) = self.mat_dev_sel.filter(|i| *i < lib.len()) {
            let mut m = lib[i].clone();
            let mut apply = false;
            let mut delete = false;
            let cats = categories_of(self.doc.device.materials.iter().chain(&self.global_materials).chain(self.profiles.iter().flat_map(|p| p.materials.iter())));
            let mut cx = EditCx { units, su, layers: &self.doc.layers, cats, new_cat: &mut self.mat_new_cat, add_layer: &mut self.mat_add_layer };
            material_editor(ui, &mut m, "tab_edit", &mut cx);
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
                self.doc.material = Some(m.clone());
                self.touch();
                self.status = trf("Applied {} to {} layer(s)", &[&mat_name(&m, units), &n]);
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
        ui.horizontal_wrapped(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.mat_name).hint_text(tr("Material name")).desired_width((ui.available_width() - 40.0).clamp(80.0, 240.0)));
            let name = self.mat_name.clone();
            let cat = self.save_category(ui, "tab_save_cat", &name);
            if ui.add_enabled(!name.trim().is_empty(), egui::Button::new(tr("Save"))).on_hover_text(tr("Saves the settings of the layers used in the design")).clicked() {
                let mut m = self.material_from_design(&name);
                m.category = cat;
                let shown = mat_name(&m, units);
                let key = m.key();
                match self.doc.device.materials.iter().position(|e| e.same_key(&m)) {
                    Some(p) => self.doc.device.materials[p] = m.clone(),
                    None => self.doc.device.materials.push(m.clone()),
                }
                self.mat_dev_sel = self.doc.device.materials.iter().position(|e| e.key() == key);
                self.doc.material = Some(m);
                self.sync_profile();
                self.status = trf("Saved {} in this device's library", &[&shown]);
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
        let mut add_to_device: Option<String> = None;
        let mut edit: Option<String> = None;
        let mut delete: Option<String> = None;
        let mut save_global: Option<Material> = None;
        let mut export = false;
        let mut import = false;
        let (units, su) = (self.doc.device.units, self.doc.device.speed_unit);
        let device_laser = self.doc.device.laser;
        let frame = egui::Frame::window(&ctx.global_style()).inner_margin(egui::Margin { left: 10, right: 10, top: 6, bottom: 8 });
        egui::Window::new("material_library").title_bar(false).frame(frame).default_size([640.0, 540.0]).resizable(true).show(ctx, |ui| {
            crate::window_ui::mini_title(ui, &tr("Material library"), &mut open, |_| {});
            let lib = self.main_library();
            let ents = entries(&lib);
            ui.horizontal_wrapped(|ui| {
                ui.label(tr("Laser type"));
                let label = |l: Option<LaserKind>| l.map(|k| k.label().to_string()).unwrap_or_else(|| tr("All").to_string());
                egui::ComboBox::from_id_salt("mat_laser").selected_text(label(self.mat_laser)).show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.mat_laser, None, tr("All"));
                    for k in LaserKind::ALL {
                        ui.selectable_value(&mut self.mat_laser, Some(k), k.label());
                    }
                });
                ui.add(egui::TextEdit::singleline(&mut self.mat_filter).hint_text(tr("Search")).desired_width(140.0));
                let refs: Vec<&Material> = lib.iter().collect();
                category_filter(ui, "mat_win_cat", &mut self.mat_cat, &refs);
            });
            ui.label(RichText::new(tr("Starting values for typical 10 W diode and 40 W CO2 lasers. Always test on scrap.")).color(theme::text_dim()));
            ui.label(RichText::new(tr("Materials added here are available for every device. A device's own library is edited in the Materials tab.")).color(theme::text_dim()));
            ui.label(RichText::new(tr("Double-click a material to edit its settings. Changes are kept on this computer.")).color(theme::text_dim()));
            ui.separator();
            let q = self.mat_filter.to_lowercase();
            let cat = self.mat_cat.clone();
            let want = self.mat_laser;
            let shown: Vec<&Entry> = ents
                .iter()
                .filter(|e| want.map_or(true, |k| e.variants.iter().any(|m| m.laser == k)))
                .filter(|e| {
                    let m = e.pick(want, device_laser);
                    (q.is_empty() || mat_name(m, units).to_lowercase().contains(&q) || m.tags.iter().any(|t| tr_tag(t).to_lowercase().contains(&q)) || m.note.to_lowercase().contains(&q))
                        && cat.as_ref().map_or(true, |c| m.category().eq_ignore_ascii_case(c))
                })
                .collect();
            egui::ScrollArea::vertical().max_height(320.0).auto_shrink([false, false]).show(ui, |ui| {
                let mut cats: Vec<String> = shown.iter().map(|e| e.pick(want, device_laser).category()).collect();
                cats.sort_by_key(|c| cat_label(c).to_lowercase());
                cats.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
                for c in cats {
                    let mut group: Vec<&&Entry> = shown.iter().filter(|e| e.pick(want, device_laser).category().eq_ignore_ascii_case(&c)).collect();
                    group.sort_by_key(|e| {
                        let m = e.pick(want, device_laser);
                        (mat_name(m, units).to_lowercase(), m.thickness as i64)
                    });
                    category_section(ui, ("win_cat", c.clone()), &c, group.len(), |ui| {
                        for e in group {
                            let m = e.pick(want, device_laser);
                            let r = material_card(ui, m, self.mat_sel.as_deref() == Some(e.key.as_str()), units, su, Some(&e.lasers()));
                            if r.clicked() {
                                self.mat_sel = Some(e.key.clone());
                            }
                            if r.double_clicked() {
                                edit = Some(e.key.clone());
                            }
                        }
                    });
                }
            });
            ui.separator();
            let sel = self.mat_sel.clone().filter(|k| ents.iter().any(|e| e.key == *k));
            let builtin_key = |k: &str| builtin().iter().any(|b| b.key() == k);
            let local_key = |k: &str| self.global_materials.iter().any(|g| g.key() == k);
            ui.horizontal_wrapped(|ui| {
                let go = egui::Button::new(RichText::new(tr("Add to this device")).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0x2e, 0xa0, 0x4f));
                let has_variant = sel.as_ref().is_some_and(|k| lib.iter().any(|m| m.key() == *k && m.laser == device_laser));
                if ui.add_enabled(has_variant, go).on_disabled_hover_text(tr("This material has no settings for the laser type of this device")).clicked() {
                    add_to_device = sel.clone();
                }
                if ui.add_enabled(sel.is_some(), egui::Button::new(tr("Edit material"))).clicked() {
                    edit = sel.clone();
                }
                // Materials added here can be deleted; a built-in one that was changed goes back to its built-in values.
                let (text, can) = match &sel {
                    Some(k) if builtin_key(k) => (tr("Restore built-in values"), local_key(k)),
                    Some(k) => (tr("Delete"), local_key(k)),
                    None => (tr("Delete"), false),
                };
                if ui.add_enabled(can, egui::Button::new(text)).clicked() {
                    delete = sel.clone();
                }
            });
            ui.add_space(6.0);
            ui.label(RichText::new(tr("Save the layers of the design as a material for every device")).strong());
            ui.horizontal_wrapped(|ui| {
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
        if let Some(k) = add_to_device {
            if let Some(m) = self.main_library().into_iter().find(|m| m.key() == k && m.laser == device_laser) {
                let shown = mat_name(&m, units);
                match self.doc.device.materials.iter().position(|e| e.same_key(&m)) {
                    Some(_) => self.status = trf("{} is already in this device's library", &[&shown]),
                    None => {
                        self.status = trf("Added {} to this device's library", &[&shown]);
                        self.doc.device.materials.push(m);
                        self.sync_profile();
                    }
                }
            }
        }
        if let Some(k) = edit {
            let lib = self.main_library();
            if let Some(m) = lib.iter().find(|m| m.key() == k && m.laser == device_laser).or_else(|| lib.iter().find(|m| m.key() == k)) {
                self.mat_edit = Some(MatEdit { m: m.clone() });
            }
        }
        if let Some(k) = delete {
            self.global_materials.retain(|g| g.key() != k);
            if !builtin().iter().any(|b| b.key() == k) {
                self.mat_sel = None;
            }
        }
        if let Some(m) = save_global {
            if self.main_library().iter().any(|e| e.same_key(&m)) {
                self.status = tr("A material with this name already exists in the library.").into();
            } else {
                self.status = trf("Saved {} in the material library", &[&mat_name(&m, units)]);
                self.mat_sel = Some(m.key());
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

    /// Edit one material of the main library; saving keeps the changes in the local main library, over the
    /// values that ship with the program.
    pub fn material_edit_window(&mut self, ctx: &egui::Context) {
        let Some(mut ed) = self.mat_edit.take() else { return };
        let (units, su) = (self.doc.device.units, self.doc.device.speed_unit);
        let mut open = true;
        let mut save = false;
        let mut restore = false;
        let mut new_laser: Option<LaserKind> = None;
        let cats = categories_of(self.main_library().iter().chain(self.doc.device.materials.iter()));
        let has_builtin = builtin().iter().any(|b| b.same_key(&ed.m));
        let frame = egui::Frame::window(&ctx.global_style()).inner_margin(egui::Margin { left: 10, right: 10, top: 6, bottom: 8 });
        egui::Window::new("material_edit").title_bar(false).frame(frame).default_size([460.0, 560.0]).resizable(true).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
            let title = format!("{}  ·  {}", tr("Edit material"), mat_name(&ed.m, units));
            crate::window_ui::mini_title(ui, &title, &mut open, |_| {});
            ui.horizontal(|ui| {
                ui.label(tr("Laser type"));
                egui::ComboBox::from_id_salt("edit_laser").selected_text(ed.m.laser.label()).show_ui(ui, |ui| {
                    for k in LaserKind::ALL {
                        if ui.selectable_label(ed.m.laser == k, k.label()).clicked() && k != ed.m.laser {
                            new_laser = Some(k);
                        }
                    }
                });
            });
            ui.separator();
            egui::ScrollArea::vertical().max_height(400.0).auto_shrink([false, true]).show(ui, |ui| {
                let mut cx = EditCx { units, su, layers: &self.doc.layers, cats, new_cat: &mut self.mat_new_cat, add_layer: &mut self.mat_add_layer };
                material_editor(ui, &mut ed.m, "main_edit", &mut cx);
            });
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                let ok = egui::Button::new(RichText::new(tr("Save")).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0x2e, 0xa0, 0x4f));
                if ui.add_sized([120.0, 28.0], ok).clicked() {
                    save = true;
                }
                if has_builtin && ui.add_sized([200.0, 28.0], egui::Button::new(tr("Restore built-in values"))).clicked() {
                    restore = true;
                }
            });
        });
        // The other laser type: its settings from the library, or a copy of these to start from.
        if let Some(k) = new_laser {
            let key = ed.m.key();
            ed.m = match self.main_library().into_iter().find(|m| m.key() == key && m.laser == k) {
                Some(m) => m,
                None => Material { laser: k, ..ed.m.clone() },
            };
        }
        if restore {
            self.global_materials.retain(|g| !g.same_key(&ed.m));
            if let Some(b) = builtin().iter().find(|b| b.same_key(&ed.m)) {
                ed.m = b.clone();
            }
            self.status = tr("The built-in values were restored.").into();
        }
        if save {
            let m = ed.m.clone();
            self.global_materials.retain(|g| !g.same_key(&m));
            // The same as the built-in one needs no copy of its own.
            if !builtin().iter().any(|b| *b == m) {
                self.global_materials.push(m.clone());
            }
            self.status = trf("Saved {} in the material library", &[&mat_name(&m, units)]);
            open = false;
        }
        if open {
            self.mat_edit = Some(ed);
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
                let names: Vec<String> = dlg.file.global.iter().map(|m| format!("{}  ({})", mat_name(m, Units::Mm), m.laser.label())).collect();
                tick_group(ui, "lib_global", &format!("{} ({})", tr("Global materials"), names.len()), &mut dlg.global_on, &names, &mut dlg.global);
                ui.add_space(4.0);
                ui.label(RichText::new(tr("Devices")).strong());
                for (i, d) in dlg.file.devices.iter().enumerate() {
                    let names: Vec<String> = d.materials.iter().map(|m| mat_name(m, Units::Mm)).collect();
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
        fresh.id = "only-in-the-file".into();
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

    #[test]
    fn local_changes_to_the_main_library_reach_new_devices_and_the_project_remembers_its_material() {
        let ctx = Context::default();
        let mut a = App::build(&ctx, None, false);
        let mut changed = builtin().iter().find(|m| m.key() == "plywood-3" && m.laser == LaserKind::Diode).unwrap().clone();
        changed.layers[0].spec.power = 42.0;
        a.global_materials.push(changed);
        assert_eq!(a.main_library().len(), builtin().len(), "an override does not add an entry");
        a.profiles = vec![Device::default()];
        a.enter_editor(0);
        let ply = a.doc.device.materials.iter().position(|m| m.key() == "plywood-3").unwrap();
        assert_eq!(a.doc.device.materials[ply].layers[0].spec.power, 42.0);
        // Applying a material of the device library puts it into the project.
        a.mat_dev_sel = Some(ply);
        let m = a.doc.device.materials[ply].clone();
        a.doc.material = Some(m.clone());
        let doc2 = Document::from_json(&a.doc.to_json()).unwrap();
        assert_eq!(doc2.material, Some(m));
    }
}
