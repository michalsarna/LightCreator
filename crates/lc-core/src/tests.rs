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
    let paths = gcode::order_paths(d.scene.shapes.iter().flat_map(|s| s.polys()).collect(), Pt::new(0.0, 0.0));
    assert!(paths[0].area().abs() < paths[1].area().abs());
}

#[test]
fn y_flip_for_front_left_origin() {
    let mut d = square_doc(LayerMode::Line);
    d.device.area.bed_h = 100.0;
    let job = gcode::generate(&d);
    assert!(job.gcode.contains("Y95.000"), "{}", job.gcode);
}

#[test]
fn svg_roundtrip() {
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect x="10" y="10" width="40" height="20" stroke="#ff0000" fill="none"/></svg>"##;
    let mut d = Document::default();
    let ids = svg::import(svg, &mut d, None).unwrap();
    assert_eq!(ids.len(), 1);
    assert_eq!(d.scene.shapes[0].layer, nearest_palette([255, 0, 0])); // red -> the nearest colour
    let b = d.scene.shapes[0].bounds().unwrap();
    assert!((b.width() - 40.0 * 25.4 / 96.0).abs() < 1e-3);
    assert!(svg::export(&d).contains("<path"));
}

#[test]
fn json_roundtrip() {
    let d = square_doc(LayerMode::Fill);
    let d2 = Document::from_json(&d.to_json()).unwrap();
    assert_eq!(d2.scene.shapes.len(), 1);
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
    assert_eq!(d.jog_step, 10.0);
    assert!(d.connection.port.is_empty());
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
    let mut s = Shape { id: 1, layer: 0, kind: Kind::Rect { w: 10.0, h: 4.0 }, xf: Xf::translate(3.0, 2.0), group: None, locked: false };
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
    let s = Shape { id: 1, layer: 0, kind: Kind::Text(t), xf: Xf::IDENTITY, group: None, locked: false };
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
    assert_eq!(o.len(), crate::LAYER_COUNT);
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
    v["sections"]["laser"].as_object_mut().unwrap().remove("order");
    v["scene"].as_object_mut().unwrap().remove("next_group");
    assert_eq!(Document::from_json(&v.to_string()).unwrap().layer_order().len(), crate::LAYER_COUNT);
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
    assert_eq!(rect.layer, nearest_palette([255, 0, 0])); // red
    let b = rect.bounds().unwrap();
    // 100 pt wide = 35.28 mm; y is flipped: the top of the page is y = 0 (page is 100 pt = 35.28 mm tall).
    assert!((b.width() - 100.0 * 25.4 / 72.0).abs() < 1e-6);
    assert!((b.min.y - (100.0 - 60.0) * 25.4 / 72.0).abs() < 1e-6);
    let curve = d.shape(ids[1]).unwrap();
    assert_eq!(curve.layer, nearest_palette([0, 0, 255])); // blue
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

#[test]
fn settings_reports_are_parsed() {
    let grbl: Vec<String> = ["$0=10", "$30=255", "$32=1", "$110=6000.000", "$111=6000.000", "$130=410.000", "$131=300.000", "ok"].iter().map(|s| s.to_string()).collect();
    let r = Controller::Grbl.parse_settings(&grbl);
    assert_eq!((r.s_max, r.bed_w, r.bed_h, r.travel_speed, r.dynamic_power), (Some(255.0), Some(410.0), Some(300.0), Some(6000.0), Some(true)));
    assert_eq!(r.count(), 5);
    let marlin: Vec<String> = ["echo:  M203 X500.00 Y400.00 Z5.00", "ok"].iter().map(|s| s.to_string()).collect();
    assert_eq!(Controller::Marlin.parse_settings(&marlin).travel_speed, Some(30000.0));
    assert_eq!(Controller::Ruida.settings_request(), None);
}

#[test]
fn automatic_shapes() {
    let tri = Contour::triangle(10.0, 8.0);
    assert_eq!(tri.nodes.len(), 3);
    assert!((tri.flatten(0.1).area().abs() - 40.0).abs() < 1e-9);
    let hex = Contour::polygon(6, 20.0, 20.0);
    assert_eq!(hex.nodes.len(), 6);
    // Regular hexagon with circumradius 10: area = 3*sqrt(3)/2 * r^2.
    assert!((hex.flatten(0.1).area().abs() - 3.0 * 3f64.sqrt() / 2.0 * 100.0).abs() < 1e-6);
    assert_eq!(Contour::polygon(1000, 10.0, 10.0).nodes.len(), 360);
    assert_eq!(Contour::polygon(1, 10.0, 10.0).nodes.len(), 3);
    let star = Contour::star(5, 0.382, 20.0, 20.0);
    assert_eq!(star.nodes.len(), 10);
    // Top tip touches the top edge of the box.
    assert!((star.nodes[0].p.y - 0.0).abs() < 1e-9);
}

#[test]
fn jobs_start_and_end_at_the_machine_origin() {
    for (origin, home) in [(Origin::FrontLeft, Pt::new(0.0, 400.0)), (Origin::BackLeft, Pt::new(0.0, 0.0))] {
        let mut d = Document::default();
        d.device.axes.origin = origin;
        d.device.gcode.return_home = true;
        d.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::translate(50.0, 50.0));
        assert_eq!(d.device.home_point(), home);
        let job = gcode::generate(&d);
        assert_eq!(job.moves[0].a, home, "starts at the machine zero");
        assert_eq!(job.moves.last().unwrap().b, home, "returns to the machine zero");
        // Machine coordinates of the home move are X0 Y0.
        assert!(job.gcode.contains("G0 X0.000 Y0.000"), "{}", job.gcode);
        assert!(job.moves.iter().all(|m| m.dur >= 0.0));
        let total: f64 = job.moves.iter().map(|m| m.dur).sum();
        assert!((total - job.est_seconds).abs() < 1e-6);
    }
}

#[test]
fn old_devices_get_link_defaults() {
    let old = r#"{"name":"x","bed_w":300.0,"bed_h":200.0,"origin":"FrontLeft","s_max":1000.0,"dynamic_power":true,"travel_speed":3000.0,"return_home":true,"baud":115200}"#;
    let d: Device = serde_json::from_str(old).unwrap();
    assert_eq!((d.connection.link, d.connection.tcp_port, d.cameras.is_empty()), (LinkKind::Serial, 3333, true));
    assert!(!d.has_target());
}

#[test]
fn ser2net_configs() {
    use ser2net::{config, shell_safe, version_from_output, Settings, Version};
    let s = Settings { serial_port: "/dev/ttyUSB0".into(), baud: 115200, tcp_port: 3333, bind: None, kick_old_user: true };
    let y = config(Version::V4, &s);
    assert!(y.contains("accepter: tcp,3333\n") && y.contains("connector: serialdev,/dev/ttyUSB0,115200n81,local") && y.contains("kickolduser: true"));
    let local = Settings { bind: Some("127.0.0.1".into()), kick_old_user: false, ..s.clone() };
    assert!(config(Version::V4, &local).contains("accepter: tcp,127.0.0.1,3333"));
    let old = config(Version::V3, &s);
    assert!(old.contains("3333:raw:0:/dev/ttyUSB0:115200 8DATABITS NONE 1STOPBIT kickolduser"));
    assert!(config(Version::V3, &local).contains("127.0.0.1,3333:raw:0:"));
    assert!(shell_safe("/dev/ttyUSB0") && !shell_safe("a'b"));
    assert_eq!(version_from_output("ser2net version 4.6.1"), Some(Version::V4));
    assert_eq!(version_from_output("ser2net version 3.5.1"), Some(Version::V3));
    assert_eq!(version_from_output("nothing"), None);
    assert_eq!(Version::V4.file(), "/etc/ser2net.yaml");
}

#[test]
fn camera_rotation_is_stored_with_the_device() {
    let mut d = Device::default();
    d.cameras = vec![CameraSource::url("A", "http://a/x", 3)];
    let back: Device = serde_json::from_str(&serde_json::to_string(&d).unwrap()).unwrap();
    assert_eq!(back.cameras[0].rotation, 3);
    assert_eq!(back, d);
    let old = r#"{"name":"x","bed_w":300.0,"bed_h":200.0,"origin":"FrontLeft","s_max":1000.0,"dynamic_power":true,"travel_speed":3000.0,"return_home":true,"baud":115200}"#;
    assert_eq!(serde_json::from_str::<Device>(old).unwrap().cameras.len(), 0);
}

#[test]
fn crossing_selection_touches_partially_overlapped_shapes() {
    let area = Rect { min: Pt::new(0.0, 0.0), max: Pt::new(10.0, 10.0) };
    let rect_at = |x: f64, y: f64, s: f64| Shape { id: 1, layer: 0, kind: Kind::Rect { w: s, h: s }, xf: Xf::translate(x, y), group: None, locked: false };
    assert!(rect_at(5.0, 5.0, 20.0).intersects_rect(&area), "partial overlap");
    assert!(rect_at(2.0, 2.0, 3.0).intersects_rect(&area), "fully inside");
    assert!(rect_at(-50.0, -50.0, 200.0).intersects_rect(&area), "area fully inside a big shape");
    assert!(!rect_at(11.0, 11.0, 5.0).intersects_rect(&area), "outside");
    // A diagonal line whose bounding box overlaps the area but which misses it.
    let line = Shape { id: 2, layer: 0, kind: Kind::Path(vec![Polyline::new(vec![Pt::new(-1.0, 8.0), Pt::new(8.0, 20.0)], false)]), xf: Xf::IDENTITY, group: None, locked: false };
    assert!(!line.intersects_rect(&Rect { min: Pt::new(0.0, 0.0), max: Pt::new(3.0, 3.0) }));
    assert!(line.intersects_rect(&area));
    // A line crossing straight through the area with no vertex inside it.
    let through = Shape { id: 3, layer: 0, kind: Kind::Path(vec![Polyline::new(vec![Pt::new(-5.0, 5.0), Pt::new(15.0, 5.0)], false)]), xf: Xf::IDENTITY, group: None, locked: false };
    assert!(through.intersects_rect(&area));
}

#[test]
fn locked_shapes_refuse_editing_access() {
    let mut d = Document::default();
    let a = d.add(0, Kind::Rect { w: 1.0, h: 1.0 }, Xf::IDENTITY);
    let b = d.add(0, Kind::Rect { w: 1.0, h: 1.0 }, Xf::IDENTITY);
    d.shape_mut(a).unwrap().locked = true;
    assert!(d.unlocked_mut(a).is_none() && d.unlocked_mut(b).is_some());
    assert_eq!(d.unlocked_ids(&[a, b, 999]), vec![b]);
    // Old files without the flag load as unlocked.
    let mut v: serde_json::Value = serde_json::from_str(&d.to_json()).unwrap();
    for s in v["scene"]["shapes"].as_array_mut().unwrap() {
        s.as_object_mut().unwrap().remove("locked");
    }
    assert!(Document::from_json(&v.to_string()).unwrap().scene.shapes.iter().all(|s| !s.locked));
}

#[test]
fn new_layers_default_to_full_power_and_1000_mm_per_minute() {
    let l = Layer::new(4);
    assert_eq!((l.power, l.speed), (100.0, 1000.0 / 60.0));
    assert!(Document::default().layers.iter().all(|l| l.power == 100.0 && (l.speed - 1000.0 / 60.0).abs() < 1e-9));
}

#[test]
fn layer_default_speeds_live_in_the_device_profile() {
    let mut dev = Device::default();
    assert_eq!(dev.layer_speeds.len(), 30);
    assert!((dev.layer_speed_mm_s(3) - 1000.0 / 60.0).abs() < 1e-9);
    dev.set_layer_speed_mm_s(3, 20.0);
    assert_eq!(dev.layer_speeds[3], 1200.0);
    // The speeds survive a save and load of the profile, and older profiles without them get the default.
    let back: Device = serde_json::from_str(&serde_json::to_string(&dev).unwrap()).unwrap();
    assert_eq!(back.layer_speeds[3], 1200.0);
    let old = r#"{"name":"x","bed_w":300.0,"bed_h":200.0,"origin":"FrontLeft","s_max":1000.0,"dynamic_power":true,"travel_speed":3000.0,"return_home":true,"baud":115200}"#;
    let d: Device = serde_json::from_str(old).unwrap();
    assert_eq!(d.layer_speeds, vec![1000.0; 30]);
}

#[test]
fn rounding_corners_of_a_rectangle() {
    let c = Contour::rect(10.0, 10.0);
    let (r, n) = fillet::round_contour(&c, 2.0, None);
    assert_eq!(n, 4);
    // Each corner loses (4 - pi) r^2 / 1 of area: area = 100 - 4 * (1 - pi/4) * r^2.
    let expect = 100.0 - 4.0 * (1.0 - std::f64::consts::FRAC_PI_4) * 4.0;
    let area = r.flatten(0.005).area().abs();
    assert!((area - expect).abs() < 0.02, "{area} vs {expect}");
    // A radius that is too large is limited to half a side: the result is a stadium-like shape, not garbage.
    let (big, n) = fillet::round_contour(&c, 50.0, None);
    assert_eq!(n, 4);
    let b = Polyline::bounds(&big.flatten(0.01)).unwrap();
    assert!((b.width() - 10.0).abs() < 1e-6 && (b.height() - 10.0).abs() < 1e-6);
    assert!(big.flatten(0.01).area().abs() < 100.0 && big.flatten(0.01).area().abs() > 70.0);
    // Only the chosen corner.
    let one: std::collections::HashSet<usize> = [0usize].into_iter().collect();
    let (_, n) = fillet::round_contour(&c, 2.0, Some(&one));
    assert_eq!(n, 1);
    // Nothing to do for a smooth ellipse.
    assert_eq!(fillet::round_contour(&Contour::ellipse(10.0, 6.0), 1.0, None).1, 0);
}

