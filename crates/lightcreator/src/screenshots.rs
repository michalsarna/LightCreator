//! Renders the README screenshots headlessly (no window, no display needed).
//!
//! `cargo test -p lightcreator --release render_screenshots -- --ignored --nocapture`
//! writes PNG files into `docs/screenshots/`.
use base_ui::clipart_ui::ClipGallery;
use base_ui::editor::EditorImages;
use base_ui::editor::EditorCommands;
use base_ui::editor::EditorHost;
use base_ui::editor::EditorShapeOps;
use crate::app::{App, Screen, SideTab, Tool};
use crate::i18n::{self, Lang};
use crate::laser::{ConsoleLine, Dir};
use crate::theme::{self, Scheme};
use egui_kittest::kittest::Queryable;
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
    a.console.polls = false;
}

#[test]
#[ignore = "writes docs/screenshots; run explicitly"]
fn render_screenshots() {
    let mut h: Shot = Harness::builder().with_accessibility_check(false).with_size([1280.0, 800.0]).build_ui_state(
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
        co2.area.bed_w = 300.0;
        co2.area.bed_h = 200.0;
        co2.axes.origin = Origin::BackLeft;
        let mut inch = Device::default();
        inch.name = "Desktop engraver (inches)".into();
        inch.units = Units::Inch;
        inch.area.bed_w = 17.0 * 25.4;
        inch.area.bed_h = 12.0 * 25.4;
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
        a.ed.view.need_fit = true;
        // Keep local paths and ports out of the pictures.
        a.status = "Imported 4 paths from badge.svg".into();
        a.ports = vec!["/dev/ttyUSB0".into()];
        a.doc.device.connection.port = "/dev/ttyUSB0".into();
    }
    h.run_steps(3);
    save(&mut h, "02-editor-properties.png");

    // The same selection in turn / slant mode: blue round and diamond handles.
    app(&mut h).ed.rotate_mode = true;
    h.run_steps(2);
    save(&mut h, "28-rotate-mode.png");
    app(&mut h).ed.rotate_mode = false;
    h.run_steps(2);

    // The question asked when closing with unsaved work.
    app(&mut h).show_close_dlg = true;
    h.run_steps(3);
    save(&mut h, "29-close-dialog.png");
    app(&mut h).show_close_dlg = false;
    h.run_steps(2);

    app(&mut h).show_about = true;
    h.run_steps(3);
    save(&mut h, "30-about.png");
    app(&mut h).show_about = false;
    h.run_steps(2);

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
            c.draft.connection.link = lc_core::LinkKind::Tcp;
            c.draft.connection.host = "raspberrypi.local".into();
            c.draft.connection.tcp_port = 3333;
            c.draft.cameras = vec![lc_core::CameraSource::url("Camera 1", "http://raspberrypi.local:8080/stream.mjpg", 0)];
        }
    }
    save(&mut h, "05-device-config.png");

    {
        let a = app(&mut h);
        a.cfg = None;
        a.side_tab = SideTab::Device;
    }
    save(&mut h, "27-device-tab.png");

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
        a.doc.scene.shapes.clear();
        a.preview_on = false;
        let id = a.doc.add(2, Kind::Ellipse { w: 120.0, h: 70.0 }, Xf::translate(40.0, 60.0));
        a.ed.sel = vec![id];
        a.ed.tool = Tool::Node;
        a.nodes_prepare();
        a.ed.node_sel = vec![(id, 0, 0), (id, 0, 1)];
        a.side_tab = SideTab::Properties;
        a.status = "Ready".into();
    }
    save(&mut h, "07-nodes.png");

    {
        let a = app(&mut h);
        a.ed.tool = Tool::Select;
        a.doc.scene.shapes.clear();
        a.ed.node_sel.clear();
        let t = TextData { text: "LightCreator\nlaser studio".into(), size: 22.0, bold: false, ..TextData::default() };
        let id = a.doc.add(0, Kind::Text(t), Xf::translate(30.0, 40.0));
        let ring = a.doc.add(5, Kind::Rect { w: 190.0, h: 80.0 }, Xf::translate(20.0, 30.0));
        let _ = ring;
        a.ed.sel = vec![id];
    }
    save(&mut h, "08-text.png");

    {
        let a = app(&mut h);
        a.doc.scene.shapes.clear();
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
        a.ed.sel = vec![id];
        a.preview_on = true;
        a.side_tab = SideTab::Layers;
        a.ed.active_layer = 0;
    }
    save(&mut h, "09-image.png");

    // Camera overlay: a procedural "wood board" picture, slightly skewed.
    {
        let a = app(&mut h);
        a.doc.scene.shapes.clear();
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
        a.ed.sel = vec![id];
        a.show_overlay = true;
        a.status = "Ready".into();
    }
    save(&mut h, "10-camera-overlay.png");

    {
        let a = app(&mut h);
        a.show_overlay = false;
        a.overlay_edit = false;
        a.overlay = None;
    }

    // Preview window: several operations, each in its own colour, with travel moves.
    {
        let a = app(&mut h);
        a.doc.scene.shapes.clear();
        a.doc.layers[2].mode = lc_core::LayerMode::FillAndLine;
        a.doc.layers[2].interval = 0.5;
        a.doc.layers[5].mode = lc_core::LayerMode::Offset;
        a.doc.layers[5].interval = 1.5;
        a.doc.add(2, Kind::Ellipse { w: 80.0, h: 50.0 }, Xf::translate(30.0, 30.0));
        a.doc.add(5, Kind::Rect { w: 70.0, h: 50.0 }, Xf::translate(140.0, 30.0));
        a.doc.add(1, Kind::Rect { w: 190.0, h: 120.0 }, Xf::translate(25.0, 20.0));
        a.doc.add(0, Kind::Text(TextData { text: "LightCreator".into(), size: 18.0, ..TextData::default() }), Xf::translate(40.0, 110.0));
        a.ed.sel.clear();
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
        a.doc.scene.shapes.clear();
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
        a.ed.img_dlg = None;
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
        a.ed.sel = vec![id];
        a.open_trace();
    }
    save(&mut h, "14-trace.png");
    {
        let a = app(&mut h);
        a.ed.trace_dlg = None;
        a.doc.scene.shapes.clear();
        a.ed.grid_prefs.minor_on = true;
        a.ed.grid_prefs.minor_mm = 5.0;
        a.ed.grid_prefs.minor_color = [0xc8, 0xd8, 0xf0, 0xff];
        a.ed.grid_prefs.main_color = [0x90, 0xa8, 0xd0, 0xff];
        a.ed.grid = 20.0;
        a.show_prefs = true;
        a.side_tab = SideTab::Layers;
        a.ed.active_layer = 2;
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
        a.doc.scene.shapes.clear();
        a.ed.grid_prefs.minor_on = false;
        a.ed.grid = 10.0;
        a.side_tab = SideTab::Properties;
        let add = |a: &mut App, ct: lc_core::Contour, x: f64, layer: usize| {
            a.doc.add(layer, Kind::Bezier(vec![ct]), Xf::translate(x, 60.0))
        };
        add(a, lc_core::Contour::triangle(60.0, 52.0), 30.0, 2);
        add(a, lc_core::Contour::star(5, 0.382, 60.0, 57.0), 110.0, 5);
        add(a, lc_core::Contour::heart(60.0, 54.0), 270.0, 2);
        let id = add(a, lc_core::Contour::polygon(9, 60.0, 60.0), 190.0, 1);
        a.ed.sel = vec![id];
        a.ed.tool = Tool::Polygon;
        a.ed.polygon_sides = 9;
        a.ed.show_shape_popup = true;
    }
    save(&mut h, "16-shapes.png");

    // Camera view as a side tab, rotated.
    {
        let a = app(&mut h);
        a.show_prefs = false;
        a.ed.show_shape_popup = false;
        a.ed.grid_prefs = crate::prefs::GridPrefs::default();
        a.ed.grid = 10.0;
        a.ed.tool = Tool::Select;
        a.doc.scene.shapes.clear();
        a.doc.device.cameras = vec![lc_core::CameraSource::url("Camera 1", "http://127.0.0.1:9/stream.mjpg", 1)];
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
        a.stream_mode = crate::stream_ui::StreamMode::Tab;
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

    // Layer names in the colour strip, layer dialog, locked object, light work area in a dark scheme.
    {
        let a = app(&mut h);
        a.show_ser2net_off();
        a.doc.scene.shapes.clear();
        a.ed.grid_prefs = crate::prefs::GridPrefs::default();
        a.ed.grid = 20.0;
        a.show_prefs = false;
        a.show_stream = false;
        a.side_tab = SideTab::Layers;
        a.doc.layers[2].mode = lc_core::LayerMode::Offset;
        a.doc.layers[2].name = "Cut".into();
        a.doc.layers[5].name = "Engrave".into();
        let r = a.doc.add(2, Kind::Rect { w: 70.0, h: 45.0 }, Xf::translate(40.0, 40.0));
        a.doc.add(5, Kind::Ellipse { w: 60.0, h: 45.0 }, Xf::translate(150.0, 40.0));
        a.ed.sel = vec![r];
        a.lock_selection(true);
        a.ed.active_layer = 2;
        a.layer_dlg = Some(2);
        a.status = "Locked 1 object(s).".into();
    }
    save(&mut h, "19-layers-lock.png");

    // Chinese and Hindi interface (fonts from the assets).
    {
        let a = app(&mut h);
        a.show_ser2net_off();
        a.layer_dlg = None;
        a.doc.scene.shapes.clear();
        a.side_tab = SideTab::Properties;
        a.lang = Lang::Zh;
        i18n::set_lang(Lang::Zh);
        a.status = "就绪".into();
    }
    save(&mut h, "20-chinese.png");
    {
        let a = app(&mut h);
        a.lang = Lang::Hi;
        i18n::set_lang(Lang::Hi);
        a.status = "तैयार".into();
    }
    save(&mut h, "21-hindi.png");
    // Greek and Ukrainian: the Greek and Cyrillic letters come from Noto Sans.
    {
        let a = app(&mut h);
        a.lang = Lang::El;
        i18n::set_lang(Lang::El);
        a.status = "Έτοιμο".into();
    }
    save(&mut h, "31-greek.png");
    {
        let a = app(&mut h);
        a.lang = Lang::Uk;
        i18n::set_lang(Lang::Uk);
        a.status = "Готово".into();
    }
    save(&mut h, "32-ukrainian.png");
    i18n::set_lang(Lang::En);

    // Help window and rounded corners.
    {
        let a = app(&mut h);
        a.lang = Lang::En;
        i18n::set_lang(Lang::En);
        a.doc.scene.shapes.clear();
        a.doc.layers[2].name = "C02".into();
        let r = a.doc.add(2, Kind::Rect { w: 80.0, h: 50.0 }, Xf::translate(40.0, 50.0));
        a.ed.sel = vec![r];
        a.ed.fillet_radius = 8.0;
        a.round_corners(8.0);
        let tri = a.doc.add(5, Kind::Bezier(vec![lc_core::Contour::triangle(60.0, 50.0)]), Xf::translate(150.0, 50.0));
        a.ed.sel = vec![tri];
        a.round_corners(6.0);
        a.ed.sel.clear();
        a.ed.show_round = true;
        a.show_guide = true;
        a.status = "Rounded 3 corner(s).".into();
    }
    save(&mut h, "22-round-help.png");
    {
        let a = app(&mut h);
        a.show_guide = false;
        a.ed.show_round = false;
    }
    save(&mut h, "23-rounded-shapes.png");

    // Clipart gallery: shipped pictures (blue) mixed with saved ones (green), and the online search.
    {
        let a = app(&mut h);
        a.show_guide = false;
        a.ed.show_round = false;
        a.doc.scene.shapes.clear();
        let dir = std::env::temp_dir().join(format!("lc-shot-clip-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        a.clip = crate::clipart_ui::new_state(lc_core::clipart::Library::open(dir));
        let pick = |cat: &str, name: &str| crate::clipart_ui::BUILTIN_CLIPART.iter().find(|c| c.0 == cat && c.1 == name).map(|c| String::from_utf8_lossy(c.2).to_string()).unwrap();
        for (cat, n, new_name) in [("Animals", "owl", "Wise owl"), ("Animals", "penguin", "Penguin"), ("Nature", "leaf", "Autumn leaf"), ("Celebrations", "gift", "Gift box")] {
            let _ = a.clip.lib.add(&pick(cat, n), new_name, cat, "https://icon-sets.iconify.design/tabler/x/", "MIT", "Tabler");
        }
        let _ = a.clip.lib.add(&pick("Tools", "hammer"), "Hammer", "Workshop", "https://api.iconify.design/ph/hammer.svg", "MIT", "Phosphor");
        a.clip.reload();
        a.clip.category = Some("Animals".to_string());
        a.side_tab = SideTab::Clipart;
        a.status = "Ready".into();
    }
    save(&mut h, "24-clipart.png");
    {
        let a = app(&mut h);
        a.clip.category = None;
        let ctx2 = ctx.clone();
        let mut o = crate::clipart_ui::OnlineDlg::new(&ctx2, "http://127.0.0.1:9");
        o.query = "cat".into();
        let names = [("cat", "mdi", "Material Design Icons", "Apache-2.0", "Apache 2.0"), ("cat-outline", "mdi", "Material Design Icons", "Apache-2.0", "Apache 2.0"), ("dog", "ph", "Phosphor", "MIT", "MIT"), ("paw", "tabler", "Tabler Icons", "MIT", "MIT"), ("bird", "fa6-solid", "Font Awesome 6 Solid", "CC-BY-4.0", "CC BY 4.0"), ("fish", "mdi", "Material Design Icons", "Apache-2.0", "Apache 2.0")];
        for (n, prefix, set, spdx, title) in names {
            let key = format!("{prefix}:{n}");
            let src = crate::clipart_ui::BUILTIN_CLIPART.iter().find(|c| c.1 == n.replace('-', " ")).or_else(|| crate::clipart_ui::BUILTIN_CLIPART.iter().find(|c| c.1 == "cat"));
            if let Some(c) = src {
                o.thumbs.insert(key, std::sync::Arc::from(c.2.to_vec().into_boxed_slice()));
            }
            o.hits.push(lc_core::clipart::iconify::Hit { prefix: prefix.into(), name: n.into(), set_name: set.into(), license_title: title.into(), license_spdx: spdx.into(), license_url: "https://www.apache.org/licenses/LICENSE-2.0".into(), author: "Pictogrammers".into() });
        }
        o.sel = Some(0);
        o.name = "cat".into();
        o.status = "6 pictures found".into();
        a.clip.online = Some(o);
    }
    save(&mut h, "25-clipart-online.png");

    // The window that adds a device, the material header, the description tab, the options with the web server.
    {
        let a = app(&mut h);
        a.lang = Lang::En;
        i18n::set_lang(Lang::En);
        a.show_ser2net_off();
        a.clip.online = None;
        a.show_prefs = false;
        a.show_stream = false;
        a.doc.scene.shapes.clear();
        a.status = "Ready".into();
        a.start_add_device();
        if let Some(c) = a.cfg.as_mut() {
            c.draft.name = "Diode laser 400 × 400".into();
        }
    }
    save(&mut h, "33-add-device.png");
    {
        let a = app(&mut h);
        a.cfg = None;
        a.add_dev = None;
        a.side_tab = SideTab::Layers;
        a.doc.add(2, Kind::Rect { w: 70.0, h: 45.0 }, Xf::translate(40.0, 40.0));
        a.doc.add(5, Kind::Ellipse { w: 60.0, h: 45.0 }, Xf::translate(150.0, 40.0));
    }
    save(&mut h, "34-material-layers.png");
    {
        let a = app(&mut h);
        a.doc.description = "Birch plywood 3 mm, two passes for the outline.\nFocus 1 mm below the surface.".into();
        a.side_tab = SideTab::Description;
    }
    save(&mut h, "35-description.png");
    {
        let a = app(&mut h);
        a.side_tab = SideTab::Properties;
        a.api_on = true;
        a.show_options = true;
    }
    save(&mut h, "36-options-web-server.png");
    app(&mut h).show_options = false;

    // Two of the coloured schemes.
    for (scheme, name) in [(Scheme::PurpleDark, "37-theme-dark-purple.png"), (Scheme::OrangeLight, "38-theme-light-orange.png")] {
        app(&mut h).scheme = scheme;
        theme::apply(&ctx, scheme);
        app(&mut h).side_tab = SideTab::Layers;
        save(&mut h, name);
    }
    app(&mut h).scheme = Scheme::LightDark;
    theme::apply(&ctx, Scheme::LightDark);
    // The calibration window and "Start from line".
    {
        let a = app(&mut h);
        a.side_tab = SideTab::Properties;
        a.connected = true;
        a.fw.values = [(100, "80.000".to_string()), (101, "80.000".to_string())].into_iter().collect();
        a.cal.open = true;
    }
    save(&mut h, "39-calibration.png");
    {
        let a = app(&mut h);
        a.cal.open = false;
        a.start_lines = "G21 G90\nM5\nG0 X10 Y10\nS600 M4\nF3000\nG1 X50 Y10\nG1 X50 Y40\nG1 X10 Y40\nG1 X10 Y10\nM5".lines().map(String::from).collect();
        a.start_from.line = 7;
        a.start_from.open = true;
    }
    save(&mut h, "40-start-from-line.png");
    {
        let a = app(&mut h);
        a.start_from.open = false;
        a.connected = false;
        // The window that manages materials: the starter materials, the colour of one, the layers of the selected one.
        a.doc.device.materials.extend(lc_core::materials::starter_materials(lc_core::LaserKind::Diode));
        a.doc.device.materials[1].color = Some([196, 150, 96]);
        a.open_materials_window();
        if let Some(d) = a.materials_dlg.as_mut() {
            d.sel = 1;
        }
    }
    save(&mut h, "41-materials-window.png");
    app(&mut h).materials_dlg = None;
}

/// The node-editing bar disappears when another window (the preview) is clicked and returns on a click in the canvas.
#[test]
fn node_bar_hides_when_another_window_is_clicked() {
    use eframe::egui::{Event, Modifiers, PointerButton};
    let mut h: Shot = Harness::builder().with_accessibility_check(false).with_size([1280.0, 800.0]).build_ui_state(
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
        a.cfg = None;
        a.enter_editor(0);
        a.doc.add(0, Kind::Rect { w: 40.0, h: 30.0 }, Xf::translate(60.0, 60.0));
        a.ed.tool = Tool::Node;
        a.ed.sel = vec![a.doc.scene.shapes[0].id];
        a.show_preview = true;
    }
    h.run_steps(4);
    let ctx = h.ctx.clone();
    let press = |h: &Shot, x: f32, y: f32| {
        let pos = eframe::egui::pos2(x, y);
        h.event(Event::PointerMoved(pos));
        h.event(Event::PointerButton { pos, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE });
        h.event(Event::PointerButton { pos, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE });
    };
    assert!(app(&mut h).ed.node_bar_active, "shown while editing nodes");
    // A click inside the preview window hides the bar.
    press(&h, 800.0, 400.0);
    h.run_steps(3);
    assert!(!app(&mut h).ed.node_bar_active, "the bar goes away");
    assert_ne!(ctx.layer_id_at(eframe::egui::pos2(150.0, 122.0)).map(|l| l.id), Some(eframe::egui::Id::unique("node_bar")));
    // A click on the work area brings it back.
    press(&h, 80.0, 400.0);
    h.run_steps(3);
    assert!(app(&mut h).ed.node_bar_active, "and returns");
    // Leaving the node tool resets it for the next time.
    app(&mut h).ed.tool = Tool::Select;
    app(&mut h).ed.node_bar_active = false;
    h.run_steps(2);
    assert!(app(&mut h).ed.node_bar_active);
}

/// Clicking a handle of the selection swaps resizing for turning (corners) and slanting (edges).
#[test]
fn clicking_a_handle_switches_between_resize_and_rotate() {
    use eframe::egui::{pos2, Event, Modifiers, PointerButton};
    let mut h: Shot = Harness::builder().with_accessibility_check(false).with_size([1280.0, 800.0]).build_ui_state(
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
        a.cfg = None;
        a.enter_editor(0);
        a.doc.add(0, Kind::Rect { w: 40.0, h: 40.0 }, Xf::translate(100.0, 100.0));
        a.ed.tool = Tool::Select;
        a.ed.sel = vec![a.doc.scene.shapes[0].id];
        a.ed.view.auto_fit = false;
    }
    h.run_steps(4);
    // Screen position of a point of the design.
    let at = |h: &mut Shot, x: f64, y: f64| {
        let a = app(h);
        a.ed.canvas_origin + a.ed.view.pan + eframe::egui::vec2(x as f32 * a.ed.view.zoom, y as f32 * a.ed.view.zoom)
    };
    let button = |h: &Shot, pos: eframe::egui::Pos2, pressed: bool| h.event(Event::PointerButton { pos, button: PointerButton::Primary, pressed, modifiers: Modifiers::NONE });
    // A plain click on the bottom right corner handle turns the handles into rotate / skew handles.
    let corner = at(&mut h, 140.0, 140.0);
    h.event(Event::PointerMoved(corner));
    button(&h, corner, true);
    button(&h, corner, false);
    h.run_steps(3);
    assert!(app(&mut h).ed.rotate_mode, "first click swaps to rotating");
    // Dragging that corner a quarter turn about the middle rotates the shape.
    let from = corner;
    let to = at(&mut h, 100.0, 140.0);
    h.event(Event::PointerMoved(from));
    button(&h, from, true);
    for k in 1..=5 {
        let t = k as f32 / 5.0;
        h.event(Event::PointerMoved(pos2(from.x + (to.x - from.x) * t, from.y + (to.y - from.y) * t)));
        h.run_steps(1);
    }
    button(&h, to, false);
    h.run_steps(3);
    let xf = app(&mut h).doc.scene.shapes[0].xf;
    assert!(xf.b.abs() > 0.9 && xf.a.abs() < 0.1, "rotated by about 90 degrees: {xf:?}");
    // The top edge handle now slants the shape instead of resizing it.
    let before = app(&mut h).doc.scene.shapes[0].xf;
    let b = app(&mut h).sel_bounds().unwrap();
    let top = at(&mut h, b.center().x, b.min.y);
    let moved = pos2(top.x + 30.0, top.y);
    h.event(Event::PointerMoved(top));
    button(&h, top, true);
    for k in 1..=4 {
        let t = k as f32 / 4.0;
        h.event(Event::PointerMoved(pos2(top.x + 30.0 * t, top.y)));
        h.run_steps(1);
    }
    button(&h, moved, false);
    h.run_steps(3);
    let after = app(&mut h).doc.scene.shapes[0].xf;
    let (det0, det1) = (before.det(), after.det());
    assert!(after != before, "the edge drag changed the shape");
    assert!((det0.abs() - det1.abs()).abs() < 1e-6, "a slant keeps the area: {det0} {det1}");
    // Clicking a handle again goes back to resizing.
    let corner = {
        let b = app(&mut h).sel_bounds().unwrap();
        at(&mut h, b.max.x, b.max.y)
    };
    h.event(Event::PointerMoved(corner));
    button(&h, corner, true);
    button(&h, corner, false);
    h.run_steps(3);
    assert!(!app(&mut h).ed.rotate_mode, "second click swaps back");
}

/// A picture from the gallery placed on the work area and opened in the node tool.
#[test]
#[ignore = "writes docs/screenshots; run explicitly"]
fn render_clipart_node_editing() {
    let mut h: Shot = Harness::builder().with_accessibility_check(false).with_size([1280.0, 800.0]).build_ui_state(
        |ui, state: &mut Option<App>| {
            let a = state.get_or_insert_with(|| App::build(ui.ctx(), None, false));
            a.draw(ui);
        },
        None,
    );
    h.run_steps(2);
    let ctx = h.ctx.clone();
    theme::apply(&ctx, Scheme::LightDark);
    {
        let a = app(&mut h);
        a.profiles = vec![Device::default()];
        a.cfg = None;
        a.enter_editor(0);
        a.clip = crate::clipart_ui::new_state(lc_core::clipart::Library::open(std::env::temp_dir().join(format!("lc-shot-node-{}", std::process::id()))));
        let svg = crate::clipart_ui::BUILTIN_CLIPART.iter().find(|c| c.1 == "owl").map(|c| c.2).unwrap();
        a.clip.size_mm = 160.0;
        a.place_clip(svg, "owl");
        a.ed.view.need_fit = true;
        a.start_node_edit();
        a.side_tab = SideTab::Properties;
        a.status = "Ready".into();
    }
    h.run_steps(4);
    {
        let a = app(&mut h);
        let id = a.ed.sel[0];
        let n = match &a.doc.shape(id).unwrap().kind {
            Kind::Bezier(cs) => cs[0].nodes.len().min(4),
            _ => 0,
        };
        a.ed.node_sel = (0..n).map(|k| (id, 0, k)).collect();
    }
    h.run_steps(3);
    let img = h.render().expect("render");
    let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/screenshots/26-clipart-nodes.png");
    img.save(&p).expect("write png");
}

/// "Fit all objects" frames everything that is on the page, wherever it is.
#[test]
fn fit_all_objects_shows_every_shape() {
    let mut h: Shot = Harness::builder().with_accessibility_check(false).with_size([1200.0, 800.0]).build_ui_state(
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
        // Two small shapes in opposite corners of a big bed.
        a.doc.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::translate(20.0, 20.0));
        a.doc.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::translate(120.0, 90.0));
    }
    h.run_steps(3);
    let before = app(&mut h).ed.view.zoom;
    let ctx = h.ctx.clone();
    app(&mut h).do_act(&ctx, crate::menu::Act::FitAll);
    h.run_steps(3);
    let a = app(&mut h);
    assert!(a.ed.view.zoom > before * 1.5, "zoomed in on the objects: {before} -> {}", a.ed.view.zoom);
    assert!(!a.ed.view.auto_fit);
    let (w, hh) = a.ed.canvas_avail;
    for p in [(20.0f32, 20.0f32), (130.0, 100.0)] {
        let (x, y) = (p.0 * a.ed.view.zoom + a.ed.view.pan.x, p.1 * a.ed.view.zoom + a.ed.view.pan.y);
        assert!(x > 0.0 && x < w && y > 0.0 && y < hh, "({x}, {y}) inside {w} x {hh}");
    }
    // The work-area button brings the whole bed back and keeps it fitted.
    app(&mut h).do_act(&ctx, crate::menu::Act::FitBed);
    h.run_steps(3);
    let a = app(&mut h);
    assert!(a.ed.view.auto_fit && (a.ed.view.zoom - before).abs() < 0.01 * before.max(1.0));
}

