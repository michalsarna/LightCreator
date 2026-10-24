//! The Control tab of the side panel, the same as in MillCreator and PenCreator (the panel is in baseCreator,
//! `base_ui::control_ui`), driven through what is special to a laser: the controllers (GRBL, Marlin…), the margins and
//! limits of a jog, and a job that counts from the machine's zero. So the positions are shown but the work zero is not moved
//! from here, there is no spindle and no probe, and nothing is sent that the controller of the device does not understand.
use crate::app::App;
use crate::i18n::tr;
use crate::laser::Cmd;
use lc_core::control::{Axis, Control};
use base_ui::control_ui::{self, ControlCtx, ControlTarget, Quick};
use eframe::egui::{self, Color32, RichText};
use lc_core::controller::Action;
use lc_core::Controller;

/// What the last report of the controller said about where the head is.
#[derive(Clone, Debug, Default)]
pub struct Where {
    pub wpos: Option<[f64; 3]>,
    pub mpos: Option<[f64; 3]>,
    wco: Option<[f64; 3]>,
    pub pins: String,
}

impl Where {
    /// A status report of a GRBL controller came in.
    pub fn take(&mut self, r: &lc_core::grbl_status::Report) {
        if let Some(w) = r.wco {
            self.wco = Some(w);
        }
        self.mpos = lc_core::grbl_status::machine_position(r, self.wco).or(self.mpos);
        self.wpos = lc_core::grbl_status::work_position(r, self.wco).or(self.wpos);
        self.pins = r.pins.clone().unwrap_or_default();
    }

    pub fn forget(&mut self) {
        *self = Where::default();
    }
}

/// The laser as the Control tab sees it.
struct LaserTarget<'a> {
    app: &'a mut App,
}

impl ControlTarget for LaserTarget<'_> {
    fn connected(&self) -> bool {
        self.app.connected
    }
    fn state(&self) -> String {
        if self.app.connected {
            self.app.machine.0.clone()
        } else {
            String::new()
        }
    }
    fn fw_label(&self) -> Option<String> {
        None
    }
    fn alarm_code(&self) -> Option<u32> {
        None
    }
    fn is_alarm(&self) -> bool {
        self.app.connected && self.app.machine.0.starts_with("Alarm")
    }
    fn wpos(&self) -> [f64; 3] {
        self.app.pos.wpos.unwrap_or([f64::NAN; 3])
    }
    fn mpos(&self) -> [f64; 3] {
        // Without a full report only the simple status is known: the position it gives is the machine one.
        self.app.pos.mpos.unwrap_or([self.app.machine.1, self.app.machine.2, self.app.machine_z])
    }
    fn pins(&self) -> String {
        self.app.pos.pins.clone()
    }
    fn wcs(&self) -> u8 {
        1
    }
    fn busy(&self) -> bool {
        self.app.job_running()
    }
    fn keyboard_jog(&self) -> bool {
        self.app.keyboard_jog
    }
    fn set_keyboard_jog(&mut self, on: bool) {
        self.app.keyboard_jog = on;
    }
    // The work zero is not moved from here (`zeroing` is off in the panel): a job counts from the machine's zero.
    fn set_position(&mut self, _values: &[(Axis, f64)]) {}
    fn select_wcs(&mut self, _n: u8) {}
    fn jog(&mut self, dx: f64, dy: f64, dz: f64, feed: f64) {
        // The pad counts Y up towards the back; the laser's Y grows downwards on the screen. The limits, the margins and the
        // controller of the device decide what is sent.
        if dx != 0.0 || dy != 0.0 {
            if let Some(a) = self.app.jog_action_at(dx, -dy, feed) {
                self.app.run_action(a);
            }
        }
        if dz != 0.0 && self.app.doc.device.axes.has_z {
            if let Some(a) = self.app.jog_z_distance(dz, feed) {
                self.app.run_action(a);
            }
        }
    }
    fn cancel_jog(&mut self) {
        let a = self.app.doc.device.controller.cancel_jog();
        self.app.run_action(a);
    }
    fn unlock(&mut self) {
        let a = self.app.doc.device.controller.unlock();
        self.app.run_action(a);
    }
    fn lines(&mut self, ls: Vec<String>) {
        for l in ls {
            self.app.link.send(Cmd::Line(l));
        }
    }
    fn line(&mut self, l: String) {
        self.app.link.send(Cmd::Line(l));
    }
    fn run_macro(&mut self, _code: &str, _safe_z: f64, _job_box: Option<([f64; 3], [f64; 3])>) -> Result<usize, String> {
        Err("the macros of a laser are listed below".into())
    }
    fn probe_closed(&self) -> bool {
        false
    }
    fn last_probe(&self) -> Option<([f64; 3], bool)> {
        None
    }
    fn probe(&mut self, _c: &Control, _axis: Axis, _dir: i8, _tool_diameter: f64) -> Result<(), String> {
        Err("a laser has no probe".into())
    }
    fn spindle_s(&self, rpm: f64, _max_rpm: f64) -> f64 {
        rpm
    }
}

