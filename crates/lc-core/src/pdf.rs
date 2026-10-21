//! Import of PDF-compatible Adobe Illustrator (`.ai`) files, and plain PDFs: vector paths of the first page
//! (reading is in base-core).
use crate::doc::*;

/// Import the first page of a PDF / Illustrator file. Colours map to the nearest layer colour unless `layer`
/// forces one. Returns the new shape ids.
pub fn import(data: &[u8], doc: &mut Document, layer: Option<usize>) -> Result<Vec<u64>, String> {
    Ok(crate::svg::place(base_core::pdf::parse(data)?, doc, layer, false))
}

/// Put outlines read from a PDF / Illustrator file on `doc`: by colour, or by the Illustrator layers they were on (the first layer
/// of the file on layer 1, the second on layer 2, ...). Returns the new shape ids.
pub fn place_outlines(outlines: Vec<base_core::svg::Outline>, doc: &mut Document, by_layers: bool) -> Vec<u64> {
    crate::svg::place(outlines, doc, None, by_layers)
}

/// What a PDF / Illustrator file was read into.
pub type Outlines = Vec<base_core::svg::Outline>;

/// Read a PDF / Illustrator file (the pages side by side) without putting it on a document yet.
pub fn read(data: &[u8]) -> Result<Outlines, String> {
    base_core::pdf::parse(data)
}

/// How many Illustrator layers the outlines are on.
pub fn layer_count(outlines: &Outlines) -> usize {
    base_core::svg::layer_names(outlines).len()
}
