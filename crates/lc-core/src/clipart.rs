//! Clipart: the library and the Iconify search live in base-core; putting a picture on the work area needs a document.
use crate::doc::*;
use base_core::geom::*;
pub use base_core::clipart::*;

/// Put an SVG picture on the work area: all of it on `layer` (colours are ignored), scaled so the longer side is
/// `target_mm`, centred on `center` and grouped. Returns the new shape ids.
pub fn import_clip(svg: &[u8], doc: &mut Document, layer: usize, target_mm: f64, center: Pt) -> Result<Vec<u64>, String> {
    let ids = crate::svg::import(svg, doc, Some(layer))?;
    if ids.is_empty() {
        return Err("the picture has no outlines".into());
    }
    let bounds = doc.bounds_of(&ids).ok_or("the picture is empty")?;
    let longest = bounds.width().max(bounds.height()).max(1e-9);
    let k = target_mm / longest;
    let c = bounds.center();
    // Scale about the picture's own centre, then move that centre to `center`.
    let xf = Xf::scale_about(k, k, c).then(Xf::translate(center.x - c.x, center.y - c.y));
    for id in &ids {
        if let Some(s) = doc.shape_mut(*id) {
            s.xf = s.xf.then(xf);
        }
    }
    if ids.len() > 1 {
        let g = doc.new_group_id();
        for id in &ids {
            if let Some(s) = doc.shape_mut(*id) {
                s.group = Some(g);
            }
        }
    }
    Ok(ids)
}
