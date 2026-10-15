//! The check before a job is sent, framed or exported: errors stop it, warnings can be confirmed.
#[cfg(test)]
use base_ui::editor::EditorCommands;
use crate::app::App;
use crate::i18n::{tr, trf};
use crate::theme;
use eframe::egui::{self, Color32, RichText};
use lc_core::check::{self, Issue, IssueKind, Level};

/// What was asked for while the check window is open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pending {
    Start,
    Frame,
    ExportGcode,
    ExportCam,
    /// The dry run with the pointer: only errors are ever shown, so there is nothing to confirm.
    DryRun,
}

pub struct CheckDlg {
    pub issues: Vec<Issue>,
    pub pending: Pending,
}

/// The text of an issue in the language of the interface; `layers` names the layers of the design.
pub fn describe(i: &Issue, layers: &[String], zones: &[String], frame: bool) -> String {
    let zone = i.zone.and_then(|z| zones.get(z)).cloned().unwrap_or_default();
    let layer = i.layer.and_then(|l| layers.get(l)).cloned().unwrap_or_default();
    match i.kind {
        IssueKind::OutsideArea if frame => tr("The frame is outside the usable area (beyond the work area or in a margin).").to_string(),
        IssueKind::OutsideLimits if frame => tr("The frame is outside the software limits.").to_string(),
        IssueKind::OutsideArea => trf("{} object(s) are outside the usable area (beyond the work area or in a margin).", &[&i.shapes.len()]),
        IssueKind::OutsideLimits => trf("{} object(s) are outside the software limits.", &[&i.shapes.len()]),
        IssueKind::MoveOutside => trf("The toolpath of layer {} leaves the allowed area (overscan or travel moves).", &[&layer]),
        IssueKind::NoGoZone if frame => trf("The frame touches the no-go zone \"{}\".", &[&zone]),
        IssueKind::NoGoZone => trf("The toolpath of layer {} touches the no-go zone \"{}\".", &[&layer, &zone]),
        IssueKind::RotaryMix => tr("Rotary layers and flat layers cannot be in one job: mark all the burning layers as rotary, or none.").to_string(),
        IssueKind::RotaryTooTall => trf("The design on layer {} is higher than the way round the workpiece.", &[&layer]),
        IssueKind::ZOutOfRange => trf("The Z heights of layer {} are outside the travel of the Z axis.", &[&layer]),
        IssueKind::SpeedAboveMax => trf("Layer {} is faster than the maximum cutting speed of the machine.", &[&layer]),
        IssueKind::ZeroPower => trf("Layer {} burns at 0 % power.", &[&layer]),
        IssueKind::NothingToBurn => trf("Layer {} has objects but no toolpath (open lines on a fill layer, for example).", &[&layer]),
    }
}

impl App {
    fn layer_names(&self) -> Vec<String> {
        self.doc.layers.iter().map(|l| l.name.clone()).collect()
    }

    /// Check the job for `what`. `true` means go ahead now; otherwise the window with the issues is open.
    pub fn guard(&mut self, what: Pending, frame_bounds: Option<lc_core::Rect>) -> bool {
        let issues = match (what, frame_bounds) {
            (Pending::Frame, Some(b)) => check::check_frame(&self.doc.device, b),
            (Pending::Start, _) => check::job_check(&self.job_doc().0),
            _ => check::job_check(&self.doc),
        };
        if issues.is_empty() {
            return true;
        }
        self.check_dlg = Some(CheckDlg { issues, pending: what });
        false
    }

