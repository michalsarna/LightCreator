//! Preview window: the generated toolpath with every burn operation in its own colour, optional travel moves
//! and a simulation slider.
use base_ui::editor::EditorHost;
use crate::app::{App, View};
use crate::i18n::{tr, trf};
use crate::theme;
use eframe::egui::{self, Color32, Pos2, RichText, Sense, Stroke};
use lc_core::gcode;
use lc_core::PALETTE;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColorBy {
    Operation,
    Layer,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PlaySpeed {
    /// The whole job takes 15 seconds.
    Fit,
    X1,
    X10,
    X50,
    X200,
}

impl PlaySpeed {
    pub const ALL: [PlaySpeed; 5] = [PlaySpeed::Fit, PlaySpeed::X1, PlaySpeed::X10, PlaySpeed::X50, PlaySpeed::X200];
    pub fn label(self) -> &'static str {
        match self {
            PlaySpeed::Fit => "Fit 15 s",
            PlaySpeed::X1 => "Real time",
            PlaySpeed::X10 => "×10",
            PlaySpeed::X50 => "×50",
            PlaySpeed::X200 => "×200",
        }
    }
    /// Simulated seconds per real second for a job lasting `total` seconds.
    pub fn rate(self, total: f64) -> f64 {
        match self {
            PlaySpeed::Fit => (total / 15.0).max(1e-6),
            PlaySpeed::X1 => 1.0,
            PlaySpeed::X10 => 10.0,
            PlaySpeed::X50 => 50.0,
            PlaySpeed::X200 => 200.0,
        }
    }
}

pub struct PreviewState {
    pub speed: PlaySpeed,
    /// End time of each move, seconds from the start of the job.
    pub cum: Vec<f64>,
    pub view: View,
    pub color_by: ColorBy,
    pub show_travel: bool,
    /// The sizes of the job over the preview, as on a technical drawing.
    pub show_dims: bool,
    /// 0..=1: how much of the job is drawn.
    pub progress: f32,
    pub playing: bool,
    pub hidden: Vec<usize>,
    /// Fit the view to the burned objects (true) or to the whole work area (false).
    pub fit_objects: bool,
    pub job: Option<(u64, gcode::Job)>,
    /// With a rotary attachment: the job on the workpiece in 3D (true), or flat as on the work area.
    pub cylinder: bool,
    /// How the cylinder is looked at.
    pub orbit: crate::cylinder_view::Orbit,
    /// The burned object in 3D (true) or flat as on the work area; `None`: in 3D for a job on more than one side, else flat.
    pub box3d: Option<bool>,
    /// How the object is looked at.
    pub orbit3: crate::box_view::Orbit3,
    /// The lines in the colour of a burn on wood (darker for more power) instead of the colours of the operations.
    pub burn_look: bool,
    /// The jobs of all the faces of a job on more than one side, for the object in 3D, made for the project as it was then.
    pub face_jobs: Option<(u64, Vec<gcode::Job>)>,
}

impl Default for PreviewState {
    fn default() -> Self {
        PreviewState {
            speed: PlaySpeed::X10,
            cum: vec![],
            view: View { zoom: 1.0, pan: egui::vec2(0.0, 0.0), need_fit: true, auto_fit: true },
            color_by: ColorBy::Operation,
            show_travel: false,
            show_dims: false,
            progress: 1.0,
            playing: false,
            hidden: vec![],
            fit_objects: true,
            job: None,
            cylinder: true,
            orbit: Default::default(),
            box3d: None,
            orbit3: Default::default(),
            burn_look: true,
            face_jobs: None,
        }
    }
}

/// Evenly spread, clearly different colours for the n-th of `n` operations, deep enough to stand out on a light work area (as the
/// pens of PenCreator do).
fn op_color(i: usize, n: usize) -> Color32 {
    let h = (i as f32 / n.max(1) as f32) * 0.85 + 0.02;
    let (s, v) = (0.85, 0.66 - 0.10 * ((i % 2) as f32));
    let k = |off: f32| {
        let t = (h * 6.0 + off).rem_euclid(6.0);
        let x = (t - 3.0).abs() - 1.0;
        v * (1.0 - s * x.clamp(0.0, 1.0))
    };
    Color32::from_rgb((k(0.0) * 255.0) as u8, (k(4.0) * 255.0) as u8, (k(2.0) * 255.0) as u8)
}

/// The colour of a layer in the preview: the colour of the layer, deepened so a light one (yellow, cyan) shows on a light work area.
fn layer_ink(p: [u8; 3]) -> Color32 {
    let k = |c: u8| (c as f32 * 0.7).round() as u8;
    Color32::from_rgb(k(p[0]), k(p[1]), k(p[2]))
}

fn op_label(job: &gcode::Job, i: usize, layers: &[lc_core::Layer]) -> String {
    let o = job.ops[i];
    let mut s = format!("{} · {}", layers[o.layer].name, tr(o.kind.label()));
    if o.passes > 1 {
        s.push_str(&format!(" · {}/{}", o.pass, o.passes));
    }
    s
}

