//! The preview of a job on a rotary attachment as the workpiece itself: a cylinder along X, seen a little from the side, with the
//! burned lines wound round it (Y of the design is the way round the surface). Only the near half of the surface is drawn; dragging
//! turns the cylinder round its axis (up and down) and swings it (left and right), the wheel zooms.
use eframe::egui::{self, Color32, Pos2, Stroke};
use lc_core::Pt;

/// How the cylinder is looked at.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Orbit {
    /// The turn of the workpiece round its axis (radians), added to the middle of the job facing the viewer.
    pub spin: f64,
    /// The swing of the axis away from the viewer (radians): 0 is side on.
    pub yaw: f64,
    /// The zoom over the fitted size.
    pub zoom: f32,
}

impl Default for Orbit {
    fn default() -> Orbit {
        Orbit { spin: 0.0, yaw: 0.35, zoom: 1.0 }
    }
}

/// The cylinder: its radius, the X it spans and the Y of the design that faces the viewer at no spin.
#[derive(Clone, Copy, Debug)]
pub struct Cylinder {
    pub r: f64,
    pub x0: f64,
    pub x1: f64,
    pub y_mid: f64,
}

/// The projection of the cylinder into `rect`: design points (X along the axis, Y round it) to the screen, and whether the point is on
/// the near side.
pub struct Projection {
    c: Cylinder,
    o: Orbit,
    k: f64,
    centre: Pos2,
}

impl Projection {
    pub fn new(rect: egui::Rect, c: Cylinder, o: Orbit) -> Projection {
        let len = (c.x1 - c.x0).max(1.0);
        let (cy, sy) = (o.yaw.cos().abs(), o.yaw.sin().abs());
        // The screen width the cylinder needs at this swing: its length foreshortened, plus the ends.
        let w = len * cy + 2.0 * c.r * sy;
        let h = 2.0 * c.r;
        let fit = ((rect.width() as f64 * 0.86) / w.max(1.0)).min((rect.height() as f64 * 0.8) / h.max(1.0));
        Projection { c, o, k: fit * o.zoom as f64, centre: rect.center() }
    }

    /// The angle round the axis of a point of the design, 0 facing the viewer.
    fn phi(&self, y: f64) -> f64 {
        (y - self.c.y_mid) / self.c.r.max(1e-6) + self.o.spin
    }

    /// The point in space: X along the axis (from the middle of the cylinder), Y down the screen, Z towards the viewer.
    fn space(&self, x: f64, phi: f64, r: f64) -> [f64; 3] {
        [x - (self.c.x0 + self.c.x1) / 2.0, r * phi.sin(), r * phi.cos()]
    }

    fn screen(&self, p: [f64; 3]) -> Pos2 {
        // Swing round the vertical axis of the screen.
        let (s, c) = self.o.yaw.sin_cos();
        let x = p[0] * c + p[2] * s;
        Pos2::new(self.centre.x + (x * self.k) as f32, self.centre.y + (p[1] * self.k) as f32)
    }

    /// A point of the design on the surface: where it is on the screen, and how much it faces the viewer (> 0 on the near side).
    pub fn surface(&self, p: Pt) -> (Pos2, f64) {
        let phi = self.phi(p.y);
        (self.screen(self.space(p.x, phi, self.c.r)), phi.cos())
    }

    /// The screen pixels for one millimetre.
    #[cfg(test)]
    pub fn scale(&self) -> f64 {
        self.k
    }
}

/// The colour of the surface at `light` (0 at the edge, 1 facing the viewer).
fn shade(base: [u8; 3], light: f64) -> Color32 {
    let f = (0.45 + 0.55 * light.clamp(0.0, 1.0)) as f32;
    Color32::from_rgb((base[0] as f32 * f) as u8, (base[1] as f32 * f) as u8, (base[2] as f32 * f) as u8)
}

