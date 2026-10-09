//! GRBL serial link running on a worker thread (character-counting streaming protocol).
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use lc_core::controller::StatusPoll;
use lc_core::Controller;
use std::time::{Duration, Instant};

pub enum Cmd {
    Connect { port: String, baud: u32, controller: Controller },
    Disconnect,
    /// A single immediate line (jog, $H, ...), queued ahead of any running job.
    Line(String),
    /// Stream a whole job.
    Job(Vec<String>),
    /// Single-byte realtime command: `?`, `!`, `~`, 0x18.
    Realtime(u8),
    /// Soft reset and drop everything queued.
    Abort,
}

/// Direction of a console line.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    /// Message from the application itself.
    Info,
    /// Sent to the controller.
    Tx,
    /// Received from the controller.
    Rx,
}

#[derive(Clone)]
pub struct ConsoleLine {
    pub dir: Dir,
    pub text: String,
    /// Part of the periodic `?` status polling (hidden by default in the console).
    pub poll: bool,
}

impl ConsoleLine {
    pub fn info(text: impl Into<String>) -> Self {
        ConsoleLine { dir: Dir::Info, text: text.into(), poll: false }
    }
}

pub enum Evt {
    Log(String),
    /// A line that crossed the serial port.
    Traffic(ConsoleLine),
    Connected(bool),
    Status { state: String, x: f64, y: f64 },
    Progress { done: usize, total: usize },
}

pub struct LaserLink {
    tx: Sender<Cmd>,
    rx: Receiver<Evt>,
}

impl LaserLink {
    pub fn spawn(ctx: eframe::egui::Context) -> Self {
        let (tx, wrx) = channel();
        let (wtx, rx) = channel();
        std::thread::spawn(move || worker(wrx, wtx, ctx));
        Self { tx, rx }
    }
    pub fn send(&self, c: Cmd) {
        let _ = self.tx.send(c);
    }
    pub fn poll(&self) -> Vec<Evt> {
        let mut v = vec![];
        while let Ok(e) = self.rx.try_recv() {
            v.push(e);
        }
        v
    }
}

pub fn list_ports() -> Vec<String> {
    serialport::available_ports().map(|p| p.into_iter().map(|x| x.port_name).collect()).unwrap_or_default()
}