impl App {
    /// Rebuild the job when the design changed and advance the playback. None while there is no job.
    pub(crate) fn preview_prepare(&mut self, ctx: &egui::Context) -> Option<gcode::Job> {
        if self.pv.job.as_ref().map(|j| j.0) != Some(self.job_stamp()) {
            let job = gcode::generate_with(&self.job_source(), None, &self.job_options());
            let mut t = 0.0;
            self.pv.cum = job.moves.iter().map(|m| { t += m.dur; t }).collect();
            self.pv.job = Some((self.job_stamp(), job));
        }
        let total = self.pv.cum.last().copied().unwrap_or(0.0);
        if self.pv.playing {
            // Real frame time keeps the motion smooth whatever the frame rate; speed is in job seconds.
            let dt = ctx.input(|i| i.unstable_dt).clamp(0.0, 0.25) as f64;
            if total > 0.0 {
                self.pv.progress += (dt * self.pv.speed.rate(total) / total) as f32;
            }
            if self.pv.progress >= 1.0 {
                self.pv.progress = 1.0;
                self.pv.playing = false;
            }
            ctx.request_repaint();
        }
        self.pv.job.take().map(|(_, j)| j)
    }

    /// The preview as a tab of the work area, in place of the drawing.
    pub fn preview_work_panel(&mut self, ui: &mut egui::Ui) {
        let Some(job) = self.preview_prepare(ui.ctx()) else { return };
        // The fitting of the picture is on its zoom bar.
        ui.label(RichText::new(tr("Preview")).strong());
        // The stamp of the job that was shown: when the body changed the project (the switch of the side), the job is made again.
        let stamp = self.job_stamp();
        egui::Frame::new().inner_margin(egui::Margin { left: 8, right: 8, top: 0, bottom: 3 }).show(ui, |ui| self.preview_body(ui, &job));
        self.pv.job = Some((stamp, job));
    }

    /// Is the preview a tab of the work area (so that the bar of the tabs has one for it)?
    pub fn preview_work_tab(&self) -> bool {
        // The preview is a tab (or a window of its own that the person took the tab out into); it is not a floating window of the program.
        self.show_preview
    }

    /// The tab was closed with its cross.
    pub fn preview_close(&mut self) {
        self.show_preview = false;
        self.pv_tab_shown = false;
        self.pv.playing = false;
    }

    pub fn preview_window(&mut self, _ctx: &egui::Context) {
        if !self.show_preview {
            self.pv_tab_seen = false;
            self.pv_tab_shown = false;
            return;
        }
        // The preview is a tab of the work area (drawn by `preview_work_panel`); it is shown the first time it opens. A window of its
        // own is the tab that the person took out (`work_panes`), never a second copy.
        if !self.pv_tab_seen {
            self.pv_tab_shown = true;
            self.cam_tab_shown = false;
        }
        self.pv_tab_seen = true;
    }

