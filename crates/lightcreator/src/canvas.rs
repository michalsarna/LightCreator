use base_ui::editor::EditorHost;
use crate::app::{App, Tool};
use crate::menu::Act;
use crate::theme;
use eframe::egui::{self, Color32, CursorIcon, PointerButton, Pos2, Sense, Stroke};
use crate::nodes::NodeRef;
use lc_core::{Kind, Node, Polyline, Pt, Rect, Xf, PALETTE};

pub use base_ui::editor::{fit_region, fit_view, Drag};
use base_ui::editor::{handle_pts, nice_step};

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

fn col(i: usize) -> Color32 {
    let c = PALETTE[i.min(29)];
    Color32::from_rgb(c[0], c[1], c[2])
}





impl App {
    /// Corner dragging for the camera overlay. Returns true while it owns the pointer.
    fn overlay_interact(&mut self, resp: &egui::Response, o: Pos2, press: Option<Pos2>, hpos: Option<Pos2>) -> bool {
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

    pub fn canvas(&mut self, ui: &mut egui::Ui) {
        let avail = ui.available_size();
        let (resp, painter) = ui.allocate_painter(avail, Sense::click_and_drag());
        let rect = resp.rect;
        let o = rect.min;
        let (bw, bh) = self.doc.device.work_size();

        self.ed.canvas_avail = (rect.width(), rect.height());
        self.ed.canvas_origin = rect.min;
        // Fit everything on the page once, on request.
        if self.ed.fit_all_req && rect.width() > 50.0 {
            self.ed.fit_all_req = false;
            let all: Vec<u64> = self.doc.scene.shapes.iter().filter(|s| self.doc.layers[s.layer].visible).map(|s| s.id).collect();
            let (min, size) = match self.doc.bounds_of(&all) {
                Some(b) => ((b.min.x, b.min.y), (b.width().max(1.0), b.height().max(1.0))),
                None => ((0.0, 0.0), (bw, bh)),
            };
            let (z, pan) = fit_region((rect.width(), rect.height()), min, size, 50.0);
            self.ed.view.zoom = z;
            self.ed.view.pan = pan;
            self.ed.view.need_fit = false;
            self.ed.view.auto_fit = false;
        }
        // Fit on request, and keep fitting while the window or the panels are resized.
        if (self.ed.view.need_fit || self.ed.view.auto_fit) && rect.width() > 50.0 {
            let (z, pan) = fit_view((rect.width(), rect.height()), (bw, bh), 50.0);
            self.ed.view.zoom = z;
            self.ed.view.pan = pan;
            self.ed.view.need_fit = false;
        }

        // ---- zoom / pan ----
        if resp.hovered() {
            let (scroll, pinch) = ui.input(|i| (i.smooth_scroll_delta.y, i.zoom_delta()));
            let f = (scroll * 0.0015).exp() * pinch;
            if (f - 1.0).abs() > 1e-4 {
                if let Some(m) = resp.hover_pos() {
                    let before = self.s2w(o, m);
                    self.ed.view.zoom = (self.ed.view.zoom * f).clamp(0.05, 200.0);
                    let after = self.w2s(o, before);
                    self.ed.view.pan += m - after;
                    self.ed.view.auto_fit = false;
                }
            }
        }
        if resp.dragged_by(PointerButton::Middle) || (self.ed.tool == Tool::Pan && resp.dragged_by(PointerButton::Primary)) {
            self.ed.view.pan += resp.drag_delta();
            self.ed.view.auto_fit = false;
        }
        self.ed.cursor_mm = resp.hover_pos().map(|m| self.s2w(o, m));

        // ---- background, bed, grid ----
        painter.rect_filled(rect, 0.0, self.workspace_fill());
        let bed = egui::Rect::from_min_max(self.w2s(o, Pt::new(0.0, 0.0)), self.w2s(o, Pt::new(bw, bh)));
        painter.rect_filled(bed.translate(egui::vec2(3.0, 3.0)), 0.0, Color32::from_black_alpha(25));
        painter.rect_filled(bed, 0.0, self.bed_fill());
        let painter = painter.with_clip_rect(rect);
        {
            let gp = &self.ed.grid_prefs;
            let z = self.ed.view.zoom as f64;
            let lines = |step: f64, color: [u8; 4], width: f32| {
                // Skip grids so dense that the lines would merge into a tint.
                if step * z < 2.5 || step <= 0.0 {
                    return;
                }
                let st = Stroke::new(width, Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]));
                let mut x = 0.0;
                while x <= bw {
                    let sx = self.w2s(o, Pt::new(x, 0.0)).x;
                    painter.line_segment([Pos2::new(sx, bed.min.y), Pos2::new(sx, bed.max.y)], st);
                    x += step;
                }
                let mut y = 0.0;
                while y <= bh {
                    let sy = self.w2s(o, Pt::new(0.0, y)).y;
                    painter.line_segment([Pos2::new(bed.min.x, sy), Pos2::new(bed.max.x, sy)], st);
                    y += step;
                }
            };
            if gp.minor_on {
                lines(gp.minor_mm.min(self.ed.grid * 0.99), gp.minor_color, 0.6);
            }
            if self.ed.show_grid {
                lines(self.ed.grid, gp.main_color, 1.0);
            }
        }
        painter.rect_stroke(bed, 0.0, Stroke::new(1.0, Color32::from_gray(0x99)), egui::StrokeKind::Outside);
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

