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
    /// Degrees of `A` for a millimetre of surface while a rotary layer is written in the fourth-axis mode.
    rot: Option<f64>,
    op: usize,
    ops: Vec<OpInfo>,
    seconds: f64,
    cut_len: f64,
}

impl<'a> Writer<'a> {
    fn new(dev: &'a Device) -> Self {
        Writer { dev, out: String::new(), moves: vec![], pos: dev.home_point(), s: None, f: None, layer: 0, head: 0, rot: None, op: 0, ops: vec![], seconds: 0.0, cut_len: 0.0 }
    }

    fn map(&self, p: Pt) -> (f64, f64) {
        self.dev.to_machine(p)
    }

    /// A move with the laser off to `p`, round the enabled no-go zones of the device (1 mm away from them) when the straight
    /// way crosses one. A move that cannot go round (it starts or ends in a zone) goes straight and the job check reports it.
    fn travel(&mut self, p: Pt) {
        if p.dist(self.pos) < 1e-6 {
            return;
        }
        if self.rot.is_none() && self.dev.zones.iter().any(|z| z.enabled) {
            let zones: Vec<ZoneShape> = self.dev.zones.iter().filter(|z| z.enabled).map(|z| z.shape).collect();
            if let Some(way) = crate::zones::route(&zones, self.pos, p, ZONE_MARGIN) {
                if !way.is_empty() {
                    let _ = writeln!(self.out, "; round a no-go zone");
                }
                for q in way {
                    self.travel_straight(q);
                }
            }
        }
        self.travel_straight(p);
    }

