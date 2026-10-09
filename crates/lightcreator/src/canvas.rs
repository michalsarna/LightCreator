use crate::app::{App, Tool};
use crate::menu::Act;
use crate::theme;
use eframe::egui::{self, Color32, CursorIcon, PointerButton, Pos2, Sense, Stroke};
use crate::nodes::NodeRef;
use lc_core::{Kind, Node, Polyline, Pt, Rect, Xf, PALETTE};

pub enum Drag {
    Move { start: Pt, orig: Vec<(u64, Xf)> },
    Scale { anchor: Pt, grab: Pt, handle: usize, orig: Vec<(u64, Xf)> },
    Marquee { start: Pt },
    Create { start: Pt },
    NodeMove { start: Pt, orig: Vec<(NodeRef, Node)> },
    HandleMove { node: NodeRef, out: bool },
    NodeMarquee { start: Pt },
}

fn col(i: usize) -> Color32 {
    let c = PALETTE[i.min(29)];
    Color32::from_rgb(c[0], c[1], c[2])
}

/// 8 resize handles: TL, T, TR, R, BR, B, BL, L.
fn handle_pts(b: &Rect) -> [Pt; 8] {
    let (l, r, t, bt) = (b.min.x, b.max.x, b.min.y, b.max.y);
    let (cx, cy) = (b.center().x, b.center().y);
    [Pt::new(l, t), Pt::new(cx, t), Pt::new(r, t), Pt::new(r, cy), Pt::new(r, bt), Pt::new(cx, bt), Pt::new(l, bt), Pt::new(l, cy)]
}

fn nice_step(min_mm: f64) -> f64 {
    for m in [1.0, 2.0, 5.0] .iter().cycle().zip(0..).map(|(m, i)| m * 10f64.powi(i / 3 - 2)) {
        if m >= min_mm {
            return m;
        }
    }
    1000.0
}

impl App {
    fn w2s(&self, o: Pos2, p: Pt) -> Pos2 {
        o + self.view.pan + egui::vec2(p.x as f32 * self.view.zoom, p.y as f32 * self.view.zoom)
    }
    fn s2w(&self, o: Pos2, s: Pos2) -> Pt {
        let v = s - o - self.view.pan;
        Pt::new((v.x / self.view.zoom) as f64, (v.y / self.view.zoom) as f64)
    }
    fn snapped(&self, p: Pt) -> Pt {
        if self.snap {
            let g = self.grid;
            Pt::new((p.x / g).round() * g, (p.y / g).round() * g)
        } else {
            p
        }
    }

