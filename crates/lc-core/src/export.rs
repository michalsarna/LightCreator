//! Vector job export for controllers that are fed from files: HPGL (`.plt`) and DXF.
//! Ruida (RDWorks) and Trocen software import both and apply speed and power per layer colour.
use crate::doc::*;
use crate::gcode::{fill_lines, order_paths};
use crate::geom::*;
use std::fmt::Write;

/// Paths of one layer as the laser would trace them (hatching and rings included, images excluded).
pub fn layer_paths(doc: &Document, li: usize) -> Vec<Polyline> {
    let layer = &doc.layers[li];
    let polys: Vec<Polyline> = doc.shapes.iter().filter(|s| s.layer == li && !s.is_image()).flat_map(|s| s.polys()).collect();
    let mut out = vec![];
    if matches!(layer.mode, LayerMode::Fill | LayerMode::FillAndLine) {
        out.extend(fill_lines(&polys, layer.interval, layer.angle, layer.bidirectional).into_iter().map(|(a, b)| Polyline::new(vec![a, b], false)));
    }
    if layer.mode == LayerMode::Offset {
        out.extend(order_paths(crate::ops::inset_rings(&polys, layer.interval, 10_000), Pt::new(0.0, 0.0)));
    }
    if matches!(layer.mode, LayerMode::Line | LayerMode::FillAndLine) {
        out.extend(order_paths(polys, Pt::new(0.0, 0.0)));
    }
    out
}

fn out_y(dev: &Device, y: f64) -> f64 {
    let y = y.clamp(0.0, dev.bed_h);
    match dev.origin {
        Origin::FrontLeft => dev.bed_h - y,
        Origin::BackLeft => y,
    }
}

/// Layers that have something to output.
fn active_layers(doc: &Document) -> Vec<usize> {
    doc.layer_order().into_iter().filter(|&li| doc.layers[li].output && doc.shapes.iter().any(|s| s.layer == li)).collect()
}

/// Number of images that cannot be represented in these formats.
pub fn skipped_images(doc: &Document) -> usize {
    doc.shapes.iter().filter(|s| s.is_image() && doc.layers[s.layer].output).count()
}

/// HPGL: 40 plotter units per millimetre, one pen per layer (pen = layer index + 1).
pub fn hpgl(doc: &Document) -> String {
    let dev = &doc.device;
    let u = |v: f64| (v * 40.0).round() as i64;
    let mut s = String::from("IN;\n");
    for li in active_layers(doc) {
        let paths = layer_paths(doc, li);
        if paths.is_empty() {
            continue;
        }
        let layer = &doc.layers[li];
        let _ = writeln!(s, "SP{};", li + 1);
        let _ = writeln!(s, "VS{:.1};", (layer.speed / 10.0).max(0.1));
        for p in paths {
            if p.pts.len() < 2 {
                continue;
            }
            let pt = |q: &Pt| format!("{},{}", u(q.x.clamp(0.0, dev.bed_w)), u(out_y(dev, q.y)));
            let _ = writeln!(s, "PU{};", pt(&p.pts[0]));
            let mut pd: Vec<String> = p.pts[1..].iter().map(pt).collect();
            if p.closed {
                pd.push(pt(&p.pts[0]));
            }
            let _ = writeln!(s, "PD{};", pd.join(","));
        }
    }
    s.push_str("PU;\nSP0;\n");
    s
}

/// Closest of the nine basic AutoCAD colour indices to a palette entry.
fn aci(rgb: [u8; 3]) -> u32 {
    const BASIC: [(u32, [u8; 3]); 9] = [
        (1, [255, 0, 0]),
        (2, [255, 255, 0]),
        (3, [0, 255, 0]),
        (4, [0, 255, 255]),
        (5, [0, 0, 255]),
        (6, [255, 0, 255]),
        (7, [0, 0, 0]),
        (8, [128, 128, 128]),
        (9, [192, 192, 192]),
    ];
    let d = |c: &[u8; 3]| -> i32 { (0..3).map(|i| (c[i] as i32 - rgb[i] as i32).pow(2)).sum() };
    BASIC.iter().min_by_key(|(_, c)| d(c)).map(|(i, _)| *i).unwrap_or(7)
}

/// DXF (R12 ASCII), millimetres, one DXF layer per LightCreator layer (named C00..C29).
pub fn dxf(doc: &Document) -> String {
    let dev = &doc.device;
    let layers = active_layers(doc);
    let mut s = String::new();
    s.push_str("0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1009\n9\n$INSUNITS\n70\n4\n0\nENDSEC\n");
    let _ = write!(s, "0\nSECTION\n2\nTABLES\n0\nTABLE\n2\nLAYER\n70\n{}\n", layers.len());
    for &li in &layers {
        let l = &doc.layers[li];
        let _ = write!(s, "0\nLAYER\n2\n{}\n70\n0\n62\n{}\n6\nCONTINUOUS\n", l.name, aci(PALETTE[l.color.min(29)]));
    }
    s.push_str("0\nENDTAB\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n");
    for &li in &layers {
        let name = &doc.layers[li].name;
        for p in layer_paths(doc, li) {
            if p.pts.len() < 2 {
                continue;
            }
            let _ = write!(s, "0\nPOLYLINE\n8\n{}\n66\n1\n70\n{}\n", name, if p.closed { 1 } else { 0 });
            for q in &p.pts {
                let _ = write!(s, "0\nVERTEX\n8\n{}\n10\n{:.4}\n20\n{:.4}\n", name, q.x.clamp(0.0, dev.bed_w), out_y(dev, q.y));
            }
            let _ = write!(s, "0\nSEQEND\n8\n{}\n", name);
        }
    }
    s.push_str("0\nENDSEC\n0\nEOF\n");
    s
}
