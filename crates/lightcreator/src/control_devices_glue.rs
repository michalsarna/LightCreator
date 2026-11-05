//! The USB keyboard or pad in this program: what its presses do to the laser (`base_ui::control_devices_ui` has the rest).
use crate::app::App;
use crate::control_glue::LaserTarget;
use eframe::egui;
use lc_core::control::Control;
use lc_core::control_devices::{Action, Fired};
use lc_core::Controller;
use base_ui::control_devices_ui::JogCfg;
use base_ui::control_ui;

/// What the keys of a laser can do.
pub const ACTIONS: &[Action] = &[
    Action::MoveLeft, Action::MoveRight, Action::MoveForward, Action::MoveBack, Action::MoveUp, Action::MoveDown, Action::Stop, Action::Pause, Action::Resume, Action::Home, Action::Unlock,
    Action::StartJob, Action::Frame, Action::Laser, Action::Air,
];

impl App {
    /// Once a frame: read the device and do what it says. Moves are shared; the rest is done as the buttons of the Control tab do it.
    pub fn control_devices_frame(&mut self, ctx: &egui::Context) {
        let listening = self.connected && self.screen == crate::app::Screen::Editor;
        let own = self.doc.device.control_device.clone();
        let fired = self.control_devices.poll(ctx, own.as_deref(), listening);
        let dev = &self.doc.device;
        let mut control = Control::default();
        control.jog.xy_feed = dev.jog_feed;
        control.jog.z_feed = dev.jog_feed;
        control.jog.xy_steps = vec![dev.jog_step];
        control.jog.z_steps = vec![dev.jog_step];
        let (step_xy, step_z, feed_xy, feed_z) = control_ui::jog_numbers(&self.ctl, &control);
        let cfg = JogCfg { step_xy, step_z, feed_xy, feed_z, hold: dev.link_controller() == Controller::Grbl, z_axis: dev.axes.has_z };
        let mut st = std::mem::take(&mut self.control_devices);
        let rest = st.apply_moves(ctx, fired, &mut LaserTarget { app: self }, &cfg);
        self.control_devices = st;
        for f in rest {
            let ctrl = self.doc.device.link_controller();
            match f {
                Fired::Press(Action::Stop) => self.control_device_stop(),
                Fired::Press(Action::Pause) => self.run_action(ctrl.pause()),
                Fired::Press(Action::Resume) => self.run_action(ctrl.resume()),
                Fired::Press(Action::Home) => self.run_action(ctrl.home()),
                Fired::Press(Action::Unlock) => self.run_action(ctrl.unlock()),
                Fired::Press(Action::StartJob) if self.can_start() => self.send_job(),
                Fired::Press(Action::Frame) if self.connected && !self.job_running() => self.frame_with(false),
                // The beam never stays on by itself: only while the key is down, and at the power of the focus beam of the device.
                Fired::Press(Action::Laser) => self.focus_set(true),
                Fired::Release(Action::Laser) => self.focus_set(false),
                Fired::Press(Action::Air) => {
                    let on = self.control_devices.toggle(Action::Air);
                    self.air_lines(on);
                }
                _ => {}
            }
        }
    }

    fn air_lines(&mut self, on: bool) {
        let (a_on, a_off) = lc_core::gcode::air_commands(&self.doc.device);
        for l in (if on { a_on } else { a_off }).lines() {
            self.link.send(crate::laser::Cmd::Line(l.to_string()));
        }
    }

    /// Stop what runs; with nothing running, switch off what the keys switched on.
    fn control_device_stop(&mut self) {
        if self.machine_busy() {
            self.abort_job();
        } else if self.control_devices.is_on(Action::Air) {
            self.air_lines(false);
        }
        self.control_devices.all_off();
    }

    /// The windows of the settings. What is chosen for this machine stays with it.
    pub fn control_devices_windows(&mut self, ctx: &egui::Context) {
        let mut own = self.doc.device.control_device.clone();
        let mut changed = self.control_devices.window(ctx, ACTIONS, &mut own, true);
        changed |= self.control_devices.choice_window(ctx, &mut own);
        if changed {
            self.doc.device.control_device = own.clone();
            if let Some(d) = self.profiles.get_mut(self.active) {
                d.control_device = own;
            }
        }
    }
}
