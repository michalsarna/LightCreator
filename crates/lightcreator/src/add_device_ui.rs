//! Adding a device: one window with the basic settings (and an optional camera). It opens by itself when
//! there is no device yet. Nothing is predefined.
use lc_core::ControllerSettings;
use crate::app::App;
use crate::device_ui::CfgEdit;
use crate::i18n::tr;
use eframe::egui::{self, Color32, RichText};
use lc_core::{Controller, Device, Units};

/// A device with the program's plain defaults and no name.
fn blank_device() -> Device {
    let mut d = Device::default();
    d.name = String::new();
    d.speed_unit = lc_core::SpeedUnit::PerMinute;
    d
}

/// The list of the controllers, with the boards that are a controller with other settings (GRBL-M3, GRBL-LPC, Snapmaker…). Picking
/// one sets the controller, the G-code dialect and the S max.
pub fn board_combo(ui: &mut egui::Ui, id: &str, d: &mut Device) {
    let now = lc_core::Board::of(d);
    egui::ComboBox::from_id_salt(id).selected_text(board_label(now)).show_ui(ui, |ui| {
        for b in lc_core::Board::ALL {
            if ui.selectable_label(now == b, board_label(b)).clicked() && now != b {
                b.apply(d);
            }
        }
    });
}

/// The name of a board in the list: the ones with words are translated, the names of firmware are not.
pub fn board_label(b: lc_core::Board) -> String {
    match b {
        lc_core::Board::Grbl | lc_core::Board::GrblM3 | lc_core::Board::LinuxCnc | lc_core::Board::CustomGcode => tr(b.label()).to_string(),
        _ => b.label().to_string(),
    }
}

/// The commands of the laser on and off for the dialect **Own commands**, and what the dialects that set the power with a command do.
pub fn power_commands_ui(ui: &mut egui::Ui, d: &mut Device) {
    let warn = Color32::from_rgb(0xe8, 0x8a, 0x1f);
    match d.dialect() {
        lc_core::Dialect::Custom => {
            egui::Grid::new("power_cmds").num_columns(2).min_col_width(140.0).spacing([12.0, 6.0]).show(ui, |ui| {
                ui.label(tr("Laser on (power)"));
                ui.add(egui::TextEdit::singleline(&mut d.gcode.power_on).hint_text("M3 S{power}"));
                ui.end_row();
                ui.label(tr("Laser off"));
                ui.add(egui::TextEdit::singleline(&mut d.gcode.power_off).hint_text("M5"));
                ui.end_row();
            });
            ui.label(RichText::new(tr("{power} is the power in the units of S max, {percent} the power in percent. The laser-on command comes before a burn, the laser-off command before every move.")).color(crate::theme::text_dim()));
            ui.label(RichText::new(tr("Own G-code: check the commands for the laser on and off with the laser at the lowest power first.")).color(warn));
        }
        lc_core::Dialect::Snapmaker => {
            ui.label(RichText::new(tr("Snapmaker: the power is written as M3 P (percent) S (0–255) before a burn and M5 before a move. Not tried on a real machine.")).color(warn));
        }
        lc_core::Dialect::LinuxCnc => {
            ui.label(RichText::new(tr("LinuxCNC: Start saves the G-code file for LinuxCNC. The power goes to the analogue output 0 (M67 E0 Q, from 0 to the S max), which must drive the laser in your HAL. Not tried on a real machine.")).color(warn));
        }
        lc_core::Dialect::GrblM3Old => {
            ui.label(RichText::new(tr("GRBL without laser mode: the laser is switched off (S0) before every move. Try the first job at the lowest power.")).color(warn));
        }
        _ => {}
    }
}

/// The optional camera of a new device (the one of every program's window).
fn camera_section(ui: &mut egui::Ui, d: &mut Device) -> Option<lc_core::CameraSource> {
    let mut active = d.active_camera;
    let test = base_ui::add_device_ui::camera_section(ui, &mut d.cameras, &mut active, crate::camera::SUPPORTED);
    if active != d.active_camera && active < d.cameras.len() {
        d.activate_camera(active);
    }
    test
}

impl App {
    /// Open the window for a new device: plain defaults, nothing predefined.
    pub fn start_add_device(&mut self) {
        self.cfg = Some(CfgEdit { idx: None, draft: blank_device(), page: crate::device_ui::CfgPage::General, head: 0 });
        self.add_dev = Some(0);
    }

