//! The parts of the canvas that belong to the laser: what the machine's bed shows (unusable strips, limits, rotary
//! marks, no-go zones, home and the live position), the camera picture and the tool path preview. The canvas
//! itself is in base-ui; these are the hooks `App` gives it.
use crate::app::App;
use crate::theme;
use base_ui::editor::{CanvasView, EditorHost};
use eframe::egui::{self, Color32, PointerButton, Pos2, Stroke};
use lc_core::{Pt, PALETTE};

pub use base_ui::editor::{fit_region, fit_view};

fn luminance(c: Color32) -> f32 {
    (0.299 * c.r() as f32 + 0.587 * c.g() as f32 + 0.114 * c.b() as f32) / 255.0
}

/// `c` made dark (on a light sheet) or light (on a dark one) enough to be seen against `bg`.
fn contrasting(c: Color32, bg: Color32) -> Color32 {
    let (lc, lb) = (luminance(c), luminance(bg));
    if (lc - lb).abs() >= 0.4 {
        return c;
    }
    let k = if lb > 0.5 { 0.0 } else { 255.0 };
    // Blend towards black or white until the difference is large enough.
    let t = ((0.4 - (lc - lb).abs()) / 0.6 + 0.15).clamp(0.0, 0.85);
    let mix = |a: u8| (a as f32 * (1.0 - t) + k * t).round() as u8;
    Color32::from_rgb(mix(c.r()), mix(c.g()), mix(c.b()))
}

/// The colour of palette entry `i`.
pub fn col(i: usize) -> Color32 {
    let c = PALETTE[i.min(lc_core::LAYER_COUNT - 1)];
    Color32::from_rgb(c[0], c[1], c[2])
}

impl App {
    /// After the bed and its grid: unusable strips, limits, rotary marks, no-go zones, home and the live position.
    pub fn paint_machine_area(&mut self, v: &CanvasView) {
        let painter = v.painter;
        let (o, bed) = (v.origin, v.bed);
        let (_bw, bh) = self.work_size();
        // Strips the machine cannot use are hatched; software limits are outlined.
        {
            let u = self.doc.device.usable_rect();
            let to_screen = |r: lc_core::Rect| egui::Rect::from_min_max(self.w2s(o, r.min), self.w2s(o, r.max));
            let usable = to_screen(u);
            let ink = Color32::from_rgba_unmultiplied(0xc0, 0x30, 0x30, 70);
            let hatch = |painter: &egui::Painter, r: egui::Rect| {
                if r.width() < 0.5 || r.height() < 0.5 {
                    return;
                }
                painter.rect_filled(r, 0.0, Color32::from_rgba_unmultiplied(0xc0, 0x30, 0x30, 25));
                let p = painter.with_clip_rect(r);
                let step = 8.0;
                let mut k = -r.height();
                while k < r.width() {
                    p.line_segment([egui::pos2(r.left() + k, r.bottom()), egui::pos2(r.left() + k + r.height(), r.top())], Stroke::new(1.0, ink));
                    k += step;
                }
            };
            for strip in [
                egui::Rect::from_min_max(bed.min, egui::pos2(bed.max.x, usable.min.y)),
                egui::Rect::from_min_max(egui::pos2(bed.min.x, usable.max.y), bed.max),
                egui::Rect::from_min_max(egui::pos2(bed.min.x, usable.min.y), egui::pos2(usable.min.x, usable.max.y)),
                egui::Rect::from_min_max(egui::pos2(usable.max.x, usable.min.y), egui::pos2(bed.max.x, usable.max.y)),
            ] {
                hatch(&painter, strip);
            }
            if self.doc.device.area.soft_limits.is_some() {
                let l = to_screen(self.doc.device.allowed_rect());
                painter.rect_stroke(l, 0.0, Stroke::new(1.5, Color32::from_rgb(0xe8, 0x8a, 0x1f)), egui::StrokeKind::Middle);
            }
        }
        // A rotary attachment: the design wraps round the workpiece, so the way round is marked where it repeats.
        if self.doc.device.rotary.enabled {
            let circ = self.doc.device.rotary.circumference();
            if circ > 1.0 {
                let mut k = 1.0;
                while circ * k < bh {
                    let y = self.w2s(o, Pt::new(0.0, circ * k)).y;
                    painter.line_segment([egui::pos2(bed.min.x, y), egui::pos2(bed.max.x, y)], Stroke::new(1.0, Color32::from_rgb(0x2e, 0xa0, 0xc0)));
                    k += 1.0;
                }
                let y = self.w2s(o, Pt::new(0.0, circ)).y;
                if circ < bh {
                    painter.text(egui::pos2(bed.min.x + 4.0, y - 2.0), egui::Align2::LEFT_BOTTOM, format!("{} {}", crate::i18n::tr("Circumference"), crate::units_ui::fmt_len(self.doc.device.units, circ, 1)), egui::FontId::proportional(11.0), Color32::from_rgb(0x2e, 0xa0, 0xc0));
                }
            }
        }
        // No-go zones: red areas the machine must not enter (switched off in the View menu).
        if self.zones_visible {
            for z in &self.doc.device.zones {
                let (fill, edge) = if z.enabled { (Color32::from_rgba_unmultiplied(0xc0, 0x30, 0x30, 70), Color32::from_rgb(0xc0, 0x30, 0x30)) } else { (Color32::from_rgba_unmultiplied(0x90, 0x90, 0x90, 30), Color32::from_gray(0x90)) };
                let b = z.shape.bounds();
                match z.shape {
                    lc_core::ZoneShape::Rect { .. } => {
                        let r = egui::Rect::from_min_max(self.w2s(o, b.min), self.w2s(o, b.max));
                        painter.rect_filled(r, 0.0, fill);
                        painter.rect_stroke(r, 0.0, Stroke::new(1.5, edge), egui::StrokeKind::Middle);
                    }
                    lc_core::ZoneShape::Circle { x, y, r } => {
                        let c = self.w2s(o, Pt::new(x, y));
                        painter.circle(c, r.abs() as f32 * self.ed.view.zoom, fill, Stroke::new(1.5, edge));
                    }
                }
                let at = self.w2s(o, Pt::new(b.min.x, b.min.y));
                painter.text(at + egui::vec2(4.0, 2.0), egui::Align2::LEFT_TOP, &z.name, egui::FontId::proportional(11.0), edge);
            }
        }
        // Origin marker (machine zero)
        painter.circle_filled(self.w2s(o, self.doc.device.home_point()), 4.0, theme::accent());
        if let Some(p) = self.live_point() {
            let c = self.w2s(o, p);
            painter.circle_filled(c, 6.0, Color32::from_rgb(0xe0, 0x30, 0x30));
            painter.circle_stroke(c, 6.0, Stroke::new(1.5, Color32::WHITE));
        }

    }