        // ---- shapes ----
        let preview = self.preview_on;
        let ctx2 = ui.ctx().clone();
        self.ensure_textures(&ctx2);
        for s in &self.doc.scene.shapes {
            let Kind::Image(im) = &s.kind else { continue };
            if !self.doc.layers[s.layer].visible {
                continue;
            }
            if let Some(tex) = self.ed.image_tex.get(&(s.id, im.invert)) {
                let corners = [Pt::new(0.0, 0.0), Pt::new(im.w, 0.0), Pt::new(im.w, im.h), Pt::new(0.0, im.h)];
                let uv = [egui::pos2(0.0, 0.0), egui::pos2(1.0, 0.0), egui::pos2(1.0, 1.0), egui::pos2(0.0, 1.0)];
                let tint = Color32::from_white_alpha(if preview { 70 } else { 255 });
                let mut mesh = egui::Mesh::with_texture(tex.id());
                for (c, t) in corners.iter().zip(uv) {
                    mesh.vertices.push(egui::epaint::Vertex { pos: self.w2s(o, s.xf.apply(*c)), uv: t, color: tint });
                }
                mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
                painter.add(egui::Shape::mesh(mesh));
            }
        }
        for s in &self.doc.scene.shapes {
            let layer = &self.doc.layers[s.layer];
            if !layer.visible {
                continue;
            }
            let mut c = col(layer.color);
            if preview {
                c = c.gamma_multiply(0.25);
            } else if !layer.output {
                c = c.gamma_multiply(0.45);
            }
            let st = Stroke::new(self.ed.grid_prefs.line_width, c);
            for p in s.polys() {
                let pts: Vec<Pos2> = p.pts.iter().map(|q| self.w2s(o, *q)).collect();
                if pts.len() < 2 {
                    continue;
                }
                painter.add(if p.closed { egui::Shape::closed_line(pts, st) } else { egui::Shape::line(pts, st) });
            }
        }
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

        // Padlock on locked objects.
        for s in self.doc.scene.shapes.iter().filter(|s| s.locked && self.doc.layers[s.layer].visible) {
            if let Some(b) = s.bounds() {
                let p = self.w2s(o, Pt::new(b.max.x, b.min.y));
                crate::icons::paint(ui, egui::Rect::from_center_size(p + egui::vec2(-8.0, 8.0), egui::vec2(14.0, 14.0)), "lock", theme::accent());
            }
        }
        // ---- selection ----
        if self.ed.sel.is_empty() || self.ed.tool != Tool::Select {
            self.ed.rotate_mode = false;
        }
        if let Some(b) = self.sel_bounds() {
            let sr = egui::Rect::from_min_max(self.w2s(o, b.min), self.w2s(o, b.max));
            // Red while the handles resize, blue while they turn and slant.
            let ink = if self.ed.rotate_mode { Color32::from_rgb(0x2f, 0x7f, 0xe8) } else { Color32::from_rgb(0xe0, 0x30, 0x30) };
            painter.rect_stroke(sr, 0.0, Stroke::new(1.0, ink), egui::StrokeKind::Outside);
            let all_locked = self.ed.sel.iter().filter_map(|id| self.doc.shape(*id)).all(|s| s.locked);
            if self.ed.tool == Tool::Select && !all_locked {
                for (i, h) in handle_pts(&b).into_iter().enumerate() {
                    let p = self.w2s(o, h);
                    if !self.ed.rotate_mode {
                        let r = egui::Rect::from_center_size(p, egui::vec2(8.0, 8.0));
                        painter.rect_filled(r, 1.0, Color32::WHITE);
                        painter.rect_stroke(r, 1.0, Stroke::new(1.2, ink), egui::StrokeKind::Middle);
                    } else if i % 2 == 0 {
                        // Corners turn the selection: round handles.
                        painter.circle_filled(p, 5.0, Color32::WHITE);
                        painter.circle_stroke(p, 5.0, Stroke::new(1.4, ink));
                    } else {
                        // Edges slant it: diamond handles.
                        let d = 5.5;
                        let pts = vec![p + egui::vec2(0.0, -d), p + egui::vec2(d, 0.0), p + egui::vec2(0.0, d), p + egui::vec2(-d, 0.0)];
                        painter.add(egui::Shape::convex_polygon(pts, Color32::WHITE, Stroke::new(1.4, ink)));
                    }
                }
            }
        }

