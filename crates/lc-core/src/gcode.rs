//! G-code generation (GRBL flavour) and toolpath preview data.
use crate::doc::*;
use crate::geom::*;
use std::fmt::Write;

/// One straight toolpath segment, in design coordinates (mm, y-down).
#[derive(Clone, Copy, Debug)]
pub struct Move {
    pub a: Pt,
    pub b: Pt,
    pub laser: bool,
    pub layer: usize,
    /// Index into `Job::ops`: which burn operation this move belongs to.
    pub op: usize,
    /// Time the move takes, seconds.
    pub dur: f64,
    /// Nominal speed of the move, mm/s.
    pub speed: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpKind {
    Image,
    Offset,
    Fill,
    Line,
}

impl OpKind {
    pub fn label(self) -> &'static str {
        match self {
            OpKind::Image => "Image",
            OpKind::Offset => "Offset fill",
            OpKind::Fill => "Fill",
            OpKind::Line => "Line",
        }
    }
}

/// One burn operation: a layer's image, fill, offset or line pass.
#[derive(Clone, Copy, Debug)]
pub struct OpInfo {
    pub layer: usize,
    pub kind: OpKind,
    pub pass: u32,
    pub passes: u32,
}

#[derive(Clone, Debug, Default)]
pub struct Job {
    pub ops: Vec<OpInfo>,
    pub gcode: String,
    pub moves: Vec<Move>,
    pub est_seconds: f64,
    pub cut_length: f64,
}

struct Writer<'a> {
    dev: &'a Device,
    out: String,
    moves: Vec<Move>,
    pos: Pt,
    /// last emitted modal values
    s: Option<f64>,
    f: Option<f64>,
    layer: usize,
    /// The laser head of the layer being written: its S max scales the power.
    head: usize,
    op: usize,
    ops: Vec<OpInfo>,
    seconds: f64,
    cut_len: f64,
}

impl<'a> Writer<'a> {
    fn new(dev: &'a Device) -> Self {
        Writer { dev, out: String::new(), moves: vec![], pos: dev.home_point(), s: None, f: None, layer: 0, head: 0, op: 0, ops: vec![], seconds: 0.0, cut_len: 0.0 }
    }

    fn map(&self, p: Pt) -> (f64, f64) {
        self.dev.to_machine(p)
    }

    fn travel(&mut self, p: Pt) {
        if p.dist(self.pos) < 1e-6 {
            return;
        }
        let (x, y) = self.map(p);
        let _ = writeln!(self.out, "G0 X{:.3} Y{:.3}", x, y);
        self.moves.push(Move { a: self.pos, b: p, laser: false, layer: self.layer, op: self.op, dur: p.dist(self.pos) / self.dev.speeds.travel_speed.max(1.0) * 60.0, speed: self.dev.speeds.travel_speed.max(1.0) / 60.0 });
        self.seconds += p.dist(self.pos) / self.dev.speeds.travel_speed.max(1.0) * 60.0;
        self.pos = p;
    }

    /// Write the lines of a hook (if it is on) with its variables filled in.
    fn hook(&mut self, hook: &Hook, name: &str, ctx: &HookCtx, object: Option<(&Shape, usize)>) {
        let lines = hook.lines();
        if lines.is_empty() {
            return;
        }
        let info = &ctx.doc.job_info;
        let value = |v: &str| -> Option<String> {
            Some(match v {
                "job_name" => info.name.clone(),
                "date" => info.date.clone(),
                "time" => info.time.clone(),
                "layer_name" => ctx.layer_name.to_string(),
                "layer_index" => ctx.layer_no.to_string(),
                "total_layers" => ctx.layers_total.to_string(),
                "workpiece_name" => object.map(|(s, _)| shape_label(s)).unwrap_or_default(),
                "workpiece_index" => object.map(|(_, n)| n.to_string()).unwrap_or_default(),
                "total_workpieces" => ctx.objects_total.to_string(),
                other => return crate::hooks::device_value(self.dev, other, ctx.head),
            })
        };
        let _ = writeln!(self.out, "; hook {name}");
        for l in lines {
            let _ = writeln!(self.out, "{}", crate::hooks::substitute(l.trim(), &value));
        }
    }