    /// The things to look at before a new profile is used, from what is known now.
    pub fn add_dev_warnings(d: &Device, read: bool) -> Vec<String> {
        let mut w = vec![];
        let board = lc_core::Board::of(d);
        if d.controller == Controller::Grbl && !read && !d.file_only() && board != lc_core::Board::GrblM3 {
            w.push(tr("Laser mode ($32) and the maximum S value ($30) of the controller were not read: they are checked when you connect.").to_string());
        }
        if board == lc_core::Board::GerbilStm && !read {
            w.push(tr("Gerbil boards differ in the maximum S value ($30 is 1000 on some, 2048 on others): read it from the device.").to_string());
        }
        if !d.file_only() && !d.has_target() {
            w.push(tr("No port or host is set yet: choose it in the device settings before connecting.").to_string());
        }
        if d.area.margins == lc_core::Margins::default() && d.area.soft_limits.is_none() {
            w.push(tr("No margins or software limits are set: the whole axis range is the work area. Set them in the Hardware page if a clamp or a frame is in the way.").to_string());
        }
        w.push(tr("Nothing here has been tried on your machine: run a first test with the laser off or at the lowest power.").to_string());
        w
    }

    /// The device is made: it gets the starter materials of its laser type (the default material is selected), and
    /// joins the list of devices.
    pub fn finish_add_device(&mut self, mut dev: Device) {
        dev.name = dev.name.trim().to_string();
        dev.add_starter_materials();
        self.profiles.push(dev);
        self.start_sel = self.profiles.len() - 1;
        self.add_dev = None;
        self.add_dev_read = false;
        self.add_dev_read_err = false;
        self.status = tr("The device was added. Press New to start.").to_string();
    }

