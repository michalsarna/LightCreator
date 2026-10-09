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
}