    /// Start a new burn operation; following moves belong to it.
    fn begin_op(&mut self, layer: usize, kind: OpKind, pass: u32, passes: u32) {
        self.ops.push(OpInfo { layer, kind, pass, passes });
        self.op = self.ops.len() - 1;
    }

    /// Engrave scan rows (image raster): each run at its own power, leaving gaps with the laser off.
    fn raster(&mut self, rows: &[crate::raster::Row], layer: &Layer) {
        let os = layer.overscan.max(0.0);
        let mut emitted = 0usize;
        for row in rows {
            if row.runs.is_empty() {
                continue;
            }
            let rev = layer.bidirectional && emitted % 2 == 1;
            emitted += 1;
            let dir = if rev { -1.0 } else { 1.0 };
            let ordered: Vec<&crate::raster::Run> = if rev { row.runs.iter().rev().collect() } else { row.runs.iter().collect() };
            let first = if rev { ordered[0].x1 } else { ordered[0].x0 };
            self.travel(Pt::new(first - dir * os, row.y));
            if os > 0.0 {
                self.line(Pt::new(first, row.y), 0.0, layer.speed);
            }
            let mut x = first;
            for run in ordered {
                let (s, e) = if rev { (run.x1, run.x0) } else { (run.x0, run.x1) };
                if (s - x).abs() > 1e-6 {
                    self.line(Pt::new(s, row.y), 0.0, layer.speed);
                }
                self.line(Pt::new(e, row.y), run.power, layer.speed);
                x = e;
            }
            if os > 0.0 {
                self.line(Pt::new(x + dir * os, row.y), 0.0, layer.speed);
            }
        }
    }

    /// Linear move with the laser at `power` percent and `speed` mm/s.
    fn line(&mut self, p: Pt, power: f64, speed: f64) {
        let (x, y) = self.map(p);
        let mut l = format!("G1 X{:.3} Y{:.3}", x, y);
        self.modal(&mut l, power, speed);
        self.out.push_str(&l);
        self.out.push('\n');
        self.record(p, power, speed);
    }

    /// The `S` and `F` words that changed since the last move.
    fn modal(&mut self, l: &mut String, power: f64, speed: f64) {
        let raw = power / 100.0 * self.dev.head(self.head).s_max;
        // Smoothieware takes a fraction of the full power, so it keeps decimals; the others take whole numbers.
        let smooth = self.dev.dialect() == Dialect::Smoothieware;
        let s = if smooth { (raw * 1000.0).round() / 1000.0 } else { raw.round() };
        let f = (speed * 60.0).round();
        if self.s != Some(s) {
            if smooth {
                let _ = write!(l, " S{}", s);
            } else {
                let _ = write!(l, " S{}", s as i64);
            }
            self.s = Some(s);
        }
        if self.f != Some(f) {
            let _ = write!(l, " F{}", f as i64);
            self.f = Some(f);
        }
    }

    /// Book-keeping of a move to `p` for the preview, the time and the cut length.
    fn record(&mut self, p: Pt, power: f64, speed: f64) {
        let d = p.dist(self.pos);
        self.moves.push(Move { a: self.pos, b: p, laser: power > 0.0, layer: self.layer, op: self.op, dur: d / speed.max(0.1), speed });
        self.seconds += d / speed.max(0.1);
        if power > 0.0 {
            self.cut_len += d;
        }
        self.pos = p;
    }

    /// Trace `pts` (the first point is where the head is): as lines, or, when the device wants arcs, as arcs and
    /// lines that stay within the arc tolerance of the path.
    fn stroke(&mut self, pts: &[Pt], power: f64, speed: f64) {
        if !self.dev.gcode.arcs || pts.len() < 2 {
            for q in &pts[1..] {
                self.line(*q, power, speed);
            }
            return;
        }
        let mut from = 0;
        for piece in crate::arcs::fit(pts, self.dev.gcode.arc_tolerance.max(1e-4)) {
            match piece {
                crate::arcs::Piece::Line { end } => self.line(pts[end], power, speed),
                crate::arcs::Piece::Arc { end, center } => {
                    self.arc(&pts[from..=end], center, power, speed);
                }
            }
            from = match piece {
                crate::arcs::Piece::Line { end } | crate::arcs::Piece::Arc { end, .. } => end,
            };
        }
    }

