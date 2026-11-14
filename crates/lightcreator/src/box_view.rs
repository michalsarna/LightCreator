//! The preview of a job as the burned object itself, in 3D: the piece of material of the project (a sheet, or a post or a block worked
//! on several faces) as a box, with what is burned drawn on its faces. The camera, the projection, the box and the letters of the
//! faces are the 3D preview of base (`base_ui::view3d`), the same as the one of MillCreator: the object lies as the laser sees the
//! shown face (that face up), so the switch of the faces turns it. Here: the lines of a face on the object, and the colour of a burn.
use base_ui::view3d::{FaceFrame, Projector};
use eframe::egui::{self, Color32, Stroke};
use lc_core::faces::Solid;
use lc_core::Pt;

/// Add a line of the drawing of face `face` (its millimetres, from the top left corner of the face) where it lies on the object, when
/// that face is turned to the eye.
pub fn face_line(out: &mut base_ui::lines::Lines, pj: &Projector, frame: &FaceFrame, solid: &Solid, face: usize, a: Pt, b: Pt, stroke: Stroke) {
    if pj.facing(frame.out(face)) <= 1e-6 {
        return;
    }
    out.add(pj.project(frame.at(solid, face, a)), pj.project(frame.at(solid, face, b)), stroke, false);
}

/// The colour of a burn on wood: the darker the more power the layer gives (a cut through is black).
pub fn burn_colour(wood: [u8; 3], power_pct: f64) -> Color32 {
    let p = (power_pct / 100.0).clamp(0.0, 1.0) as f32;
    let f = 0.55 - 0.45 * p;
    let base = base_ui::view3d::shade(wood, f);
    // A little browner than the wood is darker.
    Color32::from_rgb(base.r(), (base.g() as f32 * 0.88) as u8, (base.b() as f32 * 0.7) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base_ui::view3d::Camera;
    use eframe::egui::Pos2;

    fn rect() -> egui::Rect {
        egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0))
    }

    fn projector(frame: &FaceFrame, cam: &Camera) -> Projector {
        let (target, radius) = frame.framing();
        Projector::new(cam, target, radius, rect())
    }

    #[test]
    fn a_line_of_a_face_lies_on_that_face_and_only_a_face_turned_to_the_eye_is_drawn() {
        let s = Solid::post(40.0, 30.0, 250.0);
        let frame = FaceFrame::of(&s, 0);
        let pj = projector(&frame, &Camera::default());
        let mut out = base_ui::lines::Lines::new(rect());
        // Face A (top) and B (front) are seen, C (bottom) and D (back) are not.
        for face in 0..4 {
            face_line(&mut out, &pj, &frame, &s, face, Pt::new(10.0, 5.0), Pt::new(100.0, 5.0), Stroke::new(1.0, Color32::RED));
        }
        assert_eq!(out.shapes().len(), 2);
        // A line along the top edge of face B lies on the front edge of face A.
        let (a_edge, b_edge) = (pj.project(frame.at(&s, 0, Pt::new(50.0, 40.0))), pj.project(frame.at(&s, 1, Pt::new(50.0, 0.0))));
        assert!((a_edge - b_edge).length() < 1e-3);
    }

    #[test]
    fn the_shown_face_is_up_so_switching_the_face_turns_the_object() {
        let s = Solid::block(100.0, 80.0, 60.0);
        for shown in 0..6 {
            let frame = FaceFrame::of(&s, shown);
            let pj = projector(&frame, &Camera::default());
            // The shown face is seen from above, the face under it is not.
            assert!(pj.facing(frame.out(shown)) > 0.5, "face {shown}");
            let under = (0..6).find(|j| { let o = frame.out(*j); o[2] < -0.99 }).unwrap();
            assert!(pj.facing(frame.out(under)) < 0.0);
            assert_ne!(under, shown);
        }
        // Face B up is another picture than face A up.
        let (a, b) = (FaceFrame::of(&s, 0), FaceFrame::of(&s, 1));
        assert_ne!(a.out(1), b.out(1));
    }

    #[test]
    fn a_strong_burn_is_darker() {
        assert!(burn_colour([200, 160, 120], 100.0).r() < burn_colour([200, 160, 120], 10.0).r());
    }
}
