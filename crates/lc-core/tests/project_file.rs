//! The project file must stay byte for byte the same when the document is loaded and saved again. The fixture was
//! written by v0.1.5, before the scene moved to base-core.
use lc_core::*;

#[test]
fn a_project_saved_by_v0_1_5_is_saved_again_unchanged() {
    let path = format!("{}/tests/fixtures/every-kind.lcr", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(path).unwrap();
    let doc = Document::from_json(&text).unwrap();
    assert_eq!(doc.scene.shapes.len(), 5);
    assert_eq!(doc.group_of(1), vec![1, 2]);
    assert!(doc.shape(5).unwrap().locked);
    assert_eq!(doc.to_json(), text);
}
