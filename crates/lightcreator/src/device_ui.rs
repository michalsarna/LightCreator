//! Start screen (device chooser) and the device configuration window.
//! The editor can only be entered once at least one device profile exists and one is selected.
use crate::app::{App, Screen, APP_VERSION};
use crate::laser::{self, Cmd, Target};
use crate::i18n::{tr, trf, Lang};
use crate::menu::Act;
use crate::theme::{self, Scheme};
use eframe::egui::{self, Align, Color32, Layout, RichText};
use crate::units_ui::{drag_len, drag_speed, fmt_len, fmt_speed};
use lc_core::controller::Action;
use lc_core::{Controller, Device, LaserKind, LinkKind, Origin, SpeedUnit, Units};

/// An in-progress "read settings from the device" request.
pub struct ReadCfg {
    pub controller: Controller,
    /// Waiting for the connection to come up before asking.
    pub waiting_connect: bool,
    /// When to send the request (a short pause after connecting lets the firmware finish resetting).
    pub request_at: Option<std::time::Instant>,
    pub sent_at: Option<std::time::Instant>,
    pub lines: Vec<String>,
    pub started: std::time::Instant,
}

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
        // An empty new document takes the layer speeds of the device; a loaded one keeps its own.
        if self.doc.shapes.is_empty() && self.path.is_none() {
            self.apply_layer_speeds();
        }
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
        let mut read_clicked = false;
        let reading = self.read_cfg.is_some();
        egui::Window::new(title).collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
            if must_create {
                ui.label(RichText::new(tr("Create at least one device profile to continue.")).color(theme::text_dim()));
                ui.add_space(4.0);
            }
            let max_h = (ui.ctx().content_rect().height() - 300.0).max(240.0);
            let d = &mut cfg.draft;
            let section = |ui: &mut egui::Ui, title: &str, add: &mut dyn FnMut(&mut egui::Ui)| {
                ui.label(RichText::new(title).strong());
                egui::Frame::group(ui.style()).inner_margin(egui::Margin::symmetric(10, 8)).show(ui, |ui| {
                    ui.set_width(430.0);
                    add(ui);
                });
                ui.add_space(4.0);
            };
            egui::ScrollArea::vertical().max_height(max_h).auto_shrink([true, true]).show(ui, |ui| {
            // ---- the machine ----
            section(ui, tr("Device"), &mut |ui| {
                egui::Grid::new("dev_machine").num_columns(2).min_col_width(150.0).spacing([12.0, 6.0]).show(ui, |ui| {
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
                    ui.label(tr("Speed units"));
                    egui::ComboBox::from_id_salt("speed_units").selected_text(tr(d.speed_unit.label())).show_ui(ui, |ui| {
                        for k in SpeedUnit::ALL {
                            ui.selectable_value(&mut d.speed_unit, k, tr(k.label()));
                        }
                    });
                    ui.end_row();
                    let (u, su) = (d.units, d.speed_unit);
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
                    drag_speed(ui, u, su, true, &mut d.travel_speed, 50.0, Some((100.0, 60000.0)));
                    ui.end_row();
                    ui.label(tr("Return to origin"));
                    ui.checkbox(&mut d.return_home, "");
                    ui.end_row();
                    ui.label(tr("Jog step"));
                    drag_len(ui, u, &mut d.jog_step, 0.1, Some((0.1, 200.0)));
                    ui.end_row();
                    ui.label(tr("Jog feed"));
                    drag_speed(ui, u, su, true, &mut d.jog_feed, 10.0, Some((10.0, 20000.0)));
                    ui.end_row();
                    ui.label(tr("Frame power"));
                    ui.add(egui::DragValue::new(&mut d.frame_power).range(0.0..=10.0).suffix(" %")).on_hover_text(tr("0 % keeps the laser off while framing; 1–2 % shows a dim dot on diode lasers"));
                    ui.end_row();
                });
            });
            // ---- the link to the controller ----
            if d.controller.is_serial() {
                section(ui, tr("Connection"), &mut |ui| {
                    egui::Grid::new("dev_link").num_columns(2).min_col_width(150.0).spacing([12.0, 6.0]).show(ui, |ui| {
                        ui.label(tr("Connection type"));
                        egui::ComboBox::from_id_salt("link_kind").width(230.0).selected_text(tr(d.link.label())).show_ui(ui, |ui| {
                            for k in LinkKind::ALL {
                                ui.selectable_value(&mut d.link, k, tr(k.label()));
                            }
                        });
                        ui.end_row();
                        if d.link == LinkKind::Serial {
                            ui.label(tr("Port"));
                            ui.horizontal_wrapped(|ui| {
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
                            ui.label(tr("Baud rate"));
                            ui.add(egui::DragValue::new(&mut d.baud).range(1200..=1_000_000));
                            ui.end_row();
                        } else {
                            ui.label(tr("Host"));
                            ui.add(egui::TextEdit::singleline(&mut d.host).hint_text("raspberrypi.local").desired_width(200.0));
                            ui.end_row();
                            ui.label(tr("TCP port"));
                            ui.add(egui::DragValue::new(&mut d.tcp_port).range(1..=65535));
                            ui.end_row();
                        }
                        ui.label("");
                        let label = if reading { tr("Reading…") } else { tr("Read from device") };
                        if ui.add_enabled(!reading && d.has_target(), egui::Button::new(label)).on_hover_text(tr("Connect and read work area, S-value max and speed from the controller")).clicked() {
                            read_clicked = true;
                        }
                        ui.end_row();
                    });
                });
            }
            // ---- the camera ----
            section(ui, tr("Camera"), &mut |ui| {
                egui::Grid::new("dev_camera").num_columns(2).min_col_width(150.0).spacing([12.0, 6.0]).show(ui, |ui| {
                    ui.label(tr("Camera URL"));
                    ui.add(egui::TextEdit::singleline(&mut d.camera_url).hint_text("http://raspberrypi.local:8080/stream.mjpg").desired_width(260.0)).on_hover_text(tr("Live picture of the machine: an MJPEG stream or a JPEG snapshot address (http or https)."));
                    ui.end_row();
                    ui.label(tr("Camera rotation"));
                    ui.horizontal(|ui| {
                        for q in 0..4u8 {
                            ui.selectable_value(&mut d.camera_rotation, q, format!("{}°", q as u32 * 90));
                        }
                    });
                    ui.end_row();
                });
            });
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
        if read_clicked {
            self.start_read(&cfg.draft);
        }
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
        let half = ((ui.available_width() - 12.0) / 2.0).max(60.0);
        egui::Grid::new("dev_summary").num_columns(2).min_col_width(half).spacing([12.0, 3.0]).striped(true).show(ui, |ui| {
            let mut row = |k: &str, v: String| {
                ui.label(RichText::new(k).color(theme::text_dim()));
                ui.label(v);
                ui.end_row();
            };
            row(tr("Controller"), dev.controller.label().to_string());
            row(tr("Laser type"), dev.laser.label().to_string());
            row(tr("Units"), tr(dev.units.label()).to_string());
            row(tr("Speed units"), tr(dev.speed_unit.label()).to_string());
            row(tr("Work area"), format!("{} × {}", fmt_len(units, dev.bed_w, 0), fmt_len(units, dev.bed_h, 0)));
            row(tr("Machine zero (0,0)"), if dev.origin == Origin::FrontLeft { tr("Front-left (GRBL default)") } else { tr("Back-left") }.to_string());
            row(tr("S-value max ($30)"), format!("{}", dev.s_max));
            row(tr("Dynamic power (M4)"), yes_no(dev.dynamic_power).to_string());
            row(tr("Travel speed"), fmt_speed(units, dev.speed_unit, true, dev.travel_speed));
            row(tr("Return to origin"), yes_no(dev.return_home).to_string());
            if dev.controller.is_serial() {
                row(tr("Connection type"), tr(dev.link.label()).to_string());
                if dev.link == LinkKind::Serial {
                    row(tr("Baud rate"), dev.baud.to_string());
                }
                row(tr("Port"), if dev.has_target() { dev.target_label() } else { tr("No port").to_string() });
            }
            if !dev.camera_url.trim().is_empty() {
                row(tr("Camera URL"), dev.camera_url.clone());
                row(tr("Camera rotation"), format!("{}°", dev.camera_rotation as u32 % 4 * 90));
            }
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
        let tcp = self.doc.device.link == LinkKind::Tcp;
        ui.horizontal(|ui| {
            if tcp {
                ui.label(RichText::new(self.doc.device.target_label()).monospace());
            } else {
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
            }
            if self.connected {
                let purple = egui::Button::new(RichText::new(tr("Disconnect")).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0x8e, 0x44, 0xad));
                if ui.add(purple).clicked() {
                    self.link.send(Cmd::Disconnect);
                }
            } else {
                let mut dev = self.doc.device.clone();
                dev.port = port.clone();
                let green = egui::Button::new(RichText::new(tr("Connect")).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0x2e, 0xa0, 0x4f));
                if ui.add_enabled(dev.has_target(), green).clicked() {
                    if let Some(target) = Target::of(&dev) {
                        self.link.send(Cmd::Connect { target, controller: ctrl });
                    }
                }
            }
        });
        self.doc.device.port = port;
        ui.label(format!("{}: {}   X {:.2}  Y {:.2}", tr("State"), self.machine.0, self.machine.1, self.machine.2));
        let mut act: Option<Action> = None;
        // Options of the device that belong to jogging and framing, always editable.
        let running = self.job_running();
        ui.add_enabled_ui(!running, |ui| egui::Grid::new("jog_opts").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
            ui.label(tr("Jog step"));
            drag_len(ui, units, &mut self.doc.device.jog_step, 0.1, Some((0.1, 200.0)));
            ui.end_row();
            ui.label(tr("Jog feed"));
            drag_speed(ui, units, self.doc.device.speed_unit, true, &mut self.doc.device.jog_feed, 10.0, Some((10.0, 20000.0)));
            ui.end_row();
            ui.label(tr("Frame power"));
            ui.add(egui::DragValue::new(&mut self.doc.device.frame_power).range(0.0..=10.0).suffix(" %")).on_hover_text(tr("0 % keeps the laser off while framing; 1–2 % shows a dim dot on diode lasers"));
            ui.end_row();
        }));
        ui.separator();
        ui.add_enabled_ui(self.connected, |ui| {
            let (s, f) = (self.doc.device.jog_step, self.doc.device.jog_feed);
            // Arrow keys and the controls below are buttons of one size.
            let size = egui::vec2(60.0, 48.0);
            let icon = |ui: &mut egui::Ui, name: &str, tip: String, accent: Option<Color32>| -> bool {
                let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
                let hovered = resp.hovered() && ui.is_enabled();
                let fill = match (accent, hovered) {
                    (Some(c), true) => c.gamma_multiply(0.85),
                    (Some(c), false) => c,
                    (None, true) => theme::panel_dark(),
                    (None, false) => ui.visuals().widgets.inactive.weak_bg_fill,
                };
                ui.painter().rect_filled(rect, 4.0, fill);
                ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(1.0, theme::border()), egui::StrokeKind::Inside);
                let ink = if accent.is_some() { Color32::WHITE } else if ui.is_enabled() { theme::text() } else { theme::text().gamma_multiply(0.4) };
                crate::icons::paint(ui, egui::Rect::from_center_size(rect.center(), egui::vec2(27.0, 27.0)), name, ink);
                resp.on_hover_text(tip).clicked()
            };
            let centre = |ui: &mut egui::Ui, total: f32| ui.add_space(((ui.available_width() - total) / 2.0).max(0.0));
            // While a job runs the machine must not be moved by hand: only pause, resume and stop stay available.
            ui.add_enabled_ui(!running, |ui| ui.horizontal(|ui| {
            centre(ui, 3.0 * size.x + 2.0 * 4.0);
            egui::Grid::new("jog").spacing([4.0, 4.0]).show(ui, |ui| {
                ui.allocate_exact_size(size, egui::Sense::hover());
                if icon(ui, "arrow-up", tr("Up").to_string(), None) {
                    act = Some(ctrl.jog(0.0, s, f));
                }
                ui.allocate_exact_size(size, egui::Sense::hover());
                ui.end_row();
                if icon(ui, "arrow-left", tr("Left").to_string(), None) {
                    act = Some(ctrl.jog(-s, 0.0, f));
                }
                if icon(ui, "square", tr("Cancel jog").to_string(), Some(Color32::from_rgb(0xe8, 0x8a, 0x1f))) {
                    act = Some(ctrl.cancel_jog());
                }
                if icon(ui, "arrow-right", tr("Right").to_string(), None) {
                    act = Some(ctrl.jog(s, 0.0, f));
                }
                ui.end_row();
                ui.allocate_exact_size(size, egui::Sense::hover());
                if icon(ui, "arrow-down", tr("Down").to_string(), None) {
                    act = Some(ctrl.jog(0.0, -s, f));
                }
                ui.end_row();
            });
            }));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                centre(ui, 4.0 * size.x + 3.0 * ui.spacing().item_spacing.x);
                ui.add_enabled_ui(!running, |ui| {
                    if icon(ui, "house", tr("Home").to_string(), None) {
                        act = Some(ctrl.home());
                    }
                });
                ui.add_enabled_ui(!running, |ui| {
                    if icon(ui, "lock-open", tr("Unlock").to_string(), None) {
                        act = Some(ctrl.unlock());
                    }
                });
                if icon(ui, "pause", tr("Pause").to_string(), None) {
                    act = Some(ctrl.pause());
                }
                if icon(ui, "play", tr("Resume").to_string(), None) {
                    act = Some(ctrl.resume());
                }
            });
            ui.add_space(6.0);
            if ui.add_sized([ui.available_width(), 40.0], egui::Button::new(RichText::new(tr("STOP")).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0xc0, 0x30, 0x30))).clicked() {
                self.link.send(Cmd::Abort);
            }
        });
        if let Some(a) = act {
            self.run_action(a);
        }
        self.sync_profile();
    }

    /// Ask the controller for its settings (connecting first if needed).
    pub fn start_read(&mut self, draft: &Device) {
        let c = draft.controller;
        let Some(_) = c.settings_request() else { return };
        let mut st = ReadCfg { controller: c, waiting_connect: false, request_at: None, sent_at: None, lines: vec![], started: std::time::Instant::now() };
        if self.connected {
            st.request_at = Some(std::time::Instant::now());
        } else {
            st.waiting_connect = true;
            match Target::of(draft) {
                Some(target) => self.link.send(Cmd::Connect { target, controller: c }),
                None => return,
            }
        }
        self.read_cfg = Some(st);
        self.status = tr("Reading settings from the device…").to_string();
    }

    /// Called when the connection state changes.
    pub fn read_on_connected(&mut self, up: bool) {
        if let Some(r) = &mut self.read_cfg {
            if r.waiting_connect {
                if up {
                    r.waiting_connect = false;
                    r.request_at = Some(std::time::Instant::now() + std::time::Duration::from_millis(900));
                } else {
                    self.read_cfg = None;
                    self.status = tr("Could not connect to the device.").to_string();
                }
            }
        }
    }

    /// Collect response lines while a read is running.
    pub fn read_feed(&mut self, line: &str) {
        let Some(r) = &mut self.read_cfg else { return };
        if r.sent_at.is_none() {
            return;
        }
        if line == "ok" || line.starts_with("error") {
            self.finish_read();
        } else {
            r.lines.push(line.to_string());
        }
    }

    /// Send the request when due and give up after a timeout.
    pub fn read_tick(&mut self, ctx: &egui::Context) {
        let Some(r) = &mut self.read_cfg else { return };
        let now = std::time::Instant::now();
        if let Some(at) = r.request_at {
            if now >= at {
                r.request_at = None;
                r.sent_at = Some(now);
                if let Some(req) = r.controller.settings_request() {
                    self.link.send(Cmd::Line(req.to_string()));
                }
            }
        }
        let no_connection = r.waiting_connect && now.duration_since(r.started).as_secs_f32() > 6.0;
        let timed_out = r.sent_at.is_some_and(|t| now.duration_since(t).as_secs_f32() > 4.0);
        if no_connection {
            self.read_cfg = None;
            self.status = tr("Could not connect to the device.").to_string();
        } else if timed_out {
            self.finish_read();
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(150));
    }

    fn finish_read(&mut self) {
        let Some(r) = self.read_cfg.take() else { return };
        let reading = r.controller.parse_settings(&r.lines);
        let n = reading.count();
        if n == 0 {
            self.status = tr("The device did not report any settings.").to_string();
            return;
        }
        if let Some(cfg) = &mut self.cfg {
            let d = &mut cfg.draft;
            if let Some(v) = reading.bed_w {
                d.bed_w = v;
            }
            if let Some(v) = reading.bed_h {
                d.bed_h = v;
            }
            if let Some(v) = reading.s_max {
                d.s_max = v;
            }
            if let Some(v) = reading.travel_speed {
                d.travel_speed = v;
            }
            if let Some(v) = reading.dynamic_power {
                d.dynamic_power = v;
            }
        }
        self.status = trf("Read {} values from the device", &[&n]);
    }
}
