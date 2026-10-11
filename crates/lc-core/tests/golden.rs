//! Golden files: the G-code of fixed documents on fixed devices must stay byte for byte the same. Devices are
//! built from the profile JSON of the first releases, so the test also guards the reading of old profiles.
//! Run with `UPDATE_GOLDEN=1` to rewrite the files after a deliberate change of the output.
use lc_core::*;
use lc_core::gcode;

const BASE_PROFILE: &str = r#"{"name":"Golden","bed_w":300.0,"bed_h":200.0,"origin":"FrontLeft","s_max":1000.0,"dynamic_power":true,"travel_speed":3000.0,"return_home":true,"baud":115200}"#;

fn device(patch: &[(&str, serde_json::Value)]) -> Device {
    let mut v: serde_json::Value = serde_json::from_str(BASE_PROFILE).unwrap();
    for (k, x) in patch {
        v[*k] = x.clone();
    }
    serde_json::from_value(v).unwrap()
}

fn check(name: &str, text: &str) {
    let path = format!("{}/tests/golden/{name}.gcode", env!("CARGO_MANIFEST_DIR"));
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        std::fs::write(&path, text).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("missing golden file {path}"));
    assert_eq!(text, want, "G-code of {name} changed");
}

fn rect_doc(mode: LayerMode) -> Document {
    let mut d = Document::default();
    d.layers[0].mode = mode;
    d.layers[0].interval = 1.0;
    d.layers[0].speed = 20.0;
    d.layers[0].power = 60.0;
    d.add(0, Kind::Rect { w: 20.0, h: 10.0 }, Xf::translate(30.0, 40.0));
    d
}

fn image_doc() -> Document {
    let mut d = Document::default();
    d.layers[0].interval = 0.5;
    d.layers[0].dither = Dither::Threshold;
    d.layers[0].overscan = 1.0;
    let gray = (0..16 * 8).map(|i| if (i % 16) < 8 { 0u8 } else { 255u8 }).collect();
    d.add(0, Kind::Image(ImageData { name: "t".into(), px_w: 16, px_h: 8, gray, w: 16.0, h: 8.0, invert: false }), Xf::translate(50.0, 50.0));
    d
}

fn layers_doc() -> Document {
    let mut d = rect_doc(LayerMode::FillAndLine);
    d.layers[0].passes = 2;
    d.layers[0].air_assist = true;
    d.layers[1].mode = LayerMode::Line;
    d.layers[1].speed = 5.0;
    d.layers[1].power = 100.0;
    d.add(1, Kind::Ellipse { w: 16.0, h: 10.0 }, Xf::translate(100.0, 60.0));
    d.layers[2].mode = LayerMode::Offset;
    d.layers[2].interval = 1.5;
    d.layers[2].power = 35.0;
    d.add(2, Kind::Rect { w: 15.0, h: 15.0 }, Xf::translate(150.0, 100.0));
    d
}

fn gcode_for(mut doc: Document, dev: Device) -> String {
    doc.device = dev;
    gcode::generate(&doc).gcode
}

#[test]
fn grbl_m4_line_fill_offset_image_and_layers() {
    let dev = || device(&[]);
    check("grbl_m4_line", &gcode_for(rect_doc(LayerMode::Line), dev()));
    check("grbl_m4_fill", &gcode_for(rect_doc(LayerMode::Fill), dev()));
    check("grbl_m4_offset", &gcode_for(rect_doc(LayerMode::Offset), dev()));
    check("grbl_m4_image", &gcode_for(image_doc(), dev()));
    check("grbl_m4_layers_air_passes", &gcode_for(layers_doc(), dev()));
}

#[test]
fn grbl_m3_and_back_left_origin_without_return() {
    let m3 = device(&[("dynamic_power", false.into())]);
    check("grbl_m3_line", &gcode_for(rect_doc(LayerMode::Line), m3));
    let back = device(&[("origin", "BackLeft".into()), ("return_home", false.into())]);
    check("grbl_back_left_no_return", &gcode_for(layers_doc(), back));
}

#[test]
fn marlin_line_and_layers() {
    let dev = || device(&[("controller", "Marlin".into())]);
    check("marlin_line", &gcode_for(rect_doc(LayerMode::Line), dev()));
    check("marlin_layers_air_passes", &gcode_for(layers_doc(), dev()));
}