    /// The job waits for an `ok` and the machine stands still: the user decides.
    pub fn stall_window(&mut self, ctx: &egui::Context) {
        let Some((secs, state)) = self.stall.clone() else { return };
        let mut open = true;
        let mut stop = false;
        let mut close = false;
        crate::window_ui::popup(ctx, "job_stall", &tr("The controller is not answering"), [480.0, 190.0], &mut open, |ui| {
            ui.label(RichText::new(trf("No answer from the controller for {} s while a job is running (state: {}).", &[&secs, &state])).color(Color32::from_rgb(0xe8, 0x8a, 0x1f)).strong());
            ui.label(tr("Nothing was resumed or sent. The machine may just be slow, or the line was lost; look at the console, then choose."));
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.button(tr("Keep waiting")).clicked() {
                    close = true;
                }
                let b = egui::Button::new(RichText::new(tr("Stop the job")).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0xc0, 0x30, 0x30));
                if ui.add(b).clicked() {
                    stop = true;
                }
            });
        });
        if stop {
            self.abort_job();
        }
        if stop || close || !open {
            self.stall = None;
        }
    }

    /// The warning about what the controller reported after connecting: laser mode off, `$30` unlike the profile.
    pub fn fw_window(&mut self, ctx: &egui::Context) {
        let Some(w) = self.fw_warn.clone() else { return };
        if w.is_clean() {
            self.fw_warn = None;
            return;
        }
        let mut open = true;
        let mut close = false;
        let mut enable = false;
        let mut take_s = None;
        let mine = self.doc.device.s_max();
        crate::window_ui::popup(ctx, "fw_check", &tr("The controller reports a problem"), [520.0, 220.0], &mut open, |ui| {
            if w.laser_mode_off {
                ui.label(RichText::new(tr("Laser mode ($32) is off.")).color(Color32::from_rgb(0xc0, 0x30, 0x30)).strong());
                ui.label(tr("With laser mode off GRBL behaves like a spindle controller: the laser may fire during rapid moves and the power does not follow the speed."));
                if ui.button(tr("Enable laser mode ($32=1)")).clicked() {
                    enable = true;
                }
                ui.add_space(6.0);
            }
            if let Some(v) = w.s_max {
                ui.label(RichText::new(trf("The controller's maximum S value ($30) is {}, the profile uses {}.", &[&v, &mine])).color(Color32::from_rgb(0xe8, 0x8a, 0x1f)).strong());
                ui.label(tr("With different values the power in the G-code will not match the percentages you set."));
                if ui.button(tr("Use the value from the controller")).clicked() {
                    take_s = Some(v);
                }
                ui.add_space(6.0);
            }
            if ui.button(tr("Close")).clicked() {
                close = true;
            }
        });
        if enable {
            self.confirm = Some(crate::session_ui::Confirm {
                title: tr("Enable laser mode ($32=1)").to_string(),
                text: tr("This writes $32=1 to the controller's settings. Continue?").to_string(),
                act: crate::session_ui::ConfirmAct::EnableLaserMode,
            });
        }
        if let Some(v) = take_s {
            self.doc.device.heads[0].s_max = v;
            self.sync_profile();
            if let Some(w) = &mut self.fw_warn {
                w.s_max = None;
            }
        }
        if !open || close {
            self.fw_warn = None;
        }
    }

    pub fn check_window(&mut self, ctx: &egui::Context) {
        let Some(dlg) = &self.check_dlg else { return };
        let names = self.layer_names();
        let zone_names: Vec<String> = self.doc.device.zones.iter().map(|z| z.name.clone()).collect();
        let frame = dlg.pending == Pending::Frame;
        let blocked = check::has_error(&dlg.issues);
        let mut open = true;
        let mut go = false;
        let mut close = false;
        let mut select: Option<Vec<u64>> = None;
        crate::window_ui::popup(ctx, "job_check", &tr("Check before the job"), [520.0, 260.0], &mut open, |ui| {
            for i in &dlg.issues {
                ui.horizontal_wrapped(|ui| {
                    let (icon, colour) = match i.level {
                        Level::Error => ("✖", Color32::from_rgb(0xc0, 0x30, 0x30)),
                        Level::Warning => ("⚠", Color32::from_rgb(0xe8, 0x8a, 0x1f)),
                    };
                    ui.label(RichText::new(icon).color(colour).strong());
                    ui.label(describe(i, &names, &zone_names, frame));
                    if !i.shapes.is_empty() && ui.small_button(tr("Select objects")).clicked() {
                        select = Some(i.shapes.clone());
                    }
                });
            }
            ui.add_space(8.0);
            if blocked {
                ui.label(RichText::new(tr("Fix the errors first: nothing is sent, framed or exported with them.")).color(theme::text_dim()));
            }
            ui.horizontal(|ui| {
                if !blocked {
                    let b = egui::Button::new(RichText::new(tr("Continue anyway")).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0x2e, 0xa0, 0x4f));
                    if ui.add(b).clicked() {
                        go = true;
                    }
                }
                if ui.button(tr("Close")).clicked() {
                    close = true;
                }
            });
        });
        if let Some(ids) = select {
            self.ed.sel = ids;
        }
        if go {
            if let Some(d) = self.check_dlg.take() {
                match d.pending {
                    Pending::Start => self.send_job_now(),
                    Pending::Frame => self.frame_now(),
                    Pending::ExportGcode => self.export_gcode_now(),
                    Pending::ExportCam => self.export_cam_now(),
                    Pending::DryRun => {}
                }
            }
        } else if !open || close {
            self.check_dlg = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lc_core::{Kind, Xf};

    fn app_with_rect(x: f64) -> App {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.screen = crate::app::Screen::Editor;
        a.profiles = vec![lc_core::Device::default()];
        a.enter_editor(0);
        a.doc.add(0, Kind::Rect { w: 20.0, h: 10.0 }, Xf::translate(x, 100.0));
        a
    }

    #[test]
    fn start_frame_and_export_are_blocked_by_an_object_outside_the_area() {
        // The default work area is 400 mm wide; the rectangle ends at 410 mm.
        let mut a = app_with_rect(390.0);
        a.connected = true;
        a.send_job();
        let dlg = a.check_dlg.take().expect("the check window opens");
        assert_eq!((dlg.pending, dlg.issues[0].kind), (Pending::Start, IssueKind::OutsideArea));
        a.select_all();
        a.frame();
        let dlg = a.check_dlg.take().expect("frame is checked too");
        assert_eq!(dlg.pending, Pending::Frame);
        assert!(!a.guard(Pending::ExportGcode, None));
        assert!(check::has_error(&a.check_dlg.as_ref().unwrap().issues));
    }

    #[test]
    fn a_job_inside_the_area_goes_ahead() {
        let mut a = app_with_rect(100.0);
        assert!(a.guard(Pending::ExportGcode, None));
        assert!(a.check_dlg.is_none());
    }

    #[test]
    fn issues_are_described_in_words() {
        let names: Vec<String> = (0..lc_core::LAYER_COUNT).map(|i| format!("C{i:02}")).collect();
        let i = Issue { level: Level::Warning, kind: IssueKind::ZeroPower, shapes: vec![1], layer: Some(3), zone: None };
        assert!(describe(&i, &names, &[], false).contains("C03"));
    }
}

