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
}

#[derive(Clone, Debug, Default)]
pub struct Job {
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
    seconds: f64,
    cut_len: f64,
}

impl<'a> Writer<'a> {
    fn new(dev: &'a Device) -> Self {
        Writer { dev, out: String::new(), moves: vec![], pos: Pt::new(0.0, 0.0), s: None, f: None, layer: 0, seconds: 0.0, cut_len: 0.0 }
    }

    fn map(&self, p: Pt) -> (f64, f64) {
        let x = p.x.clamp(0.0, self.dev.bed_w);
        let y = p.y.clamp(0.0, self.dev.bed_h);
        match self.dev.origin {
            Origin::FrontLeft => (x, self.dev.bed_h - y),
            Origin::BackLeft => (x, y),
        }
    }

    fn clamp(&self, p: Pt) -> Pt {
        Pt::new(p.x.clamp(0.0, self.dev.bed_w), p.y.clamp(0.0, self.dev.bed_h))
    }

    fn travel(&mut self, p: Pt) {
        let p = self.clamp(p);
        if p.dist(self.pos) < 1e-6 {
            return;
        }
        let (x, y) = self.map(p);
        let _ = writeln!(self.out, "G0 X{:.3} Y{:.3}", x, y);
        self.moves.push(Move { a: self.pos, b: p, laser: false, layer: self.layer });
        self.seconds += p.dist(self.pos) / self.dev.travel_speed.max(1.0) * 60.0;
        self.pos = p;
    }

    /// Linear move with the laser at `power` percent and `speed` mm/s.
    fn line(&mut self, p: Pt, power: f64, speed: f64) {
        let p = self.clamp(p);
        let s = (power / 100.0 * self.dev.s_max).round();
        let f = (speed * 60.0).round();
        let (x, y) = self.map(p);
        let mut l = format!("G1 X{:.3} Y{:.3}", x, y);
        if self.s != Some(s) {
            let _ = write!(l, " S{}", s as i64);
            self.s = Some(s);
        }
        if self.f != Some(f) {
            let _ = write!(l, " F{}", f as i64);
            self.f = Some(f);
        }
        self.out.push_str(&l);
        self.out.push('\n');
        let d = p.dist(self.pos);
        self.moves.push(Move { a: self.pos, b: p, laser: power > 0.0, layer: self.layer });
        self.seconds += d / speed.max(0.1);
        if power > 0.0 {
            self.cut_len += d;
        }
        self.pos = p;
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

pub fn generate(doc: &Document) -> Job {
    let dev = &doc.device;
    let mut w = Writer::new(dev);
    let _ = writeln!(w.out, "; LightCreator job — {}", dev.name);
    let _ = writeln!(w.out, "G21 ; mm\nG90 ; absolute\nG17");
    if dev.controller == Controller::Marlin {
        // Marlin laser feature: inline power, S on each G1 sets the power.
        let _ = writeln!(w.out, "M3 I S0");
    } else {
        let _ = writeln!(w.out, "{} S0", if dev.dynamic_power { "M4" } else { "M3" });
    }

    for (li, layer) in doc.layers.iter().enumerate() {
        if !layer.output {
            continue;
        }
        let polys: Vec<Polyline> = doc.shapes.iter().filter(|s| s.layer == li).flat_map(|s| s.polys()).collect();
        if polys.is_empty() {
            continue;
        }
        w.layer = li;
        let _ = writeln!(w.out, "; layer {} ({}) speed {} mm/s power {}%", layer.name, layer.mode.label(), layer.speed, layer.power);
        for pass in 0..layer.passes.max(1) {
            if layer.passes > 1 {
                let _ = writeln!(w.out, "; pass {}/{}", pass + 1, layer.passes);
            }
            if matches!(layer.mode, LayerMode::Fill | LayerMode::FillAndLine) {
                let os = layer.overscan.max(0.0);
                for (a, b) in fill_lines(&polys, layer.interval, layer.angle, layer.bidirectional) {
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
            if matches!(layer.mode, LayerMode::Line | LayerMode::FillAndLine) {
                for p in order_paths(polys.clone(), w.pos) {
                    w.travel(p.pts[0]);
                    for q in &p.pts[1..] {
                        w.line(*q, layer.power, layer.speed);
                    }
                    if p.closed {
                        w.line(p.pts[0], layer.power, layer.speed);
                    }
                }
            }
        }
    }
    let _ = writeln!(w.out, "{}", if dev.controller == Controller::Marlin { "M5 I ; laser off" } else { "M5 ; laser off" });
    if dev.return_home {
        w.travel(Pt::new(0.0, 0.0));
    }
    if dev.controller != Controller::Marlin {
        let _ = writeln!(w.out, "M2");
    }
    Job { gcode: w.out, moves: w.moves, est_seconds: w.seconds, cut_length: w.cut_len }
}

/// G-code that traces the bounding box of the design. `power` is percent (0 = laser off, 1-2 gives a visible dot on diodes).
pub fn frame_gcode(bounds: Rect, dev: &Device, power: f64, speed_mm_s: f64) -> String {
    let mut w = Writer::new(dev);
    if dev.controller == Controller::Marlin {
        let _ = writeln!(w.out, "G21\nG90\nM3 I S0");
    } else {
        let _ = writeln!(w.out, "G21\nG90\nM4 S0");
    }
    let (a, b) = (bounds.min, bounds.max);
    w.travel(a);
    for p in [Pt::new(b.x, a.y), b, Pt::new(a.x, b.y), a] {
        w.line(p, power, speed_mm_s);
    }
    let _ = writeln!(w.out, "{}", if dev.controller == Controller::Marlin { "M5 I" } else { "M5" });
    w.out
}
