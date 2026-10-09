//! GRBL serial link running on a worker thread (character-counting streaming protocol).
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::time::{Duration, Instant};

pub enum Cmd {
    Connect { port: String, baud: u32 },
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

pub enum Evt {
    Log(String),
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

fn parse_status(line: &str) -> Option<Evt> {
    let inner = line.strip_prefix('<')?.strip_suffix('>')?;
    let mut parts = inner.split('|');
    let state = parts.next()?.to_string();
    for p in parts {
        if let Some(v) = p.strip_prefix("MPos:").or_else(|| p.strip_prefix("WPos:")) {
            let n: Vec<f64> = v.split(',').filter_map(|s| s.parse().ok()).collect();
            if n.len() >= 2 {
                return Some(Evt::Status { state, x: n[0], y: n[1] });
            }
        }
    }
    Some(Evt::Status { state, x: f64::NAN, y: f64::NAN })
}

const RX_BUFFER: usize = 120; // GRBL has 128 bytes; keep a margin.

fn worker(rx: Receiver<Cmd>, tx: Sender<Evt>, ctx: eframe::egui::Context) {
    let emit = |e: Evt| {
        let _ = tx.send(e);
        ctx.request_repaint();
    };
    let mut port: Option<Box<dyn serialport::SerialPort>> = None;
    let mut queue: VecDeque<(String, bool)> = VecDeque::new(); // (line, belongs_to_job)
    let mut pending: VecDeque<(usize, bool)> = VecDeque::new();
    let mut inbuf = Vec::<u8>::new();
    let (mut total, mut done) = (0usize, 0usize);
    let mut last_poll = Instant::now();
    loop {
        loop {
            match rx.try_recv() {
                Ok(Cmd::Connect { port: name, baud }) => {
                    match serialport::new(&name, baud).timeout(Duration::from_millis(5)).open() {
                        Ok(mut p) => {
                            let _ = p.write_all(&[0x18]);
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
                    let at = queue.iter().position(|(_, j)| *j).unwrap_or(queue.len());
                    queue.insert(at, (l, false));
                }
                Ok(Cmd::Job(lines)) => {
                    total = lines.len();
                    done = 0;
                    queue.extend(lines.into_iter().map(|l| (l, true)));
                    emit(Evt::Progress { done, total });
                }
                Ok(Cmd::Realtime(b)) => {
                    if let Some(p) = port.as_mut() {
                        let _ = p.write_all(&[b]);
                    }
                }
                Ok(Cmd::Abort) => {
                    if let Some(p) = port.as_mut() {
                        let _ = p.write_all(&[0x18]);
                    }
                    queue.clear();
                    pending.clear();
                    total = 0;
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
                if let Some(e) = parse_status(&line) {
                    emit(e);
                } else if line == "ok" || line.starts_with("error") {
                    if let Some((_, job)) = pending.pop_front() {
                        if job {
                            done += 1;
                            emit(Evt::Progress { done, total });
                        }
                    }
                    if line != "ok" {
                        emit(Evt::Log(line));
                    }
                } else {
                    emit(Evt::Log(line));
                }
            }
            // Write while the controller's RX buffer has room.
            while let Some((l, _)) = queue.front() {
                let used: usize = pending.iter().map(|(n, _)| n).sum();
                if used + l.len() + 1 > RX_BUFFER {
                    break;
                }
                let (l, job) = queue.pop_front().unwrap();
                if !job {
                    emit(Evt::Log(format!("> {l}")));
                }
                if p.write_all(format!("{l}\n").as_bytes()).is_err() {
                    lost = true;
                    break;
                }
                pending.push_back((l.len() + 1, job));
            }
            if last_poll.elapsed() > Duration::from_millis(250) {
                last_poll = Instant::now();
                let _ = p.write_all(b"?");
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
