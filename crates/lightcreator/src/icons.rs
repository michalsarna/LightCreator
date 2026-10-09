//! Hand-painted tool icons (no icon font required).
use crate::app::Tool;
use eframe::egui::{epaint::PathShape, pos2, vec2, Color32, Painter, Rect, Shape, Stroke};

pub fn paint(p: &Painter, r: Rect, tool: Tool, c: Color32) {
    let s = Stroke::new(1.6, c);
    let u = r.width().min(r.height()) / 24.0;
    let o = r.center() - vec2(12.0 * u, 12.0 * u);
    let pt = |x: f32, y: f32| pos2(o.x + x * u, o.y + y * u);
    match tool {
        Tool::Select => {
            let pts = vec![pt(7.0, 4.0), pt(7.0, 19.0), pt(11.0, 15.5), pt(14.0, 21.0), pt(16.5, 19.8), pt(13.7, 14.5), pt(18.5, 14.0)];
            p.add(Shape::Path(PathShape::convex_polygon(pts, c.gamma_multiply(0.25), s)));
        }
        Tool::Node => {
            // A curve with a node, its handle and a corner node.
            let curve: Vec<_> = (0..=20)
                .map(|k| {
                    let t = k as f32 / 20.0;
                    let (p0, c1, c2, p3) = ((4.0f32, 18.0f32), (6.0f32, 5.0f32), (16.0f32, 5.0f32), (20.0f32, 17.0f32));
                    let u = 1.0 - t;
                    let x = u * u * u * p0.0 + 3.0 * u * u * t * c1.0 + 3.0 * u * t * t * c2.0 + t * t * t * p3.0;
                    let y = u * u * u * p0.1 + 3.0 * u * u * t * c1.1 + 3.0 * u * t * t * c2.1 + t * t * t * p3.1;
                    pt(x, y)
                })
                .collect();
            p.add(Shape::line(curve, s));
            p.line_segment([pt(4.0, 18.0), pt(6.0, 5.0)], Stroke::new(1.0, c));
            p.circle_filled(pt(6.0, 5.0), 1.8 * u, c);
            p.rect_filled(Rect::from_center_size(pt(4.0, 18.0), vec2(4.5 * u, 4.5 * u)), 0.0, c);
            p.circle_filled(pt(20.0, 17.0), 2.4 * u, c);
        }
        Tool::Text => {
            p.line_segment([pt(5.0, 6.0), pt(19.0, 6.0)], Stroke::new(2.0, c));
            p.line_segment([pt(12.0, 6.0), pt(12.0, 19.0)], Stroke::new(2.0, c));
            p.line_segment([pt(9.5, 19.0), pt(14.5, 19.0)], s);
        }
        Tool::Rect => {
            p.rect_stroke(Rect::from_min_max(pt(4.5, 6.5), pt(19.5, 17.5)), 1.0, s, eframe::egui::StrokeKind::Middle);
        }
        Tool::Ellipse => {
            let n = 40;
            let pts: Vec<_> = (0..n)
                .map(|i| {
                    let t = i as f32 / n as f32 * std::f32::consts::TAU;
                    pt(12.0 + 8.0 * t.cos(), 12.0 + 6.0 * t.sin())
                })
                .collect();
            p.add(Shape::closed_line(pts, s));
        }
        Tool::Line => {
            p.line_segment([pt(5.0, 19.0), pt(19.0, 5.0)], s);
            p.circle_filled(pt(5.0, 19.0), 2.0 * u, c);
            p.circle_filled(pt(19.0, 5.0), 2.0 * u, c);
        }
        Tool::Pen => {
            let pts = vec![pt(4.0, 18.0), pt(9.0, 7.0), pt(14.0, 15.0), pt(20.0, 6.0)];
            p.add(Shape::line(pts.clone(), s));
            for q in pts {
                p.rect_filled(Rect::from_center_size(q, vec2(4.0 * u, 4.0 * u)), 0.0, c);
            }
        }
        Tool::Pan => {
            p.circle_stroke(pt(12.0, 12.0), 7.0 * u, s);
            p.line_segment([pt(12.0, 3.0), pt(12.0, 21.0)], s);
            p.line_segment([pt(3.0, 12.0), pt(21.0, 12.0)], s);
        }
        Tool::Zoom => {
            p.circle_stroke(pt(10.5, 10.5), 6.0 * u, s);
            p.line_segment([pt(15.0, 15.0), pt(20.0, 20.0)], Stroke::new(2.4, c));
            p.line_segment([pt(8.0, 10.5), pt(13.0, 10.5)], s);
            p.line_segment([pt(10.5, 8.0), pt(10.5, 13.0)], s);
        }
    }
}
