//! Import of PDF-compatible Adobe Illustrator (`.ai`) files, and plain PDFs: vector paths of the first page
//! (reading is in base-core).
use crate::doc::*;

/// Import the first page of a PDF / Illustrator file. Colours map to the nearest layer colour unless `layer`
/// forces one. Returns the new shape ids.
pub fn import(data: &[u8], doc: &mut Document, layer: Option<usize>) -> Result<Vec<u64>, String> {
    Ok(crate::svg::place(base_core::pdf::parse(data)?, doc, layer))
}