#[test]
fn filleting_two_lines() {
    // A horizontal and a vertical line that cross at (10, 10).
    let h = (Pt::new(0.0, 10.0), Pt::new(14.0, 10.0));
    let v = (Pt::new(10.0, 0.0), Pt::new(10.0, 25.0));
    let c = fillet::fillet_lines(h, v, 3.0).expect("fillet");
    // Kept: the longer parts (0,10) .. and (10,25) .. so the corner is at (10,10) turning towards +y.
    assert_eq!(c.nodes.first().unwrap().p, Pt::new(0.0, 10.0));
    assert_eq!(c.nodes.last().unwrap().p, Pt::new(10.0, 25.0));
    let tangents: Vec<Pt> = c.nodes.iter().map(|n| n.p).collect();
    assert!(tangents.contains(&Pt::new(7.0, 10.0)), "{tangents:?}");
    assert!(tangents.iter().any(|p| (p.x - 10.0).abs() < 1e-9 && (p.y - 13.0).abs() < 1e-9));
    // The arc's middle lies at distance r from its centre (7, 13) and bulges towards the corner.
    let mid = c.point_at(1, 0.5);
    assert!((mid.dist(Pt::new(7.0, 13.0)) - 3.0).abs() < 0.01, "{mid:?}");
    assert!(mid.x > 7.0 && mid.y < 13.0);
    // Parallel lines cannot be joined.
    assert!(fillet::fillet_lines((Pt::new(0.0, 0.0), Pt::new(5.0, 0.0)), (Pt::new(0.0, 3.0), Pt::new(5.0, 3.0)), 1.0).is_none());
    assert!(fillet::as_segment(&[Contour { nodes: vec![Node::corner(Pt::new(0.0, 0.0)), Node::corner(Pt::new(1.0, 1.0))], closed: false }]).is_some());
    assert!(fillet::as_segment(&[Contour::rect(1.0, 1.0)]).is_none());
}

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("lc-test-{tag}-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

const CAT_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M2,2L22,2L22,12L2,12Z M5,15L19,15L19,22L5,22Z"/></svg>"##;

#[test]
fn user_clipart_library_roundtrip() {
    let dir = temp_dir("lib");
    let mut lib = clipart::Library::open(&dir);
    assert!(lib.items.is_empty());
    let a = lib.add(CAT_SVG, "Big Cat!", "Animals", "https://example.org/cat.svg", "CC0", "Someone").unwrap();
    let b = lib.add(&CAT_SVG.replace("<path", "<path fill=\"currentColor\""), "Big Cat!", "", "file", "", "").unwrap();
    assert_eq!(a.id, "big-cat");
    assert_eq!(b.id, "big-cat-2", "same name gets a different file");
    assert_eq!(b.category, clipart::DEFAULT_CATEGORY);
    assert!(lib.svg("big-cat-2").unwrap().contains("#000000"), "currentColor is made explicit");
    lib.set_category("big-cat", "Pets").unwrap();
    lib.rename("big-cat", "Kitty").unwrap();
    // Reopen: everything is remembered.
    let lib2 = clipart::Library::open(&dir);
    assert_eq!(lib2.items.len(), 2);
    let k = lib2.items.iter().find(|c| c.id == "big-cat").unwrap();
    assert_eq!((k.name.as_str(), k.category.as_str(), k.license.as_str()), ("Kitty", "Pets", "CC0"));
    assert_eq!(lib2.categories(), vec!["My clipart".to_string(), "Pets".to_string()]);
    // Removing deletes the file and the entry; an index entry whose file vanished is dropped on open.
    let mut lib3 = clipart::Library::open(&dir);
    lib3.remove("big-cat").unwrap();
    assert!(!dir.join("big-cat.svg").exists());
    std::fs::remove_file(dir.join("big-cat-2.svg")).unwrap();
    assert!(clipart::Library::open(&dir).items.is_empty());
    // Bad input is refused.
    assert!(lib3.add("<html></html>", "x", "", "", "", "").is_err());
    assert!(lib3.add("<svg", "x", "", "", "", "").is_err());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn clipart_import_scales_centres_and_groups_on_one_layer() {
    let mut d = Document::default();
    let ids = clipart::import_clip(CAT_SVG.as_bytes(), &mut d, 7, 50.0, Pt::new(100.0, 80.0)).unwrap();
    assert_eq!(ids.len(), 1, "one path element");
    let b = d.bounds_of(&ids).unwrap();
    assert!((b.width().max(b.height()) - 50.0).abs() < 1e-6, "{b:?}");
    assert!(b.center().dist(Pt::new(100.0, 80.0)) < 1e-6);
    assert!(d.scene.shapes.iter().all(|s| s.layer == 7), "colours do not spread the picture over layers");
    // Two elements become a group.
    let two = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path fill="red" d="M2,2L10,2L10,10Z"/><path fill="blue" d="M14,14L22,14L22,22Z"/></svg>"##;
    let ids = clipart::import_clip(two.as_bytes(), &mut d, 3, 30.0, Pt::new(10.0, 10.0)).unwrap();
    assert_eq!(ids.len(), 2);
    assert_eq!(d.group_of(ids[0]).len(), 2);
    assert!(ids.iter().all(|i| d.shape(*i).unwrap().layer == 3));
}

#[test]
fn iconify_search_answers_are_read() {
    let json = r#"{"icons":["mdi:cat","tabler:cat","unknown:thing"],"total":3,"limit":3,"start":0,"collections":{
        "mdi":{"name":"Material Design Icons","author":{"name":"Pictogrammers","url":"https://x"},"license":{"title":"Apache 2.0","spdx":"Apache-2.0","url":"https://l"}},
        "tabler":{"name":"Tabler Icons","author":{"name":"Paweł Kuna"},"license":{"title":"MIT","spdx":"MIT"}}}}"#;
    let hits = clipart::iconify::parse_search(json).unwrap();
    assert_eq!(hits.len(), 3);
    assert_eq!(hits[0].key(), "mdi:cat");
    assert_eq!((hits[0].set_name.as_str(), hits[0].license_spdx.as_str(), hits[0].author.as_str()), ("Material Design Icons", "Apache-2.0", "Pictogrammers"));
    assert!(hits[0].needs_no_credit() && hits[1].needs_no_credit());
    assert!(hits[2].license_title.is_empty() && !hits[2].needs_no_credit());
    assert_eq!(hits[0].svg_url("https://api.iconify.design"), "https://api.iconify.design/mdi/cat.svg");
    assert_eq!(clipart::iconify::search_url("https://a", "paper plane", 20), "https://a/search?query=paper%20plane&limit=20");
    assert!(clipart::iconify::parse_search("nonsense").is_err());
    let by = clipart::iconify::Hit { prefix: "x".into(), name: "y".into(), set_name: String::new(), license_title: "CC BY 4.0".into(), license_spdx: "CC-BY-4.0".into(), license_url: String::new(), author: String::new() };
    assert!(!by.needs_no_credit(), "attribution licences are flagged");
}

#[test]
fn heart_shape_fills_its_box() {
    let c = Contour::heart(40.0, 36.0);
    assert!(c.closed && c.nodes.len() == 6);
    let b = c.flatten(0.01).bounds().unwrap();
    assert!(b.min.x.abs() < 0.2 && (b.max.x - 40.0).abs() < 0.2, "{b:?}");
    assert!(b.min.y.abs() < 0.2 && (b.max.y - 36.0).abs() < 0.2, "{b:?}");
    // Symmetric about the vertical middle line, with its dip above the tip.
    let area = c.flatten(0.01).area().abs();
    assert!(area > 0.5 * 40.0 * 36.0 && area < 0.9 * 40.0 * 36.0, "{area}");
    assert!(c.nodes[0].p.y < c.nodes[3].p.y);
    let cx: f64 = c.flatten(0.01).pts.iter().map(|p| p.x).sum::<f64>() / c.flatten(0.01).pts.len() as f64;
    assert!((cx - 20.0).abs() < 1.0, "centroid x {cx}");
}

#[test]
fn speed_units_convert_between_display_and_stored_values() {
    use crate::{SpeedUnit::*, Units::*};
    // Layers are stored in mm/s, travel and jog in mm/min.
    assert_eq!(PerSecond.factor(Mm, false), 1.0);
    assert_eq!(PerMinute.factor(Mm, false), 1.0 / 60.0);
    assert_eq!(PerSecond.factor(Mm, true), 60.0);
    assert_eq!(PerMinute.factor(Mm, true), 1.0);
    // 10 in/s is 254 mm/s; 60 in/min is 1524 mm/min; 1 mm/s shown per minute is 60.
    assert!((254.0 / PerSecond.factor(Inch, false) - 10.0).abs() < 1e-9);
    assert!((1524.0 / PerMinute.factor(Inch, true) - 60.0).abs() < 1e-9);
    assert!((1.0 / PerMinute.factor(Mm, false) - 60.0).abs() < 1e-9);
    // The G-code feed is mm/min whatever the display: a 20 mm/s layer burns at F1200.
    let mut doc = Document::default();
    doc.device.speed_unit = PerMinute;
    doc.device.units = Inch;
    doc.layers[0].speed = 20.0;
    let id = doc.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::IDENTITY);
    let _ = id;
    assert!(crate::gcode::generate(&doc).gcode.contains("F1200"));
}

#[test]
fn air_assist_is_switched_per_layer_and_off_at_the_end() {
    let mut doc = Document::default();
    doc.layers[0].air_assist = true;
    doc.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::IDENTITY);
    doc.add(1, Kind::Rect { w: 10.0, h: 10.0 }, Xf::translate(20.0, 0.0));
    let g = crate::gcode::generate(&doc).gcode;
    let (on, off) = (g.find("M8 ;").expect("air on"), g.rfind("M9 ;").expect("air off"));
    assert!(on < off);
    // The second layer has no air assist: it is switched off before it burns.
    assert_eq!(g.matches("M8 ;").count(), 1);
    assert!(g.matches("M9 ;").count() >= 1);
    doc.device.controller = Controller::Marlin;
    let m = crate::gcode::generate(&doc).gcode;
    assert!(m.contains("M106 S255") && m.contains("M107"));
    // Without air assist nothing is emitted.
    doc.layers[0].air_assist = false;
    let n = crate::gcode::generate(&doc).gcode;
    assert!(!n.contains("M8") && !n.contains("M9") && !n.contains("M106"));
}

#[test]
fn skew_keeps_the_anchor_line_and_shifts_points_in_proportion() {
    let o = Pt::new(10.0, 20.0);
    let sx = Xf::skew_about(0.5, 0.0, o);
    // Points on the anchor line y = 20 stay; a point 10 above moves left by 5.
    let p = sx.apply(Pt::new(30.0, 20.0));
    assert!((p.x - 30.0).abs() < 1e-9 && (p.y - 20.0).abs() < 1e-9);
    let q = sx.apply(Pt::new(30.0, 10.0));
    assert!((q.x - 25.0).abs() < 1e-9 && (q.y - 10.0).abs() < 1e-9);
    let sy = Xf::skew_about(0.0, 0.25, o);
    let r = sy.apply(Pt::new(14.0, 50.0));
    assert!((r.x - 14.0).abs() < 1e-9 && (r.y - 51.0).abs() < 1e-9);
}

#[test]
fn spiral_starts_in_the_middle_and_fills_the_box() {
    let c = Contour::spiral(3, 60.0, 40.0);
    assert!(!c.closed);
    let first = c.nodes[0].p;
    assert!((first.x - 30.0).abs() < 1e-9 && (first.y - 20.0).abs() < 1e-9);
    let last = c.nodes.last().unwrap().p;
    // Ends at the outer edge after whole turns: straight above the middle (the start angle is the top).
    assert!((last.x - 30.0).abs() < 1e-6 && last.y.abs() < 1e-6, "{last:?}");
    let flat = c.flatten(0.05);
    let (min_x, max_x) = flat.pts.iter().fold((f64::MAX, f64::MIN), |a, p| (a.0.min(p.x), a.1.max(p.x)));
    assert!(min_x >= -0.1 && max_x <= 60.1 && max_x - min_x > 45.0, "{min_x} {max_x}");
    // More turns, more nodes.
    assert!(Contour::spiral(6, 60.0, 40.0).nodes.len() > c.nodes.len());
}

/// Every character of the European translations must exist in the interface font, or the program would show
/// empty boxes in that language.
#[test]
fn the_interface_font_covers_every_european_translation() {
    let font = include_bytes!("../../../assets/fonts/NotoSans-Regular.ttf");
    let face = ttf_parser::Face::parse(font, 0).expect("font parses");
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/lang");
    let mut missing = std::collections::BTreeMap::<String, String>::new();
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "txt") {
            let name = path.file_stem().unwrap().to_string_lossy().into_owned();
            let text = std::fs::read_to_string(&path).unwrap();
            for line in text.lines() {
                let body = line.split_once(|c| c == '|' || c == '\t').map(|(_, t)| t).unwrap_or(line);
                for c in body.chars() {
                    if !c.is_control() && face.glyph_index(c).is_none() && !missing.entry(name.clone()).or_default().contains(c) {
                        missing.get_mut(&name).unwrap().push(c);
                    }
                }
            }
        }
    }
    assert!(missing.is_empty(), "characters without a glyph: {missing:?}");
}

/// A profile exactly as v0.0.11 wrote it: every setting is a flat field.
const OLD_PROFILE: &str = r#"{"name":"Old CO2","units":"Inch","speed_unit":"PerMinute","controller":"Marlin","laser":"Co2","bed_w":300.0,"bed_h":200.0,"origin":"BackLeft","s_max":255.0,"dynamic_power":false,"travel_speed":4500.0,"return_home":false,"baud":250000,"port":"/dev/ttyUSB3","jog_step":2.5,"jog_feed":1800.0,"frame_power":1.5,"camera":null,"link":"Tcp","host":"pi.local","tcp_port":4000,"camera_url":"http://pi.local:8080/stream","camera_rotation":2,"layer_speeds":[1200.0]}"#;

#[test]
fn old_flat_profile_is_read_into_the_groups() {
    let d: Device = serde_json::from_str(OLD_PROFILE).unwrap();
    assert_eq!((d.area.bed_w, d.area.bed_h, d.axes.origin), (300.0, 200.0, Origin::BackLeft));
    assert_eq!((d.speeds.travel_speed, d.gcode.dynamic_power, d.gcode.return_home), (4500.0, false, false));
    assert_eq!((d.connection.baud, d.connection.port.as_str(), d.connection.link, d.connection.host.as_str(), d.connection.tcp_port), (250000, "/dev/ttyUSB3", LinkKind::Tcp, "pi.local", 4000));
    assert_eq!(d.heads.len(), 1);
    assert_eq!((d.heads[0].s_max, d.heads[0].frame_power, d.heads[0].laser), (255.0, 1.5, LaserKind::Co2));
    assert_eq!((d.controller, d.units, d.jog_step, d.cameras[0].rotation), (Controller::Marlin, Units::Inch, 2.5, 2));
}

