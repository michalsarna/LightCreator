//! Focus mode: a weak steady beam at the head's place, to set the height by. It must never stay on by accident: it
//! goes off when the link drops, when a job starts, on Stop and when the program closes.
use crate::app::App;
use crate::i18n::tr;
use crate::laser::Cmd;
use eframe::egui::{self, Color32, RichText};

impl App {
    /// Can the focus beam be switched on now?
    pub fn focus_available(&self) -> bool {
        self.connected && !self.job_running() && self.doc.device.controller.is_serial() && self.doc.device.head(0).focus_power > 0.0
    }

    /// Switch the focus beam on or off. Switching on needs a connection, a focus power and no running job.
    pub fn focus_set(&mut self, on: bool) {
        if on == self.focus_on {
            return;
        }
        if on {
            if !self.focus_available() {
                return;
            }
        } else if !self.connected {
            self.focus_on = false;
            return;
        }
        let Some(lines) = lc_core::gcode::focus_gcode(&self.doc.device, on) else {
            self.focus_on = false;
            return;
        };
        for l in lines {
            self.link.send(Cmd::Line(l));
        }
        self.focus_on = on;
        self.status = tr(if on { "Focus beam on" } else { "Focus beam off" }).to_string();
    }

    /// Off, whatever the reason (a job starts, Stop, closing).
    pub fn focus_off(&mut self) {
        if self.focus_on {
            self.focus_set(false);
            self.focus_on = false;
        }
    }

    /// Abort the running job (and the focus beam with it).
    pub fn abort_job(&mut self) {
        self.link.send(Cmd::Abort);
        if self.focus_on {
            self.focus_on = false;
            // A soft reset stops GRBL's laser; for the others make sure.
            if self.doc.device.controller != lc_core::Controller::Grbl {
                self.link.send(Cmd::Line("M5".into()));
            }
        }
    }

    /// The button that switches the focus beam: dark while off, a red "off" button while on. `wide` fills the row.
    pub fn focus_button(&mut self, ui: &mut egui::Ui, wide: bool) {
        if self.doc.device.head(0).focus_power <= 0.0 && !self.focus_on {
            return;
        }
        if self.focus_on {
            let b = egui::Button::new(RichText::new(format!("●  {}", tr("FOCUS BEAM ON: press to switch off"))).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0xc0, 0x30, 0x30));
            let r = if wide { ui.add_sized([ui.available_width(), 34.0], b) } else { ui.add(b) };
            if r.clicked() {
                self.focus_set(false);
            }
        } else {
            let tip = if self.focus_available() { tr("Switch on a weak steady beam to set the focus height by") } else { tr("Connect to a laser first (Laser panel)") };
            let b = egui::Button::new(tr("Focus"));
            let r = ui.add_enabled(self.focus_available(), b).on_hover_text(tip);
            if r.clicked() {
                self.focus_set(true);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lc_core::{Device, Dialect};

    fn app(focus: f64) -> App {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.screen = crate::app::Screen::Editor;
        let mut d = Device::default();
        d.heads[0].focus_power = focus;
        a.profiles = vec![d];
        a.enter_editor(0);
        a
    }

    #[test]
    fn the_focus_lines_use_constant_power_in_the_heads_scale() {
        let mut d = Device::default();
        assert_eq!(lc_core::gcode::focus_gcode(&d, true), None, "no focus power, no focus mode");
        d.heads[0].focus_power = 1.5;
        d.heads[0].s_max = 1000.0;
        assert_eq!(lc_core::gcode::focus_gcode(&d, true), Some(vec!["M3 S15".to_string()]));
        assert_eq!(lc_core::gcode::focus_gcode(&d, false), Some(vec!["M5".to_string()]));
        d.gcode.dialect = Some(Dialect::Smoothieware);
        d.heads[0].s_max = 1.0;
        assert_eq!(lc_core::gcode::focus_gcode(&d, true), Some(vec!["M3 S0.015".to_string()]));
    }

    #[test]
    fn it_cannot_be_switched_on_without_a_connection_a_focus_power_or_while_a_job_runs() {
        let mut a = app(1.0);
        a.focus_set(true);
        assert!(!a.focus_on, "no connection");
        a.connected = true;
        a.progress = (3, 10);
        a.focus_set(true);
        assert!(!a.focus_on, "a job is running");
        a.progress = (0, 0);
        a.focus_set(true);
        assert!(a.focus_on);
        let mut none = app(0.0);
        none.connected = true;
        none.focus_set(true);
        assert!(!none.focus_on, "focus power 0 = no focus mode");
    }

    #[test]
    fn it_goes_off_when_a_job_starts_on_stop_and_when_the_link_drops() {
        let mut a = app(1.0);
        a.connected = true;
        a.focus_set(true);
        assert!(a.focus_on);
        a.abort_job();
        assert!(!a.focus_on, "stop");
        a.focus_set(true);
        a.focus_off();
        assert!(!a.focus_on, "explicit off");
        // Starting a job switches it off first (the job here has nothing to burn, which is fine for the test).
        a.focus_set(true);
        a.doc.add(0, lc_core::Kind::Rect { w: 10.0, h: 10.0 }, lc_core::Xf::translate(10.0, 10.0));
        a.send_job();
        assert!(!a.focus_on, "start");
        // The link drops: the flag is cleared at once (the controller has lost the connection anyway).
        a.focus_set(true);
        a.focus_on = true;
        a.connected = false;
        a.focus_set(false);
        assert!(!a.focus_on, "disconnect");
    }
}
