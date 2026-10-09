//! Start screen (device chooser) and the device configuration window.
//! The editor can only be entered once at least one device profile exists and one is selected.
use crate::app::{App, Screen, APP_VERSION};
use crate::i18n::{tr, trf, Lang};
use crate::menu::Act;
use crate::theme::{self, Scheme};
use eframe::egui::{self, Align, Color32, Layout, RichText};
use crate::units_ui::{drag_len, drag_speed_min, fmt_len};
use lc_core::{Device, Origin, Units};

/// A device profile being edited in the configuration window.
pub struct CfgEdit {
    /// `None` while creating a new profile.
    pub idx: Option<usize>,
    pub draft: Device,
}

impl App {
    /// Switch to the editor with the selected profile. Refuses (returns false) without a valid profile.
    pub fn enter_editor(&mut self, idx: usize) -> bool {
        let Some(dev) = self.profiles.get(idx).cloned() else { return false };
        self.active = idx;
        self.doc.device = dev;
        self.touch();
        self.view.need_fit = true;
        self.screen = Screen::Editor;
        true
    }

    pub fn open_cfg_new(&mut self) {
        let mut draft = Device::default();
        draft.name = String::new();
        self.cfg = Some(CfgEdit { idx: None, draft });
    }

    pub fn open_cfg_edit(&mut self, idx: usize) {
        if let Some(d) = self.profiles.get(idx) {
            self.cfg = Some(CfgEdit { idx: Some(idx), draft: d.clone() });
        }
    }

