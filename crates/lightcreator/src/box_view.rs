//! The preview of a job as the burned object itself, in 3D: the piece of material of the project (a sheet, or a post or a block worked
//! on several faces) as a box seen from a turnable point of view, with what is burned drawn on its faces. The object has the space of
//! `base_core::faces`: X to the right, Y from the back to the front, Z up; the picture turns it with the mouse (as the 3D view of
//! MillCreator does, with the same navigation cube) and zooms with the wheel. The faces are drawn plain and lit by how they turn to
//! the light; the lines of the job of each face lie on it, the far faces left out.
use eframe::egui::{self, Color32, Pos2, Stroke};
use lc_core::faces::{Solid, V3};
use lc_core::Pt;

const START_YAW: f32 = 0.6;
const START_PITCH: f32 = 0.55;

/// How the object is looked at.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Orbit3 {
    /// The turn round the vertical (radians).
    pub yaw: f32,
    /// How far the eye is above the horizon (radians), from below to above the object.
    pub pitch: f32,
    /// The zoom over the fitted size.
    pub zoom: f32,
}

impl Default for Orbit3 {
    fn default() -> Orbit3 {
        Orbit3 { yaw: START_YAW, pitch: START_PITCH, zoom: 1.0 }
    }
}

impl Orbit3 {
    /// Turn by a drag of the mouse.
    pub fn drag(&mut self, d: egui::Vec2) {
        self.yaw = (self.yaw - d.x * 0.01).rem_euclid(std::f32::consts::TAU);
        self.pitch = (self.pitch + d.y * 0.01).clamp(-1.5, 1.5);
    }
    /// Look from a direction of the world (from the object to the eye), as the navigation cube asks.
    pub fn look_from(&mut self, d: [f32; 3]) {
        self.pitch = d[2].clamp(-1.0, 1.0).asin().clamp(-1.5, 1.5);
        if d[0].hypot(d[1]) > 1e-4 {
            self.yaw = d[0].atan2(-d[1]).rem_euclid(std::f32::consts::TAU);
        }
    }
    /// The direction to the eye, and the right and up of the picture, in the world (X right, Y back, Z up).
    pub fn axes(&self) -> ([f32; 3], [f32; 3], [f32; 3]) {
        let (sy, cy, sp, cp) = (self.yaw.sin(), self.yaw.cos(), self.pitch.sin(), self.pitch.cos());
        ([sy * cp, -cy * cp, sp], [cy, sy, 0.0], [-sy * sp, cy * sp, cp])
    }
}