#[test]
fn new_profile_round_trips_and_a_minimal_old_one_gets_defaults() {
    let d: Device = serde_json::from_str(OLD_PROFILE).unwrap();
    let json = serde_json::to_string(&d).unwrap();
    assert!(json.contains("\"heads\"") && json.contains("\"area\""));
    let back: Device = serde_json::from_str(&json).unwrap();
    assert_eq!(back, d);
    let tiny: Device = serde_json::from_str(r#"{"name":"x","bed_w":100.0,"bed_h":80.0,"origin":"FrontLeft","s_max":1000.0,"dynamic_power":true,"travel_speed":3000.0,"return_home":true,"baud":115200}"#).unwrap();
    assert_eq!((tiny.area.bed_w, tiny.heads.len(), tiny.connection.tcp_port, tiny.layer_speeds.len()), (100.0, 1, 3333, 30));
}

#[test]
fn a_profile_without_heads_gets_one() {
    let mut v: serde_json::Value = serde_json::to_value(Device::default()).unwrap();
    v["heads"] = serde_json::json!([]);
    let d: Device = serde_json::from_value(v).unwrap();
    assert_eq!(d.heads.len(), 1);
}

fn axes_device(origin: Origin, flip_x: bool, flip_y: bool, orientation: Orientation) -> Device {
    let mut d = Device::default();
    d.area = Area { bed_w: 300.0, bed_h: 200.0, ..Area::default() };
    d.axes = Axes { origin, flip_x, flip_y, orientation, ..Axes::default() };
    d
}

#[test]
fn machine_transform_round_trips_for_every_corner_flip_and_orientation() {
    let pts = [Pt::new(0.0, 0.0), Pt::new(12.5, 7.25), Pt::new(100.0, 150.0), Pt::new(199.0, 1.0)];
    for origin in Origin::ALL {
        for (fx, fy) in [(false, false), (true, false), (false, true), (true, true)] {
            for o in Orientation::ALL {
                let d = axes_device(origin, fx, fy, o);
                let (w, h) = d.work_size();
                for p in pts {
                    let p = Pt::new(p.x.min(w), p.y.min(h));
                    let (x, y) = d.to_machine(p);
                    assert!(x >= -1e-9 && x <= 300.0 + 1e-9 && y >= -1e-9 && y <= 200.0 + 1e-9, "{origin:?} {fx} {fy} {o:?}: {x},{y}");
                    assert!(d.from_machine(x, y).dist(p) < 1e-9, "{origin:?} {fx} {fy} {o:?}");
                }
            }
        }
    }
}

#[test]
fn zero_is_at_the_chosen_corner_of_the_design() {
    // Design y points down: "front" is the bottom edge.
    let at = |origin| axes_device(origin, false, false, Orientation::Native).home_point();
    assert_eq!(at(Origin::FrontLeft), Pt::new(0.0, 200.0));
    assert_eq!(at(Origin::BackLeft), Pt::new(0.0, 0.0));
    assert_eq!(at(Origin::FrontRight), Pt::new(300.0, 200.0));
    assert_eq!(at(Origin::BackRight), Pt::new(300.0, 0.0));
    // Inverting X moves the zero to the other side.
    assert_eq!(axes_device(Origin::FrontLeft, true, false, Orientation::Native).home_point(), Pt::new(300.0, 200.0));
}

#[test]
fn turned_panel_swaps_the_design_size_and_maps_the_corners() {
    let left = axes_device(Origin::FrontLeft, false, false, Orientation::RotateLeft);
    assert_eq!(left.work_size(), (200.0, 300.0));
    // A quarter turn to the right takes the machine's top-left corner to the design's top-right corner.
    let right = axes_device(Origin::BackLeft, false, false, Orientation::RotateRight);
    assert_eq!(right.work_size(), (200.0, 300.0));
    assert_eq!(right.to_machine(Pt::new(200.0, 0.0)), (0.0, 0.0));
    assert_eq!(right.home_point(), Pt::new(200.0, 0.0));
    assert_eq!(left.to_machine(Pt::new(0.0, 0.0)), (300.0, 200.0));
}

#[test]
fn margins_and_soft_limits_cut_down_the_allowed_area() {
    let mut d = axes_device(Origin::FrontLeft, false, false, Orientation::Native);
    assert_eq!((d.usable_rect().min, d.usable_rect().max), (Pt::new(0.0, 0.0), Pt::new(300.0, 200.0)));
    d.area.margins = Margins { left: 10.0, top: 5.0, right: 20.0, bottom: 15.0 };
    let u = d.usable_rect();
    assert_eq!((u.min, u.max), (Pt::new(10.0, 5.0), Pt::new(280.0, 185.0)));
    // Soft limits are kept inside the usable area, whatever order the corners are given in.
    d.area.soft_limits = Some(SoftLimits { x0: 250.0, y0: 100.0, x1: 50.0, y1: 400.0 });
    let a = d.allowed_rect();
    assert_eq!((a.min, a.max), (Pt::new(50.0, 100.0), Pt::new(250.0, 185.0)));
    // Margins larger than the area leave an empty (not inverted) rectangle.
    d.area.margins = Margins { left: 400.0, top: 0.0, right: 400.0, bottom: 0.0 };
    assert!(d.usable_rect().max.x >= d.usable_rect().min.x);
}

#[test]
fn jog_stops_at_the_allowed_area_and_follows_the_axes() {
    let mut d = axes_device(Origin::FrontLeft, false, false, Orientation::Native);
    d.area.margins = Margins { left: 10.0, top: 10.0, right: 10.0, bottom: 10.0 };
    // Machine (20, 20) is design (20, 180). Up (design y -5) is machine Y +5, as GRBL expects.
    assert_eq!(d.limited_jog(Some((20.0, 20.0)), 0.0, -5.0), (0.0, 5.0));
    // 5 mm from the left margin a 20 mm step to the left is shortened to the 10 mm that are left.
    assert_eq!(d.limited_jog(Some((20.0, 20.0)), -20.0, 0.0), (-10.0, 0.0));
    // At the bottom margin (machine Y 10) going further down is not allowed at all.
    assert_eq!(d.limited_jog(Some((100.0, 10.0)), 0.0, 5.0), (0.0, 0.0));
    // Unknown position: no limiting.
    assert_eq!(d.limited_jog(None, 0.0, 5.0), (0.0, -5.0));
    assert_eq!(d.limited_jog(Some((f64::NAN, f64::NAN)), 7.0, 0.0), (7.0, 0.0));
    // A head outside the area may move back in, but not further out.
    assert_eq!(d.limited_jog(Some((2.0, 100.0)), 5.0, 0.0), (5.0, 0.0));
    assert_eq!(d.limited_jog(Some((2.0, 100.0)), -5.0, 0.0), (0.0, 0.0));
}

fn check_doc(x: f64, y: f64) -> Document {
    let mut d = Document::default();
    d.device = axes_device(Origin::FrontLeft, false, false, Orientation::Native);
    d.layers[0].mode = LayerMode::Line;
    d.add(0, Kind::Rect { w: 20.0, h: 10.0 }, Xf::translate(x, y));
    d
}

fn kinds(d: &Document) -> Vec<check::IssueKind> {
    check::job_check(d).into_iter().map(|i| i.kind).collect()
}

#[test]
fn job_inside_the_area_has_no_issues_even_with_the_zero_in_a_margin() {
    let mut d = check_doc(100.0, 100.0);
    assert!(check::job_check(&d).is_empty());
    // The way back to the zero (a corner of the machine) ends inside the margin: that is not an error.
    d.device.area.margins = Margins { left: 5.0, top: 5.0, right: 5.0, bottom: 5.0 };
    assert!(check::job_check(&d).is_empty(), "{:?}", check::job_check(&d));
}

#[test]
fn objects_outside_the_area_the_margins_or_the_limits_are_errors_and_name_the_objects() {
    let d = check_doc(295.0, 100.0);
    let issues = check::job_check(&d);
    assert_eq!(issues.len(), 1);
    assert_eq!((issues[0].level, issues[0].kind), (check::Level::Error, check::IssueKind::OutsideArea));
    assert_eq!(issues[0].shapes, vec![d.scene.shapes[0].id]);
    // Inside the work area but in a margin.
    let mut m = check_doc(2.0, 100.0);
    m.device.area.margins.left = 5.0;
    assert_eq!(kinds(&m), vec![check::IssueKind::OutsideArea]);
    // Inside the usable area but outside the software limits.
    let mut l = check_doc(100.0, 100.0);
    l.device.area.soft_limits = Some(SoftLimits { x0: 0.0, y0: 0.0, x1: 110.0, y1: 200.0 });
    assert_eq!(kinds(&l), vec![check::IssueKind::OutsideLimits]);
    // Within the tolerance (0.01 mm) is fine.
    let t = check_doc(300.0 - 20.0 + 0.005, 100.0);
    assert!(check::job_check(&t).is_empty());
    // Objects on layers that do not output are not checked.
    let mut off = check_doc(295.0, 100.0);
    off.layers[0].output = false;
    assert!(check::job_check(&off).is_empty());
}

#[test]
fn overscan_that_leaves_the_area_is_reported_for_the_layer() {
    let mut d = check_doc(0.0, 100.0);
    d.layers[0].mode = LayerMode::Fill;
    d.layers[0].interval = 1.0;
    d.layers[0].overscan = 3.0;
    let issues = check::job_check(&d);
    assert!(issues.iter().any(|i| i.kind == check::IssueKind::MoveOutside && i.layer == Some(0)), "{issues:?}");
    d.layers[0].overscan = 0.0;
    assert!(check::job_check(&d).is_empty());
}

#[test]
fn zero_power_and_empty_toolpaths_are_warnings() {
    let mut d = check_doc(100.0, 100.0);
    d.layers[0].power = 0.0;
    let issues = check::job_check(&d);
    assert_eq!(issues.iter().map(|i| (i.level, i.kind)).collect::<Vec<_>>(), vec![(check::Level::Warning, check::IssueKind::ZeroPower)]);
    assert!(!check::has_error(&issues));
    // A fill layer that holds only an open line burns nothing.
    let mut f = Document::default();
    f.device = axes_device(Origin::FrontLeft, false, false, Orientation::Native);
    f.layers[0].mode = LayerMode::Fill;
    f.add(0, Kind::Path(vec![Polyline::new(vec![Pt::new(0.0, 0.0), Pt::new(10.0, 0.0)], false)]), Xf::translate(50.0, 50.0));
    assert_eq!(kinds(&f), vec![check::IssueKind::NothingToBurn]);
}

#[test]
fn frame_outside_the_area_is_an_error() {
    let dev = axes_device(Origin::FrontLeft, false, false, Orientation::Native);
    let r = |x0, y0, x1, y1| Rect { min: Pt::new(x0, y0), max: Pt::new(x1, y1) };
    assert!(check::check_frame(&dev, r(10.0, 10.0, 50.0, 50.0)).is_empty());
    let e = check::check_frame(&dev, r(10.0, 10.0, 350.0, 50.0));
    assert_eq!((e.len(), e[0].kind, e[0].level), (1, check::IssueKind::OutsideArea, check::Level::Error));
}

#[test]
fn controller_report_is_compared_with_the_profile() {
    let lines: Vec<String> = ["$30=255", "$32=0", "$110=3000", "ok"].iter().map(|s| s.to_string()).collect();
    let r = Controller::Grbl.parse_settings(&lines);
    assert_eq!((r.s_max, r.laser_mode), (Some(255.0), Some(false)));
    let dev = Device::default(); // S max 1000
    let c = r.check(&dev);
    assert!(c.laser_mode_off);
    assert_eq!(c.s_max, Some(255.0));
    // Laser mode on and the same S max: nothing to say.
    let ok: Vec<String> = ["$30=1000", "$32=1"].iter().map(|s| s.to_string()).collect();
    assert!(Controller::Grbl.parse_settings(&ok).check(&dev).is_clean());
    // A controller that did not report the values raises nothing.
    assert!(Controller::Grbl.parse_settings(&[]).check(&dev).is_clean());
}

fn long_line_doc(speed: f64) -> Document {
    let mut d = Document::default();
    d.device.area = Area { bed_w: 300.0, bed_h: 200.0, ..Area::default() };
    d.device.axes.origin = Origin::BackLeft;
    d.device.gcode.return_home = false;
    d.layers[0].mode = LayerMode::Line;
    d.layers[0].speed = speed;
    d.layers[0].power = 50.0;
    // One open 100 mm line from the machine's zero: no travel move before it.
    d.add(0, Kind::Path(vec![Polyline::new(vec![Pt::new(0.0, 0.0), Pt::new(100.0, 0.0)], false)]), Xf::IDENTITY);
    d
}

#[test]
fn estimate_without_limits_is_distance_over_speed() {
    let job = gcode::generate(&long_line_doc(10.0));
    assert!((job.est_seconds - 10.0).abs() < 1e-6, "{}", job.est_seconds);
}

#[test]
fn estimate_with_acceleration_adds_the_ramps_and_caps_the_speed() {
    let mut d = long_line_doc(10.0);
    d.device.speeds.acceleration = 10.0;
    // 100 mm at 10 mm/s with 10 mm/s²: 10 s cruise + 1 s of ramps.
    assert!((gcode::generate(&d).est_seconds - 11.0).abs() < 1e-6);
    // The machine cannot go faster than 600 mm/min = 10 mm/s whatever the layer says.
    let mut fast = long_line_doc(40.0);
    fast.device.speeds.acceleration = 10.0;
    fast.device.speeds.max_cut_speed = 600.0;
    assert!((gcode::generate(&fast).est_seconds - 11.0).abs() < 1e-6);
    // A move too short to reach the speed is a triangle: 1 mm at a = 10 mm/s² takes 2 * sqrt(1 / 10) s.
    let mut short = long_line_doc(10.0);
    short.scene.shapes[0].kind = Kind::Path(vec![Polyline::new(vec![Pt::new(0.0, 0.0), Pt::new(1.0, 0.0)], false)]);
    short.device.speeds.acceleration = 10.0;
    assert!((gcode::generate(&short).est_seconds - 2.0 * (0.1f64).sqrt()).abs() < 1e-6);
    // Move durations still add up to the total.
    let job = gcode::generate(&d);
    assert!((job.moves.iter().map(|m| m.dur).sum::<f64>() - job.est_seconds).abs() < 1e-9);
}

#[test]
fn a_run_of_short_segments_ramps_once_and_a_layer_above_the_maximum_warns() {
    let mut d = long_line_doc(10.0);
    d.scene.shapes[0].kind = Kind::Path(vec![Polyline::new((0..=100).map(|i| Pt::new(i as f64, 0.0)).collect(), false)]);
    d.device.speeds.acceleration = 10.0;
    assert!((gcode::generate(&d).est_seconds - 11.0).abs() < 1e-6);
    d.device.speeds.max_cut_speed = 300.0; // 5 mm/s
    d.layers[0].speed = 10.0;
    assert!(check::job_check(&d).iter().any(|i| i.kind == check::IssueKind::SpeedAboveMax && i.level == check::Level::Warning));
    d.device.speeds.max_cut_speed = 0.0;
    assert!(!check::job_check(&d).iter().any(|i| i.kind == check::IssueKind::SpeedAboveMax));
    assert_eq!(d.device.suggested_overscan(10.0), Some(5.0));
    assert_eq!(Device::default().suggested_overscan(10.0), None);
}

#[test]
fn dialect_follows_the_controller_unless_chosen() {
    let mut d = Device::default();
    assert_eq!(d.dialect(), Dialect::GrblDynamic);
    d.gcode.dynamic_power = false;
    assert_eq!(d.dialect(), Dialect::Grbl);
    d.controller = Controller::Marlin;
    assert_eq!(d.dialect(), Dialect::Marlin);
    d.controller = Controller::Smoothieware;
    assert_eq!(d.dialect(), Dialect::Smoothieware);
    d.gcode.dialect = Some(Dialect::GrblDynamic);
    assert_eq!(d.dialect(), Dialect::GrblDynamic);
    assert!(Controller::Smoothieware.is_serial() && Controller::ALL.contains(&Controller::Smoothieware));
}

#[test]
fn smoothieware_status_reports_are_read() {
    let (state, x, y) = controller::parse_status("<Idle,MPos:12.5000,7.2500,0.0000,WPos:12.5000,7.2500,0.0000>").unwrap();
    assert_eq!((state.as_str(), x, y), ("Idle", 12.5, 7.25));
    // GRBL 1.1 reports still go the old way.
    let (state, x, y) = controller::parse_status("<Run|MPos:1.000,2.000,0.000|FS:500,0>").unwrap();
    assert_eq!((state.as_str(), x, y), ("Run", 1.0, 2.0));
}

#[test]
fn arc_fit_stays_within_the_tolerance_and_leaves_straight_runs_alone() {
    let tol = 0.01;
    // A quarter circle of 30 mm radius, finely flattened: one arc.
    let quarter: Vec<Pt> = (0..=60).map(|i| {
        let t = i as f64 / 60.0 * std::f64::consts::FRAC_PI_2;
        Pt::new(30.0 * t.cos(), 30.0 * t.sin())
    }).collect();
    let pieces = arcs::fit(&quarter, tol);
    assert_eq!(pieces.len(), 1, "{pieces:?}");
    let arcs::Piece::Arc { end, center } = pieces[0] else { panic!("an arc is expected") };
    assert_eq!(end, 60);
    assert!(center.dist(Pt::new(0.0, 0.0)) < 1e-6);
    // Straight lines never become arcs, whatever the number of points.
    let straight: Vec<Pt> = (0..20).map(|i| Pt::new(i as f64, 2.0 * i as f64)).collect();
    assert!(arcs::fit(&straight, tol).iter().all(|p| matches!(p, arcs::Piece::Line { .. })));
    // A line, an arc and a line: the pieces tile the path and every point is within the tolerance of its piece.
    let mut mixed: Vec<Pt> = (0..5).map(|i| Pt::new(-20.0 + 4.0 * i as f64, 30.0)).collect();
    mixed.extend((0..=40).map(|i| {
        let t = std::f64::consts::FRAC_PI_2 - i as f64 / 40.0 * std::f64::consts::FRAC_PI_2;
        Pt::new(30.0 * t.cos(), 30.0 * t.sin())
    }).skip(1));
    mixed.extend((1..6).map(|i| Pt::new(30.0, -4.0 * i as f64)));
    let pieces = arcs::fit(&mixed, tol);
    let mut at = 0;
    for p in &pieces {
        match *p {
            arcs::Piece::Line { end } => assert_eq!(end, at + 1),
            arcs::Piece::Arc { end, center } => {
                let r = center.dist(mixed[at]);
                for q in &mixed[at..=end] {
                    assert!((center.dist(*q) - r).abs() <= tol + 1e-9, "point off the arc by {}", (center.dist(*q) - r).abs());
                }
                assert!(end - at >= 3);
            }
        }
        at = match *p { arcs::Piece::Line { end } | arcs::Piece::Arc { end, .. } => end };
    }
    assert_eq!(at, mixed.len() - 1);
    assert!(pieces.iter().any(|p| matches!(p, arcs::Piece::Arc { .. })));
    // A tiny wiggle bigger than the tolerance is kept as lines.
    let zig: Vec<Pt> = (0..10).map(|i| Pt::new(i as f64, if i % 2 == 0 { 0.0 } else { 0.5 })).collect();
    assert!(arcs::fit(&zig, tol).iter().all(|p| matches!(p, arcs::Piece::Line { .. })));
}

/// Machine positions after every G0 / G1 / G2 / G3 line, with the arcs as (start, end, centre, clockwise).
fn parse_arcs(gcode: &str) -> Vec<((f64, f64), (f64, f64), (f64, f64), bool)> {
    let word = |l: &str, c: char| l.split_whitespace().find_map(|t| t.strip_prefix(c)).and_then(|v| v.parse::<f64>().ok());
    let mut pos = (0.0, 0.0);
    let mut out = vec![];
    for l in gcode.lines() {
        let l = l.split(';').next().unwrap_or("").trim();
        if !(l.starts_with("G0") || l.starts_with("G1") || l.starts_with("G2") || l.starts_with("G3")) {
            continue;
        }
        let end = (word(l, 'X').unwrap_or(pos.0), word(l, 'Y').unwrap_or(pos.1));
        if l.starts_with("G2 ") || l.starts_with("G3 ") {
            out.push((pos, end, (pos.0 + word(l, 'I').unwrap(), pos.1 + word(l, 'J').unwrap()), l.starts_with("G2 ")));
        }
        pos = end;
    }
    out
}

fn circle_doc(arcs_on: bool) -> Document {
    let mut d = Document::default();
    d.device.area = Area { bed_w: 300.0, bed_h: 200.0, ..Area::default() };
    d.device.gcode.arcs = arcs_on;
    d.layers[0].mode = LayerMode::Line;
    d.add(0, Kind::Ellipse { w: 40.0, h: 40.0 }, Xf::translate(100.0, 80.0));
    d
}

#[test]
fn circle_becomes_a_few_arcs_in_the_right_direction_for_every_axis_setup() {
    // Off by default: no arcs, many short lines.
    assert!(parse_arcs(&gcode::generate(&circle_doc(false)).gcode).is_empty());
    for origin in Origin::ALL {
        for (fx, fy) in [(false, false), (true, false), (false, true)] {
            for o in Orientation::ALL {
                let mut d = circle_doc(true);
                d.device.axes = Axes { origin, flip_x: fx, flip_y: fy, orientation: o, ..Axes::default() };
                let job = gcode::generate(&d);
                let arcs_found = parse_arcs(&job.gcode);
                assert!(!arcs_found.is_empty() && arcs_found.len() < 8, "{origin:?} {fx} {fy} {o:?}: {} arcs", arcs_found.len());
                let mut checked = 0;
                for (s, e, c, cw) in &arcs_found {
                    let r = ((s.0 - c.0).powi(2) + (s.1 - c.1).powi(2)).sqrt();
                    let re = ((e.0 - c.0).powi(2) + (e.1 - c.1).powi(2)).sqrt();
                    assert!((r - 20.0).abs() < 0.02 && (re - r).abs() < 0.02, "radius {r} / {re}");
                    // The circle's own path: take the moves of the job that lie between the arc's ends and check
                    // on which side of the chord the path bulges; that must agree with G2 / G3.
                    let (sd, ed) = (d.device.from_machine(s.0, s.1), d.device.from_machine(e.0, e.1));
                    let mid = job.moves.iter().filter(|m| m.laser).find(|m| m.a.dist(sd) < 0.02).map(|m| m.b);
                    let Some(next) = mid else { continue };
                    // Machine direction of travel around the centre from the start through the first path point.
                    let n = d.device.to_machine(next);
                    let turn = (s.0 - c.0) * (n.1 - c.1) - (s.1 - c.1) * (n.0 - c.0);
                    assert_eq!(turn < 0.0, *cw, "{origin:?} {fx} {fy} {o:?} start {sd:?} end {ed:?}");
                    checked += 1;
                }
                assert!(checked > 0, "{origin:?} {fx} {fy} {o:?}: no arc could be checked");
            }
        }
    }
}

#[test]
fn grbl_settings_report_validation_and_backup() {
    use crate::grbl_settings as gs;
    let lines: Vec<String> = ["$0=10", "$30=1000.000", "$32=0", "$110=3000.000", "$130=400.000", "ok", "[VER:1.1h.20190825:]", "$x=1", "$131="].iter().map(|s| s.to_string()).collect();
    let v = gs::parse_report(&lines);
    assert_eq!(v.len(), 5);
    assert_eq!(v[&110], "3000.000");
    // Writing: ranges, masks, flags and numbers that are not numbers.
    assert_eq!(gs::validate(110, " 5000 "), Ok("5000".into()));
    assert_eq!(gs::validate(11, "0.02"), Ok("0.02".into()));
    assert_eq!(gs::validate(110, "abc"), Err("not a number"));
    assert_eq!(gs::validate(110, "NaN"), Err("not a number"));
    assert_eq!(gs::validate(0, "1"), Err("out of range"));
    assert_eq!(gs::validate(32, "2"), Err("out of range"));
    assert_eq!(gs::validate(32, "1"), Ok("1".into()));
    assert_eq!(gs::validate(3, "7"), Ok("7".into()));
    assert_eq!(gs::validate(3, "8"), Err("out of range"));
    assert_eq!(gs::validate(3, "1.5"), Err("out of range"));
    assert_eq!(gs::write_line(110, "5000"), "$110=5000");
    // Backup text round-trips and is read strictly.
    let text = gs::backup_text(&v, &["[VER:1.1h.20190825:]".to_string()], "2026-10-11");
    assert!(text.starts_with("; LightCreator backup") && text.contains("$110=3000.000"));
    let back = gs::parse_backup(&text).unwrap();
    assert_eq!(back.len(), 5);
    assert!(back.contains(&(110, "3000".into())));
    assert!(gs::parse_backup("$110=1\n$110=2\n").unwrap_err().contains("twice"));
    assert!(gs::parse_backup("G1 X1").unwrap_err().contains("line 1"));
    assert!(gs::parse_backup("$110=zz").unwrap_err().contains("line 1"));
    assert!(gs::parse_backup("; only comments\n").is_err());
    assert!(gs::parse_backup(&"$1=1\n".repeat(10_000)).is_err());
    assert!(gs::same_value("500", "500.000") && !gs::same_value("500", "501"));
    // The table: the critical ones, and no setting twice.
    let critical: Vec<u32> = gs::SETTINGS.iter().filter(|s| s.critical).map(|s| s.id).collect();
    assert_eq!(critical, vec![30, 31, 32, 130, 131, 132]);
    let mut ids: Vec<u32> = gs::SETTINGS.iter().map(|s| s.id).collect();
    ids.dedup();
    assert_eq!(ids.len(), gs::SETTINGS.len());
}

#[test]
fn laser_heads_keep_their_settings_and_old_profiles_get_defaults() {
    let d: Device = serde_json::from_str(OLD_PROFILE).unwrap();
    let h = &d.heads[0];
    // What an old profile did not have comes with sensible defaults.
    assert_eq!((h.tool, h.frame_speed, h.focus_power, h.spot_x, h.pointer_dx), (0, 40.0, 0.0, 0.0, 0.0));
    assert_eq!((h.wavelength(), h.power_w()), (10_600, 40.0));
    assert!(!h.has_pointer_offset());
    // A head with everything set survives a round trip through JSON.
    let mut dev = Device::default();
    dev.heads[0] = LaserHead { name: "Blue".into(), tool: 0, laser: LaserKind::Diode, wavelength_nm: 450, optical_power_w: 20.0, focus_power: 1.5, spot_x: 0.08, spot_y: 0.1, pointer_dx: 12.0, pointer_dy: -3.5, ..LaserHead::default() };
    dev.heads.push(LaserHead { name: "Fibre".into(), tool: 1, laser: LaserKind::Fiber, pwm_freq_hz: 30_000.0, pulse_width_us: 4.0, ..LaserHead::default() });
    let back: Device = serde_json::from_str(&serde_json::to_string(&dev).unwrap()).unwrap();
    assert_eq!(back, dev);
    assert_eq!((back.heads[1].wavelength(), back.heads[0].wavelength()), (1064, 450));
    assert!(back.heads[0].has_pointer_offset() && LaserKind::Fiber.has_pwm() && !LaserKind::Diode.has_pwm());
    // Tool numbers must differ; one head never clashes with itself.
    assert!(!back.has_duplicate_tools());
    let mut clash = back.clone();
    clash.heads[1].tool = 0;
    assert!(clash.has_duplicate_tools() && !Device::default().has_duplicate_tools());
    // A head the profile does not have falls back to the first.
    assert_eq!(dev.head(7).name, "Blue");
}

#[test]
fn without_power_keeps_the_path_and_zeroes_every_s_of_a_move() {
    let g = "; job\nM4 S0\nG0 X1.000 Y2.000\nG1 X5.000 Y2.000 S600 F1200 ; cut\nG2 X6.000 Y3.000 I0.500 J0.500 S300\nG1 X9.000\nM5\n";
    let off = gcode::without_power(g);
    assert_eq!(off, "; job\nM4 S0\nG0 X1.000 Y2.000\nG1 X5.000 Y2.000 S0 F1200 ; cut\nG2 X6.000 Y3.000 I0.500 J0.500 S0\nG1 X9.000\nM5\n");
    // Moves and everything else are untouched; the shifted copy moves every shape.
    let mut d = Document::default();
    d.add(0, Kind::Rect { w: 4.0, h: 4.0 }, Xf::translate(10.0, 10.0));
    let s = gcode::shifted(&d, -3.0, 2.0);
    assert_eq!(s.scene.shapes[0].bounds().unwrap().min, Pt::new(7.0, 12.0));
    assert_eq!(d.scene.shapes[0].bounds().unwrap().min, Pt::new(10.0, 10.0));
}

fn macro_dev(macros: &[(&str, &str)]) -> Device {
    let mut d = Device::default();
    d.name = "Test laser".into();
    d.area = Area { bed_w: 300.0, bed_h: 200.0, ..Area::default() };
    d.macros = macros.iter().map(|(n, c)| Macro { name: n.to_string(), code: c.to_string() }).collect();
    d
}

#[test]
fn a_macro_is_expanded_without_comments_with_includes_and_variables() {
    let d = macro_dev(&[
        ("Park", "G90 ; absolute\n(move away)\nG0 X{work_width} Y0 (corner)\n\n@include(Beep)\nM5"),
        ("Beep", "M300 S440 P100 ; beep\nM117 {machine_name}"),
    ]);
    assert_eq!(hooks::expand(&d, "Park").unwrap(), vec!["G90", "G0 X300 Y0", "M300 S440 P100", "M117 Test laser", "M5"]);
    // Names are found whatever the case; unknown ones are told apart from empty ones.
    assert!(hooks::expand(&d, "park").is_ok());
    assert_eq!(hooks::expand(&d, "Nope"), Err(hooks::MacroError::Unknown("Nope".into())));
    assert_eq!(hooks::expand(&macro_dev(&[("Empty", "; only a comment\n(and another)")]), "Empty"), Err(hooks::MacroError::Empty));
    assert_eq!(hooks::validate_macros(&d), None);
    assert_eq!(hooks::strip_comments("G1 X(a;b)5 ; tail"), "G1 X5");
}

#[test]
fn includes_that_loop_or_go_too_deep_are_refused() {
    let d = macro_dev(&[("A", "@include(B)"), ("B", "@include(A)")]);
    assert!(matches!(hooks::expand(&d, "A"), Err(hooks::MacroError::Cycle(_))));
    assert!(hooks::validate_macros(&d).is_some());
    let selfie = macro_dev(&[("Self", "@include(self)")]);
    assert!(matches!(hooks::expand(&selfie, "Self"), Err(hooks::MacroError::Cycle(_))));
    // A chain of ten macros is deeper than the limit of eight; one of eight is fine.
    let chain = |n: usize| -> Device {
        let ms: Vec<(String, String)> = (0..n).map(|i| (format!("m{i}"), if i + 1 < n { format!("@include(m{})", i + 1) } else { "G4 P1".to_string() })).collect();
        let refs: Vec<(&str, &str)> = ms.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
        macro_dev(&refs)
    };
    assert_eq!(hooks::expand(&chain(10), "m0"), Err(hooks::MacroError::TooDeep));
    assert_eq!(hooks::expand(&chain(8), "m0").unwrap(), vec!["G4 P1"]);
}

#[test]
fn bad_macros_cannot_be_kept() {
    let bad = |code: &str| hooks::validate_macros(&macro_dev(&[("X", code)])).is_some();
    assert!(bad("G1 X{layer_name}"), "a job variable");
    assert!(bad("G1 X{nope}"), "an unknown variable");
    assert!(bad("G1 X{work_width"), "an open brace");
    assert!(bad("@include Beep"), "an include without brackets");
    assert!(bad("@include(Missing)"));
    assert!(bad("G1 X1\u{7}"), "a control character");
    assert!(bad(&format!("G1 X{}", "1".repeat(120))), "a line over 100 characters");
    assert!(bad(&"G4 P1\n".repeat(70)), "more than 60 lines in a macro");
    assert!(!bad("G4 P1"));
    // Names: not empty, not used twice, not absurd.
    let named = |names: &[&str]| hooks::validate_macros(&macro_dev(&names.iter().map(|n| (*n, "G4 P1")).collect::<Vec<_>>())).is_some();
    assert!(named(&[""]) && named(&["a", "A"]) && named(&["a(b)"]) && named(&[&"n".repeat(41)]));
    assert!(!named(&["Home", "Park"]));
    // Macros ride along in the profile; old profiles have none.
    let d = macro_dev(&[("Park", "G0 X0 Y0")]);
    let back: Device = serde_json::from_str(&serde_json::to_string(&d).unwrap()).unwrap();
    assert_eq!(back.macros, d.macros);
    let old: Device = serde_json::from_str(OLD_PROFILE).unwrap();
    assert!(old.macros.is_empty());
}

fn rect_zone() -> ZoneShape {
    ZoneShape::Rect { x: 10.0, y: 10.0, w: 20.0, h: 10.0 } // (10,10) to (30,20)
}

#[test]
fn segments_against_a_rectangular_zone_including_the_edge_cases() {
    let z = rect_zone();
    let hit = |a: (f64, f64), b: (f64, f64)| zones::segment_hits(&z, Pt::new(a.0, a.1), Pt::new(b.0, b.1));
    assert!(!hit((0.0, 0.0), (5.0, 5.0)), "far away");
    assert!(!hit((0.0, 25.0), (40.0, 25.0)), "passes below");
    assert!(!hit((0.0, 9.99), (40.0, 9.99)), "parallel, just outside");
    assert!(hit((0.0, 15.0), (40.0, 15.0)), "straight through");
    assert!(hit((15.0, 12.0), (25.0, 18.0)), "completely inside");
    assert!(hit((0.0, 15.0), (12.0, 15.0)), "ends inside");
    assert!(hit((0.0, 10.0), (40.0, 10.0)), "runs along the edge");
    assert!(hit((10.0, 0.0), (10.0, 10.0)), "just touches a corner");
    assert!(hit((0.0, 0.0), (10.0, 10.0)), "touches the corner from outside");
    assert!(!hit((0.0, 0.0), (9.9, 9.9)), "stops short of the corner");
    assert!(hit((20.0, 15.0), (20.0, 15.0)), "a point inside");
    assert!(!hit((5.0, 5.0), (5.0, 5.0)), "a point outside");
    assert!(hit((0.0, 0.0), (40.0, 30.0)), "diagonal across a corner region: through");
    assert!(!hit((0.0, 12.0), (9.0, 30.0)), "diagonal that misses");
    // A rectangle given with a negative size is the same rectangle.
    assert!(zones::segment_hits(&ZoneShape::Rect { x: 30.0, y: 20.0, w: -20.0, h: -10.0 }, Pt::new(0.0, 15.0), Pt::new(40.0, 15.0)));
}

#[test]
fn segments_against_a_round_zone() {
    let z = ZoneShape::Circle { x: 50.0, y: 50.0, r: 10.0 };
    let hit = |a: (f64, f64), b: (f64, f64)| zones::segment_hits(&z, Pt::new(a.0, a.1), Pt::new(b.0, b.1));
    assert!(hit((0.0, 50.0), (100.0, 50.0)), "through the middle");
    assert!(hit((0.0, 40.0), (100.0, 40.0)), "tangent: touching counts");
    assert!(!hit((0.0, 39.9), (100.0, 39.9)), "just outside");
    assert!(hit((45.0, 50.0), (55.0, 52.0)), "inside");
    assert!(hit((0.0, 0.0), (50.0, 45.0)), "ends inside");
    assert!(!hit((0.0, 0.0), (30.0, 30.0)), "stops before");
    assert!(hit((50.0, 50.0), (50.0, 50.0)));
    assert_eq!(z_bounds(), (40.0, 40.0, 60.0, 60.0));
}

fn z_bounds() -> (f64, f64, f64, f64) {
    let b = ZoneShape::Circle { x: 50.0, y: 50.0, r: 10.0 }.bounds();
    (b.min.x, b.min.y, b.max.x, b.max.y)
}

fn zoned_doc() -> Document {
    let mut d = check_doc(100.0, 100.0);
    d.device.zones = vec![Zone { name: "Clamp".into(), enabled: true, shape: ZoneShape::Rect { x: 140.0, y: 60.0, w: 40.0, h: 30.0 }, z_range: None }];
    d
}

#[test]
fn a_zone_in_the_way_of_any_move_is_an_error_naming_the_zone_and_the_layer() {
    // The rectangle sits at (100,100): the zone is not on its outline ...
    let mut d = zoned_doc();
    assert!(check::job_check(&d).is_empty(), "{:?}", check::job_check(&d));
    // ... but the way back to the machine's zero (front-left: bottom-left of the design) is a move too: put a zone on it.
    d.device.zones.push(Zone { name: "Way home".into(), enabled: true, shape: ZoneShape::Circle { x: 50.0, y: 150.0, r: 5.0 }, z_range: None });
    let issues = check::job_check(&d);
    let hit: Vec<_> = issues.iter().filter(|i| i.kind == check::IssueKind::NoGoZone).collect();
    assert_eq!(hit.len(), 1, "{issues:?}");
    assert_eq!((hit[0].zone, hit[0].level), (Some(1), check::Level::Error));
    assert_eq!(hit[0].layer, Some(0));
    // A zone on the outline itself.
    let mut on = zoned_doc();
    on.device.zones[0].shape = ZoneShape::Rect { x: 110.0, y: 95.0, w: 5.0, h: 20.0 };
    assert!(check::job_check(&on).iter().any(|i| i.kind == check::IssueKind::NoGoZone && i.zone == Some(0)));
    // Switched off, it is not a zone.
    on.device.zones[0].enabled = false;
    assert!(!check::job_check(&on).iter().any(|i| i.kind == check::IssueKind::NoGoZone));
}

#[test]
fn the_frame_may_not_touch_a_zone_either_and_zones_travel_with_the_profile() {
    let d = zoned_doc();
    let r = |x0, y0, x1, y1| Rect { min: Pt::new(x0, y0), max: Pt::new(x1, y1) };
    assert!(check::check_frame(&d.device, r(100.0, 100.0, 120.0, 110.0)).is_empty());
    // A frame that encloses the zone has its edges clear of it: only the edges are traced.
    assert!(check::check_frame(&d.device, r(100.0, 40.0, 200.0, 110.0)).is_empty());
    let hit = check::check_frame(&d.device, r(100.0, 70.0, 160.0, 110.0));
    assert!(hit.iter().any(|i| i.kind == check::IssueKind::NoGoZone && i.zone == Some(0)), "{hit:?}");
    let back: Device = serde_json::from_str(&serde_json::to_string(&d.device).unwrap()).unwrap();
    assert_eq!(back.zones, d.device.zones);
    let old: Device = serde_json::from_str(OLD_PROFILE).unwrap();
    assert!(old.zones.is_empty());
}

#[test]
fn z_heights_outside_the_travel_are_an_error_and_status_reports_give_z() {
    let mut d = check_doc(100.0, 100.0);
    d.device.axes.has_z = true;
    d.device.axes.z_min = 0.0;
    d.device.axes.z_max = 30.0;
    d.layers[0].passes = 3;
    d.layers[0].z_start = 20.0;
    d.layers[0].z_step = -5.0;
    assert!(check::job_check(&d).is_empty());
    d.layers[0].z_step = -10.0; // 20, 10, 0: still in the travel
    assert!(check::job_check(&d).is_empty());
    d.layers[0].z_step = -12.0; // 20, 8, -4: below the travel
    assert!(check::job_check(&d).iter().any(|i| i.kind == check::IssueKind::ZOutOfRange && i.layer == Some(0) && i.level == check::Level::Error));
    d.layers[0].z_step = 5.0; // 20, 25, 30
    assert!(check::job_check(&d).is_empty());
    d.layers[0].z_start = 40.0;
    assert!(check::job_check(&d).iter().any(|i| i.kind == check::IssueKind::ZOutOfRange));
    // Without a Z axis the heights mean nothing.
    d.device.axes.has_z = false;
    assert!(check::job_check(&d).is_empty());
    // Z in the reports of the controllers.
    assert_eq!(controller::parse_status_z("<Idle|MPos:1.000,2.000,3.500|FS:0,0>"), Some(3.5));
    assert_eq!(controller::parse_status_z("X:1.00 Y:2.00 Z:7.25 E:0.00 Count X:80"), Some(7.25));
    assert_eq!(controller::parse_status_z("<Idle,MPos:1.0000,2.0000,4.5000,WPos:1.0000,2.0000,4.5000>"), Some(4.5));
    assert_eq!(controller::parse_status_z("ok"), None);
    assert_eq!(controller::parse_status_z("<Idle|FS:0,0>"), None);
    // Z axis settings live in the profile; old ones have no axis.
    let old: Device = serde_json::from_str(OLD_PROFILE).unwrap();
    assert!(!old.axes.has_z);
}

#[test]
fn rotary_formulas_for_a_chuck_and_for_rollers_worked_out_by_hand() {
    use std::f64::consts::PI;
    // A chuck: the work is 50 mm across, the motor turns it: one turn = 6400 microsteps = pi * 50 = 157.0796 mm.
    let chuck = Rotary { enabled: true, mode: RotaryMode::ReplaceY, kind: RotaryKind::Chuck, diameter: 50.0, roller_diameter: 30.0, microsteps_per_rev: 6400.0 };
    assert!((chuck.circumference() - 157.0796).abs() < 1e-3);
    assert!((chuck.steps_per_mm() - 40.7437).abs() < 1e-3, "{}", chuck.steps_per_mm()); // 6400 / 157.0796
    assert!((chuck.degrees_per_mm() - 2.29183).abs() < 1e-4); // 360 / 157.0796
    // A quarter of the way round is 39.27 mm and 90 degrees.
    assert!((39.2699 * chuck.degrees_per_mm() - 90.0).abs() < 1e-3);
    // Rollers: the work (80 mm) lies on 30 mm rollers; the surface moves like the roller's surface (94.2478 mm a turn).
    let roller = Rotary { kind: RotaryKind::Roller, diameter: 80.0, roller_diameter: 30.0, ..chuck.clone() };
    assert!((roller.steps_per_mm() - 67.906).abs() < 1e-3, "{}", roller.steps_per_mm()); // 6400 / (pi * 30)
    assert!((roller.degrees_per_mm() - 3.81972).abs() < 1e-4); // 360 / (pi * 30)
    // ... but the way round the work is its own: pi * 80 = 251.327 mm.
    assert!((roller.circumference() - 251.327).abs() < 1e-3);
    assert_eq!(roller.drive_diameter(), 30.0);
    // Zero diameters do not divide by zero.
    let none = Rotary { diameter: 0.0, ..chuck };
    assert_eq!((none.steps_per_mm(), none.degrees_per_mm(), none.circumference()), (0.0, 0.0, 0.0));
    let _ = PI;
}

fn rotary_check_doc(rot: &[bool], heights: &[f64]) -> Document {
    let mut d = Document::default();
    d.device.area = Area { bed_w: 300.0, bed_h: 300.0, ..Area::default() };
    d.device.rotary = Rotary { enabled: true, diameter: 50.0, ..Rotary::default() }; // 157.08 mm round
    for (i, (r, h)) in rot.iter().zip(heights).enumerate() {
        d.layers[i].rotary = *r;
        d.layers[i].mode = LayerMode::Line;
        d.add(i, Kind::Rect { w: 20.0, h: *h }, Xf::translate(20.0 + 40.0 * i as f64, 20.0));
    }
    d
}

#[test]
fn rotary_and_flat_layers_do_not_mix_and_the_design_must_fit_round_the_work() {
    let kinds = |d: &Document| check::job_check(d).into_iter().map(|i| i.kind).collect::<Vec<_>>();
    // All rotary and low enough: fine.
    assert!(kinds(&rotary_check_doc(&[true, true], &[100.0, 150.0])).is_empty());
    // One flat layer next to a rotary one.
    assert!(kinds(&rotary_check_doc(&[true, false], &[100.0, 50.0])).contains(&check::IssueKind::RotaryMix));
    // Higher than the circumference (157.08 mm).
    let too_tall = rotary_check_doc(&[true], &[160.0]);
    let issues = check::job_check(&too_tall);
    assert!(issues.iter().any(|i| i.kind == check::IssueKind::RotaryTooTall && i.layer == Some(0) && i.level == check::Level::Error && !i.shapes.is_empty()), "{issues:?}");
    // Exactly round is allowed (within the tolerance).
    assert!(!kinds(&rotary_check_doc(&[true], &[157.0])).contains(&check::IssueKind::RotaryTooTall));
    // All flat layers on a machine with a rotary switched on are only flat jobs.
    assert!(kinds(&rotary_check_doc(&[false, false], &[250.0, 10.0])).is_empty());
    // With the rotary switched off the flags mean nothing.
    let mut off = rotary_check_doc(&[true, false], &[100.0, 50.0]);
    off.device.rotary.enabled = false;
    assert!(kinds(&off).is_empty());
    // The fourth-axis mode falls back to Y on GRBL, and stays on Marlin.
    let mut dev = Device::default();
    dev.rotary.mode = RotaryMode::TrueA;
    assert_eq!(dev.rotary_mode(), RotaryMode::ReplaceY);
    dev.controller = Controller::Marlin;
    assert_eq!(dev.rotary_mode(), RotaryMode::TrueA);
    // Old profiles have no rotary.
    let old: Device = serde_json::from_str(OLD_PROFILE).unwrap();
    assert!(!old.rotary.enabled);
}

#[test]
fn a_zone_with_a_height_range_only_stops_moves_at_those_heights_and_only_with_a_z_axis() {
    // The zone is a box 0-10 mm high standing on the table where the design's outline runs.
    let mut d = check_doc(100.0, 100.0);
    d.device.zones = vec![Zone { name: "Box".into(), enabled: true, shape: ZoneShape::Rect { x: 105.0, y: 95.0, w: 5.0, h: 20.0 }, z_range: Some((0.0, 10.0)) }];
    let hit = |d: &Document| check::job_check(d).iter().any(|i| i.kind == check::IssueKind::NoGoZone);
    // No Z axis: the range means nothing, the zone is a plain area.
    assert!(hit(&d));
    d.device.axes.has_z = true;
    d.device.axes.z_max = 100.0;
    // A layer that does not say where Z is: the box is assumed to be in the way.
    assert!(hit(&d));
    // Moving at 30 mm, above the box: free. At 5 mm and 0 mm, the box is hit.
    d.layers[0].z_start = 30.0;
    assert!(!hit(&d));
    d.layers[0].z_start = 5.0;
    assert!(hit(&d));
    d.layers[0].z_start = 10.0; // the end of the range counts
    assert!(hit(&d));
    d.layers[0].z_start = 10.5;
    assert!(!hit(&d));
    // Two passes, the second one going down into the box.
    d.layers[0].passes = 2;
    d.layers[0].z_start = 20.0;
    d.layers[0].z_step = -15.0;
    assert!(hit(&d));
    // The range is kept in the profile; old zones have none.
    let back: Device = serde_json::from_str(&serde_json::to_string(&d.device).unwrap()).unwrap();
    assert_eq!(back.zones[0].z_range, Some((0.0, 10.0)));
    let no_range: Zone = serde_json::from_str(r#"{"name":"x","shape":{"Rect":{"x":0.0,"y":0.0,"w":1.0,"h":1.0}}}"#).unwrap();
    assert!(no_range.enabled && no_range.z_range.is_none());
}

#[test]
fn copies_get_a_free_name() {
    let taken = vec!["Laser".to_string(), "Laser (copy)".to_string()];
    assert_eq!(profiles::copy_name("Laser", &[]), "Laser (copy)");
    assert_eq!(profiles::copy_name("Laser", &taken), "Laser (copy 2)");
    assert_eq!(profiles::copy_name(" Laser ", &taken), "Laser (copy 2)");
}

#[test]
fn a_profile_file_round_trips_without_the_port_and_old_files_are_read() {
    let mut d = Device::default();
    d.name = "K40".into();
    d.axes.origin = Origin::BackLeft;
    d.heads[0].laser = LaserKind::Co2;
    d.connection.port = "/dev/tty.usbserial-1410".into();
    d.connection.usb_id = "1a86:7523".into();
    d.macros = vec![Macro { name: "Park".into(), code: "G0 X0 Y0".into() }];
    d.zones = vec![Zone { name: "Clamp".into(), enabled: true, shape: ZoneShape::Circle { x: 10.0, y: 10.0, r: 5.0 }, z_range: None }];
    let text = profiles::export_profile(&d, "v0.0.11");
    assert!(text.contains("\"format\": 1") && !text.contains("usbserial"), "the port stays at home");
    let back = profiles::import_profile(&text).unwrap();
    d.connection.port.clear();
    assert_eq!(back, d);
    // A bare profile, as the program keeps it, is accepted too (older exports).
    let bare = profiles::import_profile(&serde_json::to_string(&d).unwrap()).unwrap();
    assert_eq!(bare, d);
    // An even older flat profile.
    let flat = profiles::import_profile(OLD_PROFILE).unwrap();
    assert_eq!(flat.area.bed_w, 300.0);
}

#[test]
fn a_bad_profile_file_gives_a_clear_message_and_never_a_panic() {
    let ok = profiles::export_profile(&Device::default(), "v0.0.11");
    let err = |t: &str| profiles::import_profile(t).unwrap_err();
    assert!(err("hello").contains("not a profile file"));
    assert!(err("").contains("not a profile file"));
    assert!(err("{\"profile\": 5}").contains("cannot be read"));
    assert!(err(&"x".repeat(profiles::MAX_PROFILE_FILE + 1)).contains("larger"));
    // A newer format is refused with the reason.
    let newer = ok.replace("\"format\": 1", "\"format\": 9");
    assert!(err(&newer).contains("newer version"));
    // Values out of range.
    let edit = |f: &dyn Fn(&mut serde_json::Value)| -> String {
        let mut v: serde_json::Value = serde_json::from_str(&ok).unwrap();
        f(&mut v);
        profiles::import_profile(&v.to_string()).unwrap_err()
    };
    assert!(edit(&|v| v["profile"]["area"]["bed_w"] = 1e9.into()).contains("work area"));
    assert!(edit(&|v| v["profile"]["area"]["bed_h"] = (-5.0).into()).contains("work area"));
    assert!(edit(&|v| v["profile"]["name"] = "".into()).contains("name"));
    assert!(edit(&|v| v["profile"]["name"] = "x".repeat(200).into()).contains("name"));
    assert!(edit(&|v| v["profile"]["name"] = "a\u{7}b".into()).contains("name"));
    assert!(edit(&|v| v["profile"]["heads"][0]["s_max"] = 0.0.into()).contains("laser head"));
    assert!(edit(&|v| v["profile"]["connection"]["baud"] = 5.into()).contains("connection"));
    assert!(edit(&|v| v["profile"]["connection"]["usb_id"] = "nonsense".into()).contains("connection"));
    assert!(edit(&|v| v["profile"]["gcode"]["preamble"] = "G21\u{1b}[2J".into()).contains("G-code"));
    assert!(edit(&|v| v["profile"]["gcode"]["hooks"]["layer_start"] = serde_json::json!({"enabled": true, "code": "M117 {nope}"})).contains("hook"));
    assert!(edit(&|v| v["profile"]["macros"] = serde_json::json!([{"name": "A", "code": "@include(A)"}])).contains("macro"));
    assert!(edit(&|v| v["profile"]["zones"] = serde_json::json!([{"name": "z", "shape": {"Circle": {"x": 0.0, "y": 0.0, "r": 1e12}}}])).contains("zone"));
    assert!(edit(&|v| v["profile"]["jog_step"] = 0.0.into()).contains("jog"));
    // Unknown fields are ignored, and a name with a path is just a name.
    let mut v: serde_json::Value = serde_json::from_str(&ok).unwrap();
    v["profile"]["something_new"] = "from a later version".into();
    v["profile"]["name"] = "../../etc/passwd".into();
    v["profile"]["connection"]["port"] = "/etc/passwd".into();
    let d = profiles::import_profile(&v.to_string()).unwrap();
    assert!(d.connection.port.is_empty(), "no path from the file reaches a port");
}

#[test]
fn maintenance_counts_resets_and_is_cleared_for_imports() {
    let mut m = Maintenance { total_secs: 0.0, counters: vec![Counter { name: "Tube".into(), secs: 0.0, threshold_hours: Some(1.0) }, Counter { name: "Belt".into(), secs: 0.0, threshold_hours: None }] };
    m.add_run(1800.0);
    m.add_run(f64::NAN);
    m.add_run(-5.0);
    assert_eq!((m.total_secs, m.counters[0].secs, m.counters[1].secs), (1800.0, 1800.0, 1800.0));
    assert!(m.due().is_empty());
    m.add_run(1800.0);
    assert_eq!(m.due().iter().map(|c| c.name.as_str()).collect::<Vec<_>>(), vec!["Tube"], "one hour of the limit is reached, the counter without a limit is never due");
    m.reset(0);
    m.reset(9);
    assert_eq!((m.total_secs, m.counters[0].secs, m.counters[1].secs), (3600.0, 0.0, 3600.0));
    // A profile carries them; a file made from it starts them again, and old profiles have none.
    let mut d = Device::default();
    d.maintenance = m;
    let back: Device = serde_json::from_str(&serde_json::to_string(&d).unwrap()).unwrap();
    assert_eq!(back.maintenance, d.maintenance);
    let imported = profiles::import_profile(&profiles::export_profile(&d, "v0.0.11")).unwrap();
    assert_eq!((imported.maintenance.total_secs, imported.maintenance.counters[1].secs, imported.maintenance.counters[0].name.as_str()), (0.0, 0.0, "Tube"));
    let old: Device = serde_json::from_str(OLD_PROFILE).unwrap();
    assert_eq!(old.maintenance, Maintenance::default());
}

#[test]
fn notes_live_in_the_profile_and_are_limited_in_imported_files() {
    let mut d = Device::default();
    d.notes = "# Tube\n- replaced in 2026".into();
    let back: Device = serde_json::from_str(&serde_json::to_string(&d).unwrap()).unwrap();
    assert_eq!(back.notes, d.notes);
    let imported = profiles::import_profile(&profiles::export_profile(&d, "v0.0.11")).unwrap();
    assert_eq!(imported.notes, d.notes);
    d.notes = "x".repeat(profiles::MAX_NOTES + 1);
    assert!(profiles::validate(&d).unwrap_err().contains("notes"));
    d.notes = "a\u{7}b".into();
    assert!(profiles::validate(&d).is_err());
    let old: Device = serde_json::from_str(OLD_PROFILE).unwrap();
    assert!(old.notes.is_empty());
}

fn gradient_rgb(w: usize, h: usize) -> Vec<u8> {
    (0..w * h).flat_map(|i| [(i % w * 255 / w.max(1)) as u8, (i / w * 255 / h.max(1)) as u8, 128]).collect()
}

#[test]
fn picture_adjustments_change_nothing_by_default_and_do_what_they_say() {
    let img = gradient_rgb(8, 6);
    let mut same = img.clone();
    camimg::adjust(&mut same, 8, 6, &ImageSettings::default());
    assert_eq!(same, img);
    // Brightness +100 adds 255, -100 subtracts it.
    let mut bright = img.clone();
    camimg::adjust(&mut bright, 8, 6, &ImageSettings { brightness: 20, ..Default::default() });
    assert_eq!(bright[2], 128 + 51);
    let mut dark = img.clone();
    camimg::adjust(&mut dark, 8, 6, &ImageSettings { brightness: -100, ..Default::default() });
    assert!(dark.iter().all(|v| *v == 0));
    // Contrast pushes values away from the middle: mid grey stays, lighter gets lighter, darker darker.
    let mut c = vec![128u8, 128, 128, 200, 200, 200, 60, 60, 60];
    camimg::adjust(&mut c, 3, 1, &ImageSettings { contrast: 100, ..Default::default() });
    assert_eq!(&c[0..3], &[128, 128, 128]);
    assert!(c[3] > 200 && c[6] < 60, "{c:?}");
    // White balance: a warm (2800 K) light is cooled, so blue gains on red; 6500 K is daylight and changes nothing.
    let mut warm = vec![200u8, 150, 100];
    camimg::adjust(&mut warm, 1, 1, &ImageSettings { white_balance: Some(2800), ..Default::default() });
    assert!(warm[0] < 200 && warm[2] >= 100, "{warm:?}");
    let mut day = vec![200u8, 150, 100];
    camimg::adjust(&mut day, 1, 1, &ImageSettings { white_balance: Some(6500), ..Default::default() });
    assert!(day.iter().zip([200u8, 150, 100]).all(|(a, b)| (*a as i32 - b as i32).abs() <= 4), "{day:?}");
    // Denoising smooths a lone bright pixel and leaves flat areas alone.
    let mut dot = vec![0u8; 5 * 5 * 3];
    dot[(2 * 5 + 2) * 3..(2 * 5 + 2) * 3 + 3].copy_from_slice(&[255, 255, 255]);
    camimg::adjust(&mut dot, 5, 5, &ImageSettings { denoise: 0.9, ..Default::default() });
    assert!(dot[(2 * 5 + 2) * 3] < 100 && dot[(2 * 5 + 1) * 3] > 0);
    let mut flat = vec![90u8; 6 * 6 * 3];
    camimg::adjust(&mut flat, 6, 6, &ImageSettings { denoise: 0.95, ..Default::default() });
    assert!(flat.iter().all(|v| *v == 90));
    // Values outside the ranges are clamped; garbage cannot panic.
    let wild = ImageSettings { brightness: 9999, contrast: 9999, white_balance: Some(1), denoise: f32::NAN, prefer_yuyv: false }.clamped();
    assert_eq!((wild.brightness, wild.contrast, wild.white_balance, wild.denoise), (100, 100, Some(2500), 0.0));
    let mut tiny = vec![1u8, 2, 3];
    camimg::adjust(&mut tiny, 1, 1, &ImageSettings { denoise: 0.9, brightness: 5, ..Default::default() });
    camimg::adjust(&mut tiny, 100, 100, &ImageSettings { brightness: 5, ..Default::default() }); // buffer too small: ignored
}

#[test]
fn lens_correction_with_zero_coefficients_is_the_identity_and_barrel_distortion_is_undone() {
    let img = gradient_rgb(40, 30);
    assert_eq!(camimg::undistort(&img, 40, 30, &Lens::default()), img);
    // Insane coefficients are ignored, not applied.
    assert_eq!(camimg::undistort(&img, 40, 30, &Lens { k1: f64::NAN, ..Default::default() }), img);
    assert_eq!(camimg::undistort(&img, 40, 30, &Lens { k1: 99.0, ..Default::default() }), img);
    assert_eq!(camimg::undistort(&[], 0, 0, &Lens { k1: 0.1, ..Default::default() }), Vec::<u8>::new());
    // A stretch of the picture: with k1 > 0 the corner of the result is taken from further out, so it goes black or
    // changes, while the centre keeps its colour.
    let lens = Lens { k1: 0.25, ..Default::default() };
    let out = camimg::undistort(&img, 40, 30, &lens);
    assert_eq!(out.len(), img.len());
    let centre = (15 * 40 + 20) * 3;
    assert!((out[centre] as i32 - img[centre] as i32).abs() <= 8, "the centre stays");
    assert_ne!(&out[0..3], &img[0..3], "the corner moves");
    // The mapping in the middle of a side: undistorting a picture made with the same model brings it back.
    let (w, h) = (60usize, 60usize);
    let f = w as f64;
    let src: Vec<u8> = (0..w * h).flat_map(|i| {
        let (x, y) = ((i % w) as f64, (i / w) as f64);
        // A smooth pattern, so interpolation is accurate.
        let v = (128.0 + 100.0 * ((x * 0.15).sin() * (y * 0.12).cos())) as u8;
        [v, v, v]
    }).collect();
    let l = Lens { k1: 0.05, ..Default::default() };
    let once = camimg::undistort(&src, w, h, &l);
    // A point of the result samples the source at its distorted place: check one point by hand.
    let (x, y) = (45usize, 20usize);
    let (xn, yn) = ((x as f64 + 0.5 - 30.0) / f, (y as f64 + 0.5 - 30.0) / f);
    let r2 = xn * xn + yn * yn;
    let (sx, sy) = (xn * (1.0 + l.k1 * r2) * f + 30.0 - 0.5, yn * (1.0 + l.k1 * r2) * f + 30.0 - 0.5);
    let expect = src[(sy.round() as usize * w + sx.round() as usize) * 3] as i32;
    assert!((once[(y * w + x) * 3] as i32 - expect).abs() <= 12, "{} vs {expect}", once[(y * w + x) * 3]);
}

#[test]
fn homography_from_reference_points_recovers_the_transform() {
    let mm = |u: f64, v: f64| (u * 0.5 + 10.0, v * 0.5 + 20.0); // a scale and a shift
    let pts = |list: &[(f64, f64)]| -> Vec<RefPoint> { list.iter().map(|p| RefPoint { px: *p, mm: mm(p.0, p.1) }).collect() };
    // Three points: an affine map is enough.
    let h3 = camimg::homography(&pts(&[(0.0, 0.0), (640.0, 0.0), (0.0, 480.0)])).unwrap();
    let (x, y) = camimg::apply(&h3, (320.0, 240.0)).unwrap();
    assert!((x - 170.0).abs() < 1e-6 && (y - 140.0).abs() < 1e-6, "{x},{y}");
    // Four points of a perspective view: the map sends them where they should go and recovers the others.
    let persp = |u: f64, v: f64| {
        let w = 1.0 + 0.0008 * u + 0.0005 * v;
        ((0.3 * u + 0.05 * v + 7.0) / w, (0.02 * u + 0.28 * v + 3.0) / w)
    };
    let corners = [(0.0, 0.0), (640.0, 0.0), (640.0, 480.0), (0.0, 480.0)];
    let four: Vec<RefPoint> = corners.iter().map(|p| RefPoint { px: *p, mm: persp(p.0, p.1) }).collect();
    let h = camimg::homography(&four).unwrap();
    for q in [(100.0, 100.0), (320.0, 240.0), (600.0, 50.0)] {
        let (x, y) = camimg::apply(&h, q).unwrap();
        let (ex, ey) = persp(q.0, q.1);
        assert!((x - ex).abs() < 1e-6 && (y - ey).abs() < 1e-6, "{q:?}: {x},{y} vs {ex},{ey}");
    }
    // More than four points are fitted in the least-squares sense (a point that is a little off does not break it).
    let mut five = four.clone();
    five.push(RefPoint { px: (320.0, 240.0), mm: { let p = persp(320.0, 240.0); (p.0 + 0.05, p.1 - 0.05) } });
    let h5 = camimg::homography(&five).unwrap();
    let (x, y) = camimg::apply(&h5, (100.0, 100.0)).unwrap();
    let (ex, ey) = persp(100.0, 100.0);
    assert!((x - ex).abs() < 0.2 && (y - ey).abs() < 0.2);
    // Not enough or degenerate points give nothing.
    assert!(camimg::homography(&pts(&[(0.0, 0.0), (10.0, 0.0)])).is_none());
    assert!(camimg::homography(&pts(&[(0.0, 0.0), (10.0, 10.0), (20.0, 20.0)])).is_none(), "three points on a line");
    assert!(camimg::homography(&[RefPoint { px: (f64::NAN, 0.0), mm: (0.0, 0.0) }; 4]).is_none());
    // The corners of the overlay from the map, and reference points from the corners, are each other's inverse.
    let c = camimg::corners(&h, 640.0, 480.0).unwrap();
    for (p, q) in c.iter().zip(corners.iter().map(|q| persp(q.0, q.1))) {
        assert!(p.dist(Pt::new(q.0, q.1)) < 1e-6);
    }
    let back = camimg::points_from_corners(&c, 640.0, 480.0);
    let h2 = camimg::homography(&back).unwrap();
    let (x, y) = camimg::apply(&h2, (250.0, 90.0)).unwrap();
    let (ex, ey) = persp(250.0, 90.0);
    assert!((x - ex).abs() < 1e-6 && (y - ey).abs() < 1e-6);
}

#[test]
fn cameras_of_a_device_migrate_validate_and_travel_with_the_profile() {
    // The single camera of an older profile becomes the first camera.
    let mut v: serde_json::Value = serde_json::from_str(OLD_PROFILE).unwrap();
    let d: Device = serde_json::from_value(v.clone()).unwrap();
    assert_eq!(d.cameras.len(), 1);
    assert_eq!((d.cameras[0].name.as_str(), d.cameras[0].rotation, d.cameras[0].enabled), ("Camera 1", 2, true));
    assert_eq!(d.stream_url(), Some("http://pi.local:8080/stream"));
    // No camera in the profile: none.
    v["camera_url"] = "".into();
    assert!(serde_json::from_value::<Device>(v).unwrap().cameras.is_empty());
    // Only the active, enabled and valid camera streams.
    let mut d = Device::default();
    d.cameras = vec![CameraSource::url("A", "http://a/x", 0), CameraSource::url("B", "https://b/y.mjpg", 1)];
    assert_eq!(d.stream_url(), Some("http://a/x"));
    d.active_camera = 1;
    assert_eq!((d.stream_url(), d.camera_source().unwrap().rotation), (Some("https://b/y.mjpg"), 1));
    d.cameras[1].enabled = false;
    assert_eq!(d.stream_url(), None);
    d.cameras[1].enabled = true;
    d.cameras[1].kind = CameraKind::Url("rtsp://b/y".into());
    assert_eq!(d.stream_url(), Some("rtsp://b/y"), "an RTSP stream is an address too (ffmpeg reads it)");
    d.cameras[1].kind = CameraKind::Local(0);
    assert!(d.camera_source().is_some() && d.stream_url().is_none(), "a local camera has no address");
    d.active_camera = 9;
    assert!(d.camera_source().is_none());
    // Addresses.
    for ok in ["http://pi.local:8080/stream.mjpg", "https://h/p?q=1", " http://h ", "rtsp://user:pw@192.168.1.5:554/stream1", "rtsps://cam.local/x"] {
        assert!(camimg::valid_url(ok), "{ok}");
    }
    for bad in ["", "rtsp://", "rtsp://a b/c", "http://", "http://:80", "ftp://h", "http://a b", "http://h/\u{7}", "javascript:alert(1)", "pi.local"] {
        assert!(!camimg::valid_url(bad), "{bad}");
    }
    // A profile with cameras round-trips and an import refuses a camera that is on with a bad address.
    d.active_camera = 0;
    d.cameras[1].kind = CameraKind::Url("http://b/y".into());
    d.cameras[0].image = ImageSettings { brightness: 10, contrast: 20, white_balance: Some(5000), denoise: 0.5, prefer_yuyv: true };
    d.cameras[0].lens = Lens { k1: -0.1, p1: 0.001, ..Default::default() };
    d.cameras[0].reference = vec![RefPoint { px: (1.0, 2.0), mm: (3.0, 4.0) }];
    let back = profiles::import_profile(&profiles::export_profile(&d, "v0.0.11")).unwrap();
    assert_eq!(back.cameras, d.cameras);
    let mut bad = serde_json::to_value(&d).unwrap();
    bad["cameras"][0]["kind"] = serde_json::json!({"Url": "ftp://x"});
    assert!(profiles::import_profile(&serde_json::json!({"format": 1, "profile": bad}).to_string()).unwrap_err().contains("camera"));
    let mut wild = serde_json::to_value(&d).unwrap();
    wild["cameras"][0]["image"]["brightness"] = 5000.into();
    assert!(profiles::import_profile(&serde_json::json!({"format": 1, "profile": wild}).to_string()).unwrap_err().contains("camera"));
    let mut lens = serde_json::to_value(&d).unwrap();
    lens["cameras"][0]["lens"]["k1"] = 50.0.into();
    assert!(profiles::import_profile(&serde_json::json!({"format": 1, "profile": lens}).to_string()).is_err());
}

use crate::cutopts::{Perforation, TabMode, Tabs};

fn gcode_of(layer_edit: impl FnOnce(&mut Layer)) -> String {
    let mut d = Document::default();
    d.add(0, Kind::Rect { w: 100.0, h: 100.0 }, Xf::translate(20.0, 20.0));
    layer_edit(&mut d.layers[0]);
    gcode::generate(&d).gcode
}

fn count(g: &str, word: &str) -> usize {
    g.lines().filter(|l| l.starts_with(word)).count()
}

#[test]
fn kerf_perforation_and_tabs_change_the_outline_only_when_asked() {
    let plain = gcode_of(|_| {});
    // Defaults write exactly what they always did.
    assert_eq!(plain, gcode_of(|l| l.tabs.size = 9.0), "options that are not switched on change nothing");
    // Tabs: four gaps in the 400 mm outline, one every 100 mm; the laser is off over each.
    let tabs = gcode_of(|l| l.tabs = Tabs { enabled: true, size: 2.0, spacing: 100.0, ..Default::default() });
    assert!(count(&tabs, "G0") >= count(&plain, "G0") + 4, "{} vs {}", count(&tabs, "G0"), count(&plain, "G0"));
    assert!(count(&tabs, "G1") > count(&plain, "G1"));
    // A weak cut over the tabs keeps the laser on with less power.
    let weak = gcode_of(|l| l.tabs = Tabs { enabled: true, size: 2.0, spacing: 100.0, cut_power: 25.0, ..Default::default() });
    assert!(weak.contains("S250"), "25 % of S1000 over the tabs");
    // Perforation: many short pieces.
    let perf = gcode_of(|l| l.perforation = Perforation { enabled: true, cut: 2.0, skip: 2.0 });
    assert!(count(&perf, "G0") > 90, "{}", count(&perf, "G0"));
    // Kerf: the outline moves out, so the head goes beyond the square.
    let kerf = gcode_of(|l| l.kerf = 0.5);
    let max_x = |g: &str| g.lines().filter_map(|l| l.split_whitespace().find_map(|w| w.strip_prefix('X')).and_then(|v| v.parse::<f64>().ok())).fold(f64::MIN, f64::max);
    assert!(max_x(&kerf) > max_x(&plain) + 0.3, "{} vs {}", max_x(&kerf), max_x(&plain));
}

#[test]
fn the_new_layer_options_travel_with_the_layer_and_old_files() {
    let mut l = Layer::new(0);
    l.kerf = 0.2;
    l.perforation = Perforation { enabled: true, cut: 1.0, skip: 0.5 };
    l.tabs = Tabs { enabled: true, mode: TabMode::PerShape, per_shape: 3, ..Default::default() };
    let back: Layer = serde_json::from_str(&serde_json::to_string(&l).unwrap()).unwrap();
    assert_eq!(back, l);
    // A layer saved before they existed gets the defaults.
    let mut v = serde_json::to_value(Layer::new(1)).unwrap();
    for k in ["kerf", "perforation", "tabs"] {
        v.as_object_mut().unwrap().remove(k);
    }
    let old: Layer = serde_json::from_value(v).unwrap();
    assert_eq!((old.kerf, old.perforation.enabled, old.tabs.enabled), (0.0, false, false));
    assert!(!old.has_cut_options());
}

fn image_shape(doc: &mut Document, layer: usize, x: f64) -> u64 {
    let im = ImageData { name: "t".into(), px_w: 4, px_h: 4, gray: vec![0; 16], w: 8.0, h: 8.0, invert: false };
    doc.add(layer, Kind::Image(im), Xf::translate(x, 10.0))
}

#[test]
fn a_layer_of_images_is_in_the_image_mode_and_goes_back_when_the_images_leave() {
    let mut d = Document::default();
    image_shape(&mut d, 3, 10.0);
    assert!(d.is_image_layer(3) && !d.is_image_layer(4));
    assert!(d.normalize_image_layers());
    assert_eq!(d.layers[3].mode, LayerMode::Image);
    assert!(!d.normalize_image_layers(), "nothing more to do");
    // Setting another mode does not stick.
    d.layers[3].mode = LayerMode::Fill;
    d.normalize_image_layers();
    assert_eq!(d.layers[3].mode, LayerMode::Image);
    // An outline on the same layer: the layer is mixed, so it is no longer an image layer and goes back to Line.
    d.add(3, Kind::Rect { w: 5.0, h: 5.0 }, Xf::IDENTITY);
    assert!(!d.is_image_layer(3));
    d.normalize_image_layers();
    assert_eq!(d.layers[3].mode, LayerMode::Line);
    // A mixed layer keeps the mode the user gave it.
    d.layers[3].mode = LayerMode::FillAndLine;
    assert!(!d.normalize_image_layers());
    assert_eq!(d.layers[3].mode, LayerMode::FillAndLine);
    // Old files read as before.
    let v = serde_json::to_string(&LayerMode::Image).unwrap();
    assert_eq!(serde_json::from_str::<LayerMode>(&v).unwrap(), LayerMode::Image);
}

#[test]
fn image_layers_burn_only_the_images_and_may_scan_at_an_angle() {
    let mut d = Document::default();
    image_shape(&mut d, 0, 10.0);
    d.normalize_image_layers();
    let flat = gcode::generate(&d).gcode;
    assert!(flat.contains("G1"), "the image is engraved");
    // With a scan angle the rows run at that angle: moves are no longer along X only.
    d.layers[0].angle = 45.0;
    let turned = gcode::generate(&d).gcode;
    assert_ne!(flat, turned);
    // Share of engraving moves that change Y: rows along X keep Y, turned rows change it with every move.
    let y_changes = |g: &str| {
        let ys: Vec<f64> = g.lines().filter(|l| l.starts_with("G1")).filter_map(|l| l.split_whitespace().find_map(|w| w.strip_prefix('Y')).and_then(|v| v.parse::<f64>().ok())).collect();
        ys.windows(2).filter(|w| (w[0] - w[1]).abs() > 1e-6).count() as f64 / ys.len().max(1) as f64
    };
    assert!(y_changes(&turned) > 0.8 && y_changes(&flat) < 0.6, "{} vs {}", y_changes(&turned), y_changes(&flat));
    // The same image engraved at angle 0 is byte for byte what it was.
    d.layers[0].angle = 0.0;
    assert_eq!(gcode::generate(&d).gcode, flat);
    // Vector shapes on a layer in the Image mode are not burnt.
    let mut v = Document::default();
    v.add(1, Kind::Rect { w: 10.0, h: 10.0 }, Xf::IDENTITY);
    v.layers[1].mode = LayerMode::Image;
    assert!(!gcode::generate(&v).gcode.contains("G1 X"), "{}", gcode::generate(&v).gcode);
}

#[test]
fn the_camera_view_is_stored_with_the_device_and_checked_on_import() {
    let mut d = Device::default();
    assert_eq!(d.camera_view, CameraView::default());
    d.camera_view = CameraView { mode: 1, zoom: 2.5, float_pos: Some([10.0, 20.0]), float_size: Some([640.0, 480.0]), win_pos: Some([-300.0, 40.0]), win_size: Some([800.0, 600.0]) };
    let back = profiles::import_profile(&profiles::export_profile(&d, "v0.1.2")).unwrap();
    assert_eq!(back.camera_view, d.camera_view);
    // Older profiles have the default view.
    let old: Device = serde_json::from_str(OLD_PROFILE).unwrap();
    assert_eq!(old.camera_view, CameraView::default());
    for bad in [CameraView { mode: 4, ..Default::default() }, CameraView { zoom: 0.2, ..Default::default() }, CameraView { zoom: f32::NAN, ..Default::default() }, CameraView { float_size: Some([5.0, 5.0]), ..Default::default() }, CameraView { win_pos: Some([f32::INFINITY, 0.0]), ..Default::default() }] {
        let mut x = Device::default();
        x.camera_view = bad;
        assert!(profiles::import_profile(&profiles::export_profile(&x, "v0.1.2")).is_err());
    }
}

#[test]
fn the_description_is_saved_in_the_project_and_old_projects_have_none() {
    let mut d = Document::default();
    d.description = "Birch plywood 3 mm\nair assist on".into();
    let back = Document::from_json(&d.to_json()).unwrap();
    assert_eq!(back.description, d.description);
    let mut v: serde_json::Value = serde_json::from_str(&d.to_json()).unwrap();
    v.as_object_mut().unwrap().remove("description");
    assert_eq!(Document::from_json(&v.to_string()).unwrap().description, "");
}

#[test]
fn every_device_has_a_material_and_old_profiles_get_the_default_one() {
    let d = Device::default();
    assert_eq!((d.materials.len(), d.active_material, d.materials[0].name.as_str()), (1, 0, materials::DEFAULT_MATERIAL));
    let old: Device = serde_json::from_str(OLD_PROFILE).unwrap();
    assert_eq!(old.materials.len(), 1);
    // A bad index is brought back, an empty list refilled.
    let mut x = old.clone();
    x.materials.clear();
    x.active_material = 7;
    x.ensure_materials();
    assert_eq!((x.materials.len(), x.active_material), (1, 0));
}

#[test]
fn the_layers_follow_the_selected_material_and_the_material_follows_the_layers() {
    let mut d = Document::default();
    // A layer the design uses joins the material, with the settings it has (the defaults).
    d.add(2, Kind::Rect { w: 10.0, h: 10.0 }, Xf::IDENTITY);
    assert!(d.sync_material());
    assert_eq!(d.device.materials[0].layers.len(), 1);
    assert!(d.device.materials[0].layer(2).unwrap().settings.same_settings(&Layer::new(2)));
    assert!(!d.sync_material(), "nothing more to do");
    // An edit of the layer is written into the material.
    d.layers[2].speed = 7.0;
    d.layers[2].power = 55.0;
    assert!(d.sync_material());
    assert_eq!(d.device.materials[0].layer(2).unwrap().settings.speed, 7.0);
    // A layer that is not used joins when it is changed.
    d.layers[9].passes = 3;
    d.sync_material();
    assert_eq!(d.device.materials[0].layer(9).unwrap().settings.passes, 3);
    // A second material starts empty and is edited on its own.
    let second = d.add_material("Acrylic");
    assert_eq!((second, d.device.material().name.as_str()), (1, "Acrylic"));
    d.layers[2].speed = 20.0;
    d.sync_material();
    assert_eq!(d.device.materials[0].layer(2).unwrap().settings.speed, 7.0, "the first one is untouched");
    assert_eq!(d.device.materials[1].layer(2).unwrap().settings.speed, 20.0);
    // Choosing the first one puts its settings into the layers.
    d.select_material(0);
    assert_eq!((d.layers[2].speed, d.layers[2].power, d.layers[9].passes), (7.0, 55.0, 3));
    // Names of the layers, their colours and switches are not taken from a material.
    d.layers[2].name = "Mine".into();
    d.layers[2].visible = false;
    d.select_material(1);
    assert_eq!((d.layers[2].name.as_str(), d.layers[2].visible, d.layers[2].speed), ("Mine", false, 20.0));
    // A material without a layer that the design uses: the layer is added with the default settings.
    d.device.materials[1].layers.retain(|l| l.index != 2);
    d.select_material(1);
    assert!(d.layers[2].same_settings(&Layer::new(2)), "defaults");
    assert!(d.device.materials[1].layer(2).is_some());
    // And a layer that is not used and not in the material goes back to the defaults.
    assert!(d.layers[9].same_settings(&Layer::new(9)) || d.device.materials[1].layer(9).is_some());
}

#[test]
fn materials_are_named_renamed_and_deleted_but_one_stays() {
    let mut d = Document::default();
    assert!(!d.delete_material(), "the last one stays");
    d.add_material("Plywood");
    d.add_material("Plywood");
    let names: Vec<&str> = d.device.materials.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, [materials::DEFAULT_MATERIAL, "Plywood", "Plywood (2)"]);
    d.rename_material("Plywood");
    assert_eq!(d.device.material().name, "Plywood (2)", "its own name is free, the other one is taken: it keeps a free name");
    d.rename_material("Leather");
    assert_eq!(d.device.material().name, "Leather");
    assert!(d.delete_material());
    assert_eq!((d.device.materials.len(), d.device.active_material), (2, 1));
    assert_eq!(materials::free_name(&[], "  "), "Material");
}

#[test]
fn materials_travel_with_the_profile_and_with_the_project_and_are_checked_on_import() {
    let mut d = Document::default();
    d.add(1, Kind::Rect { w: 5.0, h: 5.0 }, Xf::IDENTITY);
    d.layers[1].speed = 12.0;
    d.sync_material();
    d.add_material("Acrylic");
    // The project carries the selected material with its layers; the library belongs to the device.
    let back = Document::from_json(&d.to_json()).unwrap();
    assert_eq!(back.device.materials, vec![d.device.materials[1].clone()]);
    assert_eq!(back.device.active_material, 0);
    let ok = profiles::import_profile(&profiles::export_profile(&d.device, "v0.1.2")).unwrap();
    assert_eq!(ok.materials, d.device.materials);
    // A wrong selected material is simply brought back to the first.
    let mut v: serde_json::Value = serde_json::from_str(&profiles::export_profile(&d.device, "v0.1.2")).unwrap();
    v["profile"]["active_material"] = 9.into();
    assert_eq!(profiles::import_profile(&v.to_string()).unwrap().active_material, 0);
    for mutate in [
        |v: &mut serde_json::Value| v["profile"]["materials"][0]["name"] = "".into(),
        |v: &mut serde_json::Value| v["profile"]["materials"][0]["layers"][0]["index"] = 99.into(),
        |v: &mut serde_json::Value| v["profile"]["materials"][0]["layers"][0]["settings"]["speed"] = (-5.0).into(),
    ] {
        let mut v: serde_json::Value = serde_json::from_str(&profiles::export_profile(&d.device, "v0.1.2")).unwrap();
        mutate(&mut v);
        assert!(profiles::import_profile(&v.to_string()).is_err());
    }
}

#[test]
fn a_new_device_comes_with_starter_materials_for_its_laser_and_a_new_material_is_empty() {
    let mut diode = Device::default();
    diode.add_starter_materials();
    let names: Vec<&str> = diode.materials.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, [materials::DEFAULT_MATERIAL, "Plywood 3 mm", "Cardboard", "Leather 2 mm", "Paper"]);
    assert!(diode.materials[0].layers.is_empty(), "the default one is empty");
    for m in &diode.materials[1..] {
        assert_eq!(m.layers.len(), 2);
        assert_eq!((m.layers[0].settings.mode, m.layers[1].settings.mode), (LayerMode::Fill, LayerMode::Line));
        assert!(m.layers.iter().all(|l| l.settings.speed > 0.0 && l.settings.power > 0.0 && l.settings.power <= 100.0));
    }
    let mut co2 = Device::default();
    co2.heads[0].laser = LaserKind::Co2;
    co2.add_starter_materials();
    assert!(co2.materials.iter().any(|m| m.name == "Acrylic 3 mm"));
    assert!(!diode.materials.iter().any(|m| m.name == "Acrylic 3 mm"), "a diode laser does not cut clear acrylic");
    // Changing the laser type of a device that still has its starters gives it those of the other type...
    let mut x = diode.clone();
    x.heads[0].laser = LaserKind::Co2;
    x.add_starter_materials();
    assert_eq!(x.materials, co2.materials);
    // ...but a device whose materials were touched is left alone.
    let mut y = diode.clone();
    y.materials[1].layers[0].settings.power = 77.0;
    y.heads[0].laser = LaserKind::Co2;
    let before = y.materials.clone();
    y.add_starter_materials();
    assert_eq!(y.materials, before);
    // Every starter passes the check of an imported profile.
    assert!(profiles::import_profile(&profiles::export_profile(&co2, "v0.1.2")).is_ok());
    // A new material is empty: the layers go back to the defaults and the used ones join with them.
    let mut d = Document::default();
    d.add(1, Kind::Rect { w: 5.0, h: 5.0 }, Xf::IDENTITY);
    d.layers[1].speed = 3.0;
    d.sync_material();
    let i = d.add_material("Felt");
    assert_eq!(d.device.materials[i].layers.len(), 1, "only the layer in use");
    assert!(d.layers[1].same_settings(&Layer::new(1)), "reset to the defaults");
    assert_eq!(d.device.materials[0].layer(1).unwrap().settings.speed, 3.0, "the first material keeps its own");
}