#[cfg(test)]
mod fw_tests {
    use super::*;
    use crate::device_ui::ReadCfg;
    use lc_core::Controller;

    fn probe(lines: &[&str]) -> App {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.screen = crate::app::Screen::Editor;
        a.profiles = vec![lc_core::Device::default()];
        a.enter_editor(0);
        let now = std::time::Instant::now();
        a.read_cfg = Some(ReadCfg { controller: Controller::Grbl, waiting_connect: false, request_at: None, sent_at: Some(now), lines: lines.iter().map(|l| l.to_string()).collect(), started: now, kind: crate::device_ui::ReadKind::Probe });
        a.finish_read();
        a
    }

    #[test]
    fn laser_mode_off_and_a_different_s_max_raise_the_warning() {
        let a = probe(&["$30=255", "$32=0"]);
        let w = a.fw_warn.expect("warning");
        assert!(w.laser_mode_off);
        assert_eq!(w.s_max, Some(255.0));
        // The probe never touches the profile by itself.
        assert_eq!(a.doc.device.s_max(), 1000.0);
    }

    #[test]
    fn a_healthy_controller_raises_nothing_and_a_probe_does_not_change_the_status() {
        let a = probe(&["$30=1000", "$32=1"]);
        assert!(a.fw_warn.is_none());
    }