    fn travel_straight(&mut self, p: Pt) {
        if p.dist(self.pos) < 1e-6 {
            return;
        }
        let (x, y) = self.map(p);
        // The laser is off for a move: the dialects that set the power with a command switch it off first; old GRBL (no laser
        // mode) needs S0, as it keeps the laser on during G0.
        let d = self.dev.dialect();
        if d.by_command() && self.s.is_some_and(|s| s > 0.0) {
            if let Some((_, off)) = self.dev.power_commands() {
                let _ = writeln!(self.out, "{off}");
            }
            self.s = Some(0.0);
        }
        let off = if d == Dialect::GrblM3Old && self.s != Some(0.0) {
            self.s = Some(0.0);
            " S0"
        } else {
            ""
        };
        let _ = writeln!(self.out, "G0 X{:.3} {}{off}", x, self.y_word(y));
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
        let rot = crate::raster::scan_rotation(layer.angle);
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
            self.travel(rot.apply(Pt::new(first - dir * os, row.y)));
            if os > 0.0 {
                self.line(rot.apply(Pt::new(first, row.y)), 0.0, layer.speed);
            }
            let mut x = first;
            for run in ordered {
                let (s, e) = if rev { (run.x1, run.x0) } else { (run.x0, run.x1) };
                if (s - x).abs() > 1e-6 {
                    self.line(rot.apply(Pt::new(s, row.y)), 0.0, layer.speed);
                }
                self.line(rot.apply(Pt::new(e, row.y)), run.power, layer.speed);
                x = e;
            }
            if os > 0.0 {
                self.line(rot.apply(Pt::new(x + dir * os, row.y)), 0.0, layer.speed);
            }
        }
    }

    /// Linear move with the laser at `power` percent and `speed` mm/s.
    fn line(&mut self, p: Pt, power: f64, speed: f64) {
        // The dialects that set the power with a command: the command before the move, when the power changed.
        if let Some((on, off)) = self.dev.power_commands() {
            let s = (power / 100.0 * self.dev.head(self.head).s_max).round();
            if self.s != Some(s) {
                let cmd = if s > 0.0 { on.replace("{power}", &format!("{}", s as i64)).replace("{percent}", &format!("{}", power.round() as i64)) } else { off };
                let _ = writeln!(self.out, "{cmd}");
                self.s = Some(s);
            }
        }
        let (x, y) = self.map(p);
        let mut l = format!("G1 X{:.3} {}", x, self.y_word(y));
        self.modal(&mut l, power, speed);
        self.out.push_str(&l);
        self.out.push('\n');
        self.record(p, power, speed);
    }

    /// The word of the second axis: `Y<mm>`, or `A<degrees>` while a rotary layer is written with a fourth axis.
    fn y_word(&self, y: f64) -> String {
        match self.rot {
            Some(deg_per_mm) => format!("A{:.3}", y * deg_per_mm),
            None => format!("Y{:.3}", y),
        }
    }

    /// The `S` and `F` words that changed since the last move.
    fn modal(&mut self, l: &mut String, power: f64, speed: f64) {
        let raw = power / 100.0 * self.dev.head(self.head).s_max;
        // Smoothieware takes a fraction of the full power, so it keeps decimals; the others take whole numbers.
        let smooth = self.dev.dialect() == Dialect::Smoothieware;
        let s = if smooth { (raw * 1000.0).round() / 1000.0 } else { raw.round() };
        let f = (speed * 60.0).round();
        // A dialect that sets the power with a command has no S on its moves.
        if self.dev.dialect().by_command() {
            if self.f != Some(f) {
                let _ = write!(l, " F{}", f as i64);
                self.f = Some(f);
            }
            return;
        }
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
        // No arcs on the fourth axis: a circle on the surface is not a circle in X and A.
        if !self.dev.gcode.arcs || pts.len() < 2 || self.rot.is_some() {
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
    order_tagged(paths.into_iter().map(|p| (p, ())).collect(), start).into_iter().map(|(p, _)| p).collect()
}

/// `order_paths` for paths that carry a tag (such as the object they belong to): each keeps its tag.
pub fn order_tagged<T: Clone>(paths: Vec<(Polyline, T)>, start: Pt) -> Vec<(Polyline, T)> {
    let (paths, tags): (Vec<Polyline>, Vec<T>) = paths.into_iter().filter(|(p, _)| p.pts.len() >= 2).unzip();
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
        let mut group: Vec<(Polyline, &T)> =
            paths.iter().enumerate().filter(|(i, _)| depth[*i] == d).map(|(i, p)| (p.clone(), &tags[i])).collect();
        while !group.is_empty() {
            let mut best = (0usize, f64::INFINITY, false, 0usize);
            for (i, (p, _)) in group.iter().enumerate() {
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
            let (mut p, tag) = group.swap_remove(best.0);
            if p.closed {
                p.pts.rotate_left(best.3);
            } else if best.2 {
                p.pts.reverse();
            }
            cur = if p.closed { p.pts[0] } else { p.pts[p.pts.len() - 1] };
            result.push((p, tag.clone()));
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
/// The length of a line, in mm (closed: with the way back to the start).
fn path_length(p: &Polyline) -> f64 {
    let mut pts = p.pts.clone();
    if p.closed && !pts.is_empty() {
        pts.push(pts[0]);
    }
    pts.windows(2).map(|w| ((w[1].x - w[0].x).powi(2) + (w[1].y - w[0].y).powi(2)).sqrt()).sum()
}

/// The comment before a line that is cut or engraved in one go.
fn path_start(w: &mut Writer, i: usize, n: usize, p: &Polyline) {
    let _ = writeln!(w.out, "; path {} of {} start: {}, {:.1} mm", i + 1, n, if p.closed { "closed" } else { "open" }, path_length(p));
}

/// What an object is, in a few words for a comment of the G-code (nothing that could end or open a comment).
fn describe(sh: &Shape) -> String {
    let s = match &sh.kind {
        Kind::Rect { .. } => "rectangle".to_string(),
        Kind::Ellipse { .. } => "ellipse".to_string(),
        Kind::Path(p) => format!("lines ({})", p.len()),
        Kind::Bezier(c) => format!("curve ({} parts)", c.len()),
        Kind::Image(_) => "picture".to_string(),
        Kind::Text(t) => format!("text \"{}\"", t.text.chars().take(30).collect::<String>()),
    };
    s.chars().map(|c| if matches!(c, ';' | '(' | ')' | '\n' | '\r') { ' ' } else { c }).collect()
}

/// The object a path belongs to, for the comments: its number in the job and what it is.
type Tag = Option<(usize, String)>;

/// Comments where the paths go from one object to another (`seen`: the objects begun before, which get "again").
fn object_switch(w: &mut Writer, from: &Tag, to: &Tag, total: usize, seen: &mut Vec<usize>) {
    if from.as_ref().map(|t| t.0) == to.as_ref().map(|t| t.0) {
        return;
    }
    if let Some((no, _)) = from {
        let _ = writeln!(w.out, "; object {no} end");
    }
    if let Some((no, what)) = to {
        let again = if seen.contains(no) { " (again)" } else { "" };
        seen.push(*no);
        let _ = writeln!(w.out, "; object {no} of {total} start{again}: {what}");
    }
}

/// `tags`: for each of `polys` the object it comes from (empty when the paths are of one object that has its own comments).
#[allow(clippy::too_many_arguments)]
fn burn(w: &mut Writer, li: usize, layer: &Layer, polys: &[Polyline], tags: &[Tag], total: usize, images: &[&Shape], pn: u32, pt: u32) {
    for (i, sh) in images.iter().enumerate() {
        if let Kind::Image(im) = &sh.kind {
            w.begin_op(li, OpKind::Image, pn, pt);
            let rows = crate::raster::rows_for(sh, im, layer);
            let _ = writeln!(w.out, "; picture {} of {} start: {} rows", i + 1, images.len(), rows.len());
            w.raster(&rows, layer);
            let _ = writeln!(w.out, "; picture {} end", i + 1);
        }
    }
    if layer.mode == LayerMode::Offset {
        w.begin_op(li, OpKind::Offset, pn, pt);
        let rings = order_paths(crate::ops::inset_rings(polys, layer.interval, 10_000), w.pos);
        let _ = writeln!(w.out, "; offset fill start: {} rings, {} mm apart", rings.len(), layer.interval);
        for (i, p) in rings.iter().enumerate() {
            let _ = writeln!(w.out, "; ring {} of {} start: {:.1} mm", i + 1, rings.len(), path_length(p));
            w.travel(p.pts[0]);
            let mut pts = p.pts.clone();
            if p.closed {
                pts.push(p.pts[0]);
            }
            w.stroke(&pts, layer.power, layer.speed);
            let _ = writeln!(w.out, "; ring {} end", i + 1);
        }
        let _ = writeln!(w.out, "; offset fill end");
    }
    if matches!(layer.mode, LayerMode::Fill | LayerMode::FillAndLine) {
        w.begin_op(li, OpKind::Fill, pn, pt);
        let os = layer.overscan.max(0.0);
        let lines = fill_lines(polys, layer.interval, layer.angle, layer.bidirectional);
        let _ = writeln!(w.out, "; fill start: {} lines, {} mm apart, at {}°, overscan {} mm", lines.len(), layer.interval, layer.angle, os);
        for (a, b) in lines {
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
        let _ = writeln!(w.out, "; fill end");
    }
    if matches!(layer.mode, LayerMode::Line | LayerMode::FillAndLine) && !polys.is_empty() && layer.has_cut_options() {
        w.begin_op(li, OpKind::Line, pn, pt);
        let paths = if layer.kerf != 0.0 { crate::cutopts::kerf_paths(polys, layer.kerf) } else { polys.to_vec() };
        let all = paths.clone();
        let ordered = order_paths(paths, w.pos);
        let n = ordered.len();
        for (i, p) in ordered.into_iter().enumerate() {
            path_start(w, i, n, &p);
            let is_inner = layer.tabs.enabled && layer.tabs.skip_inner && crate::cutopts::is_inner(&p, &all);
            // Leads of a closed outline: a hole (inside an odd number of the others) has its waste inside.
            let led = (p.closed && p.pts.len() >= 3 && !layer.lead.is_off()).then(|| {
                let p = base_core::lead::good_start(&p);
                let first = p.pts[0];
                let hole = all.iter().filter(|q| q.closed && q.pts.len() >= 3 && q.pts != p.pts && q.contains(first) && q.area().abs() > p.area().abs()).count() % 2 == 1;
                let leads = base_core::lead::plan(&p, hole, &layer.lead, &all);
                (p, leads)
            });
            let p = led.as_ref().map_or(p.clone(), |(q, _)| q.clone());
            if let Some((_, leads)) = &led {
                if leads.lead_in.len() > 1 {
                    let _ = writeln!(w.out, "; lead in");
                    w.travel(leads.lead_in[0]);
                    w.stroke(&leads.lead_in, layer.power, layer.speed);
                }
            }
            for piece in crate::cutopts::pieces(&p, &layer.perforation, &layer.tabs, is_inner) {
                if piece.pts.len() < 2 {
                    continue;
                }
                w.travel(piece.pts[0]);
                if piece.factor <= 0.0 {
                    w.travel(*piece.pts.last().unwrap());
                } else {
                    w.stroke(&piece.pts, layer.power * piece.factor, layer.speed);
                }
            }
            if let Some((ring, leads)) = &led {
                if leads.overlap > 0.0 {
                    let way = base_core::lead::ring_with_overlap(ring, leads.overlap);
                    let tail: Vec<Pt> = way[ring.pts.len()..].to_vec();
                    w.stroke(&tail, layer.power, layer.speed);
                }
                if leads.lead_out.len() > 1 {
                    let _ = writeln!(w.out, "; lead out");
                    w.travel(leads.lead_out[0]);
                    w.stroke(&leads.lead_out, layer.power, layer.speed);
                }
            }
            let _ = writeln!(w.out, "; path {} end", i + 1);
        }
    } else if matches!(layer.mode, LayerMode::Line | LayerMode::FillAndLine) && !polys.is_empty() {
        w.begin_op(li, OpKind::Line, pn, pt);
        let tagged: Vec<(Polyline, Tag)> = polys.iter().enumerate().map(|(i, p)| (p.clone(), tags.get(i).cloned().flatten())).collect();
        let ordered = order_tagged(tagged, w.pos);
        let n = ordered.len();
        let (mut current, mut seen): (Tag, Vec<usize>) = (None, vec![]);
        for (i, (p, tag)) in ordered.into_iter().enumerate() {
            object_switch(w, &current, &tag, total, &mut seen);
            current = tag;
            path_start(w, i, n, &p);
            w.travel(p.pts[0]);
            let mut pts = p.pts.clone();
            if p.closed {
                pts.push(p.pts[0]);
            }
            w.stroke(&pts, layer.power, layer.speed);
            let _ = writeln!(w.out, "; path {} end", i + 1);
        }
        object_switch(w, &current, &None, total, &mut seen);
    }
}

pub fn generate(doc: &Document) -> Job {
    generate_from(doc, None)
}

/// What the program asks of a job beyond the document and the device (an option of the Control tab, never saved).
#[derive(Clone, Debug, Default)]
pub struct JobOptions {
    /// Burn the elements one after the other: all the layers and passes of an element (a group, or a single object that is in
    /// none) before the next one, the elements in the order that keeps the head's travel short. Off: a layer after a layer.
    pub one_by_one: bool,
}

/// The corner of the burned work that is put on the head when a job starts from the current position: the top-left
/// of all the laser moves (design coordinates, y down). `None` when nothing is burned.
pub fn burn_corner(doc: &Document) -> Option<Pt> {
    let job = generate(doc);
    let mut pts = job.moves.iter().filter(|m| m.laser).flat_map(|m| [m.a, m.b]);
    let first = pts.next()?;
    Some(pts.fold(first, |c, p| Pt::new(c.x.min(p.x), c.y.min(p.y))))
}

/// The document moved so that its top-left corner (see `burn_corner`) lies on `head`; unchanged when nothing is burned.
pub fn placed_at(doc: &Document, head: Pt) -> Document {
    match burn_corner(doc) {
        Some(c) => shifted(doc, head.x - c.x, head.y - c.y),
        None => doc.clone(),
    }
}

/// The job; `back` is where it ends instead of the machine's zero when the device returns to its origin (a job that
/// started from the head's position goes back there).
pub fn generate_from(doc: &Document, back: Option<Pt>) -> Job {
    generate_with(doc, back, &JobOptions::default())
}

/// The elements of the job in the order they are burnt one by one: groups (even over several layers) and single shapes, each
/// next one the nearest to where the head was left (it starts at `from`). Each element is its shapes in the order of the drawing.
fn elements_by_distance<'a>(shapes: &[&'a Shape], tol: f64, from: Pt) -> Vec<Vec<&'a Shape>> {
    let mut elements: Vec<(Option<u64>, Vec<&Shape>)> = vec![];
    for &s in shapes {
        match s.group.and_then(|g| elements.iter().position(|(k, _)| *k == Some(g))) {
            Some(i) => elements[i].1.push(s),
            None => elements.push((s.group, vec![s])),
        }
    }
    let center = |e: &[&Shape]| -> Pt {
        let pts: Vec<Pt> = e.iter().flat_map(|s| s.polys_tol(tol)).flat_map(|p| p.pts).collect();
        if pts.is_empty() {
            return Pt::new(0.0, 0.0);
        }
        let (mut lo, mut hi) = (pts[0], pts[0]);
        for p in &pts {
            lo = Pt::new(lo.x.min(p.x), lo.y.min(p.y));
            hi = Pt::new(hi.x.max(p.x), hi.y.max(p.y));
        }
        Pt::new((lo.x + hi.x) / 2.0, (lo.y + hi.y) / 2.0)
    };
    let mut left: Vec<(Pt, Vec<&Shape>)> = elements.into_iter().map(|(_, e)| (center(&e), e)).collect();
    let (mut at, mut out) = (from, vec![]);
    while !left.is_empty() {
        let next = (0..left.len()).min_by(|&a, &b| {
            let d = |i: usize| (left[i].0.x - at.x).powi(2) + (left[i].0.y - at.y).powi(2);
            d(a).total_cmp(&d(b))
        });
        let (c, e) = left.remove(next.unwrap_or(0));
        at = c;
        out.push(e);
    }
    out
}

/// The job as `generate_from` makes it, with what the program asks of it (`JobOptions`).
/// The program with the very start and the very end of the file (their variables put in) and the flavour of the device applied.
/// With the default settings it is the text as it was.
fn frame_and_flavour(text: &str, doc: &Document, dev: &Device, moves: &[Move]) -> String {
    let g = &dev.gcode;
    if g.frame == Default::default() && g.flavour.is_default() {
        return text.to_string();
    }
    let info = &doc.job_info;
    let mut common = base_core::gcode_out::Info { project_name: info.name.clone(), project_path: info.path.clone(), machine_name: dev.name.clone(), date: info.date.clone(), time: info.time.clone(), bounds: None, notes: String::new() };
    let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
    for m in moves.iter().filter(|m| m.laser) {
        for p in [m.a, m.b] {
            lo = [lo[0].min(p.x), lo[1].min(p.y)];
            hi = [hi[0].max(p.x), hi[1].max(p.y)];
        }
    }
    if lo[0].is_finite() {
        common.bounds = Some(([lo[0], lo[1], 0.0], [hi[0], hi[1], 0.0]));
    }
    let value = |v: &str| -> Option<String> {
        match v {
            "job_name" => Some(info.name.clone()),
            other => common.value(other).or_else(|| crate::hooks::device_value(dev, other, 0)),
        }
    };
    let lines = |t: &str| -> Vec<String> { t.lines().map(str::trim).filter(|l| !l.is_empty()).map(|l| base_core::gcode_out::substitute(l, &value)).collect() };
    base_core::gcode_out::finish(text, &lines(&g.frame.very_start), &lines(&g.frame.very_end), &g.flavour)
}

/// The commands that switch the air on and off: M8 / M9 for GRBL, the fan commands for Marlin and Smoothieware, or the device's own.
/// Each may be several lines.
pub fn air_commands(dev: &Device) -> (String, String) {
    let (def_on, def_off) = if dev.dialect().is_grbl() { ("M8", "M9") } else { ("M106 S255", "M107") };
    let own = |text: &str, def: &str| match GcodeCfg::custom(text) {
        Some(l) => l.iter().map(|s| s.trim()).filter(|s| !s.is_empty()).collect::<Vec<_>>().join("\n"),
        None => def.to_string(),
    };
    (own(&dev.gcode.air_on, def_on), own(&dev.gcode.air_off, def_off))
}

pub fn generate_with(doc: &Document, back: Option<Pt>, opts: &JobOptions) -> Job {
    let dev = &doc.device;
    let shapes: Vec<&Shape> = doc.scene.shapes.iter().collect();
    let mut w = Writer::new(dev);
    let _ = writeln!(w.out, "; LightCreator job — {}", dev.name);
    for l in base_core::gcode_out::note_lines(&doc.description) {
        let _ = writeln!(w.out, "; Note: {l}");
    }
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
                Dialect::Grbl | Dialect::Smoothieware | Dialect::GrblM3Old => {
                    let _ = writeln!(w.out, "M3 S0");
                }
                Dialect::Snapmaker | Dialect::LinuxCnc | Dialect::Custom => {
                    if let Some((_, off)) = dev.power_commands() {
                        let _ = writeln!(w.out, "{off}");
                    }
                }
            }
        }
    }

    // Air assist: M8 / M9 for GRBL, the fan commands for Marlin and Smoothieware, or the device's own commands.
    let (air_on, air_off) = air_commands(dev);
    let (air_on, air_off) = (air_on.as_str(), air_off.as_str());
    let mut air = false;
    // The GRBL-like dialects can switch between constant and dynamic power from layer to layer.
    let mut mode = if matches!(dialect, Dialect::Grbl | Dialect::GrblM3Old) { PowerMode::Constant } else { PowerMode::Dynamic };
    // The head that is active when the job starts is the first one.
    let mut tool = 0usize;
    // Where we are in the job, for the variables of hooks.
    let burning: Vec<usize> = doc.layer_order().into_iter().filter(|&li| doc.layers[li].output && shapes.iter().any(|s| s.layer == li)).collect();
    let tol = dev.gcode.flatten_tolerance.clamp(0.001, 5.0);
    // What is burnt in one go: the shapes of a layer, or (one by one) the shapes of an element on a layer.
    let mut items: Vec<(usize, Vec<&Shape>, Option<usize>)> = vec![];
    let mut elements_total = 0;
    if opts.one_by_one {
        let on_output: Vec<&Shape> = shapes.iter().copied().filter(|s| burning.contains(&s.layer)).collect();
        let elements = elements_by_distance(&on_output, tol, dev.home_point());
        elements_total = elements.len();
        for (ei, element) in elements.into_iter().enumerate() {
            for &li in &burning {
                let on_layer: Vec<&Shape> = element.iter().copied().filter(|s| s.layer == li).collect();
                if !on_layer.is_empty() {
                    items.push((li, on_layer, Some(ei)));
                }
            }
        }
    } else {
        for &li in &burning {
            items.push((li, shapes.iter().copied().filter(|s| s.layer == li).collect(), None));
        }
    }
    let layers_total = if opts.one_by_one { items.len() } else { burning.len() };
    let objects_total = shapes.iter().filter(|s| burning.contains(&s.layer)).count();
    // The objects in the order they are burnt, for `{workpiece_index}`.
    let object_order: Vec<u64> = items.iter().flat_map(|(_, m, _)| m.iter().map(|s| s.id)).collect();
    // (Not "; layers…": a comment that starts with "layer" marks the start of a layer for the sender.)
    let _ = writeln!(w.out, "; job: {} layer(s) to burn, {} object(s){}", burning.len(), objects_total, if opts.one_by_one { format!(", one element after the other ({elements_total})") } else { String::new() });
    let mut layer_no = 0usize;
    let mut element: Option<usize> = None;
    for (li, members, elem) in items {
        if elem != element {
            if let Some(e) = element {
                let _ = writeln!(w.out, "; element {} end", e + 1);
            }
            if let Some(e) = elem {
                let _ = writeln!(w.out, "; element {} of {} start: {} object(s)", e + 1, elements_total, members.len());
            }
            element = elem;
        }
        let layer = &doc.layers[li];
        let polys: Vec<Polyline> = members.iter().filter(|s| !s.is_image()).flat_map(|s| s.polys_tol(tol)).collect();
        let images: Vec<&Shape> = members.iter().copied().filter(|s| s.is_image()).collect();
        if polys.is_empty() && images.is_empty() {
            continue;
        }
        w.layer = li;
        w.head = layer.head.min(dev.heads.len() - 1);
        w.rot = (dev.rotary.enabled && layer.rotary && dev.rotary_mode() == RotaryMode::TrueA).then(|| dev.rotary.degrees_per_mm());
        let round = |v: f64| (v * 1000.0).round() / 1000.0;
        let _ = writeln!(w.out, "; layer {} ({}) speed {} mm/s power {}%", layer.name, layer.mode.label(), round(layer.speed), round(layer.power));
        let _ = writeln!(
            w.out,
            "; settings: number {} of {} to burn, {} object(s), {} pass(es), air assist {}{}",
            layer_no + 1,
            layers_total,
            members.len(),
            layer.passes.max(1),
            if layer.air_assist { "on" } else { "off" },
            if dev.axes.has_z && layer.passes > 1 { format!(", Z step {} mm", layer.z_step) } else { String::new() }
        );
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
        // Objects with hooks are burnt one by one; otherwise what is burnt in one go is one set of paths (nearest neighbour
        // over all).
        let per_object = dev.gcode.hooks.per_object();
        let passes = layer.passes.max(1);
        let number = |s: &Shape| object_order.iter().position(|id| *id == s.id).map_or(0, |i| i + 1);
        let tags: Vec<Tag> = if per_object { vec![] } else { members.iter().filter(|s| !s.is_image()).flat_map(|s| s.polys_tol(tol).into_iter().map(move |_| Some((number(s), describe(s))))).collect() };
        let sets: Vec<(Vec<Polyline>, Vec<&Shape>, Option<&Shape>)> = if per_object {
            members
                .iter()
                .copied()
                .map(|s| {
                    let hooked = per_object.then_some(s);
                    if s.is_image() { (vec![], vec![s], hooked) } else { (s.polys_tol(tol), vec![], hooked) }
                })
                .filter(|(p, i, _)| !p.is_empty() || !i.is_empty())
                .collect()
        } else {
            vec![(polys.clone(), images.clone(), None)]
        };
        layer_no += 1;
        let ctx = HookCtx { doc, layer_name: &layer.name, layer_no, layers_total, head: w.head, objects_total };
        w.hook(&dev.gcode.hooks.layer_start, "layer_start", &ctx, None);
        // Pass by pass over the objects.
        let runs: Vec<(u32, usize)> = (0..passes).flat_map(|p| (0..sets.len()).map(move |i| (p, i))).collect();
        let mut last_pass = None;
        for (pass, i) in runs {
            let (polys, images, shape) = &sets[i];
            let (pn, pt) = (pass + 1, passes);
            // A new pass is announced, and a machine with a Z axis goes to its height first.
            if last_pass != Some(pass) {
                if layer.passes > 1 {
                    let _ = writeln!(w.out, "; pass {}/{}", pass + 1, layer.passes);
                }
                if dev.axes.has_z {
                    if let Some(z) = layer.z_levels().get(pass as usize) {
                        let _ = writeln!(w.out, "G0 Z{z:.3} ; Z for pass {pn}");
                    }
                }
            }
            last_pass = Some(pass);
            if let Some(sh) = shape {
                let no = object_order.iter().position(|id| *id == sh.id).map_or(0, |i| i + 1);
                let _ = writeln!(w.out, "; object {} of {} start: {}", no, objects_total, describe(sh));
                w.hook(&dev.gcode.hooks.object_start, "object_start", &ctx, Some((sh, no)));
                burn(&mut w, li, layer, polys, &[], objects_total, images, pn, pt);
                w.hook(&dev.gcode.hooks.object_end, "object_end", &ctx, Some((sh, no)));
                let _ = writeln!(w.out, "; object {} end", no);
            } else {
                burn(&mut w, li, layer, polys, &tags, objects_total, images, pn, pt);
            }
        }
        w.hook(&dev.gcode.hooks.layer_end, "layer_end", &ctx, None);
        let _ = writeln!(w.out, "; end of layer {}", layer.name);
    }
    if let Some(e) = element {
        let _ = writeln!(w.out, "; element {} end", e + 1);
    }
    let _ = writeln!(w.out, "; end of the job");
    w.rot = None;
    match (dialect, dev.power_commands()) {
        (_, Some((_, off))) => {
            let _ = writeln!(w.out, "{off} ; laser off");
        }
        (Dialect::Marlin, _) => {
            let _ = writeln!(w.out, "M5 I ; laser off");
        }
        _ => {
            let _ = writeln!(w.out, "M5 ; laser off");
        }
    }
    // Off now: the move home needs no second off.
    w.s = Some(0.0);
    if dev.gcode.return_home {
        w.travel(back.unwrap_or_else(|| dev.home_point()));
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
        None if dialect.is_grbl() || matches!(dialect, Dialect::GrblM3Old | Dialect::LinuxCnc) => {
            let _ = writeln!(w.out, "M2");
        }
        None => {}
    }
    let est = timing(&mut w.moves, dev, w.seconds);
    let text = frame_and_flavour(&w.out, doc, dev, &w.moves);
    Job { ops: w.ops, gcode: text, moves: w.moves, est_seconds: est, cut_length: w.cut_len }
}