/// The object in the world of the picture: Y of the object goes to the front, the world's to the back.
fn world(p: V3) -> [f64; 3] {
    [p[0], -p[1], p[2]]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// The projection of the object into a rectangle of the screen.
pub struct BoxProjection {
    right: [f64; 3],
    up: [f64; 3],
    eye: [f64; 3],
    mid: [f64; 3],
    k: f64,
    centre: Pos2,
}

impl BoxProjection {
    pub fn new(rect: egui::Rect, solid: &Solid, o: Orbit3) -> BoxProjection {
        let (eye, right, up) = o.axes();
        let f = |v: [f32; 3]| [v[0] as f64, v[1] as f64, v[2] as f64];
        let b = solid.bbox();
        let mid = world([b[0] / 2.0, b[1] / 2.0, b[2] / 2.0]);
        let radius = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt() / 2.0;
        let fit = (rect.width().min(rect.height()) as f64 * 0.9) / (2.0 * radius.max(1.0));
        BoxProjection { right: f(right), up: f(up), eye: f(eye), mid, k: fit * o.zoom as f64, centre: rect.center() }
    }

    /// A point of the object on the screen.
    pub fn at(&self, p: V3) -> Pos2 {
        let w = world(p);
        let d = [w[0] - self.mid[0], w[1] - self.mid[1], w[2] - self.mid[2]];
        Pos2::new(self.centre.x + (dot(d, self.right) * self.k) as f32, self.centre.y - (dot(d, self.up) * self.k) as f32)
    }

    /// Is a face with this outward direction (in the object) turned to the eye?
    pub fn facing(&self, out: V3) -> f64 {
        dot(world(out), self.eye)
    }

    /// The screen pixels for one millimetre.
    pub fn scale(&self) -> f64 {
        self.k
    }
}

/// The colour `base` made lighter or darker by `f`.
pub fn shade(base: [u8; 3], f: f32) -> Color32 {
    let m = |v: u8| ((v as f32) * f).round().clamp(0.0, 255.0) as u8;
    Color32::from_rgb(m(base[0]), m(base[1]), m(base[2]))
}

/// How light a face is: lit from the upper left in front.
fn light_of(out: V3) -> f32 {
    let l = [-0.35f64, -0.45, 0.82];
    (0.62 + 0.38 * dot(world(out), l).max(0.0)) as f32
}

/// The six sides of the box of the object as it lies for face A, each with its outward direction.
fn box_sides(b: V3) -> [([V3; 4], V3); 6] {
    let (x, y, z) = (b[0], b[1], b[2]);
    [
        ([[0.0, 0.0, z], [x, 0.0, z], [x, y, z], [0.0, y, z]], [0.0, 0.0, 1.0]),
        ([[0.0, 0.0, 0.0], [x, 0.0, 0.0], [x, y, 0.0], [0.0, y, 0.0]], [0.0, 0.0, -1.0]),
        ([[0.0, y, 0.0], [x, y, 0.0], [x, y, z], [0.0, y, z]], [0.0, 1.0, 0.0]),
        ([[0.0, 0.0, 0.0], [x, 0.0, 0.0], [x, 0.0, z], [0.0, 0.0, z]], [0.0, -1.0, 0.0]),
        ([[0.0, 0.0, 0.0], [0.0, y, 0.0], [0.0, y, z], [0.0, 0.0, z]], [-1.0, 0.0, 0.0]),
        ([[x, 0.0, 0.0], [x, y, 0.0], [x, y, z], [x, 0.0, z]], [1.0, 0.0, 0.0]),
    ]
}

/// Draw the object: the sides of its box that are turned to the eye, plain and lit, with the letters of the faces that are worked on
/// (faint, in their middle) when there is more than one.
pub fn paint_body(painter: &egui::Painter, pj: &BoxProjection, solid: &Solid, wood: [u8; 3], letters: bool) {
    let b = solid.bbox();
    for (corners, out) in box_sides(b) {
        if pj.facing(out) <= 1e-6 {
            continue;
        }
        let lit = light_of(out);
        let pts: Vec<Pos2> = corners.iter().map(|c| pj.at(*c)).collect();
        painter.add(egui::Shape::convex_polygon(pts, shade(wood, lit), Stroke::new(1.0, shade(wood, lit * 0.62))));
    }
    if !letters {
        return;
    }
    for (i, f) in solid.faces().iter().enumerate() {
        if pj.facing(f.out) <= 0.05 {
            continue;
        }
        let mid = pj.at(f.at(Pt::new(f.w / 2.0, f.h / 2.0), 0.0));
        let size = ((f.w.min(f.h) * pj.scale()) as f32 * 0.35).clamp(10.0, 42.0);
        painter.text(mid, egui::Align2::CENTER_CENTER, Solid::letter(i), egui::FontId::proportional(size), shade(wood, 0.5).gamma_multiply(0.55));
    }
}

/// Add a line of the drawing of face `face` (its millimetres, from the top left corner of the face) where it lies on the object, when
/// that face is turned to the eye.
pub fn face_line(out: &mut Vec<egui::Shape>, pj: &BoxProjection, solid: &Solid, face: usize, a: Pt, b: Pt, stroke: Stroke) {
    let f = solid.face(face);
    if pj.facing(f.out) <= 1e-6 {
        return;
    }
    out.push(egui::Shape::line_segment([pj.at(f.at(a, 0.0)), pj.at(f.at(b, 0.0))], stroke));
}

/// The colour of a burn on wood: the darker the more power the layer gives (a cut through is black).
pub fn burn_colour(wood: [u8; 3], power_pct: f64) -> Color32 {
    let p = (power_pct / 100.0).clamp(0.0, 1.0) as f32;
    let f = 0.55 - 0.45 * p;
    let base = shade(wood, f);
    // A little browner than the wood is darker.
    Color32::from_rgb(base.r(), (base.g() as f32 * 0.88) as u8, (base.b() as f32 * 0.7) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lc_core::setup::FlipAxis;

    fn rect() -> egui::Rect {
        egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0))
    }

    #[test]
    fn from_above_the_top_is_seen_and_the_bottom_is_not() {
        let s = Solid::sheet(200.0, 100.0, 10.0, FlipAxis::SideToSide);
        let pj = BoxProjection::new(rect(), &s, Orbit3::default());
        assert!(pj.facing([0.0, 0.0, 1.0]) > 0.0 && pj.facing([0.0, 0.0, -1.0]) < 0.0);
        // The front (the bottom of the drawing) faces the viewer at the first turn; the back does not.
        assert!(pj.facing([0.0, 1.0, 0.0]) > 0.0 && pj.facing([0.0, -1.0, 0.0]) < 0.0);
        // From below the bottom is seen.
        let under = BoxProjection::new(rect(), &s, Orbit3 { pitch: -0.8, ..Orbit3::default() });
        assert!(under.facing([0.0, 0.0, -1.0]) > 0.0);
        // The middle of the object is the middle of the picture.
        let c = pj.at([100.0, 50.0, 5.0]);
        assert!((c - rect().center()).length() < 1e-3);
    }

    #[test]
    fn a_line_of_a_face_lies_on_that_face_and_only_a_face_turned_to_the_eye_is_drawn() {
        let s = Solid::post(40.0, 30.0, 250.0);
        let pj = BoxProjection::new(rect(), &s, Orbit3::default());
        let mut out = vec![];
        // Face A (top) and B (front) are seen, C (bottom) and D (back) are not.
        for face in 0..4 {
            face_line(&mut out, &pj, &s, face, Pt::new(10.0, 5.0), Pt::new(100.0, 5.0), Stroke::new(1.0, Color32::RED));
        }
        assert_eq!(out.len(), 2);
        // A line along the top edge of face B lies on the front edge of face A.
        let (a_edge, b_edge) = (pj.at(s.face(0).at(Pt::new(50.0, 40.0), 0.0)), pj.at(s.face(1).at(Pt::new(50.0, 0.0), 0.0)));
        assert!((a_edge - b_edge).length() < 1e-3);
    }

    #[test]
    fn the_cube_turns_the_view_and_a_strong_burn_is_darker() {
        let mut o = Orbit3::default();
        o.look_from([0.0, -1.0, 0.0]);
        let (eye, _, _) = o.axes();
        assert!(eye[1] < -0.99, "from the front: {eye:?}");
        assert!(burn_colour([200, 160, 120], 100.0).r() < burn_colour([200, 160, 120], 10.0).r());
    }
}
