//! Clipart: the library and the Iconify search live in base-core; putting a picture on the work area needs a document.
use crate::doc::*;
use base_core::geom::*;
pub use base_core::clipart::*;

/// Put an SVG picture on the work area: all of it on `layer` (colours are ignored), scaled so the longer side is
/// `target_mm`, centred on `center` and grouped. Returns the new shape ids.
pub fn import_clip(svg: &[u8], doc: &mut Document, layer: usize, target_mm: f64, center: Pt) -> Result<Vec<u64>, String> {
    base_core::clipart::import_clip(svg, &mut doc.scene, layer, target_mm, center)
}