/// How far a move through the air keeps from a no-go zone (mm).
const ZONE_MARGIN: f64 = 1.0;

/// Durations of the moves with the machine's limits: cutting speed capped at the maximum, and with the acceleration the
/// planner of the controller (`base_core::motion`): the speeding up and slowing down, slower corners, a lookahead of the
/// controller's buffer. Without limits the plain distance / speed times stay. Returns the total.
fn timing(moves: &mut [Move], dev: &Device, plain_total: f64) -> f64 {
    use base_core::motion::{durations_with, Block, Motion, LOOKAHEAD};
    let (a, vmax) = (dev.speeds.acceleration, dev.speeds.max_cut_speed / 60.0);
    if (a <= 0.0 && vmax <= 0.0) || moves.is_empty() {
        return plain_total;
    }
    let travel_v = dev.speeds.travel_speed.max(1.0) / 60.0;
    let blocks: Vec<Block> = moves
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let travel = !m.laser && m.speed == travel_v;
            let v = if !travel && vmax > 0.0 { m.speed.min(vmax) } else { m.speed };
            Block { to: [m.b.x, m.b.y, 0.0], feed: v * 60.0, stop: i > 0 && moves[i - 1].b.dist(m.a) > 1e-9 }
        })
        .collect();
    // Smoothieware plans 32 moves ahead, LinuxCNC blends as far as it needs; GRBL and Marlin 15.
    let lookahead = match dev.dialect() {
        Dialect::Smoothieware => 31,
        Dialect::LinuxCnc => usize::MAX / 2,
        _ => LOOKAHEAD,
    };
    let start = [moves[0].a.x, moves[0].a.y, 0.0];
    let d = durations_with(start, &blocks, &Motion::flat(a.max(0.0)), lookahead);
    for (m, t) in moves.iter_mut().zip(d) {
        m.dur = t;
    }
    moves.iter().map(|m| m.dur).sum()
}

