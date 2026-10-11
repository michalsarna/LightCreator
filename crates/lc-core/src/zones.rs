//! No-go zones: parts of the work area the machine must never enter (a clamp, a fixture, a mirror). A move that
//! touches an enabled zone stops the job from being sent, framed or exported.
use crate::doc::*;
use crate::geom::*;

/// Does the straight move `a` to `b` touch the zone (its inside or its edge)?
pub fn segment_hits(shape: &ZoneShape, a: Pt, b: Pt) -> bool {
    match *shape {
        ZoneShape::Rect { x, y, w, h } => {
            // Clip the segment to the rectangle (Liang-Barsky); what is left means it touches.
            let (x0, x1) = (x.min(x + w), x.max(x + w));
            let (y0, y1) = (y.min(y + h), y.max(y + h));
            let (dx, dy) = (b.x - a.x, b.y - a.y);
            let (mut t0, mut t1) = (0.0f64, 1.0f64);
            for (p, q) in [(-dx, a.x - x0), (dx, x1 - a.x), (-dy, a.y - y0), (dy, y1 - a.y)] {
                if p == 0.0 {
                    if q < 0.0 {
                        return false;
                    }
                } else {
                    let r = q / p;
                    if p < 0.0 {
                        if r > t1 {
                            return false;
                        }
                        t0 = t0.max(r);
                    } else {
                        if r < t0 {
                            return false;
                        }
                        t1 = t1.min(r);
                    }
                }
            }
            t0 <= t1
        }
        ZoneShape::Circle { x, y, r } => dist_to_segment(Pt::new(x, y), a, b) <= r.abs(),
    }
}

fn dist_to_segment(p: Pt, a: Pt, b: Pt) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len2 = dx * dx + dy * dy;
    if len2 < 1e-18 {
        return p.dist(a);
    }
    let t = (((p.x - a.x) * dx + (p.y - a.y) * dy) / len2).clamp(0.0, 1.0);
    p.dist(Pt::new(a.x + t * dx, a.y + t * dy))
}

/// The enabled zones the move touches, by index.
pub fn zones_hit(dev: &Device, a: Pt, b: Pt) -> Vec<usize> {
    dev.zones.iter().enumerate().filter(|(_, z)| z.enabled && segment_hits(&z.shape, a, b)).map(|(i, _)| i).collect()
}

impl ZoneShape {
    /// The smallest rectangle that holds the zone.
    pub fn bounds(&self) -> Rect {
        match *self {
            ZoneShape::Rect { x, y, w, h } => Rect { min: Pt::new(x.min(x + w), y.min(y + h)), max: Pt::new(x.max(x + w), y.max(y + h)) },
            ZoneShape::Circle { x, y, r } => Rect { min: Pt::new(x - r.abs(), y - r.abs()), max: Pt::new(x + r.abs(), y + r.abs()) },
        }
    }
}
