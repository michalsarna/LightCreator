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
    let mut s = Shape { id: 1, layer: 0, kind: Kind::Rect { w: 10.0, h: 4.0 }, xf: Xf::translate(3.0, 2.0), group: None };
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
    let s = Shape { id: 1, layer: 0, kind: Kind::Text(t), xf: Xf::IDENTITY, group: None };
    let b = s.bounds().unwrap();
    assert!(b.width() > 5.0 && b.height() > 4.0 && b.height() < 12.0, "{:?}", b);
}

#[test]
fn hpgl_and_dxf_export() {
    let mut d = Document::default();
    d.layers[2].mode = LayerMode::Line;
    d.add(2, Kind::Rect { w: 10.0, h: 5.0 }, Xf::translate(20.0, 30.0));
    d.add(0, Kind::Image(ImageData { name: "i".into(), px_w: 1, px_h: 1, gray: vec![0], w: 5.0, h: 5.0, invert: false }), Xf::IDENTITY);
    assert_eq!(export::skipped_images(&d), 1);
    let h = export::hpgl(&d);
    assert!(h.starts_with("IN;") && h.contains("SP3;") && h.contains("PD"));
    // Front-left origin: y is flipped, so (20, 30) mm becomes (800, 370*40) plotter units.
    assert!(h.contains(&format!("PU800,{}", (400.0f64 - 30.0) as i64 * 40)), "{h}");
    let x = export::dxf(&d);
    assert!(x.contains("POLYLINE") && x.contains("SEQEND") && x.contains("C02") && x.ends_with("EOF\n"));
}

#[test]
fn material_presets_apply_to_layers() {
    let lib = materials::builtin();
    assert!(lib.iter().any(|p| p.laser == LaserKind::Co2) && lib.iter().any(|p| p.laser == LaserKind::Diode));
    assert!(lib.iter().all(|p| p.speed > 0.0 && p.power > 0.0 && p.power <= 100.0 && p.passes >= 1));
    let mut l = Layer::new(0);
    let cut = lib.iter().find(|p| p.material == "Plywood 3 mm" && p.operation == "Cut" && p.laser == LaserKind::Diode).unwrap();
    cut.apply(&mut l);
    assert_eq!((l.mode, l.passes, l.power), (LayerMode::Line, 2, 100.0));
    let user = materials::Preset::from_layer("Mine", "Cut", LaserKind::Diode, &l);
    assert!(user.user && user.speed == l.speed);
}

#[test]
fn layer_order_groups_and_old_files() {
    let mut d = Document::default();
    d.move_layer(0, false);
    assert_eq!(&d.layer_order()[..3], &[1, 0, 2]);
    d.move_layer(0, true);
    assert_eq!(&d.layer_order()[..3], &[0, 1, 2]);
    d.move_layer(0, true); // already first: no change
    assert_eq!(d.layer_order()[0], 0);
    // Broken order lists are repaired.
    d.order = vec![5, 5, 99, 1];
    let o = d.layer_order();
    assert_eq!(o.len(), 30);
    assert_eq!(&o[..2], &[5, 1]);
    // Output follows the order: layer 1 is burnt before layer 0.
    let mut d = Document::default();
    d.layers[0].power = 11.0;
    d.layers[1].power = 22.0;
    d.add(0, Kind::Rect { w: 5.0, h: 5.0 }, Xf::IDENTITY);
    d.add(1, Kind::Rect { w: 5.0, h: 5.0 }, Xf::translate(20.0, 0.0));
    d.move_layer(1, true);
    d.move_layer(1, true);
    let job = gcode::generate(&d);
    let first_layer = job.moves.iter().find(|m| m.laser).unwrap().layer;
    assert_eq!(first_layer, 1);
    // Groups.
    let a = d.add(0, Kind::Rect { w: 1.0, h: 1.0 }, Xf::IDENTITY);
    let b = d.add(1, Kind::Rect { w: 1.0, h: 1.0 }, Xf::IDENTITY);
    let g = d.new_group_id();
    d.shape_mut(a).unwrap().group = Some(g);
    d.shape_mut(b).unwrap().group = Some(g);
    assert_eq!(d.group_of(a).len(), 2);
    assert_eq!((d.shape(a).unwrap().layer, d.shape(b).unwrap().layer), (0, 1));
    // A document saved before these fields existed still loads.
    let mut v: serde_json::Value = serde_json::from_str(&Document::default().to_json()).unwrap();
    v.as_object_mut().unwrap().remove("order");
    v.as_object_mut().unwrap().remove("next_group");
    assert_eq!(Document::from_json(&v.to_string()).unwrap().layer_order().len(), 30);
}