impl App {
    /// Home, Unlock, Pause and Resume, 2 × 2 with icons: in the Device tab under the connection and at the top of the Control tab.
    pub fn quick_row(&mut self, ui: &mut egui::Ui) {
        let ctrl = self.doc.device.controller;
        let moves = self.connected && !self.job_running();
        let action = match control_ui::quick_buttons(ui, moves, self.connected) {
            Some(Quick::Home) => ctrl.home(),
            Some(Quick::Unlock) => ctrl.unlock(),
            Some(Quick::Pause) => ctrl.pause(),
            Some(Quick::Resume) => ctrl.resume(),
            None => Action::None,
        };
        self.run_action(action);
    }

    /// The step and the speed of a jog are the ones of the device (also edited in the Device tab): the panel starts from them,
    /// and what is changed in it goes back, so the two places cannot differ.
    fn sync_jog_settings(&mut self) {
        let fmt = |v: f64| format!("{v}");
        let parse = |t: &str| t.trim().replace(',', ".").parse::<f64>().ok().filter(|v| v.is_finite() && *v >= 0.1 && *v <= 200.0);
        let step = self.doc.device.jog_step;
        match (self.jog_seen_step, parse(&self.ctl.xy_text)) {
            // Changed in the panel since the last frame: the device follows.
            (Some(seen), Some(typed)) if (seen - step).abs() < 1e-9 && (typed - step).abs() > 1e-9 => self.doc.device.jog_step = typed,
            // The device's own step is shown (first frame, or it was changed in the Device tab).
            _ => {
                if parse(&self.ctl.xy_text).map_or(true, |t| (t - step).abs() > 1e-9) {
                    self.ctl.xy_text = fmt(step);
                }
            }
        }
        self.jog_seen_step = Some(self.doc.device.jog_step);
        if let Some(f) = self.ctl.feed_xy.take() {
            self.doc.device.jog_feed = f.clamp(10.0, 20000.0);
        }
    }

