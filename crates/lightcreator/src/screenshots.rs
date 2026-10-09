//! Renders the README screenshots headlessly (no window, no display needed).
//!
//! `cargo test -p lightcreator --release render_screenshots -- --ignored --nocapture`
//! writes PNG files into `docs/screenshots/`.
use crate::app::{App, Screen, SideTab, Tool};
use crate::i18n::{self, Lang};
use crate::laser::{ConsoleLine, Dir};
use crate::theme::{self, Scheme};
use egui_kittest::Harness;
use lc_core::{Device, ImageData, Kind, Origin, TextData, Units, Xf};
use std::path::PathBuf;

type Shot = Harness<'static, Option<App>>;

fn out_dir() -> PathBuf {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/screenshots");
    std::fs::create_dir_all(&d).expect("create screenshot dir");
    d
}

fn save(h: &mut Shot, name: &str) {
    h.run_steps(5);
    let img = h.render().expect("render");
    let p = out_dir().join(name);
    img.save(&p).expect("write png");
    println!("wrote {}", p.display());
}

fn app(h: &mut Shot) -> &mut App {
    h.state_mut().as_mut().expect("app built")
}

fn console_sample(a: &mut App) {
    let tx = |t: &str| ConsoleLine { dir: Dir::Tx, text: t.into(), poll: false };
    let rx = |t: &str| ConsoleLine { dir: Dir::Rx, text: t.into(), poll: false };
    a.console.clear();
    a.console.push(ConsoleLine::info("Connected to /dev/ttyUSB0 @ 115200"));
    a.console.push(rx("Grbl 1.1h ['$' for help]"));
    for (t, r) in [("$H", "ok"), ("$J=G91 G21 X5 Y0 F3000", "ok"), ("G21 G90", "ok"), ("M4 S0", "ok"), ("G0 X20.000 Y20.000", "ok"), ("G1 X60.000 Y20.000 S600 F1200", "ok"), ("G1 X60.000 Y45.000", "ok"), ("G1 X20.000 Y45.000", "ok"), ("M5", "ok")] {
        a.console.push(tx(t));
        a.console.push(rx(r));
    }
    a.console.push(rx("error:20"));
    a.console.push(ConsoleLine::info("Aborted (soft reset)"));
    a.console_polls = false;
}