    /// Options, totals and the picture of the toolpath (the same in the window and in the tab).
    pub(crate) fn preview_body(&mut self, ui: &mut egui::Ui, job: &gcode::Job) {
        let units = self.doc.device.units;
        let bed_fill = self.ed.grid_prefs.preview_fill(self.scheme.is_dark());
        let line_w = self.ed.grid_prefs.line_width;
        let (bw, bh) = self.doc.device.work_size();
        let total = self.pv.cum.last().copied().unwrap_or(0.0);
        let rotary = self.doc.device.rotary.enabled;
        let box3d = self.pv.box3d.unwrap_or(self.doc.two.is_some());
        let solid = self.doc.solid();
        let shown = self.doc.two.as_ref().map(|t| t.shown);
        let mut want = shown;
        ui.horizontal_wrapped(|ui| {
            // A job on more than one side: the preview is of one side, the same one as the work area shows (and the one that is
            // burned).
            if let (Some(solid), Some(w)) = (solid.as_ref(), want.as_mut()) {
                for i in 0..solid.count() {
                    let r = ui.selectable_value(w, i, base_ui::two_sided::face_label(solid, i));
                    if solid.count() > 2 {
                        r.on_hover_text(base_ui::two_sided::face_tip(solid, i));
                    }
                }
                ui.separator();
            }
            if rotary {
                ui.selectable_value(&mut self.pv.cylinder, true, tr("Cylinder (3D)")).on_hover_text(tr("The workpiece on the rotary attachment with what is burned on it. Drag to turn it, the wheel zooms."));
                ui.selectable_value(&mut self.pv.cylinder, false, tr("Flat")).on_hover_text(tr("The surface of the workpiece unrolled, as on the work area"));
                ui.separator();
            } else {
                let mut on = box3d;
                ui.selectable_value(&mut on, true, "3D").on_hover_text(tr("The burned object in 3D, as big as the material of the project setup, with what is burned on each of its faces. Drag to turn it, the wheel zooms."));
                ui.selectable_value(&mut on, false, tr("Flat")).on_hover_text(tr("The job on the work area, as the machine burns it"));
                if on != box3d {
                    self.pv.box3d = Some(on);
                }
                if on {
                    base_ui::widgets::toggle(ui, &mut self.pv.burn_look, tr("Burn colours"), tr("The lines darker for more power, as burned on wood; off: in the colours of the operations or the layers"));
                }
                ui.separator();
            }
            ui.label(tr("Colour by"));
            ui.selectable_value(&mut self.pv.color_by, ColorBy::Operation, tr("Operation"));
            ui.selectable_value(&mut self.pv.color_by, ColorBy::Layer, tr("Layer"));
            ui.separator();
            base_ui::widgets::toggle(ui, &mut self.pv.show_travel, tr("Show travel moves"), tr("Moves of the machine with the laser off"));
            if !(rotary && self.pv.cylinder) && !(!rotary && self.pv.box3d.unwrap_or(self.doc.two.is_some())) {
                base_ui::widgets::toggle(ui, &mut self.pv.show_dims, tr("Dimensions"), tr("Show the sizes of the job as on a technical drawing"));
            }
            ui.separator();
            let auto = self.bed_fill();
            base_ui::widgets::area_colour(ui, tr("Work area colour"), &mut self.ed.grid_prefs.preview_color, auto);
            ui.separator();
            // Fixed width, so the slider next to it does not move when the label changes.
            let (icon, label) = if self.pv.playing { ("pause", tr("Pause")) } else { ("play", tr("Play")) };
            let play = egui::Button::image_and_text(crate::icons::slot(Some(icon), theme::text()), label);
            if ui.add_sized([86.0, 22.0], play).clicked() {
                if self.pv.progress >= 1.0 {
                    self.pv.progress = 0.0;
                }
                self.pv.playing = !self.pv.playing;
            }
            ui.spacing_mut().slider_width = 220.0;
            ui.add(egui::Slider::new(&mut self.pv.progress, 0.0..=1.0).show_value(false));
            egui::ComboBox::from_id_salt("pv_speed").width(90.0).selected_text(tr(self.pv.speed.label())).show_ui(ui, |ui| {
                for s in PlaySpeed::ALL {
                    ui.selectable_value(&mut self.pv.speed, s, tr(s.label()));
                }
            });
        });
        if let (Some(now), Some(want)) = (shown, want) {
            if now != want {
                self.show_face(want);
            }
        }
        ui.separator();
        let n_ops = job.ops.len();
        // Totals under the picture: time, length, number of passes (operations) and moves.
        egui::Panel::bottom("pv_totals").frame(egui::Frame::NONE).show(ui, |ui| {
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                // Real time of the job at the point shown: where the slider stands, out of the whole.
                let now = self.pv.progress as f64 * job.est_seconds;
                ui.label(RichText::new(format!("{} {} / {}", tr("Time"), crate::app::fmt_time(now), crate::app::fmt_time(job.est_seconds))).strong());
                ui.separator();
                ui.label(format!("{} {}", tr("cut length"), crate::units_ui::fmt_len(units, job.cut_length, 0)));
                ui.separator();
                ui.label(trf("{} passes", &[&n_ops]));
                ui.separator();
                ui.label(trf("{} moves", &[&job.moves.len()]));
            });
        });
        egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| {
        ui.horizontal_top(|ui| {
            // Legend with a switch per operation.
            ui.vertical(|ui| {
                ui.set_width(210.0);
                ui.label(RichText::new(tr("Operations")).strong());
                egui::ScrollArea::vertical().id_salt("pv_ops").max_height(ui.available_height().max(120.0)).show(ui, |ui| {
                    if n_ops == 0 {
                        ui.label(RichText::new(tr("Nothing to burn: no shapes on output layers")).color(theme::text_dim()));
                    }
                    for i in 0..n_ops {
                        ui.horizontal(|ui| {
                            let mut shown = !self.pv.hidden.contains(&i);
                            if ui.checkbox(&mut shown, "").changed() {
                                if shown {
                                    self.pv.hidden.retain(|h| *h != i);
                                } else {
                                    self.pv.hidden.push(i);
                                }
                            }
                            let c = match self.pv.color_by {
                                ColorBy::Operation => op_color(i, n_ops),
                                ColorBy::Layer => {
                                    layer_ink(PALETTE[self.doc.layers[job.ops[i].layer].color.min(lc_core::LAYER_COUNT - 1)])
                                }
                            };
                            let (r, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), Sense::hover());
                            ui.painter().rect_filled(r, 3.0, c);
                            ui.label(format!("{}. {}", i + 1, op_label(&job, i, &self.doc.layers)));
                        });
                    }
                });
            });
            ui.separator();
            // Leave room under the picture for the totals row.
            let avail = ui.available_size();
            let (resp, painter) = ui.allocate_painter(avail, Sense::click_and_drag());
            let rect = resp.rect;
            painter.rect_filled(rect, 0.0, self.workspace_fill());
            if rotary && self.pv.cylinder {
                self.cylinder_picture(ui, &resp, &painter.with_clip_rect(rect), job, total, line_w);
                return;
            }
            if !rotary && self.pv.box3d.unwrap_or(self.doc.two.is_some()) {
                self.box_picture(ui, &resp, &painter.with_clip_rect(rect), job, total, line_w);
                return;
            }
            if (self.pv.view.need_fit || self.pv.view.auto_fit) && rect.width() > 50.0 {
                let mut bounds: Option<(f64, f64, f64, f64)> = None;
                if self.pv.fit_objects {
                    for m in job.moves.iter().filter(|m| m.laser) {
                        for p in [m.a, m.b] {
                            let b = bounds.get_or_insert((p.x, p.y, p.x, p.y));
                            *b = (b.0.min(p.x), b.1.min(p.y), b.2.max(p.x), b.3.max(p.y));
                        }
                    }
                }
                let (z, pan) = match bounds {
                    Some((x0, y0, x1, y1)) => {
                        crate::canvas::fit_region((rect.width(), rect.height()), (x0, y0), ((x1 - x0).max(1.0), (y1 - y0).max(1.0)), 20.0)
                    }
                    None => crate::canvas::fit_view((rect.width(), rect.height()), (bw, bh), 20.0),
                };
                self.pv.view.zoom = z;
                self.pv.view.pan = pan;
                self.pv.view.need_fit = false;
            }
            let o = rect.min;
            if resp.hovered() {
                let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                if let (Some(m), true) = (resp.hover_pos(), scroll.abs() > 0.0) {
                    let f = (scroll * 0.0015).exp();
                    let before = (m - o - self.pv.view.pan) / self.pv.view.zoom;
                    self.pv.view.zoom = (self.pv.view.zoom * f).clamp(0.05, 200.0);
                    self.pv.view.pan = m - o - before * self.pv.view.zoom;
                    self.pv.view.auto_fit = false;
                }
            }
            if resp.dragged() {
                self.pv.view.pan += resp.drag_delta();
                self.pv.view.auto_fit = false;
            }
            let (zoom, pan) = (self.pv.view.zoom, self.pv.view.pan);
            let w2s = |p: lc_core::Pt| -> Pos2 { o + pan + egui::vec2(p.x as f32 * zoom, p.y as f32 * zoom) };
            let bed = egui::Rect::from_min_max(w2s(lc_core::Pt::new(0.0, 0.0)), w2s(lc_core::Pt::new(bw, bh)));
            let painter = painter.with_clip_rect(rect);
            painter.rect_filled(bed, 0.0, bed_fill);
            painter.rect_stroke(bed, 0.0, Stroke::new(1.0, Color32::from_gray(0x99)), egui::StrokeKind::Outside);
            // Machine zero.
            let home = w2s(self.doc.device.home_point());
            painter.circle_filled(home, 5.0, theme::accent());
            painter.circle_stroke(home, 8.0, Stroke::new(1.2, theme::accent()));
            // Everything before the current time is drawn whole; the current move only up to the head.
            let t_now = self.pv.progress as f64 * total;
            let cur = self.pv.cum.partition_point(|c| *c <= t_now).min(job.moves.len());
            let mut shapes = Vec::with_capacity(cur);
            let skip_travel = job.moves.len() > 200_000;
            let style_of = |m: &gcode::Move| -> Option<Stroke> {
                if self.pv.hidden.contains(&m.op) {
                    return None;
                }
                if m.laser {
                    let c = match self.pv.color_by {
                        ColorBy::Operation => op_color(m.op, n_ops),
                        ColorBy::Layer => {
                            layer_ink(PALETTE[self.doc.layers[m.layer].color.min(lc_core::LAYER_COUNT - 1)])
                        }
                    };
                    Some(Stroke::new(line_w * 0.8, c))
                } else if self.pv.show_travel && !skip_travel {
                    Some(Stroke::new(0.5, Color32::from_rgb(0x90, 0x90, 0xa0).gamma_multiply(0.8)))
                } else {
                    None
                }
            };
            for m in job.moves.iter().take(cur) {
                if let Some(st) = style_of(m) {
                    shapes.push(egui::Shape::line_segment([w2s(m.a), w2s(m.b)], st));
                }
            }
            let mut head = home;
            if let Some(m) = job.moves.get(cur) {
                let before = if cur == 0 { 0.0 } else { self.pv.cum[cur - 1] };
                let f = if m.dur > 1e-12 { ((t_now - before) / m.dur).clamp(0.0, 1.0) } else { 1.0 };
                let p = lc_core::Pt::new(m.a.x + (m.b.x - m.a.x) * f, m.a.y + (m.b.y - m.a.y) * f);
                if let Some(st) = style_of(m) {
                    shapes.push(egui::Shape::line_segment([w2s(m.a), w2s(p)], st));
                }
                head = w2s(p);
            } else if let Some(m) = job.moves.last() {
                head = w2s(m.b);
            }
            painter.extend(shapes);
            painter.circle_filled(head, 4.5, Color32::from_rgb(0xe0, 0x20, 0x20));
            painter.circle_stroke(head, 4.5, Stroke::new(1.0, Color32::WHITE));
            if self.pv.show_dims {
                // The objects that are burnt, from the zero of the machine.
                let burnt = self.doc.scene.shapes.iter().filter(|s| self.doc.layers.get(s.layer).is_some_and(|l| l.output && l.visible));
                let f = base_ui::dims::Features::of(burnt, &|p| p);
                let shown = base_ui::editor::EditorHost::units(self);
                let st = base_ui::dims::DimStyle::on(bed_fill).in_units(shown, self.doc.device.units);
                base_ui::dims::drawing(&painter, &w2s, &f, Some(self.doc.device.home_point()), &st);
            }
            if skip_travel && self.pv.show_travel {
                // Above the zoom bar.
                painter.text(rect.left_bottom() + egui::vec2(8.0, -44.0), egui::Align2::LEFT_BOTTOM, tr("Too many moves: travel lines are hidden."), egui::FontId::proportional(12.0), theme::text_dim());
            }
            // The zoom bar of every picture, in the lower left corner: the same as the wheel. Its zoom counts from the whole work area.
            let whole = crate::canvas::fit_view((rect.width(), rect.height()), (bw, bh), 20.0).0.max(1e-6);
            let zoom_about_middle = |v: &mut base_ui::editor::View, f: f32| {
                let c = rect.size() / 2.0;
                let z = (v.zoom * f).clamp(0.05, 200.0);
                v.pan = c - (c - v.pan) * (z / v.zoom);
                v.zoom = z;
                v.auto_fit = false;
            };
            // Its fourth button fits all the objects to burn; the whole picture is the work area.
            match base_ui::zoom_bar::overlay_with(ui, rect, self.pv.view.zoom / whole, &tr("Drag: move · wheel: zoom"), true) {
                Some(base_ui::zoom_bar::ZoomCmd::FitObjects) => {
                    self.pv.fit_objects = true;
                    self.pv.view.need_fit = true;
                    self.pv.view.auto_fit = true;
                }
                Some(base_ui::zoom_bar::ZoomCmd::In) => zoom_about_middle(&mut self.pv.view, base_ui::zoom_bar::STEP),
                Some(base_ui::zoom_bar::ZoomCmd::Out) => zoom_about_middle(&mut self.pv.view, 1.0 / base_ui::zoom_bar::STEP),
                Some(base_ui::zoom_bar::ZoomCmd::Whole) => {
                    self.pv.fit_objects = false;
                    self.pv.view.need_fit = true;
                    self.pv.view.auto_fit = true;
                }
                None => {}
            }
        });
        });

    }
}

