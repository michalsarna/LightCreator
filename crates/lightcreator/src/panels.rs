use base_ui::widgets::section_title;
use base_ui::clipart_ui::ClipGallery;
use crate::app::{App, SideTab};
use crate::units_ui::{drag_len, drag_speed};
use crate::i18n::{tr, trf};
use crate::laser::Cmd;
use crate::theme;
use eframe::egui::{self, Color32, RichText};
use lc_core::{Dither, Kind, LayerMode, TextData, PALETTE};

impl App {
    pub fn side_panel(&mut self, ui: &mut egui::Ui) {
        self.side_w = ui.max_rect().width();
        let mut tabs: Vec<(SideTab, &'static str)> = self.tab_order.clone();
        let camera_tab = self.show_stream && self.stream_mode == crate::stream_ui::StreamMode::Tab;
        if camera_tab {
            tabs.push((SideTab::Camera, "Camera view"));
        }
        if self.side_tab == SideTab::Camera && !camera_tab {
            self.side_tab = SideTab::Properties;
        }
        // Tabs can be dragged to another place; the order is kept in the settings.
        let mut moved: Option<(usize, usize)> = None;
        ui.horizontal_wrapped(|ui| {
            let mut rects: Vec<egui::Rect> = vec![];
            let mut dragging: Option<usize> = None;
            for (i, (t, label)) in tabs.iter().enumerate() {
                let b = egui::Button::selectable(self.side_tab == *t, RichText::new(tr(label)).strong()).sense(egui::Sense::click_and_drag());
                let r = ui.add(b);
                if r.clicked() {
                    self.side_tab = *t;
                }
                if r.dragged() {
                    dragging = Some(i);
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                }
                if r.drag_stopped() {
                    moved = Some((i, usize::MAX));
                }
                rects.push(r.rect);
            }
            // Where a dragged tab would land: next to the tab nearest to the pointer.
            let pointer = ui.input(|i| i.pointer.interact_pos());
            let dragged = dragging.or(moved.map(|m| m.0));
            if let (Some(from), Some(p)) = (dragged, pointer) {
                let dist = |r: &egui::Rect| r.distance_to_pos(p);
                let to = (0..rects.len()).filter(|i| *i < self.tab_order.len()).min_by(|a, b| dist(&rects[*a]).total_cmp(&dist(&rects[*b])));
                if let Some(to) = to {
                    if dragging.is_some() && to != from {
                        let x = if to > from { rects[to].right() + 2.0 } else { rects[to].left() - 2.0 };
                        ui.painter().line_segment([egui::pos2(x, rects[to].top()), egui::pos2(x, rects[to].bottom())], egui::Stroke::new(2.0, theme::accent()));
                    }
                    if let Some(m) = &mut moved {
                        m.1 = to;
                    }
                }
            }
        });
        if let Some((from, to)) = moved {
            if from < self.tab_order.len() && to < self.tab_order.len() && from != to {
                let t = self.tab_order.remove(from);
                self.tab_order.insert(to, t);
            }
        }
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
            SideTab::Description => self.description_panel(ui),
            SideTab::Camera => {
                egui::ScrollArea::vertical().id_salt("tab_camera").auto_shrink([false, false]).show(ui, |ui| self.stream_view(ui));
            }
        }
    }

    /// The description of the project, saved in the project file (the tab is the one of baseCreator).
    fn description_panel(&mut self, ui: &mut egui::Ui) {
        if base_ui::description_ui::panel(ui, &mut self.doc.description) {
            self.touch();
        }
    }