    /// The camera picture under the design.
    pub fn paint_camera_overlay(&mut self, v: &CanvasView) {
        let painter = v.painter;
        let o = v.origin;
        // ---- camera overlay (under the design) ----
        if self.overlay_visible {
            if let Some(ov) = &self.overlay {
                let n = 16usize;
                let tint = Color32::from_white_alpha((ov.opacity * 255.0) as u8);
                let mut mesh = egui::Mesh::with_texture(ov.tex.id());
                for j in 0..=n {
                    for i in 0..=n {
                        let (u, v) = (i as f64 / n as f64, j as f64 / n as f64);
                        let p = crate::overlay::quad_map(&ov.corners, u, v);
                        mesh.vertices.push(egui::epaint::Vertex { pos: self.w2s(o, p), uv: egui::pos2(u as f32, v as f32), color: tint });
                    }
                }
                for j in 0..n {
                    for i in 0..n {
                        let k = (j * (n + 1) + i) as u32;
                        let w = (n + 1) as u32;
                        mesh.indices.extend_from_slice(&[k, k + 1, k + w + 1, k, k + w + 1, k + w]);
                    }
                }
                painter.add(egui::Shape::mesh(mesh));
                if self.overlay_edit {
                    let quad: Vec<Pos2> = ov.corners.iter().map(|c| self.w2s(o, *c)).collect();
                    painter.add(egui::Shape::closed_line(quad.clone(), Stroke::new(1.5, theme::accent())));
                    for q in quad {
                        painter.circle_filled(q, 7.0, Color32::WHITE);
                        painter.circle_stroke(q, 7.0, Stroke::new(2.0, theme::accent()));
                    }
                }
            }
        }

    }

    /// The tool path preview over the design.
    pub fn paint_preview(&mut self, v: &CanvasView) {
        let painter = v.painter;
        let (o, rect) = (v.origin, v.rect);
        let preview = self.preview_on;
        if preview {
            if let Some((_, job)) = &self.preview {
                let skip_travel = job.moves.len() > 150_000;
                let bed_c = self.bed_fill();
                let mut shapes = Vec::with_capacity(job.moves.len());
                for m in &job.moves {
                    if !m.laser && skip_travel {
                        continue;
                    }
                    let st = if m.laser {
                        Stroke::new(self.ed.grid_prefs.line_width.max(1.2), contrasting(col(self.doc.layers[m.layer].color), bed_c))
                    } else {
                        Stroke::new(0.6, contrasting(Color32::from_rgb(0x90, 0x90, 0xa0), bed_c))
                    };
                    shapes.push(egui::Shape::line_segment([self.w2s(o, m.a), self.w2s(o, m.b)], st));
                }
                painter.extend(shapes);
                let r = painter.text(rect.left_bottom() + egui::vec2(30.0, -12.0), egui::Align2::LEFT_BOTTOM, format!("{} {}   {} {}", crate::i18n::tr("Estimated time"), crate::app::fmt_time(job.est_seconds), crate::i18n::tr("cut length"), crate::units_ui::fmt_len(self.doc.device.units, job.cut_length, 0)), egui::FontId::proportional(13.0), theme::text());
                let _ = r;
            }
        }

    }

    /// Corner dragging for the camera overlay. Returns true while it owns the pointer.
    pub fn overlay_interact(&mut self, resp: &egui::Response, o: Pos2, press: Option<Pos2>, hpos: Option<Pos2>) -> bool {
        if !self.overlay_edit || self.overlay.is_none() {
            self.overlay_drag = None;
            return false;
        }
        if resp.drag_started_by(PointerButton::Primary) {
            if let (Some(m), Some(ov)) = (press, &self.overlay) {
                self.overlay_drag = (0..4).find(|i| self.w2s(o, ov.corners[*i]).distance(m) <= 12.0);
            }
        }
        if let Some(i) = self.overlay_drag {
            if resp.dragged_by(PointerButton::Primary) {
                if let Some(m) = hpos {
                    let w = self.s2w(o, m);
                    if let Some(ov) = &mut self.overlay {
                        ov.corners[i] = w;
                    }
                }
            }
            if resp.drag_stopped() {
                self.overlay_drag = None;
                self.overlay_save_cfg();
            }
            return true;
        }
        false
    }
}