        // ---- node overlay ----
        if self.ed.tool == Tool::Node {
            let acc = theme::accent();
            let sel_nodes = self.ed.node_sel.clone();
            for (id, cs) in self.bezier_shapes() {
                for (ci, c) in cs.iter().enumerate() {
                    for (ni, n) in c.nodes.iter().enumerate() {
                        let selected = sel_nodes.contains(&(id, ci, ni));
                        let p = self.w2s(o, n.p);
                        if selected {
                            for (has, h) in [(n.has_in(), n.hin), (n.has_out(), n.hout)] {
                                if has {
                                    let hp = self.w2s(o, h);
                                    painter.line_segment([p, hp], Stroke::new(1.0, acc));
                                    painter.circle_filled(hp, 3.5, Color32::WHITE);
                                    painter.circle_stroke(hp, 3.5, Stroke::new(1.2, acc));
                                }
                            }
                        }
                        let fill = if selected { acc } else { Color32::WHITE };
                        if n.smooth {
                            painter.circle_filled(p, 4.5, fill);
                            painter.circle_stroke(p, 4.5, Stroke::new(1.3, acc));
                        } else {
                            let r = egui::Rect::from_center_size(p, egui::vec2(8.0, 8.0));
                            painter.rect_filled(r, 1.0, fill);
                            painter.rect_stroke(r, 1.0, Stroke::new(1.3, acc), egui::StrokeKind::Middle);
                        }
                    }
                    if let Some((aid, aci, asi)) = self.ed.active_seg {
                        if aid == id && aci == ci && asi < c.seg_count() {
                            let poly: Vec<Pos2> = (0..=16).map(|k| self.w2s(o, c.point_at(asi, k as f64 / 16.0))).collect();
                            painter.add(egui::Shape::line(poly, Stroke::new(3.0, acc.gamma_multiply(0.6))));
                        }
                    }
                }
            }
        }