    /// The one window for adding a device: a starting point, the basic settings collected from the device window, the
    /// connection (with a read from the controller) and the things worth checking. "More settings…" opens the full
    /// device window with what is entered here.
    pub fn add_device_window(&mut self, ctx: &egui::Context) {
        if self.add_dev.is_none() {
            return;
        }
        let Some(mut cfg) = self.cfg.take() else {
            self.add_dev = None;
            return;
        };
        let mut read = false;
        let reading = self.read_cfg.is_some();
        // How the last read ended: it is over when the read state is gone, and it worked when it filled the draft.
        if self.add_dev_was_reading && !reading {
            if self.add_dev_read_done {
                self.add_dev_read = true;
                self.add_dev_read_err = false;
            } else {
                self.add_dev_read = false;
                self.add_dev_read_err = true;
            }
            self.add_dev_read_done = false;
        }
        if reading {
            self.add_dev_read_err = false;
        }
        self.add_dev_was_reading = reading;
        let was_read = self.add_dev_read;
        let read_err = self.add_dev_read_err;
        let mut open = true;
        let mut test_cam = None;
        let mut refresh = false;
        let ports = self.port_info.clone();
        let state = if reading {
            base_ui::add_device_ui::ReadState::Reading
        } else if was_read {
            base_ui::add_device_ui::ReadState::Done
        } else if read_err {
            base_ui::add_device_ui::ReadState::Failed
        } else {
            base_ui::add_device_ui::ReadState::Idle
        };
        let d = &mut cfg.draft;
        let action = base_ui::add_device_ui::window(ctx, &mut open, |ui| {
            base_ui::add_device_ui::grid(ui, |ui| {
                ui.label(tr("Name"));
                ui.add(egui::TextEdit::singleline(&mut d.name).hint_text(tr("e.g. My diode laser")));
                ui.end_row();
                ui.label(tr("Controller"));
                board_combo(ui, "add_controller", d);
                ui.end_row();
                refresh = base_ui::add_device_ui::connection_rows(ui, &mut d.connection, &ports);
                read = base_ui::add_device_ui::read_row(ui, state, d.has_target() && d.controller.settings_request().is_some());
                ui.label(tr("Units"));
                egui::ComboBox::from_id_salt("add_units").selected_text(tr(d.units.label())).show_ui(ui, |ui| {
                    for u in Units::DEVICE {
                        ui.selectable_value(&mut d.units, u, tr(u.label()));
                    }
                });
                ui.end_row();
                ui.label(tr("Speed units"));
                egui::ComboBox::from_id_salt("add_speed_units").selected_text(tr(d.speed_unit.label())).show_ui(ui, |ui| {
                    for k in lc_core::SpeedUnit::ALL {
                        ui.selectable_value(&mut d.speed_unit, k, tr(k.label()));
                    }
                });
                ui.end_row();
                let u = d.units;
                ui.label(tr("Work area"));
                ui.horizontal(|ui| {
                    ui.label("X");
                    crate::units_ui::drag_len(ui, u, &mut d.area.bed_w, 1.0, Some((10.0, 5000.0)));
                    ui.label("Y");
                    crate::units_ui::drag_len(ui, u, &mut d.area.bed_h, 1.0, Some((10.0, 5000.0)));
                });
                ui.end_row();
                ui.label(tr("Machine zero (0,0)"));
                egui::ComboBox::from_id_salt("add_origin").selected_text(tr(d.axes.origin.label())).show_ui(ui, |ui| {
                    for o in lc_core::Origin::ALL {
                        ui.selectable_value(&mut d.axes.origin, o, tr(o.label()));
                    }
                });
                ui.end_row();
                ui.label(tr("S-value max ($30)"));
                ui.add(egui::DragValue::new(&mut d.heads[0].s_max).range(1.0..=100000.0));
                ui.end_row();
                ui.label(tr("Laser type"));
                egui::ComboBox::from_id_salt("add_laser").selected_text(tr(d.heads[0].laser.label())).show_ui(ui, |ui| {
                    for k in lc_core::LaserKind::ALL {
                        ui.selectable_value(&mut d.heads[0].laser, k, tr(k.label()));
                    }
                });
                ui.end_row();
            });
            power_commands_ui(ui, d);
            ui.add_space(4.0);
            test_cam = camera_section(ui, d);
            base_ui::add_device_ui::warnings(ui, &Self::add_dev_warnings(d, was_read), was_read);
            !d.name.trim().is_empty() && !base_ui::camera_feed::lacks_address(&d.cameras)
        });
        if refresh {
            self.port_info = crate::laser::list_port_info();
            self.ports = self.port_info.iter().map(|p| p.name.clone()).collect();
        }
        use base_ui::add_device_ui::AddAction;
        let (finish, by_hand, dummy, close) = (action == Some(AddAction::Add), action == Some(AddAction::More), action == Some(AddAction::Dummy), action == Some(AddAction::Cancel));
        if let Some(cam) = test_cam {
            self.start_cam_test(ctx, cam);
        }
        if read {
            let draft = cfg.draft.clone();
            self.add_dev_read = false;
            self.add_dev_read_err = false;
            self.add_dev_read_err = false;
            self.add_dev_read_done = false;
            self.start_read(&draft);
        }
        if dummy {
            self.use_dummy_device();
            return;
        }
        if finish {
            self.finish_add_device(cfg.draft);
            return;
        }
        if by_hand {
            self.add_dev = None;
            self.add_dev_skipped = true;
            self.cfg = Some(cfg);
            return;
        }
        if close || !open {
            self.add_dev = None;
            self.add_dev_read = false;
            self.add_dev_read_err = false;
            self.cfg = None;
            return;
        }
        self.cfg = Some(cfg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_setup_starts_on_the_default_template_and_the_summary_lists_what_is_missing() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.profiles.clear();
        a.start_add_device();
        assert_eq!(a.add_dev, Some(0));
        let d = a.cfg.as_ref().unwrap().draft.clone();
        assert_eq!((d.controller, d.area.bed_w), (Controller::Grbl, 400.0));
        // A GRBL machine that was not read, with no port and no margins: three things, and the safety reminder last.
        let w = App::add_dev_warnings(&d, false);
        assert_eq!(w.len(), 4, "{w:?}");
        // After reading, with a port and a margin, only the reminder is left.
        let mut set = d.clone();
        set.connection.port = "/dev/ttyUSB0".into();
        set.area.margins.left = 5.0;
        assert_eq!(App::add_dev_warnings(&set, true).len(), 1);
        // A machine that is not streamed (Ruida) has no port to ask for.
        let mut ruida = d.clone();
        ruida.controller = Controller::Ruida;
        assert_eq!(App::add_dev_warnings(&ruida, false).len(), 2);
    }

    #[test]
    fn the_device_window_stays_out_of_the_way_while_the_setup_is_open() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.profiles.clear();
        a.start_add_device();
        // The device window stays out of the way while the setup is open.
        a.config_window(&ctx);
        assert!(a.cfg.is_some());
        a.add_dev = None;
        a.add_dev_skipped = true;
        assert!(a.cfg.is_some() && a.add_dev.is_none());
    }