#[test]
fn one_camera_is_on_at_a_time_and_a_new_one_without_an_address_is_allowed() {
    let mut d = Device::default();
    d.cameras = vec![CameraSource::url("A", "http://a/x", 0), CameraSource::url("B", "http://b/y", 0), CameraSource::url("C", "", 0)];
    d.activate_camera(1);
    assert_eq!((d.active_camera, d.cameras.iter().map(|c| c.enabled).collect::<Vec<_>>()), (1, vec![false, true, false]));
    assert_eq!(d.stream_url(), Some("http://b/y"));
    d.activate_camera(7);
    assert_eq!(d.active_camera, 1, "no such camera");
    // A camera that is on but has no address yet shows nothing and is still a valid profile.
    d.activate_camera(2);
    assert_eq!(d.stream_url(), None);
    assert!(profiles::import_profile(&profiles::export_profile(&d, "v0.1.2")).is_ok());
    // A bad address that is on is still refused.
    d.cameras[2].kind = CameraKind::Url("ftp://x".into());
    assert!(profiles::import_profile(&profiles::export_profile(&d, "v0.1.2")).is_err());
}

#[test]
fn a_job_from_the_current_position_puts_the_top_left_corner_on_the_head_and_ends_there() {
    let mut d = Document::default();
    d.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::translate(50.0, 60.0));
    assert_eq!(d.device.start_from, StartFrom::Home);
    let c = gcode::burn_corner(&d).unwrap();
    assert!((c.x - 50.0).abs() < 1e-6 && (c.y - 60.0).abs() < 1e-6, "{c:?}");
    let head = Pt::new(20.0, 30.0);
    let moved = gcode::placed_at(&d, head);
    let c = gcode::burn_corner(&moved).unwrap();
    assert!((c.x - head.x).abs() < 1e-6 && (c.y - head.y).abs() < 1e-6, "{c:?}");
    // The way back goes to the head, not to the machine's zero.
    d.device.gcode.return_home = true;
    let job = gcode::generate_from(&moved, Some(head));
    let last = job.moves.last().unwrap();
    assert!(last.b.dist(head) < 1e-6 && !last.laser, "{last:?}");
    let home = gcode::generate(&moved);
    assert!(home.moves.last().unwrap().b.dist(moved.device.home_point()) < 1e-6);
    // Nothing burned: the document stays as it is.
    let empty = Document::default();
    assert!(gcode::burn_corner(&empty).is_none());
    assert_eq!(gcode::placed_at(&empty, head).scene.shapes.len(), 0);
}