/// Remove comments/blank lines so only real commands get streamed.
pub fn clean_gcode(text: &str) -> Vec<String> {
    text.lines()
        .map(|l| l.split(';').next().unwrap_or("").trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

fn realtime_name(b: u8) -> String {
    match b {
        0x18 => "0x18 (soft reset)".into(),
        0x85 => "0x85 (jog cancel)".into(),
        b'!' => "! (feed hold)".into(),
        b'~' => "~ (cycle start)".into(),
        b'?' => "?".into(),
        _ => format!("0x{b:02X}"),
    }
}

const RX_BUFFER: usize = 120; // GRBL has 128 bytes; keep a margin.

/// Why a line is in the queue; decides how its `ok` is shown in the console.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Job,
    Immediate,
    Poll,
}

fn worker(rx: Receiver<Cmd>, tx: Sender<Evt>, ctx: eframe::egui::Context) {
    let emit = |e: Evt| {
        let _ = tx.send(e);
        ctx.request_repaint();
    };
    let traffic = |dir: Dir, text: String, poll: bool| Evt::Traffic(ConsoleLine { dir, text, poll });
    let mut port: Option<Box<dyn serialport::SerialPort>> = None;
    let mut controller = Controller::Grbl;
    let mut queue: VecDeque<(String, Kind)> = VecDeque::new();
    let mut pending: VecDeque<(usize, Kind)> = VecDeque::new();
    let mut inbuf = Vec::<u8>::new();
    let (mut total, mut done) = (0usize, 0usize);
    let mut last_poll = Instant::now();
    loop {
        loop {
            match rx.try_recv() {
                Ok(Cmd::Connect { port: name, baud, controller: c }) => {
                    match serialport::new(&name, baud).timeout(Duration::from_millis(5)).open() {
                        Ok(mut p) => {
                            controller = c;
                            if let Some(b) = c.on_connect() {
                                let _ = p.write_all(&[b]);
                            }
                            port = Some(p);
                            queue.clear();
                            pending.clear();
                            emit(Evt::Log(crate::i18n::trf("Connected to {} @ {}", &[&name, &baud])));
                            emit(Evt::Connected(true));
                        }
                        Err(e) => emit(Evt::Log(crate::i18n::trf("Cannot open {}: {}", &[&name, &e]))),
                    }
                }
                Ok(Cmd::Disconnect) => {
                    port = None;
                    queue.clear();
                    pending.clear();
                    emit(Evt::Connected(false));
                    emit(Evt::Log(crate::i18n::tr("Disconnected").into()));
                }
                Ok(Cmd::Line(l)) => {
                    // Immediate lines go ahead of job lines.
                    let at = queue.iter().position(|(_, k)| *k == Kind::Job).unwrap_or(queue.len());
                    queue.insert(at, (l, Kind::Immediate));
                }
                Ok(Cmd::Job(lines)) => {
                    total = lines.len();
                    done = 0;
                    queue.extend(lines.into_iter().map(|l| (l, Kind::Job)));
                    emit(Evt::Progress { done, total });
                }
                Ok(Cmd::Realtime(b)) => {
                    if let Some(p) = port.as_mut() {
                        let _ = p.write_all(&[b]);
                        emit(traffic(Dir::Tx, realtime_name(b), false));
                    }
                }
                Ok(Cmd::Abort) => {
                    queue.clear();
                    pending.clear();
                    total = 0;
                    if let Some(p) = port.as_mut() {
                        if let Some(b) = controller.abort_byte() {
                            let _ = p.write_all(&[b]);
                            emit(traffic(Dir::Tx, realtime_name(b), false));
                        }
                        if let Some(l) = controller.abort_line() {
                            queue.push_back((l.to_string(), Kind::Immediate));
                        }
                    }
                    emit(Evt::Progress { done: 0, total: 0 });
                    emit(Evt::Log(crate::i18n::tr("Aborted (soft reset)").into()));
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return,
            }
        }
        let mut lost = false;
        if let Some(p) = port.as_mut() {
            // Read
            let mut buf = [0u8; 256];
            match p.read(&mut buf) {
                Ok(n) if n > 0 => inbuf.extend_from_slice(&buf[..n]),
                Err(e) if e.kind() != std::io::ErrorKind::TimedOut => lost = true,
                _ => {}
            }
            while let Some(i) = inbuf.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = inbuf.drain(..=i).collect();
                let line = String::from_utf8_lossy(&line).trim().to_string();
                if line.is_empty() {
                    continue;
                }
                if let Some((state, x, y)) = lc_core::controller::parse_status(&line) {
                    emit(traffic(Dir::Rx, line.clone(), true));
                    emit(Evt::Status { state, x, y });
                } else if line == "ok" || line.starts_with("error") {
                    let kind = pending.pop_front().map(|(_, k)| k);
                    emit(traffic(Dir::Rx, line.clone(), kind == Some(Kind::Poll)));
                    if kind == Some(Kind::Job) {
                        done += 1;
                        emit(Evt::Progress { done, total });
                    }
                } else {
                    emit(traffic(Dir::Rx, line, false));
                }
            }
            // Write while the controller's RX buffer has room.
            while let Some((l, _)) = queue.front() {
                let used: usize = pending.iter().map(|(n, _)| n).sum();
                if used + l.len() + 1 > RX_BUFFER {
                    break;
                }
                let (l, kind) = queue.pop_front().unwrap();
                emit(traffic(Dir::Tx, l.clone(), kind == Kind::Poll));
                if p.write_all(format!("{l}\n").as_bytes()).is_err() {
                    lost = true;
                    break;
                }
                pending.push_back((l.len() + 1, kind));
            }
            if last_poll.elapsed() > Duration::from_millis(controller.poll_interval_ms()) {
                last_poll = Instant::now();
                match controller.status_poll() {
                    StatusPoll::Byte(b) => {
                        let _ = p.write_all(&[b]);
                        emit(traffic(Dir::Tx, (b as char).to_string(), true));
                    }
                    // Line-based polling only when nothing else is waiting, so jobs are not slowed down.
                    StatusPoll::Line(l) if queue.is_empty() && pending.is_empty() => queue.push_back((l.to_string(), Kind::Poll)),
                    _ => {}
                }
            }
        }
        if lost {
            port = None;
            queue.clear();
            pending.clear();
            emit(Evt::Connected(false));
            emit(Evt::Log(crate::i18n::tr("Connection lost").into()));
        }
        std::thread::sleep(Duration::from_millis(if port.is_some() { 2 } else { 30 }));
    }
}