    /// The Control tab.
    pub fn control_panel(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().id_salt("tab_control").auto_shrink([false, false]).show(ui, |ui| {
            self.quick_row(ui);
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);
            if ui.add_sized([ui.available_width(), 40.0], egui::Button::new(RichText::new(tr("STOP")).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0xc0, 0x30, 0x30))).clicked() {
                self.abort_job();
            }
            ui.add_space(6.0);
            self.sync_jog_settings();
            let dev = &self.doc.device;
            let mut control = Control::default();
            control.jog.xy_feed = dev.jog_feed;
            control.jog.z_feed = dev.jog_feed;
            let ctx = ControlCtx {
                control: &control,
                safe_z: 0.0,
                tool_diameter: 0.0,
                job_box: None,
                probe: false,
                travel: [dev.area.bed_w, dev.area.bed_h, (dev.axes.z_max - dev.axes.z_min).abs()],
                spindle: None,
                zeroing: false,
                z_axis: dev.axes.has_z,
                hold_to_move: dev.controller == Controller::Grbl,
            };
            let mut st = std::mem::take(&mut self.ctl);
            let say = control_ui::panel(ui, &mut LaserTarget { app: self }, &mut st, &ctx);
            self.ctl = st;
            if let Some(s) = say {
                self.status = s;
            }
            self.control_extras(ui);
        });
    }

    /// What a laser has beyond the shared panel: the focus beam, the homing of one axis, the macros of the device and the
    /// pointer of a red-dot laser.
    fn control_extras(&mut self, ui: &mut egui::Ui) {
        let running = self.job_running();
        ui.add_space(4.0);
        self.focus_button(ui, true);
        if self.connected && self.single_axis_homing {
            // Only when the firmware reported it was built with `$HX` / `$HY`.
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.add_enabled_ui(!running, |ui| {
                    for axis in ["X", "Y"] {
                        if ui.small_button(crate::i18n::trf("Home {}", &[&axis])).on_hover_text(crate::i18n::trf("Homes the {} axis only ($H{})", &[&axis, &axis])).clicked() {
                            self.run_action(Action::Lines(vec![format!("$H{axis}")]));
                        }
                    }
                });
            });
        }
        if !self.doc.device.macros.is_empty() {
            ui.add_space(4.0);
            ui.separator();
            ui.label(RichText::new(tr("Macros")).strong());
            ui.horizontal_wrapped(|ui| {
                for m in self.doc.device.macros.clone() {
                    if ui.add_enabled(self.connected && !running, egui::Button::new(&m.name)).on_hover_text(tr("Runs this macro on the machine")).clicked() {
                        self.request_macro(&m.name);
                    }
                }
            });
        }
        if self.doc.device.head(0).has_pointer_offset() {
            ui.add_space(4.0);
            ui.separator();
            ui.checkbox(&mut self.pointer_align, tr("Align to the pointer")).on_hover_text(tr("This session only: the frame and the dry run move back by the offset of the red pointer dot, so the dot lands where the beam would. Jobs and jogging are never moved."));
            if ui.add_enabled(self.connected && !running, egui::Button::new(tr("Dry run with the pointer"))).on_hover_text(tr("Runs the path of the job shifted by the pointer offset with the laser off, to see where it would go.")).clicked() {
                self.pointer_dry_run();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::SideTab;
    use egui_kittest::kittest::Queryable;
    use egui_kittest::Harness;

    fn harness(connected: bool, has_z: bool) -> Harness<'static, Option<App>> {
        let mut h = Harness::builder().with_accessibility_check(false).with_size([420.0, 900.0]).build_ui_state(
            |ui, state: &mut Option<App>| {
                let a = state.get_or_insert_with(|| App::build(ui.ctx(), None, false));
                egui::CentralPanel::default().show(ui, |ui| a.control_panel(ui));
            },
            None,
        );
        h.run_steps(2);
        let a = h.state_mut().as_mut().unwrap();
        a.connected = connected;
        a.doc.device.axes.has_z = has_z;
        h.run_steps(2);
        h
    }

    #[test]
    fn the_control_tab_is_listed_even_in_a_tab_order_saved_before_it_existed() {
        let tabs = base_ui::side_tabs::SideTabs::new(&SideTab::IDS, "properties,layers,device,console,clipart,description", false);
        assert!(tabs.order().contains(&"control"), "{:?}", tabs.order());
        assert_eq!(SideTab::from_id("control"), Some(SideTab::Control));
    }

    #[test]
    fn the_panel_shows_positions_but_no_zeroing_no_spindle_and_no_probe() {
        let h = harness(true, true);
        assert!(h.query_by_label("Home").is_some() && h.query_by_label("Unlock").is_some());
        assert!(h.query_by_label("Pause").is_some() && h.query_by_label("Resume").is_some());
        assert!(h.query_by_label("STOP").is_some());
        assert!(h.query_by_label("Position").is_some());
        for hidden in ["Zero all", "Zero X and Y", "Go to X0 Y0", "Coordinates", "Spindle", "Probe"] {
            assert!(h.query_by_label(hidden).is_none(), "{hidden} must not be there");
        }
        assert!(h.query_all_by_label("Z").count() > 0, "a machine with a Z axis has the Z rows");
    }

    #[test]
    fn a_laser_without_a_z_axis_has_no_z_buttons() {
        let h = harness(true, false);
        assert_eq!(h.query_all_by_label("Z").count(), 0, "no Z rows: the position, the step, the speed");
    }

    #[test]
    fn the_step_and_the_speed_are_the_ones_of_the_device_in_both_places() {
        let mut h = harness(true, true);
        h.state_mut().as_mut().unwrap().doc.device.jog_step = 2.0;
        h.run_steps(3);
        assert_eq!(h.state().as_ref().unwrap().ctl.xy_text, "2", "the panel starts from the device's step");
        // Typed in the panel: the device follows.
        h.state_mut().as_mut().unwrap().ctl.xy_text = "5".into();
        h.run_steps(3);
        assert_eq!(h.state().as_ref().unwrap().doc.device.jog_step, 5.0);
        // Changed in the Device tab: the panel follows.
        h.state_mut().as_mut().unwrap().doc.device.jog_step = 0.5;
        h.run_steps(3);
        assert_eq!(h.state().as_ref().unwrap().ctl.xy_text, "0.5");
    }

    #[test]
    fn a_jog_from_the_panel_is_limited_like_every_other_jog() {
        let mut a = App::build(&egui::Context::default(), None, false);
        a.connected = true;
        a.machine = ("Idle".into(), 0.0, 0.0);
        // A move of ten metres is cut down to the work area, and the feed that is asked for is the one that is sent.
        let Some(lc_core::controller::Action::Lines(l)) = a.jog_action_at(10_000.0, 0.0, 1000.0) else { panic!("a move was expected") };
        let line = l.join(" ");
        assert!(line.contains("F1000"), "{line}");
        let x: f64 = line.split_whitespace().find_map(|w| w.strip_prefix('X')).and_then(|v| v.parse().ok()).expect("an X word");
        assert!(x.abs() <= a.doc.device.area.bed_w + 1e-6, "{line}");
    }

    #[test]
    fn the_head_position_comes_from_the_report() {
        let mut p = Where::default();
        let r = lc_core::grbl_status::parse("<Idle|MPos:10.000,20.000,0.000|WCO:2.000,3.000,0.000|Pn:P>").expect("a report");
        p.take(&r);
        assert_eq!(p.mpos, Some([10.0, 20.0, 0.0]));
        assert_eq!(p.wpos, Some([8.0, 17.0, 0.0]));
        assert_eq!(p.pins, "P");
        p.forget();
        assert!(p.mpos.is_none());
    }
}