    fn properties(&mut self, ui: &mut egui::Ui) {
        let picked: Vec<_> = self.ed.sel.iter().filter_map(|id| self.doc.shape(*id)).collect();
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
        // Position and size, rotation, mirror and centre, alignment: the sections shared with MillCreator.
        if !base_ui::editor::properties_sections(self, ui) {
            ui.label(RichText::new(tr("Nothing selected")).color(theme::text_dim()));
            return;
        }
        ui.separator();
        // Layer of selection
        section_title(ui, "layers", &tr("Layer"));
        let first = self.doc.shape(self.ed.sel[0]).map(|s| s.layer).unwrap_or(0);
        let mut layer = first;
        ui.horizontal_wrapped(|ui| {
            let lc = PALETTE[self.doc.layers[layer].color.min(lc_core::LAYER_COUNT - 1)];
            let (r, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
            ui.painter().rect_filled(r, 3.0, Color32::from_rgb(lc[0], lc[1], lc[2]));
            ui.painter().rect_stroke(r, 3.0, egui::Stroke::new(1.0, theme::border()), egui::StrokeKind::Inside);
            ui.label(tr("Layer"));
            egui::ComboBox::from_id_salt("sel_layer").selected_text(self.doc.layers[layer].name.clone()).show_ui(ui, |ui| {
                for i in 0..lc_core::LAYER_COUNT {
                    ui.selectable_value(&mut layer, i, self.doc.layers[i].name.clone());
                }
            });
        });
        if layer != first {
            self.assign_layer(layer);
        }
        if self.ed.sel.len() == 1 {
            self.text_props(ui);
            self.image_props(ui);
        }
    }

    fn text_props(&mut self, ui: &mut egui::Ui) {
        let id = self.ed.sel[0];
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
        let id = self.ed.sel[0];
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
        let used: Vec<bool> = (0..lc_core::LAYER_COUNT).map(|i| self.doc.scene.shapes.iter().any(|s| s.layer == i)).collect();
        self.material_header(ui);
        ui.label(RichText::new(tr("Layers are burnt from top to bottom.")).color(theme::text_dim()));
        // Headings of the three switches at the end of each row.
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                for (icon, tip) in [("wind", "Air pump on"), ("eye", "Visible"), ("zap", "Output (burn this layer)")] {
                    ui.allocate_ui_with_layout(egui::vec2(SWITCH_W, 18.0), egui::Layout::top_down(egui::Align::Center), |ui| {
                        let r = ui.add(crate::icons::slot(Some(icon), theme::text_dim()));
                        r.on_hover_text(tr(tip));
                    });
                }
            });
        });
        let order = self.doc.layer_order();
        let mut move_req: Option<(usize, bool)> = None;
        base_ui::widgets::list_with_splitter(ui, "layers_list", 230.0, |ui| {
        for &i in &order {
            if !(used[i] || self.show_all_layers || i == self.ed.active_layer) {
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
                if self.ed.active_layer == i {
                    ui.painter().rect_stroke(r.expand(1.5), 4.0, egui::Stroke::new(2.0, theme::accent()), egui::StrokeKind::Outside);
                }
                if resp.clicked() {
                    self.ed.active_layer = i;
                }
                let l = &mut self.doc.layers[i];
                let name = ui.selectable_label(self.ed.active_layer == i, &l.name).on_hover_text(tr("Double-click to edit the layer options"));
                if name.clicked() {
                    self.ed.active_layer = i;
                }
                if name.double_clicked() {
                    self.ed.active_layer = i;
                    self.layer_dlg = Some(i);
                }
                drag_speed(ui, units, su, false, &mut l.speed, 0.5, Some((0.5, 1000.0)));
                ui.add(egui::DragValue::new(&mut l.power).range(0.0..=100.0).suffix(" %"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Right to left: air pump, visible, output.
                    for (flag, tip) in [(&mut l.air_assist, "Air pump on"), (&mut l.visible, "Visible"), (&mut l.output, "Output (burn this layer)")] {
                        ui.allocate_ui_with_layout(egui::vec2(SWITCH_W, 20.0), egui::Layout::top_down(egui::Align::Center), |ui| {
                            ui.checkbox(flag, "").on_hover_text(tr(tip));
                        });
                    }
                });
            });
        }
        });
        if let Some((i, up)) = move_req {
            self.checkpoint();
            self.doc.move_layer(i, up);
        }
        ui.checkbox(&mut self.show_all_layers, trf("Show all {} layers", &[&lc_core::LAYER_COUNT]));
        let has_images = self.doc.scene.shapes.iter().any(|s| s.layer == self.ed.active_layer && s.is_image());
        let mut layer_ctx = LayerCtx::of(&self.doc.device);
        layer_ctx.image_only = self.doc.is_image_layer(self.ed.active_layer);
        let l = &mut self.doc.layers[self.ed.active_layer];
        ui.horizontal(|ui| {
            let c = PALETTE[l.color.min(lc_core::LAYER_COUNT - 1)];
            let (r, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
            ui.painter().rect_filled(r, 3.0, Color32::from_rgb(c[0], c[1], c[2]));
            ui.painter().rect_stroke(r, 3.0, egui::Stroke::new(1.0, theme::border()), egui::StrokeKind::Inside);
            ui.label(RichText::new(trf("Cut settings — {}", &[&l.name])).strong());
        });
        let panel_scope = format!("panel-{}", self.ed.active_layer);
        layer_settings_ui(ui, (units, su), l, panel_scope.as_str(), has_images, &layer_ctx);
        let now = ui.input(|i| i.time);
        self.commit_layer_edit(now, before);
    }

    /// Record an undo step (one per editing burst) and refresh when the layers changed.
    pub fn commit_layer_edit(&mut self, now: f64, before: Vec<lc_core::Layer>) {
        if self.doc.layers != before {
            // The settings of the layers belong to the project (and to the material it was made with), not to the device.
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
        let has_images = self.doc.scene.shapes.iter().any(|s| s.layer == i && s.is_image());
        let mut layer_ctx = LayerCtx::of(&self.doc.device);
        layer_ctx.image_only = self.doc.is_image_layer(i);
        let mut open = true;
        let title = trf("Layer options — {}", &[&self.doc.layers[i].name]);
        crate::window_ui::popup(ctx, "layer_dialog", &title, [340.0, 220.0], &mut open, |ui| {
            let l = &mut self.doc.layers[i];
            egui::Grid::new("layer_head").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                ui.label(tr("Name"));
                ui.add(egui::TextEdit::singleline(&mut l.name).desired_width(140.0));
                ui.end_row();
                ui.label(tr("Colour"));
                let c = PALETTE[l.color.min(lc_core::LAYER_COUNT - 1)];
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
            layer_settings_ui(ui, (units, su), l, "dialog", has_images, &layer_ctx);
        });
        let now = ctx.input(|i| i.time);
        self.commit_layer_edit(now, before);
        if !open {
            self.layer_dlg = None;
        }
    }

    /// Live view of everything crossing the serial port.
    fn console_panel(&mut self, ui: &mut egui::Ui) {
        if let Some(line) = self.console.show(ui, self.connected) {
            self.link.send(Cmd::Line(line));
        }
    }
}