    /// One `G2` / `G3` along `pts` (start, points on the way, end) around `center`.
    fn arc(&mut self, pts: &[Pt], center: Pt, power: f64, speed: f64) {
        let (start, mid, end) = (pts[0], pts[pts.len() / 2], pts[pts.len() - 1]);
        // The direction in the machine's own coordinates: mirrored axes turn a clockwise arc into a counter-clockwise one.
        let (s, m, e) = (self.map(start), self.map(mid), self.map(end));
        let cross = (m.0 - s.0) * (e.1 - s.1) - (m.1 - s.1) * (e.0 - s.0);
        let c = self.map(center);
        let mut l = format!("{} X{:.3} Y{:.3} I{:.3} J{:.3}", if cross < 0.0 { "G2" } else { "G3" }, e.0, e.1, c.0 - s.0, c.1 - s.1);
        self.modal(&mut l, power, speed);
        self.out.push_str(&l);
        self.out.push('\n');
        for p in &pts[1..] {
            self.record(*p, power, speed);
        }
    }
}

/// Reorder paths: inner closed shapes first, then nearest-neighbour.
pub fn order_paths(paths: Vec<Polyline>, start: Pt) -> Vec<Polyline> {
    let paths: Vec<Polyline> = paths.into_iter().filter(|p| p.pts.len() >= 2).collect();
    let n = paths.len();
    let depth: Vec<usize> = (0..n)
        .map(|i| {
            if !paths[i].closed {
                return 0;
            }
            let a = paths[i].area().abs();
            (0..n)
                .filter(|&j| j != i && paths[j].closed && paths[j].area().abs() > a && paths[j].contains(paths[i].pts[0]))
                .count()
        })
        .collect();
    let max_depth = depth.iter().copied().max().unwrap_or(0);
    let mut result = Vec::with_capacity(n);
    let mut cur = start;
    for d in (0..=max_depth).rev() {
        let mut group: Vec<Polyline> =
            paths.iter().enumerate().filter(|(i, _)| depth[*i] == d).map(|(_, p)| p.clone()).collect();
        while !group.is_empty() {
            let mut best = (0usize, f64::INFINITY, false, 0usize);
            for (i, p) in group.iter().enumerate() {
                if p.closed {
                    for (k, q) in p.pts.iter().enumerate() {
                        let dd = q.dist(cur);
                        if dd < best.1 {
                            best = (i, dd, false, k);
                        }
                    }
                } else {
                    let (da, db) = (p.pts[0].dist(cur), p.pts[p.pts.len() - 1].dist(cur));
                    if da < best.1 {
                        best = (i, da, false, 0);
                    }
                    if db < best.1 {
                        best = (i, db, true, 0);
                    }
                }
            }
            let mut p = group.swap_remove(best.0);
            if p.closed {
                p.pts.rotate_left(best.3);
            } else if best.2 {
                p.pts.reverse();
            }
            cur = if p.closed { p.pts[0] } else { p.pts[p.pts.len() - 1] };
            result.push(p);
        }
    }
    result
}