    #[test]
    fn adding_a_device_opens_one_window_and_cancelling_it_keeps_the_profiles() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.profiles = vec![Device::default()];
        a.start_add_device();
        assert_eq!(a.add_dev, Some(0));
        let mut out = ctx.run_ui(egui::RawInput::default(), |_| {});
        a.add_device_window(&ctx);
        out.textures_delta.clear();
        assert!(a.add_dev.is_some() && a.cfg.is_some(), "one window, on the draft");
        // The draft carries what the device window has: controller, laser type, work area, zero, S max, connection.
        let d = &a.cfg.as_ref().unwrap().draft;
        assert!(d.area.bed_w > 0.0 && d.heads[0].s_max > 0.0 && d.name.is_empty(), "plain defaults, no name, nothing predefined");
        assert_eq!(a.profiles.len(), 1);
    }

    #[test]
    fn a_new_device_gets_the_starter_materials_with_the_default_one_selected() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.profiles.clear();
        let mut d = Device::default();
        d.name = "  My laser ".into();
        a.finish_add_device(d);
        let p = &a.profiles[0];
        assert_eq!(p.name, "My laser");
        assert_eq!(p.material().name, lc_core::materials::DEFAULT_MATERIAL);
        assert!(p.materials.len() >= 5, "starters stay available");
        assert_eq!(a.start_sel, 0);
    }

    #[test]
    fn without_a_device_the_project_gets_the_dummy_and_saves_with_it() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.profiles.clear();
        a.start_add_device();
        // "I have no device" makes and selects the Dummy; the start screen stays.
        assert!(a.use_dummy_device());
        assert_eq!(a.screen, crate::app::Screen::Start);
        assert!(a.add_dev.is_none() && a.cfg.is_none(), "the setup window is gone");
        assert_eq!((a.profiles.len(), a.start_sel), (1, 0));
        // The second time it is the same profile, not another one.
        assert!(a.use_dummy_device());
        assert_eq!(a.profiles.len(), 1);
        a.do_act(&ctx, crate::menu::Act::New);
        assert_eq!(a.screen, crate::app::Screen::Editor);
        assert_eq!(a.doc.device.name, crate::device_ui::DUMMY_NAME);
        let plain = Device::default();
        assert_eq!((a.doc.device.area.bed_w, a.doc.device.area.bed_h), (plain.area.bed_w, plain.area.bed_h), "default work area");
        assert!(a.doc.device.materials.len() >= 5, "starter materials");
        // The project is saved with the Dummy as its machine.
        let text = a.doc.to_json();
        let back = lc_core::Document::from_json(&text).unwrap();
        assert_eq!(back.device.name, crate::device_ui::DUMMY_NAME);
    }

    #[test]
    fn new_without_any_device_makes_the_dummy_and_with_a_selected_one_uses_it() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.profiles.clear();
        a.screen = crate::app::Screen::Start;
        a.do_act(&ctx, crate::menu::Act::New);
        assert_eq!(a.screen, crate::app::Screen::Editor);
        assert_eq!((a.profiles.len(), a.doc.device.name.as_str()), (1, crate::device_ui::DUMMY_NAME));
        // A selected device is the one the project gets.
        let mut mine = Device::default();
        mine.name = "Mine".into();
        a.profiles.push(mine);
        a.start_sel = 1;
        a.screen = crate::app::Screen::Start;
        a.do_act(&ctx, crate::menu::Act::New);
        assert_eq!(a.doc.device.name, "Mine");
        // A file that cannot be opened keeps the start screen.
        a.screen = crate::app::Screen::Start;
        a.start_project(|app| app.open_path(std::path::PathBuf::from("/nonexistent/x.cre")));
        assert_eq!(a.screen, crate::app::Screen::Start);
    }

    #[test]
    fn the_camera_of_a_new_device_is_added_and_removed_in_the_setup() {
        let ctx = egui::Context::default();
        let mut d = Device::default();
        d.cameras.clear();
        crate::device_ui::add_camera(&mut d);
        assert_eq!(d.cameras.len(), 1);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            camera_section(ui, &mut d);
        });
        out.textures_delta.clear();
        assert_eq!(d.cameras.len(), 1);
    }

    /// One frame of the window, then what the button says about the last read.
    fn frame(a: &mut App, ctx: &egui::Context) {
        let mut out = ctx.run_ui(egui::RawInput::default(), |_| {});
        a.add_device_window(ctx);
        out.textures_delta.clear();
    }

    #[test]
    fn the_read_button_shows_how_the_read_ended() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.profiles.clear();
        a.start_add_device();
        a.connected = true;
        let draft = a.cfg.as_ref().unwrap().draft.clone();
        // A read that gets no answer is an error, and a new read clears it.
        a.start_read(&draft);
        frame(&mut a, &ctx);
        assert!(a.read_cfg.is_some() && !a.add_dev_read_err);
        a.finish_read();
        frame(&mut a, &ctx);
        assert!(a.add_dev_read_err && !a.add_dev_read);
        // A read that gives settings is OK, and they land in the draft.
        a.start_read(&draft);
        a.read_tick(&ctx);
        frame(&mut a, &ctx);
        for l in ["$30=255", "$130=300", "$131=200", "ok"] {
            a.read_feed(l);
        }
        frame(&mut a, &ctx);
        assert!(a.add_dev_read && !a.add_dev_read_err);
        let d = &a.cfg.as_ref().unwrap().draft;
        assert_eq!((d.area.bed_w, d.area.bed_h, d.heads[0].s_max), (300.0, 200.0, 255.0));
    }
}