    #[test]
    fn confirming_laser_mode_clears_that_part_of_the_warning() {
        let mut a = probe(&["$32=0", "$30=255"]);
        a.do_confirmed(&egui::Context::default(), crate::session_ui::ConfirmAct::EnableLaserMode);
        let w = a.fw_warn.expect("the $30 part remains");
        assert!(!w.laser_mode_off && w.s_max == Some(255.0));
    }
}

#[cfg(test)]
mod reconnect_tests {
    use super::*;

    fn app() -> App {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.screen = crate::app::Screen::Editor;
        let mut d = lc_core::Device::default();
        d.connection.port = "/dev/does-not-exist".into();
        d.connection.auto_reconnect = true;
        a.profiles = vec![d];
        a.enter_editor(0);
        a
    }

    #[test]
    fn a_lost_link_is_opened_again_only_when_asked_for_and_not_after_a_disconnect() {
        let ctx = egui::Context::default();
        let mut a = app();
        a.want_link = true;
        a.reconnect_at = Some(std::time::Instant::now() - std::time::Duration::from_secs(1));
        a.reconnect_tick(&ctx);
        assert_eq!(a.status, tr("Connecting again…"));
        let next = a.reconnect_at.expect("another try is scheduled");
        assert!(next > std::time::Instant::now());
        // The user pressed Disconnect: nothing more happens.
        a.want_link = false;
        a.reconnect_tick(&ctx);
        assert!(a.reconnect_at.is_none());
        // The option is off: no tries either.
        let mut b = app();
        b.want_link = true;
        b.doc.device.connection.auto_reconnect = false;
        b.reconnect_at = Some(std::time::Instant::now() - std::time::Duration::from_secs(1));
        b.reconnect_tick(&ctx);
        assert!(b.reconnect_at.is_none());
    }
}

#[cfg(test)]
mod connect_tests {
    use super::*;
    use crate::device_ui::{ReadCfg, ReadKind};

    fn app(home: bool, unlock: bool) -> App {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.screen = crate::app::Screen::Editor;
        let mut d = lc_core::Device::default();
        d.connection.home_on_connect = home;
        d.connection.unlock_on_connect = unlock;
        a.profiles = vec![d];
        a.enter_editor(0);
        a
    }

    #[test]
    fn the_actions_after_connecting_run_once_and_only_when_asked_for() {
        let ctx = egui::Context::default();
        let mut quiet = app(false, false);
        quiet.read_on_connected(true);
        assert!(quiet.connect_actions_at.is_none());
        let mut a = app(true, true);
        a.connected = true;
        a.read_on_connected(true);
        assert!(a.connect_actions_at.is_some());
        a.connect_actions_at = Some(std::time::Instant::now() - std::time::Duration::from_millis(1));
        a.connect_actions_tick(&ctx);
        assert!(a.connect_actions_at.is_none(), "done once");
        // Losing the link cancels what was planned.
        a.read_on_connected(true);
        a.read_on_connected(false);
        assert!(a.connect_actions_at.is_none());
    }