/// Scan-line segments `(start, end)` filling the closed `polys` (even-odd rule).
pub fn fill_lines(polys: &[Polyline], interval: f64, angle_deg: f64, bidirectional: bool) -> Vec<(Pt, Pt)> {
    let interval = interval.max(0.005);
    let fwd = Xf::rotate(-angle_deg.to_radians());
    let back = Xf::rotate(angle_deg.to_radians());
    let rot: Vec<Polyline> = polys.iter().filter(|p| p.closed && p.pts.len() >= 3).map(|p| p.transformed(&fwd)).collect();
    let Some(b) = rot.iter().filter_map(|p| p.bounds()).reduce(Rect::union) else { return vec![] };
    let mut out = vec![];
    let mut k = 0usize;
    let mut y = b.min.y + interval / 2.0;
    while y < b.max.y {
        let mut xs: Vec<f64> = vec![];
        for p in &rot {
            let n = p.pts.len();
            for i in 0..n {
                let (a, c) = (p.pts[i], p.pts[(i + 1) % n]);
                if (a.y <= y && c.y > y) || (c.y <= y && a.y > y) {
                    xs.push(a.x + (y - a.y) / (c.y - a.y) * (c.x - a.x));
                }
            }
        }
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mut spans: Vec<(f64, f64)> = xs.chunks_exact(2).map(|c| (c[0], c[1])).filter(|(a, b)| b - a > 1e-6).collect();
        let reverse = bidirectional && k % 2 == 1;
        if reverse {
            spans.reverse();
        }
        for (a, c) in spans {
            let (s, e) = if reverse { (c, a) } else { (a, c) };
            out.push((back.apply(Pt::new(s, y)), back.apply(Pt::new(e, y))));
        }
        y += interval;
        k += 1;
    }
    out
}

/// What a hook may say about the job: where in it we are.
struct HookCtx<'a> {
    doc: &'a Document,
    layer_name: &'a str,
    layer_no: usize,
    layers_total: usize,
    head: usize,
    objects_total: usize,
}

/// A short name for an object, for `{workpiece_name}`.
fn shape_label(sh: &Shape) -> String {
    match &sh.kind {
        Kind::Rect { .. } => "Rectangle".into(),
        Kind::Ellipse { .. } => "Ellipse".into(),
        Kind::Path(_) => "Path".into(),
        Kind::Bezier(_) => "Curve".into(),
        Kind::Image(im) => if im.name.is_empty() { "Image".into() } else { im.name.clone() },
        Kind::Text(t) => t.text.lines().next().unwrap_or("Text").chars().take(40).collect(),
    }
}

/// Burn one set of paths and images with the settings of `layer`: images, offset rings, fill, outline.
fn burn(w: &mut Writer, li: usize, layer: &Layer, polys: &[Polyline], images: &[&Shape], pn: u32, pt: u32) {
    for sh in images {
        if let Kind::Image(im) = &sh.kind {
            w.begin_op(li, OpKind::Image, pn, pt);
            let rows = crate::raster::rows_for(sh, im, layer);
            w.raster(&rows, layer);
        }
    }
    if layer.mode == LayerMode::Offset {
        w.begin_op(li, OpKind::Offset, pn, pt);
        let rings = crate::ops::inset_rings(polys, layer.interval, 10_000);
        for p in order_paths(rings, w.pos) {
            w.travel(p.pts[0]);
            let mut pts = p.pts.clone();
            if p.closed {
                pts.push(p.pts[0]);
            }
            w.stroke(&pts, layer.power, layer.speed);
        }
    }
    if matches!(layer.mode, LayerMode::Fill | LayerMode::FillAndLine) {
        w.begin_op(li, OpKind::Fill, pn, pt);
        let os = layer.overscan.max(0.0);
        for (a, b) in fill_lines(polys, layer.interval, layer.angle, layer.bidirectional) {
            let d = Pt::new(b.x - a.x, b.y - a.y);
            let len = (d.x * d.x + d.y * d.y).sqrt();
            let u = Pt::new(d.x / len, d.y / len);
            let (pre, post) = (Pt::new(a.x - u.x * os, a.y - u.y * os), Pt::new(b.x + u.x * os, b.y + u.y * os));
            w.travel(pre);
            if os > 0.0 {
                w.line(a, 0.0, layer.speed);
            }
            w.line(b, layer.power, layer.speed);
            if os > 0.0 {
                w.line(post, 0.0, layer.speed);
            }
        }
    }
    if matches!(layer.mode, LayerMode::Line | LayerMode::FillAndLine) && !polys.is_empty() {
        w.begin_op(li, OpKind::Line, pn, pt);
        for p in order_paths(polys.to_vec(), w.pos) {
            w.travel(p.pts[0]);
            let mut pts = p.pts.clone();
            if p.closed {
                pts.push(p.pts[0]);
            }
            w.stroke(&pts, layer.power, layer.speed);
        }
    }
}