impl App {
    /// The colour of a move in the preview, or `None` when it is not drawn.
    fn move_stroke(&self, m: &gcode::Move, n_ops: usize, line_w: f32, travel: bool) -> Option<Stroke> {
        if self.pv.hidden.contains(&m.op) {
            return None;
        }
        if m.laser {
            let c = match self.pv.color_by {
                ColorBy::Operation => op_color(m.op, n_ops),
                ColorBy::Layer => layer_ink(PALETTE[self.doc.layers[m.layer].color.min(lc_core::LAYER_COUNT - 1)]),
            };
            Some(Stroke::new(line_w * 0.8, c))
        } else if travel {
            Some(Stroke::new(0.5, Color32::from_rgb(0x90, 0x90, 0xa0).gamma_multiply(0.8)))
        } else {
            None
        }
    }

    /// The object of the project in 3D: the piece of material (or the post or the block) with the jobs of all its faces burned on
    /// them, the shown face up to the time of the slider, and the head.
    fn box_picture(&mut self, ui: &mut egui::Ui, resp: &egui::Response, painter: &egui::Painter, job: &gcode::Job, total: f64, line_w: f32) {
        use crate::box_view::{self as bv, BoxProjection, Orbit3};
        use lc_core::faces::Solid;
        let rect = resp.rect;
        let solid = self.doc.solid().unwrap_or_else(|| {
            let (w, h) = self.doc.piece_size();
            Solid::sheet(w, h, self.doc.piece.map_or(3.0, |p| p[2]), lc_core::setup::FlipAxis::SideToSide)
        });
        let shown = self.doc.two.as_ref().map_or(0, |t| t.shown);
        if resp.dragged() {
            self.pv.orbit3.drag(resp.drag_delta());
        }
        if resp.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll.abs() > 0.0 {
                self.pv.orbit3.zoom = (self.pv.orbit3.zoom * (scroll * 0.0015).exp()).clamp(0.2, 30.0);
            }
        }
        if resp.double_clicked() {
            self.pv.orbit3 = Orbit3::default();
        }
        // The jobs of the other faces, whole.
        let stamp = self.job_stamp();
        if self.doc.two.is_some() && self.pv.face_jobs.as_ref().is_none_or(|(k, _)| *k != stamp) {
            let opts = gcode::JobOptions { one_by_one: self.one_by_one, side_note: None, then: vec![] };
            let jobs = (0..solid.count()).map(|i| gcode::generate_with(&self.doc.with_face(i), None, &opts)).collect();
            self.pv.face_jobs = Some((stamp, jobs));
        }
        let pj = BoxProjection::new(rect, &solid, self.pv.orbit3);
        bv::paint_body(painter, &pj, &solid, crate::setup_glue::WOOD, self.doc.two.is_some());
        let n_ops = job.ops.len();
        let travel = self.pv.show_travel && job.moves.len() <= 200_000;
        let o = self.doc.piece_origin();
        let local = |p: lc_core::Pt| lc_core::Pt::new(p.x - o.x, p.y - o.y);
        let line_w = line_w.max(1.3);
        let stroke = |m: &gcode::Move, n_ops: usize| -> Option<Stroke> {
            if self.pv.burn_look && m.laser && !self.pv.hidden.contains(&m.op) {
                let p = self.doc.layers.get(m.layer).map_or(50.0, |l| l.power);
                return Some(Stroke::new(line_w, bv::burn_colour(crate::setup_glue::WOOD, p)));
            }
            self.move_stroke(m, n_ops, line_w, travel)
        };
        let mut shapes = vec![];
        if let Some((_, jobs)) = &self.pv.face_jobs {
            for (j, fj) in jobs.iter().enumerate().filter(|(j, _)| *j != shown && self.doc.two.is_some()) {
                for m in fj.moves.iter().filter(|m| m.laser) {
                    if let Some(st) = stroke(m, fj.ops.len()) {
                        bv::face_line(&mut shapes, &pj, &solid, j, local(m.a), local(m.b), st);
                    }
                }
            }
        }
        let t_now = self.pv.progress as f64 * total;
        let cur = self.pv.cum.partition_point(|c| *c <= t_now).min(job.moves.len());
        for m in job.moves.iter().take(cur) {
            if let Some(st) = stroke(m, n_ops) {
                bv::face_line(&mut shapes, &pj, &solid, shown, local(m.a), local(m.b), st);
            }
        }
        let mut head = None;
        if let Some(m) = job.moves.get(cur) {
            let before = if cur == 0 { 0.0 } else { self.pv.cum[cur - 1] };
            let f = if m.dur > 1e-12 { ((t_now - before) / m.dur).clamp(0.0, 1.0) } else { 1.0 };
            let p = lc_core::Pt::new(m.a.x + (m.b.x - m.a.x) * f, m.a.y + (m.b.y - m.a.y) * f);
            if let Some(st) = stroke(m, n_ops) {
                bv::face_line(&mut shapes, &pj, &solid, shown, local(m.a), local(p), st);
            }
            head = Some(p);
        }
        painter.extend(shapes);
        let face = solid.face(shown);
        if let (Some(p), true) = (head, pj.facing(face.out) > 0.0 && self.pv.progress < 1.0) {
            let at = pj.at(face.at(local(p), 0.0));
            painter.circle_filled(at, 4.5, Color32::from_rgb(0xe0, 0x20, 0x20));
            painter.circle_stroke(at, 4.5, Stroke::new(1.0, Color32::WHITE));
        }
        let units = self.doc.device.units;
        let s = solid.bbox();
        let size = format!("{} × {} × {}", crate::units_ui::fmt_len(units, s[0], 1), crate::units_ui::fmt_len(units, s[1], 1), crate::units_ui::fmt_len(units, s[2], 1));
        painter.text(rect.left_top() + egui::vec2(8.0, 8.0), egui::Align2::LEFT_TOP, size, egui::FontId::proportional(12.0), theme::text_dim());
        let (eye, right, up) = self.pv.orbit3.axes();
        match base_ui::navicube::show(ui, rect, &[right, up, eye]) {
            Some(base_ui::navicube::Nav::LookFrom(d)) => self.pv.orbit3.look_from(d),
            Some(base_ui::navicube::Nav::Home) => self.pv.orbit3 = Orbit3::default(),
            None => {}
        }
        match base_ui::zoom_bar::overlay_with(ui, rect, self.pv.orbit3.zoom, &tr("Drag: turn · wheel: zoom"), true) {
            Some(base_ui::zoom_bar::ZoomCmd::In) => self.pv.orbit3.zoom = (self.pv.orbit3.zoom * base_ui::zoom_bar::STEP).min(30.0),
            Some(base_ui::zoom_bar::ZoomCmd::Out) => self.pv.orbit3.zoom = (self.pv.orbit3.zoom / base_ui::zoom_bar::STEP).max(0.2),
            Some(base_ui::zoom_bar::ZoomCmd::FitObjects | base_ui::zoom_bar::ZoomCmd::Whole) => self.pv.orbit3 = Orbit3::default(),
            None => {}
        }
    }

    /// The cylinder of the rotary in the preview: the workpiece of the diameter of the device, as long as the job and a little more,
    /// the middle of the job facing the viewer; the burned lines wound round it up to the time of the slider, and the head.
    fn cylinder_picture(&mut self, ui: &mut egui::Ui, resp: &egui::Response, painter: &egui::Painter, job: &gcode::Job, total: f64, line_w: f32) {
        use crate::cylinder_view::{self as cv, Cylinder, Orbit, Projection};
        let rect = resp.rect;
        let r = (self.doc.device.rotary.diameter / 2.0).max(0.5);
        let burns = job.moves.iter().filter(|m| m.laser).flat_map(|m| [m.a, m.b]);
        let (x0, x1, y0, y1) = burns.fold((f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY), |b, p| (b.0.min(p.x), b.1.max(p.x), b.2.min(p.y), b.3.max(p.y)));
        let (bw, _) = self.doc.device.work_size();
        let (x0, x1, y_mid) = if x0.is_finite() { (x0 - 15.0, x1 + 15.0, (y0 + y1) / 2.0) } else { (0.0, bw, 0.0) };
        let (x0, x1) = if x1 - x0 < 2.0 * r { ((x0 + x1) / 2.0 - r, (x0 + x1) / 2.0 + r) } else { (x0, x1) };
        // Dragging turns the cylinder (up and down) and swings it (left and right); the wheel zooms.
        if resp.dragged() {
            let d = resp.drag_delta();
            let o = &mut self.pv.orbit;
            o.spin += d.y as f64 / (r * 2.0).max(10.0) * 0.6;
            o.yaw = (o.yaw + d.x as f64 * 0.01).clamp(-1.2, 1.2);
        }
        if resp.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll.abs() > 0.0 {
                self.pv.orbit.zoom = (self.pv.orbit.zoom * (scroll * 0.0015).exp()).clamp(0.2, 20.0);
            }
        }
        let pj = Projection::new(rect, Cylinder { r, x0, x1, y_mid }, self.pv.orbit);
        cv::paint_body(painter, &pj, [0xd8, 0xb0, 0x7c]);
        let n_ops = job.ops.len();
        let skip_travel = job.moves.len() > 200_000;
        let travel = self.pv.show_travel && !skip_travel;
        let t_now = self.pv.progress as f64 * total;
        let cur = self.pv.cum.partition_point(|c| *c <= t_now).min(job.moves.len());
        let mut shapes = Vec::with_capacity(cur);
        // A little thicker than on the flat picture: the lines lie on a shaded surface.
        let line_w = line_w.max(1.6);
        for m in job.moves.iter().take(cur) {
            if let Some(st) = self.move_stroke(m, n_ops, line_w, travel) {
                cv::wound_line(&mut shapes, &pj, m.a, m.b, st);
            }
        }
        let mut head = None;
        if let Some(m) = job.moves.get(cur) {
            let before = if cur == 0 { 0.0 } else { self.pv.cum[cur - 1] };
            let f = if m.dur > 1e-12 { ((t_now - before) / m.dur).clamp(0.0, 1.0) } else { 1.0 };
            let p = lc_core::Pt::new(m.a.x + (m.b.x - m.a.x) * f, m.a.y + (m.b.y - m.a.y) * f);
            if let Some(st) = self.move_stroke(m, n_ops, line_w, travel) {
                cv::wound_line(&mut shapes, &pj, m.a, p, st);
            }
            head = Some(p);
        } else if let Some(m) = job.moves.last() {
            head = Some(m.b);
        }
        painter.extend(shapes);
        if let Some(p) = head {
            let (at, facing) = pj.surface(p);
            if facing > 0.0 {
                painter.circle_filled(at, 4.5, Color32::from_rgb(0xe0, 0x20, 0x20));
                painter.circle_stroke(at, 4.5, Stroke::new(1.0, Color32::WHITE));
            }
        }
        painter.text(rect.left_top() + egui::vec2(8.0, 8.0), egui::Align2::LEFT_TOP, format!("Ø {}", crate::units_ui::fmt_len(self.doc.device.units, 2.0 * r, 1)), egui::FontId::proportional(12.0), theme::text_dim());
        match base_ui::zoom_bar::overlay_with(ui, rect, self.pv.orbit.zoom, &tr("Drag: turn · wheel: zoom"), true) {
            Some(base_ui::zoom_bar::ZoomCmd::In) => self.pv.orbit.zoom = (self.pv.orbit.zoom * base_ui::zoom_bar::STEP).min(20.0),
            Some(base_ui::zoom_bar::ZoomCmd::Out) => self.pv.orbit.zoom = (self.pv.orbit.zoom / base_ui::zoom_bar::STEP).max(0.2),
            Some(base_ui::zoom_bar::ZoomCmd::FitObjects | base_ui::zoom_bar::ZoomCmd::Whole) => self.pv.orbit = Orbit::default(),
            None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::{App, Screen};
    use eframe::egui;

    fn editor() -> (App, egui::Context) {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.screen = Screen::Editor;
        (a, ctx)
    }

    #[test]
    fn the_preview_is_always_a_tab_of_the_work_area_and_draws_there() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        assert!(!a.preview_work_tab(), "closed to begin with");
        a.show_preview = true;
        assert!(a.preview_work_tab(), "a tab, not a floating window");
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            a.preview_window(ui.ctx());
            egui::CentralPanel::no_frame().show(ui, |ui| a.preview_work_panel(ui));
        });
        out.textures_delta.clear();
        a.preview_close();
        assert!(!a.preview_work_tab() && !a.show_preview);
    }

    #[test]
    fn the_tab_opens_shown_again_after_it_was_closed_and_the_camera_tab_gives_way() {
        let (mut a, ctx) = editor();
        a.show_preview = true;
        a.cam_tab_shown = true;
        a.preview_close();
        assert!(!a.show_preview && !a.pv_tab_shown);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            a.preview_window(ui.ctx());
            egui::CentralPanel::no_frame().show(ui, |ui| a.preview_work_panel(ui));
        });
        out.textures_delta.clear();
        // Opened again: it is still a tab, and the shown one.
        a.show_preview = true;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            a.preview_window(ui.ctx());
            egui::CentralPanel::no_frame().show(ui, |ui| a.preview_work_panel(ui));
        });
        out.textures_delta.clear();
        assert!(a.preview_work_tab() && a.pv_tab_shown && !a.cam_tab_shown);
    }

    #[test]
    fn the_preview_has_the_colour_of_the_work_area_and_paints_it() {
        let (mut a, ctx) = editor();
        a.doc.add(0, lc_core::Kind::Rect { w: 20.0, h: 10.0 }, lc_core::Xf::IDENTITY);
        a.show_preview = true;
        a.ed.grid_prefs.preview_color = Some([0x12, 0x34, 0x56, 0xff]);
        let mut seen = (false, false);
        for _ in 0..3 {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
                a.preview_window(ui.ctx());
                egui::CentralPanel::no_frame().show(ui, |ui| a.preview_work_panel(ui));
            });
            out.textures_delta.clear();
            for s in &out.shapes {
                match &s.shape {
                    egui::epaint::Shape::Text(t) if t.galley.text() == "Work area colour" => seen.0 = true,
                    egui::epaint::Shape::Rect(r) if r.fill == egui::Color32::from_rgb(0x12, 0x34, 0x56) => seen.1 = true,
                    _ => {}
                }
            }
        }
        assert_eq!(seen, (true, true), "the label in the bar and the work area in that colour");
    }
}