    #[test]
    fn single_axis_homing_is_known_from_the_build_options() {
        let lines: Vec<String> = ["[VER:1.1h.20190825:]", "[OPT:VH,15,128]"].iter().map(|s| s.to_string()).collect();
        assert_eq!(lc_core::controller::grbl_options(&lines).as_deref(), Some("VH"));
        assert!(lc_core::controller::single_axis_homing("VH"));
        let plain = vec!["[OPT:V,15,128]".to_string()];
        assert!(!lc_core::controller::single_axis_homing(&lc_core::controller::grbl_options(&plain).unwrap()));
        assert_eq!(lc_core::controller::grbl_options(&[]), None);
        // The app learns it from the Info read and forgets it when the link goes down.
        let mut a = app(false, false);
        let now = std::time::Instant::now();
        a.read_cfg = Some(ReadCfg { controller: lc_core::Controller::Grbl, waiting_connect: false, request_at: None, sent_at: Some(now), lines, started: now, kind: ReadKind::Info });
        a.finish_read();
        assert!(a.single_axis_homing);
        a.read_on_connected(false);
        assert!(!a.single_axis_homing);
    }
}

#[cfg(test)]
mod stamp_tests {
    use super::*;

    #[test]
    fn the_job_carries_the_file_name_and_the_time_for_hooks() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.path = Some(std::path::PathBuf::from("/tmp/projects/badge.cre"));
        a.stamp_job();
        let i = &a.doc.job_info;
        assert_eq!(i.name, "badge");
        // 2026-... and 14:05 in form, whatever the moment is.
        assert!(i.date.len() == 10 && i.date.as_bytes()[4] == b'-' && i.date.as_bytes()[7] == b'-', "{}", i.date);
        assert!(i.time.len() == 5 && i.time.as_bytes()[2] == b':', "{}", i.time);
        // Not saved with the project.
        assert!(!a.doc.to_json().contains("badge"));
    }
}

#[cfg(test)]
mod macro_tests {
    use super::*;
    use crate::session_ui::ConfirmAct;

    fn app() -> App {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.screen = crate::app::Screen::Editor;
        let mut d = lc_core::Device::default();
        d.macros = vec![lc_core::Macro { name: "Park".into(), code: "G0 X0 Y0 ; park\nM5".into() }];
        a.profiles = vec![d];
        a.enter_editor(0);
        a.connected = true;
        a
    }

    #[test]
    fn the_first_run_of_a_macro_in_a_session_asks_and_later_ones_do_not() {
        let mut a = app();
        a.request_macro("Park");
        let c = a.confirm.take().expect("the first run asks");
        assert!(matches!(c.act, ConfirmAct::RunMacro(ref n) if n == "Park"));
        assert!(!a.macros_ok);
        a.do_confirmed(&egui::Context::default(), c.act);
        assert!(a.macros_ok);
        // The console tells what was sent; the second run does not ask.
        let before = a.console.len();
        a.request_macro("Park");
        assert!(a.confirm.is_none());
        assert!(a.console.len() > before);
    }

    #[test]
    fn nothing_runs_without_a_connection_or_during_a_job_and_errors_are_reported() {
        let mut a = app();
        a.macros_ok = true;
        a.connected = false;
        let n = a.console.len();
        a.run_macro("Park");
        assert_eq!(a.console.len(), n);
        a.connected = true;
        a.progress = (2, 10);
        a.run_macro("Park");
        assert_eq!(a.console.len(), n);
        a.progress = (0, 0);
        a.run_macro("Missing");
        assert!(a.status.contains("Missing"), "{}", a.status);
        assert_eq!(a.console.len(), n);
    }
}

#[cfg(test)]
mod zone_tests {
    use super::*;
    use crate::menu::Act;
    use lc_core::{Kind, Xf, Zone, ZoneShape};

    fn app() -> App {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.screen = crate::app::Screen::Editor;
        let mut d = lc_core::Device::default();
        d.zones = vec![Zone { name: "Clamp".into(), enabled: true, shape: ZoneShape::Rect { x: 100.0, y: 100.0, w: 30.0, h: 30.0 }, z_range: None }];
        a.profiles = vec![d];
        a.enter_editor(0);
        a.connected = true;
        a.doc.add(0, Kind::Rect { w: 20.0, h: 10.0 }, Xf::translate(110.0, 105.0));
        a
    }