pub fn generate(doc: &Document) -> Job {
    let dev = &doc.device;
    let mut w = Writer::new(dev);
    let _ = writeln!(w.out, "; LightCreator job — {}", dev.name);
    let dialect = dev.dialect();
    match GcodeCfg::custom(&dev.gcode.preamble) {
        Some(lines) => {
            let _ = writeln!(w.out, "; own start of the job");
            for l in lines {
                let _ = writeln!(w.out, "{l}");
            }
        }
        None => {
            let _ = writeln!(w.out, "G21 ; mm\nG90 ; absolute\nG17");
            match dialect {
                // Marlin laser feature: inline power, S on each G1 sets the power.
                Dialect::Marlin => {
                    let _ = writeln!(w.out, "M3 I S0");
                }
                Dialect::GrblDynamic => {
                    let _ = writeln!(w.out, "M4 S0");
                }
                Dialect::Grbl | Dialect::Smoothieware => {
                    let _ = writeln!(w.out, "M3 S0");
                }
            }
        }
    }

    // Air assist: M8 / M9 for GRBL, the fan commands for Marlin and Smoothieware, or the device's own commands.
    let (def_on, def_off) = if dialect.is_grbl() { ("M8", "M9") } else { ("M106 S255", "M107") };
    let own = |text: &str, def: &str| match GcodeCfg::custom(text) {
        Some(l) => l.iter().map(|s| s.trim()).filter(|s| !s.is_empty()).collect::<Vec<_>>().join("\n"),
        None => def.to_string(),
    };
    let (air_on, air_off) = (own(&dev.gcode.air_on, def_on), own(&dev.gcode.air_off, def_off));
    let (air_on, air_off) = (air_on.as_str(), air_off.as_str());
    let mut air = false;
    // The GRBL-like dialects can switch between constant and dynamic power from layer to layer.
    let mut mode = if dialect == Dialect::Grbl { PowerMode::Constant } else { PowerMode::Dynamic };
    // The head that is active when the job starts is the first one.
    let mut tool = 0usize;
    // Where we are in the job, for the variables of hooks.
    let burning: Vec<usize> = doc.layer_order().into_iter().filter(|&li| doc.layers[li].output && doc.shapes.iter().any(|s| s.layer == li)).collect();
    let layers_total = burning.len();
    let objects_total = doc.shapes.iter().filter(|s| burning.contains(&s.layer)).count();
    // The objects in the order they are burnt, for `{workpiece_index}`.
    let object_order: Vec<u64> = burning.iter().flat_map(|&li| doc.shapes.iter().filter(move |s| s.layer == li).map(|s| s.id)).collect();
    let mut layer_no = 0usize;
    for li in doc.layer_order() {
        let layer = &doc.layers[li];
        if !layer.output {
            continue;
        }
        let tol = dev.gcode.flatten_tolerance.clamp(0.001, 5.0);
        let polys: Vec<Polyline> = doc.shapes.iter().filter(|s| s.layer == li && !s.is_image()).flat_map(|s| s.polys_tol(tol)).collect();
        let images: Vec<&Shape> = doc.shapes.iter().filter(|s| s.layer == li && s.is_image()).collect();
        if polys.is_empty() && images.is_empty() {
            continue;
        }
        w.layer = li;
        w.head = layer.head.min(dev.heads.len() - 1);
        let _ = writeln!(w.out, "; layer {} ({}) speed {} mm/s power {}%", layer.name, layer.mode.label(), layer.speed, layer.power);
        // With several heads the controller is told which one burns this layer.
        if dev.heads.len() > 1 && w.head != tool {
            tool = w.head;
            let h = &dev.heads[tool];
            let _ = writeln!(w.out, "T{} ; laser head {}", h.tool, if h.name.trim().is_empty() { (tool + 1).to_string() } else { h.name.clone() });
            // The new head has its own S scale: the modal S is no longer what the controller holds.
            w.s = None;
        }
        if dialect.is_grbl() {
            let want = match layer.power_mode {
                PowerMode::Inherit if dialect == Dialect::Grbl => PowerMode::Constant,
                PowerMode::Inherit => PowerMode::Dynamic,
                m => m,
            };
            if want != mode {
                mode = want;
                let _ = writeln!(w.out, "{} ; {} power", if mode == PowerMode::Constant { "M3" } else { "M4" }, if mode == PowerMode::Constant { "constant" } else { "dynamic" });
            }
        }
        if layer.air_assist != air {
            air = layer.air_assist;
            let _ = writeln!(w.out, "{} ; air assist {}", if air { air_on } else { air_off }, if air { "on" } else { "off" });
        }
        // Objects with hooks are burnt one by one; otherwise a layer is one set of paths (nearest neighbour over all).
        let per_object = dev.gcode.hooks.per_object();
        let sets: Vec<(Vec<Polyline>, Vec<&Shape>, Option<&Shape>)> = if per_object {
            doc.shapes
                .iter()
                .filter(|s| s.layer == li)
                .map(|s| if s.is_image() { (vec![], vec![s], Some(s)) } else { (s.polys_tol(tol), vec![], Some(s)) })
                .filter(|(p, i, _)| !p.is_empty() || !i.is_empty())
                .collect()
        } else {
            vec![(polys.clone(), images.clone(), None)]
        };
        layer_no += 1;
        let ctx = HookCtx { doc, layer_name: &layer.name, layer_no, layers_total, head: w.head, objects_total };
        w.hook(&dev.gcode.hooks.layer_start, "layer_start", &ctx, None);
        for pass in 0..layer.passes.max(1) {
            if layer.passes > 1 {
                let _ = writeln!(w.out, "; pass {}/{}", pass + 1, layer.passes);
            }
            let (pn, pt) = (pass + 1, layer.passes.max(1));
            for (polys, images, shape) in &sets {
                if let Some(sh) = shape {
                    let no = object_order.iter().position(|id| *id == sh.id).map_or(0, |i| i + 1);
                    w.hook(&dev.gcode.hooks.object_start, "object_start", &ctx, Some((sh, no)));
                    burn(&mut w, li, layer, polys, images, pn, pt);
                    w.hook(&dev.gcode.hooks.object_end, "object_end", &ctx, Some((sh, no)));
                } else {
                    burn(&mut w, li, layer, polys, images, pn, pt);
                }
            }
        }
        w.hook(&dev.gcode.hooks.layer_end, "layer_end", &ctx, None);
    }
    let _ = writeln!(w.out, "{}", if dialect == Dialect::Marlin { "M5 I ; laser off" } else { "M5 ; laser off" });
    if dev.gcode.return_home {
        w.travel(dev.home_point());
    }
    if air {
        let _ = writeln!(w.out, "{air_off} ; air assist off");
    }
    match GcodeCfg::custom(&dev.gcode.postscript) {
        Some(lines) => {
            let _ = writeln!(w.out, "; own end of the job");
            for l in lines {
                let _ = writeln!(w.out, "{l}");
            }
        }
        None if dialect.is_grbl() => {
            let _ = writeln!(w.out, "M2");
        }
        None => {}
    }
    let est = timing(&mut w.moves, dev, w.seconds);
    Job { ops: w.ops, gcode: w.out, moves: w.moves, est_seconds: est, cut_length: w.cut_len }
}

