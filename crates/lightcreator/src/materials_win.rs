//! The window that manages the materials of the device and their layers: the list of materials (add, copy, rename, delete,
//! move, a colour), the layers of the selected one with all their settings, and export and import to a file.
use crate::app::App;
use crate::i18n::{tr, trf};
use crate::materials_ui::material_label;
use crate::panels::{layer_settings_ui, LayerCtx};
use crate::theme;
use eframe::egui::{self, Color32, RichText};
use lc_core::materials::{self, Material};

/// What the window remembers while it is open.
#[derive(Clone, Debug, Default)]
pub struct MaterialsDlg {
    /// The material shown on the right.
    pub sel: usize,
    /// The name and the colour being typed for it, and the material they belong to (they are read again when it changes).
    name: String,
    colour: Option<[u8; 3]>,
    loaded: Option<usize>,
    /// Import: a material with a name that is there replaces it, instead of being kept under another name.
    pub replace: bool,
    /// The layer chosen to be added.
    add_index: usize,
    /// The last thing that was done or went wrong.
    pub message: String,
}

/// What a click in the window asked for; it is carried out after the window was drawn.
enum Do {
    Select(usize),
    New,
    Duplicate,
    Delete,
    Move(bool),
    Rename,
    Export(bool),
    Import,
    AddLayer(usize),
    RemoveLayer(usize),
}

impl App {
    pub fn open_materials_window(&mut self) {
        self.doc.sync_material();
        self.materials_dlg = Some(MaterialsDlg { sel: self.doc.device.active_material, ..Default::default() });
    }

