use crate::app::{App, SideTab};
use crate::units_ui::{drag_len, drag_speed_min, drag_speed_s};
use crate::i18n::{tr, trf};
use crate::laser::{self, Cmd, ConsoleLine, Dir};
use crate::theme;
use eframe::egui::{self, Color32, RichText};
use lc_core::{LayerMode, Xf, PALETTE};

impl App {
    pub fn side_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            for (t, label) in SideTab::ALL {
                if ui.selectable_label(self.side_tab == t, RichText::new(tr(label)).strong()).clicked() {
                    self.side_tab = t;
                }
            }
        });
        ui.separator();
        match self.side_tab {
            SideTab::Properties => {
                egui::ScrollArea::vertical().id_salt("tab_props").auto_shrink([false, false]).show(ui, |ui| self.properties(ui));
            }
            SideTab::Layers => {
                egui::ScrollArea::vertical().id_salt("tab_layers").auto_shrink([false, false]).show(ui, |ui| self.layers_panel(ui));
            }
            SideTab::Device => {
                egui::ScrollArea::vertical().id_salt("tab_device").auto_shrink([false, false]).show(ui, |ui| self.laser_panel(ui));
            }
            SideTab::Console => self.console_panel(ui),
        }
    }

    fn properties(&mut self, ui: &mut egui::Ui) {
        let Some(b) = self.sel_bounds() else {
            ui.label(RichText::new(tr("Nothing selected")).color(theme::text_dim()));
            return;
        };
        let units = self.doc.device.units;
        let (mut x, mut y, mut w, mut h) = (b.min.x, b.min.y, b.width(), b.height());
        let mut changed = None;
        egui::Grid::new("props").num_columns(4).spacing([6.0, 6.0]).show(ui, |ui| {
            let f = |ui: &mut egui::Ui, label: &str, v: &mut f64| -> egui::Response {
                ui.label(label);
                drag_len(ui, units, v, 0.1, None)
            };
            let rx = f(ui, "X", &mut x);
            let ry = f(ui, "Y", &mut y);
            ui.end_row();
            let rw = f(ui, "W", &mut w);
            let rh = f(ui, "H", &mut h);
            ui.end_row();
            for (r, which) in [(&rx, 0), (&ry, 1), (&rw, 2), (&rh, 3)] {
                if r.drag_started() || r.gained_focus() {
                    self.checkpoint();
                }
                if r.changed() {
                    changed = Some(which);
                }
            }
        });
        ui.checkbox(&mut self.lock_aspect, tr("Lock aspect ratio"));
        if let Some(which) = changed {
            let t = match which {
                0 | 1 => Xf::translate(x - b.min.x, y - b.min.y),
                _ => {
                    let (mut sx, mut sy) = (if b.width() > 1e-9 { w / b.width() } else { 1.0 }, if b.height() > 1e-9 { h / b.height() } else { 1.0 });
                    if self.lock_aspect {
                        if which == 2 {
                            sy = sx;
                        } else {
                            sx = sy;
                        }
                    }
                    Xf::scale_about(sx.max(1e-4), sy.max(1e-4), b.min)
                }
            };
            self.transform_selection(t);
        }
        ui.horizontal(|ui| {
            ui.label(tr("Rotate"));
            ui.add(egui::DragValue::new(&mut self.rot_input).suffix("°"));
            if ui.button(tr("Apply")).clicked() {
                self.rotate_sel(self.rot_input);
            }
            if ui.button("90°").clicked() {
                self.rotate_sel(90.0);
            }
        });
        ui.horizontal(|ui| {
            if ui.button(tr("Flip H")).clicked() {
                self.flip(true);
            }
            if ui.button(tr("Flip V")).clicked() {
                self.flip(false);
            }
            if ui.button(tr("Centre")).clicked() {
                self.center_on_bed();
            }
        });
        ui.horizontal_wrapped(|ui| {
            for (i, n) in [tr("Left"), tr("Mid-H"), tr("Right"), tr("Top"), tr("Mid-V"), tr("Bottom")].iter().enumerate() {
                let tips = [tr("Align left"), tr("Align centre"), tr("Align right"), tr("Align top"), tr("Align middle"), tr("Align bottom")];
                if ui.button(*n).on_hover_text(tips[i]).clicked() {
                    self.align(i as u8);
                }
            }
        });
        // Layer of selection
        let first = self.doc.shape(self.sel[0]).map(|s| s.layer).unwrap_or(0);
        let mut layer = first;
        ui.horizontal(|ui| {
            ui.label(tr("Layer"));
            egui::ComboBox::from_id_salt("sel_layer").selected_text(self.doc.layers[layer].name.clone()).show_ui(ui, |ui| {
                for i in 0..30 {
                    ui.selectable_value(&mut layer, i, self.doc.layers[i].name.clone());
                }
            });
        });
        if layer != first {
            self.assign_layer(layer);
        }
    }

    fn layers_panel(&mut self, ui: &mut egui::Ui) {
        let units = self.doc.device.units;
        let before = self.doc.layers.clone();
        let used: Vec<bool> = (0..30).map(|i| self.doc.shapes.iter().any(|s| s.layer == i)).collect();
        ui.checkbox(&mut self.show_all_layers, tr("Show all 30 layers"));
        for i in 0..30 {
            if !(used[i] || self.show_all_layers || i == self.active_layer) {
                continue;
            }
            let c = PALETTE[i];
            ui.horizontal(|ui| {
                let (r, resp) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::click());
                ui.painter().rect_filled(r, 3.0, Color32::from_rgb(c[0], c[1], c[2]));
                if self.active_layer == i {
                    ui.painter().rect_stroke(r.expand(1.5), 4.0, egui::Stroke::new(2.0, theme::accent()), egui::StrokeKind::Outside);
                }
                if resp.clicked() {
                    self.active_layer = i;
                }
                let l = &mut self.doc.layers[i];
                if ui.selectable_label(self.active_layer == i, &l.name).clicked() {
                    self.active_layer = i;
                }
                egui::ComboBox::from_id_salt(("mode", i)).width(78.0).selected_text(tr(l.mode.label())).show_ui(ui, |ui| {
                    for m in LayerMode::ALL {
                        ui.selectable_value(&mut l.mode, m, tr(m.label()));
                    }
                });
                drag_speed_s(ui, units, &mut l.speed, 0.5, Some((0.5, 1000.0)));
                ui.add(egui::DragValue::new(&mut l.power).range(0.0..=100.0).suffix(" %"));
                ui.checkbox(&mut l.output, "").on_hover_text(tr("Output (burn this layer)"));
                ui.checkbox(&mut l.visible, "").on_hover_text(tr("Visible"));
            });
        }
        ui.separator();
        let l = &mut self.doc.layers[self.active_layer];
        ui.label(RichText::new(trf("Cut settings — {}", &[&l.name])).strong());
        egui::Grid::new("cut").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
            ui.label(tr("Mode"));
            egui::ComboBox::from_id_salt("cm").selected_text(tr(l.mode.label())).show_ui(ui, |ui| {
                for m in LayerMode::ALL {
                    ui.selectable_value(&mut l.mode, m, tr(m.label()));
                }
            });
            ui.end_row();
            ui.label(tr("Speed"));
            drag_speed_s(ui, units, &mut l.speed, 0.5, Some((0.5, 1000.0)));
            ui.end_row();
            ui.label(tr("Power (%)"));
            ui.add(egui::Slider::new(&mut l.power, 0.0..=100.0));
            ui.end_row();
            ui.label(tr("Passes"));
            ui.add(egui::DragValue::new(&mut l.passes).range(1..=100));
            ui.end_row();
            if l.mode != LayerMode::Line {
                ui.label(tr("Interval"));
                drag_len(ui, units, &mut l.interval, 0.005, Some((0.01, 5.0)));
                ui.end_row();
                ui.label(tr("Scan angle"));
                ui.add(egui::DragValue::new(&mut l.angle).range(-180.0..=180.0).suffix("°"));
                ui.end_row();
                ui.label(tr("Overscan"));
                drag_len(ui, units, &mut l.overscan, 0.1, Some((0.0, 20.0)));
                ui.end_row();
                ui.label(tr("Bidirectional"));
                ui.checkbox(&mut l.bidirectional, "");
                ui.end_row();
            }
        });
        if self.doc.layers != before {
            let now = ui.input(|i| i.time);
            if now - self.last_layer_undo > 0.8 {
                let mut snap = self.doc.clone();
                snap.layers = before;
                self.undo.push(snap);
                self.redo.clear();
            }
            self.last_layer_undo = now;
            self.touch();
        }
    }

    fn laser_panel(&mut self, ui: &mut egui::Ui) {
        let units = self.doc.device.units;
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("port").width(150.0).selected_text(if self.port.is_empty() { tr("No port") } else { self.port.as_str() }).show_ui(ui, |ui| {
                for p in self.ports.clone() {
                    ui.selectable_value(&mut self.port, p.clone(), p);
                }
            });
            if ui.button(tr("Refresh")).on_hover_text(tr("Rescan serial ports")).clicked() {
                self.ports = laser::list_ports();
                if !self.ports.contains(&self.port) {
                    self.port = self.ports.first().cloned().unwrap_or_default();
                }
            }
            if self.connected {
                if ui.button(tr("Disconnect")).clicked() {
                    self.link.send(Cmd::Disconnect);
                }
            } else if ui.add_enabled(!self.port.is_empty(), egui::Button::new(tr("Connect"))).clicked() {
                self.link.send(Cmd::Connect { port: self.port.clone(), baud: self.doc.device.baud });
            }
        });
        ui.label(format!("{}: {}   X {:.2}  Y {:.2}", tr("State"), self.machine.0, self.machine.1, self.machine.2));
        ui.add_enabled_ui(self.connected, |ui| {
            ui.horizontal(|ui| {
                ui.label(tr("Step"));
                drag_len(ui, units, &mut self.jog_step, 0.1, Some((0.1, 200.0)));
                ui.label(tr("Feed"));
                drag_speed_min(ui, units, &mut self.jog_feed, 10.0, Some((10.0, 20000.0)));
            });
            let (s, f) = (self.jog_step, self.jog_feed);
            let jog = |app: &App, dx: f64, dy: f64| app.link.send(Cmd::Line(format!("$J=G91 G21 X{dx} Y{dy} F{f}")));
            egui::Grid::new("jog").spacing([4.0, 4.0]).show(ui, |ui| {
                ui.label("");
                if ui.button(format!("  {}  ", tr("Up"))).clicked() {
                    jog(self, 0.0, s);
                }
                ui.label("");
                ui.end_row();
                if ui.button(format!(" {} ", tr("Left"))).clicked() {
                    jog(self, -s, 0.0);
                }
                if ui.button(format!("  {}  ", tr("Stop"))).on_hover_text(tr("Cancel jog")).clicked() {
                    self.link.send(Cmd::Realtime(0x85));
                }
                if ui.button(format!(" {} ", tr("Right"))).clicked() {
                    jog(self, s, 0.0);
                }
                ui.end_row();
                ui.label("");
                if ui.button(format!(" {} ", tr("Down"))).clicked() {
                    jog(self, 0.0, -s);
                }
                ui.end_row();
            });
            ui.horizontal_wrapped(|ui| {
                if ui.button(tr("Home $H")).clicked() {
                    self.link.send(Cmd::Line("$H".into()));
                }
                if ui.button(tr("Unlock $X")).clicked() {
                    self.link.send(Cmd::Line("$X".into()));
                }
                if ui.button(tr("Pause")).clicked() {
                    self.link.send(Cmd::Realtime(b'!'));
                }
                if ui.button(tr("Resume")).clicked() {
                    self.link.send(Cmd::Realtime(b'~'));
                }
                if ui.add(egui::Button::new(RichText::new(tr("STOP")).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0xc0, 0x30, 0x30))).clicked() {
                    self.link.send(Cmd::Abort);
                }
            });
            ui.horizontal(|ui| {
                ui.label(tr("Frame power"));
                ui.add(egui::DragValue::new(&mut self.frame_power).range(0.0..=10.0).suffix(" %")).on_hover_text(tr("0 % keeps the laser off while framing; 1–2 % shows a dim dot on diode lasers"));
            });
        });
    }

    /// Live view of everything crossing the serial port.
    fn console_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut self.console_polls, tr("Status polls"));
            ui.checkbox(&mut self.console_autoscroll, tr("Auto-scroll"));
            if ui.button(tr("Clear")).clicked() {
                self.console.clear();
            }
        });
        let accent_tx = Color32::from_rgb(0x3b, 0x82, 0xd8);
        let accent_rx = Color32::from_rgb(0x2e, 0xa0, 0x4f);
        let show_polls = self.console_polls;
        let rows: Vec<&ConsoleLine> = self.console.iter().filter(|l| show_polls || !l.poll).collect();
        let row_h = 14.0;
        let input_h = 34.0;
        let h = (ui.available_height() - input_h).max(80.0);
        egui::ScrollArea::vertical().id_salt("console").max_height(h).auto_shrink([false, false]).stick_to_bottom(self.console_autoscroll).show_rows(ui, row_h, rows.len(), |ui, range| {
            for l in &rows[range] {
                let (prefix, color) = match l.dir {
                    Dir::Tx => ("> ", accent_tx),
                    Dir::Rx => ("< ", accent_rx),
                    Dir::Info => ("# ", theme::text_dim()),
                };
                ui.label(RichText::new(format!("{prefix}{}", l.text)).monospace().size(11.0).color(color));
            }
        });
        ui.horizontal(|ui| {
            let r = ui.add(egui::TextEdit::singleline(&mut self.console_input).desired_width(ui.available_width() - 70.0).hint_text(tr("send G-code / $ command")));
            let send = ui.button(tr("Send")).clicked() || (r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
            if send && self.connected && !self.console_input.trim().is_empty() {
                self.link.send(Cmd::Line(self.console_input.trim().to_string()));
                self.console_input.clear();
                r.request_focus();
            }
        });
    }
}
