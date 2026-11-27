//! The performance test of the program on the shared test project of baseCreator (`testdata/perf/perf-block.cre`: a block with six
//! faces of drawings and 5000 objects beside the work areas). Run before every release by `scripts/perf.sh` of baseCreator, which
//! keeps the results (`testdata/perf/RESULTS.md`):
//! `cargo test --release -p lightcreator perf_ -- --ignored --nocapture --test-threads=1`.
use crate::app::{App, Screen};
use eframe::egui;
use std::time::Instant;

/// The test project: `CREATOR_PERF_FILE`, or the one of the baseCreator beside this repository.
fn perf_file() -> std::path::PathBuf {
    std::env::var_os("CREATOR_PERF_FILE").map(Into::into).unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../baseCreator/testdata/perf/perf-block.cre"))
}

/// One result, as `scripts/perf.sh` reads it.
fn report(what: &str, ms: f64, detail: &str) {
    println!("PERF\tLightCreator\t{}\t{what}\t{ms:.0}\t{detail}", crate::app::APP_VERSION);
}

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}

/// The settings of the laser layers of the test project: lines, fills, an offset fill and both (written into its `laser` section).
#[test]
#[ignore = "makes the performance test project; run by hand"]
fn perf_prepare_the_project() {
    let p = perf_file();
    let mut a = App::build(&egui::Context::default(), None, false);
    assert!(a.open_path(p.clone()), "the test project opens");
    for (i, l) in a.doc.layers.iter_mut().enumerate().take(13) {
        l.output = true;
        l.visible = true;
        l.mode = match i {
            1 | 6 => lc_core::LayerMode::Fill,
            3 => lc_core::LayerMode::Offset,
            4 => lc_core::LayerMode::FillAndLine,
            _ => lc_core::LayerMode::Line,
        };
        l.interval = if l.mode == lc_core::LayerMode::Line { l.interval } else { 0.3 };
    }
    std::fs::write(&p, a.doc.to_json()).expect("written");
}

#[test]
#[ignore = "performance; run by scripts/perf.sh of baseCreator"]
fn perf_open_gcode_and_preview() {
    let p = perf_file();
    let ctx = egui::Context::default();
    let mut a = App::build(&ctx, None, false);
    let t = Instant::now();
    assert!(a.open_path(p), "the test project opens");
    report("open", ms(t), &format!("{} shapes on face A", a.doc.scene.shapes.len()));
    a.screen = Screen::Editor;
    let faces = a.doc.solid().map_or(1, |s| s.count());
    // The G-code of every layer, of every face.
    let t = Instant::now();
    let mut bytes = 0;
    for i in 0..faces {
        a.show_face(i);
        bytes += a.gcode_text_now().map_or(0, |g| g.len());
    }
    report("gcode", ms(t), &format!("{faces} faces, {} kB", bytes / 1024));
    // The preview of every face: the job and a frame of its tab.
    let t = Instant::now();
    a.show_preview = true;
    for i in 0..faces {
        a.show_face(i);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            a.preview_window(ui.ctx());
            egui::CentralPanel::no_frame().show(ui, |ui| a.preview_work_panel(ui));
        });
        out.textures_delta.clear();
    }
    report("preview", ms(t), &format!("{faces} faces"));
}