/// The Space key really switches to the hand tool while it is held down.
#[test]
fn space_key_pans_while_held() {
    let mut h: Shot = Harness::builder().with_accessibility_check(false).with_size([1000.0, 700.0]).build_ui_state(
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
        a.ed.tool = Tool::Rect;
    }
    h.run_steps(2);
    h.key_down(eframe::egui::Key::Space);
    h.run_steps(2);
    assert_eq!(app(&mut h).ed.tool, Tool::Pan, "hand tool while Space is held");
    h.key_up(eframe::egui::Key::Space);
    h.run_steps(2);
    assert_eq!(app(&mut h).ed.tool, Tool::Rect, "back to the rectangle tool");
}

/// Renders every embedded icon (`target/menu-icons.png`) so the whole set can be checked by eye.
#[test]
#[ignore = "visual check only"]
fn render_menu_icon_sheet() {
    let names = crate::icons::all_names();
    let mut h = Harness::builder().with_accessibility_check(false).with_size([720.0, 330.0]).build_ui(move |ui| {
        egui_extras::install_image_loaders(ui.ctx());
        eframe::egui::Frame::new().fill(eframe::egui::Color32::from_gray(0x3c)).show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            ui.horizontal_wrapped(|ui| {
                for n in &names {
                    let (r, _) = ui.allocate_exact_size(eframe::egui::vec2(36.0, 36.0), eframe::egui::Sense::hover());
                    crate::icons::paint(ui, r.shrink(4.0), n, eframe::egui::Color32::WHITE);
                }
            });
        });
    });
    h.run_steps(8);
    let img = h.render().expect("render");
    let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/menu-icons.png");
    img.save(&p).expect("write png");
}