    #[test]
    fn a_job_through_a_zone_is_stopped_everywhere_and_the_message_names_the_zone() {
        let mut a = app();
        a.send_job();
        let dlg = a.check_dlg.take().expect("Start is stopped");
        assert!(dlg.issues.iter().any(|i| i.kind == IssueKind::NoGoZone));
        assert!(!a.guard(Pending::ExportGcode, None));
        let issue = a.check_dlg.take().unwrap().issues.into_iter().find(|i| i.kind == IssueKind::NoGoZone).unwrap();
        let text = describe(&issue, &(0..lc_core::LAYER_COUNT).map(|i| format!("C{i:02}")).collect::<Vec<_>>(), &["Clamp".to_string()], false);
        assert!(text.contains("Clamp") && text.contains("C00"), "{text}");
        a.select_all();
        a.frame();
        assert!(a.check_dlg.take().is_some_and(|d| d.issues.iter().any(|i| i.kind == IssueKind::NoGoZone)), "the frame is checked too");
        // Switch the zone off and the same job goes ahead.
        a.doc.device.zones[0].enabled = false;
        assert!(a.guard(Pending::ExportGcode, None));
    }

    #[test]
    fn the_zones_can_be_shown_or_hidden_from_the_menu_when_there_are_any() {
        let ctx = egui::Context::default();
        let mut a = app();
        assert!(a.act_enabled(Act::ToggleZones));
        assert_eq!(a.act_checked(Act::ToggleZones), Some(true));
        a.do_act(&ctx, Act::ToggleZones);
        assert_eq!(a.act_checked(Act::ToggleZones), Some(false));
        a.doc.device.zones.clear();
        assert!(!a.act_enabled(Act::ToggleZones));
        assert!(a.settings_pairs().iter().any(|(k, v)| *k == "zones_visible" && v == "0"));
    }
}

#[cfg(test)]
mod z_tests {
    use super::*;

    fn app() -> App {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.screen = crate::app::Screen::Editor;
        let mut d = lc_core::Device::default();
        d.axes.has_z = true;
        d.axes.z_min = 0.0;
        d.axes.z_max = 50.0;
        d.jog_step = 5.0;
        a.profiles = vec![d];
        a.enter_editor(0);
        a.connected = true;
        a
    }

    #[test]
    fn the_z_jog_stops_at_the_travel_and_follows_the_direction_setting() {
        use lc_core::controller::Action;
        let jog = |a: &mut App, dir: f64| a.jog_z_action(dir).map(|act| match act {
            Action::Lines(l) => l.join(" | "),
            _ => String::new(),
        });
        let mut a = app();
        a.machine_z = 10.0;
        assert_eq!(jog(&mut a, 1.0).unwrap(), "$J=G91 G21 Z5 F3000");
        a.machine_z = 48.0;
        assert_eq!(jog(&mut a, 1.0).unwrap(), "$J=G91 G21 Z2 F3000", "shortened to the end of the travel");
        a.machine_z = 50.0;
        assert!(jog(&mut a, 1.0).is_none(), "at the top");
        assert!(jog(&mut a, -1.0).is_some());
        a.machine_z = 3.0;
        assert_eq!(jog(&mut a, -1.0).unwrap(), "$J=G91 G21 Z-3 F3000");
        // Up is the negative direction on this machine.
        a.doc.device.axes.flip_z = true;
        a.machine_z = 10.0;
        assert_eq!(jog(&mut a, 1.0).unwrap(), "$J=G91 G21 Z-5 F3000");
        // Without a reported position the step is taken as it is.
        a.machine_z = f64::NAN;
        assert_eq!(jog(&mut a, -1.0).unwrap(), "$J=G91 G21 Z5 F3000");
    }
}

#[cfg(test)]
mod maintenance_tests {
    use super::*;
    use crate::session_ui::ConfirmAct;
    use lc_core::Counter;
    use std::time::{Duration, Instant};