    pub fn materials_window(&mut self, ctx: &egui::Context) {
        let Some(mut dlg) = self.materials_dlg.take() else { return };
        self.doc.device.ensure_materials();
        let n = self.doc.device.materials.len();
        dlg.sel = dlg.sel.min(n - 1);
        if dlg.loaded != Some(dlg.sel) {
            let m = &self.doc.device.materials[dlg.sel];
            dlg.name = m.name.clone();
            dlg.colour = m.color;
            dlg.loaded = Some(dlg.sel);
        }
        let (units, su) = (self.doc.device.units, self.doc.device.speed_unit);
        let layer_ctx = LayerCtx::of(&self.doc.device);
        let names: Vec<(String, Option<[u8; 3]>)> = self.doc.device.materials.iter().map(|m| (material_label(&m.name), m.color)).collect();
        let active = self.doc.device.active_material;
        let mut edited: Material = self.doc.device.materials[dlg.sel].clone();
        let before = edited.clone();
        let mut todo: Option<Do> = None;
        let mut open = true;
        crate::window_ui::popup(ctx, "materials_window", &tr("Materials"), [820.0, 560.0], &mut open, |ui| {
            ui.horizontal_top(|ui| {
                // The list of materials and what can be done with them.
                ui.vertical(|ui| {
                    ui.set_width(230.0);
                    egui::ScrollArea::vertical().id_salt("materials_list").max_height(300.0).auto_shrink([false, false]).show(ui, |ui| {
                        for (i, (name, colour)) in names.iter().enumerate() {
                            ui.horizontal(|ui| {
                                let (r, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                                if let Some(c) = colour {
                                    ui.painter().rect_filled(r, 2.0, Color32::from_rgb(c[0], c[1], c[2]));
                                }
                                let label = if i == active { format!("{name} ●") } else { name.clone() };
                                if ui.selectable_label(i == dlg.sel, label).clicked() {
                                    todo = Some(Do::Select(i));
                                }
                            });
                        }
                    });
                    ui.add_space(4.0);
                    let icon = |name: &str| crate::icons::slot(Some(name), theme::text());
                    ui.horizontal_wrapped(|ui| {
                        if ui.add(egui::Button::image(icon("plus"))).on_hover_text(tr("New, empty material")).clicked() {
                            todo = Some(Do::New);
                        }
                        if ui.add(egui::Button::image(icon("copy"))).on_hover_text(tr("Duplicate")).clicked() {
                            todo = Some(Do::Duplicate);
                        }
                        if ui.add_enabled(dlg.sel > 0, egui::Button::image(icon("chevron-up"))).on_hover_text(tr("Up")).clicked() {
                            todo = Some(Do::Move(true));
                        }
                        if ui.add_enabled(dlg.sel + 1 < n, egui::Button::image(icon("chevron-down"))).on_hover_text(tr("Down")).clicked() {
                            todo = Some(Do::Move(false));
                        }
                        if ui.add_enabled_ui(n > 1, |ui| crate::window_ui::delete_icon(ui, &tr("Delete this material"))).inner.clicked() {
                            todo = Some(Do::Delete);
                        }
                    });
                    ui.separator();
                    ui.horizontal_wrapped(|ui| {
                        if ui.button(tr("Export…")).clicked() {
                            todo = Some(Do::Export(false));
                        }
                        if ui.button(tr("Export all…")).clicked() {
                            todo = Some(Do::Export(true));
                        }
                        if ui.button(tr("Import…")).clicked() {
                            todo = Some(Do::Import);
                        }
                    });
                    ui.checkbox(&mut dlg.replace, tr("Replace materials that have the same name"));
                    if !dlg.message.is_empty() {
                        ui.label(RichText::new(&dlg.message).color(theme::text_dim()));
                    }
                });
                ui.separator();
                // The selected material: its name, its colour and its layers.
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        let mut col = dlg.colour.unwrap_or([128, 128, 128]);
                        let picked = ui.scope(|ui| {
                            ui.spacing_mut().interact_size = egui::vec2(22.0, 20.0);
                            ui.color_edit_button_srgb(&mut col).on_hover_text(tr("The colour of the material is only shown; it changes nothing.")).changed()
                        });
                        if picked.inner {
                            dlg.colour = Some(col);
                        }
                        if dlg.colour.is_some() && ui.add(egui::Button::image(crate::icons::slot(Some("x"), theme::text())).small()).on_hover_text(tr("Clear")).clicked() {
                            dlg.colour = None;
                        }
                        ui.add(egui::TextEdit::singleline(&mut dlg.name).desired_width(220.0));
                        let changed = dlg.name.trim() != self_name(&before) || dlg.colour != before.color;
                        let save = egui::Button::new(RichText::new(tr("Save")).color(Color32::WHITE).strong()).fill(crate::window_ui::SAVE_GREEN);
                        if ui.add_enabled(changed && !dlg.name.trim().is_empty(), save).clicked() {
                            todo = Some(Do::Rename);
                        }
                    });
                    ui.add_space(6.0);
                    ui.label(RichText::new(tr("Layers")).strong());
                    egui::ScrollArea::vertical().id_salt("materials_layers").max_height(340.0).auto_shrink([false, false]).show(ui, |ui| {
                        if edited.layers.is_empty() {
                            ui.label(RichText::new(tr("This material has no layers yet. Add one below.")).color(theme::text_dim()));
                        }
                        for ml in &mut edited.layers {
                            let title = format!("{} · {}", ml.settings.name, tr(ml.settings.mode.label()));
                            let id = ("mat_layer", dlg.sel, ml.index);
                            egui::CollapsingHeader::new(title).id_salt(id).show(ui, |ui| {
                                layer_settings_ui(ui, (units, su), &mut ml.settings, &format!("mw-{}-{}", dlg.sel, ml.index), false, &layer_ctx);
                                if ui.add(egui::Button::image_and_text(crate::icons::slot(Some("trash"), theme::text()), tr("Remove"))).on_hover_text(tr("Remove this layer from the material")).clicked() {
                                    todo = Some(Do::RemoveLayer(ml.index));
                                }
                            });
                        }
                    });
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let free: Vec<usize> = (0..lc_core::LAYER_COUNT).filter(|i| edited.layer(*i).is_none()).collect();
                        if !free.contains(&dlg.add_index) {
                            dlg.add_index = free.first().copied().unwrap_or(0);
                        }
                        let shown = free.iter().find(|i| **i == dlg.add_index).map(|i| format!("C{i:02}")).unwrap_or_default();
                        egui::ComboBox::from_id_salt("mat_add_layer").width(90.0).selected_text(shown).show_ui(ui, |ui| {
                            for i in &free {
                                ui.selectable_value(&mut dlg.add_index, *i, format!("C{i:02}"));
                            }
                        });
                        if ui.add_enabled(!free.is_empty(), egui::Button::new(tr("Add layer"))).clicked() {
                            todo = Some(Do::AddLayer(dlg.add_index));
                        }
                    });
                });
            });
        });
        // Settings of the layers that were edited in place.
        if edited != before {
            self.doc.device.materials[dlg.sel] = edited.clone();
            self.doc.apply_material_edit(dlg.sel);
            self.sync_profile();
        }
        if let Some(d) = todo {
            self.materials_do(d, &mut dlg, edited);
        }
        if open {
            self.materials_dlg = Some(dlg);
        }
    }

    /// Carry out what a click in the window asked for.
    fn materials_do(&mut self, d: Do, dlg: &mut MaterialsDlg, mut edited: Material) {
        dlg.message.clear();
        match d {
            Do::Select(i) => dlg.sel = i,
            Do::New => {
                self.checkpoint();
                // A new material is selected for the design too (as with the + of the Layers tab); the window follows it.
                let i = self.doc.add_material(&tr("Material"));
                dlg.sel = i;
                self.sync_profile();
            }
            Do::Duplicate => {
                self.checkpoint();
                if let Some(i) = self.doc.duplicate_material(dlg.sel) {
                    dlg.sel = i;
                    self.sync_profile();
                }
            }
            Do::Delete => {
                self.checkpoint();
                if self.doc.delete_material_at(dlg.sel) {
                    dlg.sel = dlg.sel.min(self.doc.device.materials.len() - 1);
                    dlg.loaded = None;
                    self.sync_profile();
                }
            }
            Do::Move(up) => {
                dlg.sel = self.doc.move_material(dlg.sel, up);
                self.sync_profile();
            }
            Do::Rename => {
                self.doc.rename_material_at(dlg.sel, &dlg.name);
                self.doc.device.materials[dlg.sel].color = dlg.colour;
                dlg.loaded = None;
                self.sync_profile();
            }
            Do::AddLayer(i) => {
                if edited.add_layer(i) {
                    self.doc.device.materials[dlg.sel] = edited;
                    self.doc.apply_material_edit(dlg.sel);
                    self.sync_profile();
                }
            }
            Do::RemoveLayer(i) => {
                if edited.remove_layer(i) {
                    self.doc.device.materials[dlg.sel] = edited;
                    self.doc.apply_material_edit(dlg.sel);
                    self.sync_profile();
                }
            }
            Do::Export(all) => {
                let list: Vec<Material> = if all { self.doc.device.materials.clone() } else { vec![self.doc.device.materials[dlg.sel].clone()] };
                let file = if all { "materials.json".to_string() } else { format!("{}.json", list[0].name.replace(|c: char| !c.is_alphanumeric() && c != ' ' && c != '-' && c != '_', "")) };
                if let Some(path) = rfd::FileDialog::new().add_filter("Materials", &["json"]).set_file_name(&file).save_file() {
                    dlg.message = match std::fs::write(&path, materials::export_materials(&list, crate::app::APP_VERSION)) {
                        Ok(()) => trf("Exported {} material(s): {}", &[&list.len(), &path.display()]),
                        Err(e) => trf("Cannot use the file: {}", &[&e]),
                    };
                }
            }
            Do::Import => {
                if let Some(path) = rfd::FileDialog::new().add_filter("Materials", &["json"]).pick_file() {
                    dlg.message = self.import_materials_from(&path, dlg.replace);
                    dlg.loaded = None;
                }
            }
        }
    }

    /// Read a file of materials into the list. Returns the message to show.
    pub fn import_materials_from(&mut self, path: &std::path::Path, replace: bool) -> String {
        let read = match std::fs::metadata(path) {
            Ok(m) if m.len() as usize > materials::MAX_MATERIALS_FILE => Err(format!("the file is larger than {} KB", materials::MAX_MATERIALS_FILE / 1000)),
            Ok(_) => std::fs::read_to_string(path).map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        };
        match read.and_then(|t| materials::import_materials(&t)) {
            Ok(list) => {
                self.checkpoint();
                // The selected material stays the one that is selected, whatever happens to the list.
                self.doc.sync_material();
                let active_name = self.doc.device.materials[self.doc.device.active_material].name.clone();
                let (added, replaced) = materials::merge_imported(&mut self.doc.device.materials, list, replace);
                if let Some(i) = self.doc.device.materials.iter().position(|m| m.name == active_name) {
                    self.doc.select_material(i);
                }
                self.sync_profile();
                trf("Imported {} material(s); {} replaced.", &[&added, &replaced])
            }
            Err(e) => trf("Cannot use the file: {}", &[&e]),
        }
    }
}

