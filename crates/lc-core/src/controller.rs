//! Controller dialects: the small differences between GRBL and Marlin that the UI and the serial worker need. The
//! dialects themselves are in base-core; what is here reads the settings of a controller into a laser profile.
use crate::doc::Controller;
pub use base_core::machine::{grbl_options, parse_status, parse_status_z, single_axis_homing, Action, StatusPoll};

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

/// Reading the settings of a controller (laser profile values).
pub trait ControllerSettings {
    fn settings_request(self) -> Option<&'static str>;
    fn parse_settings(self, lines: &[String]) -> Reading;
}

impl ControllerSettings for Controller {
    /// Command that makes the controller list its settings.
    fn settings_request(self) -> Option<&'static str> {
        match self {
            Controller::Grbl => Some("$$"),
            Controller::Marlin => Some("M503"),
            _ => None,
        }
    }

    /// Pull machine values out of the controller's settings report.
    fn parse_settings(self, lines: &[String]) -> Reading {
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