#[test]
fn start_from_is_saved_with_the_device_and_old_profiles_start_from_home() {
    let mut dev = Device::default();
    dev.start_from = StartFrom::CurrentPosition;
    let back: Device = serde_json::from_str(&serde_json::to_string(&dev).unwrap()).unwrap();
    assert_eq!(back.start_from, StartFrom::CurrentPosition);
    let old = r#"{"name":"x","bed_w":300.0,"bed_h":200.0,"origin":"FrontLeft","s_max":1000.0,"dynamic_power":true,"travel_speed":3000.0,"return_home":true,"baud":115200}"#;
    let d: Device = serde_json::from_str(old).unwrap();
    assert_eq!(d.start_from, StartFrom::Home);
}

#[test]
fn a_project_keeps_the_machine_it_was_made_for_but_not_the_connection_or_cameras() {
    let mut d = Document::default();
    d.device.name = "Garage laser".into();
    d.device.area.bed_w = 410.0;
    d.device.connection.host = "192.168.1.50".into();
    d.device.connection.port = "/dev/secret-port".into();
    d.device.cameras = vec![CameraSource::url("Cam", "http://user:pass@10.0.0.9/stream", 0)];
    d.device.notes = "private notes".into();
    let text = d.to_json();
    for secret in ["192.168.1.50", "secret-port", "user:pass", "private notes"] {
        assert!(!text.contains(secret), "{secret} is in the project file");
    }
    let back = Document::from_json(&text).unwrap();
    assert_eq!(back.device.name, "Garage laser");
    assert_eq!(back.device.area.bed_w, 410.0);
    assert!(back.device.connection.host.is_empty() && back.device.cameras.is_empty());
}

