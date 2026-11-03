//! What LightCreator prints: the PDF itself is written in baseCreator (`base_core::print`), the same in all the programs.
use crate::doc::*;
pub use base_core::print::*;

/// The design as the printing sees it.
pub fn source(doc: &Document) -> PrintSource<'_> {
    let layers = doc
        .layers
        .iter()
        .map(|l| PrintLayer { rgb: PALETTE[l.color % PALETTE.len()], filled: matches!(l.mode, LayerMode::Fill | LayerMode::FillAndLine | LayerMode::Offset), visible: l.visible })
        .collect();
    PrintSource { shapes: &doc.scene.shapes, layers, order: doc.layer_order(), work: doc.device.work_size(), program: "LightCreator", device: &doc.device.name }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::*;

    #[test]
    fn the_design_is_printed_with_its_layers_and_the_size_of_the_work_area() {
        let mut d = Document::default();
        d.device.area.bed_w = 200.0;
        d.device.area.bed_h = 100.0;
        d.device.name = "Laser (test)".into();
        d.layers[0].mode = LayerMode::Fill;
        d.add(0, Kind::Rect { w: 50.0, h: 30.0 }, Xf::translate(10.0, 10.0));
        let text = String::from_utf8_lossy(&pdf(&source(&d), &PrintOptions::default()).unwrap()).to_string();
        assert!(text.starts_with("%PDF-1.4") && text.contains("f*"), "a fill layer is tinted");
        assert!(text.contains("(LightCreator - Laser \\(test\\) - work area"), "{text}");
        assert_eq!(region(&source(&d), &PrintOptions::default()).unwrap().max.x, 200.0);
        d.layers[0].visible = false;
        assert!(included(&source(&d), &PrintOptions::default()).is_empty());
    }
}
