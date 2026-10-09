//! SVG import (via usvg) and export.
use crate::doc::*;
use crate::bezier::{Contour, Node};
use crate::geom::*;
use std::fmt::Write;
use usvg::tiny_skia_path::PathSegment;

const PX_TO_MM: f64 = 25.4 / 96.0;

/// Import an SVG into `doc`. Elements are assigned to the layer whose palette colour is closest to their
/// stroke (or fill) colour, unless `layer` forces one. Returns the new shape ids.
pub fn import(data: &[u8], doc: &mut Document, layer: Option<usize>) -> Result<Vec<u64>, String> {
    let tree = usvg::Tree::from_data(data, &usvg::Options::default()).map_err(|e| e.to_string())?;
    let mut ids = vec![];
    walk(tree.root(), doc, layer, &mut ids);
    Ok(ids)
}

fn walk(g: &usvg::Group, doc: &mut Document, layer: Option<usize>, ids: &mut Vec<u64>) {
    for node in g.children() {
        match node {
            usvg::Node::Group(g) => walk(g, doc, layer, ids),
            usvg::Node::Text(t) => walk(t.flattened(), doc, layer, ids),
            usvg::Node::Path(p) => {
                let t = p.abs_transform();
                let xf = Xf { a: t.sx as f64, b: t.ky as f64, c: t.kx as f64, d: t.sy as f64, e: t.tx as f64, f: t.ty as f64 }
                    .then(Xf::scale(PX_TO_MM, PX_TO_MM));
                let polys = contours(p.data(), &xf);
                if polys.is_empty() {
                    continue;
                }
                let color = [p.stroke().map(|s| s.paint()), p.fill().map(|f| f.paint())]
                    .into_iter()
                    .flatten()
                    .find_map(|paint| match paint {
                        usvg::Paint::Color(c) => Some([c.red, c.green, c.blue]),
                        _ => None,
                    })
                    .unwrap_or([0, 0, 0]);
                let l = layer.unwrap_or_else(|| nearest_palette(color));
                ids.push(doc.add(l, Kind::Bezier(polys), Xf::IDENTITY));
            }
            _ => {}
        }
    }
}

/// Convert an SVG path into editable Bézier contours (curves are kept, not flattened).
fn contours(path: &usvg::tiny_skia_path::Path, xf: &Xf) -> Vec<Contour> {
    let tp = |p: usvg::tiny_skia_path::Point| xf.apply(Pt::new(p.x as f64, p.y as f64));
    let mut out: Vec<Contour> = vec![];
    let mut cur: Vec<Node> = vec![];
    let finish = |cur: &mut Vec<Node>, closed: bool, out: &mut Vec<Contour>| {
        let mut nodes = std::mem::take(cur);
        if closed && nodes.len() > 2 && nodes[0].p.dist(nodes[nodes.len() - 1].p) < 1e-6 {
            let last = nodes.pop().unwrap_or(nodes[0]);
            nodes[0].hin = last.hin;
        }
        if nodes.len() >= 2 {
            for n in &mut nodes {
                n.smooth = n.has_in() && n.has_out() && {
                    let (a, b) = (Pt::new(n.p.x - n.hin.x, n.p.y - n.hin.y), Pt::new(n.hout.x - n.p.x, n.hout.y - n.p.y));
                    let cross = a.x * b.y - a.y * b.x;
                    let dot = a.x * b.x + a.y * b.y;
                    dot > 0.0 && cross.abs() <= 0.02 * (dot.abs() + 1e-12)
                };
            }
            out.push(Contour { nodes, closed });
        }
    };
    let mut last = Pt::default();
    for seg in path.segments() {
        match seg {
            PathSegment::MoveTo(p) => {
                finish(&mut cur, false, &mut out);
                last = tp(p);
                cur.push(Node::corner(last));
            }
            PathSegment::LineTo(p) => {
                last = tp(p);
                cur.push(Node::corner(last));
            }
            PathSegment::QuadTo(c, p) => {
                let (c, p) = (tp(c), tp(p));
                let c1 = Pt::new(last.x + 2.0 / 3.0 * (c.x - last.x), last.y + 2.0 / 3.0 * (c.y - last.y));
                let c2 = Pt::new(p.x + 2.0 / 3.0 * (c.x - p.x), p.y + 2.0 / 3.0 * (c.y - p.y));
                if let Some(n) = cur.last_mut() {
                    n.hout = c1;
                }
                cur.push(Node { p, hin: c2, hout: p, smooth: false });
                last = p;
            }
            PathSegment::CubicTo(c1, c2, p) => {
                let (c1, c2, p) = (tp(c1), tp(c2), tp(p));
                if let Some(n) = cur.last_mut() {
                    n.hout = c1;
                }
                cur.push(Node { p, hin: c2, hout: p, smooth: false });
                last = p;
            }
            PathSegment::Close => {
                let start = cur.first().map(|n| n.p);
                finish(&mut cur, true, &mut out);
                if let Some(s) = start {
                    last = s;
                    cur.push(Node::corner(s));
                }
            }
        }
    }
    finish(&mut cur, false, &mut out);
    out
}

/// Export the document as an SVG sized to the machine bed (units: mm).
pub fn export(doc: &Document) -> String {
    let d = &doc.device;
    let mut s = String::new();
    let _ = writeln!(
        s,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}mm\" height=\"{h}mm\" viewBox=\"0 0 {w} {h}\">",
        w = d.bed_w,
        h = d.bed_h
    );
    for sh in &doc.shapes {
        let c = PALETTE[doc.layers[sh.layer].color.min(29)];
        let mut data = String::new();
        for p in sh.polys() {
            for (i, q) in p.pts.iter().enumerate() {
                let _ = write!(data, "{}{:.3} {:.3} ", if i == 0 { "M" } else { "L" }, q.x, q.y);
            }
            if p.closed {
                data.push('Z');
            }
        }
        let _ = writeln!(
            s,
            "  <path d=\"{}\" fill=\"none\" stroke=\"#{:02x}{:02x}{:02x}\" stroke-width=\"0.1\"/>",
            data.trim(),
            c[0],
            c[1],
            c[2]
        );
    }
    s.push_str("</svg>\n");
    s
}
