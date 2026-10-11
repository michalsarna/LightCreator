//! The materials of a device in the Layers tab: a header names the selected material and lets you choose another,
//! add one (a copy of the current), rename or delete it. The layers below are the layers of that material.
use crate::app::App;
use crate::i18n::tr;
use crate::theme;
use eframe::egui::{self, Color32, RichText};
use lc_core::materials::DEFAULT_MATERIAL;

/// A material name as shown: the name every device starts with is translated, the others are the user's own words.
pub fn material_label(name: &str) -> String {
    match name {
        DEFAULT_MATERIAL => tr("Default material"),
        "Plywood 3 mm" => tr("Plywood 3 mm"),
        "Acrylic 3 mm" => tr("Acrylic 3 mm"),
        "Cardboard" => tr("Cardboard"),
        "Leather 2 mm" => tr("Leather 2 mm"),
        "Paper" => tr("Paper"),
        other => return other.to_string(),
    }
    .to_string()
}

/// What the header is doing besides showing the name.
#[derive(Clone, Debug, PartialEq)]
pub enum MatEdit {
    New(String),
    Rename(String),
}

impl App {
    /// Make the selected material the one with this index: the layers take its settings.
    pub fn choose_material(&mut self, i: usize) {
        if i == self.doc.device.active_material {
            return;
        }
        self.checkpoint();
        self.doc.select_material(i);
        self.sync_profile();
        self.status = crate::i18n::trf("Material: {}", &[&material_label(&self.doc.device.material().name)]);
    }

    /// Keep the selected material in step with the layers (the layers of a project are the material's layers).
    pub fn material_tick(&mut self) {
        if self.screen != crate::app::Screen::Editor || self.mat_rev == self.revision {
            return;
        }
        self.mat_rev = self.revision;
        if self.doc.sync_material() {
            self.sync_profile();
        }
    }