#[test]
fn machines_that_differ_are_reported_in_words_of_the_interface() {
    let a = Machine::of(&Device::default());
    let mut b = a.clone();
    assert!(a.differences(&b).is_empty());
    b.area.bed_w += 100.0;
    b.name = "Another name".into();
    assert_eq!(a.differences(&b), vec!["Work area"]);
    b.controller = Controller::Marlin;
    assert_eq!(a.differences(&b), vec!["Controller", "Work area"]);
}

#[test]
fn the_parts_of_other_programs_survive_opening_and_saving_here() {
    let mut p = Document::default().to_project();
    p.set_section("mill", &serde_json::json!({ "format": 2, "tools": ["6 mm flat"] }));
    let mut v: serde_json::Value = serde_json::from_str(&p.to_json()).unwrap();
    v["future"] = serde_json::json!(true);
    let mut d = Document::from_json(&v.to_string()).unwrap();
    d.add(0, Kind::Rect { w: 5.0, h: 5.0 }, Xf::IDENTITY);
    let v2: serde_json::Value = serde_json::from_str(&d.to_json()).unwrap();
    assert_eq!(v2["sections"]["mill"]["tools"][0], "6 mm flat");
    assert_eq!(v2["future"], true);
    assert_eq!(v2["scene"]["shapes"].as_array().unwrap().len(), 1);
}