/// Time of `len` mm at `v` mm/s from and to rest with acceleration `a` mm/s² (trapezoid, or triangle when short).
fn ramp_time(len: f64, v: f64, a: f64) -> f64 {
    if len <= 0.0 || v <= 0.0 {
        return 0.0;
    }
    if a <= 0.0 {
        return len / v;
    }
    let ramp = v * v / a;
    if len >= ramp { len / v + v / a } else { 2.0 * (len / a).sqrt() }
}

/// Durations of the moves with the machine's limits: cutting speed capped at the maximum, and a start and a stop
/// ramp for every run of moves at one speed (travel moves each start and stop). Without limits the plain
/// distance / speed times stay. Returns the total.
fn timing(moves: &mut [Move], dev: &Device, plain_total: f64) -> f64 {
    let (a, vmax) = (dev.speeds.acceleration, dev.speeds.max_cut_speed / 60.0);
    if a <= 0.0 && vmax <= 0.0 {
        return plain_total;
    }
    let mut i = 0;
    while i < moves.len() {
        let travel = !moves[i].laser && moves[i].speed == dev.speeds.travel_speed.max(1.0) / 60.0;
        let mut j = i + 1;
        // Travel moves stand alone; cutting moves at the same speed form one run.
        while !travel && j < moves.len() && moves[j].laser == moves[i].laser && (moves[j].speed - moves[i].speed).abs() < 1e-9 && moves[j].a.dist(moves[j - 1].b) < 1e-9 {
            j += 1;
        }
        let v = if !travel && vmax > 0.0 { moves[i].speed.min(vmax) } else { moves[i].speed };
        let len: f64 = moves[i..j].iter().map(|m| m.a.dist(m.b)).sum();
        let t = ramp_time(len, v, a);
        for m in &mut moves[i..j] {
            m.dur = if len > 0.0 { t * m.a.dist(m.b) / len } else { 0.0 };
        }
        i = j;
    }
    moves.iter().map(|m| m.dur).sum()
}