#[test]
fn frame_gcode_for_both_dialects() {
    let b = lc_core::Rect { min: Pt::new(30.0, 40.0), max: Pt::new(80.0, 70.0) };
    check("frame_grbl", &gcode::frame_gcode(b, &device(&[]), 1.5, 25.0));
    check("frame_marlin", &gcode::frame_gcode(b, &device(&[("controller", "Marlin".into())]), 0.0, 25.0));
}

#[test]
fn smoothieware_and_forced_dialects() {
    let smooth = || device(&[("controller", "Smoothieware".into()), ("s_max", 1.0.into())]);
    check("smoothie_line", &gcode_for(rect_doc(LayerMode::Line), smooth()));
    check("smoothie_layers_air_passes", &gcode_for(layers_doc(), smooth()));
    check("frame_smoothie", &gcode::frame_gcode(lc_core::Rect { min: Pt::new(30.0, 40.0), max: Pt::new(80.0, 70.0) }, &smooth(), 1.5, 25.0));
    // A chosen GRBL dialect decides between M3 and M4 whatever the "dynamic power" flag says.
    let mut forced = device(&[("dynamic_power", true.into())]);
    forced.gcode.dialect = Some(Dialect::Grbl);
    check("grbl_forced_m3", &gcode_for(rect_doc(LayerMode::Line), forced));
}

#[test]
fn own_start_end_and_air_commands() {
    let mut dev = device(&[]);
    dev.gcode.preamble = "G21\nG90\n; own start\nM3 S0".into();
    dev.gcode.postscript = "M5\nM30".into();
    dev.gcode.air_on = "M7".into();
    dev.gcode.air_off = "M9\nM8".into();
    check("own_start_end_air", &gcode_for(layers_doc(), dev));
}

#[test]
fn unfit_own_gcode_falls_back_to_the_built_in() {
    let mut dev = device(&[]);
    dev.gcode.preamble = "G21\u{7}".into();
    dev.gcode.postscript = "M5\u{1b}[2J".into();
    assert_eq!(dev.gcode.invalid_fields(), vec!["preamble", "postscript"]);
    let out = gcode_for(rect_doc(LayerMode::Line), dev);
    assert_eq!(out, std::fs::read_to_string(format!("{}/tests/golden/grbl_m4_line.gcode", env!("CARGO_MANIFEST_DIR"))).unwrap());
}

#[test]
fn snippet_limits() {
    assert!(snippet_ok("") && snippet_ok("G21\n\n; comment\nM3 S0\n"));
    assert!(!snippet_ok("G1\tX1")); // tabs are control characters too
    assert!(!snippet_ok(&"G0 X1\n".repeat(61)));
    assert!(!snippet_ok(&"X".repeat(201)));
    assert!(!snippet_ok(&format!("{}\n", "G1 ; ".repeat(500))));
    let mut c = GcodeCfg::default();
    c.air_on = "M7".into();
    assert!(c.invalid_fields().is_empty());
    c.reset_custom();
    assert!(c.air_on.is_empty());
}

#[test]
fn layers_switch_between_constant_and_dynamic_power() {
    let mut doc = layers_doc();
    doc.layers[1].power_mode = PowerMode::Constant;
    // Layer 2 inherits: after a constant layer the device's dynamic mode comes back.
    check("grbl_m4_layer_power_modes", &gcode_for(doc.clone(), device(&[])));
    let text = gcode_for(doc.clone(), device(&[]));
    let modes: Vec<&str> = text.lines().filter(|l| l.contains("power") && (l.starts_with("M3") || l.starts_with("M4"))).collect();
    assert_eq!(modes, vec!["M3 ; constant power", "M4 ; dynamic power"]);
    // Dialects that cannot switch ignore the setting.
    let marlin = gcode_for(doc.clone(), device(&[("controller", "Marlin".into())]));
    assert!(!marlin.contains("constant power") && !marlin.contains("dynamic power"));
    // A constant-power device with a dynamic layer switches the other way round.
    let mut dynamic_layer = layers_doc();
    dynamic_layer.layers[2].power_mode = PowerMode::Dynamic;
    let t = gcode_for(dynamic_layer, device(&[("dynamic_power", false.into())]));
    assert!(t.contains("M4 ; dynamic power") && !t.contains("M3 ; constant power"));
}