#[test]
#[ignore = "writes docs/screenshots; run explicitly"]
fn render_screenshots() {
    let mut h: Shot = Harness::builder().with_size([1280.0, 800.0]).build_ui_state(
        |ui, state: &mut Option<App>| {
            let a = state.get_or_insert_with(|| App::build(ui.ctx(), None, false));
            a.draw(ui);
        },
        None,
    );
    h.run_steps(2);
    let badge = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/badge.svg").canonicalize().expect("example svg");
    {
        let a = app(&mut h);
        a.lang = Lang::En;
        i18n::set_lang(Lang::En);
        a.scheme = Scheme::LightDark;
    }
    // Apply the scheme to the harness' own context.
    let ctx = h.ctx.clone();
    theme::apply(&ctx, Scheme::LightDark);
    {
        let a = app(&mut h);
        let mut diode = Device::default();
        diode.name = "Diode laser 400 × 400".into();
        let mut co2 = Device::default();
        co2.name = "CO2 K40 (300 × 200 mm)".into();
        co2.bed_w = 300.0;
        co2.bed_h = 200.0;
        co2.origin = Origin::BackLeft;
        let mut inch = Device::default();
        inch.name = "Desktop engraver (inches)".into();
        inch.units = Units::Inch;
        inch.bed_w = 17.0 * 25.4;
        inch.bed_h = 12.0 * 25.4;
        a.profiles = vec![diode, co2, inch];
        a.start_sel = 0;
        a.cfg = None;
        a.screen = Screen::Start;
    }
    save(&mut h, "01-start.png");

    {
        let a = app(&mut h);
        a.enter_editor(0);
        a.open_path(badge);
        a.select_all();
        a.preview_on = true;
        a.side_tab = SideTab::Properties;
        a.view.need_fit = true;
        // Keep local paths and ports out of the pictures.
        a.status = "Imported 4 paths from badge.svg".into();
        a.ports = vec!["/dev/ttyUSB0".into()];
        a.doc.device.port = "/dev/ttyUSB0".into();
    }
    h.run_steps(3);
    save(&mut h, "02-editor-properties.png");

    app(&mut h).side_tab = SideTab::Layers;
    save(&mut h, "03-layers.png");

    {
        let a = app(&mut h);
        a.side_tab = SideTab::Console;
        console_sample(a);
    }
    save(&mut h, "04-console.png");

    {
        let a = app(&mut h);
        a.side_tab = SideTab::Device;
        a.open_cfg_edit(0);
        if let Some(c) = a.cfg.as_mut() {
            c.draft.link = lc_core::LinkKind::Tcp;
            c.draft.host = "raspberrypi.local".into();
            c.draft.tcp_port = 3333;
            c.draft.camera_url = "http://raspberrypi.local:8080/stream.mjpg".into();
        }
    }
    save(&mut h, "05-device-config.png");

    {
        let a = app(&mut h);
        a.cfg = None;
        a.side_tab = SideTab::Properties;
        a.scheme = Scheme::Light;
        a.lang = Lang::Pl;
        i18n::set_lang(Lang::Pl);
    }
    theme::apply(&ctx, Scheme::Light);
    save(&mut h, "06-light-polish.png");
    i18n::set_lang(Lang::En);
    theme::apply(&ctx, Scheme::LightDark);
    {
        let a = app(&mut h);
        a.scheme = Scheme::LightDark;
        a.lang = Lang::En;
        // A fresh page: one curved shape for node editing.
        a.doc.shapes.clear();
        a.preview_on = false;
        let id = a.doc.add(2, Kind::Ellipse { w: 120.0, h: 70.0 }, Xf::translate(40.0, 60.0));
        a.sel = vec![id];
        a.tool = Tool::Node;
        a.nodes_prepare();
        a.node_sel = vec![(id, 0, 0), (id, 0, 1)];
        a.side_tab = SideTab::Properties;
        a.status = "Ready".into();
    }
    save(&mut h, "07-nodes.png");

    {
        let a = app(&mut h);
        a.tool = Tool::Select;
        a.doc.shapes.clear();
        a.node_sel.clear();
        let t = TextData { text: "LightCreator\nlaser studio".into(), size: 22.0, bold: false, ..TextData::default() };
        let id = a.doc.add(0, Kind::Text(t), Xf::translate(30.0, 40.0));
        let ring = a.doc.add(5, Kind::Rect { w: 190.0, h: 80.0 }, Xf::translate(20.0, 30.0));
        let _ = ring;
        a.sel = vec![id];
    }
    save(&mut h, "08-text.png");

    {
        let a = app(&mut h);
        a.doc.shapes.clear();
        let (w, hh) = (240u32, 160u32);
        let gray: Vec<u8> = (0..w * hh)
            .map(|i| {
                let (x, y) = ((i % w) as f32 - 120.0, (i / w) as f32 - 80.0);
                let r = (x * x + y * y).sqrt();
                (255.0 - (255.0 * (1.0 - r / 110.0)).clamp(0.0, 255.0)) as u8
            })
            .collect();
        let im = ImageData { name: "photo.png".into(), px_w: w, px_h: hh, gray, w: 96.0, h: 64.0, invert: false };
        a.doc.layers[0].interval = 0.4;
        let id = a.doc.add(0, Kind::Image(im), Xf::translate(60.0, 60.0));
        a.sel = vec![id];
        a.preview_on = true;
        a.side_tab = SideTab::Layers;
        a.active_layer = 0;
    }
    save(&mut h, "09-image.png");

    // Camera overlay: a procedural "wood board" picture, slightly skewed.
    {
        let a = app(&mut h);
        a.doc.shapes.clear();
        a.preview_on = false;
        a.show_overlay = false;
        let (w, hh) = (320usize, 240usize);
        let mut px = Vec::with_capacity(w * hh * 4);
        for y in 0..hh {
            for x in 0..w {
                let g = ((x as f32 * 0.09 + (y as f32 * 0.05).sin() * 3.0).sin() * 0.5 + 0.5) * 0.35;
                let (r, gg, b) = (200.0 - g * 120.0, 150.0 - g * 100.0, 95.0 - g * 70.0);
                px.extend_from_slice(&[r as u8, gg as u8, b as u8, 255]);
            }
        }
        let img = eframe::egui::ColorImage::from_rgba_unmultiplied([w, hh], &px);
        let ctx2 = ctx.clone();
        a.overlay_set_image(&ctx2, img, "Camera");
        if let Some(ov) = &mut a.overlay {
            ov.corners = [lc_core::Pt::new(30.0, 40.0), lc_core::Pt::new(380.0, 25.0), lc_core::Pt::new(365.0, 330.0), lc_core::Pt::new(20.0, 350.0)];
            ov.opacity = 0.9;
        }
        a.overlay_edit = true;
        let id = a.doc.add(0, Kind::Text(TextData { text: "Hello".into(), size: 40.0, ..TextData::default() }), Xf::translate(110.0, 150.0));
        a.sel = vec![id];
        a.show_overlay = true;
        a.status = "Ready".into();
    }
    save(&mut h, "10-camera-overlay.png");

    {
        let a = app(&mut h);
        a.show_overlay = false;
        a.overlay_edit = false;
        a.overlay = None;
        a.show_materials = true;
        a.mat_laser = Some(lc_core::LaserKind::Diode);
        a.mat_sel = Some(1);
    }
    save(&mut h, "11-materials.png");

    // Preview window: several operations, each in its own colour, with travel moves.
    {
        let a = app(&mut h);
        a.show_materials = false;
        a.doc.shapes.clear();
        a.doc.layers[2].mode = lc_core::LayerMode::FillAndLine;
        a.doc.layers[2].interval = 0.5;
        a.doc.layers[5].mode = lc_core::LayerMode::Offset;
        a.doc.layers[5].interval = 1.5;
        a.doc.add(2, Kind::Ellipse { w: 80.0, h: 50.0 }, Xf::translate(30.0, 30.0));
        a.doc.add(5, Kind::Rect { w: 70.0, h: 50.0 }, Xf::translate(140.0, 30.0));
        a.doc.add(1, Kind::Rect { w: 190.0, h: 120.0 }, Xf::translate(25.0, 20.0));
        a.doc.add(0, Kind::Text(TextData { text: "LightCreator".into(), size: 18.0, ..TextData::default() }), Xf::translate(40.0, 110.0));
        a.sel.clear();
        a.touch();
        a.show_preview = true;
        a.pv.show_travel = true;
        a.pv.progress = 0.6;
        a.status = "Ready".into();
    }
    save(&mut h, "12-preview.png");

    // Image import dialog and trace dialog.
    {
        let a = app(&mut h);
        a.show_preview = false;
        a.doc.shapes.clear();
        let (w, hh) = (200u32, 140u32);
        let gray: Vec<u8> = (0..w * hh)
            .map(|i| {
                let (x, y) = ((i % w) as f32 - 100.0, (i / w) as f32 - 70.0);
                let ring = ((x * x + y * y).sqrt() - 45.0).abs() < 14.0;
                let bar = (x + 70.0).abs() < 8.0 && y.abs() < 50.0;
                if ring || bar { 40 } else { 220 }
            })
            .collect();
        let im = ImageData { name: "logo.png".into(), px_w: w, px_h: hh, gray, w: 100.0, h: 70.0, invert: false };
        a.open_import_dialog_with(im);
    }
    save(&mut h, "13-image-import.png");
    {
        let a = app(&mut h);
        a.img_dlg = None;
        let im = ImageData {
            name: "logo.png".into(),
            px_w: 200,
            px_h: 140,
            gray: (0..200u32 * 140).map(|i| { let (x, y) = ((i % 200) as f32 - 100.0, (i / 200) as f32 - 70.0); if (((x * x + y * y).sqrt() - 45.0).abs() < 14.0) || ((x + 70.0).abs() < 8.0 && y.abs() < 50.0) { 40 } else { 220 } }).collect(),
            w: 100.0,
            h: 70.0,
            invert: false,
        };
        let id = a.doc.add(0, Kind::Image(im), Xf::translate(60.0, 60.0));
        a.sel = vec![id];
        a.open_trace();
    }
    save(&mut h, "14-trace.png");
    {
        let a = app(&mut h);
        a.trace_dlg = None;
        a.doc.shapes.clear();
        a.grid_prefs.minor_on = true;
        a.grid_prefs.minor_mm = 5.0;
        a.grid_prefs.minor_color = [0xc8, 0xd8, 0xf0, 0xff];
        a.grid_prefs.main_color = [0x90, 0xa8, 0xd0, 0xff];
        a.grid = 20.0;
        a.show_prefs = true;
        a.side_tab = SideTab::Layers;
        a.active_layer = 2;
        a.show_all_layers = false;
        let l = a.doc.add(2, Kind::Rect { w: 40.0, h: 30.0 }, Xf::translate(40.0, 40.0));
        let _ = l;
        a.doc.add(0, Kind::Ellipse { w: 40.0, h: 30.0 }, Xf::translate(100.0, 40.0));
        a.doc.move_layer(2, true);
        a.doc.move_layer(2, true);
    }
    save(&mut h, "15-grid-layers.png");

    // Automatic shapes and the polygon popup.
    {
        let a = app(&mut h);
        a.show_prefs = false;
        a.doc.shapes.clear();
        a.grid_prefs.minor_on = false;
        a.grid = 10.0;
        a.side_tab = SideTab::Properties;
        let add = |a: &mut App, ct: lc_core::Contour, x: f64, layer: usize| {
            a.doc.add(layer, Kind::Bezier(vec![ct]), Xf::translate(x, 60.0))
        };
        add(a, lc_core::Contour::triangle(60.0, 52.0), 30.0, 2);
        add(a, lc_core::Contour::star(5, 0.382, 60.0, 57.0), 110.0, 5);
        let id = add(a, lc_core::Contour::polygon(9, 60.0, 60.0), 190.0, 1);
        a.sel = vec![id];
        a.tool = Tool::Polygon;
        a.polygon_sides = 9;
        a.show_polygon = true;
    }
    save(&mut h, "16-shapes.png");

    // Camera view as a side tab, rotated.
    {
        let a = app(&mut h);
        a.show_prefs = false;
        a.show_polygon = false;
        a.grid_prefs = crate::prefs::GridPrefs::default();
        a.grid = 10.0;
        a.tool = Tool::Select;
        a.doc.shapes.clear();
        a.doc.device.camera_url = "http://127.0.0.1:9/stream.mjpg".into();
        a.doc.device.camera_rotation = 1;
        a.stream_mode = crate::stream_ui::StreamMode::Tab;
        a.show_stream = true;
        a.side_tab = SideTab::Camera;
        let (w, hh) = (160usize, 90usize);
        let rgb: Vec<u8> = (0..w * hh).flat_map(|i| { let (x, y) = (i % w, i / w); [(x * 255 / w) as u8, (y * 255 / hh) as u8, 140] }).collect();
        let (nw, nh, rot) = crate::camera::rotate_rgb(&rgb, w as u32, hh as u32, 1);
        let img = eframe::egui::ColorImage::from_rgb([nw as usize, nh as usize], &rot);
        a.stream_tex = Some(ctx.load_texture("t", img, eframe::egui::TextureOptions::LINEAR));
        a.status = "Ready".into();
    }
    save(&mut h, "17-camera-tab.png");

    // ser2net sharing dialog (what a Linux user sees).
    {
        let a = app(&mut h);
        a.show_stream = false;
        a.stream_mode = crate::stream_ui::StreamMode::Floating;
        a.side_tab = SideTab::Properties;
        a.open_ser2net();
        if let Some(d) = a.ser2net.as_mut() {
            d.serial_port = "/dev/ttyUSB0".into();
            d.installed = Some("ser2net version 4.6.1".into());
            d.addresses = vec!["192.168.1.50".into()];
            d.version = lc_core::ser2net::Version::V4;
        }
    }
    save(&mut h, "18-ser2net.png");
}