    fn hit_shape(&self, p: Pt) -> Option<u64> {
        let tol = 5.0 / self.view.zoom as f64;
        let visible = || self.doc.shapes.iter().rev().filter(|s| self.doc.layers[s.layer].visible);
        if let Some(s) = visible().find(|s| s.polys().iter().any(|q| q.dist_to(p) <= tol)) {
            return Some(s.id);
        }
        // Inside a closed shape: pick the smallest one that contains the point.
        visible()
            .filter_map(|s| {
                let polys = s.polys();
                let inside = polys.iter().filter(|q| q.closed).fold(false, |a, q| a ^ q.contains(p));
                inside.then(|| (s.id, polys.iter().map(|q| q.area().abs()).sum::<f64>()))
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(id, _)| id)
    }

    fn hit_handle(&self, o: Pos2, mouse: Pos2) -> Option<usize> {
        let b = self.sel_bounds()?;
        handle_pts(&b).iter().position(|h| self.w2s(o, *h).distance(mouse) <= 8.0)
    }

    /// Selected Bézier shapes with their id.
    fn bezier_shapes(&self) -> Vec<(u64, &Vec<lc_core::Contour>)> {
        self.sel
            .iter()
            .filter_map(|id| match &self.doc.shape(*id)?.kind {
                Kind::Bezier(cs) => Some((*id, cs)),
                _ => None,
            })
            .collect()
    }

    fn hit_node(&self, o: Pos2, mouse: Pos2) -> Option<NodeRef> {
        let mut best: Option<(f32, NodeRef)> = None;
        for (id, cs) in self.bezier_shapes() {
            for (ci, c) in cs.iter().enumerate() {
                for (ni, n) in c.nodes.iter().enumerate() {
                    let d = self.w2s(o, n.p).distance(mouse);
                    if d <= 8.0 && best.map_or(true, |b| d < b.0) {
                        best = Some((d, (id, ci, ni)));
                    }
                }
            }
        }
        best.map(|b| b.1)
    }

    /// A handle of a selected node under the mouse: (node, is_outgoing).
    fn hit_node_handle(&self, o: Pos2, mouse: Pos2) -> Option<(NodeRef, bool)> {
        for r in &self.node_sel {
            let Some(n) = self.node_at(*r) else { continue };
            if n.has_out() && self.w2s(o, n.hout).distance(mouse) <= 7.0 {
                return Some((*r, true));
            }
            if n.has_in() && self.w2s(o, n.hin).distance(mouse) <= 7.0 {
                return Some((*r, false));
            }
        }
        None
    }

    /// The segment of a selected Bézier shape close to `w`: ((shape, contour, segment), t).
    fn hit_segment(&self, w: Pt) -> Option<(NodeRef, f64)> {
        let tol = 6.0 / self.view.zoom as f64;
        let mut best: Option<(f64, NodeRef, f64)> = None;
        for (id, cs) in self.bezier_shapes() {
            for (ci, c) in cs.iter().enumerate() {
                if let Some((seg, t, d)) = c.nearest(w) {
                    if d <= tol && best.map_or(true, |b| d < b.0) {
                        best = Some((d, (id, ci, seg), t));
                    }
                }
            }
        }
        best.map(|b| (b.1, b.2))
    }

    /// Upload image textures that are missing and drop those of deleted images.
    fn ensure_textures(&mut self, ctx: &egui::Context) {
        let mut keep = vec![];
        for s in &self.doc.shapes {
            if let Kind::Image(im) = &s.kind {
                let key = (s.id, im.invert);
                keep.push(key);
                if !self.image_tex.contains_key(&key) {
                    let px: Vec<u8> = if im.invert { im.gray.iter().map(|g| 255 - g).collect() } else { im.gray.clone() };
                    let img = egui::ColorImage::from_gray([im.px_w as usize, im.px_h as usize], &px);
                    let tex = ctx.load_texture(format!("img-{}-{}", s.id, im.invert), img, egui::TextureOptions::LINEAR);
                    self.image_tex.insert(key, tex);
                }
            }
        }
        self.image_tex.retain(|k, _| keep.contains(k));
    }

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

    fn finish_pen(&mut self, closed: bool) {
        if self.pen_pts.len() >= 2 {
            self.checkpoint();
            let pts = std::mem::take(&mut self.pen_pts);
            let id = self.doc.add(self.active_layer, Kind::Path(vec![Polyline::new(pts, closed)]), Xf::IDENTITY);
            self.sel = vec![id];
        }
        self.pen_pts.clear();
    }

    pub fn canvas(&mut self, ui: &mut egui::Ui) {
        let avail = ui.available_size();
        let (resp, painter) = ui.allocate_painter(avail, Sense::click_and_drag());
        let rect = resp.rect;
        let o = rect.min;
        let (bw, bh) = (self.doc.device.bed_w, self.doc.device.bed_h);

        if self.view.need_fit && rect.width() > 50.0 {
            let z = ((rect.width() - 100.0) / bw as f32).min((rect.height() - 100.0) / bh as f32).max(0.05);
            self.view.zoom = z;
            self.view.pan = egui::vec2((rect.width() - bw as f32 * z) / 2.0, (rect.height() - bh as f32 * z) / 2.0);
            self.view.need_fit = false;
        }

        // ---- zoom / pan ----
        if resp.hovered() {
            let (scroll, pinch) = ui.input(|i| (i.smooth_scroll_delta.y, i.zoom_delta()));
            let f = (scroll * 0.0015).exp() * pinch;
            if (f - 1.0).abs() > 1e-4 {
                if let Some(m) = resp.hover_pos() {
                    let before = self.s2w(o, m);
                    self.view.zoom = (self.view.zoom * f).clamp(0.05, 200.0);
                    let after = self.w2s(o, before);
                    self.view.pan += m - after;
                }
            }
        }
        if resp.dragged_by(PointerButton::Middle) || (self.tool == Tool::Pan && resp.dragged_by(PointerButton::Primary)) {
            self.view.pan += resp.drag_delta();
        }
        self.cursor_mm = resp.hover_pos().map(|m| self.s2w(o, m));

        // ---- background, bed, grid ----
        painter.rect_filled(rect, 0.0, theme::workspace());
        let bed = egui::Rect::from_min_max(self.w2s(o, Pt::new(0.0, 0.0)), self.w2s(o, Pt::new(bw, bh)));
        painter.rect_filled(bed.translate(egui::vec2(3.0, 3.0)), 0.0, Color32::from_black_alpha(25));
        painter.rect_filled(bed, 0.0, Color32::WHITE);
        let painter = painter.with_clip_rect(rect);
        {
            let gp = &self.grid_prefs;
            let z = self.view.zoom as f64;
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
                lines(gp.minor_mm.min(self.grid * 0.99), gp.minor_color, 0.6);
            }
            if self.show_grid {
                lines(self.grid, gp.main_color, 1.0);
            }
        }
        painter.rect_stroke(bed, 0.0, Stroke::new(1.0, Color32::from_gray(0x99)), egui::StrokeKind::Outside);
        // Origin marker (machine zero)
        let zero = match self.doc.device.origin {
            lc_core::Origin::FrontLeft => Pt::new(0.0, bh),
            lc_core::Origin::BackLeft => Pt::new(0.0, 0.0),
        };
        painter.circle_filled(self.w2s(o, zero), 4.0, theme::accent());

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
        for s in &self.doc.shapes {
            let Kind::Image(im) = &s.kind else { continue };
            if !self.doc.layers[s.layer].visible {
                continue;
            }
            if let Some(tex) = self.image_tex.get(&(s.id, im.invert)) {
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
        for s in &self.doc.shapes {
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
            let st = Stroke::new(1.5, c);
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
                let mut shapes = Vec::with_capacity(job.moves.len());
                for m in &job.moves {
                    if !m.laser && skip_travel {
                        continue;
                    }
                    let st = if m.laser { Stroke::new(1.2, col(self.doc.layers[m.layer].color)) } else { Stroke::new(0.6, Color32::from_rgb(0xb0, 0xb0, 0xc0)) };
                    shapes.push(egui::Shape::line_segment([self.w2s(o, m.a), self.w2s(o, m.b)], st));
                }
                painter.extend(shapes);
                let r = painter.text(rect.left_bottom() + egui::vec2(30.0, -12.0), egui::Align2::LEFT_BOTTOM, format!("{} {}   {} {}", crate::i18n::tr("Estimated time"), crate::app::fmt_time(job.est_seconds), crate::i18n::tr("cut length"), crate::units_ui::fmt_len(self.doc.device.units, job.cut_length, 0)), egui::FontId::proportional(13.0), theme::text());
                let _ = r;
            }
        }

        // ---- selection ----
        if let Some(b) = self.sel_bounds() {
            let sr = egui::Rect::from_min_max(self.w2s(o, b.min), self.w2s(o, b.max));
            painter.rect_stroke(sr, 0.0, Stroke::new(1.0, theme::accent()), egui::StrokeKind::Outside);
            if self.tool == Tool::Select {
                for h in handle_pts(&b) {
                    let r = egui::Rect::from_center_size(self.w2s(o, h), egui::vec2(8.0, 8.0));
                    painter.rect_filled(r, 1.0, Color32::WHITE);
                    painter.rect_stroke(r, 1.0, Stroke::new(1.2, theme::accent()), egui::StrokeKind::Middle);
                }
            }
        }

        // ---- node overlay ----
        if self.tool == Tool::Node {
            let acc = theme::accent();
            let sel_nodes = self.node_sel.clone();
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
                    if let Some((aid, aci, asi)) = self.active_seg {
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
        if resp.secondary_clicked() && self.tool == Tool::Select {
            if let Some(m) = hpos {
                match self.hit_shape(self.s2w(o, m)) {
                    Some(id) if !self.sel.contains(&id) => self.sel = self.doc.group_of(id),
                    None => self.sel.clear(),
                    _ => {}
                }
            }
        }
        if !matches!(self.tool, Tool::Pen | Tool::Pan | Tool::Zoom) && !self.overlay_edit {
            let mut act: Option<Act> = None;
            resp.context_menu(|ui| act = self.context_menu(ui));
            if let Some(a) = act {
                let ctx = ui.ctx().clone();
                self.do_act(&ctx, a);
            }
        }
        let overlay_busy = self.overlay_interact(&resp, o, press, hpos);
        if !overlay_busy {
        match self.tool {
            Tool::Select => {
                if let Some(m) = mouse {
                    if let Some(h) = self.hit_handle(o, m) {
                        ui.ctx().set_cursor_icon(match h {
                            0 | 4 => CursorIcon::ResizeNwSe,
                            2 | 6 => CursorIcon::ResizeNeSw,
                            1 | 5 => CursorIcon::ResizeVertical,
                            _ => CursorIcon::ResizeHorizontal,
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
                            let orig = self.sel.iter().filter_map(|i| self.doc.shape(*i).map(|s| (*i, s.xf))).collect();
                            self.drag = Some(Drag::Scale { anchor: hp[(h + 4) % 8], grab: hp[h], handle: h, orig });
                        } else if let Some(id) = self.hit_shape(w) {
                            if !self.sel.contains(&id) {
                                if !shift {
                                    self.sel.clear();
                                }
                                for m in self.doc.group_of(id) {
                                    if !self.sel.contains(&m) {
                                        self.sel.push(m);
                                    }
                                }
                            }
                            self.checkpoint();
                            let orig = self.sel.iter().filter_map(|i| self.doc.shape(*i).map(|s| (*i, s.xf))).collect();
                            self.drag = Some(Drag::Move { start: w, orig });
                        } else {
                            if !shift {
                                self.sel.clear();
                            }
                            self.drag = Some(Drag::Marquee { start: w });
                        }
                    }
                } else if resp.clicked() {
                    if let Some(m) = hpos {
                        let hit = self.hit_shape(self.s2w(o, m));
                        match hit {
                            Some(id) if shift => {
                                let members = self.doc.group_of(id);
                                if self.sel.contains(&id) {
                                    self.sel.retain(|x| !members.contains(x));
                                } else {
                                    self.sel.extend(members);
                                }
                            }
                            Some(id) => self.sel = self.doc.group_of(id),
                            None => self.sel.clear(),
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
                            self.drag = Some(Drag::HandleMove { node: nr, out });
                        } else if let Some(nr) = self.hit_node(o, m) {
                            if !self.node_sel.contains(&nr) {
                                if !shift {
                                    self.node_sel.clear();
                                }
                                self.node_sel.push(nr);
                            }
                            self.active_seg = None;
                            self.checkpoint();
                            let orig: Vec<(NodeRef, Node)> = self.node_sel.iter().filter_map(|r| self.node_at(*r).map(|n| (*r, n))).collect();
                            self.drag = Some(Drag::NodeMove { start: w, orig });
                        } else {
                            if !shift {
                                self.node_sel.clear();
                            }
                            self.active_seg = None;
                            self.drag = Some(Drag::NodeMarquee { start: w });
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
                                if let Some(i) = self.node_sel.iter().position(|x| *x == nr) {
                                    self.node_sel.remove(i);
                                } else {
                                    self.node_sel.push(nr);
                                }
                            } else {
                                self.node_sel = vec![nr];
                            }
                            self.active_seg = None;
                        } else if let Some((seg, _)) = self.hit_segment(w) {
                            self.active_seg = Some(seg);
                            self.node_sel.clear();
                        } else if let Some(id) = self.hit_shape(w) {
                            self.sel = vec![id];
                            self.node_sel.clear();
                            self.active_seg = None;
                        } else {
                            self.sel.clear();
                            self.node_sel.clear();
                            self.active_seg = None;
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
            Tool::Rect | Tool::Ellipse | Tool::Line => {
                ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
                if resp.drag_started_by(PointerButton::Primary) {
                    if let Some(m) = press {
                        self.drag = Some(Drag::Create { start: self.snapped(self.s2w(o, m)) });
                    }
                }
            }
            Tool::Pen => {
                ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
                if resp.clicked_by(PointerButton::Primary) {
                    if let Some(m) = hpos {
                        let p = self.snapped(self.s2w(o, m));
                        let close = self.pen_pts.len() >= 3 && self.w2s(o, self.pen_pts[0]).distance(m) < 9.0;
                        if close {
                            self.finish_pen(true);
                        } else {
                            self.pen_pts.push(p);
                        }
                    }
                }
                if resp.double_clicked() {
                    self.pen_pts.pop();
                    self.finish_pen(false);
                }
                if resp.secondary_clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.finish_pen(false);
                }
            }
            Tool::Zoom => {
                ui.ctx().set_cursor_icon(CursorIcon::ZoomIn);
                if resp.clicked() {
                    if let Some(m) = hpos {
                        let f = if shift { 0.5 } else { 2.0 };
                        let before = self.s2w(o, m);
                        self.view.zoom = (self.view.zoom * f).clamp(0.05, 200.0);
                        let after = self.w2s(o, before);
                        self.view.pan += m - after;
                    }
                }
            }
            Tool::Pan => ui.ctx().set_cursor_icon(if resp.dragged() { CursorIcon::Grabbing } else { CursorIcon::Grab }),
        }
        }

        // drag update
        if resp.dragged_by(PointerButton::Primary) {
            if let (Some(m), Some(d)) = (hpos, self.drag.as_ref()) {
                let w = self.s2w(o, m);
                match d {
                    Drag::Move { start, orig } => {
                        let (mut dx, mut dy) = (w.x - start.x, w.y - start.y);
                        if self.snap {
                            dx = (dx / self.grid).round() * self.grid;
                            dy = (dy / self.grid).round() * self.grid;
                        }
                        let orig = orig.clone();
                        for (id, xf) in orig {
                            if let Some(s) = self.doc.shape_mut(id) {
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
                            if let Some(s) = self.doc.shape_mut(id) {
                                s.xf = xf.then(Xf::scale_about(sx, sy, anchor));
                            }
                        }
                        self.touch();
                    }
                    Drag::NodeMove { start, orig } => {
                        let (mut dx, mut dy) = (w.x - start.x, w.y - start.y);
                        if self.snap {
                            dx = (dx / self.grid).round() * self.grid;
                            dy = (dy / self.grid).round() * self.grid;
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
                        let r = egui::Rect::from_two_pos(self.w2s(o, *start), m);
                        painter.rect_filled(r, 0.0, theme::accent().gamma_multiply(0.1));
                        painter.rect_stroke(r, 0.0, Stroke::new(1.0, theme::accent()), egui::StrokeKind::Middle);
                    }
                    Drag::Create { start } => {
                        let end = self.snapped(w);
                        let st = Stroke::new(1.5, col(self.active_layer));
                        let (a, b) = (self.w2s(o, *start), self.w2s(o, end));
                        match self.tool {
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
                            _ => {
                                painter.line_segment([a, b], st);
                            }
                        }
                    }
                }
            }
        }
        if resp.drag_stopped_by(PointerButton::Primary) {
            if let (Some(d), Some(m)) = (self.drag.take(), hpos) {
                let w = self.s2w(o, m);
                match d {
                    Drag::Marquee { start } => {
                        let r = Rect::from_pts([start, w]).unwrap();
                        let hits: Vec<u64> = self
                            .doc
                            .shapes
                            .iter()
                            .filter(|s| self.doc.layers[s.layer].visible)
                            .filter(|s| s.bounds().is_some_and(|b| r.contains(b.min) && r.contains(b.max)))
                            .map(|s| s.id)
                            .collect();
                        for id in hits {
                            for m in self.doc.group_of(id) {
                                if !self.sel.contains(&m) {
                                    self.sel.push(m);
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
                            if !self.node_sel.contains(&h) {
                                self.node_sel.push(h);
                            }
                        }
                    }
                    Drag::Create { start } => {
                        let mut end = self.snapped(w);
                        if shift && matches!(self.tool, Tool::Rect | Tool::Ellipse) {
                            let s = (end.x - start.x).abs().max((end.y - start.y).abs());
                            end = Pt::new(start.x + s * (end.x - start.x).signum(), start.y + s * (end.y - start.y).signum());
                        }
                        let r = Rect::from_pts([start, end]).unwrap();
                        if r.width().max(r.height()) > 0.2 {
                            self.checkpoint();
                            let layer = self.active_layer;
                            let id = match self.tool {
                                Tool::Rect => self.doc.add(layer, Kind::Rect { w: r.width(), h: r.height() }, Xf::translate(r.min.x, r.min.y)),
                                Tool::Ellipse => self.doc.add(layer, Kind::Ellipse { w: r.width(), h: r.height() }, Xf::translate(r.min.x, r.min.y)),
                                _ => self.doc.add(layer, Kind::Path(vec![Polyline::new(vec![start, end], false)]), Xf::IDENTITY),
                            };
                            self.sel = vec![id];
                        }
                    }
                    _ => {}
                }
            }
        }

        // pen preview
        if self.tool == Tool::Pen && !self.pen_pts.is_empty() {
            let st = Stroke::new(1.5, col(self.active_layer));
            let mut pts: Vec<Pos2> = self.pen_pts.iter().map(|p| self.w2s(o, *p)).collect();
            if let Some(m) = mouse {
                pts.push(m);
            }
            for p in &pts {
                painter.circle_filled(*p, 3.0, theme::accent());
            }
            painter.add(egui::Shape::line(pts, st));
        }

        self.rulers(&painter, rect);

        // Floating node-editing toolbar.
        if self.tool == Tool::Node {
            let ctx = ui.ctx().clone();
            egui::Area::new(egui::Id::new("node_bar")).order(egui::Order::Foreground).fixed_pos(rect.min + egui::vec2(28.0, 26.0)).show(&ctx, |ui| {
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
        let z = self.view.zoom as f64;
        let major = nice_step(60.0 / (z * k));
        let minor = major / 5.0;
        let decimals = if major >= 1.0 { 0 } else { (-major.log10()).ceil() as usize };
        let label = |v: f64| format!("{:.*}", decimals, v);
        let font = egui::FontId::proportional(9.5);
        let tick = Stroke::new(1.0, theme::text_dim());
        // horizontal
        let (x0, x1) = ((-self.view.pan.x) as f64 / (z * k), (rect.width() - self.view.pan.x) as f64 / (z * k));
        let mut v = (x0 / minor).floor() * minor;
        while v <= x1 {
            let sx = rect.min.x + self.view.pan.x + (v * k * z) as f32;
            let is_major = ((v / major).round() * major - v).abs() < minor * 0.01;
            if sx > rect.min.x + t {
                painter.line_segment([Pos2::new(sx, rect.min.y + if is_major { 2.0 } else { 12.0 }), Pos2::new(sx, rect.min.y + t)], tick);
                if is_major {
                    painter.text(Pos2::new(sx + 2.0, rect.min.y + 1.0), egui::Align2::LEFT_TOP, label(v), font.clone(), theme::text_dim());
                }
            }
            v += minor;
        }
        let (y0, y1) = ((-self.view.pan.y) as f64 / (z * k), (rect.height() - self.view.pan.y) as f64 / (z * k));
        let mut v = (y0 / minor).floor() * minor;
        while v <= y1 {
            let sy = rect.min.y + self.view.pan.y + (v * k * z) as f32;
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