    pub fn start_screen(&mut self, ui: &mut egui::Ui) {
        // No profile yet: the user must create one before anything else.
        if self.profiles.is_empty() && self.cfg.is_none() {
            self.open_cfg_new();
        }
        let ctx = ui.ctx().clone();
        egui::CentralPanel::default().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space((ui.available_height() * 0.06).max(12.0));
                if let Some(logo) = &self.logo {
                    ui.image((logo.id(), egui::vec2(96.0, 96.0)));
                }
                ui.label(RichText::new("LightCreator").size(28.0).strong());
                ui.label(RichText::new(APP_VERSION).color(theme::text_dim()));
                ui.add_space(18.0);
                ui.label(RichText::new(tr("Select device")).size(16.0).strong());
                ui.label(RichText::new(tr("Choose the device you will work with.")).color(theme::text_dim()));
                ui.add_space(8.0);
                let frame = egui::Frame::group(ui.style()).inner_margin(egui::Margin::same(12));
                frame.show(ui, |ui| {
                    ui.set_width(460.0);
                    ui.with_layout(Layout::top_down(Align::LEFT), |ui| self.device_list(ui));
                });
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    // centre the two combo boxes
                    ui.add_space(((ui.available_width() - 360.0) / 2.0).max(0.0));
                    ui.label(tr("Language"));
                    let mut lang = self.lang;
                    egui::ComboBox::from_id_salt("start_lang").selected_text(lang.name()).show_ui(ui, |ui| {
                        for l in Lang::ALL {
                            ui.selectable_value(&mut lang, l, l.name());
                        }
                    });
                    if lang != self.lang {
                        self.do_act(&ctx, Act::SetLang(lang));
                    }
                    ui.add_space(12.0);
                    ui.label(tr("Colour scheme"));
                    let mut sc = self.scheme;
                    egui::ComboBox::from_id_salt("start_scheme").selected_text(tr(sc.label())).show_ui(ui, |ui| {
                        for s in Scheme::ALL {
                            ui.selectable_value(&mut sc, s, tr(s.label()));
                        }
                    });
                    if sc != self.scheme {
                        self.do_act(&ctx, Act::SetScheme(sc));
                    }
                });
            });
        });
    }

    fn device_list(&mut self, ui: &mut egui::Ui) {
        if self.profiles.is_empty() {
            ui.label(RichText::new(tr("Create at least one device profile to continue.")).color(theme::text_dim()));
        }
        self.start_sel = self.start_sel.min(self.profiles.len().saturating_sub(1));
        let mut go = false;
        egui::ScrollArea::vertical().max_height(220.0).auto_shrink([false, true]).show(ui, |ui| {
            for i in 0..self.profiles.len() {
                let d = &self.profiles[i];
                let text = format!("{}    {} × {}", d.name, fmt_len(d.units, d.bed_w, 0), fmt_len(d.units, d.bed_h, 0));
                let r = ui.add_sized([ui.available_width(), 28.0], egui::Button::selectable(self.start_sel == i, text));
                if r.clicked() {
                    self.start_sel = i;
                    self.confirm_delete = false;
                }
                if r.double_clicked() {
                    go = true;
                }
            }
        });
        ui.add_space(8.0);
        let has = !self.profiles.is_empty();
        ui.horizontal(|ui| {
            let cont = egui::Button::new(RichText::new(format!("  {}  ", tr("Continue"))).color(Color32::WHITE).strong()).fill(theme::accent());
            if ui.add_enabled(has, cont).clicked() {
                go = true;
            }
            if ui.button(tr("Add device…")).clicked() {
                self.open_cfg_new();
            }
            if ui.add_enabled(has, egui::Button::new(tr("Edit…"))).clicked() {
                self.open_cfg_edit(self.start_sel);
            }
            if self.confirm_delete && has {
                if ui.button(RichText::new(tr("Confirm delete")).color(Color32::from_rgb(0xc0, 0x30, 0x30))).clicked() {
                    self.profiles.remove(self.start_sel);
                    self.confirm_delete = false;
                    self.active = 0;
                }
                if ui.button(tr("Cancel")).clicked() {
                    self.confirm_delete = false;
                }
            } else if ui.add_enabled(has, egui::Button::new(tr("Delete"))).clicked() {
                self.confirm_delete = true;
            }
        });
        if go {
            self.enter_editor(self.start_sel);
        }
    }

    /// Device configuration window (create or edit a profile). Closable only when a profile exists.
    pub fn config_window(&mut self, ctx: &egui::Context) {
        // "Device settings…" from the editor edits the active profile.
        if self.show_device {
            self.show_device = false;
            if self.cfg.is_none() {
                self.open_cfg_edit(self.active);
            }
        }
        let Some(mut cfg) = self.cfg.take() else { return };
        let must_create = self.profiles.is_empty();
        let title = if cfg.idx.is_some() { tr("Device configuration") } else { tr("New device") };
        let mut save = false;
        let mut cancel = false;
        egui::Window::new(title).collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
            if must_create {
                ui.label(RichText::new(tr("Create at least one device profile to continue.")).color(theme::text_dim()));
                ui.add_space(4.0);
            }
            let d = &mut cfg.draft;
            egui::Grid::new("dev").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                ui.label(tr("Name"));
                ui.add(egui::TextEdit::singleline(&mut d.name).hint_text(tr("e.g. My diode laser")));
                ui.end_row();
                ui.label(tr("Units"));
                egui::ComboBox::from_id_salt("units").selected_text(tr(d.units.label())).show_ui(ui, |ui| {
                    for u in Units::ALL {
                        ui.selectable_value(&mut d.units, u, tr(u.label()));
                    }
                });
                ui.end_row();
                let u = d.units;
                ui.label(tr("Work area X"));
                drag_len(ui, u, &mut d.bed_w, 1.0, Some((10.0, 5000.0)));
                ui.end_row();
                ui.label(tr("Work area Y"));
                drag_len(ui, u, &mut d.bed_h, 1.0, Some((10.0, 5000.0)));
                ui.end_row();
                ui.label(tr("Machine zero (0,0)"));
                egui::ComboBox::from_id_salt("origin")
                    .selected_text(if d.origin == Origin::FrontLeft { tr("Front-left (GRBL default)") } else { tr("Back-left") })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut d.origin, Origin::FrontLeft, tr("Front-left (GRBL default)"));
                        ui.selectable_value(&mut d.origin, Origin::BackLeft, tr("Back-left"));
                    });
                ui.end_row();
                ui.label(tr("S-value max ($30)"));
                ui.add(egui::DragValue::new(&mut d.s_max).range(1.0..=100000.0));
                ui.end_row();
                ui.label(tr("Dynamic power (M4)"));
                ui.checkbox(&mut d.dynamic_power, "");
                ui.end_row();
                ui.label(tr("Travel speed"));
                drag_speed_min(ui, u, &mut d.travel_speed, 50.0, Some((100.0, 60000.0)));
                ui.end_row();
                ui.label(tr("Return to origin"));
                ui.checkbox(&mut d.return_home, "");
                ui.end_row();
                ui.label(tr("Baud rate"));
                ui.add(egui::DragValue::new(&mut d.baud).range(1200..=1_000_000));
                ui.end_row();
            });
            ui.add_space(8.0);
            let valid = !cfg.draft.name.trim().is_empty();
            if !valid {
                ui.label(RichText::new(tr("A profile name is required.")).color(Color32::from_rgb(0xc0, 0x30, 0x30)));
            }
            ui.horizontal(|ui| {
                if ui.add_enabled(valid, egui::Button::new(tr("Save"))).clicked() {
                    save = true;
                }
                if !must_create && ui.button(tr("Cancel")).clicked() {
                    cancel = true;
                }
            });
        });
        if save {
            cfg.draft.name = cfg.draft.name.trim().to_string();
            let idx = match cfg.idx {
                Some(i) if i < self.profiles.len() => {
                    self.profiles[i] = cfg.draft.clone();
                    i
                }
                _ => {
                    self.profiles.push(cfg.draft.clone());
                    self.profiles.len() - 1
                }
            };
            self.start_sel = idx;
            if self.screen == Screen::Editor && idx == self.active {
                self.doc.device = cfg.draft;
                self.touch();
            }
        } else if !cancel {
            self.cfg = Some(cfg);
        }
    }

    /// Short description for the status bar / logs.
    #[allow(dead_code)]
    pub fn active_name(&self) -> String {
        trf("Device: {}", &[&self.profiles.get(self.active).map(|d| d.name.clone()).unwrap_or_default()])
    }
}
