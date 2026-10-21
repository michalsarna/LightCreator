//! SVG import (reading is in base-core) and export.
use crate::doc::*;
use std::fmt::Write;

/// Import an SVG into `doc`. Elements are assigned to the layer whose palette colour is closest to their
/// stroke (or fill) colour, unless `layer` forces one. Returns the new shape ids.
pub fn import(data: &[u8], doc: &mut Document, layer: Option<usize>) -> Result<Vec<u64>, String> {
    Ok(place(base_core::svg::parse(data)?, doc, layer, false))
}

/// Put read outlines on layers: the forced one, or the layer whose palette colour is nearest to the outline's. With `by_layers`
/// the outlines that are on an Illustrator layer go on the layer of the program with the number of that layer in the file (the
/// first on layer 1, ...; the rest on the last one); the others still go by colour.
pub(crate) fn place(outlines: Vec<base_core::svg::Outline>, doc: &mut Document, layer: Option<usize>, by_layers: bool) -> Vec<u64> {
    let names = base_core::svg::layer_names(&outlines);
    outlines
        .into_iter()
        .map(|o| {
            let named = if by_layers { o.layer.as_ref().and_then(|n| names.iter().position(|x| x == n)) } else { None };
            let l = layer.or(named.map(|k| k.min(LAYER_COUNT - 1))).unwrap_or_else(|| nearest_palette(o.color));
            doc.add(l, Kind::Bezier(o.contours), base_core::geom::Xf::IDENTITY)
        })
        .collect()
}

/// Export the document as an SVG sized to the machine bed (units: mm).
pub fn export(doc: &Document) -> String {
    let (bw, bh) = doc.device.work_size();
    let mut s = String::new();
    let _ = writeln!(
        s,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}mm\" height=\"{h}mm\" viewBox=\"0 0 {w} {h}\">",
        w = bw,
        h = bh
    );
    for sh in &doc.scene.shapes {
        let c = PALETTE[doc.layers[sh.layer].color.min(crate::LAYER_COUNT - 1)];
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