    fn app() -> App {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.screen = crate::app::Screen::Editor;
        let mut d = lc_core::Device::default();
        d.maintenance.counters = vec![Counter { name: "Tube".into(), secs: 0.0, threshold_hours: Some(0.01) }, Counter { name: "Lens".into(), secs: 0.0, threshold_hours: None }];
        a.profiles = vec![d];
        a.enter_editor(0);
        a.connected = true;
        a
    }

    #[test]
    fn the_work_time_is_the_time_a_job_really_ran_and_the_counters_follow() {
        let mut a = app();
        // A job starts: the clock starts with it.
        a.progress = (2, 10);
        a.job_clock_tick();
        assert!(a.job_started.is_some());
        // 90 s later the last line is acknowledged but the machine still moves: the clock keeps running.
        a.job_started = Some(Instant::now() - Duration::from_secs(90));
        a.progress = (10, 10);
        a.machine.0 = "Run".into();
        a.job_clock_tick();
        assert!(a.job_started.is_some() && a.doc.device.maintenance.total_secs == 0.0);
        // The machine is idle: the time is counted, in the total and in every counter, and kept in the profile.
        a.machine.0 = "Idle".into();
        a.job_clock_tick();
        assert!(a.job_started.is_none());
        let m = &a.doc.device.maintenance;
        assert!((m.total_secs - 90.0).abs() < 1.0, "{}", m.total_secs);
        assert!((m.counters[0].secs - m.total_secs).abs() < 1e-9 && (m.counters[1].secs - m.total_secs).abs() < 1e-9);
        assert_eq!(a.profiles[0].maintenance, *m);
        // The tube has a limit of 36 s: it is due, announced once.
        assert!(m.counters[0].is_due() && !m.counters[1].is_due());
        assert!(a.status.contains("Tube"), "{}", a.status);
        a.status.clear();
        a.maintenance_add(10.0);
        assert!(a.status.is_empty(), "announced only once per session: {}", a.status);
    }

    #[test]
    fn a_stopped_job_or_a_lost_link_counts_what_ran_and_nothing_runs_without_a_job() {
        let mut a = app();
        a.job_clock_tick();
        assert!(a.job_started.is_none() && a.doc.device.maintenance.total_secs == 0.0);
        a.progress = (3, 10);
        a.job_clock_tick();
        a.job_started = Some(Instant::now() - Duration::from_secs(30));
        a.progress = (0, 0); // Stop
        a.job_clock_tick();
        assert!((a.doc.device.maintenance.total_secs - 30.0).abs() < 1.0);
        a.progress = (1, 5);
        a.job_clock_tick();
        a.job_started = Some(Instant::now() - Duration::from_secs(10));
        a.connected = false;
        a.job_clock_tick();
        assert!((a.doc.device.maintenance.total_secs - 40.0).abs() < 1.0);
    }

    #[test]
    fn a_counter_can_be_reset_but_the_total_stays_and_copies_start_at_zero() {
        let mut a = app();
        a.maintenance_add(100.0);
        a.open_cfg_edit(0);
        a.do_confirmed(&egui::Context::default(), ConfirmAct::ResetCounter(0));
        assert_eq!(a.doc.device.maintenance.counters[0].secs, 0.0);
        assert!((a.doc.device.maintenance.counters[1].secs - 100.0).abs() < 1e-9);
        assert!((a.doc.device.maintenance.total_secs - 100.0).abs() < 1e-9, "the total cannot be reset");
        assert_eq!(a.profiles[0].maintenance.counters[0].secs, 0.0);
        // A copy of the profile starts a new life: no work time, same counters.
        a.duplicate_profile(0);
        let copy = &a.profiles[1].maintenance;
        assert_eq!((copy.total_secs, copy.counters.len(), copy.counters[1].secs), (0.0, 2, 0.0));
        assert_eq!(copy.counters[0].name, "Tube");
    }
}
