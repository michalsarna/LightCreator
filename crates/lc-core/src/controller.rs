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
            Controller::Grbl | Controller::Smoothieware => StatusPoll::Byte(b'?'),
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
            Controller::Grbl | Controller::Smoothieware => Action::Lines(vec!["$H".into()]),
            Controller::Marlin => Action::Lines(vec!["G28 X Y".into()]),
            _ => Action::None,
        }
    }
    /// Clear an alarm / halted state.
    pub fn unlock(self) -> Action {
        match self {
            Controller::Grbl => Action::Lines(vec!["$X".into()]),
            Controller::Marlin | Controller::Smoothieware => Action::Lines(vec!["M999".into()]),
            _ => Action::None,
        }
    }
    pub fn pause(self) -> Action {
        match self {
            Controller::Grbl | Controller::Smoothieware => Action::Byte(b'!'),
            Controller::Marlin => Action::Lines(vec!["M25".into()]),
            _ => Action::None,
        }
    }
    pub fn resume(self) -> Action {
        match self {
            Controller::Grbl | Controller::Smoothieware => Action::Byte(b'~'),
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
            Controller::Marlin | Controller::Smoothieware => Action::Lines(vec!["G91".into(), format!("G0 X{dx} Y{dy} F{feed}"), "G90".into()]),
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
            Controller::Grbl | Controller::Smoothieware => Some(0x18),
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

/// The option letters GRBL reports in its build info (`$I`): `[OPT:VH,15,128]` gives `"VH"`.
pub fn grbl_options(lines: &[String]) -> Option<String> {
    lines.iter().find_map(|l| l.trim().strip_prefix("[OPT:")).map(|o| o.split([',', ']']).next().unwrap_or("").to_string())
}

/// Was this GRBL built with `$HX` / `$HY` / `$HZ` (single-axis homing, option letter `H`)?
pub fn single_axis_homing(options: &str) -> bool {
    options.contains('H')
}

/// Parse a status report into (state, x, y). GRBL: `<Idle|MPos:1,2,0|...>`; Marlin: `X:1.00 Y:2.00 Z:0.00 ...`.
pub fn parse_status(line: &str) -> Option<(String, f64, f64)> {
    // Smoothieware: `<Idle,MPos:1.0000,2.0000,3.0000,WPos:...>` (commas everywhere).
    if let Some(inner) = line.strip_prefix('<').and_then(|l| l.strip_suffix('>')).filter(|l| !l.contains('|') && l.contains(',')) {
        let parts: Vec<&str> = inner.split(',').collect();
        let state = parts.first()?.to_string();
        let at = parts.iter().position(|p| p.starts_with("MPos:") || p.starts_with("WPos:"))?;
        let x = parts[at].split(':').nth(1)?.parse::<f64>().ok()?;
        let y = parts.get(at + 1)?.parse::<f64>().ok()?;
        return Some((state, x, y));
    }
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

/// Machine values that could be read from a controller.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Reading {
    pub bed_w: Option<f64>,
    pub bed_h: Option<f64>,
    pub s_max: Option<f64>,
    /// mm/min
    pub travel_speed: Option<f64>,
    pub dynamic_power: Option<bool>,
    /// GRBL `$32`: laser mode on. Without it GRBL treats the laser like a spindle.
    pub laser_mode: Option<bool>,
}

/// What a controller reports that does not agree with the profile or is unsafe.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FwCheck {
    /// GRBL's laser mode (`$32`) is off.
    pub laser_mode_off: bool,
    /// `$30` of the controller when it differs from the S max of the profile.
    pub s_max: Option<f64>,
}

impl FwCheck {
    pub fn is_clean(&self) -> bool {
        !self.laser_mode_off && self.s_max.is_none()
    }
}

impl Reading {
    /// Compare what the controller reported with the profile of the device.
    pub fn check(&self, dev: &crate::doc::Device) -> FwCheck {
        FwCheck {
            laser_mode_off: self.laser_mode == Some(false),
            s_max: self.s_max.filter(|v| (*v - dev.s_max()).abs() > 1e-6),
        }
    }
}

impl Reading {
    pub fn count(&self) -> usize {
        [self.bed_w.is_some(), self.bed_h.is_some(), self.s_max.is_some(), self.travel_speed.is_some(), self.dynamic_power.is_some()].iter().filter(|b| **b).count()
    }
}

impl Controller {
    /// Command that makes the controller list its settings.
    pub fn settings_request(self) -> Option<&'static str> {
        match self {
            Controller::Grbl => Some("$$"),
            Controller::Marlin => Some("M503"),
            _ => None,
        }
    }

    /// Pull machine values out of the controller's settings report.
    pub fn parse_settings(self, lines: &[String]) -> Reading {
        let mut r = Reading::default();
        match self {
            Controller::Grbl => {
                for l in lines {
                    let Some((k, v)) = l.trim().strip_prefix('$').and_then(|s| s.split_once('=')) else { continue };
                    let Ok(v) = v.trim().parse::<f64>() else { continue };
                    match k.trim() {
                        "30" => r.s_max = Some(v),
                        "110" => r.travel_speed = Some(v),
                        "130" => r.bed_w = Some(v),
                        "131" => r.bed_h = Some(v),
                        "32" => {
                            r.dynamic_power = Some(v >= 1.0);
                            r.laser_mode = Some(v >= 1.0);
                        }
                        _ => {}
                    }
                }
            }
            Controller::Marlin => {
                for l in lines {
                    // "echo:  M203 X500.00 Y500.00 ..." is the maximum feedrate in mm/s.
                    if let Some(i) = l.find("M203") {
                        let x = l[i..].split_whitespace().find_map(|t| t.strip_prefix('X')).and_then(|t| t.parse::<f64>().ok());
                        if let Some(x) = x {
                            r.travel_speed = Some(x * 60.0);
                        }
                    }
                    // Some builds report the software end stops: "... Max:  X200.00 Y200.00 ...".
                    if let Some(i) = l.find("Max:") {
                        let t: Vec<&str> = l[i..].split_whitespace().collect();
                        r.bed_w = t.iter().find_map(|s| s.strip_prefix('X')).and_then(|s| s.parse().ok());
                        r.bed_h = t.iter().find_map(|s| s.strip_prefix('Y')).and_then(|s| s.parse().ok());
                    }
                }
            }
            _ => {}
        }
        r
    }
}
