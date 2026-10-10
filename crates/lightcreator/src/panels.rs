use crate::app::{App, SideTab};
use crate::units_ui::{drag_len, drag_speed};
use crate::i18n::{tr, trf};
use crate::laser::{Cmd, ConsoleLine, Dir};
use crate::theme;
use eframe::egui::{self, Color32, RichText};
use lc_core::{Dither, Kind, LayerMode, TextData, Xf, PALETTE};

impl App {
    pub fn side_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            let mut tabs: Vec<(SideTab, &'static str)> = SideTab::ALL.to_vec();
            let camera_tab = self.show_stream && self.stream_mode == crate::stream_ui::StreamMode::Tab;
            if camera_tab {
                tabs.push((SideTab::Camera, "Camera view"));
            }
            if self.side_tab == SideTab::Camera && !camera_tab {
                self.side_tab = SideTab::Properties;
            }
            for (t, label) in tabs {
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
                egui::ScrollArea::vertical().id_salt("tab_device").auto_shrink([false, false]).show(ui, |ui| self.device_tab(ui));
            }
            SideTab::Console => self.console_panel(ui),
            SideTab::Clipart => self.clipart_panel(ui),
            SideTab::Camera => {
                egui::ScrollArea::vertical().id_salt("tab_camera").auto_shrink([false, false]).show(ui, |ui| self.stream_view(ui));
            }
        }
    }

    fn properties(&mut self, ui: &mut egui::Ui) {
        let picked: Vec<_> = self.sel.iter().filter_map(|id| self.doc.shape(*id)).collect();
        let locked = !picked.is_empty() && picked.iter().all(|s| s.locked);
        if locked {
            ui.horizontal_wrapped(|ui| {
                if let Some(img) = crate::icons::image("lock", 15.0, theme::accent()) {
                    ui.add(img);
                }
                ui.label(RichText::new(tr("Locked: unlock to edit. Copies are not locked.")).color(theme::accent()));
            });
        }
        ui.add_enabled_ui(!locked, |ui| self.properties_inner(ui));
    }

    fn properties_inner(&mut self, ui: &mut egui::Ui) {
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
            let lc = PALETTE[self.doc.layers[layer].color.min(29)];
            let (r, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
            ui.painter().rect_filled(r, 3.0, Color32::from_rgb(lc[0], lc[1], lc[2]));
            ui.painter().rect_stroke(r, 3.0, egui::Stroke::new(1.0, theme::border()), egui::StrokeKind::Inside);
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
        if self.sel.len() == 1 {
            self.text_props(ui);
            self.image_props(ui);
        }
    }

    fn text_props(&mut self, ui: &mut egui::Ui) {
        let id = self.sel[0];
        let Some(Kind::Text(orig)) = self.doc.shape(id).map(|s| s.kind.clone()) else { return };
        let units = self.doc.device.units;
        let mut t: TextData = orig.clone();
        ui.separator();
        ui.label(RichText::new(tr("Text")).strong());
        let r = ui.add(egui::TextEdit::multiline(&mut t.text).desired_rows(3).desired_width(f32::INFINITY));
        if self.focus_text {
            r.request_focus();
            self.focus_text = false;
        }
        let mut undo_point = r.gained_focus();
        egui::Grid::new("text_props").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
            ui.label(tr("Font"));
            egui::ComboBox::from_id_salt("font_family").width(190.0).selected_text(t.family.clone()).show_ui(ui, |ui| {
                ui.add(egui::TextEdit::singleline(&mut self.font_filter).hint_text(tr("Search")));
                let q = self.font_filter.to_lowercase();
                egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                    for fam in lc_core::text::families() {
                        if (q.is_empty() || fam.to_lowercase().contains(&q)) && ui.selectable_label(t.family == fam, &fam).clicked() {
                            t.family = fam;
                            undo_point = true;
                        }
                    }
                });
            });
            ui.end_row();
            ui.label(tr("Size"));
            drag_len(ui, units, &mut t.size, 0.2, Some((0.5, 1000.0)));
            ui.end_row();
            ui.label(tr("Style"));
            ui.horizontal(|ui| {
                ui.checkbox(&mut t.bold, tr("Bold"));
                ui.checkbox(&mut t.italic, tr("Italic"));
            });
            ui.end_row();
            ui.label(tr("Letter spacing"));
            ui.add(egui::DragValue::new(&mut t.spacing).speed(0.005).range(-0.5..=3.0));
            ui.end_row();
            ui.label(tr("Line spacing"));
            ui.add(egui::DragValue::new(&mut t.line_spacing).speed(0.01).range(0.5..=5.0));
            ui.end_row();
            ui.label(tr("Align"));
            ui.horizontal(|ui| {
                for (i, n) in [tr("Left"), tr("Centre"), tr("Right")].into_iter().enumerate() {
                    ui.selectable_value(&mut t.align, i as u8, n);
                }
            });
            ui.end_row();
        });
        if ui.button(tr("Convert to curves")).on_hover_text(tr("Turn the text into editable Bézier shapes")).clicked() {
            self.to_curves();
            return;
        }
        if t != orig {
            if undo_point || t.text != orig.text {
                // One undo step per editing burst: the first change after focus gets a checkpoint.
                let now = ui.input(|i| i.time);
                if undo_point || now - self.last_layer_undo > 0.8 {
                    self.checkpoint();
                }
                self.last_layer_undo = now;
            }
            self.text_default = TextData { text: self.text_default.text.clone(), ..t.clone() };
            if let Some(s) = self.doc.unlocked_mut(id) {
                s.kind = Kind::Text(t);
            }
            self.touch();
        }
    }

    fn image_props(&mut self, ui: &mut egui::Ui) {
        let id = self.sel[0];
        let Some(Kind::Image(im)) = self.doc.shape(id).map(|s| s.kind.clone()) else { return };
        let units = self.doc.device.units;
        ui.separator();
        ui.label(RichText::new(tr("Image")).strong());
        ui.label(RichText::new(&im.name).color(theme::text_dim()));
        let dpi = im.px_w as f64 / (im.w / 25.4).max(1e-6);
        ui.label(format!("{} × {} px   {:.0} DPI   {} × {}", im.px_w, im.px_h, dpi, crate::units_ui::fmt_len(units, im.w, 1), crate::units_ui::fmt_len(units, im.h, 1)));
        let mut invert = im.invert;
        if ui.checkbox(&mut invert, tr("Negative image")).on_hover_text(tr("Engrave the light parts instead of the dark parts")).changed() {
            self.checkpoint();
            if let Some(Kind::Image(m)) = self.doc.unlocked_mut(id).map(|s| &mut s.kind) {
                m.invert = invert;
            }
        }
        ui.label(RichText::new(tr("Engraving uses the layer's interval, speed, power and dithering.")).color(theme::text_dim()));
    }

    fn layers_panel(&mut self, ui: &mut egui::Ui) {
        let (units, su) = (self.doc.device.units, self.doc.device.speed_unit);
        let before = self.doc.layers.clone();
        let used: Vec<bool> = (0..30).map(|i| self.doc.shapes.iter().any(|s| s.layer == i)).collect();
        ui.checkbox(&mut self.show_all_layers, tr("Show all 30 layers"));
        let order = self.doc.layer_order();
        let mut move_req: Option<(usize, bool)> = None;
        for &i in &order {
            if !(used[i] || self.show_all_layers || i == self.active_layer) {
                continue;
            }
            let c = PALETTE[i];
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 3.0;
                if ui.add(egui::Button::image(crate::icons::slot(Some("chevron-up"), theme::text())).small()).on_hover_text(tr("Burn earlier")).clicked() {
                    move_req = Some((i, true));
                }
                if ui.add(egui::Button::image(crate::icons::slot(Some("chevron-down"), theme::text())).small()).on_hover_text(tr("Burn later")).clicked() {
                    move_req = Some((i, false));
                }
                let (r, resp) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::click());
                ui.painter().rect_filled(r, 3.0, Color32::from_rgb(c[0], c[1], c[2]));
                if self.active_layer == i {
                    ui.painter().rect_stroke(r.expand(1.5), 4.0, egui::Stroke::new(2.0, theme::accent()), egui::StrokeKind::Outside);
                }
                if resp.clicked() {
                    self.active_layer = i;
                }
                let l = &mut self.doc.layers[i];
                let name = ui.selectable_label(self.active_layer == i, &l.name).on_hover_text(tr("Double-click to edit the layer options"));
                if name.clicked() {
                    self.active_layer = i;
                }
                if name.double_clicked() {
                    self.active_layer = i;
                    self.layer_dlg = Some(i);
                }
                egui::ComboBox::from_id_salt(("mode", i)).width(78.0).selected_text(tr(l.mode.label())).show_ui(ui, |ui| {
                    for m in LayerMode::ALL {
                        ui.selectable_value(&mut l.mode, m, tr(m.label()));
                    }
                });
                drag_speed(ui, units, su, false, &mut l.speed, 0.5, Some((0.5, 1000.0)));
                ui.add(egui::DragValue::new(&mut l.power).range(0.0..=100.0).suffix(" %"));
                ui.checkbox(&mut l.output, "").on_hover_text(tr("Output (burn this layer)"));
                ui.checkbox(&mut l.visible, "").on_hover_text(tr("Visible"));
            });
        }
        if let Some((i, up)) = move_req {
            self.checkpoint();
            self.doc.move_layer(i, up);
        }
        ui.label(RichText::new(tr("Layers are burnt from top to bottom.")).color(theme::text_dim()));
        ui.separator();
        let l = &mut self.doc.layers[self.active_layer];
        ui.label(RichText::new(trf("Cut settings — {}", &[&l.name])).strong());
        let panel_scope = format!("panel-{}", self.active_layer);
        layer_settings_ui(ui, (units, su), l, panel_scope.as_str());
        let now = ui.input(|i| i.time);
        self.commit_layer_edit(now, before);
    }

    /// Record an undo step (one per editing burst) and refresh when the layers changed.
    fn commit_layer_edit(&mut self, now: f64, before: Vec<lc_core::Layer>) {
        if self.doc.layers != before {
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

    /// Window with every option of one layer (opened by double-clicking its name).
    pub fn layer_dialog(&mut self, ctx: &egui::Context) {
        let Some(i) = self.layer_dlg else { return };
        let before = self.doc.layers.clone();
        let (units, su) = (self.doc.device.units, self.doc.device.speed_unit);
        let mut open = true;
        let title = trf("Layer options — {}", &[&self.doc.layers[i].name]);
        egui::Window::new(title).id(egui::Id::new("layer_dialog")).open(&mut open).collapsible(false).resizable(false).show(ctx, |ui| {
            let l = &mut self.doc.layers[i];
            egui::Grid::new("layer_head").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                ui.label(tr("Name"));
                ui.add(egui::TextEdit::singleline(&mut l.name).desired_width(140.0));
                ui.end_row();
                ui.label(tr("Colour"));
                let c = PALETTE[l.color.min(29)];
                let (r, _) = ui.allocate_exact_size(egui::vec2(34.0, 18.0), egui::Sense::hover());
                ui.painter().rect_filled(r, 3.0, Color32::from_rgb(c[0], c[1], c[2]));
                ui.end_row();
                ui.label(tr("Output (burn this layer)"));
                ui.checkbox(&mut l.output, "");
                ui.end_row();
                ui.label(tr("Visible"));
                ui.checkbox(&mut l.visible, "");
                ui.end_row();
            });
            ui.separator();
            layer_settings_ui(ui, (units, su), l, "dialog");
        });
        let now = ctx.input(|i| i.time);
        self.commit_layer_edit(now, before);
        if !open {
            self.layer_dlg = None;
        }
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

/// The cut settings of one layer: mode, speed, power, passes, fill options and image options.
/// `id_scope` keeps the widget ids apart when the same settings are shown in the panel and in the dialog.
pub fn layer_settings_ui(ui: &mut egui::Ui, (units, su): (lc_core::Units, lc_core::SpeedUnit), l: &mut lc_core::Layer, id_scope: &str) {
    egui::Grid::new(("cut", id_scope)).num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
        ui.label(tr("Mode"));
        egui::ComboBox::from_id_salt(("cm", id_scope)).selected_text(tr(l.mode.label())).show_ui(ui, |ui| {
            for m in LayerMode::ALL {
                ui.selectable_value(&mut l.mode, m, tr(m.label()));
            }
        });
        ui.end_row();
        ui.label(tr("Speed"));
        drag_speed(ui, units, su, false, &mut l.speed, 0.5, Some((0.5, 1000.0)));
        ui.end_row();
        ui.label(tr("Power (%)"));
        ui.add(egui::Slider::new(&mut l.power, 0.0..=100.0));
        ui.end_row();
        ui.label(tr("Passes"));
        ui.add(egui::DragValue::new(&mut l.passes).range(1..=100));
        ui.end_row();
        if l.mode == LayerMode::Offset {
            ui.label(tr("Interval"));
            drag_len(ui, units, &mut l.interval, 0.005, Some((0.01, 5.0)));
            ui.end_row();
        } else if l.mode != LayerMode::Line {
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
    egui::CollapsingHeader::new(tr("Image settings")).id_salt(("img_hdr", id_scope)).default_open(false).show(ui, |ui| {
        ui.label(RichText::new(tr("Images use the interval, speed, power and overscan of their layer.")).color(theme::text_dim()));
        egui::Grid::new(("img_cut", id_scope)).num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
            ui.label(tr("Dithering"));
            egui::ComboBox::from_id_salt(("dither", id_scope)).selected_text(tr(l.dither.label())).show_ui(ui, |ui| {
                for d in Dither::ALL {
                    ui.selectable_value(&mut l.dither, d, tr(d.label()));
                }
            });
            ui.end_row();
            ui.label(tr("Min power (%)"));
            let max = l.power;
            ui.add(egui::Slider::new(&mut l.min_power, 0.0..=max));
            ui.end_row();
        });
    });
}