    /// The heading of the Layers tab.
    pub fn material_header(&mut self, ui: &mut egui::Ui) {
        let names: Vec<String> = self.doc.device.materials.iter().map(|m| material_label(&m.name)).collect();
        let active = self.doc.device.active_material.min(names.len().saturating_sub(1));
        let mut pick: Option<usize> = None;
        let mut edit = self.mat_edit.take();
        let mut commit = false;
        let mut cancel = false;
        let mut delete = false;
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width((ui.available_width() - 16.0).max(60.0));
            ui.horizontal(|ui| {
                ui.add(crate::icons::slot(Some("package"), theme::text())).on_hover_text(tr("Material"));
                egui::ComboBox::from_id_salt("material_pick").width((ui.available_width() - 110.0).clamp(60.0, 260.0)).selected_text(names.get(active).cloned().unwrap_or_default()).show_ui(ui, |ui| {
                    for (i, n) in names.iter().enumerate() {
                        if ui.selectable_label(i == active, n).clicked() {
                            pick = Some(i);
                        }
                    }
                });
                if ui.add(egui::Button::image(crate::icons::slot(Some("plus"), theme::text()))).on_hover_text(tr("New, empty material")).clicked() {
                    edit = Some(MatEdit::New(String::new()));
                }
                if ui.add(egui::Button::image(crate::icons::slot(Some("pencil"), theme::text()))).on_hover_text(tr("Rename this material")).clicked() {
                    edit = Some(MatEdit::Rename(names.get(active).cloned().unwrap_or_default()));
                }
                let can = names.len() > 1;
                if ui.add_enabled_ui(can, |ui| crate::window_ui::delete_icon(ui, &tr("Delete this material"))).inner.clicked() {
                    delete = true;
                }
            });
            if let Some(e) = &mut edit {
                let (text, hint) = match e {
                    MatEdit::New(t) => (t, tr("Name of the new material")),
                    MatEdit::Rename(t) => (t, tr("New name")),
                };
                ui.horizontal(|ui| {
                    let r = ui.add(egui::TextEdit::singleline(text).hint_text(hint).desired_width((ui.available_width() - 130.0).max(60.0)));
                    let ok = !text.trim().is_empty();
                    let go = egui::Button::new(RichText::new(tr("Save")).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0x2e, 0xa0, 0x4f));
                    if ui.add_enabled(ok, go).clicked() || (r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && ok) {
                        commit = true;
                    }
                    if ui.button(tr("Cancel")).clicked() {
                        cancel = true;
                    }
                });
            }
        });
        if let Some(i) = pick {
            self.choose_material(i);
        }
        if delete {
            self.checkpoint();
            self.doc.delete_material();
            self.sync_profile();
        }
        if commit {
            match &edit {
                Some(MatEdit::New(t)) => {
                    self.checkpoint();
                    self.doc.add_material(t);
                }
                Some(MatEdit::Rename(t)) => self.doc.rename_material(t),
                None => {}
            }
            self.sync_profile();
            edit = None;
        }
        if cancel {
            edit = None;
        }
        self.mat_edit = edit;
    }

    /// A project was opened with its device replaced by the profile: its material joins the library of the profile
    /// (or is found there) and is selected, without touching the layers of the project.
    pub fn adopt_project_material(&mut self, incoming: lc_core::materials::Material) {
        let same = |a: &lc_core::materials::Material, b: &lc_core::materials::Material| b.layers.iter().all(|bl| a.layer(bl.index).is_some_and(|al| al.settings.same_settings(&bl.settings)));
        let lib = &self.doc.device.materials;
        let found = lib.iter().position(|m| m.name == incoming.name && same(m, &incoming));
        let i = match found {
            Some(i) => i,
            None => {
                let names: Vec<String> = lib.iter().map(|m| m.name.clone()).collect();
                let mut m = incoming;
                m.name = lc_core::materials::free_name(&names, &if names.iter().any(|n| *n == m.name) { format!("{} ({})", m.name, tr("project")) } else { m.name.clone() });
                self.doc.device.materials.push(m);
                self.doc.device.materials.len() - 1
            }
        };
        self.doc.device.active_material = i;
        self.doc.sync_material();
        self.sync_profile();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lc_core::{Kind, Xf};

    fn app() -> App {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.profiles = vec![lc_core::Device::default()];
        a.enter_editor(0);
        a.screen = crate::app::Screen::Editor;
        a
    }

    #[test]
    fn the_default_material_name_is_translated_and_user_names_are_kept() {
        assert_eq!(material_label("Birch plywood"), "Birch plywood");
        assert_eq!(material_label(DEFAULT_MATERIAL), tr("Default material"));
    }

    #[test]
    fn layers_follow_the_chosen_material_and_the_profile_keeps_the_materials() {
        let mut a = app();
        a.doc.add(3, Kind::Rect { w: 10.0, h: 10.0 }, Xf::IDENTITY);
        a.touch();
        a.material_tick();
        assert!(a.profiles[0].material().layer(3).is_some(), "a used layer joined the material of the profile");
        a.doc.layers[3].speed = 9.0;
        a.touch();
        a.material_tick();
        assert_eq!(a.profiles[0].material().layer(3).unwrap().settings.speed, 9.0);
        // A new material, edited on its own.
        a.doc.add_material("Acrylic");
        a.sync_profile();
        a.doc.layers[3].speed = 3.0;
        a.touch();
        a.material_tick();
        assert_eq!(a.profiles[0].materials[1].layer(3).unwrap().settings.speed, 3.0);
        assert_eq!(a.profiles[0].materials[0].layer(3).unwrap().settings.speed, 9.0);
        // Choosing the first one brings its settings back to the layers.
        a.choose_material(0);
        assert_eq!(a.doc.layers[3].speed, 9.0);
        assert_eq!(a.profiles[0].active_material, 0);
        // A new document starts with the layers of the selected material.
        a.new_doc();
        assert_eq!(a.doc.device.active_material, 0);
    }

    #[test]
    fn a_new_device_comes_with_a_material_and_a_new_document_uses_it() {
        let mut a = app();
        assert_eq!(a.doc.device.materials.len(), 1);
        a.doc.device.materials[0].layers.push(lc_core::materials::MaterialLayer { index: 4, settings: { let mut l = lc_core::Layer::new(4); l.speed = 11.0; l } });
        a.sync_profile();
        a.new_doc();
        assert_eq!(a.doc.layers[4].speed, 11.0);
    }

    #[test]
    fn a_project_brings_its_material_into_the_library_of_the_device() {
        let mut a = app();
        let mut inc = lc_core::materials::Material::new("Leather");
        inc.layers.push(lc_core::materials::MaterialLayer { index: 1, settings: { let mut l = lc_core::Layer::new(1); l.power = 40.0; l } });
        // The layers of the opened project are the ones of its material.
        a.doc.layers[1].power = 40.0;
        a.adopt_project_material(inc.clone());
        assert_eq!(a.doc.device.material().name, "Leather");
        assert_eq!(a.doc.device.materials.len(), 2);
        // The same material again is found, not added twice.
        a.adopt_project_material(inc.clone());
        assert_eq!(a.doc.device.materials.len(), 2);
        // The same name with other settings is added under another name.
        inc.layers[0].settings.power = 90.0;
        a.doc.layers[1].power = 90.0;
        a.adopt_project_material(inc);
        assert_eq!(a.doc.device.materials.len(), 3);
        assert_ne!(a.doc.device.material().name, "Leather");
    }
}