/// Renders every context-menu icon large and small so they can be checked by eye (`target/menu-icons.png`).
#[test]
#[ignore = "visual check only"]
fn render_menu_icon_sheet() {
    use crate::icons::{paint_menu_icon, MenuIcon};
    let all = [
        MenuIcon::FlipH, MenuIcon::FlipV, MenuIcon::RotCw, MenuIcon::RotCcw, MenuIcon::ToFront, MenuIcon::ToBack, MenuIcon::AlignLeft,
        MenuIcon::AlignCenterH, MenuIcon::AlignRight, MenuIcon::AlignTop, MenuIcon::AlignCenterV, MenuIcon::AlignBottom, MenuIcon::CenterOnBed,
    ];
    let mut h = Harness::builder().with_size([560.0, 160.0]).build_ui(move |ui| {
        ui.painter().rect_filled(ui.max_rect(), 0.0, eframe::egui::Color32::from_gray(0x3c));
        for (i, ic) in all.iter().enumerate() {
            let x = 20.0 + i as f32 * 40.0;
            paint_menu_icon(ui.painter(), eframe::egui::Rect::from_center_size(eframe::egui::pos2(x, 40.0), eframe::egui::vec2(32.0, 32.0)), *ic, eframe::egui::Color32::WHITE);
            paint_menu_icon(ui.painter(), eframe::egui::Rect::from_center_size(eframe::egui::pos2(x, 110.0), eframe::egui::vec2(16.0, 16.0)), *ic, eframe::egui::Color32::WHITE);
        }
    });
    h.run_steps(2);
    let img = h.render().expect("render");
    let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/menu-icons.png");
    img.save(&p).expect("write png");
}