#[test]
fn a_project_made_by_another_program_opens_with_the_default_layers() {
    let mut p = base_core::project::Project::default();
    p.scene.add(0, Kind::Rect { w: 5.0, h: 5.0 }, Xf::IDENTITY);
    p.set_section("pen", &serde_json::json!({ "format": 1 }));
    let d = Document::from_json(&p.to_json()).unwrap();
    assert_eq!((d.scene.shapes.len(), d.layers.len(), d.layer_order().len()), (1, crate::LAYER_COUNT, crate::LAYER_COUNT));
    assert!(d.to_json().contains("\"pen\""));
}

#[test]
fn a_project_file_is_saved_again_unchanged() {
    let path = format!("{}/tests/fixtures/every-kind.cre", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(path).unwrap();
    let doc = Document::from_json(&text).unwrap();
    assert_eq!(doc.scene.shapes.len(), 5);
    assert_eq!(doc.group_of(1), vec![1, 2]);
    assert!(doc.shape(5).unwrap().locked);
    assert_eq!(doc.to_json(), text);
}

#[test]
#[ignore]
fn regenerate_the_every_kind_fixture() {
    let path = format!("{}/tests/fixtures/every-kind.cre", env!("CARGO_MANIFEST_DIR"));
    let doc = Document::from_json(&std::fs::read_to_string(&path).unwrap()).unwrap();
    std::fs::write(path, doc.to_json()).unwrap();
}

#[test]
fn a_material_colour_is_saved_when_set_and_old_materials_have_none() {
    use crate::materials::Material;
    let mut m = Material::new("Plywood");
    assert!(m.color.is_none());
    // Without a colour the file is the same as it always was.
    let plain = serde_json::to_string(&m).unwrap();
    assert!(!plain.contains("color"), "{plain}");
    let old: Material = serde_json::from_str(&plain).unwrap();
    assert_eq!(old.color, None);
    // With one it is written and read back.
    m.color = Some([200, 150, 90]);
    let json = serde_json::to_string(&m).unwrap();
    let back: Material = serde_json::from_str(&json).unwrap();
    assert_eq!(back.color, Some([200, 150, 90]));
    // The colour is only shown: the layers of two materials that differ only in it are the same settings.
    let mut a = Material::new("A");
    let mut b = Material::new("A");
    b.color = Some([1, 2, 3]);
    assert_eq!(a.layers, b.layers);
    a.color = b.color;
    assert_eq!(a, b);
}

#[test]
fn a_layer_description_is_saved_comes_with_a_material_and_old_files_have_none() {
    let mut l = Layer::new(3);
    assert!(l.description.is_empty());
    l.description = "cut through, two passes".into();
    // Saved and read back.
    let json = serde_json::to_string(&l).unwrap();
    let back: Layer = serde_json::from_str(&json).unwrap();
    assert_eq!(back.description, "cut through, two passes");
    // A file from before the description existed has none.
    let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
    v.as_object_mut().unwrap().remove("description");
    let old: Layer = serde_json::from_value(v).unwrap();
    assert!(old.description.is_empty());
    // It comes with the settings of a material (the name and the colour stay).
    let mut other = Layer::new(5);
    other.take_settings(&l);
    assert_eq!(other.description, "cut through, two passes");
    assert_eq!(other.name, "C05");
    // It has a limit.
    l.description = "x".repeat(MAX_LAYER_DESCRIPTION + 50);
    l.trim_description();
    assert_eq!(l.description.chars().count(), MAX_LAYER_DESCRIPTION);
}

fn doc_with_materials(names: &[&str]) -> Document {
    use crate::materials::Material;
    let mut d = Document::default();
    d.device.materials = names.iter().map(|n| Material::new(n)).collect();
    d.device.active_material = 0;
    d
}

#[test]
fn materials_are_duplicated_deleted_and_moved_and_the_selected_one_stays_selected() {
    let mut d = doc_with_materials(&["A", "B", "C"]);
    d.device.materials[1].add_layer(4);
    // A copy goes after the original with a free name and becomes the selected one.
    assert_eq!(d.duplicate_material(1), Some(2));
    assert_eq!(d.device.materials[2].name, "B (copy)");
    assert!(d.device.materials[2].layer(4).is_some(), "with its layers");
    assert_eq!(d.device.active_material, 2);
    // Moving keeps the selected material the same one.
    d.device.active_material = 0;
    assert_eq!(d.move_material(0, false), 1);
    assert_eq!(d.device.materials[1].name, "A");
    assert_eq!(d.device.active_material, 1, "the selected material moved with its place");
    assert_eq!(d.move_material(0, true), 0, "the first cannot go up");
    // Deleting before the selected one keeps it selected; deleting the selected one takes the one before it.
    let selected = d.device.materials[d.device.active_material].name.clone();
    assert!(d.delete_material_at(0));
    assert_eq!(d.device.materials[d.device.active_material].name, selected);
    let n = d.device.materials.len();
    assert!(d.delete_material_at(d.device.active_material));
    assert_eq!(d.device.materials.len(), n - 1);
    while d.device.materials.len() > 1 {
        assert!(d.delete_material_at(0));
    }
    assert!(!d.delete_material_at(0), "never the last one");
}

#[test]
fn a_material_gets_and_loses_layers() {
    use crate::materials::Material;
    let mut m = Material::new("M");
    assert!(m.add_layer(3) && m.add_layer(1));
    assert!(!m.add_layer(3), "once");
    assert!(!m.add_layer(LAYER_COUNT), "out of range");
    assert_eq!(m.layers.iter().map(|l| l.index).collect::<Vec<_>>(), vec![1, 3], "in order");
    assert!(m.remove_layer(1) && !m.remove_layer(1));
}

#[test]
fn materials_go_through_a_file_and_a_bad_file_is_refused() {
    use crate::materials::{export_materials, import_materials, Material, MAX_MATERIALS_FILE};
    let mut a = Material::new("Plywood");
    a.color = Some([200, 150, 90]);
    a.add_layer(0);
    let mut l = Layer::new(0);
    l.description = "engrave".into();
    l.speed = 55.0;
    a.layers[0].settings = l;
    let text = export_materials(&[a.clone(), Material::new("Acrylic")], "v0.0.0");
    let back = import_materials(&text).unwrap();
    assert_eq!(back.len(), 2);
    assert_eq!(back[0], a, "everything comes back: the colour, the description, the settings");
    // Not a file of materials, a newer format, no materials, too many, too big.
    assert!(import_materials("{\"a\":1}").is_err() && import_materials("nonsense").is_err());
    assert!(import_materials(&text.replace("lightcreator-materials", "something")).is_err());
    assert!(import_materials(&text.replace("\"format\": 1", "\"format\": 99")).unwrap_err().contains("newer"));
    assert!(import_materials(&export_materials(&[], "v")).is_err());
    let many: Vec<Material> = (0..201).map(|i| Material::new(&format!("m{i}"))).collect();
    assert!(import_materials(&export_materials(&many, "v")).is_err());
    assert!(import_materials(&" ".repeat(MAX_MATERIALS_FILE + 1)).is_err());
    // A number out of range, a name that is empty, a layer twice, a layer out of range.
    let mut bad = a.clone();
    bad.layers[0].settings.power = 1000.0;
    assert!(import_materials(&export_materials(&[bad], "v")).is_err());
    assert!(import_materials(&export_materials(&[Material::new("  ")], "v")).is_err());
    let mut twice = a.clone();
    twice.layers.push(twice.layers[0].clone());
    assert!(import_materials(&export_materials(&[twice], "v")).is_err());
    let mut high = a.clone();
    high.layers[0].index = LAYER_COUNT;
    assert!(import_materials(&export_materials(&[high], "v")).is_err());
    let mut long = a.clone();
    long.layers[0].settings.description = "x".repeat(600);
    assert!(import_materials(&export_materials(&[long], "v")).is_err());
}

#[test]
fn imported_materials_keep_their_names_apart_or_replace_the_ones_with_the_same_name() {
    use crate::materials::{merge_imported, Material};
    let mut list = vec![Material::new("Plywood"), Material::new("Paper")];
    let mut new_ply = Material::new("plywood");
    new_ply.color = Some([1, 1, 1]);
    // Kept apart: a free name.
    assert_eq!(merge_imported(&mut list, vec![new_ply.clone(), Material::new("Felt")], false), (2, 0));
    assert_eq!(list.iter().map(|m| m.name.as_str()).collect::<Vec<_>>(), vec!["Plywood", "Paper", "plywood (2)", "Felt"]);
    // Replaced: the same name (any case) takes the new one.
    assert_eq!(merge_imported(&mut list, vec![new_ply], true), (0, 1));
    assert_eq!(list[0].color, Some([1, 1, 1]));
    assert_eq!(list.len(), 4);
    // The list has a limit.
    let mut full: Vec<Material> = (0..200).map(|i| Material::new(&format!("m{i}"))).collect();
    assert_eq!(merge_imported(&mut full, vec![Material::new("more")], false), (0, 0));
    assert_eq!(full.len(), 200);
}

#[test]
fn illustrator_layers_go_on_the_layers_of_the_program_in_file_order() {
    use base_core::svg::Outline;
    let c = || vec![base_core::bezier::Contour::from_polyline(&base_core::geom::Polyline::new(vec![Pt::new(0.0, 0.0), Pt::new(5.0, 0.0), Pt::new(5.0, 5.0)], true))];
    let o = |layer: Option<&str>| Outline { contours: c(), color: [255, 0, 0], layer: layer.map(|s| s.to_string()) };
    let make = || vec![o(Some("Cut")), o(Some("Engrave")), o(Some("Cut")), o(None)];
    assert_eq!(pdf::layer_count(&make()), 2);
    let mut by_colour = Document::default();
    let ids = pdf::place_outlines(make(), &mut by_colour, false);
    let colour_layer = by_colour.scene.shapes.iter().find(|s| s.id == ids[0]).unwrap().layer;
    assert!(ids.iter().all(|id| by_colour.scene.shapes.iter().find(|s| s.id == *id).unwrap().layer == colour_layer), "all red: one layer");
    let mut by_layers = Document::default();
    let ids = pdf::place_outlines(make(), &mut by_layers, true);
    let layers: Vec<usize> = ids.iter().map(|id| by_layers.scene.shapes.iter().find(|s| s.id == *id).unwrap().layer).collect();
    assert_eq!(&layers[..3], &[0, 1, 0]);
    assert_eq!(layers[3], colour_layer, "content outside of layers still goes by colour");
}

#[test]
fn the_outline_that_wraps_points_is_their_convex_hull() {
    let pts: Vec<Pt> = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0), (5.0, 5.0), (5.0, 0.0), (3.0, 9.0), (0.0, 0.0)].iter().map(|(x, y)| Pt::new(*x, *y)).collect();
    let h = gcode::convex_hull(&pts);
    assert_eq!(h.len(), 4, "{h:?}: the points inside and on the sides are not corners");
    for c in [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)] {
        assert!(h.iter().any(|p| (p.x - c.0).abs() < 1e-9 && (p.y - c.1).abs() < 1e-9), "{c:?} missing in {h:?}");
    }
    // A triangle with a far point makes a quadrilateral; two points stay two.
    assert_eq!(gcode::convex_hull(&[Pt::new(0.0, 0.0), Pt::new(4.0, 0.0), Pt::new(0.0, 3.0), Pt::new(9.0, 9.0)]).len(), 4);
    assert_eq!(gcode::convex_hull(&[Pt::new(0.0, 0.0), Pt::new(1.0, 1.0)]).len(), 2);
    assert!(gcode::convex_hull(&[]).is_empty());
}