/// The work area follows the window size while auto-fit is on, and stays put after a manual zoom.
#[test]
fn work_area_follows_window_resizes() {
    let mut h: Shot = Harness::builder().with_accessibility_check(false).with_size([1200.0, 800.0]).build_ui_state(
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
    let big = app(&mut h).ed.view.zoom;
    h.set_size(eframe::egui::vec2(700.0, 520.0));
    h.run_steps(3);
    let small = app(&mut h).ed.view.zoom;
    assert!(small < big * 0.8, "zoom shrinks with the window: {big} -> {small}");
    h.set_size(eframe::egui::vec2(1400.0, 900.0));
    h.run_steps(3);
    let bigger = app(&mut h).ed.view.zoom;
    assert!(bigger > big, "zoom grows with the window: {big} -> {bigger}");
    // After a manual zoom the view is left alone.
    {
        let a = app(&mut h);
        a.ed.view.auto_fit = false;
        a.ed.view.zoom = 3.0;
    }
    h.set_size(eframe::egui::vec2(900.0, 600.0));
    h.run_steps(3);
    assert_eq!(app(&mut h).ed.view.zoom, 3.0);
}

/// The Escape key reaches the editor (the double-press logic itself is tested in `app`).
#[test]
fn escape_key_reaches_the_editor() {
    let mut h: Shot = Harness::builder().with_accessibility_check(false).with_size([1000.0, 700.0]).build_ui_state(
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
        a.ed.tool = Tool::Rect;
    }
    h.run_steps(2);
    h.key_press(eframe::egui::Key::Escape);
    h.run_steps(1);
    assert!(app(&mut h).esc_time > -1.0, "the key press was seen");
}

/// The width of the side panel after showing `tab` in `lang`, from a fresh start (a panel never gets narrower again).
fn side_width(lang: Lang, tab: SideTab) -> f32 {
    let mut h: Shot = Harness::builder().with_accessibility_check(false).with_size([1280.0, 800.0]).build_ui_state(
        |ui, state: &mut Option<App>| {
            let a = state.get_or_insert_with(|| App::build(ui.ctx(), None, false));
            a.draw(ui);
        },
        None,
    );
    h.run_steps(2);
    i18n::set_lang(lang);
    {
        let a = app(&mut h);
        a.lang = lang;
        a.profiles = vec![Device::default()];
        a.enter_editor(0);
        // The material has a colour: the heading of the Layers tab has the most in it then.
        a.doc.device.materials[0].color = Some([200, 150, 90]);
        a.doc.add(0, Kind::Rect { w: 20.0, h: 10.0 }, Xf::IDENTITY);
        a.select_all();
        a.side_tab = tab;
    }
    h.run_steps(6);
    app(&mut h).side_w
}

/// No tab may push the side panel wider than the others, in any language: a wide widget inside would squeeze the canvas.
#[test]
fn no_tab_widens_the_side_panel() {
    let base = side_width(Lang::En, SideTab::Properties);
    let mut bad: Vec<String> = vec![];
    // The widest texts: all 30 languages were checked once; these few keep it so.
    for lang in [Lang::En, Lang::Pl, Lang::De, Lang::Es, Lang::Bg, Lang::Uk] {
        for (t, _) in SideTab::ALL {
            let w = side_width(lang, t);
            if (w - base).abs() >= 1.0 {
                bad.push(format!("{:?} in {:?}: {w:.0} instead of {base:.0}", t, lang));
            }
        }
    }
    i18n::set_lang(Lang::En);
    assert!(bad.is_empty(), "the side panel changes its width:\n{}", bad.join("\n"));
}

fn editor_harness() -> Shot {
    let mut h: Shot = Harness::builder().with_accessibility_check(false).with_size([1280.0, 800.0]).build_ui_state(
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
    h
}

/// Every page of the device window draws, and what was changed on them is saved with the profile.
#[test]
fn device_window_pages_open_and_save() {
    use crate::device_ui::CfgPage;
    let mut h = editor_harness();
    app(&mut h).open_cfg_edit(0);
    for p in CfgPage::visible(lc_core::Controller::Grbl) {
        app(&mut h).cfg.as_mut().unwrap().page = p;
        h.run_steps(3);
    }
    // The firmware page with a table to draw as well.
    {
        let a = app(&mut h);
        a.connected = true;
        a.fw.values = lc_core::grbl_settings::parse_report(&["$30=1000.000".to_string(), "$32=1".to_string(), "$110=3000.000".to_string(), "$130=400.000".to_string()]);
        a.fw.typed.insert(110, "99999999".into());
        a.cfg.as_mut().unwrap().page = CfgPage::Firmware;
    }
    h.run_steps(3);
    {
        let a = app(&mut h);
        a.connected = false;
        a.fw.values.clear();
        a.fw.typed.clear();
    }
    {
        let c = app(&mut h).cfg.as_mut().unwrap();
        c.draft.area.bed_w = 123.0;
        c.draft.heads[0].s_max = 500.0;
        c.page = CfgPage::Laser;
    }
    h.run_steps(3);
    h.get_by_role_and_label(eframe::egui::accesskit::Role::Button, "Save").click();
    h.run_steps(4);
    let a = app(&mut h);
    assert!(a.cfg.is_none(), "the window closes after Save");
    assert_eq!((a.profiles[0].area.bed_w, a.profiles[0].heads[0].s_max), (123.0, 500.0));
    assert_eq!(a.doc.device.area.bed_w, 123.0);
}

#[test]
fn connection_page_only_for_controllers_with_a_link() {
    use crate::device_ui::CfgPage;
    assert!(CfgPage::visible(lc_core::Controller::Grbl).contains(&CfgPage::Connection));
    assert!(!CfgPage::visible(lc_core::Controller::Ruida).contains(&CfgPage::Connection));
}

#[cfg(test)]
mod resize_tests {
    use super::*;
    use eframe::egui::{self, Event, Id, PointerButton, Pos2};

    fn about_rect(h: &Shot) -> eframe::egui::Rect {
        h.ctx.memory(|m| m.area_rect(Id::unique(Some("about_window")))).unwrap_or_else(|| panic!("about window shown; areas: {:?}", h.ctx.memory(|m| m.areas().visible_layer_ids())))
    }

    fn drag(h: &mut Shot, from: Pos2, to: Pos2) {
        let m = egui::Modifiers::NONE;
        h.event(Event::PointerMoved(from));
        h.run_steps(2);
        h.event(Event::PointerButton { pos: from, button: PointerButton::Primary, pressed: true, modifiers: m });
        h.run_steps(2);
        for k in 1..=5 {
            h.event(Event::PointerMoved(from + (to - from) * (k as f32 / 5.0)));
            h.run_steps(1);
        }
        h.event(Event::PointerButton { pos: to, button: PointerButton::Primary, pressed: false, modifiers: m });
        h.run_steps(3);
    }

    #[test]
    fn about_window_can_be_resized_both_ways() {
        let mut h: Shot = Harness::builder().with_accessibility_check(false).with_size([1280.0, 800.0]).build_ui_state(
            |ui, state: &mut Option<App>| {
                let a = state.get_or_insert_with(|| App::build(ui.ctx(), None, false));
                a.draw(ui);
            },
            None,
        );
        h.run_steps(2);
        app(&mut h).enter_editor(0);
        app(&mut h).add_dev = None;
        h.run_steps(2);
        app(&mut h).show_about = true;
        h.run_steps(4);
        let r0 = about_rect(&h);
        // Grow from the bottom-right corner.
        drag(&mut h, r0.right_bottom() - egui::vec2(1.0, 1.0), r0.right_bottom() + egui::vec2(120.0, 90.0));
        let r1 = about_rect(&h);
        assert!(r1.width() > r0.width() + 80.0 && r1.height() > r0.height() + 60.0, "grow: {r0:?} -> {r1:?}");
        // Shrink it again.
        drag(&mut h, r1.right_bottom() - egui::vec2(1.0, 1.0), r1.right_bottom() - egui::vec2(100.0, 70.0));
        let r2 = about_rect(&h);
        assert!(r2.width() < r1.width() - 60.0, "shrink: {r1:?} -> {r2:?}");
    }
}

#[cfg(test)]
mod material_colour_tests {
    use super::*;
    use egui_kittest::kittest::Queryable;

    #[test]
    fn the_colour_of_a_material_is_set_when_it_is_renamed_or_added_and_cleared_again() {
        let mut h: Shot = Harness::builder().with_accessibility_check(false).with_size([1280.0, 800.0]).build_ui_state(
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
            a.side_tab = SideTab::Layers;
            a.mat_edit = Some(crate::materials_ui::MatEdit::Rename("Birch".into(), Some([10, 20, 30])));
        }
        h.run_steps(4);
        h.get_by_role_and_label(eframe::egui::accesskit::Role::Button, "Save").click();
        h.run_steps(4);
        let a = app(&mut h);
        let m = &a.doc.device.materials[a.doc.device.active_material];
        assert_eq!((m.name.as_str(), m.color), ("Birch", Some([10, 20, 30])), "renamed with a colour");
        // A new material takes its colour, and a rename without one clears it.
        a.mat_edit = Some(crate::materials_ui::MatEdit::New("Acrylic".into(), Some([1, 2, 3])));
        h.run_steps(4);
        h.get_by_role_and_label(eframe::egui::accesskit::Role::Button, "Save").click();
        h.run_steps(4);
        let a = app(&mut h);
        let m = &a.doc.device.materials[a.doc.device.active_material];
        assert_eq!((m.name.as_str(), m.color), ("Acrylic", Some([1, 2, 3])), "a new material");
        a.mat_edit = Some(crate::materials_ui::MatEdit::Rename("Acrylic".into(), None));
        h.run_steps(4);
        h.get_by_role_and_label(eframe::egui::accesskit::Role::Button, "Save").click();
        h.run_steps(4);
        let a = app(&mut h);
        assert_eq!(a.doc.device.materials[a.doc.device.active_material].color, None, "cleared");
    }
}

#[cfg(test)]
mod materials_window_tests {
    use super::*;
    use egui_kittest::kittest::Queryable;

    #[test]
    fn the_materials_window_adds_a_layer_copies_and_deletes_a_material() {
        let mut h: Shot = Harness::builder().with_accessibility_check(false).with_size([1280.0, 800.0]).build_ui_state(
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
            a.open_materials_window();
        }
        h.run_steps(4);
        let height = |h: &mut Shot| h.ctx.memory(|m| m.area_rect(eframe::egui::Id::unique(Some("materials_window")))).map(|r| r.height());
        let first = height(&mut h);
        h.run_steps(30);
        let later = height(&mut h);
        assert!(first.is_some() && (first.unwrap() - later.unwrap()).abs() < 1.0, "the window keeps its height: {first:?} then {later:?}");
        assert!(later.unwrap() < 700.0, "and it does not fill the screen: {later:?}");
        let before = app(&mut h).doc.device.materials[0].layers.len();
        h.get_by_label("Add layer").click();
        h.run_steps(4);
        assert_eq!(app(&mut h).doc.device.materials[0].layers.len(), before + 1, "a layer was added to the material");
        // The layer is in the design too, because the material is the selected one.
        let n = app(&mut h).doc.device.materials.len();
        assert!(n >= 1);
        if std::env::var("SHOT").is_ok() {
            let p = std::env::var("SHOT").unwrap();
            h.render().unwrap().save(p).unwrap();
        }
    }
}

#[cfg(test)]
mod plugin_import_window_tests {
    use super::*;
    use egui_kittest::kittest::Queryable;

    /// An importer with a form and a preview: a window opens, the preview is made in the background, and Import puts the shapes
    /// of the form as it was left into the drawing.
    #[test]
    fn an_importer_with_a_form_opens_its_window_and_import_adds_the_shapes() {
        let base = std::env::temp_dir().join(format!("lc-import-window-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let p = base.join("plugins/form-demo");
        std::fs::create_dir_all(&p).unwrap();
        std::fs::write(p.join("plugin.json"), r#"{"id":"form-demo","name":"Form demo","importers":[{"id":"frm","name":"Form","extensions":["frm"],"script":"read.rhai","preview":true,"params":[{"id":"w","label":"Width","kind":"number","default":10}]}]}"#).unwrap();
        std::fs::write(p.join("read.rhai"), "fn read(text, ctx) { rect(0.0, 0.0, ctx.p.w, 5.0) }").unwrap();
        let file = base.join("a.frm");
        std::fs::write(&file, "x").unwrap();
        let mut h: Shot = Harness::builder().with_accessibility_check(false).with_size([1280.0, 800.0]).build_ui_state(
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
            a.plugins = base_ui::plugins_ui::PluginsUi::new("lightcreator", &base);
            assert_eq!(a.try_plugin_import(&file), Some(true));
            assert!(a.doc.scene.shapes.is_empty(), "nothing is added until Import is pressed");
        }
        for _ in 0..200 {
            h.run_steps(2);
            std::thread::sleep(std::time::Duration::from_millis(10));
            if h.query_by_label_contains("10.0 × 5.0").is_some() {
                break;
            }
        }
        h.get_by_label("Import").click();
        h.run_steps(4);
        let a = app(&mut h);
        assert_eq!(a.doc.scene.shapes.len(), 1, "{}", a.status);
        let b = a.doc.scene.shapes[0].bounds().unwrap();
        assert!((b.max.x - b.min.x - 10.0).abs() < 1e-6, "the width of the form: {b:?}");
        let _ = std::fs::remove_dir_all(&base);
    }
}

#[test]
#[ignore = "writes docs/screenshots; run explicitly"]
fn render_gcode_viewer() {
    let mut h: Shot = Harness::builder().with_accessibility_check(false).with_size([1280.0, 800.0]).build_ui_state(
        |ui, state: &mut Option<App>| {
            let a = state.get_or_insert_with(|| App::build(ui.ctx(), None, false));
            a.draw(ui);
        },
        None,
    );
    h.run_steps(2);
    app(&mut h).lang = Lang::En;
    i18n::set_lang(Lang::En);
    let ctx = h.ctx.clone();
    theme::apply(&ctx, Scheme::LightDark);
    {
        let a = app(&mut h);
        a.profiles = vec![Device::default()];
        a.cfg = None;
        assert!(a.enter_editor(0));
    }
    // A flat laser file: a circle and a square with different power.
    let flat = "G21 G90\nG0 X20 Y20\nM4 S1000\nG1 X80 Y20 F1000\nG1 X80 Y80 S400\nG1 X20 Y80\nG1 X20 Y20\nG0 X50 Y30\nG1 X50 Y30 S800\nG2 X50 Y30 I0 J20\nM5\n";
    // A 3D file: a spiral going down.
    let mut spiral = String::from("G21 G90\nG0 Z5\nG0 X30 Y0\nM3 S8000\n");
    for i in 0..400 {
        let a = i as f64 * 0.12;
        spiral += &format!("G1 X{:.3} Y{:.3} Z{:.3} F400\n", 30.0 * a.cos(), 30.0 * a.sin(), 5.0 - i as f64 * 0.04);
    }
    spiral += "G0 Z10\nM5\n";
    let dir = std::env::temp_dir().join("lc-gview-shots");
    std::fs::create_dir_all(&dir).unwrap();
    for (name, text) in [("gcode-viewer-2d.png", flat.to_string()), ("gcode-viewer-3d.png", spiral)] {
        let p = dir.join("job.nc");
        std::fs::write(&p, text).unwrap();
        assert!(app(&mut h).open_gcode(p));
        save(&mut h, name);
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// The tabs of open drawings: three of them, one with unsaved work.
#[test]
#[ignore = "writes docs/screenshots; run explicitly"]
fn render_tabs() {
    let mut h: Shot = Harness::builder().with_accessibility_check(false).with_size([1280.0, 800.0]).build_ui_state(
        |ui, state: &mut Option<App>| {
            let a = state.get_or_insert_with(|| App::build(ui.ctx(), None, false));
            a.draw(ui);
        },
        None,
    );
    h.run_steps(2);
    app(&mut h).lang = Lang::En;
    i18n::set_lang(Lang::En);
    let ctx = h.ctx.clone();
    theme::apply(&ctx, Scheme::LightDark);
    let examples = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    {
        let a = app(&mut h);
        a.profiles = vec![Device::default()];
        a.cfg = None;
        assert!(a.enter_editor(0));
        assert!(a.open_path(examples.join("badge.svg").canonicalize().unwrap()));
        a.select_all();
        a.preview_on = true;
        a.ports = vec!["/dev/ttyUSB0".into()];
        a.doc.device.connection.port = "/dev/ttyUSB0".into();
        // A project of its own in the second tab, and a third tab with a new drawing that is not saved.
        assert!(a.open_path(examples.join("badge.cre").canonicalize().unwrap()));
        a.new_tab();
        a.doc.add(0, lc_core::Kind::Rect { w: 40.0, h: 25.0 }, lc_core::Xf::translate(30.0, 30.0));
        a.touch();
        a.ed.view.need_fit = true;
        // Keep local paths out of the picture.
        a.status = "Ready".into();
    }
    h.run_steps(4);
    save(&mut h, "42-tabs.png");
}

/// The Control tab (position, jog buttons, Home / Unlock / Pause / Resume) of a connected laser, and the Device tab.
#[test]
#[ignore = "writes docs/screenshots; run explicitly"]
fn render_control_tab() {
    let mut h: Shot = Harness::builder().with_accessibility_check(false).with_size([1280.0, 800.0]).build_ui_state(
        |ui, state: &mut Option<App>| {
            let a = state.get_or_insert_with(|| App::build(ui.ctx(), None, false));
            a.draw(ui);
        },
        None,
    );
    h.run_steps(2);
    app(&mut h).lang = Lang::En;
    i18n::set_lang(Lang::En);
    let ctx = h.ctx.clone();
    theme::apply(&ctx, Scheme::LightDark);
    let examples = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    {
        let a = app(&mut h);
        a.profiles = vec![Device::default()];
        a.cfg = None;
        assert!(a.enter_editor(0));
        assert!(a.open_path(examples.join("badge.cre").canonicalize().unwrap()));
        a.select_all();
        a.ports = vec!["/dev/ttyUSB0".into()];
        a.doc.device.connection.port = "/dev/ttyUSB0".into();
        a.connected = true;
        a.machine = ("Idle".into(), 120.0, 85.0);
        a.machine_z = 0.0;
        a.pos.mpos = Some([120.0, 85.0, 0.0]);
        a.pos.wpos = Some([120.0, 85.0, 0.0]);
        a.status = "Connected".into();
        a.side_tab = SideTab::Control;
    }
    h.run_steps(4);
    save(&mut h, "43-control-tab.png");
    app(&mut h).side_tab = SideTab::Device;
    h.run_steps(3);
    save(&mut h, "44-device-tab-machine.png");
}

/// Writes `examples/badge.cre` (the badge as a project) and `examples/badge.gcode` (its G-code).
/// `cargo test -p lightcreator write_examples -- --ignored --nocapture`
#[test]
#[ignore = "writes examples/; run explicitly"]
fn write_examples() {
    let mut h: Shot = Harness::builder().with_accessibility_check(false).with_size([1280.0, 800.0]).build_ui_state(
        |ui, state: &mut Option<App>| {
            let a = state.get_or_insert_with(|| App::build(ui.ctx(), None, false));
            a.draw(ui);
        },
        None,
    );
    h.run_steps(2);
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    {
        let a = app(&mut h);
        a.profiles = vec![Device::default()];
        a.enter_editor(0);
        a.open_path(dir.join("badge.svg").canonicalize().expect("example svg"));
    }
    h.run_steps(3);
    let a = app(&mut h);
    std::fs::write(dir.join("badge.cre"), a.doc.to_json()).expect("write project");
    let job = lc_core::gcode::generate(&a.doc);
    std::fs::write(dir.join("badge.gcode"), a.postprocessed(job.gcode).expect("no postprocessor")).expect("write G-code");
}

/// The bar of the tabs answers to the pointer: "+" makes a tab, a click on a tab shows it, the cross closes it.
#[cfg(test)]
mod tab_click_tests {
    use super::*;
    use eframe::egui::{self, Event, Modifiers, PointerButton};

    fn click(h: &mut Shot, pos: egui::Pos2) {
        h.event(Event::PointerMoved(pos));
        h.run_steps(2);
        h.event(Event::PointerButton { pos, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE });
        h.run_steps(2);
        h.event(Event::PointerButton { pos, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE });
        h.run_steps(3);
    }

    #[test]
    fn the_plus_the_tab_and_the_cross_work() {
        let mut h: Shot = Harness::builder().with_accessibility_check(false).with_size([1280.0, 800.0]).build_ui_state(
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
            a.cfg = None;
            assert!(a.enter_editor(0));
        }
        h.run_steps(3);
        // One tab, "Untitled" (about 86 px wide from x = 8), then the plus.
        click(&mut h, egui::pos2(107.0, 125.0));
        assert_eq!(app(&mut h).tab_count(), 2, "the plus makes a tab");
        assert_eq!(app(&mut h).tabs_state.active, 1);
        // The first tab is shown by a click on its name.
        click(&mut h, egui::pos2(40.0, 125.0));
        assert_eq!(app(&mut h).tabs_state.active, 0);
        // The cross of a tab without unsaved work closes it at once.
        click(&mut h, egui::pos2(81.0, 125.0));
        assert_eq!(app(&mut h).tab_count(), 1, "the cross closes the tab");
    }
}

#[cfg(test)]
mod device_window_tests {
    use super::*;

    #[test]
    fn the_device_window_has_its_pages_in_a_row_and_its_buttons_at_the_foot_and_keeps_its_size() {
        let mut h: Shot = Harness::builder().with_accessibility_check(false).with_size([1280.0, 800.0]).build_ui_state(
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
            a.open_cfg_edit(0);
        }
        h.run_steps(4);
        let rect = |h: &mut Shot| h.ctx.memory(|m| m.area_rect(eframe::egui::Id::unique(Some("device_cfg")))).expect("the window");
        let first = rect(&mut h);
        h.run_steps(30);
        let later = rect(&mut h);
        assert!((first.height() - later.height()).abs() < 1.0, "the window keeps its height: {first:?} then {later:?}");
        assert!(later.height() < 700.0, "and it does not fill the screen: {later:?}");
        if let Ok(p) = std::env::var("SHOT") {
            h.render().unwrap().save(p).unwrap();
        }
    }
}
