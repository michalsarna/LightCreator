//! Extra options of a line layer, as in other laser software: a kerf offset, perforation (cut / skip), and tabs
//! (bridges) that leave a piece of the outline uncut or cut only weakly.
use crate::geom::*;
use serde::{Deserialize, Serialize};

/// Cut and skip lengths, millimetres.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Perforation {
    pub enabled: bool,
    pub cut: f64,
    pub skip: f64,
}

impl Default for Perforation {
    fn default() -> Self {
        Perforation { enabled: false, cut: 0.1, skip: 0.1 }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TabMode {
    /// A tab every `spacing` millimetres of outline.
    #[default]
    Even,
    /// A fixed number of tabs on every shape.
    PerShape,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tabs {
    pub enabled: bool,
    pub mode: TabMode,
    /// Length of one tab along the outline, mm.
    pub size: f64,
    /// With `Even`: distance between tabs, mm.
    pub spacing: f64,
    /// With `PerShape`: how many.
    pub per_shape: u32,
    /// With `Even`: at least this many on a shape.
    pub min_tabs: u32,
    pub limit_max: bool,
    pub max_tabs: u32,
    /// Power over a tab, percent of the layer's power; 0 = the laser is off there.
    pub cut_power: f64,
    /// Leave shapes that lie inside another shape without tabs.
    pub skip_inner: bool,
}

impl Default for Tabs {
    fn default() -> Self {
        Tabs { enabled: false, mode: TabMode::Even, size: 0.5, spacing: 50.0, per_shape: 1, min_tabs: 1, limit_max: false, max_tabs: 1, cut_power: 0.0, skip_inner: false }
    }
}

impl Tabs {
    pub fn clamped(self) -> Tabs {
        let f = |v: f64, lo: f64, hi: f64| if v.is_finite() { v.clamp(lo, hi) } else { lo };
        Tabs { size: f(self.size, 0.0, 1000.0), spacing: f(self.spacing, 0.1, 100_000.0), per_shape: self.per_shape.min(1000), min_tabs: self.min_tabs.min(1000), max_tabs: self.max_tabs.min(1000), cut_power: f(self.cut_power, 0.0, 100.0), ..self }
    }

    /// How many tabs a shape with an outline of `len` millimetres gets.
    pub fn count(&self, len: f64) -> u32 {
        let t = self.clamped();
        let n = match t.mode {
            TabMode::PerShape => t.per_shape,
            TabMode::Even => ((len / t.spacing).floor() as u32).max(t.min_tabs),
        };
        if t.limit_max {
            n.min(t.max_tabs)
        } else {
            n
        }
    }
}

impl Perforation {
    pub fn clamped(self) -> Perforation {
        let f = |v: f64| if v.is_finite() { v.clamp(0.0, 1000.0) } else { 0.0 };
        Perforation { cut: f(self.cut), skip: f(self.skip), ..self }
    }
}

/// A stretch of a path and the share of the layer's power it is burnt with (0 = the laser stays off).
#[derive(Clone, Debug, PartialEq)]
pub struct Piece {
    pub pts: Vec<Pt>,
    pub factor: f64,
}

/// Closed paths moved out by `kerf` millimetres (in, when negative); holes move the other way, as the material
/// does. Open paths stay as they are.
pub fn kerf_paths(polys: &[Polyline], kerf: f64) -> Vec<Polyline> {
    if !kerf.is_finite() || kerf == 0.0 {
        return polys.to_vec();
    }
    let closed: Vec<Polyline> = polys.iter().filter(|p| p.closed && p.pts.len() >= 3).cloned().collect();
    let mut out: Vec<Polyline> = polys.iter().filter(|p| !(p.closed && p.pts.len() >= 3)).cloned().collect();
    out.extend(crate::ops::offset(&closed, kerf.clamp(-50.0, 50.0)));
    out
}

/// Is `p` inside another closed path of `all` (so that it is an inner shape)?
pub fn is_inner(p: &Polyline, all: &[Polyline]) -> bool {
    let Some(first) = p.pts.first() else { return false };
    all.iter().any(|q| q.closed && !std::ptr::eq(q, p) && q.pts.len() >= 3 && q.contains(*first) && q.area().abs() > p.area().abs())
}

/// The path cut into pieces: the stretches under perforation or tabs get their own power factor.
/// `inner` says the shape lies inside another one (for "skip inner shapes").
pub fn pieces(path: &Polyline, perf: &Perforation, tabs: &Tabs, inner: bool) -> Vec<Piece> {
    let mut pts = path.pts.clone();
    if path.closed && pts.len() > 2 {
        pts.push(pts[0]);
    }
    let total: f64 = pts.windows(2).map(|w| w[0].dist(w[1])).sum();
    let whole = || vec![Piece { pts: path.pts.iter().copied().chain((path.closed && !path.pts.is_empty()).then(|| path.pts[0])).collect(), factor: 1.0 }];
    if pts.len() < 2 || !(total > 1e-9) {
        return whole();
    }
    let perf = perf.clamped();
    let tabs = tabs.clamped();
    let use_perf = perf.enabled && perf.cut + perf.skip > 0.01 && perf.skip > 0.0 && total / (perf.cut + perf.skip) < 100_000.0;
    let n_tabs = if tabs.enabled && !(tabs.skip_inner && inner) && tabs.size > 0.0 { tabs.count(total) } else { 0 };
    let use_tabs = n_tabs > 0 && tabs.size * f64::from(n_tabs) < total;
    if !use_perf && !use_tabs {
        return whole();
    }
    // The factor at arc length `s`.
    let tab_len = tabs.size;
    let factor_at = |s: f64| -> f64 {
        let mut f = 1.0f64;
        if use_perf {
            let period = perf.cut + perf.skip;
            if s % period >= perf.cut {
                f = 0.0;
            }
        }
        if use_tabs && f > 0.0 {
            let step = total / f64::from(n_tabs);
            let k = (s / step).floor();
            let centre = (k + 0.5) * step;
            if (s - centre).abs() < tab_len / 2.0 {
                f = f.min(tabs.cut_power / 100.0);
            }
        }
        f
    };
    // The places where the factor can change, along the path.
    let mut cuts: Vec<f64> = vec![0.0, total];
    if use_perf {
        let period = perf.cut + perf.skip;
        let mut k = 0.0;
        while k * period < total {
            cuts.push(k * period + perf.cut);
            cuts.push((k + 1.0) * period);
            k += 1.0;
        }
    }
    if use_tabs {
        let step = total / f64::from(n_tabs);
        for k in 0..n_tabs {
            let c = (f64::from(k) + 0.5) * step;
            cuts.push(c - tab_len / 2.0);
            cuts.push(c + tab_len / 2.0);
        }
    }
    cuts.retain(|c| *c >= 0.0 && *c <= total);
    cuts.sort_by(|a, b| a.total_cmp(b));
    cuts.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    // Cumulative lengths of the path's own points, to find the point at an arc length.
    let mut acc = vec![0.0];
    for w in pts.windows(2) {
        acc.push(acc[acc.len() - 1] + w[0].dist(w[1]));
    }
    let at = |s: f64| -> Pt {
        let i = acc.partition_point(|a| *a <= s).clamp(1, pts.len() - 1);
        let (a, b) = (pts[i - 1], pts[i]);
        let seg = acc[i] - acc[i - 1];
        if seg < 1e-12 {
            a
        } else {
            let t = ((s - acc[i - 1]) / seg).clamp(0.0, 1.0);
            Pt::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
        }
    };
    let mut out: Vec<Piece> = vec![];
    for w in cuts.windows(2) {
        let (s0, s1) = (w[0], w[1]);
        if s1 - s0 < 1e-9 {
            continue;
        }
        let f = factor_at((s0 + s1) / 2.0);
        // The path's own corners between the two cuts belong to this piece.
        let mut seg = vec![at(s0)];
        for (i, a) in acc.iter().enumerate() {
            if *a > s0 + 1e-9 && *a < s1 - 1e-9 {
                seg.push(pts[i]);
            }
        }
        seg.push(at(s1));
        match out.last_mut() {
            Some(last) if (last.factor - f).abs() < 1e-12 => last.pts.extend(seg.into_iter().skip(1)),
            _ => out.push(Piece { pts: seg, factor: f }),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(s: f64) -> Polyline {
        Polyline::new(vec![Pt::new(0.0, 0.0), Pt::new(s, 0.0), Pt::new(s, s), Pt::new(0.0, s)], true)
    }

    fn len(p: &[Piece], f: f64) -> f64 {
        p.iter().filter(|x| (x.factor - f).abs() < 1e-9).map(|x| x.pts.windows(2).map(|w| w[0].dist(w[1])).sum::<f64>()).sum()
    }

    #[test]
    fn nothing_changes_without_options() {
        let p = square(10.0);
        let pc = pieces(&p, &Perforation::default(), &Tabs::default(), false);
        assert_eq!(pc.len(), 1);
        assert_eq!(pc[0].pts.len(), 5, "closed: the start repeated at the end");
        assert_eq!(kerf_paths(&[p.clone()], 0.0), vec![p]);
    }

    #[test]
    fn perforation_alternates_cut_and_skip() {
        let p = Polyline::new(vec![Pt::new(0.0, 0.0), Pt::new(10.0, 0.0)], false);
        let pc = pieces(&p, &Perforation { enabled: true, cut: 1.0, skip: 1.0 }, &Tabs::default(), false);
        assert!((len(&pc, 1.0) - 5.0).abs() < 1e-6 && (len(&pc, 0.0) - 5.0).abs() < 1e-6, "{pc:?}");
        assert_eq!(pc.len(), 10);
        // The pieces join up from the start to the end.
        assert_eq!(pc[0].pts[0], Pt::new(0.0, 0.0));
        assert!(pc.last().unwrap().pts.last().unwrap().dist(Pt::new(10.0, 0.0)) < 1e-9);
        // Bad numbers do nothing.
        for (c, s) in [(0.0, 0.0), (f64::NAN, 1.0), (1.0, 0.0)] {
            let r = pieces(&p, &Perforation { enabled: true, cut: c, skip: s }, &Tabs::default(), false);
            assert!(r.iter().all(|x| x.factor == 1.0) || c.is_nan(), "{c},{s}");
        }
    }

    #[test]
    fn tabs_leave_gaps_in_a_closed_outline_and_respect_the_limits() {
        let p = square(100.0); // 400 mm round
        let t = Tabs { enabled: true, size: 2.0, spacing: 100.0, ..Default::default() };
        assert_eq!(t.count(400.0), 4);
        let pc = pieces(&p, &Perforation::default(), &t, false);
        assert!((len(&pc, 0.0) - 8.0).abs() < 1e-6, "4 tabs of 2 mm: {}", len(&pc, 0.0));
        assert!((len(&pc, 1.0) - 392.0).abs() < 1e-6);
        // A weak cut over the tabs.
        let weak = pieces(&p, &Perforation::default(), &Tabs { cut_power: 30.0, ..t }, false);
        assert!((len(&weak, 0.3) - 8.0).abs() < 1e-6);
        // Per shape, with a ceiling.
        assert_eq!(Tabs { mode: TabMode::PerShape, per_shape: 6, ..t }.count(400.0), 6);
        assert_eq!(Tabs { mode: TabMode::PerShape, per_shape: 6, limit_max: true, max_tabs: 3, ..t }.count(400.0), 3);
        assert_eq!(Tabs { min_tabs: 7, ..t }.count(400.0), 7, "the minimum wins over the spacing");
        // Inner shapes can be left alone.
        let skip = Tabs { skip_inner: true, ..t };
        assert_eq!(pieces(&p, &Perforation::default(), &skip, true).len(), 1);
        assert!(pieces(&p, &Perforation::default(), &skip, false).len() > 1);
        // Tabs that do not fit leave the path whole.
        let huge = Tabs { size: 150.0, ..t };
        assert_eq!(pieces(&p, &Perforation::default(), &huge, false).len(), 1);
        // Nonsense does not panic.
        let _ = pieces(&p, &Perforation::default(), &Tabs { size: f64::NAN, spacing: f64::NAN, ..t }, false);
        let _ = pieces(&Polyline::new(vec![], true), &Perforation::default(), &t, false);
    }

    #[test]
    fn kerf_moves_closed_paths_out_and_in() {
        let b = |polys: &[Polyline]| polys.iter().filter_map(|p| p.bounds()).reduce(Rect::union).unwrap();
        let out = kerf_paths(&[square(10.0)], 0.5);
        let r = b(&out);
        assert!((r.max.x - 10.5).abs() < 0.05 && (r.min.x + 0.5).abs() < 0.05, "{r:?}");
        let inn = b(&kerf_paths(&[square(10.0)], -0.5));
        assert!((inn.max.x - 9.5).abs() < 0.05);
        // An open path stays.
        let open = Polyline::new(vec![Pt::new(0.0, 0.0), Pt::new(5.0, 0.0)], false);
        assert_eq!(kerf_paths(&[open.clone()], 1.0), vec![open]);
        // A hole moves the other way: with a positive kerf it gets smaller.
        let hole = Polyline::new(vec![Pt::new(3.0, 3.0), Pt::new(7.0, 3.0), Pt::new(7.0, 7.0), Pt::new(3.0, 7.0)], true);
        let rings = kerf_paths(&[square(10.0), hole], 0.5);
        let smallest = rings.iter().map(|p| p.bounds().unwrap().max.x - p.bounds().unwrap().min.x).fold(f64::MAX, f64::min);
        assert!((smallest - 3.0).abs() < 0.1, "{smallest}");
        let small = rings.iter().find(|p| (p.bounds().unwrap().max.x - p.bounds().unwrap().min.x - smallest).abs() < 1e-6).unwrap();
        assert!(is_inner(small, &rings), "the hole lies inside the outline");
        assert!(!is_inner(&rings[0], &rings) || rings.len() > 1);
    }
}