#[cfg(test)]
mod cylinder_tests {
    use crate::app::{App, Screen};
    use eframe::egui;

    fn rotary_app() -> App {
        let mut a = App::build(&egui::Context::default(), None, false);
        a.screen = Screen::Editor;
        a.doc.device.rotary.enabled = true;
        a.doc.device.rotary.diameter = 60.0;
        a.doc.add(0, lc_core::Kind::Rect { w: 80.0, h: 40.0 }, lc_core::Xf::translate(60.0, 40.0));
        a.doc.add(0, lc_core::Kind::Ellipse { w: 30.0, h: 30.0 }, lc_core::Xf::translate(85.0, 45.0));
        a.show_preview = true;
        a
    }

    /// With a rotary attachment the preview starts on the cylinder, and Flat brings back the work area; both draw.
    #[test]
    fn with_a_rotary_the_preview_shows_the_cylinder_and_can_go_flat() {
        let a = rotary_app();
        assert!(a.pv.cylinder);
        let shot = std::env::var("LC_CYL_SHOT").ok();
        let mut h = egui_kittest::Harness::builder().with_accessibility_check(false).with_size([1000.0, 560.0]).with_context_setup(|ctx| {
            base_ui::fonts::install(ctx);
            crate::icons::install(ctx);
        }).build_ui_state(|ui, a: &mut App| a.preview_work_panel(ui), a);
        h.run_steps(4);
        if let Some(p) = shot {
            h.render().unwrap().save(p).unwrap();
        }
        h.state_mut().pv.cylinder = false;
        h.run_steps(2);
        assert!(!h.state().pv.cylinder);
    }
}