/// Width of the cell of one of the switches (output, visible, air pump) in the layer list.
const SWITCH_W: f32 = 22.0;

/// The cut settings of one layer: mode, speed, power, passes, fill options and image options.
/// `id_scope` keeps the widget ids apart when the same settings are shown in the panel and in the dialog.
/// What the layer settings need to know about the device.
pub struct LayerCtx {
    /// Acceleration of the machine, mm/s² (0 = not known): the braking distance at the layer's speed is suggested as overscan.
    pub accel: f64,
    /// The GRBL-like dialects can switch between constant and dynamic power per layer.
    pub power_modes: bool,
    /// The machine has a Z axis: layers can set a height and a step per pass.
    pub has_z: bool,
    /// The machine has a rotary attachment switched on: layers can be marked as rotary.
    pub rotary: bool,
    /// Names and beam spots (mm, the smaller side; 0 = not known) of the laser heads.
    pub heads: Vec<(String, f64)>,
    /// The layer holds only images: its mode is Image and cannot be changed.
    pub image_only: bool,
}

impl LayerCtx {
    pub fn of(dev: &lc_core::Device) -> LayerCtx {
        LayerCtx {
            accel: dev.speeds.acceleration,
            power_modes: dev.dialect().is_grbl(),
            has_z: dev.axes.has_z,
            rotary: dev.rotary.enabled,
            image_only: false,
            heads: dev.heads.iter().enumerate().map(|(i, h)| (if h.name.trim().is_empty() { trf("Head {}", &[&(i + 1)]) } else { h.name.clone() }, if h.spot_x > 0.0 && h.spot_y > 0.0 { h.spot_x.min(h.spot_y) } else { h.spot_x.max(h.spot_y) })).collect(),
        }
    }
}

