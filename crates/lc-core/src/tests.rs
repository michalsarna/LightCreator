use crate::*;

fn square_doc(mode: LayerMode) -> Document {
    let mut d = Document::default();
    d.layers[0].mode = mode;
    d.layers[0].interval = 1.0;
    d.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::translate(5.0, 5.0));
    d
}

#[test]
fn xf_inverse_roundtrip() {
    let x = Xf::rotate(0.7).then(Xf::scale(2.0, 3.0)).then(Xf::translate(4.0, -1.0));
    let p = Pt::new(1.5, 2.5);
    let q = x.inverse().unwrap().apply(x.apply(p));
    assert!(p.dist(q) < 1e-9);
}

#[test]
fn line_mode_cuts_perimeter() {
    let job = gcode::generate(&square_doc(LayerMode::Line));
    assert!((job.cut_length - 40.0).abs() < 1e-6, "{}", job.cut_length);
    assert!(job.gcode.contains("M4 S0") && job.gcode.contains("M5"));
}

#[test]
fn fill_mode_covers_area() {
    let job = gcode::generate(&square_doc(LayerMode::Fill));
    // 10 scan lines of length 10 (overscan moves are S0 and not counted).
    assert!((job.cut_length - 100.0).abs() < 1e-6, "{}", job.cut_length);
}

#[test]
fn fill_even_odd_hole() {
    let outer = Polyline::new(vec![Pt::new(0., 0.), Pt::new(10., 0.), Pt::new(10., 10.), Pt::new(0., 10.)], true);
    let hole = Polyline::new(vec![Pt::new(2., 2.), Pt::new(8., 2.), Pt::new(8., 8.), Pt::new(2., 8.)], true);
    let lines = gcode::fill_lines(&[outer, hole], 1.0, 0.0, true);
    let total: f64 = lines.iter().map(|(a, b)| a.dist(*b)).sum();
    assert!((total - (100.0 - 36.0)).abs() < 1e-6, "{total}");
}

#[test]
fn inner_cut_before_outer() {
    let mut d = Document::default();
    d.add(0, Kind::Rect { w: 50.0, h: 50.0 }, Xf::IDENTITY);
    d.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::translate(20.0, 20.0));
    let paths = gcode::order_paths(d.shapes.iter().flat_map(|s| s.polys()).collect(), Pt::new(0.0, 0.0));
    assert!(paths[0].area().abs() < paths[1].area().abs());
}

#[test]
fn y_flip_for_front_left_origin() {
    let mut d = square_doc(LayerMode::Line);
    d.device.bed_h = 100.0;
    let job = gcode::generate(&d);
    assert!(job.gcode.contains("Y95.000"), "{}", job.gcode);
}

#[test]
fn svg_roundtrip() {
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect x="10" y="10" width="40" height="20" stroke="#ff0000" fill="none"/></svg>"##;
    let mut d = Document::default();
    let ids = svg::import(svg, &mut d, None).unwrap();
    assert_eq!(ids.len(), 1);
    assert_eq!(d.shapes[0].layer, 2); // red -> C02
    let b = d.shapes[0].bounds().unwrap();
    assert!((b.width() - 40.0 * 25.4 / 96.0).abs() < 1e-3);
    assert!(svg::export(&d).contains("<path"));
}

#[test]
fn json_roundtrip() {
    let d = square_doc(LayerMode::Fill);
    let d2 = Document::from_json(&d.to_json()).unwrap();
    assert_eq!(d2.shapes.len(), 1);
}

#[test]
fn units_convert_and_old_profiles_default_to_mm() {
    assert!((Units::Inch.to_mm(2.0) - 50.8).abs() < 1e-9);
    assert!((Units::Inch.from_mm(25.4) - 1.0).abs() < 1e-9);
    // A device saved before units existed has no "units" key.
    let old = r#"{"name":"x","bed_w":300.0,"bed_h":200.0,"origin":"FrontLeft","s_max":1000.0,"dynamic_power":true,"travel_speed":3000.0,"return_home":true,"baud":115200}"#;
    let d: Device = serde_json::from_str(old).unwrap();
    assert_eq!(d.units, Units::Mm);
}

#[test]
fn status_parsing_for_both_dialects() {
    let (st, x, y) = controller::parse_status("<Idle|MPos:12.500,3.250,0.000|FS:0,0>").unwrap();
    assert_eq!((st.as_str(), x, y), ("Idle", 12.5, 3.25));
    let (_, x, y) = controller::parse_status("X:10.00 Y:20.50 Z:0.00 E:0.00 Count X:800 Y:1640").unwrap();
    assert_eq!((x, y), (10.0, 20.5));
    assert!(controller::parse_status("ok").is_none());
}

#[test]
fn old_device_json_gets_new_fields() {
    let old = r#"{"name":"x","bed_w":300.0,"bed_h":200.0,"origin":"FrontLeft","s_max":1000.0,"dynamic_power":true,"travel_speed":3000.0,"return_home":true,"baud":115200}"#;
    let d: Device = serde_json::from_str(old).unwrap();
    assert_eq!(d.controller, Controller::Grbl);
    assert_eq!(d.jog_step, 5.0);
    assert!(d.port.is_empty());
}
