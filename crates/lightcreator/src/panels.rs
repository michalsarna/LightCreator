use crate::app::App;
use crate::laser::{self, Cmd};
use crate::theme;
use eframe::egui::{self, Color32, RichText};
use lc_core::{LayerMode, Xf, PALETTE};

impl App {
    pub fn side_panel(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            egui::CollapsingHeader::new(RichText::new("Properties").strong()).default_open(true).show(ui, |ui| self.properties(ui));
            egui::CollapsingHeader::new(RichText::new("Cuts / Layers").strong()).default_open(true).show(ui, |ui| self.layers_panel(ui));
            egui::CollapsingHeader::new(RichText::new("Laser").strong()).default_open(true).show(ui, |ui| self.laser_panel(ui));
        });
    }

    fn properties(&mut self, ui: &mut egui::Ui) {
        let Some(b) = self.sel_bounds() else {
            ui.label(RichText::new("Nothing selected").color(theme::TEXT_DIM));
            return;
        };
        let (mut x, mut y, mut w, mut h) = (b.min.x, b.min.y, b.width(), b.height());
        let mut changed = None;
        egui::Grid::new("props").num_columns(4).spacing([6.0, 6.0]).show(ui, |ui| {
            let f = |ui: &mut egui::Ui, label: &str, v: &mut f64| -> egui::Response {
                ui.label(label);
                ui.add(egui::DragValue::new(v).speed(0.1).suffix(" mm").max_decimals(3))
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
        ui.checkbox(&mut self.lock_aspect, "Lock aspect ratio");
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
            ui.label("Rotate");
            ui.add(egui::DragValue::new(&mut self.rot_input).suffix("°"));
            if ui.button("Apply").clicked() {
                self.rotate_sel(self.rot_input);
            }
            if ui.button("90°").clicked() {
                self.rotate_sel(90.0);
            }
        });
        ui.horizontal(|ui| {
            if ui.button("Flip H").clicked() {
                self.flip(true);
            }
            if ui.button("Flip V").clicked() {
                self.flip(false);
            }
            if ui.button("Centre").clicked() {
                self.center_on_bed();
            }
        });
        ui.horizontal_wrapped(|ui| {
            for (i, n) in ["Left", "Mid-H", "Right", "Top", "Mid-V", "Bottom"].iter().enumerate() {
                let tips = ["Align left", "Align centre", "Align right", "Align top", "Align middle", "Align bottom"];
                if ui.button(*n).on_hover_text(tips[i]).clicked() {
                    self.align(i as u8);
                }
            }
        });
        // Layer of selection
        let first = self.doc.shape(self.sel[0]).map(|s| s.layer).unwrap_or(0);
        let mut layer = first;
        ui.horizontal(|ui| {
            ui.label("Layer");
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
        let before = self.doc.layers.clone();
        let used: Vec<bool> = (0..30).map(|i| self.doc.shapes.iter().any(|s| s.layer == i)).collect();
        ui.checkbox(&mut self.show_all_layers, "Show all 30 layers");
        for i in 0..30 {
            if !(used[i] || self.show_all_layers || i == self.active_layer) {
                continue;
            }
            let c = PALETTE[i];
            ui.horizontal(|ui| {
                let (r, resp) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::click());
                ui.painter().rect_filled(r, 3.0, Color32::from_rgb(c[0], c[1], c[2]));
                if self.active_layer == i {
                    ui.painter().rect_stroke(r.expand(1.5), 4.0, egui::Stroke::new(2.0, theme::ACCENT), egui::StrokeKind::Outside);
                }
                if resp.clicked() {
                    self.active_layer = i;
                }
                let l = &mut self.doc.layers[i];
                if ui.selectable_label(self.active_layer == i, &l.name).clicked() {
                    self.active_layer = i;
                }
                egui::ComboBox::from_id_salt(("mode", i)).width(78.0).selected_text(l.mode.label()).show_ui(ui, |ui| {
                    for m in LayerMode::ALL {
                        ui.selectable_value(&mut l.mode, m, m.label());
                    }
                });
                ui.add(egui::DragValue::new(&mut l.speed).range(0.5..=1000.0).suffix(" mm/s"));
                ui.add(egui::DragValue::new(&mut l.power).range(0.0..=100.0).suffix(" %"));
                ui.checkbox(&mut l.output, "").on_hover_text("Output (burn this layer)");
                ui.checkbox(&mut l.visible, "").on_hover_text("Visible");
            });
        }
        ui.separator();
        let l = &mut self.doc.layers[self.active_layer];
        ui.label(RichText::new(format!("Cut settings — {}", l.name)).strong());
        egui::Grid::new("cut").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
            ui.label("Mode");
            egui::ComboBox::from_id_salt("cm").selected_text(l.mode.label()).show_ui(ui, |ui| {
                for m in LayerMode::ALL {
                    ui.selectable_value(&mut l.mode, m, m.label());
                }
            });
            ui.end_row();
            ui.label("Speed (mm/s)");
            ui.add(egui::DragValue::new(&mut l.speed).range(0.5..=1000.0));
            ui.end_row();
            ui.label("Power (%)");
            ui.add(egui::Slider::new(&mut l.power, 0.0..=100.0));
            ui.end_row();
            ui.label("Passes");
            ui.add(egui::DragValue::new(&mut l.passes).range(1..=100));
            ui.end_row();
            if l.mode != LayerMode::Line {
                ui.label("Interval (mm)");
                ui.add(egui::DragValue::new(&mut l.interval).range(0.01..=5.0).speed(0.005));
                ui.end_row();
                ui.label("Scan angle");
                ui.add(egui::DragValue::new(&mut l.angle).range(-180.0..=180.0).suffix("°"));
                ui.end_row();
                ui.label("Overscan (mm)");
                ui.add(egui::DragValue::new(&mut l.overscan).range(0.0..=20.0).speed(0.1));
                ui.end_row();
                ui.label("Bidirectional");
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
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("port").width(150.0).selected_text(if self.port.is_empty() { "No port" } else { &self.port }).show_ui(ui, |ui| {
                for p in self.ports.clone() {
                    ui.selectable_value(&mut self.port, p.clone(), p);
                }
            });
            if ui.button("Refresh").on_hover_text("Rescan serial ports").clicked() {
                self.ports = laser::list_ports();
                if !self.ports.contains(&self.port) {
                    self.port = self.ports.first().cloned().unwrap_or_default();
                }
            }
            if self.connected {
                if ui.button("Disconnect").clicked() {
                    self.link.send(Cmd::Disconnect);
                }
            } else if ui.add_enabled(!self.port.is_empty(), egui::Button::new("Connect")).clicked() {
                self.link.send(Cmd::Connect { port: self.port.clone(), baud: self.doc.device.baud });
            }
        });
        ui.label(format!("State: {}   X {:.2}  Y {:.2}", self.machine.0, self.machine.1, self.machine.2));
        ui.add_enabled_ui(self.connected, |ui| {
            ui.horizontal(|ui| {
                ui.label("Step");
                ui.add(egui::DragValue::new(&mut self.jog_step).range(0.1..=200.0).suffix(" mm"));
                ui.label("Feed");
                ui.add(egui::DragValue::new(&mut self.jog_feed).range(10.0..=20000.0));
            });
            let (s, f) = (self.jog_step, self.jog_feed);
            let jog = |app: &App, dx: f64, dy: f64| app.link.send(Cmd::Line(format!("$J=G91 G21 X{dx} Y{dy} F{f}")));
            egui::Grid::new("jog").spacing([4.0, 4.0]).show(ui, |ui| {
                ui.label("");
                if ui.button("  Up  ").clicked() {
                    jog(self, 0.0, s);
                }
                ui.label("");
                ui.end_row();
                if ui.button(" Left ").clicked() {
                    jog(self, -s, 0.0);
                }
                if ui.button("  Stop  ").on_hover_text("Cancel jog").clicked() {
                    self.link.send(Cmd::Realtime(0x85));
                }
                if ui.button(" Right ").clicked() {
                    jog(self, s, 0.0);
                }
                ui.end_row();
                ui.label("");
                if ui.button(" Down ").clicked() {
                    jog(self, 0.0, -s);
                }
                ui.end_row();
            });
            ui.horizontal_wrapped(|ui| {
                if ui.button("Home $H").clicked() {
                    self.link.send(Cmd::Line("$H".into()));
                }
                if ui.button("Unlock $X").clicked() {
                    self.link.send(Cmd::Line("$X".into()));
                }
                if ui.button("Pause").clicked() {
                    self.link.send(Cmd::Realtime(b'!'));
                }
                if ui.button("Resume").clicked() {
                    self.link.send(Cmd::Realtime(b'~'));
                }
                if ui.add(egui::Button::new(RichText::new("STOP").color(Color32::WHITE).strong()).fill(Color32::from_rgb(0xc0, 0x30, 0x30))).clicked() {
                    self.link.send(Cmd::Abort);
                }
            });
            ui.horizontal(|ui| {
                ui.label("Frame power");
                ui.add(egui::DragValue::new(&mut self.frame_power).range(0.0..=10.0).suffix(" %")).on_hover_text("0 % keeps the laser off while framing; 1–2 % shows a dim dot on diode lasers");
            });
        });
        ui.separator();
        egui::ScrollArea::vertical().id_salt("console").max_height(150.0).stick_to_bottom(true).show(ui, |ui| {
            for l in &self.console {
                ui.label(RichText::new(l).monospace().size(11.0).color(theme::TEXT_DIM));
            }
        });
        ui.horizontal(|ui| {
            let r = ui.add(egui::TextEdit::singleline(&mut self.console_input).desired_width(200.0).hint_text("send G-code / $ command"));
            let send = ui.button("Send").clicked() || (r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
            if send && self.connected && !self.console_input.trim().is_empty() {
                self.link.send(Cmd::Line(self.console_input.trim().to_string()));
                self.console_input.clear();
                r.request_focus();
            }
        });
    }
}
