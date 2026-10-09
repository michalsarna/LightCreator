//! SVG import (via usvg) and export.
use crate::doc::*;
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
                let polys = flatten(p.data(), &xf);
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
                ids.push(doc.add(l, Kind::Path(polys), Xf::IDENTITY));
            }
            _ => {}
        }
    }
}

fn flatten(path: &usvg::tiny_skia_path::Path, xf: &Xf) -> Vec<Polyline> {
    let tp = |p: usvg::tiny_skia_path::Point| xf.apply(Pt::new(p.x as f64, p.y as f64));
    let mut out = vec![];
    let mut cur: Vec<Pt> = vec![];
    let mut last = Pt::default();
    let flush = |cur: &mut Vec<Pt>, closed: bool, out: &mut Vec<Polyline>| {
        if cur.len() >= 2 {
            let mut pts = std::mem::take(cur);
            if closed && pts.len() > 2 && pts[0].dist(*pts.last().unwrap()) < 1e-6 {
                pts.pop();
            }
            out.push(Polyline::new(pts, closed));
        } else {
            cur.clear();
        }
    };
    for seg in path.segments() {
        match seg {
            PathSegment::MoveTo(p) => {
                flush(&mut cur, false, &mut out);
                last = tp(p);
                cur.push(last);
            }
            PathSegment::LineTo(p) => {
                last = tp(p);
                cur.push(last);
            }
            PathSegment::QuadTo(c, p) => {
                let (c, p) = (tp(c), tp(p));
                flatten_quad(last, c, p, 0.05, &mut cur);
                last = p;
            }
            PathSegment::CubicTo(c1, c2, p) => {
                let (c1, c2, p) = (tp(c1), tp(c2), tp(p));
                flatten_cubic(last, c1, c2, p, 0.05, &mut cur);
                last = p;
            }
            PathSegment::Close => {
                let start = cur.first().copied();
                flush(&mut cur, true, &mut out);
                if let Some(s) = start {
                    last = s;
                    cur.push(s);
                }
            }
        }
    }
    flush(&mut cur, false, &mut out);
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