fn self_name(m: &Material) -> &str {
    m.name.trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.profiles = vec![lc_core::Device::default()];
        a.enter_editor(0);
        a.screen = crate::app::Screen::Editor;
        a
    }

    #[test]
    fn a_file_of_materials_is_read_into_the_list_and_the_selected_material_stays_selected() {
        let mut a = app();
        a.doc.device.materials = vec![Material::new("Plywood"), Material::new("Paper")];
        a.doc.device.active_material = 1;
        let mut other = Material::new("Plywood");
        other.color = Some([9, 9, 9]);
        let mut felt = Material::new("Felt");
        felt.add_layer(2);
        let dir = std::env::temp_dir().join(format!("lc-materials-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("m.json");
        std::fs::write(&file, materials::export_materials(&[other, felt], "v0")).unwrap();
        let msg = a.import_materials_from(&file, false);
        assert!(msg.contains("2 material(s); 0 replaced"), "{msg}");
        let names: Vec<&str> = a.doc.device.materials.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(names, vec!["Plywood", "Paper", "Plywood (2)", "Felt"]);
        assert_eq!(a.doc.device.materials[a.doc.device.active_material].name, "Paper", "the selected material stays");
        // With replace the one with the same name is replaced.
        let again = a.import_materials_from(&file, true);
        assert!(again.contains("0 material(s); 2 replaced"), "{again}");
        assert_eq!(a.doc.device.materials[0].color, Some([9, 9, 9]));
        // A broken file changes nothing and says why.
        std::fs::write(&file, "nonsense").unwrap();
        let n = a.doc.device.materials.len();
        assert!(!a.import_materials_from(&file, false).is_empty());
        assert_eq!(a.doc.device.materials.len(), n);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