fn tiny_pdf(content: &str) -> Vec<u8> {
    use lopdf::{dictionary, Document as Pdf, Object, Stream};
    let mut pdf = Pdf::with_version("1.5");
    let pages_id = pdf.new_object_id();
    let content_id = pdf.add_object(Stream::new(dictionary! {}, content.as_bytes().to_vec()));
    let page_id = pdf.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 200.into(), 100.into()],
        "Contents" => content_id,
    });
    pdf.objects.insert(pages_id, Object::Dictionary(dictionary! {
        "Type" => "Pages", "Kids" => vec![page_id.into()], "Count" => 1,
    }));
    let catalog = pdf.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    pdf.trailer.set("Root", catalog);
    let mut out = Vec::new();
    pdf.save_to(&mut out).unwrap();
    out
}

#[test]
fn illustrator_pdf_paths_become_shapes() {
    // Red stroked rectangle (x 10..110, y 20..60 in PDF space, y up) and a blue curve.
    let data = tiny_pdf("1 0 0 RG 10 20 100 40 re S 0 0 1 RG 0 0 m 30 50 70 50 100 0 c S");
    let mut d = Document::default();
    let ids = pdf::import(&data, &mut d, None).unwrap();
    assert_eq!(ids.len(), 2);
    let rect = d.shape(ids[0]).unwrap();
    assert_eq!(rect.layer, 2); // red
    let b = rect.bounds().unwrap();
    // 100 pt wide = 35.28 mm; y is flipped: the top of the page is y = 0 (page is 100 pt = 35.28 mm tall).
    assert!((b.width() - 100.0 * 25.4 / 72.0).abs() < 1e-6);
    assert!((b.min.y - (100.0 - 60.0) * 25.4 / 72.0).abs() < 1e-6);
    let curve = d.shape(ids[1]).unwrap();
    assert_eq!(curve.layer, 1); // blue
    assert!(matches!(&curve.kind, Kind::Bezier(cs) if cs[0].nodes.len() == 2 && cs[0].seg_is_curve(0)));
}

fn square_with_hole() -> ImageData {
    // 40x40 white image, black 20x20 square at (10,10) with a white 6x6 hole.
    let mut g = vec![255u8; 40 * 40];
    for y in 10..30 {
        for x in 10..30 {
            g[y * 40 + x] = 0;
        }
    }
    for y in 17..23 {
        for x in 17..23 {
            g[y * 40 + x] = 255;
        }
    }
    ImageData { name: "t".into(), px_w: 40, px_h: 40, gray: g, w: 40.0, h: 40.0, invert: false }
}

#[test]
fn trace_finds_outline_and_hole() {
    let im = square_with_hole();
    let cs = trace::trace(&im, &trace::TraceParams { smooth: false, tolerance: 0.0, ..Default::default() });
    assert_eq!(cs.len(), 2, "outline and hole");
    let areas: Vec<f64> = cs.iter().map(|c| c.flatten(0.01).area().abs()).collect();
    let (big, small) = (areas.iter().cloned().fold(0.0, f64::max), areas.iter().cloned().fold(f64::MAX, f64::min));
    assert!((big - 400.0).abs() < 1.0 && (small - 36.0).abs() < 1.0, "{areas:?}");
    // Speckle filter drops the hole.
    let cs = trace::trace(&im, &trace::TraceParams { min_area: 50.0, ..Default::default() });
    assert_eq!(cs.len(), 1);
    // Tracing the light areas finds the background frame and the hole's island.
    let cs = trace::trace(&im, &trace::TraceParams { invert: true, smooth: false, min_area: 1.0, ..Default::default() });
    assert_eq!(cs.len(), 3);
}

#[test]
fn image_adjustments() {
    let im = square_with_hole();
    let rot = im.adjusted(&Adjust { quarter_turns: 1, ..Default::default() });
    assert_eq!((rot.px_w, rot.px_h), (40, 40));
    let wide = ImageData { px_w: 4, px_h: 2, gray: vec![0, 10, 20, 30, 40, 50, 60, 70], w: 40.0, h: 20.0, ..im.clone() };
    let r = wide.adjusted(&Adjust { quarter_turns: 1, ..Default::default() });
    assert_eq!((r.px_w, r.px_h, r.w, r.h), (2, 4, 20.0, 40.0));
    // Clockwise: the top-left pixel ends up top-right.
    assert_eq!(r.gray[1], 0);
    let f = wide.adjusted(&Adjust { flip_h: true, ..Default::default() });
    assert_eq!(&f.gray[..4], &[30, 20, 10, 0]);
    let b = wide.adjusted(&Adjust { brightness: 0.5, ..Default::default() });
    assert!(b.gray[0] >= 127 && b.gray[7] > wide.gray[7]);
    let c = wide.adjusted(&Adjust { auto_levels: true, ..Default::default() });
    assert_eq!((c.gray[0], c.gray[7]), (0, 255));
    let free = im.adjusted(&Adjust { angle_deg: 45.0, ..Default::default() });
    assert!(free.px_w > 40 && free.px_w < 60);
}