/// Draw the workpiece: the near half of its side in bands shaded by how they face the light, and the end that is turned towards the
/// viewer.
pub fn paint_body(painter: &egui::Painter, pj: &Projection, wood: [u8; 3]) {
    let c = pj.c;
    let n = 120;
    // The whole way round, the far half first: with the axis swung, the far side shows past the near end.
    let mut bands: Vec<(f64, f64)> = (0..n).map(|i| {
        let a0 = std::f64::consts::TAU * i as f64 / n as f64;
        (a0, a0 + std::f64::consts::TAU / n as f64)
    }).collect();
    bands.sort_by(|a, b| ((a.0 + a.1) / 2.0).cos().total_cmp(&((b.0 + b.1) / 2.0).cos()));
    for (a0, a1) in bands {
        let quad = vec![
            pj.screen(pj.space(c.x0, a0, c.r)),
            pj.screen(pj.space(c.x1, a0, c.r)),
            pj.screen(pj.space(c.x1, a1, c.r)),
            pj.screen(pj.space(c.x0, a1, c.r)),
        ];
        // Light from the upper left, a little above the viewer.
        let mid = (a0 + a1) / 2.0;
        // The far half shows only past the near end, round its edge: it is lit as the near half is, so the body reads as one.
        let light = (mid.cos().abs() * 0.85 - mid.sin() * 0.35).max(0.0);
        let col = shade(wood, light);
        painter.add(egui::Shape::convex_polygon(quad, col, Stroke::new(1.0, col)));
    }
    // The end facing the viewer: the right one when the axis swings that way.
    let end_x = if pj.o.yaw >= 0.0 { c.x1 } else { c.x0 };
    let ring: Vec<Pos2> = (0..64).map(|i| pj.screen(pj.space(end_x, std::f64::consts::TAU * i as f64 / 64.0, c.r))).collect();
    if pj.o.yaw.abs() > 0.02 {
        painter.add(egui::Shape::convex_polygon(ring.clone(), shade(wood, 0.35), Stroke::new(1.0, shade(wood, 0.2))));
        // The growth rings of the wood on the end.
        for f in [0.35, 0.65] {
            let inner: Vec<Pos2> = (0..=64).map(|i| pj.screen(pj.space(end_x, std::f64::consts::TAU * i as f64 / 64.0, c.r * f))).collect();
            painter.add(egui::Shape::line(inner, Stroke::new(1.0, shade(wood, 0.25))));
        }
    }
    let edge = Stroke::new(1.0, shade(wood, 0.15));
    for a in [-std::f64::consts::FRAC_PI_2, std::f64::consts::FRAC_PI_2] {
        painter.line_segment([pj.screen(pj.space(c.x0, a, c.r)), pj.screen(pj.space(c.x1, a, c.r))], edge);
    }
}

/// Add a line of the design wound round the surface: cut into short pieces (it curves when it goes round), the pieces on the far
/// side left out.
pub fn wound_line(out: &mut Vec<egui::Shape>, pj: &Projection, a: Pt, b: Pt, stroke: Stroke) {
    // Pieces of at most 3° of the way round.
    let step = pj.c.r.max(0.1) * 3f64.to_radians();
    let n = (((b.y - a.y).abs() / step).ceil() as usize).clamp(1, 2000);
    let mut prev = pj.surface(a);
    for i in 1..=n {
        let f = i as f64 / n as f64;
        let p = Pt::new(a.x + (b.x - a.x) * f, a.y + (b.y - a.y) * f);
        let now = pj.surface(p);
        if prev.1 > 0.0 && now.1 > 0.0 {
            out.push(egui::Shape::line_segment([prev.0, now.0], stroke));
        }
        prev = now;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pj(o: Orbit) -> Projection {
        let rect = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 400.0));
        Projection::new(rect, Cylinder { r: 25.0, x0: 0.0, x1: 200.0, y_mid: 50.0 }, o)
    }

    #[test]
    fn the_middle_of_the_job_faces_the_viewer_and_half_way_round_is_out_of_sight() {
        let p = pj(Orbit { yaw: 0.0, ..Orbit::default() });
        let (front, f) = p.surface(Pt::new(100.0, 50.0));
        assert!((front - Pos2::new(400.0, 200.0)).length() < 1e-3 && (f - 1.0).abs() < 1e-9);
        // A quarter of the way round down the design is the lower edge; half way round is at the back.
        let quarter = 50.0 + std::f64::consts::FRAC_PI_2 * 25.0;
        let (edge, fe) = p.surface(Pt::new(100.0, quarter));
        assert!(fe.abs() < 1e-9 && edge.y > front.y + 10.0);
        let (_, back) = p.surface(Pt::new(100.0, 50.0 + std::f64::consts::PI * 25.0));
        assert!(back < 0.0);
        // A whole way round comes back to the same place.
        let (again, _) = p.surface(Pt::new(100.0, 50.0 + std::f64::consts::TAU * 25.0));
        assert!((again - front).length() < 1e-3);
    }

    #[test]
    fn a_line_round_the_cylinder_shows_only_its_near_half_and_a_spin_turns_it() {
        let p = pj(Orbit::default());
        let mut out = vec![];
        // A ring all the way round at x = 100.
        wound_line(&mut out, &p, Pt::new(100.0, 50.0 - std::f64::consts::PI * 25.0), Pt::new(100.0, 50.0 + std::f64::consts::PI * 25.0), Stroke::new(1.0, Color32::RED));
        assert!(!out.is_empty() && out.len() < 70, "about half of 120 pieces: {}", out.len());
        let turned = pj(Orbit { spin: std::f64::consts::PI, ..Orbit::default() });
        assert!(turned.surface(Pt::new(100.0, 50.0)).1 < 0.0, "turned half way: the middle is at the back");
        // The zoom scales the picture.
        let big = pj(Orbit { zoom: 2.0, ..Orbit::default() });
        assert!((big.scale() / p.scale() - 2.0).abs() < 1e-9);
    }
}
