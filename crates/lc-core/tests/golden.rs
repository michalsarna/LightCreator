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
    assert!(!snippet_ok(&"X".repeat(101)));
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

#[test]
fn arcs_for_circles_when_asked() {
    let mut doc = Document::default();
    doc.layers[0].mode = LayerMode::Line;
    doc.layers[0].speed = 20.0;
    doc.layers[0].power = 60.0;
    doc.add(0, Kind::Ellipse { w: 40.0, h: 40.0 }, Xf::translate(100.0, 80.0));
    doc.add(0, Kind::Rect { w: 20.0, h: 10.0 }, Xf::translate(30.0, 40.0));
    let mut dev = device(&[]);
    dev.gcode.arcs = true;
    let text = gcode_for(doc.clone(), dev.clone());
    check("grbl_m4_arcs", &text);
    assert!(text.contains("G2 ") || text.contains("G3 "));
    // The rectangle stays four straight lines.
    assert!(text.lines().any(|l| l.starts_with("G1 X") && l.contains("Y160")));
    // Without the option the same document has no arcs, and the files of the other tests are untouched.
    assert!(!gcode_for(doc, device(&[])).contains("G2 "));
}

#[test]
fn two_heads_get_their_tool_numbers_and_their_own_s_scale() {
    let mut dev = device(&[]);
    dev.heads[0].name = "Blue diode".into();
    dev.heads.push(LaserHead { name: "Fibre".into(), tool: 2, s_max: 255.0, laser: LaserKind::Fiber, ..LaserHead::default() });
    let mut doc = layers_doc();
    doc.layers[1].head = 1; // layer 0 and 2 burn with the first head, layer 1 with the second
    let text = gcode_for(doc.clone(), dev.clone());
    check("grbl_m4_two_heads", &text);
    let tools: Vec<&str> = text.lines().filter(|l| l.starts_with('T')).collect();
    assert_eq!(tools, vec!["T2 ; laser head Fibre", "T0 ; laser head Blue diode"]);
    // Layer 1 is at 100 %: full scale of its own head (255), the others use 1000 for 60 % and 35 %.
    assert!(text.contains(" S255"));
    assert!(text.contains(" S600"));
    assert!(text.contains(" S350"));
    // A layer that names a head the device does not have uses the first one; one head never writes T.
    let mut lost = layers_doc();
    lost.layers[0].head = 9;
    assert!(!gcode_for(lost, device(&[])).lines().any(|l| l.starts_with('T')));
    // With a second head defined but not used, nothing changes either.
    let unused = gcode_for(layers_doc(), dev);
    assert!(!unused.lines().any(|l| l.starts_with('T')));
}

fn hooked_doc() -> (Document, Device) {
    let mut doc = Document::default();
    doc.layers[0].mode = LayerMode::Line;
    doc.layers[0].speed = 20.0;
    doc.layers[0].power = 60.0;
    doc.layers[1].mode = LayerMode::Line;
    doc.add(0, Kind::Rect { w: 20.0, h: 10.0 }, Xf::translate(30.0, 40.0));
    doc.add(0, Kind::Ellipse { w: 12.0, h: 12.0 }, Xf::translate(80.0, 40.0));
    doc.add(1, Kind::Rect { w: 5.0, h: 5.0 }, Xf::translate(120.0, 40.0));
    doc.job_info = JobInfo { name: "badge".into(), date: "2026-10-11".into(), time: "14:05".into() };
    let mut dev = device(&[]);
    dev.name = "Golden".into();
    dev.gcode.hooks.layer_start = Hook { enabled: true, code: "M117 {layer_name} {layer_index}/{total_layers}\n; {job_name} {date} {time}".into() };
    dev.gcode.hooks.layer_end = Hook { enabled: true, code: "M117 done {layer_name}".into() };
    dev.gcode.hooks.object_start = Hook { enabled: true, code: "; start {workpiece_name} {workpiece_index}/{total_workpieces} on {machine_name}\n; {work_width}x{work_height} T{tool_number}".into() };
    dev.gcode.hooks.object_end = Hook { enabled: true, code: "; end {workpiece_index}".into() };
    (doc, dev)
}

#[test]
fn hooks_run_in_order_with_their_variables() {
    let (doc, dev) = hooked_doc();
    let text = gcode_for(doc, dev);
    check("hooks_all_four", &text);
    let marks: Vec<&str> = text.lines().filter(|l| l.starts_with("; hook")).collect();
    assert_eq!(
        marks,
        vec![
            "; hook layer_start", "; hook object_start", "; hook object_end", "; hook object_start", "; hook object_end", "; hook layer_end",
            "; hook layer_start", "; hook object_start", "; hook object_end", "; hook layer_end",
        ]
    );
    assert!(text.contains("M117 C00 1/2") && text.contains("M117 C01 2/2"));
    assert!(text.contains("; badge 2026-10-11 14:05"));
    assert!(text.contains("; start Rectangle 1/3 on Golden") && text.contains("; 300x200 T0") && text.contains("; start Ellipse 2/3") && text.contains("; start Rectangle 3/3"));
    assert!(text.contains("; end 1") && text.contains("M117 done C01"));
}

#[test]
fn without_hooks_or_with_them_switched_off_the_job_is_the_plain_one() {
    let (doc, mut dev) = hooked_doc();
    for i in 0..4 {
        dev.gcode.hooks.get_mut(i).enabled = false;
    }
    let off = gcode_for(doc.clone(), dev);
    assert!(!off.contains("hook"));
    let mut plain = device(&[]);
    plain.name = "Golden".into();
    assert_eq!(off, gcode_for(doc, plain));
}

#[test]
fn only_layer_hooks_keep_the_paths_of_a_layer_together() {
    let (doc, mut dev) = hooked_doc();
    dev.gcode.hooks.object_start.enabled = false;
    dev.gcode.hooks.object_end.enabled = false;
    let with_layer_hooks = gcode_for(doc.clone(), dev);
    assert!(with_layer_hooks.contains("hook layer_start") && !with_layer_hooks.contains("hook object_start"));
    // The moves between the hooks are those of a job without any hook.
    let strip = |t: &str| t.lines().filter(|l| l.starts_with('G')).collect::<Vec<_>>().join("\n");
    let mut plain = device(&[]);
    plain.name = "Golden".into();
    assert_eq!(strip(&with_layer_hooks), strip(&gcode_for(doc, plain)));
}

#[test]
fn hooks_are_validated_before_they_are_kept() {
    let mut h = Hooks::default();
    assert_eq!(h.problem(), None);
    h.layer_start.code = "M117 {layer_nam}".into();
    assert_eq!(h.problem().map(|p| p.0), Some("layer_start"));
    h.layer_start.code = "M117 {layer_name".into();
    assert!(h.problem().is_some());
    h.layer_start.code = "M117 }".into();
    assert!(h.problem().is_some());
    h.layer_start.code = "M117 {}".into();
    assert!(h.problem().is_some());
    h.layer_start.code = "M117 {layer_name}\u{7}".into();
    assert!(h.problem().is_some());
    h.layer_start.code = "M117 {layer_name} {job_name} {pwm_freq}".into();
    h.object_end.code = "; {workpiece_index}".into();
    assert_eq!(h.problem(), None);
    // Variables of a job are not allowed in a macro; the ones of the machine are.
    assert!(lc_core::hooks::check_variables("G4 P{layer_index}", lc_core::hooks::MACRO_VARS).is_err());
    assert!(lc_core::hooks::check_variables("M117 {machine_name} {pulse_width}", lc_core::hooks::MACRO_VARS).is_ok());
}