        // ---- interaction ----
        let mouse = resp.hover_pos();
        let hpos = resp.interact_pointer_pos().or(mouse);
        let press = if resp.drag_started() { ui.input(|i| i.pointer.press_origin()).or(hpos) } else { hpos };
        let shift = ui.input(|i| i.modifiers.shift);
        // Right click selects what is under the pointer, then the context menu offers the usual commands.
        if resp.secondary_clicked() && self.ed.tool == Tool::Select {
            if let Some(m) = hpos {
                match self.hit_shape(self.s2w(o, m)) {
                    Some(id) if !self.ed.sel.contains(&id) => self.ed.sel = self.doc.group_of(id),
                    None => self.ed.sel.clear(),
                    _ => {}
                }
            }
        }
        if !matches!(self.ed.tool, Tool::Pen | Tool::Pan | Tool::Zoom | Tool::ZoomOut) && !self.overlay_edit {
            let mut act: Option<Act> = None;
            resp.context_menu(|ui| act = self.context_menu(ui));
            if let Some(a) = act {
                let ctx = ui.ctx().clone();
                self.do_act(&ctx, a);
            }
        }
        let overlay_busy = self.overlay_interact(&resp, o, press, hpos);
        if !overlay_busy {
        match self.ed.tool {
            Tool::Select => {
                // Double-clicking a shape opens it in the node tool (text and pictures excluded).
                if resp.double_clicked() {
                    if let Some(m) = hpos {
                        if let Some(id) = self.hit_shape(self.s2w(o, m)) {
                            let editable = self.doc.shape(id).is_some_and(|s| !s.locked && !matches!(s.kind, Kind::Text(_) | Kind::Image(_)));
                            if editable {
                                self.ed.sel = self.doc.group_of(id);
                                self.ed.node_sel.clear();
                                self.ed.tool = Tool::Node;
                            }
                        }
                    }
                }
                if let Some(m) = mouse {
                    if let Some(h) = self.hit_handle(o, m) {
                        ui.ctx().set_cursor_icon(match (self.ed.rotate_mode, h) {
                            (true, 0 | 2 | 4 | 6) => CursorIcon::Alias,
                            (true, 1 | 5) => CursorIcon::ResizeHorizontal,
                            (true, _) => CursorIcon::ResizeVertical,
                            (false, 0 | 4) => CursorIcon::ResizeNwSe,
                            (false, 2 | 6) => CursorIcon::ResizeNeSw,
                            (false, 1 | 5) => CursorIcon::ResizeVertical,
                            (false, _) => CursorIcon::ResizeHorizontal,
                        });
                    } else if self.hit_shape(self.s2w(o, m)).is_some() {
                        ui.ctx().set_cursor_icon(CursorIcon::Move);
                    }
                }
                if resp.drag_started_by(PointerButton::Primary) {
                    if let Some(m) = press {
                        let w = self.s2w(o, m);
                        if let Some(h) = self.hit_handle(o, m) {
                            let b = self.sel_bounds().unwrap();
                            let hp = handle_pts(&b);
                            self.checkpoint();
                            let orig = self.ed.sel.iter().filter_map(|i| self.doc.shape(*i).filter(|s| !s.locked).map(|s| (*i, s.xf))).collect();
                            self.ed.drag = Some(if !self.ed.rotate_mode {
                                Drag::Scale { anchor: hp[(h + 4) % 8], grab: hp[h], handle: h, orig }
                            } else if h % 2 == 0 {
                                let centre = b.center();
                                Drag::Rotate { centre, from: (hp[h].y - centre.y).atan2(hp[h].x - centre.x), orig }
                            } else {
                                // Top and bottom edges slant sideways, left and right edges slant up and down.
                                Drag::Skew { horizontal: h % 4 == 1, anchor: hp[(h + 4) % 8], grab: hp[h], orig }
                            });
                        } else if let Some(id) = self.hit_shape(w) {
                            if !self.ed.sel.contains(&id) {
                                if !shift {
                                    self.ed.sel.clear();
                                }
                                for m in self.doc.group_of(id) {
                                    if !self.ed.sel.contains(&m) {
                                        self.ed.sel.push(m);
                                    }
                                }
                            }
                            self.checkpoint();
                            let orig = self.ed.sel.iter().filter_map(|i| self.doc.shape(*i).filter(|s| !s.locked).map(|s| (*i, s.xf))).collect();
                            self.ed.drag = Some(Drag::Move { start: w, orig });
                        } else {
                            if !shift {
                                self.ed.sel.clear();
                            }
                            self.ed.drag = Some(Drag::Marquee { start: w });
                        }
                    }
                } else if resp.clicked() && hpos.is_some_and(|m| self.hit_handle(o, m).is_some()) {
                    // A click on a handle swaps between resizing and turning / slanting.
                    self.ed.rotate_mode = !self.ed.rotate_mode;
                } else if resp.clicked() {
                    if let Some(m) = hpos {
                        let hit = self.hit_shape(self.s2w(o, m));
                        match hit {
                            Some(id) if shift => {
                                let members = self.doc.group_of(id);
                                if self.ed.sel.contains(&id) {
                                    self.ed.sel.retain(|x| !members.contains(x));
                                } else {
                                    self.ed.sel.extend(members);
                                }
                            }
                            Some(id) => self.ed.sel = self.doc.group_of(id),
                            None => self.ed.sel.clear(),
                        }
                    }
                }
            }
            Tool::Node => {
                self.nodes_prepare();
                if let Some(m) = mouse {
                    if self.hit_node_handle(o, m).is_some() || self.hit_node(o, m).is_some() {
                        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                    }
                }
                if resp.drag_started_by(PointerButton::Primary) {
                    if let Some(m) = press {
                        let w = self.s2w(o, m);
                        if let Some((nr, out)) = self.hit_node_handle(o, m) {
                            self.checkpoint();
                            self.ed.drag = Some(Drag::HandleMove { node: nr, out });
                        } else if let Some(nr) = self.hit_node(o, m) {
                            if !self.ed.node_sel.contains(&nr) {
                                if !shift {
                                    self.ed.node_sel.clear();
                                }
                                self.ed.node_sel.push(nr);
                            }
                            self.ed.active_seg = None;
                            self.checkpoint();
                            let orig: Vec<(NodeRef, Node)> = self.ed.node_sel.iter().filter_map(|r| self.node_at(*r).map(|n| (*r, n))).collect();
                            self.ed.drag = Some(Drag::NodeMove { start: w, orig });
                        } else {
                            if !shift {
                                self.ed.node_sel.clear();
                            }
                            self.ed.active_seg = None;
                            self.ed.drag = Some(Drag::NodeMarquee { start: w });
                        }
                    }
                } else if resp.double_clicked() {
                    if let Some(m) = hpos {
                        if let Some((seg, t)) = self.hit_segment(self.s2w(o, m)) {
                            self.nodes_insert(Some((seg, t)));
                        }
                    }
                } else if resp.clicked() {
                    if let Some(m) = hpos {
                        let w = self.s2w(o, m);
                        if let Some(nr) = self.hit_node(o, m) {
                            if shift {
                                if let Some(i) = self.ed.node_sel.iter().position(|x| *x == nr) {
                                    self.ed.node_sel.remove(i);
                                } else {
                                    self.ed.node_sel.push(nr);
                                }
                            } else {
                                self.ed.node_sel = vec![nr];
                            }
                            self.ed.active_seg = None;
                        } else if let Some((seg, _)) = self.hit_segment(w) {
                            self.ed.active_seg = Some(seg);
                            self.ed.node_sel.clear();
                        } else if let Some(id) = self.hit_shape(w) {
                            self.ed.sel = vec![id];
                            self.ed.node_sel.clear();
                            self.ed.active_seg = None;
                        } else {
                            self.ed.sel.clear();
                            self.ed.node_sel.clear();
                            self.ed.active_seg = None;
                        }
                    }
                }
            }
            Tool::Text => {
                ui.ctx().set_cursor_icon(CursorIcon::Text);
                if resp.clicked_by(PointerButton::Primary) {
                    if let Some(m) = hpos {
                        let p = self.snapped(self.s2w(o, m));
                        self.add_text_at(p);
                    }
                }
            }
            Tool::Rect | Tool::Ellipse | Tool::Triangle | Tool::Star | Tool::Polygon | Tool::Heart | Tool::Spiral | Tool::Line => {
                ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
                if resp.drag_started_by(PointerButton::Primary) {
                    if let Some(m) = press {
                        // Starting another polygon or spiral brings the small window back.
                        self.ed.show_shape_popup = matches!(self.ed.tool, Tool::Polygon | Tool::Spiral);
                        self.ed.drag = Some(Drag::Create { start: self.snapped(self.s2w(o, m)) });
                    }
                }
            }
            Tool::Pen => {
                ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
                if resp.clicked_by(PointerButton::Primary) {
                    if let Some(m) = hpos {
                        let p = self.snapped(self.s2w(o, m));
                        let close = self.ed.pen_pts.len() >= 3 && self.w2s(o, self.ed.pen_pts[0]).distance(m) < 9.0;
                        if close {
                            self.finish_pen(true);
                        } else {
                            self.ed.pen_pts.push(p);
                        }
                    }
                }
                if resp.double_clicked() {
                    self.ed.pen_pts.pop();
                    self.finish_pen(false);
                }
                if resp.secondary_clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.finish_pen(false);
                }
            }
            Tool::Zoom | Tool::ZoomOut => {
                // Zoom in clicks in, zoom out clicks out; Shift swaps them.
                let out = (self.ed.tool == Tool::ZoomOut) != shift;
                ui.ctx().set_cursor_icon(if out { CursorIcon::ZoomOut } else { CursorIcon::ZoomIn });
                if resp.clicked() {
                    if let Some(m) = hpos {
                        let f = if out { 0.5 } else { 2.0 };
                        let before = self.s2w(o, m);
                        self.ed.view.zoom = (self.ed.view.zoom * f).clamp(0.05, 200.0);
                        let after = self.w2s(o, before);
                        self.ed.view.pan += m - after;
                        self.ed.view.auto_fit = false;
                    }
                }
            }
            Tool::Pan => ui.ctx().set_cursor_icon(if resp.dragged() { CursorIcon::Grabbing } else { CursorIcon::Grab }),
        }
        }

        // drag update
        if resp.dragged_by(PointerButton::Primary) {
            if let (Some(m), Some(d)) = (hpos, self.ed.drag.as_ref()) {
                let w = self.s2w(o, m);
                match d {
                    Drag::Move { start, orig } => {
                        let (mut dx, mut dy) = (w.x - start.x, w.y - start.y);
                        if self.ed.snap {
                            dx = (dx / self.ed.grid).round() * self.ed.grid;
                            dy = (dy / self.ed.grid).round() * self.ed.grid;
                        }
                        let orig = orig.clone();
                        if self.ed.snap_objects {
                            // Where the selection would be, against the other objects: lines within a few pixels join.
                            let ids: Vec<u64> = orig.iter().map(|(id, _)| *id).collect();
                            if self.ed.snap_targets.is_none() {
                                let others = self.doc.scene.shapes.iter().filter(|s| !ids.contains(&s.id) && self.doc.layers[s.layer].visible).filter_map(|s| s.bounds()).collect();
                                self.ed.snap_targets = Some(others);
                            }
                            for (id, xf) in &orig {
                                if let Some(s) = self.doc.unlocked_mut(*id) {
                                    s.xf = *xf;
                                }
                            }
                            if let Some(b) = self.doc.bounds_of(&ids) {
                                let moved = Rect { min: Pt::new(b.min.x + dx, b.min.y + dy), max: Pt::new(b.max.x + dx, b.max.y + dy) };
                                let s = lc_core::snap::snap_to_objects(moved, self.ed.snap_targets.as_deref().unwrap_or(&[]), 8.0 / self.ed.view.zoom as f64);
                                dx += s.dx;
                                dy += s.dy;
                                self.ed.snap_guides = (s.guide_x, s.guide_y);
                            }
                        }
                        for (id, xf) in orig {
                            if let Some(s) = self.doc.unlocked_mut(id) {
                                s.xf = xf.then(Xf::translate(dx, dy));
                            }
                        }
                        self.touch();
                    }
                    Drag::Scale { anchor, grab, handle, orig } => {
                        let (anchor, grab, handle) = (*anchor, *grab, *handle);
                        let w = self.snapped(w);
                        let rx = if (grab.x - anchor.x).abs() > 1e-6 { (w.x - anchor.x) / (grab.x - anchor.x) } else { 1.0 };
                        let ry = if (grab.y - anchor.y).abs() > 1e-6 { (w.y - anchor.y) / (grab.y - anchor.y) } else { 1.0 };
                        let (mut sx, mut sy) = match handle {
                            1 | 5 => (1.0, ry),
                            3 | 7 => (rx, 1.0),
                            _ => (rx, ry),
                        };
                        if matches!(handle, 0 | 2 | 4 | 6) && !shift {
                            let k = if rx.abs() > ry.abs() { rx } else { ry };
                            sx = k;
                            sy = k;
                        }
                        let (sx, sy) = (if sx.abs() < 0.001 { 0.001 } else { sx }, if sy.abs() < 0.001 { 0.001 } else { sy });
                        let orig = orig.clone();
                        for (id, xf) in orig {
                            if let Some(s) = self.doc.unlocked_mut(id) {
                                s.xf = xf.then(Xf::scale_about(sx, sy, anchor));
                            }
                        }
                        self.touch();
                    }
                    Drag::Rotate { centre, from, orig } => {
                        let (centre, from) = (*centre, *from);
                        let mut a = (w.y - centre.y).atan2(w.x - centre.x) - from;
                        if shift {
                            let step = 15f64.to_radians();
                            a = (a / step).round() * step;
                        }
                        let orig = orig.clone();
                        for (id, xf) in orig {
                            if let Some(s) = self.doc.unlocked_mut(id) {
                                s.xf = xf.then(Xf::rotate_about(a, centre));
                            }
                        }
                        self.touch();
                    }
                    Drag::Skew { horizontal, anchor, grab, orig } => {
                        let (horizontal, anchor, grab) = (*horizontal, *anchor, *grab);
                        let t = if horizontal {
                            let span = grab.y - anchor.y;
                            let k = if span.abs() > 1e-6 { (w.x - grab.x) / span } else { 0.0 };
                            Xf::skew_about(k.clamp(-4.0, 4.0), 0.0, anchor)
                        } else {
                            let span = grab.x - anchor.x;
                            let k = if span.abs() > 1e-6 { (w.y - grab.y) / span } else { 0.0 };
                            Xf::skew_about(0.0, k.clamp(-4.0, 4.0), anchor)
                        };
                        let orig = orig.clone();
                        for (id, xf) in orig {
                            if let Some(s) = self.doc.unlocked_mut(id) {
                                s.xf = xf.then(t);
                            }
                        }
                        self.touch();
                    }
                    Drag::NodeMove { start, orig } => {
                        let (mut dx, mut dy) = (w.x - start.x, w.y - start.y);
                        if self.ed.snap {
                            dx = (dx / self.ed.grid).round() * self.ed.grid;
                            dy = (dy / self.ed.grid).round() * self.ed.grid;
                        }
                        let orig = orig.clone();
                        for ((id, ci, ni), n0) in orig {
                            if let Some(c) = self.contour_mut(id, ci) {
                                if ni < c.nodes.len() {
                                    let mut n = n0;
                                    n.translate(dx, dy);
                                    c.nodes[ni] = n;
                                }
                            }
                        }
                        self.touch();
                    }
                    Drag::HandleMove { node, out } => {
                        let (node, out) = (*node, *out);
                        let w = self.snapped(w);
                        if let Some(c) = self.contour_mut(node.0, node.1) {
                            if node.2 < c.nodes.len() {
                                c.drag_handle(node.2, out, w);
                            }
                        }
                        self.touch();
                    }
                    Drag::NodeMarquee { start } => {
                        let r = egui::Rect::from_two_pos(self.w2s(o, *start), m);
                        painter.rect_filled(r, 0.0, theme::accent().gamma_multiply(0.1));
                        painter.rect_stroke(r, 0.0, Stroke::new(1.0, theme::accent()), egui::StrokeKind::Middle);
                    }
                    Drag::Marquee { start } => {
                        // Left to right selects what is wholly inside (blue); right to left also what is only touched (green).
                        let c = if w.x >= start.x { theme::accent() } else { Color32::from_rgb(0x2e, 0xa0, 0x4f) };
                        let r = egui::Rect::from_two_pos(self.w2s(o, *start), m);
                        painter.rect_filled(r, 0.0, c.gamma_multiply(0.1));
                        painter.rect_stroke(r, 0.0, Stroke::new(1.0, c), egui::StrokeKind::Middle);
                    }
                    Drag::Create { start } => {
                        let end = self.snapped(w);
                        let st = Stroke::new(self.ed.grid_prefs.line_width, col(self.ed.active_layer));
                        let (a, b) = (self.w2s(o, *start), self.w2s(o, end));
                        match self.ed.tool {
                            Tool::Rect => {
                                painter.rect_stroke(egui::Rect::from_two_pos(a, b), 0.0, st, egui::StrokeKind::Middle);
                            }
                            Tool::Ellipse => {
                                let r = egui::Rect::from_two_pos(a, b);
                                let pts: Vec<Pos2> = (0..64)
                                    .map(|i| {
                                        let t = i as f32 / 64.0 * std::f32::consts::TAU;
                                        r.center() + egui::vec2(t.cos() * r.width() / 2.0, t.sin() * r.height() / 2.0)
                                    })
                                    .collect();
                                painter.add(egui::Shape::closed_line(pts, st));
                            }
                            Tool::Triangle | Tool::Star | Tool::Polygon | Tool::Heart | Tool::Spiral => {
                                let r = egui::Rect::from_two_pos(a, b);
                                let (wm, hm) = ((r.width() / self.ed.view.zoom) as f64, (r.height() / self.ed.view.zoom) as f64);
                                if let Some(ct) = self.auto_contour(wm, hm) {
                                    let pts: Vec<Pos2> = ct.flatten(0.1).pts.iter().map(|p| r.min + egui::vec2(p.x as f32 * self.ed.view.zoom, p.y as f32 * self.ed.view.zoom)).collect();
                                    painter.add(if ct.closed { egui::Shape::closed_line(pts, st) } else { egui::Shape::line(pts, st) });
                                }
                            }
                            _ => {
                                painter.line_segment([a, b], st);
                            }
                        }
                    }
                }
            }
        }
        // Guide lines of the object snapping, through the whole work area.
        if self.ed.drag.is_some() {
            let st = Stroke::new(1.0, Color32::from_rgb(0xe0, 0x30, 0xc0));
            if let Some(x) = self.ed.snap_guides.0 {
                let sx = self.w2s(o, Pt::new(x, 0.0)).x;
                painter.line_segment([Pos2::new(sx, bed.min.y), Pos2::new(sx, bed.max.y)], st);
            }
            if let Some(y) = self.ed.snap_guides.1 {
                let sy = self.w2s(o, Pt::new(0.0, y)).y;
                painter.line_segment([Pos2::new(bed.min.x, sy), Pos2::new(bed.max.x, sy)], st);
            }
        }
        if resp.drag_stopped_by(PointerButton::Primary) {
            self.ed.snap_guides = (None, None);
            self.ed.snap_targets = None;
            if let (Some(d), Some(m)) = (self.ed.drag.take(), hpos) {
                let w = self.s2w(o, m);
                match d {
                    Drag::Marquee { start } => {
                        let r = Rect::from_pts([start, w]).unwrap();
                        let hits = self.marquee_hits(&r, w.x >= start.x);
                        for id in hits {
                            for m in self.doc.group_of(id) {
                                if !self.ed.sel.contains(&m) {
                                    self.ed.sel.push(m);
                                }
                            }
                        }
                    }
                    Drag::NodeMarquee { start } => {
                        let r = Rect::from_pts([start, w]).unwrap();
                        let mut hits = vec![];
                        for (id, cs) in self.bezier_shapes() {
                            for (ci, c) in cs.iter().enumerate() {
                                for (ni, n) in c.nodes.iter().enumerate() {
                                    if r.contains(n.p) {
                                        hits.push((id, ci, ni));
                                    }
                                }
                            }
                        }
                        for h in hits {
                            if !self.ed.node_sel.contains(&h) {
                                self.ed.node_sel.push(h);
                            }
                        }
                    }
                    Drag::Create { start } => {
                        let mut end = self.snapped(w);
                        if shift && matches!(self.ed.tool, Tool::Rect | Tool::Ellipse | Tool::Triangle | Tool::Star | Tool::Polygon | Tool::Heart | Tool::Spiral) {
                            let s = (end.x - start.x).abs().max((end.y - start.y).abs());
                            end = Pt::new(start.x + s * (end.x - start.x).signum(), start.y + s * (end.y - start.y).signum());
                        }
                        let r = Rect::from_pts([start, end]).unwrap();
                        if r.width().max(r.height()) > 0.2 {
                            self.checkpoint();
                            let layer = self.ed.active_layer;
                            let id = match self.ed.tool {
                                Tool::Rect => self.doc.add(layer, Kind::Rect { w: r.width(), h: r.height() }, Xf::translate(r.min.x, r.min.y)),
                                Tool::Ellipse => self.doc.add(layer, Kind::Ellipse { w: r.width(), h: r.height() }, Xf::translate(r.min.x, r.min.y)),
                                Tool::Triangle | Tool::Star | Tool::Polygon | Tool::Heart | Tool::Spiral => match self.auto_contour(r.width(), r.height()) {
                                    Some(ct) => self.doc.add(layer, Kind::Bezier(vec![ct]), Xf::translate(r.min.x, r.min.y)),
                                    None => 0,
                                },
                                _ => self.doc.add(layer, Kind::Path(vec![Polyline::new(vec![start, end], false)]), Xf::IDENTITY),
                            };
                            self.ed.sel = vec![id];
                            if matches!(self.ed.tool, Tool::Polygon | Tool::Spiral) {
                                // The small window stays while the tool is active; its number now edits this shape.
                                self.ed.last_param = Some((self.ed.tool, id, r.width(), r.height()));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // pen preview
        if self.ed.tool == Tool::Pen && !self.ed.pen_pts.is_empty() {
            let st = Stroke::new(self.ed.grid_prefs.line_width, col(self.ed.active_layer));
            let mut pts: Vec<Pos2> = self.ed.pen_pts.iter().map(|p| self.w2s(o, *p)).collect();
            if let Some(m) = mouse {
                pts.push(m);
            }
            for p in &pts {
                painter.circle_filled(*p, 3.0, theme::accent());
            }
            painter.add(egui::Shape::line(pts, st));
        }

        self.rulers(&painter, rect);

        // Floating node-editing toolbar: it goes away when you click another window, such as the preview, and comes
        // back when you click the work area again.
        if self.ed.tool != Tool::Node {
            self.ed.node_bar_active = true;
        } else if let Some(p) = ui.input(|i| if i.pointer.any_pressed() { i.pointer.interact_pos() } else { None }) {
            self.ed.node_bar_active = ui.ctx().layer_id_at(p).map_or(true, |l| l.order == egui::Order::Background || l.id == egui::Id::new("node_bar"));
        }
        if self.ed.tool == Tool::Node && self.ed.node_bar_active {
            let ctx = ui.ctx().clone();
            egui::Area::new(egui::Id::new("node_bar")).order(egui::Order::Middle).fixed_pos(rect.min + egui::vec2(28.0, 26.0)).show(&ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| self.node_bar(ui));
                });
            });
        }
    }

    fn rulers(&self, painter: &egui::Painter, rect: egui::Rect) {
        let t = 18.0;
        let bg = theme::panel();
        painter.rect_filled(egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), t)), 0.0, bg);
        painter.rect_filled(egui::Rect::from_min_size(rect.min, egui::vec2(t, rect.height())), 0.0, bg);
        // Work in display units: `k` is millimetres per unit, `z` pixels per millimetre.
        let units = self.doc.device.units;
        let k = units.to_mm(1.0);
        let z = self.ed.view.zoom as f64;
        let major = nice_step(60.0 / (z * k));
        let minor = major / 5.0;
        let decimals = if major >= 1.0 { 0 } else { (-major.log10()).ceil() as usize };
        let label = |v: f64| format!("{:.*}", decimals, v);
        let font = egui::FontId::proportional(9.5);
        let tick = Stroke::new(1.0, theme::text_dim());
        // horizontal
        let (x0, x1) = ((-self.ed.view.pan.x) as f64 / (z * k), (rect.width() - self.ed.view.pan.x) as f64 / (z * k));
        let mut v = (x0 / minor).floor() * minor;
        while v <= x1 {
            let sx = rect.min.x + self.ed.view.pan.x + (v * k * z) as f32;
            let is_major = ((v / major).round() * major - v).abs() < minor * 0.01;
            if sx > rect.min.x + t {
                painter.line_segment([Pos2::new(sx, rect.min.y + if is_major { 2.0 } else { 12.0 }), Pos2::new(sx, rect.min.y + t)], tick);
                if is_major {
                    painter.text(Pos2::new(sx + 2.0, rect.min.y + 1.0), egui::Align2::LEFT_TOP, label(v), font.clone(), theme::text_dim());
                }
            }
            v += minor;
        }
        let (y0, y1) = ((-self.ed.view.pan.y) as f64 / (z * k), (rect.height() - self.ed.view.pan.y) as f64 / (z * k));
        let mut v = (y0 / minor).floor() * minor;
        while v <= y1 {
            let sy = rect.min.y + self.ed.view.pan.y + (v * k * z) as f32;
            let is_major = ((v / major).round() * major - v).abs() < minor * 0.01;
            if sy > rect.min.y + t {
                painter.line_segment([Pos2::new(rect.min.x + if is_major { 2.0 } else { 12.0 }, sy), Pos2::new(rect.min.x + t, sy)], tick);
                if is_major {
                    painter.text(Pos2::new(rect.min.x + 1.0, sy + 2.0), egui::Align2::LEFT_TOP, label(v), font.clone(), theme::text_dim());
                }
            }
            v += minor;
        }
        painter.rect_filled(egui::Rect::from_min_size(rect.min, egui::vec2(t, t)), 0.0, theme::panel_dark());
    }
}
