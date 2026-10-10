//! End-to-end tests of the GRBL serial worker against the real GRBL 1.1h firmware, compiled for the PC by grbl-sim.
//!
//! They are `#[ignore]`d because they need the simulator binary. Build it once and run them with:
//!
//! ```sh
//! tools/grbl-sim/build.sh
//! cargo test -p lightcreator grbl_sim -- --ignored --test-threads=1
//! ```
//!
//! `LC_GRBL_SIM_EXE` overrides the simulator path. See docs/grbl-sim.md.
use crate::laser::{Cmd, Dir, Evt, LaserLink, Target};
use lc_core::{gcode, Controller, Document, Kind, LayerMode, Xf};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// `tools/grbl-sim/serve.py` on a free port, killed when dropped.
struct Sim {
    child: Child,
    port: u16,
}

impl Sim {
    fn start() -> Sim {
        let root = repo_root();
        let exe = std::env::var("LC_GRBL_SIM_EXE").map(PathBuf::from).unwrap_or_else(|_| root.join("target/grbl-sim/grbl/grbl/sim/grbl_sim.exe"));
        assert!(exe.exists(), "grbl-sim not found at {}; run tools/grbl-sim/build.sh first (or set LC_GRBL_SIM_EXE)", exe.display());
        let mut child = Command::new("python3")
            .arg(root.join("tools/grbl-sim/serve.py"))
            .args(["--port", "0", "--sim"])
            .arg(&exe)
            .stdout(Stdio::piped())
            .spawn()
            .expect("python3 is needed to run tools/grbl-sim/serve.py");
        let mut line = String::new();
        BufReader::new(child.stdout.take().unwrap()).read_line(&mut line).unwrap();
        let port = line.trim().rsplit(':').next().and_then(|p| p.parse().ok()).unwrap_or_else(|| panic!("unexpected bridge output: {line:?}"));
        Sim { child, port }
    }
}

impl Drop for Sim {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Everything the worker reported so far, collected from its event channel.
#[derive(Default)]
struct Seen {
    connected: Option<bool>,
    rx: Vec<String>,
    state: String,
    pos: (f64, f64),
    progress: (usize, usize),
    /// Replies before this index are ignored by `errors()` (start-up output, see `wait_banner`).
    mark: usize,
}

struct Session {
    link: LaserLink,
    seen: Seen,
    _sim: Sim,
}

impl Session {
    /// Connect the worker to a fresh simulator, the same way the app connects to a ser2net laser.
    fn connect() -> Session {
        let sim = Sim::start();
        let link = LaserLink::spawn(eframe::egui::Context::default());
        link.send(Cmd::Connect { target: Target::Tcp { host: "127.0.0.1".into(), port: sim.port }, controller: Controller::Grbl });
        let mut s = Session { link, seen: Seen::default(), _sim: sim };
        s.wait("connected", 5.0, |v| v.connected == Some(true));
        s
    }

    fn pump(&mut self) {
        for e in self.link.poll() {
            match e {
                Evt::Connected(c) => self.seen.connected = Some(c),
                Evt::Traffic(l) if l.dir == Dir::Rx && !l.poll => self.seen.rx.push(l.text),
                Evt::Status { state, x, y } => {
                    self.seen.state = state;
                    self.seen.pos = (x, y);
                }
                Evt::Progress { done, total } => self.seen.progress = (done, total),
                _ => {}
            }
        }
    }