pub fn layer_settings_ui(ui: &mut egui::Ui, (units, su): (lc_core::Units, lc_core::SpeedUnit), l: &mut lc_core::Layer, id_scope: &str, has_images: bool, dev: &LayerCtx) {
    let overscan_hint = |v: f64| (dev.accel > 0.0).then(|| v * v / (2.0 * dev.accel));
    let power_modes = dev.power_modes;
    egui::Grid::new(("cut", id_scope)).num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
        ui.label(tr("Mode"));
        // A layer of images is in the Image mode and stays there; everything else can still be set.
        ui.add_enabled_ui(!dev.image_only, |ui| {
            egui::ComboBox::from_id_salt(("cm", id_scope)).selected_text(tr(l.mode.label())).show_ui(ui, |ui| {
                for m in LayerMode::ALL {
                    ui.selectable_value(&mut l.mode, m, tr(m.label()));
                }
            });
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
        if matches!(l.mode, LayerMode::Line | LayerMode::FillAndLine) {
            ui.label(tr("Kerf offset"));
            drag_len(ui, units, &mut l.kerf, 0.01, Some((-5.0, 5.0))).on_hover_text(tr("Moves the outline out (in, when negative) to make up for the width of the cut. 0 = off."));
            ui.end_row();
        }
        if dev.rotary {
            ui.label(tr("Rotary"));
            ui.checkbox(&mut l.rotary, tr("Burn this layer on the rotary")).on_hover_text(tr("All burning layers must be rotary, or none. The design may not be higher than the way round the workpiece."));
            ui.end_row();
        }
        if dev.has_z {
            ui.label(tr("Z height"));
            ui.add(egui::DragValue::new(&mut l.z_start).speed(0.1).suffix(" mm")).on_hover_text(tr("Machine Z the head goes to before the first pass. 0 together with 0 per pass = Z is not moved."));
            ui.end_row();
            ui.label(tr("Z per pass"));
            ui.add(egui::DragValue::new(&mut l.z_step).speed(0.05).suffix(" mm")).on_hover_text(tr("How far Z moves with each further pass (negative goes down on a machine where up is +Z). 0 = Z stays."));
            ui.end_row();
        }
        if dev.heads.len() > 1 {
            ui.label(tr("Laser head"));
            let shown = dev.heads.get(l.head).map(|h| h.0.clone()).unwrap_or_else(|| dev.heads[0].0.clone());
            egui::ComboBox::from_id_salt(("head", id_scope)).selected_text(shown).show_ui(ui, |ui| {
                for (i, (name, _)) in dev.heads.iter().enumerate() {
                    ui.selectable_value(&mut l.head, i, name);
                }
            });
            ui.end_row();
        }
        ui.label(tr("Air assist"));
        ui.checkbox(&mut l.air_assist, tr("Air pump on")).on_hover_text(tr("Switch the air pump on while this layer burns (M8 / M9)"));
        ui.end_row();
        if power_modes {
            ui.label(tr("Power mode"));
            egui::ComboBox::from_id_salt(("pm", id_scope)).selected_text(tr(l.power_mode.label())).show_ui(ui, |ui| {
                for m in lc_core::PowerMode::ALL {
                    ui.selectable_value(&mut l.power_mode, m, tr(m.label()));
                }
            })
            .response
            .on_hover_text(tr("M4 (dynamic) lowers the power in corners and while accelerating; M3 (constant) keeps it. The G-code switches between them when layers differ."));
            ui.end_row();
        }
        if l.mode == LayerMode::Offset {
            ui.label(tr("Interval"));
            drag_len(ui, units, &mut l.interval, 0.005, Some((0.01, 5.0)));
            ui.end_row();
            let spot = dev.heads.get(l.head).map(|h| h.1).unwrap_or(0.0);
            if spot > 0.0 && l.interval < spot - 1e-9 {
                ui.label("");
                ui.label(RichText::new(trf("The interval is smaller than the beam spot ({}): neighbouring lines overlap.", &[&crate::units_ui::fmt_len(units, spot, 3)])).small().color(Color32::from_rgb(0xe8, 0x8a, 0x1f)));
                ui.end_row();
            }
        } else if l.mode != LayerMode::Line {
            ui.label(tr("Interval"));
            drag_len(ui, units, &mut l.interval, 0.005, Some((0.01, 5.0)));
            ui.end_row();
            let spot = dev.heads.get(l.head).map(|h| h.1).unwrap_or(0.0);
            if spot > 0.0 && l.interval < spot - 1e-9 {
                ui.label("");
                ui.label(RichText::new(trf("The interval is smaller than the beam spot ({}): neighbouring lines overlap.", &[&crate::units_ui::fmt_len(units, spot, 3)])).small().color(Color32::from_rgb(0xe8, 0x8a, 0x1f)));
                ui.end_row();
            }
            ui.label(tr("Scan angle"));
            ui.add(egui::DragValue::new(&mut l.angle).range(-180.0..=180.0).suffix("°"));
            ui.end_row();
            ui.label(tr("Overscan"));
            let r = drag_len(ui, units, &mut l.overscan, 0.1, Some((0.0, 20.0)));
            if let Some(mm) = overscan_hint(l.speed) {
                r.on_hover_text(trf("Suggested: {} (the braking distance at this speed and the machine's acceleration)", &[&crate::units_ui::fmt_len(units, mm, 2)]));
            }
            ui.end_row();
            ui.label(tr("Bidirectional"));
            ui.checkbox(&mut l.bidirectional, "");
            ui.end_row();
        }
    });
    if matches!(l.mode, LayerMode::Line | LayerMode::FillAndLine) {
        cut_options_ui(ui, units, l, id_scope);
    }
    if l.mode == LayerMode::Image && !dev.image_only {
        ui.label(RichText::new(tr("Vector shapes on a layer in the Image mode are not burnt.")).small().color(Color32::from_rgb(0xe8, 0x8a, 0x1f)));
    }
    // Dithering only matters for bitmap images, not for vector shapes.
    if !has_images {
        return;
    }
    egui::CollapsingHeader::new(tr("Image settings")).id_salt(("img_hdr", id_scope)).default_open(dev.image_only).show(ui, |ui| {
        ui.label(RichText::new(tr("Images use the interval, speed, power and overscan of their layer.")).color(theme::text_dim()));
        egui::Grid::new(("img_cut", id_scope)).num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
            ui.label(tr("Dithering"));
            egui::ComboBox::from_id_salt(("dither", id_scope)).selected_text(tr(l.dither.label())).show_ui(ui, |ui| {
                for d in Dither::ALL {
                    ui.selectable_value(&mut l.dither, d, tr(d.label())).on_hover_text(tr(d.hint()));
                }
            })
            .response
            .on_hover_text(tr(l.dither.hint()));
            ui.end_row();
            ui.label(tr("Min power (%)"));
            let max = l.power;
            ui.add(egui::Slider::new(&mut l.min_power, 0.0..=max));
            ui.end_row();
        });
    });
}

