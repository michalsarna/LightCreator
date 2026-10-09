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
        Tool::Triangle => {
            p.add(Shape::closed_line(vec![pt(12.0, 4.5), pt(20.0, 19.0), pt(4.0, 19.0)], s));
        }
        Tool::Star => {
            let pts: Vec<_> = (0..10)
                .map(|k| {
                    let a = -std::f32::consts::FRAC_PI_2 + k as f32 / 10.0 * std::f32::consts::TAU;
                    let r = if k % 2 == 0 { 9.0 } else { 3.8 };
                    pt(12.0 + r * a.cos(), 12.5 + r * a.sin())
                })
                .collect();
            p.add(Shape::closed_line(pts, s));
        }
        Tool::Polygon => {
            let pts: Vec<_> = (0..6)
                .map(|k| {
                    let a = -std::f32::consts::FRAC_PI_2 + k as f32 / 6.0 * std::f32::consts::TAU;
                    pt(12.0 + 8.0 * a.cos(), 12.0 + 8.0 * a.sin())
                })
                .collect();
            p.add(Shape::closed_line(pts, s));
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

/// Small icons shown next to context-menu entries.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuIcon {
    FlipH,
    FlipV,
    RotCw,
    RotCcw,
    ToFront,
    ToBack,
    AlignLeft,
    AlignCenterH,
    AlignRight,
    AlignTop,
    AlignCenterV,
    AlignBottom,
    CenterOnBed,
}