#[test]
fn a_frame_along_the_outline_of_two_far_shapes_is_not_their_box() {
    let mut doc = Document::default();
    let a = doc.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::translate(0.0, 0.0));
    let b = doc.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::translate(60.0, 60.0));
    let hull = doc.outline_of(&[a, b]);
    // Two squares on a diagonal: the hull is a hexagon, the box has four corners.
    assert_eq!(hull.len(), 6, "{hull:?}");
    let g = gcode::frame_path_gcode(&hull, &doc.device, 1.0, 25.0);
    assert_eq!(g.matches("G1 ").count(), 6, "six sides, back to the start: {g}");
    // The box version is still the same four sides as before.
    let boxed = gcode::frame_gcode(doc.bounds_of(&[a, b]).unwrap(), &doc.device, 1.0, 25.0);
    assert_eq!(boxed.matches("G1 ").count(), 4, "{boxed}");
}

/// The laser moves of a job, as the list of the rectangles they were burnt on (by the x of the move, each rectangle 10 wide and
/// 100 apart: 0, 100, 200 ...), one entry for each stretch of moves on the same rectangle.
fn burnt_order(job: &gcode::Job) -> Vec<usize> {
    let mut out: Vec<usize> = vec![];
    for m in job.moves.iter().filter(|m| m.laser) {
        let r = ((m.a.x + m.b.x) / 2.0 / 100.0).round() as usize;
        if out.last() != Some(&r) {
            out.push(r);
        }
    }
    out
}

/// Rectangles on layer 0 at x = 0, 100 and 200 added in the order 200, 0, 100.
fn three_rects() -> (Document, [u64; 3]) {
    let mut d = Document::default();
    d.layers[0].mode = LayerMode::Line;
    let far = d.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::translate(200.0, 0.0));
    let near = d.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::translate(0.0, 0.0));
    let mid = d.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::translate(100.0, 0.0));
    (d, [far, near, mid])
}

#[test]
fn one_by_one_burns_all_layers_and_passes_of_an_element_before_the_next_and_the_nearest_element_comes_first() {
    // Two groups, each a rectangle on layer 0 and a circle on layer 1: the pieces of a workpiece.
    let mut d = Document::default();
    d.layers[0].mode = LayerMode::Line;
    d.layers[1].mode = LayerMode::Line;
    d.layers[0].passes = 2;
    for x in [200.0, 0.0] {
        let r = d.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::translate(x, 0.0));
        let c = d.add(1, Kind::Ellipse { w: 4.0, h: 4.0 }, Xf::translate(x + 3.0, 3.0));
        let g = d.scene.new_group_id();
        for s in d.scene.shapes.iter_mut().filter(|s| s.id == r || s.id == c) {
            s.group = Some(g);
        }
    }
    let layers = |g: &str| -> Vec<String> { g.lines().filter(|l| l.starts_with("; layer") || l.starts_with("; pass")).map(|l| l.split(" (").next().unwrap().to_string()).collect() };
    let (l0, l1) = (format!("; layer {}", d.layers[0].name), format!("; layer {}", d.layers[1].name));
    // As before: layer 0 with its two passes, then layer 1.
    let by_layer = gcode::generate(&d);
    assert_eq!(layers(&by_layer.gcode), vec![l0.clone(), "; pass 1/2".into(), "; pass 2/2".into(), l1.clone()]);
    // One by one: the group at x = 0 is nearer to the head than the one at 200, and is done completely first.
    let one = gcode::generate_with(&d, None, &gcode::JobOptions { one_by_one: true });
    let want: Vec<String> = [l0.clone(), "; pass 1/2".into(), "; pass 2/2".into(), l1.clone(), l0, "; pass 1/2".into(), "; pass 2/2".into(), l1].to_vec();
    assert_eq!(layers(&one.gcode), want);
    assert_eq!(burnt_order(&one).first(), Some(&0), "the near group is first: {:?}", burnt_order(&one));
    assert_eq!(burnt_order(&one).last(), Some(&2), "the far group is last");
    // The same work, in another order.
    assert_eq!(one.moves.iter().filter(|m| m.laser).count(), by_layer.moves.iter().filter(|m| m.laser).count());
}

#[test]
fn single_shapes_that_are_in_no_group_are_elements_of_their_own_in_the_order_of_distance() {
    let (d, _) = three_rects();
    let one = gcode::generate_with(&d, None, &gcode::JobOptions { one_by_one: true });
    assert_eq!(burnt_order(&one), vec![0, 1, 2], "nearest first, not the order they were drawn in");
}

#[test]
fn a_job_of_the_selection_burns_only_those_shapes_in_either_order() {
    let (d, [far, near, mid]) = three_rects();
    let burnt = |d: &Document, one_by_one: bool| burnt_order(&gcode::generate_with(d, None, &gcode::JobOptions { one_by_one }));
    assert_eq!(burnt(&d.only_shapes(&[mid]), false), vec![1]);
    assert_eq!(burnt(&d.only_shapes(&[far, near]), true), vec![0, 2]);
    // Nothing selected: nothing burnt (and the job has no laser move).
    assert!(gcode::generate(&d.only_shapes(&[])).moves.iter().all(|m| !m.laser));
    // The layers and the device are kept.
    assert_eq!(d.only_shapes(&[mid]).layers.len(), d.layers.len());
    // The start from the current position counts the corner of what is burnt, not of the whole drawing.
    let placed = gcode::placed_at(&d.only_shapes(&[mid]), Pt::new(0.0, 0.0));
    assert_eq!(burnt(&placed, false), vec![0]);
}
