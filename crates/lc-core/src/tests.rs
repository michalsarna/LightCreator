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

#[test]
fn bezier_ellipse_area_and_node_insert() {
    let c = Contour::ellipse(20.0, 10.0);
    let area = c.flatten(0.01).area().abs();
    assert!((area - std::f64::consts::PI * 10.0 * 5.0).abs() < 0.5, "area {area}");
    // Inserting a node must not change the outline.
    let mut c2 = c.clone();
    let before = c2.point_at(1, 0.37);
    let at = c2.insert_node(1, 0.37);
    assert!(c2.nodes[at].p.dist(before) < 1e-9);
    let area2 = c2.flatten(0.01).area().abs();
    assert!((area - area2).abs() < 0.1);
    // Straight segment insert lands on the line.
    let mut r = Contour::rect(10.0, 10.0);
    let at = r.insert_node(0, 0.5);
    assert_eq!(r.nodes[at].p, Pt::new(5.0, 0.0));
}

#[test]
fn shape_to_bezier_keeps_geometry() {
    let mut s = Shape { id: 1, layer: 0, kind: Kind::Rect { w: 10.0, h: 4.0 }, xf: Xf::translate(3.0, 2.0) };
    let before = s.bounds().unwrap();
    s.to_bezier();
    let after = s.bounds().unwrap();
    assert!((before.min.x - after.min.x).abs() < 1e-9 && (before.max.y - after.max.y).abs() < 1e-9);
    assert!(matches!(s.kind, Kind::Bezier(_)));
}

fn sq(x: f64, y: f64, s: f64) -> Polyline {
    Polyline::new(vec![Pt::new(x, y), Pt::new(x + s, y), Pt::new(x + s, y + s), Pt::new(x, y + s)], true)
}

fn total_area(p: &[Polyline]) -> f64 {
    // Outer rings positive, holes negative (opposite orientation).
    p.iter().map(|q| q.area()).sum::<f64>().abs()
}

#[test]
fn boolean_ops_on_squares() {
    let (a, b) = (vec![sq(0.0, 0.0, 4.0)], vec![sq(2.0, 2.0, 4.0)]);
    assert!((total_area(&ops::boolean(&a, &b, ops::BoolOp::Union)) - 28.0).abs() < 1e-6);
    assert!((total_area(&ops::boolean(&a, &b, ops::BoolOp::Intersect)) - 4.0).abs() < 1e-6);
    assert!((total_area(&ops::boolean(&a, &b, ops::BoolOp::Difference)) - 12.0).abs() < 1e-6);
    assert!((total_area(&ops::boolean(&a, &b, ops::BoolOp::Xor)) - 24.0).abs() < 1e-6);
}

#[test]
fn offset_shrinks_and_grows() {
    let a = vec![sq(0.0, 0.0, 10.0)];
    let inner = ops::offset(&a, -1.0);
    assert!((total_area(&inner) - 64.0).abs() < 0.5, "{}", total_area(&inner));
    let outer = ops::offset(&a, 1.0);
    assert!(total_area(&outer) > 120.0);
    let rings = ops::inset_rings(&a, 1.0, 100);
    assert!(rings.len() >= 4 && rings.len() <= 6, "{}", rings.len());
}

fn gradient_image(w: u32, h: u32) -> ImageData {
    // Left half black, right half white.
    let gray = (0..w * h).map(|i| if (i % w) < w / 2 { 0u8 } else { 255u8 }).collect();
    ImageData { name: "t".into(), px_w: w, px_h: h, gray, w: 20.0, h: 10.0, invert: false }
}

#[test]
fn image_raster_burns_only_the_dark_half() {
    let mut d = Document::default();
    d.layers[0].interval = 0.5;
    d.layers[0].dither = Dither::Threshold;
    d.layers[0].overscan = 0.0;
    d.add(0, Kind::Image(gradient_image(20, 10)), Xf::translate(10.0, 10.0));
    let job = gcode::generate(&d);
    let burnt: f64 = job.moves.iter().filter(|m| m.laser).map(|m| m.a.dist(m.b)).sum();
    // 20 rows of 10 mm each.
    assert!((burnt - 200.0).abs() < 1.0, "burnt {burnt}");
    assert!(job.moves.iter().filter(|m| m.laser).all(|m| m.a.x >= 10.0 - 1e-6 && m.b.x <= 20.0 + 1e-6));
}

#[test]
fn grayscale_scales_power_and_dithering_keeps_density() {
    let mut d = Document::default();
    // Uniform 50 % grey.
    let im = ImageData { name: "g".into(), px_w: 10, px_h: 10, gray: vec![128; 100], w: 10.0, h: 10.0, invert: false };
    d.layers[0].interval = 0.25;
    d.layers[0].dither = Dither::FloydSteinberg;
    d.add(0, Kind::Image(im.clone()), Xf::IDENTITY);
    let job = gcode::generate(&d);
    let burnt: f64 = job.moves.iter().filter(|m| m.laser).map(|m| m.a.dist(m.b)).sum();
    // About half of 40 rows x 10 mm.
    assert!((burnt - 200.0).abs() < 25.0, "burnt {burnt}");
    d.layers[0].dither = Dither::Grayscale;
    d.layers[0].power = 80.0;
    let job = gcode::generate(&d);
    assert!(job.gcode.contains("S399") || job.gcode.contains("S400"), "{}", &job.gcode[..200.min(job.gcode.len())]);
}

#[test]
fn offset_layer_makes_concentric_rings() {
    let mut d = Document::default();
    d.layers[0].mode = LayerMode::Offset;
    d.layers[0].interval = 1.0;
    d.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::translate(5.0, 5.0));
    let job = gcode::generate(&d);
    let cut: f64 = job.cut_length;
    // Rings 10, 8, 6, 4, 2 mm squares: perimeters 40+32+24+16+8 = 120 (a few more or less at the centre).
    assert!(cut > 100.0 && cut < 140.0, "cut {cut}");
}

#[test]
fn text_produces_outlines() {
    let t = TextData { text: "Hi".into(), size: 10.0, ..TextData::default() };
    let c = text::contours(&t);
    assert!(c.len() >= 3, "{}", c.len());
    let s = Shape { id: 1, layer: 0, kind: Kind::Text(t), xf: Xf::IDENTITY };
    let b = s.bounds().unwrap();
    assert!(b.width() > 5.0 && b.height() > 4.0 && b.height() < 12.0, "{:?}", b);
}