pub fn paint_menu_icon(p: &Painter, r: Rect, icon: MenuIcon, c: Color32) {
    use std::f32::consts::{FRAC_PI_2, TAU};
    let s = Stroke::new(1.4, c);
    let u = r.width().min(r.height()) / 16.0;
    let o = r.center() - vec2(8.0 * u, 8.0 * u);
    let pt = |x: f32, y: f32| pos2(o.x + x * u, o.y + y * u);
    let fill = c.gamma_multiply(0.35);
    let axis = Stroke::new(1.0, c.gamma_multiply(0.7));
    let bar = |p: &Painter, a: (f32, f32), b: (f32, f32)| p.rect_filled(Rect::from_min_max(pt(a.0, a.1), pt(b.0, b.1)), 1.0, c);
    match icon {
        MenuIcon::FlipH => {
            // Mirror axis (vertical, dashed) with a triangle on each side.
            for k in 0..4 {
                p.line_segment([pt(8.0, 1.0 + k as f32 * 4.0), pt(8.0, 3.0 + k as f32 * 4.0)], axis);
            }
            p.add(Shape::Path(PathShape::convex_polygon(vec![pt(1.5, 13.0), pt(6.5, 13.0), pt(6.5, 3.0)], fill, s)));
            p.add(Shape::closed_line(vec![pt(14.5, 13.0), pt(9.5, 13.0), pt(9.5, 3.0)], s));
        }
        MenuIcon::FlipV => {
            for k in 0..4 {
                p.line_segment([pt(1.0 + k as f32 * 4.0, 8.0), pt(3.0 + k as f32 * 4.0, 8.0)], axis);
            }
            p.add(Shape::Path(PathShape::convex_polygon(vec![pt(3.0, 1.5), pt(3.0, 6.5), pt(13.0, 6.5)], fill, s)));
            p.add(Shape::closed_line(vec![pt(3.0, 14.5), pt(3.0, 9.5), pt(13.0, 9.5)], s));
        }
        MenuIcon::RotCw | MenuIcon::RotCcw => {
            let cw = icon == MenuIcon::RotCw;
            // Three quarters of a circle with an arrow head at the end.
            let (a0, a1) = (-FRAC_PI_2 + 0.5, -FRAC_PI_2 + TAU * 0.78);
            let pts: Vec<_> = (0..=24)
                .map(|k| {
                    let a = a0 + (a1 - a0) * k as f32 / 24.0;
                    let x = if cw { 8.0 + 5.5 * a.cos() } else { 8.0 - 5.5 * a.cos() };
                    pt(x, 8.0 + 5.5 * a.sin())
                })
                .collect();
            let end = *pts.last().unwrap_or(&pt(8.0, 8.0));
            p.add(Shape::line(pts, s));
            // Arrow head along the direction of travel at the end of the arc.
            let t = if cw { vec2(-a1.sin(), a1.cos()) } else { vec2(a1.sin(), a1.cos()) };
            let n = vec2(-t.y, t.x);
            let tip = end + t * 1.5 * u;
            p.add(Shape::line(vec![tip - t * 3.5 * u + n * 2.6 * u, tip, tip - t * 3.5 * u - n * 2.6 * u], s));
        }
        MenuIcon::ToFront | MenuIcon::ToBack => {
            let front = icon == MenuIcon::ToFront;
            let (a, b) = (Rect::from_min_max(pt(1.5, 1.5), pt(10.0, 10.0)), Rect::from_min_max(pt(6.0, 6.0), pt(14.5, 14.5)));
            let (back, top) = if front { (a, b) } else { (b, a) };
            p.rect_stroke(back, 1.0, s, eframe::egui::StrokeKind::Middle);
            p.rect_filled(top, 1.0, Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 90));
            p.rect_stroke(top, 1.0, s, eframe::egui::StrokeKind::Middle);
        }
        MenuIcon::AlignLeft => {
            bar(p, (1.5, 1.0), (2.5, 15.0));
            bar(p, (4.0, 3.0), (14.0, 6.5));
            bar(p, (4.0, 9.5), (10.0, 13.0));
        }
        MenuIcon::AlignRight => {
            bar(p, (13.5, 1.0), (14.5, 15.0));
            bar(p, (2.0, 3.0), (12.0, 6.5));
            bar(p, (6.0, 9.5), (12.0, 13.0));
        }
        MenuIcon::AlignCenterH => {
            bar(p, (7.5, 1.0), (8.5, 15.0));
            bar(p, (2.0, 3.0), (14.0, 6.5));
            bar(p, (4.5, 9.5), (11.5, 13.0));
        }
        MenuIcon::AlignTop => {
            bar(p, (1.0, 1.5), (15.0, 2.5));
            bar(p, (3.0, 4.0), (6.5, 14.0));
            bar(p, (9.5, 4.0), (13.0, 10.0));
        }
        MenuIcon::AlignBottom => {
            bar(p, (1.0, 13.5), (15.0, 14.5));
            bar(p, (3.0, 2.0), (6.5, 12.0));
            bar(p, (9.5, 6.0), (13.0, 12.0));
        }
        MenuIcon::AlignCenterV => {
            bar(p, (1.0, 7.5), (15.0, 8.5));
            bar(p, (3.0, 2.0), (6.5, 14.0));
            bar(p, (9.5, 4.5), (13.0, 11.5));
        }
        MenuIcon::CenterOnBed => {
            p.rect_stroke(Rect::from_min_max(pt(1.5, 1.5), pt(14.5, 14.5)), 1.0, s, eframe::egui::StrokeKind::Middle);
            p.line_segment([pt(8.0, 4.0), pt(8.0, 12.0)], axis);
            p.line_segment([pt(4.0, 8.0), pt(12.0, 8.0)], axis);
            p.circle_filled(pt(8.0, 8.0), 2.0 * u, c);
        }
    }
}

/// A small padlock, drawn at the corner of locked objects.
pub fn paint_lock(p: &Painter, center: eframe::egui::Pos2, c: Color32) {
    let body = Rect::from_center_size(center + vec2(0.0, 2.0), vec2(9.0, 7.0));
    let arc: Vec<_> = (0..=10)
        .map(|k| {
            let a = std::f32::consts::PI * (1.0 + k as f32 / 10.0);
            center + vec2(3.0 * a.cos(), -1.5 + 3.4 * a.sin())
        })
        .collect();
    p.add(Shape::line(arc, Stroke::new(1.4, c)));
    p.rect_filled(body.expand(0.8), 1.5, Color32::from_black_alpha(90));
    p.rect_filled(body, 1.5, c);
}