/// The work area follows the window size while auto-fit is on, and stays put after a manual zoom.
#[test]
fn work_area_follows_window_resizes() {
    let mut h: Shot = Harness::builder().with_size([1200.0, 800.0]).build_ui_state(
        |ui, state: &mut Option<App>| {
            let a = state.get_or_insert_with(|| App::build(ui.ctx(), None, false));
            a.draw(ui);
        },
        None,
    );
    h.run_steps(2);
    {
        let a = app(&mut h);
        a.profiles = vec![Device::default()];
        a.enter_editor(0);
    }
    h.run_steps(3);
    let big = app(&mut h).view.zoom;
    h.set_size(eframe::egui::vec2(700.0, 520.0));
    h.run_steps(3);
    let small = app(&mut h).view.zoom;
    assert!(small < big * 0.8, "zoom shrinks with the window: {big} -> {small}");
    h.set_size(eframe::egui::vec2(1400.0, 900.0));
    h.run_steps(3);
    let bigger = app(&mut h).view.zoom;
    assert!(bigger > big, "zoom grows with the window: {big} -> {bigger}");
    // After a manual zoom the view is left alone.
    {
        let a = app(&mut h);
        a.view.auto_fit = false;
        a.view.zoom = 3.0;
    }
    h.set_size(eframe::egui::vec2(900.0, 600.0));
    h.run_steps(3);
    assert_eq!(app(&mut h).view.zoom, 3.0);
}