/// The convex hull of points: the corners of the outline that wraps them like a rubber band, counter-clockwise on the screen
/// axes, without repeats (fewer than three distinct points give those points).
pub fn convex_hull(points: &[Pt]) -> Vec<Pt> {
    let mut p: Vec<Pt> = points.iter().copied().filter(|q| q.x.is_finite() && q.y.is_finite()).collect();
    p.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
    p.dedup_by(|a, b| (a.x - b.x).abs() < 1e-9 && (a.y - b.y).abs() < 1e-9);
    if p.len() < 3 {
        return p;
    }
    let cross = |o: Pt, a: Pt, b: Pt| (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x);
    let half = |it: &mut dyn Iterator<Item = Pt>| {
        let mut h: Vec<Pt> = vec![];
        for q in it {
            while h.len() >= 2 && cross(h[h.len() - 2], h[h.len() - 1], q) <= 1e-12 {
                h.pop();
            }
            h.push(q);
        }
        h.pop();
        h
    };
    let mut hull = half(&mut p.iter().copied());
    hull.extend(half(&mut p.iter().rev().copied()));
    hull
}

/// G-code that traces the bounding box of the design. `power` is percent (0 = laser off, 1-2 gives a visible dot on diodes).
pub fn frame_gcode(bounds: Rect, dev: &Device, power: f64, speed_mm_s: f64) -> String {
    let (a, b) = (bounds.min, bounds.max);
    frame_path_gcode(&[a, Pt::new(b.x, a.y), b, Pt::new(a.x, b.y)], dev, power, speed_mm_s)
}

