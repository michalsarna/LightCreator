//! Controller dialects: the small differences between GRBL and Marlin that the UI and the serial worker need.
use crate::doc::Controller;

/// How the periodic status poll is done.
pub enum StatusPoll {
    /// A single realtime byte (GRBL `?`).
    Byte(u8),
    /// A normal queued line (Marlin `M114`).
    Line(&'static str),
    /// Not supported.
    None,
}

/// Realtime-style actions the Device tab offers.
pub enum Action {
    /// One or more lines queued like any other command.
    Lines(Vec<String>),
    /// A single realtime byte, sent immediately.
    Byte(u8),
    None,
}

impl Controller {
    pub fn status_poll(self) -> StatusPoll {
        match self {
            Controller::Grbl => StatusPoll::Byte(b'?'),
            Controller::Marlin => StatusPoll::Line("M114"),
            _ => StatusPoll::None,
        }
    }
    /// Milliseconds between status polls.
    pub fn poll_interval_ms(self) -> u64 {
        match self {
            Controller::Marlin => 1000,
            _ => 250,
        }
    }
    pub fn home(self) -> Action {
        match self {
            Controller::Grbl => Action::Lines(vec!["$H".into()]),
            Controller::Marlin => Action::Lines(vec!["G28 X Y".into()]),
            _ => Action::None,
        }
    }
    /// Clear an alarm / halted state.
    pub fn unlock(self) -> Action {
        match self {
            Controller::Grbl => Action::Lines(vec!["$X".into()]),
            Controller::Marlin => Action::Lines(vec!["M999".into()]),
            _ => Action::None,
        }
    }
    pub fn pause(self) -> Action {
        match self {
            Controller::Grbl => Action::Byte(b'!'),
            Controller::Marlin => Action::Lines(vec!["M25".into()]),
            _ => Action::None,
        }
    }
    pub fn resume(self) -> Action {
        match self {
            Controller::Grbl => Action::Byte(b'~'),
            Controller::Marlin => Action::Lines(vec!["M24".into()]),
            _ => Action::None,
        }
    }
    pub fn cancel_jog(self) -> Action {
        match self {
            Controller::Grbl => Action::Byte(0x85),
            Controller::Marlin => Action::Lines(vec!["M410".into()]),
            _ => Action::None,
        }
    }
    /// Jog by (dx, dy) millimetres at `feed` mm/min.
    pub fn jog(self, dx: f64, dy: f64, feed: f64) -> Action {
        match self {
            Controller::Grbl => Action::Lines(vec![format!("$J=G91 G21 X{dx} Y{dy} F{feed}")]),
            Controller::Marlin => Action::Lines(vec!["G91".into(), format!("G0 X{dx} Y{dy} F{feed}"), "G90".into()]),
            _ => Action::None,
        }
    }
    /// Byte(s) sent right after connecting.
    pub fn on_connect(self) -> Option<u8> {
        match self {
            Controller::Grbl => Some(0x18),
            _ => None,
        }
    }
    /// Emergency stop / abort byte, if the controller has a realtime one.
    pub fn abort_byte(self) -> Option<u8> {
        match self {
            Controller::Grbl => Some(0x18),
            _ => None,
        }
    }
    /// Line queued after an abort for controllers without a realtime abort.
    pub fn abort_line(self) -> Option<&'static str> {
        match self {
            Controller::Marlin => Some("M410"),
            _ => None,
        }
    }
}

/// Parse a status report into (state, x, y). GRBL: `<Idle|MPos:1,2,0|...>`; Marlin: `X:1.00 Y:2.00 Z:0.00 ...`.
pub fn parse_status(line: &str) -> Option<(String, f64, f64)> {
    if let Some(inner) = line.strip_prefix('<').and_then(|l| l.strip_suffix('>')) {
        let mut parts = inner.split('|');
        let state = parts.next()?.to_string();
        for p in parts {
            if let Some(v) = p.strip_prefix("MPos:").or_else(|| p.strip_prefix("WPos:")) {
                let n: Vec<f64> = v.split(',').filter_map(|s| s.parse().ok()).collect();
                if n.len() >= 2 {
                    return Some((state, n[0], n[1]));
                }
            }
        }
        return Some((state, f64::NAN, f64::NAN));
    }
    if line.starts_with("X:") {
        let num = |key: &str| line.split_whitespace().find_map(|t| t.strip_prefix(key)).and_then(|v| v.parse::<f64>().ok());
        if let (Some(x), Some(y)) = (num("X:"), num("Y:")) {
            return Some(("Idle".into(), x, y));
        }
    }
    None
}