/// Perforation and tabs of a line layer (collapsed until switched on).
fn cut_options_ui(ui: &mut egui::Ui, units: lc_core::Units, l: &mut lc_core::Layer, id_scope: &str) {
    use lc_core::cutopts::TabMode;
    egui::CollapsingHeader::new(tr("Perforation")).id_salt(("perf", id_scope)).default_open(l.perforation.enabled).show(ui, |ui| {
        ui.checkbox(&mut l.perforation.enabled, tr("Perforation mode")).on_hover_text(tr("Alternates cutting and skipping along the outline."));
        ui.add_enabled_ui(l.perforation.enabled, |ui| {
            egui::Grid::new(("perf_g", id_scope)).num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                ui.label(tr("Cut length"));
                drag_len(ui, units, &mut l.perforation.cut, 0.05, Some((0.01, 100.0)));
                ui.end_row();
                ui.label(tr("Skip length"));
                drag_len(ui, units, &mut l.perforation.skip, 0.05, Some((0.01, 100.0)));
                ui.end_row();
            });
        });
    });
    egui::CollapsingHeader::new(tr("Tabs / bridges")).id_salt(("tabs", id_scope)).default_open(l.tabs.enabled).show(ui, |ui| {
        ui.checkbox(&mut l.tabs.enabled, tr("Tabs / bridges")).on_hover_text(tr("Leaves short stretches of the outline uncut (or cut weakly) so the piece stays in place. They are placed automatically."));
        ui.add_enabled_ui(l.tabs.enabled, |ui| {
            egui::Grid::new(("tabs_g", id_scope)).num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                ui.label(tr("Tab size"));
                drag_len(ui, units, &mut l.tabs.size, 0.05, Some((0.05, 100.0)));
                ui.end_row();
                ui.label(tr("Placement"));
                ui.vertical(|ui| {
                    ui.selectable_value(&mut l.tabs.mode, TabMode::Even, tr("Even spacing"));
                    ui.selectable_value(&mut l.tabs.mode, TabMode::PerShape, tr("Tabs per shape"));
                });
                ui.end_row();
                if l.tabs.mode == TabMode::Even {
                    ui.label(tr("Spacing"));
                    drag_len(ui, units, &mut l.tabs.spacing, 0.5, Some((0.5, 10_000.0)));
                    ui.end_row();
                    ui.label(tr("Min tabs"));
                    ui.add(egui::DragValue::new(&mut l.tabs.min_tabs).range(0..=1000));
                    ui.end_row();
                } else {
                    ui.label(tr("Tabs per shape"));
                    ui.add(egui::DragValue::new(&mut l.tabs.per_shape).range(0..=1000));
                    ui.end_row();
                }
                ui.label(tr("Limit max tabs"));
                ui.horizontal(|ui| {
                    ui.checkbox(&mut l.tabs.limit_max, "");
                    ui.add_enabled(l.tabs.limit_max, egui::DragValue::new(&mut l.tabs.max_tabs).range(0..=1000));
                });
                ui.end_row();
                ui.label(tr("Tab cut power"));
                ui.add(egui::DragValue::new(&mut l.tabs.cut_power).range(0.0..=100.0).suffix(" %")).on_hover_text(tr("Percent of the layer's power over a tab; 0 = the laser is off."));
                ui.end_row();
                ui.label(tr("Skip inner shapes"));
                ui.checkbox(&mut l.tabs.skip_inner, "");
                ui.end_row();
            });
        });
    });
}