    fn wait(&mut self, what: &str, secs: f32, mut pred: impl FnMut(&Seen) -> bool) {
        let t = Instant::now();
        while t.elapsed().as_secs_f32() < secs {
            self.pump();
            if pred(&self.seen) {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("timed out waiting for {what}; state {:?} at {:?}, progress {:?}, last replies {:?}", self.seen.state, self.seen.pos, self.seen.progress, last(&self.seen.rx, 12));
    }

    /// Wait for GRBL's banner after the soft reset the worker sends on connect. Replies up to here are not checked.
    fn wait_banner(&mut self) {
        self.wait("GRBL banner", 5.0, |v| v.rx.iter().any(|l| l.starts_with("Grbl ")));
        self.seen.mark = self.seen.rx.len();
    }

    /// Fast machine settings so jobs run in a few seconds of (roughly real) simulated time, laser mode on. Steps per
    /// mm stay at GRBL's default: with very few (e.g. `$100=20`) grbl-sim slowed to a crawl on GitHub's runners
    /// while being polled with `?`.
    fn fast_machine(&mut self) {
        let lines = ["$110=30000", "$111=30000", "$120=5000", "$121=5000", "$32=1"];
        let before = self.ok_count();
        for l in lines {
            self.link.send(Cmd::Line(l.into()));
        }
        self.wait("settings accepted", 5.0, |v| v.rx.iter().filter(|l| *l == "ok").count() >= before + lines.len());
    }

    fn ok_count(&mut self) -> usize {
        self.pump();
        self.seen.rx.iter().filter(|l| *l == "ok").count()
    }

    fn run_job(&mut self, lines: Vec<String>, secs: f32) {
        let n = lines.len();
        self.link.send(Cmd::Job(lines));
        self.wait("every job line acknowledged", secs, |v| v.progress == (n, n));
    }

    fn wait_idle_at(&mut self, x: f64, y: f64, secs: f32) {
        self.wait("machine idle at the end position", secs, |v| v.state == "Idle" && (v.pos.0 - x).abs() < 0.01 && (v.pos.1 - y).abs() < 0.01);
    }

    fn errors(&self) -> Vec<&String> {
        self.seen.rx[self.seen.mark..].iter().filter(|l| l.starts_with("error") || l.starts_with("ALARM")).collect()
    }
}

fn last(v: &[String], n: usize) -> &[String] {
    &v[v.len().saturating_sub(n)..]
}

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
#[ignore = "needs grbl-sim: tools/grbl-sim/build.sh"]
fn grbl_sim_keeps_up_with_real_time() {
    // The other tests assume the simulator runs at roughly real speed; this one says so plainly when it does not.
    let mut s = Session::connect();
    s.wait_banner();
    // 10 mm at GRBL's defaults (500 mm/min, 10 mm/s²): 0.83 s up, 0.37 s cruise, 0.83 s down = 2.0 s.
    let t = Instant::now();
    s.run_job(["G21", "G90", "G1 X10 F600"].iter().map(|l| l.to_string()).collect(), 20.0);
    s.wait_idle_at(10.0, 0.0, 60.0);
    let secs = t.elapsed().as_secs_f32();
    eprintln!("grbl-sim: a 2.0 s move took {secs:.1} s");
    assert!(secs < 5.0, "grbl-sim runs {:.1}x slower than real time", secs / 2.0);
}

#[test]
#[ignore = "needs grbl-sim: tools/grbl-sim/build.sh"]
fn grbl_sim_streams_a_generated_job() {
    let mut s = Session::connect();
    s.wait_banner();
    s.fast_machine();
    let job = sample_job(1.0);
    assert!(job.len() > 30, "job too small to be meaningful: {}", job.len());
    s.run_job(job, 60.0);
    // The job ends with a travel back to machine zero (front-left origin, return home on).
    s.wait_idle_at(0.0, 0.0, 30.0);
    assert!(s.errors().is_empty(), "GRBL rejected lines: {:?}", s.errors());
}

#[test]
#[ignore = "needs grbl-sim: tools/grbl-sim/build.sh"]
fn grbl_sim_long_job_keeps_character_counting_in_sync() {
    // A dense fill gives a few hundred short lines that execute faster than they arrive, so the 120-byte RX window is
    // full all the time. A wrong byte count would overflow GRBL's 128-byte buffer (lost or garbled lines -> errors)
    // or stall the stream. (Denser jobs work too, but grbl-sim can fall far behind real time on a shared CI runner.)
    let mut s = Session::connect();
    s.wait_banner();
    s.fast_machine();
    let job = sample_job(0.4);
    assert!(job.len() > 200, "{}", job.len());
    s.run_job(job, 240.0);
    s.wait_idle_at(0.0, 0.0, 60.0);
    assert!(s.errors().is_empty(), "GRBL rejected lines: {:?}", s.errors());
}

#[test]
#[ignore = "needs grbl-sim: tools/grbl-sim/build.sh"]
fn grbl_sim_rejected_line_does_not_desync_the_stream() {
    let mut s = Session::connect();
    s.wait_banner();
    s.fast_machine();
    let job: Vec<String> = ["G21", "G90", "G1 X5 Y5 F3000", "G999", "G1 X10 Y10", "G0 X0 Y0"].iter().map(|l| l.to_string()).collect();
    s.run_job(job, 20.0);
    s.wait_idle_at(0.0, 0.0, 20.0);
    // GRBL answers the unsupported command with error:20 and keeps going; the worker counts it as acknowledged.
    assert_eq!(s.errors(), vec!["error:20"]);
}

#[test]
#[ignore = "needs grbl-sim: tools/grbl-sim/build.sh"]
fn grbl_sim_feed_hold_and_resume() {
    let mut s = Session::connect();
    s.wait_banner();
    // Slow moves so the hold lands mid-job.
    let job: Vec<String> = ["G21", "G90", "G1 X40 Y0 F600", "G1 X40 Y10", "G0 X0 Y0"].iter().map(|l| l.to_string()).collect();
    s.link.send(Cmd::Job(job));
    s.wait("moving", 10.0, |v| v.state == "Run" && v.pos.0 > 1.0);
    s.link.send(Cmd::Realtime(b'!'));
    s.wait("held", 10.0, |v| v.state.starts_with("Hold:0"));
    let held_at = s.seen.pos;
    std::thread::sleep(Duration::from_millis(600));
    s.pump();
    assert!((s.seen.pos.0 - held_at.0).abs() < 1e-6, "machine moved while held");
    s.link.send(Cmd::Realtime(b'~'));
    s.wait("job finished", 60.0, |v| v.progress == (5, 5));
    s.wait_idle_at(0.0, 0.0, 60.0);
    assert!(s.errors().is_empty(), "{:?}", s.errors());
}

#[test]
#[ignore = "needs grbl-sim: tools/grbl-sim/build.sh"]
fn grbl_sim_abort_then_new_job_runs() {
    let mut s = Session::connect();
    s.wait_banner();
    s.fast_machine();
    s.link.send(Cmd::Job(sample_job(1.0)));
    s.wait("job started", 20.0, |v| v.progress.0 > 10);
    s.link.send(Cmd::Abort);
    // Real GRBL 1.1 comes back in alarm (ALARM:3, position lost) after a reset while moving and needs $X;
    // grbl-sim comes back idle, where $X is a harmless `ok`.
    s.wait("reset banner", 20.0, |v| v.rx.iter().filter(|l| l.starts_with("Grbl ")).count() >= 2);
    s.seen.mark = s.seen.rx.len();
    s.link.send(Cmd::Line("$X".into()));
    s.link.send(Cmd::Line("G10 L20 P1 X0 Y0".into()));
    let job: Vec<String> = ["G21", "G90", "G1 X3 Y4 F3000", "M5"].iter().map(|l| l.to_string()).collect();
    s.run_job(job, 20.0);
    s.wait("idle", 20.0, |v| v.state == "Idle");
    assert!(s.errors().is_empty(), "{:?}", s.errors());
}

#[test]
#[ignore = "needs grbl-sim: tools/grbl-sim/build.sh"]
fn grbl_sim_job_sent_right_after_connect_is_not_lost() {
    // The app lets the user press Start as soon as the link reports "connected", while GRBL is still restarting from
    // the soft reset (0x18) the worker sends on connect and throws away whatever it receives. The worker has to hold
    // the job back until the banner, or the lines are lost and the job never finishes.
    let mut s = Session::connect();
    let job: Vec<String> = ["G21", "G90", "G0 X2 Y3", "M5"].iter().map(|l| l.to_string()).collect();
    s.run_job(job, 15.0);
    s.wait_idle_at(2.0, 3.0, 15.0);
}
