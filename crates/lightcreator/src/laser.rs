//! GRBL serial link running on a worker thread (character-counting streaming protocol).
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use lc_core::controller::StatusPoll;
use lc_core::Controller;
use std::time::{Duration, Instant};

/// Where to connect.
#[derive(Clone, Debug)]
pub enum Target {
    Serial { port: String, baud: u32 },
    /// A raw TCP socket, e.g. `ser2net` on a Raspberry Pi.
    Tcp { host: String, port: u16 },
}

impl Target {
    /// The connection settings of a device profile, if a target is configured.
    pub fn of(dev: &lc_core::Device) -> Option<Target> {
        if !dev.has_target() {
            return None;
        }
        Some(match dev.link {
            lc_core::LinkKind::Serial => Target::Serial { port: dev.port.clone(), baud: dev.baud },
            lc_core::LinkKind::Tcp => Target::Tcp { host: dev.host.trim().to_string(), port: dev.tcp_port },
        })
    }
    fn label(&self) -> String {
        match self {
            Target::Serial { port, baud } => format!("{port} @ {baud}"),
            Target::Tcp { host, port } => format!("{host}:{port}"),
        }
    }
}

/// A byte stream to the controller: a serial port or a TCP socket.
trait Link: Read + Write + Send {}
impl<T: Read + Write + Send> Link for T {}

fn open_link(t: &Target) -> Result<(Box<dyn Link>, bool), String> {
    match t {
        Target::Serial { port, baud } => serialport::new(port, *baud).timeout(Duration::from_millis(5)).open().map(|p| (Box::new(p) as Box<dyn Link>, false)).map_err(|e| e.to_string()),
        Target::Tcp { host, port } => {
            use std::net::{TcpStream, ToSocketAddrs};
            let addrs = (host.as_str(), *port).to_socket_addrs().map_err(|e| e.to_string())?;
            let mut last = "no address".to_string();
            for a in addrs {
                match TcpStream::connect_timeout(&a, Duration::from_secs(4)) {
                    Ok(s) => {
                        let _ = s.set_nodelay(true);
                        let _ = s.set_read_timeout(Some(Duration::from_millis(5)));
                        let _ = s.set_write_timeout(Some(Duration::from_secs(3)));
                        return Ok((Box::new(s), true));
                    }
                    Err(e) => last = e.to_string(),
                }
            }
            Err(last)
        }
    }
}

