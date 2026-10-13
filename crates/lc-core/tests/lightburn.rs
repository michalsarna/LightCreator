//! A project saved by LightBurn 2.1.04 with seven layers of different kinds, one rectangle on each.
use lc_core::lightburn::import;
use lc_core::*;

#[test]
fn a_lightburn_project_becomes_a_document_with_its_layers() {
    let text = std::fs::read_to_string(format!("{}/tests/fixtures/lightburn-layers.lbrn2", env!("CARGO_MANIFEST_DIR"))).unwrap();
    let device = Device::default();
    let h = device.work_size().1;
    let d = Document::from_lightburn(import(&text, h).unwrap(), device);
    assert_eq!(d.scene.shapes.len(), 7);
    // C00 is a line at 500 mm/min and full power, C01 a fill at 83.3 mm/s and half power, C02 an offset fill.
    assert_eq!((d.layers[0].mode, d.layers[1].mode, d.layers[2].mode), (LayerMode::Line, LayerMode::Fill, LayerMode::Offset));
    assert!((d.layers[0].speed - 8.33333).abs() < 1e-9 && d.layers[0].power == 100.0);
    assert!((d.layers[1].speed - 83.3333).abs() < 1e-9 && d.layers[1].power == 50.0);
    assert_eq!(d.layers[6].power, 30.0);
    assert_eq!(d.layers[0].name, "C00");
    // The first rectangle (67 x 58 at LightBurn's 87.5, 308) lands with the drawing turned over.
    let b = d.scene.shapes[0].bounds().unwrap();
    assert!((b.width() - 67.0).abs() < 1e-9 && (b.min.x - 54.0).abs() < 1e-9);
    assert!((b.min.y - (h - 308.0 - 29.0)).abs() < 1e-9);
    // The shape is on its layer, the layers it does not use stay as new documents have them.
    assert_eq!(d.scene.shapes[3].layer, 6);
    assert_eq!(d.layers[10].mode, LayerMode::Line);
    // The result is an ordinary project: it can be saved and read again.
    let back = Document::from_json(&d.to_json()).unwrap();
    assert_eq!(back.scene.shapes.len(), 7);
}
