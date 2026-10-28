//! End-to-end tests of what LightCreator generates, against the real GRBL 1.1h firmware compiled for the PC (grbl-sim).
//!
//! The harness and the tests of the serial link itself (streaming, feed hold, abort, settings, pauses) are in baseCreator
//! (`base_ui::sim_test`, docs/grbl-sim.md there); these check the laser parts: the jobs of `lc-core`, laser mode, the focus
//! beam. They are `#[ignore]`d because they need the simulator binary: build it with `tools/grbl-sim/build.sh` of baseCreator
//! into `target/grbl-sim` (or point `GRBL_SIM_EXE` at it) and run
//!
//! ```sh
//! cargo test -p lightcreator grbl_sim -- --ignored --test-threads=1
//! ```
use base_ui::link::Cmd;
use base_ui::sim_test::Session;
use lc_core::ControllerSettings;
use lc_core::{gcode, Controller, Document, Kind, LayerMode, Xf};

/// Laser mode on, as the jobs of a laser expect it.
const LASER: &[&str] = &["$32=1"];

/// A small but realistic document: a filled square and a cut circle, as the app would generate them.
fn sample_job(fill_interval: f64) -> Vec<String> {
    let mut d = Document::default();
    d.layers[0].mode = LayerMode::FillAndLine;
    d.layers[0].interval = fill_interval;
    d.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::translate(5.0, 5.0));
    d.add(0, Kind::Ellipse { w: 12.0, h: 12.0 }, Xf::translate(24.0, 6.0));
    crate::laser::clean_gcode(&gcode::generate(&d).gcode)
}

#[test]
#[ignore = "needs grbl-sim: tools/grbl-sim/build.sh of baseCreator"]
fn grbl_sim_streams_a_generated_job() {
    let mut s = Session::connect();
    s.wait_banner();
    s.fast_machine(LASER);
    let job = sample_job(1.0);
    assert!(job.len() > 30, "job too small to be meaningful: {}", job.len());
    let n = job.len();
    s.run_job(job, 240.0);
    // The job ends with a travel back to machine zero (front-left origin, return home on), but not `wait_idle_at(0, 0)`: the
    // simulator now and then stops a millimetre or so away from the target (seen: -1.0 in X) although every line was acknowledged
    // (its two threads share the step counters). The short job below still checks the end position.
    s.wait("machine idle after the job", 60.0, |v| v.state == "Idle" && v.progress == (n, n));
    assert!(s.errors().is_empty(), "GRBL rejected lines: {:?}", s.errors_described());
}

#[test]
#[ignore = "needs grbl-sim: tools/grbl-sim/build.sh of baseCreator"]
fn grbl_sim_starts_with_laser_mode_off_and_accepts_enabling_it() {
    // What the app does after connecting: read `$$`, warn when `$32` is 0, and write `$32=1` once the user agrees.
    let mut s = Session::connect();
    s.wait_banner();
    let dev = lc_core::Device::default();
    let before = Controller::Grbl.parse_settings(&s.read_settings()).check(&dev);
    assert!(before.laser_mode_off, "grbl-sim's default is $32=0");
    s.link.send(Cmd::Line("$32=1".into()));
    let after = Controller::Grbl.parse_settings(&s.read_settings());
    assert_eq!(after.laser_mode, Some(true));
    assert!(s.errors().is_empty(), "{:?}", s.errors_described());
}

#[test]
#[ignore = "needs grbl-sim: tools/grbl-sim/build.sh of baseCreator"]
fn grbl_sim_runs_a_job_with_arcs_and_ends_where_expected() {
    let mut s = Session::connect();
    s.wait_banner();
    s.fast_machine(LASER);
    let mut d = Document::default();
    d.device.gcode.arcs = true;
    d.layers[0].mode = LayerMode::Line;
    d.add(0, Kind::Ellipse { w: 20.0, h: 20.0 }, Xf::translate(30.0, 30.0));
    let job = crate::laser::clean_gcode(&gcode::generate(&d).gcode);
    assert!(job.iter().any(|l| l.starts_with("G2 ") || l.starts_with("G3 ")), "the circle should have become arcs");
    s.run_job(job, 60.0);
    // The job ends with the way back to the machine's zero.
    s.wait_idle_at(0.0, 0.0, 30.0);
    assert!(s.errors().is_empty(), "GRBL rejected lines: {:?}", s.errors_described());
}

#[test]
#[ignore = "needs grbl-sim: tools/grbl-sim/build.sh of baseCreator"]
fn grbl_sim_focus_beam_switches_on_with_m3_and_off_with_m5() {
    let mut s = Session::connect();
    s.wait_banner();
    // Laser mode on, as the focus beam is meant for it.
    s.link.send(Cmd::Line("$32=1".into()));
    let mut dev = lc_core::Device::default();
    dev.heads[0].focus_power = 1.5;
    let on = gcode::focus_gcode(&dev, true).expect("focus lines");
    let off = gcode::focus_gcode(&dev, false).expect("off lines");
    for l in &on {
        s.link.send(Cmd::Line(l.clone()));
    }
    assert!(s.parser_state().contains("M3"), "the beam is on");
    for l in &off {
        s.link.send(Cmd::Line(l.clone()));
    }
    assert!(s.parser_state().contains("M5"), "the beam is off");
    assert!(s.errors().is_empty(), "{:?}", s.errors_described());
}

#[test]
#[ignore = "needs grbl-sim: tools/grbl-sim/build.sh of baseCreator"]
fn grbl_sim_pass_by_pass_and_object_by_object_both_run() {
    // The two orders of the passes (`passes_per_object`) give the same number of cuts in another order; both are accepted.
    let mut s = Session::connect();
    s.wait_banner();
    s.fast_machine(LASER);
    for per_object in [false, true] {
        let mut d = Document::default();
        d.layers[0].mode = LayerMode::Line;
        d.layers[0].passes = 2;
        d.device.gcode.passes_per_object = per_object;
        d.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::translate(5.0, 5.0));
        d.add(0, Kind::Rect { w: 8.0, h: 8.0 }, Xf::translate(25.0, 5.0));
        let job = crate::laser::clean_gcode(&gcode::generate(&d).gcode);
        let n = job.len();
        s.link.send(Cmd::Job(job));
        s.wait("job finished", 60.0, |v| v.progress == (n, n));
        s.wait("idle", 30.0, |v| v.state == "Idle");
    }
    assert!(s.errors().is_empty(), "{:?}", s.errors_described());
}