pub enum Cmd {
    Connect { target: Target, controller: Controller },
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
/// Longest wait for GRBL's start-up banner after a soft reset before streaming anyway.
const BANNER_TIMEOUT: Duration = Duration::from_millis(2500);

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
    let mut port: Option<Box<dyn Link>> = None;
    let mut is_tcp = false;
    let mut controller = Controller::Grbl;
    let mut queue: VecDeque<(String, Kind)> = VecDeque::new();
    let mut pending: VecDeque<(usize, Kind)> = VecDeque::new();
    let mut inbuf = Vec::<u8>::new();
    let (mut total, mut done) = (0usize, 0usize);
    let mut last_poll = Instant::now();
    // Set after a soft reset: GRBL drops everything it receives until it has restarted and printed its banner
    // (`Grbl 1.1h ['$' for help]`), so queued lines are held back until then.
    let mut restarting: Option<Instant> = None;
    loop {
        loop {
            match rx.try_recv() {
                Ok(Cmd::Connect { target, controller: c }) => {
                    let label = target.label();
                    match open_link(&target) {
                        Ok((mut p, tcp)) => {
                            is_tcp = tcp;
                            controller = c;
                            restarting = None;
                            if let Some(b) = c.on_connect() {
                                let _ = p.write_all(&[b]);
                                restarting = Some(Instant::now());
                            }
                            port = Some(p);
                            queue.clear();
                            pending.clear();
                            emit(Evt::Log(crate::i18n::trf("Connected to {}", &[&label])));
                            emit(Evt::Connected(true));
                        }
                        Err(e) => emit(Evt::Log(crate::i18n::trf("Cannot open {}: {}", &[&label, &e]))),
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
                            restarting = Some(Instant::now());
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
                // A TCP peer that closed the socket reads as zero bytes.
                Ok(_) if is_tcp => lost = true,
                Err(e) if !matches!(e.kind(), std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted) => lost = true,
                _ => {}
            }
            while let Some(i) = inbuf.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = inbuf.drain(..=i).collect();
                let line = String::from_utf8_lossy(&line).trim().to_string();
                if line.is_empty() {
                    continue;
                }
                if restarting.is_some() && line.starts_with("Grbl ") {
                    // Restarted: anything sent before the reset is gone, and so are its acknowledgements.
                    restarting = None;
                    pending.clear();
                    emit(traffic(Dir::Rx, line, false));
                } else if let Some((state, x, y)) = lc_core::controller::parse_status(&line) {
                    emit(traffic(Dir::Rx, line.clone(), true));
                    emit(Evt::Status { state, x, y });
                } else if restarting.is_some() && (line == "ok" || line.starts_with("error")) {
                    // Start-up noise (e.g. `error:7` from an empty EEPROM) or a late reply to a line sent before the
                    // reset: not the acknowledgement of anything queued now.
                    emit(traffic(Dir::Rx, line, false));
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
            if restarting.is_some_and(|t| t.elapsed() > BANNER_TIMEOUT) {
                // No banner (a clone that does not print one?): stream anyway rather than hang.
                restarting = None;
                pending.clear();
            }
            // Write while the controller's RX buffer has room.
            while let Some((l, _)) = queue.front().filter(|_| restarting.is_none()) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    /// A stand-in for `ser2net` + GRBL: answers every line with `ok`, every `?` with a status report and a soft reset
    /// with the start-up banner.
    fn fake_grbl(listener: TcpListener, close_after_ms: u64) {
        std::thread::spawn(move || {
            let (mut s, _) = listener.accept().unwrap();
            let _ = s.set_read_timeout(Some(Duration::from_millis(20)));
            let start = Instant::now();
            let mut buf = Vec::new();
            let mut chunk = [0u8; 64];
            while start.elapsed() < Duration::from_millis(close_after_ms) {
                if let Ok(n) = s.read(&mut chunk) {
                    for &b in &chunk[..n] {
                        match b {
                            b'?' => {
                                let _ = s.write_all(b"<Idle|MPos:1.000,2.000,0.000|FS:0,0>\n");
                            }
                            b'\n' => {
                                buf.clear();
                                let _ = s.write_all(b"ok\n");
                            }
                            0x18 => {
                                // Like GRBL after a soft reset.
                                buf.clear();
                                let _ = s.write_all(b"\r\nGrbl 1.1h ['$' for help]\r\n");
                            }
                            _ => buf.push(b),
                        }
                    }
                }
            }
            // Dropping the socket closes the connection.
        });
    }

    fn wait_for(link: &LaserLink, secs: f32, mut pred: impl FnMut(&Evt) -> bool) -> bool {
        let t = Instant::now();
        while t.elapsed().as_secs_f32() < secs {
            for e in link.poll() {
                if pred(&e) {
                    return true;
                }
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        false
    }

    #[test]
    fn streams_gcode_over_tcp_like_ser2net() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        fake_grbl(listener, 1500);
        let link = LaserLink::spawn(eframe::egui::Context::default());
        link.send(Cmd::Connect { target: Target::Tcp { host: "127.0.0.1".into(), port }, controller: Controller::Grbl });
        assert!(wait_for(&link, 3.0, |e| matches!(e, Evt::Connected(true))), "connected");
        link.send(Cmd::Job(vec!["G21".into(), "G0 X5 Y5".into(), "M5".into()]));
        let mut done = false;
        assert!(wait_for(&link, 3.0, |e| {
            if let Evt::Progress { done: d, total } = e {
                done = *d == 3 && *total == 3;
            }
            done
        }), "three lines acknowledged");
        // A status report from the poll arrives too.
        assert!(wait_for(&link, 3.0, |e| matches!(e, Evt::Status { x, y, .. } if (*x - 1.0).abs() < 1e-9 && (*y - 2.0).abs() < 1e-9)), "status");
        // The server closes the socket: the link reports the loss.
        assert!(wait_for(&link, 4.0, |e| matches!(e, Evt::Connected(false))), "connection lost detected");
    }

    #[test]
    fn unreachable_host_reports_an_error() {
        let link = LaserLink::spawn(eframe::egui::Context::default());
        // Port 1 on localhost is closed.
        link.send(Cmd::Connect { target: Target::Tcp { host: "127.0.0.1".into(), port: 1 }, controller: Controller::Grbl });
        assert!(wait_for(&link, 5.0, |e| matches!(e, Evt::Log(s) if s.contains("127.0.0.1:1"))));
    }
}
