//! Start screen (device chooser) and the device configuration window.
//! The editor can only be entered once at least one device profile exists and one is selected.
use crate::app::{App, Screen, APP_VERSION};
use crate::laser::{self, Cmd};
use crate::i18n::{tr, trf, Lang};
use crate::menu::Act;
use crate::theme::{self, Scheme};
use eframe::egui::{self, Align, Color32, Layout, RichText};
use crate::units_ui::{drag_len, drag_speed_min, fmt_len, fmt_speed_min};
use lc_core::controller::Action;
use lc_core::{Controller, Device, LaserKind, Origin, Units};

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
                ui.label(tr("Controller"));
                egui::ComboBox::from_id_salt("controller").selected_text(d.controller.label()).show_ui(ui, |ui| {
                    for c in Controller::ALL {
                        ui.selectable_value(&mut d.controller, c, c.label());
                    }
                });
                ui.end_row();
                ui.label(tr("Laser type"));
                egui::ComboBox::from_id_salt("laser_kind").selected_text(d.laser.label()).show_ui(ui, |ui| {
                    for k in LaserKind::ALL {
                        ui.selectable_value(&mut d.laser, k, k.label());
                    }
                });
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
                if d.controller.is_serial() {
                    ui.label(tr("Port"));
                    ui.horizontal(|ui| {
                        egui::ComboBox::from_id_salt("cfg_port").width(170.0).selected_text(if d.port.is_empty() { tr("No port") } else { d.port.as_str() }).show_ui(ui, |ui| {
                            for p in &self.ports {
                                ui.selectable_value(&mut d.port, p.clone(), p);
                            }
                        });
                        if ui.button(tr("Refresh")).clicked() {
                            self.ports = laser::list_ports();
                        }
                    });
                    ui.end_row();
                }
                ui.label(tr("Jog step"));
                drag_len(ui, u, &mut d.jog_step, 0.1, Some((0.1, 200.0)));
                ui.end_row();
                ui.label(tr("Jog feed"));
                drag_speed_min(ui, u, &mut d.jog_feed, 10.0, Some((10.0, 20000.0)));
                ui.end_row();
                ui.label(tr("Frame power"));
                ui.add(egui::DragValue::new(&mut d.frame_power).range(0.0..=10.0).suffix(" %")).on_hover_text(tr("0 % keeps the laser off while framing; 1–2 % shows a dim dot on diode lasers"));
                ui.end_row();
            });
            ui.add_space(8.0);
            let valid = !cfg.draft.name.trim().is_empty();
            if !cfg.draft.controller.is_serial() {
                ui.label(RichText::new(tr("This controller has no live connection: jobs are exported as files.")).color(theme::text_dim()));
            }
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

    /// Make `idx` the active profile (disconnects first).
    pub fn switch_profile(&mut self, idx: usize) {
        let Some(dev) = self.profiles.get(idx).cloned() else { return };
        if self.connected {
            self.link.send(Cmd::Disconnect);
        }
        self.active = idx;
        self.start_sel = idx;
        self.doc.device = dev;
        self.touch();
    }

    /// Keep the stored profile in step with live edits made in the Device tab.
    pub fn sync_profile(&mut self) {
        if let Some(p) = self.profiles.get_mut(self.active) {
            if *p != self.doc.device {
                *p = self.doc.device.clone();
            }
        }
    }

    pub fn run_action(&mut self, a: Action) {
        match a {
            Action::Lines(ls) => {
                for l in ls {
                    self.link.send(Cmd::Line(l));
                }
            }
            Action::Byte(b) => self.link.send(Cmd::Realtime(b)),
            Action::None => {}
        }
    }

    /// The Device tab: choose a device, see its settings, connect and move the machine.
    pub fn device_tab(&mut self, ui: &mut egui::Ui) {
        let units = self.doc.device.units;
        let ctrl = self.doc.device.controller;
        ui.horizontal(|ui| {
            ui.label(tr("Device"));
            let mut sel = self.active;
            egui::ComboBox::from_id_salt("dev_sel")
                .width(190.0)
                .selected_text(self.profiles.get(self.active).map(|d| d.name.clone()).unwrap_or_default())
                .show_ui(ui, |ui| {
                    for (i, d) in self.profiles.iter().enumerate() {
                        ui.selectable_value(&mut sel, i, &d.name);
                    }
                });
            if sel != self.active {
                self.switch_profile(sel);
            }
        });
        ui.horizontal(|ui| {
            if ui.button(tr("Edit…")).clicked() {
                self.open_cfg_edit(self.active);
            }
            if ui.button(tr("Add device…")).clicked() {
                self.open_cfg_new();
            }
            if ui.button(tr("Switch device…")).clicked() {
                self.start_sel = self.active;
                self.confirm_delete = false;
                self.screen = Screen::Start;
            }
        });
        ui.add_space(4.0);
        let dev = self.doc.device.clone();
        let yes_no = |b: bool| if b { tr("Yes") } else { tr("No") };
        egui::Grid::new("dev_summary").num_columns(2).spacing([12.0, 3.0]).striped(true).show(ui, |ui| {
            let mut row = |k: &str, v: String| {
                ui.label(RichText::new(k).color(theme::text_dim()));
                ui.label(v);
                ui.end_row();
            };
            row(tr("Controller"), dev.controller.label().to_string());
            row(tr("Laser type"), dev.laser.label().to_string());
            row(tr("Units"), tr(dev.units.label()).to_string());
            row(tr("Work area"), format!("{} × {}", fmt_len(units, dev.bed_w, 0), fmt_len(units, dev.bed_h, 0)));
            row(tr("Machine zero (0,0)"), if dev.origin == Origin::FrontLeft { tr("Front-left (GRBL default)") } else { tr("Back-left") }.to_string());
            row(tr("S-value max ($30)"), format!("{}", dev.s_max));
            row(tr("Dynamic power (M4)"), yes_no(dev.dynamic_power).to_string());
            row(tr("Travel speed"), fmt_speed_min(units, dev.travel_speed));
            row(tr("Return to origin"), yes_no(dev.return_home).to_string());
            row(tr("Baud rate"), dev.baud.to_string());
        });
        ui.add_space(6.0);
        ui.separator();
        ui.label(RichText::new(tr("Connection")).strong());
        if !ctrl.is_serial() {
            ui.label(RichText::new(tr("This controller has no live connection: jobs are exported as files.")).color(theme::text_dim()));
            ui.label(RichText::new(tr("Export PLT or DXF and open it in the controller's own software (for example RDWorks for Ruida), which sets speed and power per layer colour.")).color(theme::text_dim()));
            if ui.button(tr("Export PLT / DXF…")).clicked() {
                self.export_cam();
            }
            self.sync_profile();
            return;
        }
        let mut port = self.doc.device.port.clone();
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("port").width(150.0).selected_text(if port.is_empty() { tr("No port") } else { port.as_str() }).show_ui(ui, |ui| {
                for p in self.ports.clone() {
                    ui.selectable_value(&mut port, p.clone(), p);
                }
            });
            if ui.button(tr("Refresh")).on_hover_text(tr("Rescan serial ports")).clicked() {
                self.ports = laser::list_ports();
                if !self.ports.contains(&port) {
                    port = self.ports.first().cloned().unwrap_or_default();
                }
            }
            if self.connected {
                if ui.button(tr("Disconnect")).clicked() {
                    self.link.send(Cmd::Disconnect);
                }
            } else if ui.add_enabled(!port.is_empty(), egui::Button::new(tr("Connect"))).clicked() {
                self.link.send(Cmd::Connect { port: port.clone(), baud: self.doc.device.baud, controller: ctrl });
            }
        });
        self.doc.device.port = port;
        ui.label(format!("{}: {}   X {:.2}  Y {:.2}", tr("State"), self.machine.0, self.machine.1, self.machine.2));
        let mut act: Option<Action> = None;
        ui.add_enabled_ui(self.connected, |ui| {
            ui.horizontal(|ui| {
                ui.label(tr("Jog step"));
                drag_len(ui, units, &mut self.doc.device.jog_step, 0.1, Some((0.1, 200.0)));
                ui.label(tr("Jog feed"));
                drag_speed_min(ui, units, &mut self.doc.device.jog_feed, 10.0, Some((10.0, 20000.0)));
            });
            let (s, f) = (self.doc.device.jog_step, self.doc.device.jog_feed);
            egui::Grid::new("jog").spacing([4.0, 4.0]).show(ui, |ui| {
                ui.label("");
                if ui.button(format!("  {}  ", tr("Up"))).clicked() {
                    act = Some(ctrl.jog(0.0, s, f));
                }
                ui.label("");
                ui.end_row();
                if ui.button(format!(" {} ", tr("Left"))).clicked() {
                    act = Some(ctrl.jog(-s, 0.0, f));
                }
                if ui.button(format!("  {}  ", tr("Stop"))).on_hover_text(tr("Cancel jog")).clicked() {
                    act = Some(ctrl.cancel_jog());
                }
                if ui.button(format!(" {} ", tr("Right"))).clicked() {
                    act = Some(ctrl.jog(s, 0.0, f));
                }
                ui.end_row();
                ui.label("");
                if ui.button(format!(" {} ", tr("Down"))).clicked() {
                    act = Some(ctrl.jog(0.0, -s, f));
                }
                ui.end_row();
            });
            ui.horizontal_wrapped(|ui| {
                if ui.button(tr("Home")).clicked() {
                    act = Some(ctrl.home());
                }
                if ui.button(tr("Unlock")).clicked() {
                    act = Some(ctrl.unlock());
                }
                if ui.button(tr("Pause")).clicked() {
                    act = Some(ctrl.pause());
                }
                if ui.button(tr("Resume")).clicked() {
                    act = Some(ctrl.resume());
                }
                if ui.add(egui::Button::new(RichText::new(tr("STOP")).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0xc0, 0x30, 0x30))).clicked() {
                    self.link.send(Cmd::Abort);
                }
            });
            ui.horizontal(|ui| {
                ui.label(tr("Frame power"));
                ui.add(egui::DragValue::new(&mut self.doc.device.frame_power).range(0.0..=10.0).suffix(" %")).on_hover_text(tr("0 % keeps the laser off while framing; 1–2 % shows a dim dot on diode lasers"));
            });
        });
        if let Some(a) = act {
            self.run_action(a);
        }
        self.sync_profile();
    }
}