/// G-code that traces a closed path (the corners of `points`, and back to the first one) with the laser at `power`.
pub fn frame_path_gcode(points: &[Pt], dev: &Device, power: f64, speed_mm_s: f64) -> String {
    let mut w = Writer::new(dev);
    match dev.dialect() {
        Dialect::Marlin => {
            let _ = writeln!(w.out, "G21\nG90\nM3 I S0");
        }
        Dialect::Smoothieware | Dialect::GrblM3Old => {
            let _ = writeln!(w.out, "G21\nG90\nM3 S0");
        }
        Dialect::Grbl | Dialect::GrblDynamic => {
            let _ = writeln!(w.out, "G21\nG90\nM4 S0");
        }
        Dialect::Snapmaker | Dialect::LinuxCnc | Dialect::Custom => {
            let off = dev.power_commands().map(|c| c.1).unwrap_or_default();
            let _ = writeln!(w.out, "G21\nG90\n{off}");
        }
    }
    let Some(first) = points.first().copied() else { return w.out };
    w.travel(first);
    for p in points.iter().skip(1).copied().chain(std::iter::once(first)) {
        w.line(p, power, speed_mm_s);
    }
    match dev.power_commands() {
        Some((_, off)) => {
            let _ = writeln!(w.out, "{off}");
        }
        None => {
            let _ = writeln!(w.out, "{}", if dev.dialect() == Dialect::Marlin { "M5 I" } else { "M5" });
        }
    }
    w.out
}

/// The lines that switch the focus beam (a weak steady spot to set the height by) on or off, for the first head.
/// `None` when the head has no focus power.
pub fn focus_gcode(dev: &Device, on: bool) -> Option<Vec<String>> {
    let h = dev.head(0);
    if h.focus_power <= 0.0 {
        return None;
    }
    if let Some((on_cmd, off)) = dev.power_commands() {
        let s = (h.focus_power / 100.0 * h.s_max).round();
        return Some(vec![if on { on_cmd.replace("{power}", &format!("{}", s as i64)).replace("{percent}", &format!("{}", h.focus_power.round() as i64)) } else { off }]);
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
    for s in &mut d.scene.shapes {
        s.xf = s.xf.then(Xf::translate(dx, dy));
    }
    d
}