#[cfg(test)]
mod side_tests {
    use crate::app::{App, Screen};
    use eframe::egui;

    /// A two-sided job: the preview shows the side chosen in it, which is also the side of the work area.
    #[test]
    fn the_preview_of_a_two_sided_job_shows_the_side_chosen() {
        let mut a = App::build(&egui::Context::default(), None, false);
        a.screen = Screen::Editor;
        a.setup.sides = base_ui::project_setup::Sides::Two;
        a.setup.marks = false;
        a.apply_setup();
        a.doc.add(0, lc_core::Kind::Rect { w: 20.0, h: 10.0 }, lc_core::Xf::IDENTITY);
        a.show_side(true);
        a.doc.add(0, lc_core::Kind::Rect { w: 5.0, h: 5.0 }, lc_core::Xf::translate(50.0, 50.0));
        a.show_preview = true;
        let mut h = egui_kittest::Harness::builder().with_accessibility_check(false).with_size([1000.0, 600.0]).with_context_setup(|ctx| {
            base_ui::fonts::install(ctx);
            crate::icons::install(ctx);
        }).build_ui_state(|ui, a: &mut App| a.preview_work_panel(ui), a);
        h.run_steps(3);
        use egui_kittest::kittest::Queryable;
        h.get_by_label("Side A").click();
        h.run_steps(3);
        assert!(!h.state().doc.two.as_ref().unwrap().shown_b(), "side A is shown");
        assert_eq!(h.state().doc.scene.shapes.len(), 1);
        // The picture is of side A too (its rectangle at the left), not the job of side B made before the switch.
        let burned = |a: &App| -> f64 { a.pv.job.as_ref().map_or(f64::NAN, |(_, j)| j.moves.iter().filter(|m| m.laser).map(|m| m.a.x.max(m.b.x)).fold(0.0, f64::max)) };
        assert!(burned(h.state()) <= 20.0 + 1e-6, "side A burned: {}", burned(h.state()));
        h.get_by_label("Side B").click();
        h.run_steps(3);
        assert!(burned(h.state()) >= 50.0, "side B burned: {}", burned(h.state()));
    }
}
