//! Preview window: the generated toolpath with every burn operation in its own colour, optional travel moves
//! and a simulation slider.
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

pub struct PreviewState {
    pub view: View,
    pub color_by: ColorBy,
    pub show_travel: bool,
    /// 0..=1: how much of the job is drawn.
    pub progress: f32,
    pub playing: bool,
    pub hidden: Vec<usize>,
    pub job: Option<(u64, gcode::Job)>,
}

impl Default for PreviewState {
    fn default() -> Self {
        PreviewState {
            view: View { zoom: 1.0, pan: egui::vec2(0.0, 0.0), need_fit: true },
            color_by: ColorBy::Operation,
            show_travel: false,
            progress: 1.0,
            playing: false,
            hidden: vec![],
            job: None,
        }
    }
}

/// Evenly spread, clearly different colours for the n-th of `n` operations.
fn op_color(i: usize, n: usize) -> Color32 {
    let h = (i as f32 / n.max(1) as f32) * 0.85 + 0.02;
    let (s, v) = (0.78, 0.88 - 0.12 * ((i % 2) as f32));
    let k = |off: f32| {
        let t = (h * 6.0 + off).rem_euclid(6.0);
        let x = (t - 3.0).abs() - 1.0;
        v * (1.0 - s * x.clamp(0.0, 1.0))
    };
    Color32::from_rgb((k(0.0) * 255.0) as u8, (k(4.0) * 255.0) as u8, (k(2.0) * 255.0) as u8)
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
    pub fn preview_window(&mut self, ctx: &egui::Context) {
        if !self.show_preview {
            return;
        }
        // Rebuild the job when the design changed.
        if self.pv.job.as_ref().map(|j| j.0) != Some(self.revision) {
            self.pv.job = Some((self.revision, gcode::generate(&self.doc)));
        }
        if self.pv.playing {
            let dt = ctx.input(|i| i.stable_dt).min(0.1);
            self.pv.progress += dt / 12.0;
            if self.pv.progress >= 1.0 {
                self.pv.progress = 1.0;
                self.pv.playing = false;
            }
            ctx.request_repaint();
        }
        let mut open = true;
        let units = self.doc.device.units;
        let (bw, bh) = (self.doc.device.bed_w, self.doc.device.bed_h);
        let Some((_, job)) = self.pv.job.take() else { return };
        egui::Window::new(tr("Preview")).open(&mut open).default_size([980.0, 640.0]).resizable(true).show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(tr("Colour by"));
                ui.selectable_value(&mut self.pv.color_by, ColorBy::Operation, tr("Operation"));
                ui.selectable_value(&mut self.pv.color_by, ColorBy::Layer, tr("Layer"));
                ui.separator();
                ui.checkbox(&mut self.pv.show_travel, tr("Show travel moves")).on_hover_text(tr("Moves of the machine with the laser off"));
                ui.separator();
                if ui.button(if self.pv.playing { tr("Pause") } else { tr("Play") }).clicked() {
                    if self.pv.progress >= 1.0 {
                        self.pv.progress = 0.0;
                    }
                    self.pv.playing = !self.pv.playing;
                }
                ui.spacing_mut().slider_width = 220.0;
                ui.add(egui::Slider::new(&mut self.pv.progress, 0.0..=1.0).show_value(false));
                if ui.button(tr("Fit")).clicked() {
                    self.pv.view.need_fit = true;
                }
                ui.label(RichText::new(format!("{} {}   {} {}", tr("Estimated time"), crate::app::fmt_time(job.est_seconds), tr("cut length"), crate::units_ui::fmt_len(units, job.cut_length, 0))).color(theme::text_dim()));
            });
            ui.separator();
            let n_ops = job.ops.len();
            ui.horizontal_top(|ui| {
                // Legend with a switch per operation.
                ui.vertical(|ui| {
                    ui.set_width(210.0);
                    ui.label(RichText::new(tr("Operations")).strong());
                    egui::ScrollArea::vertical().id_salt("pv_ops").max_height(ui.available_height().max(200.0)).show(ui, |ui| {
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
                                        let p = PALETTE[self.doc.layers[job.ops[i].layer].color.min(29)];
                                        Color32::from_rgb(p[0], p[1], p[2])
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
                let avail = ui.available_size();
                let (resp, painter) = ui.allocate_painter(avail, Sense::click_and_drag());
                let rect = resp.rect;
                painter.rect_filled(rect, 0.0, theme::workspace());
                if self.pv.view.need_fit && rect.width() > 50.0 {
                    let z = ((rect.width() - 40.0) / bw as f32).min((rect.height() - 40.0) / bh as f32).max(0.05);
                    self.pv.view.zoom = z;
                    self.pv.view.pan = egui::vec2((rect.width() - bw as f32 * z) / 2.0, (rect.height() - bh as f32 * z) / 2.0);
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
                    }
                }
                if resp.dragged() {
                    self.pv.view.pan += resp.drag_delta();
                }
                let (zoom, pan) = (self.pv.view.zoom, self.pv.view.pan);
                let w2s = |p: lc_core::Pt| -> Pos2 { o + pan + egui::vec2(p.x as f32 * zoom, p.y as f32 * zoom) };
                let bed = egui::Rect::from_min_max(w2s(lc_core::Pt::new(0.0, 0.0)), w2s(lc_core::Pt::new(bw, bh)));
                let painter = painter.with_clip_rect(rect);
                painter.rect_filled(bed, 0.0, Color32::WHITE);
                painter.rect_stroke(bed, 0.0, Stroke::new(1.0, Color32::from_gray(0x99)), egui::StrokeKind::Outside);
                let upto = ((job.moves.len() as f32) * self.pv.progress).round() as usize;
                let mut shapes = Vec::with_capacity(upto.min(job.moves.len()));
                let skip_travel = job.moves.len() > 200_000;
                for m in job.moves.iter().take(upto) {
                    if self.pv.hidden.contains(&m.op) {
                        continue;
                    }
                    if m.laser {
                        let c = match self.pv.color_by {
                            ColorBy::Operation => op_color(m.op, n_ops),
                            ColorBy::Layer => {
                                let p = PALETTE[self.doc.layers[m.layer].color.min(29)];
                                Color32::from_rgb(p[0], p[1], p[2])
                            }
                        };
                        shapes.push(egui::Shape::line_segment([w2s(m.a), w2s(m.b)], Stroke::new(1.4, c)));
                    } else if self.pv.show_travel && !skip_travel {
                        shapes.push(egui::Shape::line_segment([w2s(m.a), w2s(m.b)], Stroke::new(0.8, Color32::from_rgb(0x90, 0x90, 0xa0).gamma_multiply(0.8))));
                    }
                }
                painter.extend(shapes);
                if let Some(m) = job.moves.get(upto.saturating_sub(1)) {
                    painter.circle_filled(w2s(m.b), 4.0, Color32::from_rgb(0xe0, 0x20, 0x20));
                }
                if skip_travel && self.pv.show_travel {
                    painter.text(rect.left_bottom() + egui::vec2(8.0, -8.0), egui::Align2::LEFT_BOTTOM, tr("Too many moves: travel lines are hidden."), egui::FontId::proportional(12.0), theme::text_dim());
                }
                painter.text(rect.right_bottom() + egui::vec2(-8.0, -8.0), egui::Align2::RIGHT_BOTTOM, trf("{} moves", &[&job.moves.len()]), egui::FontId::proportional(11.0), theme::text_dim());
            });
        });
        self.pv.job = Some((self.revision, job));
        self.show_preview = open;
        if !open {
            self.pv.playing = false;
        }
    }
}