/// G-code that traces the bounding box of the design. `power` is percent (0 = laser off, 1-2 gives a visible dot on diodes).
pub fn frame_gcode(bounds: Rect, dev: &Device, power: f64, speed_mm_s: f64) -> String {
    let mut w = Writer::new(dev);
    match dev.dialect() {
        Dialect::Marlin => {
            let _ = writeln!(w.out, "G21\nG90\nM3 I S0");
        }
        Dialect::Smoothieware => {
            let _ = writeln!(w.out, "G21\nG90\nM3 S0");
        }
        Dialect::Grbl | Dialect::GrblDynamic => {
            let _ = writeln!(w.out, "G21\nG90\nM4 S0");
        }
    }
    let (a, b) = (bounds.min, bounds.max);
    w.travel(a);
    for p in [Pt::new(b.x, a.y), b, Pt::new(a.x, b.y), a] {
        w.line(p, power, speed_mm_s);
    }
    let _ = writeln!(w.out, "{}", if dev.dialect() == Dialect::Marlin { "M5 I" } else { "M5" });
    w.out
}

/// The lines that switch the focus beam (a weak steady spot to set the height by) on or off, for the first head.
/// `None` when the head has no focus power.
pub fn focus_gcode(dev: &Device, on: bool) -> Option<Vec<String>> {
    let h = dev.head(0);
    if h.focus_power <= 0.0 {
        return None;
    }
    if !on {
        return Some(vec!["M5".to_string()]);
    }
    let raw = h.focus_power / 100.0 * h.s_max;
    // Smoothieware takes a fraction, the others whole numbers.
    let s = if dev.dialect() == Dialect::Smoothieware { format!("{}", (raw * 1000.0).round() / 1000.0) } else { format!("{}", raw.round() as i64) };
    // Constant power (M3), not the dynamic mode: in that one the laser is dark while the head stands still.
    Some(vec![format!("M3 S{s}")])
}

/// The G-code of a job with the laser kept off: every `S` word of a move is written as `S0`. The path is the same,
/// for a dry run.
pub fn without_power(gcode: &str) -> String {
    let mut out = String::with_capacity(gcode.len());
    for line in gcode.lines() {
        let (code, comment) = match line.split_once(';') {
            Some((c, r)) => (c, Some(r)),
            None => (line, None),
        };
        let is_move = code.starts_with("G1 ") || code.starts_with("G2 ") || code.starts_with("G3 ");
        if is_move {
            let words: Vec<String> = code.split_whitespace().map(|w| if w.starts_with('S') && w[1..].parse::<f64>().is_ok() { "S0".to_string() } else { w.to_string() }).collect();
            out.push_str(&words.join(" "));
            if let Some(c) = comment {
                out.push_str(" ;");
                out.push_str(c);
            }
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

/// A copy of the document with everything moved by `(dx, dy)` design millimetres.
pub fn shifted(doc: &Document, dx: f64, dy: f64) -> Document {
    let mut d = doc.clone();
    for s in &mut d.shapes {
        s.xf = s.xf.then(Xf::translate(dx, dy));
    }
    d
}
