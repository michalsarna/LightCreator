//! Start screen (device chooser) and the device configuration window.
//! The editor can only be entered once at least one device profile exists and one is selected.
use crate::app::{App, Screen, APP_VERSION};
use crate::laser::{self, Cmd, Target};
use crate::i18n::{tr, trf, Lang};
use crate::menu::Act;
use crate::theme::{self, Scheme};
use eframe::egui::{self, Align, Color32, Layout, RichText};
use crate::units_ui::{drag_len, drag_speed, fmt_len, fmt_speed};
use lc_core::controller::Action;
use lc_core::{Controller, Device, LaserKind, LinkKind, Orientation, Origin, SpeedUnit, Units};

/// An in-progress "read settings from the device" request.
pub struct ReadCfg {
    pub controller: Controller,
    /// Waiting for the connection to come up before asking.
    pub waiting_connect: bool,
    /// When to send the request (a short pause after connecting lets the firmware finish resetting).
    pub request_at: Option<std::time::Instant>,
    pub sent_at: Option<std::time::Instant>,
    pub lines: Vec<String>,
    pub started: std::time::Instant,
    pub kind: ReadKind,
}

/// What a read of the controller's settings is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadKind {
    /// "Read from device" in the configuration window: the values go into the profile being edited.
    Config,
    /// Made after connecting, to check laser mode and `$30`; does not touch any profile.
    Probe,
    /// The build info (`$I`), to learn which options the firmware was built with.
    Info,
    /// The settings for the Firmware ($$) page.
    Firmware,
}

impl ReadCfg {
    fn request(&self) -> Option<&'static str> {
        match self.kind {
            ReadKind::Info => Some("$I"),
            _ => self.controller.settings_request(),
        }
    }
}

/// A device profile being edited in the configuration window.
pub struct CfgEdit {
    /// `None` while creating a new profile.
    pub idx: Option<usize>,
    pub draft: Device,
    pub page: CfgPage,
    /// The laser head shown on the Laser page.
    pub head: usize,
}

/// The pages of the device configuration window.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum CfgPage {
    #[default]
    General,
    Hardware,
    Laser,
    Gcode,
    Connection,
    Camera,
    Rotary,
    Zones,
    Maintenance,
    Notes,
    Advanced,
    Firmware,
}

impl CfgPage {
    pub fn label(self) -> &'static str {
        match self {
            CfgPage::General => "General",
            CfgPage::Hardware => "Hardware",
            CfgPage::Laser => "Laser",
            CfgPage::Gcode => "G-code",
            CfgPage::Connection => "Connection",
            CfgPage::Camera => "Camera",
            CfgPage::Rotary => "Rotary",
            CfgPage::Zones => "No-go zones",
            CfgPage::Maintenance => "Maintenance",
            CfgPage::Notes => "Notes",
            CfgPage::Advanced => "Advanced",
            CfgPage::Firmware => "Firmware ($$)",
        }
    }

    /// The pages offered for a controller: the connection page only exists for the ones with a live link.
    pub fn visible(controller: Controller) -> Vec<CfgPage> {
        let mut v = vec![CfgPage::General, CfgPage::Hardware, CfgPage::Laser, CfgPage::Gcode];
        if controller.is_serial() {
            v.push(CfgPage::Connection);
        }
        v.push(CfgPage::Camera);
        v.push(CfgPage::Rotary);
        v.push(CfgPage::Zones);
        v.push(CfgPage::Maintenance);
        v.push(CfgPage::Notes);
        v.push(CfgPage::Advanced);
        if matches!(controller, Controller::Grbl | Controller::Marlin) {
            v.push(CfgPage::Firmware);
        }
        v
    }
}

impl App {
    /// Switch to the editor with the selected profile. Refuses (returns false) without a valid profile.
    pub fn enter_editor(&mut self, idx: usize) -> bool {
        let Some(dev) = self.profiles.get(idx).cloned() else { return false };
        self.active = idx;
        self.doc.device = dev;
        self.pointer_align = false;
        // The first-run setup belongs to the start screen.
        self.wizard = None;
        self.load_camera_view();
        // A new, empty document takes the layers of the device's selected material; a loaded one keeps its own.
        if self.doc.shapes.is_empty() && self.path.is_none() {
            self.doc.select_material(self.doc.device.active_material);
        }
        self.touch();
        self.view.need_fit = true;
        self.screen = Screen::Editor;
        true
    }

    /// Open the configuration window on a new profile made from `draft`.
    /// A copy of a profile, under a free name, selected.
    pub fn duplicate_profile(&mut self, idx: usize) {
        let Some(src) = self.profiles.get(idx).cloned() else { return };
        let taken: Vec<String> = self.profiles.iter().map(|p| p.name.clone()).collect();
        let mut copy = src;
        copy.name = lc_core::profiles::copy_name(&copy.name, &taken);
        lc_core::profiles::without_history(&mut copy);
        self.status = trf("Copied to {}", &[&copy.name]);
        self.profiles.push(copy);
        self.start_sel = self.profiles.len() - 1;
    }

    /// Save one profile to a file of its own.
    pub fn export_profile_dialog(&mut self, idx: usize) {
        let Some(p) = self.profiles.get(idx) else { return };
        let stem: String = p.name.chars().map(|c| if c.is_alphanumeric() { c } else { '-' }).collect::<String>().trim_matches('-').to_string();
        let name = format!("{}.json", if stem.is_empty() { "device".to_string() } else { stem });
        let Some(path) = rfd::FileDialog::new().add_filter("LightCreator device", &["json"]).set_file_name(&name).save_file() else { return };
        let text = lc_core::profiles::export_profile(p, APP_VERSION);
        self.status = match std::fs::write(&path, text) {
            Ok(_) => trf("Saved the profile to {}", &[&path.display()]),
            Err(e) => trf("Export failed: {}", &[&e]),
        };
    }

    /// Read a profile from a file chosen by the user; it is checked strictly first.
    pub fn import_profile_dialog(&mut self) {
        let Some(path) = rfd::FileDialog::new().add_filter("LightCreator device", &["json"]).pick_file() else { return };
        self.import_profile_from(&path);
    }

    pub fn import_profile_from(&mut self, path: &std::path::Path) {
        let read = match std::fs::metadata(path) {
            Ok(m) if m.len() as usize > lc_core::profiles::MAX_PROFILE_FILE => Err(format!("the file is larger than {} KB", lc_core::profiles::MAX_PROFILE_FILE / 1000)),
            Ok(_) => std::fs::read_to_string(path).map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        };
        match read.and_then(|t| lc_core::profiles::import_profile(&t)) {
            Ok(mut dev) => {
                let taken: Vec<String> = self.profiles.iter().map(|p| p.name.clone()).collect();
                if taken.contains(&dev.name) {
                    dev.name = lc_core::profiles::copy_name(&dev.name, &taken);
                }
                self.status = trf("Imported the profile {}", &[&dev.name]);
                self.profiles.push(dev);
                self.start_sel = self.profiles.len() - 1;
            }
            Err(e) => self.status = trf("Cannot use the file: {}", &[&e]),
        }
    }

    pub fn open_cfg_new(&mut self) {
        let mut draft = Device::default();
        draft.name = String::new();
        self.cfg = Some(CfgEdit { idx: None, draft, page: CfgPage::General, head: 0 });
    }

    pub fn open_cfg_edit(&mut self, idx: usize) {
        if let Some(d) = self.profiles.get(idx) {
            self.cfg = Some(CfgEdit { idx: Some(idx), draft: d.clone(), page: CfgPage::General, head: 0 });
        }
    }

    pub fn start_screen(&mut self, ui: &mut egui::Ui) {
        // No profile yet: the user must create one before anything else. The first run goes through a short setup.
        if self.profiles.is_empty() && self.cfg.is_none() {
            if self.wizard_skipped {
                self.open_cfg_new();
            } else {
                self.start_wizard();
            }
        }
        let ctx = ui.ctx().clone();
        egui::CentralPanel::default().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space((ui.available_height() * 0.06).max(12.0));
                if let Some(logo) = &self.logo {
                    ui.image((logo.id(), egui::vec2(96.0, 96.0)));
                }
                ui.label(RichText::new("LightCreator").size(28.0).strong());
                ui.label(RichText::new(APP_VERSION).color(theme::text_dim()));
                ui.add_space(18.0);
                ui.label(RichText::new(tr("Select device")).size(16.0).strong());
                ui.label(RichText::new(tr("Choose the device you will work with.")).color(theme::text_dim()));
                ui.add_space(8.0);
                let frame = egui::Frame::group(ui.style()).inner_margin(egui::Margin::same(12));
                frame.show(ui, |ui| {
                    ui.set_width(460.0);
                    ui.with_layout(Layout::top_down(Align::LEFT), |ui| self.device_list(ui));
                });
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    // centre the two combo boxes
                    ui.add_space(((ui.available_width() - 360.0) / 2.0).max(0.0));
                    ui.label(tr("Language"));
                    let mut lang = self.lang;
                    egui::ComboBox::from_id_salt("start_lang").selected_text(lang.name()).show_ui(ui, |ui| {
                        for l in Lang::ALL {
                            ui.selectable_value(&mut lang, l, l.name());
                        }
                    });
                    if lang != self.lang {
                        self.do_act(&ctx, Act::SetLang(lang));
                    }
                    ui.add_space(12.0);
                    ui.label(tr("Colour scheme"));
                    let mut sc = self.scheme;
                    egui::ComboBox::from_id_salt("start_scheme").selected_text(tr(sc.label())).show_ui(ui, |ui| {
                        for s in Scheme::ALL {
                            ui.selectable_value(&mut sc, s, tr(s.label()));
                        }
                    });
                    if sc != self.scheme {
                        self.do_act(&ctx, Act::SetScheme(sc));
                    }
                });
            });
        });
    }

    fn device_list(&mut self, ui: &mut egui::Ui) {
        if self.profiles.is_empty() {
            ui.label(RichText::new(tr("Create at least one device profile to continue.")).color(theme::text_dim()));
            // No profile found: offer to bring the settings back from a backup.
            ui.add_space(4.0);
            ui.label(RichText::new(tr("No profile files were found. Have a backup?")).color(theme::text_dim()));
            let btn = egui::Button::image_and_text(crate::icons::slot(Some("package-open"), Color32::WHITE), RichText::new(tr("Restore from backup…")).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0x2e, 0xa0, 0x4f));
            if ui.add_sized([ui.available_width(), 30.0], btn).clicked() {
                let ctx = ui.ctx().clone();
                self.restore_backup_dialog(&ctx, false);
            }
            ui.add_space(6.0);
        }
        self.start_sel = self.start_sel.min(self.profiles.len().saturating_sub(1));
        let mut go = false;
        egui::ScrollArea::vertical().max_height(220.0).auto_shrink([false, true]).show(ui, |ui| {
            for i in 0..self.profiles.len() {
                let d = &self.profiles[i];
                let due: Vec<&str> = d.maintenance.due().iter().map(|c| c.name.as_str()).collect();
                let mut text = format!("{}    {} × {}", d.name, fmt_len(d.units, d.area.bed_w, 0), fmt_len(d.units, d.area.bed_h, 0));
                if !due.is_empty() {
                    text.push_str(&format!("    ⚠ {}", trf("Maintenance due: {}", &[&due.join(", ")])));
                }
                let r = ui.add_sized([ui.available_width(), 28.0], egui::Button::selectable(self.start_sel == i, text));
                if r.clicked() {
                    self.start_sel = i;
                    self.confirm_delete = false;
                }
                if r.double_clicked() {
                    go = true;
                }
            }
        });
        ui.add_space(8.0);
        let has = !self.profiles.is_empty();
        ui.horizontal_wrapped(|ui| {
            let cont = egui::Button::new(RichText::new(format!("  {}  ", tr("Continue"))).color(Color32::WHITE).strong()).fill(theme::accent());
            if ui.add_enabled(has, cont).clicked() {
                go = true;
            }
            if ui.button(tr("Add device…")).clicked() {
                self.start_wizard();
            }
            if ui.add_enabled(has, egui::Button::new(tr("Edit…"))).clicked() {
                self.open_cfg_edit(self.start_sel);
            }
            if ui.add_enabled(has, egui::Button::new(tr("Duplicate"))).on_hover_text(tr("A copy of this profile")).clicked() {
                self.duplicate_profile(self.start_sel);
            }
            if ui.add_enabled(has, egui::Button::new(tr("Export…"))).on_hover_text(tr("Save this profile to a file")).clicked() {
                self.export_profile_dialog(self.start_sel);
            }
            if ui.button(tr("Import…")).on_hover_text(tr("Read a profile from a file")).clicked() {
                self.import_profile_dialog();
            }
            if self.confirm_delete && has {
                if crate::window_ui::delete_button(ui, &tr("Confirm delete")).clicked() {
                    self.profiles.remove(self.start_sel);
                    self.confirm_delete = false;
                    self.active = 0;
                }
                if ui.button(tr("Cancel")).clicked() {
                    self.confirm_delete = false;
                }
            } else if crate::window_ui::delete_button_enabled(ui, has, &tr("Delete")).clicked() {
                self.confirm_delete = true;
            }
        });
        if go {
            self.enter_editor(self.start_sel);
        }
    }

    /// Device configuration window (create or edit a profile). Closable only when a profile exists.
    pub fn config_window(&mut self, ctx: &egui::Context) {
        // "Device settings…" from the editor edits the active profile.
        if self.show_device {
            self.show_device = false;
            if self.cfg.is_none() {
                self.open_cfg_edit(self.active);
            }
        }
        // The first-run setup has its own window on the same draft.
        if self.wizard.is_some() {
            return;
        }
        let Some(mut cfg) = self.cfg.take() else { return };
        let must_create = self.profiles.is_empty();
        let title = if cfg.idx.is_some() { tr("Device configuration") } else { tr("New device") };
        let mut save = false;
        let mut cancel = false;
        let mut read_clicked = false;
        let reading = self.read_cfg.is_some();
        let mut cfg_open = true;
        let mut fw_acts: Vec<crate::firmware_ui::FwAct> = vec![];
        let mut cfg_head = cfg.head;
        let mut maint_reset: Option<usize> = None;
        let mut test_cam: Option<lc_core::CameraSource> = None;
        crate::window_ui::popup(ctx, "device_cfg", &title, [720.0, 500.0], &mut cfg_open, |ui| {
            if must_create {
                ui.label(RichText::new(tr("Create at least one device profile to continue.")).color(theme::text_dim()));
                ui.add_space(4.0);
            }
            let body_h = (ui.ctx().content_rect().height() - 230.0).clamp(200.0, 440.0);
            let pages = CfgPage::visible(cfg.draft.controller);
            let mut page = if pages.contains(&cfg.page) { cfg.page } else { CfgPage::General };
            let d = &mut cfg.draft;
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(120.0);
                    for p in &pages {
                        if ui.selectable_label(page == *p, tr(p.label())).clicked() {
                            page = *p;
                        }
                    }
                });
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    egui::ScrollArea::vertical().max_height(body_h).auto_shrink([false, false]).show(ui, |ui| {
                        let (u, su) = (d.units, d.speed_unit);
                        match page {
                            CfgPage::General => {
                                egui::Grid::new("dev_general").num_columns(2).min_col_width(150.0).spacing([12.0, 6.0]).show(ui, |ui| {
                                    ui.label(tr("Name"));
                                    ui.add(egui::TextEdit::singleline(&mut d.name).hint_text(tr("e.g. My diode laser")));
                                    ui.end_row();
                                    ui.label(tr("Controller"));
                                    egui::ComboBox::from_id_salt("controller").selected_text(d.controller.label()).show_ui(ui, |ui| {
                                        for c in Controller::ALL {
                                            ui.selectable_value(&mut d.controller, c, c.label());
                                        }
                                    });
                                    ui.end_row();
                                    ui.label(tr("Units"));
                                    egui::ComboBox::from_id_salt("units").selected_text(tr(d.units.label())).show_ui(ui, |ui| {
                                        for u in Units::ALL {
                                            ui.selectable_value(&mut d.units, u, tr(u.label()));
                                        }
                                    });
                                    ui.end_row();
                                    ui.label(tr("Speed units"));
                                    egui::ComboBox::from_id_salt("speed_units").selected_text(tr(d.speed_unit.label())).show_ui(ui, |ui| {
                                        for k in SpeedUnit::ALL {
                                            ui.selectable_value(&mut d.speed_unit, k, tr(k.label()));
                                        }
                                    });
                                    ui.end_row();
                                    ui.label(tr("Travel speed"));
                                    drag_speed(ui, u, su, true, &mut d.speeds.travel_speed, 50.0, Some((100.0, 60000.0)));
                                    ui.end_row();
                                    ui.label(tr("Max cutting speed"));
                                    drag_speed(ui, u, su, true, &mut d.speeds.max_cut_speed, 50.0, Some((0.0, 60000.0))).on_hover_text(tr("0 = no limit. Layers faster than this get a warning before the job; the time estimate uses at most this speed."));
                                    ui.end_row();
                                    ui.label(tr("Acceleration"));
                                    ui.add(egui::DragValue::new(&mut d.speeds.acceleration).range(0.0..=100000.0).suffix(" mm/s²")).on_hover_text(tr("0 = not known. With a value the time estimate includes the start and stop ramps and the layer panel suggests an overscan."));
                                    ui.end_row();
                                    ui.label(tr("Jog step"));
                                    drag_len(ui, u, &mut d.jog_step, 0.1, Some((0.1, 200.0)));
                                    ui.end_row();
                                    ui.label(tr("Jog feed"));
                                    drag_speed(ui, u, su, true, &mut d.jog_feed, 10.0, Some((10.0, 20000.0)));
                                    ui.end_row();
                                });
                            }
                            CfgPage::Hardware => {
                                egui::Grid::new("dev_hardware").num_columns(2).min_col_width(150.0).spacing([12.0, 6.0]).show(ui, |ui| {
                                    ui.label(tr("Work area X"));
                                    drag_len(ui, u, &mut d.area.bed_w, 1.0, Some((10.0, 5000.0)));
                                    ui.end_row();
                                    ui.label(tr("Work area Y"));
                                    drag_len(ui, u, &mut d.area.bed_h, 1.0, Some((10.0, 5000.0)));
                                    ui.end_row();
                                    ui.label(tr("Margins"));
                                    ui.vertical(|ui| {
                                        for (name, v) in [("Left", &mut d.area.margins.left), ("Top", &mut d.area.margins.top), ("Right", &mut d.area.margins.right), ("Bottom", &mut d.area.margins.bottom)] {
                                            ui.horizontal(|ui| {
                                                ui.label(tr(name));
                                                drag_len(ui, u, v, 0.5, Some((0.0, 500.0)));
                                            });
                                        }
                                    })
                                    .response
                                    .on_hover_text(tr("Strips along the edges of the work area that the machine cannot use (clamps, frame); nothing may be placed or moved there"));
                                    ui.end_row();
                                    ui.label(tr("Software limits"));
                                    ui.vertical(|ui| {
                                        let mut on = d.area.soft_limits.is_some();
                                        if ui.checkbox(&mut on, tr("Own limits")).changed() {
                                            d.area.soft_limits = if on { Some(lc_core::SoftLimits { x0: 0.0, y0: 0.0, x1: d.work_size().0, y1: d.work_size().1 }) } else { None };
                                        }
                                        if let Some(l) = &mut d.area.soft_limits {
                                            for (name, v) in [("Left", &mut l.x0), ("Top", &mut l.y0), ("Right", &mut l.x1), ("Bottom", &mut l.y1)] {
                                                ui.horizontal(|ui| {
                                                    ui.label(tr(name));
                                                    drag_len(ui, u, v, 0.5, Some((0.0, 5000.0)));
                                                });
                                            }
                                        }
                                    });
                                    ui.end_row();
                                    ui.label(tr("Z axis"));
                                    ui.vertical(|ui| {
                                        ui.checkbox(&mut d.axes.has_z, tr("The machine has a Z axis"));
                                        if d.axes.has_z {
                                            ui.horizontal(|ui| {
                                                ui.label(tr("Travel"));
                                                ui.add(egui::DragValue::new(&mut d.axes.z_min).speed(0.5).suffix(" mm"));
                                                ui.label("–");
                                                ui.add(egui::DragValue::new(&mut d.axes.z_max).speed(0.5).suffix(" mm"));
                                            });
                                            ui.checkbox(&mut d.axes.flip_z, tr("Up is the negative Z direction"));
                                        }
                                    });
                                    ui.end_row();
                                    ui.label(tr("Machine zero (0,0)"));
                                    egui::ComboBox::from_id_salt("origin").selected_text(tr(d.axes.origin.label())).show_ui(ui, |ui| {
                                        for o in Origin::ALL {
                                            ui.selectable_value(&mut d.axes.origin, o, tr(o.label()));
                                        }
                                    });
                                    ui.end_row();
                                    ui.label(tr("Invert axes"));
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut d.axes.flip_x, "X");
                                        ui.checkbox(&mut d.axes.flip_y, "Y");
                                    });
                                    ui.end_row();
                                    ui.label(tr("Panel orientation"));
                                    egui::ComboBox::from_id_salt("orientation").selected_text(tr(d.axes.orientation.label())).show_ui(ui, |ui| {
                                        for o in Orientation::ALL {
                                            ui.selectable_value(&mut d.axes.orientation, o, tr(o.label()));
                                        }
                                    });
                                    ui.end_row();
                                });
                            }
                            CfgPage::Laser => {
                                let mut head = cfg_head.min(d.heads.len() - 1);
                                ui.horizontal_wrapped(|ui| {
                                    for (i, h) in d.heads.iter().enumerate() {
                                        let name = if h.name.trim().is_empty() { trf("Head {}", &[&(i + 1)]) } else { h.name.clone() };
                                        let (r, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                                        ui.painter().rect_filled(r, 2.0, Color32::from_rgb(h.colour[0], h.colour[1], h.colour[2]));
                                        if ui.selectable_label(head == i, name).clicked() {
                                            head = i;
                                        }
                                    }
                                    if ui.small_button("+").on_hover_text(tr("Add a laser head")).clicked() {
                                        let tool = d.heads.iter().map(|h| h.tool).max().unwrap_or(0) + 1;
                                        let mut h = d.heads[head].clone();
                                        h.name = String::new();
                                        h.tool = tool;
                                        d.heads.push(h);
                                        head = d.heads.len() - 1;
                                    }
                                    if ui.add_enabled_ui(d.heads.len() > 1, |ui| crate::window_ui::delete_icon(ui, &tr("Remove this laser head"))).inner.clicked() {
                                        d.heads.remove(head);
                                        head = head.saturating_sub(1).min(d.heads.len() - 1);
                                    }
                                });
                                ui.add_space(4.0);
                                let multi = d.heads.len() > 1;
                                let h = &mut d.heads[head];
                                egui::Grid::new("dev_laser").num_columns(2).min_col_width(150.0).spacing([12.0, 6.0]).show(ui, |ui| {
                                    ui.label(tr("Name"));
                                    ui.add(egui::TextEdit::singleline(&mut h.name).hint_text(tr("e.g. Blue diode")));
                                    ui.end_row();
                                    if multi {
                                        ui.label(tr("Tool number (T)"));
                                        ui.add(egui::DragValue::new(&mut h.tool).range(0..=99)).on_hover_text(tr("The number the controller knows this head by; it is written as T<n> when a layer uses another head. It must match the mapping in the controller."));
                                        ui.end_row();
                                    }
                                    ui.label(tr("Colour"));
                                    ui.color_edit_button_srgb(&mut h.colour);
                                    ui.end_row();
                                    ui.label(tr("Laser type"));
                                    egui::ComboBox::from_id_salt("laser_kind").selected_text(h.laser.label()).show_ui(ui, |ui| {
                                        for k in LaserKind::ALL {
                                            ui.selectable_value(&mut h.laser, k, k.label());
                                        }
                                    });
                                    ui.end_row();
                                    ui.label(tr("Wavelength"));
                                    ui.add(egui::DragValue::new(&mut h.wavelength_nm).range(0..=20000).suffix(" nm")).on_hover_text(trf("0 = the usual one ({} nm)", &[&h.laser.default_wavelength()]));
                                    ui.end_row();
                                    ui.label(tr("Optical power"));
                                    ui.add(egui::DragValue::new(&mut h.optical_power_w).range(0.0..=1000.0).suffix(" W")).on_hover_text(trf("0 = a typical one ({} W)", &[&h.laser.default_power()]));
                                    ui.end_row();
                                    ui.label(tr("S-value max ($30)"));
                                    ui.add(egui::DragValue::new(&mut h.s_max).range(1.0..=100000.0));
                                    ui.end_row();
                                    ui.label(tr("Frame power"));
                                    ui.add(egui::DragValue::new(&mut h.frame_power).range(0.0..=10.0).suffix(" %")).on_hover_text(tr("0 % keeps the laser off while framing; 1–2 % shows a dim dot on diode lasers"));
                                    ui.end_row();
                                    ui.label(tr("Frame speed"));
                                    drag_speed(ui, u, su, false, &mut h.frame_speed, 0.5, Some((1.0, 1000.0)));
                                    ui.end_row();
                                    ui.label(tr("Focus power"));
                                    ui.add(egui::DragValue::new(&mut h.focus_power).range(0.0..=10.0).suffix(" %")).on_hover_text(tr("A weak steady beam to set the height by (focus mode). 0 = no focus mode."));
                                    ui.end_row();
                                    ui.label(tr("Beam spot"));
                                    ui.horizontal(|ui| {
                                        drag_len(ui, u, &mut h.spot_x, 0.005, Some((0.0, 5.0)));
                                        ui.label("×");
                                        drag_len(ui, u, &mut h.spot_y, 0.005, Some((0.0, 5.0)));
                                    })
                                    .response
                                    .on_hover_text(tr("Size of the beam on the material (0 = not known). A fill interval below it makes neighbouring lines overlap."));
                                    ui.end_row();
                                    ui.label(tr("Pointer offset"));
                                    ui.horizontal(|ui| {
                                        drag_len(ui, u, &mut h.pointer_dx, 0.1, Some((-200.0, 200.0)));
                                        ui.label("×");
                                        drag_len(ui, u, &mut h.pointer_dy, 0.1, Some((-200.0, 200.0)));
                                    })
                                    .response
                                    .on_hover_text(tr("Where the red pointer dot is relative to the beam (x right, y down). Used by the pointer alignment in the Device tab."));
                                    ui.end_row();
                                    if h.laser.has_pwm() {
                                        ui.label(tr("PWM frequency"));
                                        ui.add(egui::DragValue::new(&mut h.pwm_freq_hz).range(0.0..=1_000_000.0).suffix(" Hz"));
                                        ui.end_row();
                                        ui.label(tr("PWM maximum frequency"));
                                        ui.add(egui::DragValue::new(&mut h.pwm_max_freq_hz).range(0.0..=1_000_000.0).suffix(" Hz"));
                                        ui.end_row();
                                        ui.label(tr("Pulse width"));
                                        ui.add(egui::DragValue::new(&mut h.pulse_width_us).range(0.0..=100_000.0).suffix(" µs"));
                                        ui.end_row();
                                        ui.label(tr("Pulse width range"));
                                        ui.horizontal(|ui| {
                                            ui.add(egui::DragValue::new(&mut h.pulse_min_us).range(0.0..=100_000.0).suffix(" µs"));
                                            ui.label("–");
                                            ui.add(egui::DragValue::new(&mut h.pulse_max_us).range(0.0..=100_000.0).suffix(" µs"));
                                        });
                                        ui.end_row();
                                    }
                                });
                                if h.laser.has_pwm() {
                                    ui.label(RichText::new(tr("The PWM values are kept with the profile and offered as variables for macros; they are not sent to the controller.")).color(theme::text_dim()));
                                }
                                cfg_head = head;
                            }
                            CfgPage::Gcode => {
                                egui::Grid::new("dev_gcode").num_columns(2).min_col_width(150.0).spacing([12.0, 6.0]).show(ui, |ui| {
                                    ui.label(tr("G-code dialect"));
                                    let shown = d.gcode.dialect.map(|x| x.label()).unwrap_or("");
                                    egui::ComboBox::from_id_salt("dialect").selected_text(if d.gcode.dialect.is_none() { tr("Automatic (from the controller)") } else { shown.into() }).show_ui(ui, |ui| {
                                        ui.selectable_value(&mut d.gcode.dialect, None, tr("Automatic (from the controller)"));
                                        for x in lc_core::Dialect::ALL {
                                            ui.selectable_value(&mut d.gcode.dialect, Some(x), x.label());
                                        }
                                    });
                                    ui.end_row();
                                    ui.label(tr("Dynamic power (M4)"));
                                    ui.add_enabled(d.gcode.dialect.is_none(), egui::Checkbox::without_text(&mut d.gcode.dynamic_power)).on_hover_text(tr("Used when the dialect is automatic; a chosen GRBL dialect decides between M3 and M4 itself."));
                                    ui.end_row();
                                    ui.label(tr("Return to origin"));
                                    ui.checkbox(&mut d.gcode.return_home, "");
                                    ui.end_row();
                                    ui.label(tr("Arcs (G2 / G3)"));
                                    ui.checkbox(&mut d.gcode.arcs, tr("Write curves as arcs where they fit"));
                                    ui.end_row();
                                    ui.label(tr("Arc tolerance"));
                                    drag_len(ui, u, &mut d.gcode.arc_tolerance, 0.001, Some((0.001, 1.0)));
                                    ui.end_row();
                                    ui.label(tr("Curve tolerance"));
                                    drag_len(ui, u, &mut d.gcode.flatten_tolerance, 0.005, Some((0.005, 1.0))).on_hover_text(tr("How far a flattened curve (Bézier, text) may stray from the true curve."));
                                    ui.end_row();
                                });
                                if d.gcode.arcs {
                                    ui.add_space(6.0);
                                    ui.label(RichText::new(tr("Arcs: GRBL usually handles G2 / G3, Marlin only when built with ARC_SUPPORT, and Smoothieware has its own behaviour. Some controllers ignore arcs or run them wrongly: test with the laser off. Arcs are used for outlines and rings, not for scan lines or images.")).color(Color32::from_rgb(0xe8, 0x8a, 0x1f)));
                                }
                                if d.dialect() == lc_core::Dialect::Smoothieware {
                                    ui.add_space(6.0);
                                    ui.label(RichText::new(tr("Smoothieware: S is written as a fraction of the full power (set S-value max to 1). Streaming works like GRBL's; none of this has been tried on real hardware.")).color(theme::text_dim()));
                                }
                            }
                            CfgPage::Connection => {
                                egui::Grid::new("dev_link").num_columns(2).min_col_width(150.0).spacing([12.0, 6.0]).show(ui, |ui| {
                                    ui.label(tr("Connection type"));
                                    egui::ComboBox::from_id_salt("link_kind").width(230.0).selected_text(tr(d.connection.link.label())).show_ui(ui, |ui| {
                                        for k in LinkKind::ALL {
                                            ui.selectable_value(&mut d.connection.link, k, tr(k.label()));
                                        }
                                    });
                                    ui.end_row();
                                    if d.connection.link == LinkKind::Serial {
                                        ui.label(tr("Port"));
                                        ui.horizontal_wrapped(|ui| {
                                            egui::ComboBox::from_id_salt("cfg_port").width(220.0).selected_text(if d.connection.port.is_empty() { tr("No port") } else { d.connection.port.as_str() }).show_ui(ui, |ui| {
                                                for p in &self.port_info {
                                                    ui.selectable_value(&mut d.connection.port, p.name.clone(), p.label());
                                                }
                                            });
                                            if ui.button(tr("Refresh")).clicked() {
                                                self.refresh_ports();
                                            }
                                        });
                                        ui.end_row();
                                        ui.label(tr("Port or USB id"));
                                        ui.add(egui::TextEdit::singleline(&mut d.connection.port).hint_text("/dev/ttyUSB0  or  0403:6001").desired_width(220.0)).on_hover_text(tr("A port name, or the USB vendor and product id (VID:PID, hexadecimal): the port that has these ids is used."));
                                        ui.end_row();
                                        ui.label(tr("Bound USB device"));
                                        ui.horizontal(|ui| {
                                            let ids = self.port_info.iter().find(|p| p.name == d.connection.port).and_then(|p| p.usb);
                                            if d.connection.usb_id.is_empty() {
                                                ui.label(RichText::new(tr("none")).color(theme::text_dim()));
                                            } else {
                                                ui.label(RichText::new(&d.connection.usb_id).monospace());
                                            }
                                            if ui.add_enabled(ids.is_some(), egui::Button::new(tr("Bind to this port"))).on_hover_text(tr("Remember the USB ids of the chosen port: the port is found again by them when its name changes.")).clicked() {
                                                if let Some((v, p)) = ids {
                                                    d.connection.usb_id = format!("{v:04x}:{p:04x}");
                                                }
                                            }
                                            if ui.add_enabled(!d.connection.usb_id.is_empty(), egui::Button::new(tr("Unbind"))).clicked() {
                                                d.connection.usb_id.clear();
                                            }
                                        });
                                        ui.end_row();
                                        ui.label(tr("Baud rate"));
                                        ui.add(egui::DragValue::new(&mut d.connection.baud).range(1200..=1_000_000));
                                        ui.end_row();
                                    } else {
                                        ui.label(tr("Host"));
                                        ui.add(egui::TextEdit::singleline(&mut d.connection.host).hint_text("raspberrypi.local").desired_width(200.0));
                                        ui.end_row();
                                        ui.label(tr("TCP port"));
                                        ui.add(egui::DragValue::new(&mut d.connection.tcp_port).range(1..=65535));
                                        ui.end_row();
                                    }
                                    ui.label(tr("Reconnect"));
                                    ui.checkbox(&mut d.connection.auto_reconnect, tr("Connect again automatically")).on_hover_text(tr("When the link drops by itself, try again every few seconds until you press Disconnect."));
                                    ui.end_row();
                                    ui.label(tr("After connecting"));
                                    ui.vertical(|ui| {
                                        ui.checkbox(&mut d.connection.home_on_connect, tr("Home the machine"));
                                        ui.checkbox(&mut d.connection.unlock_on_connect, tr("Clear an alarm"));
                                        if d.connection.unlock_on_connect {
                                            ui.label(RichText::new(tr("Clearing the alarm skips homing: use it only when the head is known to stand in a safe place.")).color(Color32::from_rgb(0xe8, 0x8a, 0x1f)));
                                        }
                                    });
                                    ui.end_row();
                                    ui.label(tr("Streaming"));
                                    egui::ComboBox::from_id_salt("streaming").selected_text(tr(d.connection.streaming.label())).show_ui(ui, |ui| {
                                        for m in lc_core::Streaming::ALL {
                                            ui.selectable_value(&mut d.connection.streaming, m, tr(m.label()));
                                        }
                                    })
                                    .response
                                    .on_hover_text(tr("Counting fills the controller's buffer and is fast. Simple sends one line and waits for its ok: slow, but for firmware that answers wrongly."));
                                    ui.end_row();
                                    ui.label("");
                                    let label = if reading { tr("Reading…") } else { tr("Read from device") };
                                    if ui.add_enabled(!reading && d.has_target(), egui::Button::new(label)).on_hover_text(tr("Connect and read work area, S-value max and speed from the controller")).clicked() {
                                        read_clicked = true;
                                    }
                                    ui.end_row();
                                });
                            }
                            CfgPage::Camera => {
                                ui.label(RichText::new(tr("Cameras that watch the machine. The active one gives the live picture. A camera can only be switched on when its address is valid.")).color(theme::text_dim()));
                                let mut remove = None;
                                let n = d.cameras.len();
                                for i in 0..n {
                                    let active = d.active_camera == i;
                                    let c = &mut d.cameras[i];
                                    egui::Frame::group(ui.style()).show(ui, |ui| {
                                        ui.horizontal_wrapped(|ui| {
                                            let ok = c.is_valid();
                                            if ui.add_enabled(ok || c.enabled, egui::Checkbox::new(&mut c.enabled, "")).on_hover_text(if ok { tr("Switch this camera on or off") } else { tr("The address is not valid: an http or https address is needed") }).changed() && !ok {
                                                c.enabled = false;
                                            }
                                            ui.add(egui::TextEdit::singleline(&mut c.name).hint_text(tr("Camera name")).desired_width(150.0));
                                            if ui.radio(active, tr("Active")).clicked() {
                                                d.active_camera = i;
                                            }
                                            if crate::window_ui::delete_icon(ui, &tr("Delete this camera")).clicked() {
                                                remove = Some(i);
                                            }
                                        });
                                        ui.horizontal_wrapped(|ui| {
                                            let local = matches!(c.kind, lc_core::CameraKind::Local(_));
                                            let mut want_local = local;
                                            ui.radio_value(&mut want_local, false, tr("Address"));
                                            ui.radio_value(&mut want_local, true, tr("Camera of this computer"));
                                            if want_local != local {
                                                c.kind = if want_local { lc_core::CameraKind::Local(0) } else { lc_core::CameraKind::Url(String::new()) };
                                                if !want_local {
                                                    c.enabled = false;
                                                }
                                            }
                                        });
                                        match &mut c.kind {
                                            lc_core::CameraKind::Url(u) => {
                                                ui.add(egui::TextEdit::singleline(u).hint_text("http://raspberrypi.local:8080/stream.mjpg").desired_width(300.0)).on_hover_text(tr("Live picture of the machine: an MJPEG stream or a JPEG snapshot address (http or https). RTSP is not supported."));
                                                if !lc_core::camimg::valid_url(u) && !u.trim().is_empty() {
                                                    ui.label(RichText::new(tr("Not an http or https address.")).color(Color32::from_rgb(0xc0, 0x30, 0x30)));
                                                }
                                            }
                                            lc_core::CameraKind::Local(i) => {
                                                if crate::camera::SUPPORTED {
                                                    ui.horizontal(|ui| {
                                                        ui.label(tr("Camera number"));
                                                        ui.add(egui::DragValue::new(i).range(0..=15));
                                                    });
                                                } else {
                                                    ui.label(RichText::new(tr("This build has no camera support. Build with: cargo run --release --features camera")).color(theme::text_dim()));
                                                }
                                            }
                                        }
                                        let testable = match &c.kind {
                                            lc_core::CameraKind::Url(u) => lc_core::camimg::valid_url(u),
                                            lc_core::CameraKind::Local(_) => crate::camera::SUPPORTED,
                                        };
                                        if ui.add_enabled(testable, egui::Button::new(tr("Test"))).on_hover_text(tr("Opens the picture of this camera as set here, without saving")).clicked() {
                                            test_cam = Some(c.clone());
                                        }
                                        ui.horizontal(|ui| {
                                            ui.label(tr("Camera rotation"));
                                            for q in 0..4u8 {
                                                ui.selectable_value(&mut c.rotation, q, format!("{}°", q as u32 * 90));
                                            }
                                        });
                                        egui::CollapsingHeader::new(tr("Picture")).id_salt(("cam_img", i)).show(ui, |ui| {
                                            egui::Grid::new(("cam_img_grid", i)).num_columns(2).spacing([12.0, 4.0]).show(ui, |ui| {
                                                ui.label(tr("Brightness"));
                                                ui.add(egui::Slider::new(&mut c.image.brightness, -100..=100));
                                                ui.end_row();
                                                ui.label(tr("Contrast"));
                                                ui.add(egui::Slider::new(&mut c.image.contrast, 0..=100));
                                                ui.end_row();
                                                ui.label(tr("White balance"));
                                                ui.horizontal(|ui| {
                                                    let mut auto = c.image.white_balance.is_none();
                                                    if ui.checkbox(&mut auto, tr("Auto")).changed() {
                                                        c.image.white_balance = if auto { None } else { Some(6500) };
                                                    }
                                                    if let Some(k) = &mut c.image.white_balance {
                                                        ui.add(egui::DragValue::new(k).range(2500..=10_000).speed(50.0).suffix(" K"));
                                                    }
                                                });
                                                ui.end_row();
                                                ui.label(tr("Noise reduction"));
                                                ui.add(egui::Slider::new(&mut c.image.denoise, 0.0..=0.95));
                                                ui.end_row();
                                                ui.label(tr("Format"));
                                                ui.checkbox(&mut c.image.prefer_yuyv, tr("Prefer YUYV")).on_hover_text(tr("For a camera of this computer: ask for the uncompressed YUYV format where the system allows it."));
                                                ui.end_row();
                                            });
                                        });
                                        egui::CollapsingHeader::new(tr("Lens correction")).id_salt(("cam_lens", i)).show(ui, |ui| {
                                            ui.label(RichText::new(tr("Coefficients of the lens distortion (radial k1–k3, tangential p1, p2) for coordinates measured from the centre in units of the longer side of the picture. All 0 = no correction. Entered by hand: there is no automatic calibration.")).color(theme::text_dim()));
                                            ui.horizontal_wrapped(|ui| {
                                                for (name, v) in [("k1", &mut c.lens.k1), ("k2", &mut c.lens.k2), ("k3", &mut c.lens.k3), ("p1", &mut c.lens.p1), ("p2", &mut c.lens.p2)] {
                                                    ui.label(name);
                                                    ui.add(egui::DragValue::new(v).speed(0.001).range(-5.0..=5.0).max_decimals(4));
                                                }
                                                if ui.small_button(tr("Reset")).clicked() {
                                                    c.lens = lc_core::Lens::default();
                                                }
                                            });
                                        });
                                    });
                                }
                                if let Some(i) = remove {
                                    d.cameras.remove(i);
                                    d.active_camera = d.active_camera.min(d.cameras.len().saturating_sub(1));
                                }
                                if d.cameras.len() < 8 && ui.button(tr("Add a camera")).clicked() {
                                    add_camera(d);
                                }
                            }
                            CfgPage::Rotary => {
                                let grbl = d.dialect().is_grbl();
                                ui.checkbox(&mut d.rotary.enabled, tr("A rotary attachment is connected"));
                                ui.add_enabled_ui(d.rotary.enabled, |ui| {
                                    egui::Grid::new("dev_rotary").num_columns(2).min_col_width(150.0).spacing([12.0, 6.0]).show(ui, |ui| {
                                        ui.label(tr("Rotation in the G-code"));
                                        ui.vertical(|ui| {
                                            ui.radio_value(&mut d.rotary.mode, lc_core::RotaryMode::ReplaceY, tr("Replaces Y (the Y motor turns the work)"));
                                            ui.add_enabled_ui(!grbl, |ui| {
                                                ui.radio_value(&mut d.rotary.mode, lc_core::RotaryMode::TrueA, tr("True A axis (degrees)"));
                                            });
                                            if grbl {
                                                ui.label(RichText::new(tr("GRBL 1.1 has three axes: the A axis is not available, Y is used.")).color(Color32::from_rgb(0xe8, 0x8a, 0x1f)));
                                            }
                                        });
                                        ui.end_row();
                                        ui.label(tr("Kind"));
                                        ui.horizontal(|ui| {
                                            ui.radio_value(&mut d.rotary.kind, lc_core::RotaryKind::Chuck, tr("Chuck"));
                                            ui.radio_value(&mut d.rotary.kind, lc_core::RotaryKind::Roller, tr("Rollers"));
                                        });
                                        ui.end_row();
                                        ui.label(tr("Workpiece diameter"));
                                        drag_len(ui, u, &mut d.rotary.diameter, 0.5, Some((1.0, 1000.0)));
                                        ui.end_row();
                                        if d.rotary.kind == lc_core::RotaryKind::Roller {
                                            ui.label(tr("Roller diameter"));
                                            drag_len(ui, u, &mut d.rotary.roller_diameter, 0.5, Some((1.0, 1000.0)));
                                            ui.end_row();
                                        }
                                        ui.label(tr("Microsteps per turn"));
                                        ui.add(egui::DragValue::new(&mut d.rotary.microsteps_per_rev).range(1.0..=1_000_000.0)).on_hover_text(tr("Microsteps of the motor for one full turn of what it drives (the chuck, or one roller)."));
                                        ui.end_row();
                                        ui.label(tr("Circumference"));
                                        ui.label(fmt_len(u, d.rotary.circumference(), 2)).on_hover_text(tr("The way round the workpiece: the design may not be higher."));
                                        ui.end_row();
                                    });
                                    if d.rotary.mode == lc_core::RotaryMode::TrueA && !grbl {
                                        ui.label(trf("A turns {} degrees for each millimetre on the surface.", &[&format!("{:.4}", d.rotary.degrees_per_mm())]));
                                    } else {
                                        let steps = d.rotary.steps_per_mm();
                                        ui.label(trf("The controller needs {} steps per mm on the Y axis ($101).", &[&format!("{:.3}", steps)]));
                                        if d.controller == lc_core::Controller::Grbl && self.connected && steps > 0.0 && ui.button(tr("Write $101 to the controller…")).on_hover_text(tr("Asks for a confirmation first. Only the Y axis of the controller changes.")).clicked() {
                                            fw_acts.push(crate::firmware_ui::FwAct::Write(101, format!("{:.3}", steps)));
                                        }
                                    }
                                });
                                ui.add_space(4.0);
                                ui.label(RichText::new(tr("Layers marked as rotary are burnt on the attachment. The model is simple: a chuck turns by the workpiece diameter, rollers move the surface like their own; a real machine needs testing.")).color(theme::text_dim()));
                            }
                            CfgPage::Notes => {
                                if !d.template_notes.trim().is_empty() {
                                    ui.label(RichText::new(tr("From the template")).strong());
                                    egui::Frame::group(ui.style()).show(ui, |ui| crate::markdown_ui::show(ui, &d.template_notes));
                                    ui.add_space(6.0);
                                }
                                ui.label(RichText::new(tr("Your notes (Markdown)")).strong());
                                ui.add(egui::TextEdit::multiline(&mut d.notes).font(egui::TextStyle::Monospace).desired_rows(8).desired_width(f32::INFINITY).hint_text(tr("e.g. Settings that worked: air assist 20 %, focus 12 mm")));
                                if d.notes.chars().count() > lc_core::profiles::MAX_NOTES {
                                    ui.label(RichText::new(trf("The notes are longer than {} characters and cannot be saved.", &[&lc_core::profiles::MAX_NOTES])).color(Color32::from_rgb(0xc0, 0x30, 0x30)));
                                }
                                ui.add_space(4.0);
                                ui.label(RichText::new(tr("Preview")).strong());
                                egui::Frame::group(ui.style()).show(ui, |ui| crate::markdown_ui::show(ui, &d.notes));
                            }
                            CfgPage::Maintenance => {
                                ui.label(RichText::new(trf("Total working time: {}", &[&crate::app::fmt_time(d.maintenance.total_secs)])).strong());
                                ui.label(RichText::new(tr("Counted from the time jobs really run (from start to the end or Stop), not from estimates. The total cannot be reset.")).color(theme::text_dim()));
                                ui.add_space(4.0);
                                let mut remove = None;
                                let mut reset = None;
                                for (i, c) in d.maintenance.counters.iter_mut().enumerate() {
                                    ui.horizontal_wrapped(|ui| {
                                        ui.add(egui::TextEdit::singleline(&mut c.name).hint_text(tr("Counter name (a tube, a lens, a belt)")).desired_width(180.0));
                                        let mut limited = c.threshold_hours.is_some();
                                        if ui.checkbox(&mut limited, tr("Remind after")).changed() {
                                            c.threshold_hours = if limited { Some(100.0) } else { None };
                                        }
                                        if let Some(h) = &mut c.threshold_hours {
                                            ui.add(egui::DragValue::new(h).range(0.1..=100_000.0).speed(1.0).suffix(" h"));
                                        }
                                        let text = RichText::new(crate::app::fmt_time(c.secs));
                                        ui.label(if c.is_due() { text.color(Color32::from_rgb(0xe8, 0x8a, 0x1f)).strong() } else { text });
                                        if ui.button(tr("Reset")).clicked() {
                                            reset = Some(i);
                                        }
                                        if crate::window_ui::delete_icon(ui, &tr("Delete this counter")).clicked() {
                                            remove = Some(i);
                                        }
                                    });
                                }
                                if let Some(i) = remove {
                                    d.maintenance.counters.remove(i);
                                }
                                if let Some(i) = reset {
                                    maint_reset = Some(i);
                                }
                                if ui.button(tr("Add a counter")).clicked() {
                                    d.maintenance.counters.push(lc_core::Counter { name: trf("Counter {}", &[&(d.maintenance.counters.len() + 1)]), secs: 0.0, threshold_hours: None });
                                }
                            }
                            CfgPage::Zones => {
                                ui.label(RichText::new(tr("Parts of the work area the machine must never enter (a clamp, a fixture). A move that touches an enabled zone, also one with the laser off, stops Start, Frame and export. The program does not route around zones.")).color(theme::text_dim()));
                                ui.add_space(4.0);
                                let mut remove = None;
                                let has_z = d.axes.has_z;
                                for (i, z) in d.zones.iter_mut().enumerate() {
                                    ui.horizontal_wrapped(|ui| {
                                        ui.checkbox(&mut z.enabled, "");
                                        ui.add(egui::TextEdit::singleline(&mut z.name).hint_text(tr("Zone name")).desired_width(150.0));
                                        let rect = matches!(z.shape, lc_core::ZoneShape::Rect { .. });
                                        let mut want_rect = rect;
                                        egui::ComboBox::from_id_salt(("zone_shape", i)).selected_text(tr(if rect { "Rectangle" } else { "Circle" })).show_ui(ui, |ui| {
                                            ui.selectable_value(&mut want_rect, true, tr("Rectangle"));
                                            ui.selectable_value(&mut want_rect, false, tr("Circle"));
                                        });
                                        if want_rect != rect {
                                            let b = z.shape.bounds();
                                            z.shape = if want_rect { lc_core::ZoneShape::Rect { x: b.min.x, y: b.min.y, w: b.width(), h: b.height() } } else { lc_core::ZoneShape::Circle { x: b.center().x, y: b.center().y, r: (b.width().max(b.height())) / 2.0 } };
                                        }
                                        if crate::window_ui::delete_icon(ui, &tr("Delete this zone")).clicked() {
                                            remove = Some(i);
                                        }
                                    });
                                    if has_z {
                                        ui.horizontal_wrapped(|ui| {
                                            ui.add_space(24.0);
                                            let mut limited = z.z_range.is_some();
                                            if ui.checkbox(&mut limited, tr("Only at these heights (Z)")).on_hover_text(tr("Like a box or a cylinder standing on the table: moves above or below the range are free.")).changed() {
                                                z.z_range = if limited { Some((0.0, 10.0)) } else { None };
                                            }
                                            if let Some((lo, hi)) = &mut z.z_range {
                                                ui.add(egui::DragValue::new(lo).speed(0.5).suffix(" mm"));
                                                ui.label("–");
                                                ui.add(egui::DragValue::new(hi).speed(0.5).suffix(" mm"));
                                            }
                                        });
                                    }
                                    ui.horizontal_wrapped(|ui| {
                                        ui.add_space(24.0);
                                        match &mut z.shape {
                                            lc_core::ZoneShape::Rect { x, y, w, h } => {
                                                for (label, v) in [("X", x), ("Y", y), ("W", w), ("H", h)] {
                                                    ui.label(label);
                                                    drag_len(ui, u, v, 0.5, Some((-5000.0, 5000.0)));
                                                }
                                            }
                                            lc_core::ZoneShape::Circle { x, y, r } => {
                                                for (label, v) in [("X", x), ("Y", y), ("R", r)] {
                                                    ui.label(label);
                                                    drag_len(ui, u, v, 0.5, Some((-5000.0, 5000.0)));
                                                }
                                            }
                                        }
                                    });
                                }
                                if let Some(i) = remove {
                                    d.zones.remove(i);
                                }
                                if ui.button(tr("Add a zone")).clicked() {
                                    d.zones.push(lc_core::Zone { name: trf("Zone {}", &[&(d.zones.len() + 1)]), enabled: true, shape: lc_core::ZoneShape::Rect { x: 10.0, y: 10.0, w: 50.0, h: 30.0 }, z_range: None });
                                }
                            }
                            CfgPage::Firmware => {
                                if self.connected {
                                    ui.label(RichText::new(trf("Connected to: {}", &[&self.doc.device.target_label()])).small().color(theme::text_dim()));
                                }
                                crate::firmware_ui::page(ui, &mut self.fw, self.connected, d.controller, &mut fw_acts);
                            }
                            CfgPage::Advanced => {
                                egui::Frame::new().fill(Color32::from_rgba_unmultiplied(0xc0, 0x30, 0x30, 40)).stroke(egui::Stroke::new(1.0, Color32::from_rgb(0xc0, 0x30, 0x30))).inner_margin(8.0).corner_radius(4.0).show(ui, |ui| {
                                    ui.label(RichText::new(tr("Wrong G-code can fire the laser, drive the head into the end stops or ruin the material. Test with the laser off or at the lowest power.")).strong());
                                });
                                ui.add_space(6.0);
                                let mono = |ui: &mut egui::Ui, text: &mut String, hint: &str, rows: usize| {
                                    ui.add(egui::TextEdit::multiline(text).font(egui::TextStyle::Monospace).hint_text(hint).desired_rows(rows).desired_width(f32::INFINITY));
                                };
                                ui.label(RichText::new(tr("Start of the job")).strong());
                                mono(ui, &mut d.gcode.preamble, "G21 / G90 / G17 / M4 S0", 4);
                                ui.label(RichText::new(tr("Replaces the units, absolute mode, plane and laser mode lines. Empty = built in.")).color(theme::text_dim()));
                                ui.add_space(4.0);
                                ui.label(RichText::new(tr("End of the job")).strong());
                                mono(ui, &mut d.gcode.postscript, "M2", 3);
                                ui.label(RichText::new(tr("Replaces the closing M2. The laser is always switched off and the head sent home (when set) before it. Empty = built in.")).color(theme::text_dim()));
                                ui.add_space(4.0);
                                ui.label(RichText::new(tr("Air assist commands")).strong());
                                egui::Grid::new("dev_air").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                                    ui.label(tr("On"));
                                    ui.add(egui::TextEdit::singleline(&mut d.gcode.air_on).font(egui::TextStyle::Monospace).hint_text("M8"));
                                    ui.end_row();
                                    ui.label(tr("Off"));
                                    ui.add(egui::TextEdit::singleline(&mut d.gcode.air_off).font(egui::TextStyle::Monospace).hint_text("M9"));
                                    ui.end_row();
                                });
                                ui.add_space(8.0);
                                ui.label(RichText::new(tr("Hooks")).strong());
                                ui.label(RichText::new(tr("Own G-code written into the job: when a layer starts, before and after each object, when a layer ends. With object hooks a layer is burnt object by object (and once per pass).")).color(theme::text_dim()));
                                for (i, title) in ["Layer start", "Layer end", "Object start", "Object end"].into_iter().enumerate() {
                                    let h = d.gcode.hooks.get_mut(i);
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut h.enabled, "");
                                        ui.label(RichText::new(tr(title)).strong());
                                    });
                                    ui.add_enabled(h.enabled, egui::TextEdit::multiline(&mut h.code).font(egui::TextStyle::Monospace).desired_rows(2).desired_width(f32::INFINITY));
                                }
                                ui.label(RichText::new(trf("Variables: {}", &[&lc_core::hooks::JOB_VARS.iter().map(|v| format!("{{{v}}}")).collect::<Vec<_>>().join(" ")])).small().color(theme::text_dim()));
                                ui.add_space(8.0);
                                ui.label(RichText::new(tr("Macros")).strong());
                                ui.label(RichText::new(tr("Named scripts you run by hand from the Device tab. One command per line; comments after ; or in brackets; @include(name) inserts another macro. Variables of the machine can be used.")).color(theme::text_dim()));
                                let mut remove = None;
                                for (i, m) in d.macros.iter_mut().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(&mut m.name).hint_text(tr("Macro name")).desired_width(200.0));
                                        if crate::window_ui::delete_icon(ui, &tr("Delete this macro")).clicked() {
                                            remove = Some(i);
                                        }
                                    });
                                    ui.add(egui::TextEdit::multiline(&mut m.code).font(egui::TextStyle::Monospace).desired_rows(3).desired_width(f32::INFINITY));
                                }
                                if let Some(i) = remove {
                                    d.macros.remove(i);
                                }
                                if ui.button(tr("Add a macro")).clicked() {
                                    d.macros.push(lc_core::Macro { name: trf("Macro {}", &[&(d.macros.len() + 1)]), code: String::new() });
                                }
                                ui.add_space(6.0);
                                if ui.button(tr("Restore defaults")).clicked() {
                                    d.gcode.reset_custom();
                                }
                            }
                        }
                    });
                });
            });
            cfg.page = page;
            cfg.head = cfg_head;
            ui.add_space(8.0);
            let bad_gcode = !cfg.draft.gcode.invalid_fields().is_empty();
            let hook_problem = cfg.draft.gcode.hooks.problem();
            if let Some((name, why)) = &hook_problem {
                ui.label(RichText::new(trf("Hook {}: {}", &[name, why])).color(Color32::from_rgb(0xc0, 0x30, 0x30)));
            }
            let macro_problem = lc_core::hooks::validate_macros(&cfg.draft);
            if let Some((name, why)) = &macro_problem {
                ui.label(RichText::new(trf("Macro {}: {}", &[name, why])).color(Color32::from_rgb(0xc0, 0x30, 0x30)));
            }
            let dup_tools = cfg.draft.has_duplicate_tools();
            let notes_long = cfg.draft.notes.chars().count() > lc_core::profiles::MAX_NOTES;
            let valid = !cfg.draft.name.trim().is_empty() && !notes_long && !bad_gcode && !dup_tools && hook_problem.is_none() && macro_problem.is_none();
            if dup_tools {
                ui.label(RichText::new(tr("Two laser heads have the same tool number.")).color(Color32::from_rgb(0xc0, 0x30, 0x30)));
            }
            if bad_gcode {
                ui.label(RichText::new(tr("A G-code field on the Advanced page has a control character or is too long.")).color(Color32::from_rgb(0xc0, 0x30, 0x30)));
            }
            if !cfg.draft.controller.is_serial() {
                ui.label(RichText::new(tr("This controller has no live connection: jobs are exported as files.")).color(theme::text_dim()));
            }
            if !valid {
                ui.label(RichText::new(tr("A profile name is required.")).color(Color32::from_rgb(0xc0, 0x30, 0x30)));
            }
            ui.horizontal(|ui| {
                if ui.add_enabled(valid, egui::Button::new(tr("Save"))).clicked() {
                    save = true;
                }
                if !must_create && ui.button(tr("Cancel")).clicked() {
                    cancel = true;
                }
            });
        });
        if let Some(cam) = test_cam {
            self.start_cam_test(ctx, cam);
        }
        if !cfg_open && !must_create {
            cancel = true;
        }
        self.run_fw_acts(fw_acts);
        if let Some(i) = maint_reset {
            let name = cfg.draft.maintenance.counters.get(i).map(|c| c.name.clone()).unwrap_or_default();
            self.confirm = Some(crate::session_ui::Confirm {
                title: tr("Reset the counter").to_string(),
                text: trf("Start the counter \"{}\" again from zero?", &[&name]),
                act: crate::session_ui::ConfirmAct::ResetCounter(i),
            });
        }
        if read_clicked {
            self.start_read(&cfg.draft);
        }
        if save {
            cfg.draft.name = cfg.draft.name.trim().to_string();
            // The time counted while the window was open belongs to the profile, not to the draft: names and limits
            // come from the draft, the times from the profile.
            if let Some(live) = cfg.idx.and_then(|i| self.profiles.get(i)) {
                cfg.draft.maintenance.total_secs = live.maintenance.total_secs;
                for (i, c) in cfg.draft.maintenance.counters.iter_mut().enumerate() {
                    if let Some(l) = live.maintenance.counters.get(i) {
                        c.secs = l.secs;
                    }
                }
            }
            let idx = match cfg.idx {
                Some(i) if i < self.profiles.len() => {
                    self.profiles[i] = cfg.draft.clone();
                    i
                }
                _ => {
                    cfg.draft.add_starter_materials();
                    self.profiles.push(cfg.draft.clone());
                    self.profiles.len() - 1
                }
            };
            self.start_sel = idx;
            if self.screen == Screen::Editor && idx == self.active {
                self.doc.device = cfg.draft;
                self.touch();
            }
        } else if !cancel {
            self.cfg = Some(cfg);
        }
    }

    /// The path of the job, shifted so the red pointer dot follows it, with the laser off. Errors (the shifted path
    /// leaves the allowed area) come back as issues.
    pub fn dry_run_lines(&self) -> Result<Vec<String>, Vec<lc_core::check::Issue>> {
        let h = self.doc.device.head(0);
        let moved = lc_core::gcode::shifted(&self.doc, -h.pointer_dx, -h.pointer_dy);
        let errors: Vec<_> = lc_core::check::job_check(&moved).into_iter().filter(|i| i.level == lc_core::check::Level::Error).collect();
        if !errors.is_empty() {
            return Err(errors);
        }
        let job = lc_core::gcode::generate(&moved);
        Ok(laser::clean_gcode(&lc_core::gcode::without_power(&job.gcode)))
    }

    /// Run the job's path with the laser off, shifted by the pointer offset.
    pub fn pointer_dry_run(&mut self) {
        if !self.connected || self.job_running() {
            return;
        }
        match self.dry_run_lines() {
            Ok(lines) if lines.len() > 1 => {
                self.focus_off();
                self.push_console(crate::laser::ConsoleLine::info(trf("Dry run with the pointer: {} lines, laser off", &[&lines.len()])));
                self.link.send(Cmd::Job(lines));
            }
            Ok(_) => self.status = tr("Nothing to run: no shapes on output layers").to_string(),
            Err(issues) => {
                self.check_dlg = Some(crate::check_ui::CheckDlg { issues, pending: crate::check_ui::Pending::DryRun });
            }
        }
    }

    /// Run a macro: the first time in a session the user is warned that it can fire the laser or move the head.
    pub fn request_macro(&mut self, name: &str) {
        if self.macros_ok {
            self.run_macro(name);
        } else {
            self.confirm = Some(crate::session_ui::Confirm {
                title: trf("Run the macro {}", &[&name]),
                text: tr("A macro is your own G-code: it can fire the laser or move the head. Run it only when you know what it does. This question is asked once per session.").to_string(),
                act: crate::session_ui::ConfirmAct::RunMacro(name.to_string()),
            });
        }
    }

    /// Send the lines of a macro to the machine.
    pub fn run_macro(&mut self, name: &str) {
        if !self.connected || self.job_running() {
            return;
        }
        match lc_core::hooks::expand(&self.doc.device, name) {
            Ok(lines) => {
                self.focus_off();
                self.push_console(crate::laser::ConsoleLine::info(trf("Macro {}: {} lines", &[&name, &lines.len()])));
                for l in lines {
                    self.link.send(Cmd::Line(l));
                }
            }
            Err(e) => self.status = trf("Macro {}: {}", &[&name, &e.to_string()]),
        }
    }

    /// Rescan the serial ports.
    pub fn refresh_ports(&mut self) {
        self.port_info = laser::list_port_info();
        self.ports = self.port_info.iter().map(|p| p.name.clone()).collect();
    }

    /// Open the link of the active device again, now (used after the link was lost).
    pub fn connect_now(&mut self) {
        let dev = self.doc.device.clone();
        if let Some(target) = Target::of(&dev) {
            self.link.send(Cmd::Connect { target, controller: dev.controller, streaming: dev.connection.streaming });
        }
    }

    /// Try again every few seconds after a lost link, while the user has not asked to disconnect.
    pub fn reconnect_tick(&mut self, ctx: &egui::Context) {
        let Some(at) = self.reconnect_at else { return };
        if self.connected || !self.want_link || !self.doc.device.connection.auto_reconnect {
            self.reconnect_at = None;
            return;
        }
        if std::time::Instant::now() >= at {
            self.status = tr("Connecting again…").to_string();
            self.connect_now();
            self.reconnect_at = Some(std::time::Instant::now() + std::time::Duration::from_secs(4));
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(500));
    }

    /// A jog step of the Z axis, `dir` = 1 for up, -1 for down, kept inside the travel of the axis.
    pub fn jog_z_action(&mut self, dir: f64) -> Option<Action> {
        let dev = &self.doc.device;
        let (lo, hi) = (dev.axes.z_min.min(dev.axes.z_max), dev.axes.z_min.max(dev.axes.z_max));
        let mut dz = dir * if dev.axes.flip_z { -1.0 } else { 1.0 } * dev.jog_step;
        if self.connected && self.machine_z.is_finite() {
            let target = (self.machine_z + dz).clamp(lo, hi);
            dz = target - self.machine_z;
        }
        if dz.abs() < 1e-9 {
            self.status = tr("The head is at the edge of the allowed area.").to_string();
            return None;
        }
        Some(dev.controller.jog_z(dz, dev.jog_feed))
    }

    /// A jog step in the design's directions (x right, y down), kept inside the allowed area. `None` when there is no room.
    pub(crate) fn jog_action(&mut self, dx: f64, dy: f64) -> Option<Action> {
        let dev = &self.doc.device;
        let at = (self.connected && self.machine.1.is_finite()).then_some((self.machine.1, self.machine.2));
        let (mx, my) = dev.limited_jog(at, dx, dy);
        if mx == 0.0 && my == 0.0 {
            self.status = tr("The head is at the edge of the allowed area.").to_string();
            return None;
        }
        Some(dev.controller.jog(mx, my, dev.jog_feed))
    }

    /// Short description for the status bar / logs.
    #[allow(dead_code)]
    pub fn active_name(&self) -> String {
        trf("Device: {}", &[&self.profiles.get(self.active).map(|d| d.name.clone()).unwrap_or_default()])
    }

    /// Make `idx` the active profile (disconnects first).
    pub fn switch_profile(&mut self, idx: usize) {
        let Some(dev) = self.profiles.get(idx).cloned() else { return };
        if self.connected {
            self.link.send(Cmd::Disconnect);
        }
        self.active = idx;
        self.start_sel = idx;
        self.doc.device = dev;
        self.touch();
    }

    /// Keep the stored profile in step with live edits made in the Device tab.
    pub fn sync_profile(&mut self) {
        if let Some(p) = self.profiles.get_mut(self.active) {
            if *p != self.doc.device {
                *p = self.doc.device.clone();
            }
        }
    }

    pub fn run_action(&mut self, a: Action) {
        match a {
            Action::Lines(ls) => {
                for l in ls {
                    self.link.send(Cmd::Line(l));
                }
            }
            Action::Byte(b) => self.link.send(Cmd::Realtime(b)),
            Action::None => {}
        }
    }

    /// The Device tab: choose a device, see its settings, connect and move the machine.
    pub fn device_tab(&mut self, ui: &mut egui::Ui) {
        let units = self.doc.device.units;
        let ctrl = self.doc.device.controller;
        ui.horizontal_wrapped(|ui| {
            ui.label(tr("Device"));
            let mut sel = self.active;
            egui::ComboBox::from_id_salt("dev_sel")
                .width((ui.available_width() - 70.0).clamp(80.0, 190.0))
                .selected_text(self.profiles.get(self.active).map(|d| d.name.clone()).unwrap_or_default())
                .show_ui(ui, |ui| {
                    for (i, d) in self.profiles.iter().enumerate() {
                        ui.selectable_value(&mut sel, i, &d.name);
                    }
                });
            if sel != self.active {
                self.switch_profile(sel);
            }
        });
        ui.horizontal_wrapped(|ui| {
            if ui.button(tr("Edit…")).clicked() {
                self.open_cfg_edit(self.active);
            }
            if ui.button(tr("Add device…")).clicked() {
                self.open_cfg_new();
            }
            if ui.button(tr("Switch device…")).clicked() {
                self.start_sel = self.active;
                self.confirm_delete = false;
                self.screen = Screen::Start;
            }
        });
        ui.add_space(4.0);
        let dev = self.doc.device.clone();
        let yes_no = |b: bool| if b { tr("Yes") } else { tr("No") };
        let half = ((ui.available_width() - 12.0) / 2.0).max(60.0);
        egui::Grid::new("dev_summary").num_columns(2).min_col_width(half).max_col_width(half).spacing([12.0, 3.0]).striped(true).show(ui, |ui| {
            let mut row = |k: &str, v: String| {
                ui.label(RichText::new(k).color(theme::text_dim()));
                ui.label(v);
                ui.end_row();
            };
            row(tr("Controller"), dev.controller.label().to_string());
            row(tr("Laser type"), dev.laser().label().to_string());
            row(tr("Units"), tr(dev.units.label()).to_string());
            row(tr("Speed units"), tr(dev.speed_unit.label()).to_string());
            row(tr("Work area"), format!("{} × {}", fmt_len(units, dev.area.bed_w, 0), fmt_len(units, dev.area.bed_h, 0)));
            row(tr("Machine zero (0,0)"), tr(dev.axes.origin.label()).to_string());
            row(tr("S-value max ($30)"), format!("{}", dev.s_max()));
            row(tr("Dynamic power (M4)"), yes_no(dev.gcode.dynamic_power).to_string());
            row(tr("Travel speed"), fmt_speed(units, dev.speed_unit, true, dev.speeds.travel_speed));
            row(tr("Return to origin"), yes_no(dev.gcode.return_home).to_string());
            if dev.controller.is_serial() {
                row(tr("Connection type"), tr(dev.connection.link.label()).to_string());
                if dev.connection.link == LinkKind::Serial {
                    row(tr("Baud rate"), dev.connection.baud.to_string());
                }
                row(tr("Port"), if dev.has_target() { dev.target_label() } else { tr("No port").to_string() });
            }
            if let Some(c) = dev.camera_source() {
                row(tr("Camera"), c.name.clone());
                if let lc_core::CameraKind::Url(u) = &c.kind {
                    row(tr("Camera URL"), u.clone());
                }
                row(tr("Camera rotation"), format!("{}°", c.rotation as u32 % 4 * 90));
            }
        });
        ui.add_space(6.0);
        ui.separator();
        ui.label(RichText::new(tr("Connection")).strong());
        if !ctrl.is_serial() {
            ui.label(RichText::new(tr("This controller has no live connection: jobs are exported as files.")).color(theme::text_dim()));
            ui.label(RichText::new(tr("Export PLT or DXF and open it in the controller's own software (for example RDWorks for Ruida), which sets speed and power per layer colour.")).color(theme::text_dim()));
            if ui.button(tr("Export PLT / DXF…")).clicked() {
                self.export_cam();
            }
            self.sync_profile();
            return;
        }
        let mut port = self.doc.device.connection.port.clone();
        let tcp = self.doc.device.connection.link == LinkKind::Tcp;
        ui.horizontal_wrapped(|ui| {
            if tcp {
                ui.label(RichText::new(self.doc.device.target_label()).monospace());
            } else {
                egui::ComboBox::from_id_salt("port").width(150.0).selected_text(if port.is_empty() { tr("No port") } else { port.as_str() }).show_ui(ui, |ui| {
                    for p in self.port_info.clone() {
                        ui.selectable_value(&mut port, p.name.clone(), p.label());
                    }
                });
                if ui.button(tr("Refresh")).on_hover_text(tr("Rescan serial ports")).clicked() {
                    self.refresh_ports();
                    if !self.ports.contains(&port) {
                        port = self.ports.first().cloned().unwrap_or_default();
                    }
                }
            }
            if self.connected {
                let purple = egui::Button::new(RichText::new(tr("Disconnect")).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0x8e, 0x44, 0xad));
                if ui.add(purple).clicked() {
                    self.want_link = false;
                    self.reconnect_at = None;
                    self.link.send(Cmd::Disconnect);
                }
            } else {
                let mut dev = self.doc.device.clone();
                dev.connection.port = port.clone();
                let green = egui::Button::new(RichText::new(tr("Connect")).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0x2e, 0xa0, 0x4f));
                if ui.add_enabled(dev.has_target(), green).clicked() {
                    if let Some(target) = Target::of(&dev) {
                        self.want_link = true;
                        self.link.send(Cmd::Connect { target, controller: ctrl, streaming: self.doc.device.connection.streaming });
                    }
                }
            }
        });
        self.doc.device.connection.port = port;
        ui.label(format!("{}: {}   X {:.2}  Y {:.2}", tr("State"), self.machine.0, self.machine.1, self.machine.2));
        let mut act: Option<Action> = None;
        // Options of the device that belong to jogging and framing, always editable.
        let running = self.job_running();
        ui.add_enabled_ui(!running, |ui| egui::Grid::new("jog_opts").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
            ui.label(tr("Jog step"));
            drag_len(ui, units, &mut self.doc.device.jog_step, 0.1, Some((0.1, 200.0)));
            ui.end_row();
            ui.label(tr("Jog feed"));
            drag_speed(ui, units, self.doc.device.speed_unit, true, &mut self.doc.device.jog_feed, 10.0, Some((10.0, 20000.0)));
            ui.end_row();
            ui.label(tr("Frame power"));
            ui.add(egui::DragValue::new(&mut self.doc.device.heads[0].frame_power).range(0.0..=10.0).suffix(" %")).on_hover_text(tr("0 % keeps the laser off while framing; 1–2 % shows a dim dot on diode lasers"));
            ui.end_row();
        }));
        ui.separator();
        ui.add_enabled_ui(self.connected, |ui| {
            let s = self.doc.device.jog_step;
            // Arrow keys and the controls below are buttons of one size.
            let size = egui::vec2(60.0, 48.0);
            let icon = |ui: &mut egui::Ui, name: &str, tip: String, accent: Option<Color32>| -> bool {
                let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
                let hovered = resp.hovered() && ui.is_enabled();
                let fill = match (accent, hovered) {
                    (Some(c), true) => c.gamma_multiply(0.85),
                    (Some(c), false) => c,
                    (None, true) => theme::panel_dark(),
                    (None, false) => ui.visuals().widgets.inactive.weak_bg_fill,
                };
                ui.painter().rect_filled(rect, 4.0, fill);
                ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(1.0, theme::border()), egui::StrokeKind::Inside);
                let ink = if accent.is_some() { Color32::WHITE } else if ui.is_enabled() { theme::text() } else { theme::text().gamma_multiply(0.4) };
                crate::icons::paint(ui, egui::Rect::from_center_size(rect.center(), egui::vec2(27.0, 27.0)), name, ink);
                resp.on_hover_text(tip).clicked()
            };
            let jog_tip = |a: &'static str, b: &'static str| if b.is_empty() { tr(a).to_string() } else { format!("{} + {}", tr(a), tr(b)) };
            let centre = |ui: &mut egui::Ui, total: f32| ui.add_space(((ui.available_width() - total) / 2.0).max(0.0));
            // While a job runs the machine must not be moved by hand: only pause, resume and stop stay available.
            ui.add_enabled_ui(!running, |ui| ui.horizontal(|ui| {
            centre(ui, 3.0 * size.x + 2.0 * 4.0);
            egui::Grid::new("jog").spacing([4.0, 4.0]).show(ui, |ui| {
                // The four diagonals move both axes by one step at once.
                for (name, a, b, dx, dy) in [("arrow-up-left", "Up", "Left", -s, -s), ("arrow-up", "Up", "", 0.0, -s), ("arrow-up-right", "Up", "Right", s, -s)] {
                    if icon(ui, name, jog_tip(a, b), None) {
                        act = self.jog_action(dx, dy);
                    }
                }
                ui.end_row();
                if icon(ui, "arrow-left", tr("Left").to_string(), None) {
                    act = self.jog_action(-s, 0.0);
                }
                if icon(ui, "square", tr("Cancel jog").to_string(), Some(Color32::from_rgb(0xe8, 0x8a, 0x1f))) {
                    act = Some(ctrl.cancel_jog());
                }
                if icon(ui, "arrow-right", tr("Right").to_string(), None) {
                    act = self.jog_action(s, 0.0);
                }
                ui.end_row();
                for (name, a, b, dx, dy) in [("arrow-down-left", "Down", "Left", -s, s), ("arrow-down", "Down", "", 0.0, s), ("arrow-down-right", "Down", "Right", s, s)] {
                    if icon(ui, name, jog_tip(a, b), None) {
                        act = self.jog_action(dx, dy);
                    }
                }
                ui.end_row();
            });
            }));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                centre(ui, 4.0 * size.x + 3.0 * ui.spacing().item_spacing.x);
                ui.add_enabled_ui(!running, |ui| {
                    if icon(ui, "house", tr("Home").to_string(), None) {
                        act = Some(ctrl.home());
                    }
                });
                if self.single_axis_homing {
                    // Only when the firmware reported it was built with `$HX` / `$HY`.
                    ui.add_enabled_ui(!running, |ui| {
                        for axis in ["X", "Y"] {
                            if ui.small_button(trf("Home {}", &[&axis])).on_hover_text(trf("Homes the {} axis only ($H{})", &[&axis, &axis])).clicked() {
                                act = Some(Action::Lines(vec![format!("$H{axis}")]));
                            }
                        }
                    });
                }
                ui.add_enabled_ui(!running, |ui| {
                    if icon(ui, "lock-open", tr("Unlock").to_string(), None) {
                        act = Some(ctrl.unlock());
                    }
                });
                if icon(ui, "pause", tr("Pause").to_string(), None) {
                    act = Some(ctrl.pause());
                }
                if icon(ui, "play", tr("Resume").to_string(), None) {
                    act = Some(ctrl.resume());
                }
            });
            ui.add_space(6.0);
            if ui.add_sized([ui.available_width(), 40.0], egui::Button::new(RichText::new(tr("STOP")).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0xc0, 0x30, 0x30))).clicked() {
                self.abort_job();
            }
            ui.add_space(4.0);
            self.focus_button(ui, true);
            if self.doc.device.axes.has_z {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(tr("Z"));
                    ui.add_enabled_ui(!running, |ui| {
                        if ui.button(tr("Up")).clicked() {
                            act = self.jog_z_action(1.0);
                        }
                        if ui.button(tr("Down")).clicked() {
                            act = self.jog_z_action(-1.0);
                        }
                    });
                    if self.machine_z.is_finite() {
                        ui.label(RichText::new(format!("Z {:.2}", self.machine_z)).color(theme::text_dim()));
                    }
                });
            }
            if !self.doc.device.macros.is_empty() {
                ui.add_space(4.0);
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
                ui.checkbox(&mut self.pointer_align, tr("Align to the pointer")).on_hover_text(tr("This session only: the frame and the dry run move back by the offset of the red pointer dot, so the dot lands where the beam would. Jobs and jogging are never moved."));
                if ui.add_enabled(self.connected && !running, egui::Button::new(tr("Dry run with the pointer"))).on_hover_text(tr("Runs the path of the job shifted by the pointer offset with the laser off, to see where it would go.")).clicked() {
                    self.pointer_dry_run();
                }
            }
        });
        if let Some(a) = act {
            self.run_action(a);
        }
        self.sync_profile();
    }

    /// Ask the controller for its settings (connecting first if needed).
    pub fn start_read(&mut self, draft: &Device) {
        let c = draft.controller;
        let Some(_) = c.settings_request() else { return };
        let mut st = ReadCfg { controller: c, waiting_connect: false, request_at: None, sent_at: None, lines: vec![], started: std::time::Instant::now(), kind: ReadKind::Config };
        if self.connected {
            st.request_at = Some(std::time::Instant::now());
        } else {
            st.waiting_connect = true;
            match Target::of(draft) {
                Some(target) => self.link.send(Cmd::Connect { target, controller: c, streaming: draft.connection.streaming }),
                None => return,
            }
        }
        self.read_cfg = Some(st);
        self.status = tr("Reading settings from the device…").to_string();
    }

    /// Called when the connection state changes.
    pub fn read_on_connected(&mut self, up: bool) {
        if up {
            let c = &self.doc.device.connection;
            if c.home_on_connect || c.unlock_on_connect {
                self.connect_actions_at = Some(std::time::Instant::now() + std::time::Duration::from_millis(1400));
            }
        } else {
            self.single_axis_homing = false;
            self.connect_actions_at = None;
        }
        // Once per connection a GRBL controller is asked for its settings, to warn about laser mode and $30.
        if up && self.read_cfg.is_none() && self.doc.device.controller == Controller::Grbl && self.screen == Screen::Editor {
            let now = std::time::Instant::now();
            self.read_cfg = Some(ReadCfg { controller: Controller::Grbl, waiting_connect: false, request_at: Some(now + std::time::Duration::from_millis(900)), sent_at: None, lines: vec![], started: now, kind: ReadKind::Probe });
        }
        if let Some(r) = &mut self.read_cfg {
            if r.waiting_connect {
                if up {
                    r.waiting_connect = false;
                    r.request_at = Some(std::time::Instant::now() + std::time::Duration::from_millis(900));
                } else {
                    self.read_cfg = None;
                    self.status = tr("Could not connect to the device.").to_string();
                }
            }
        }
    }

    /// Collect response lines while a read is running.
    pub fn read_feed(&mut self, line: &str) {
        let Some(r) = &mut self.read_cfg else { return };
        if r.sent_at.is_none() {
            return;
        }
        if line == "ok" || line.starts_with("error") {
            self.finish_read();
        } else {
            r.lines.push(line.to_string());
        }
    }

    /// What the device is set to do when the link comes up: clear an alarm, then home.
    pub fn connect_actions_tick(&mut self, ctx: &egui::Context) {
        let Some(at) = self.connect_actions_at else { return };
        if std::time::Instant::now() < at {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
            return;
        }
        self.connect_actions_at = None;
        if !self.connected || self.job_running() {
            return;
        }
        let (c, ctrl) = (self.doc.device.connection.clone(), self.doc.device.controller);
        if c.unlock_on_connect {
            self.run_action(ctrl.unlock());
        }
        if c.home_on_connect {
            self.run_action(ctrl.home());
        }
    }

    /// Send the request when due and give up after a timeout.
    pub fn read_tick(&mut self, ctx: &egui::Context) {
        self.connect_actions_tick(ctx);
        let Some(r) = &mut self.read_cfg else { return };
        let now = std::time::Instant::now();
        if let Some(at) = r.request_at {
            if now >= at {
                r.request_at = None;
                r.sent_at = Some(now);
                if let Some(req) = r.request() {
                    self.link.send(Cmd::Line(req.to_string()));
                }
            }
        }
        let no_connection = r.waiting_connect && now.duration_since(r.started).as_secs_f32() > 6.0;
        let timed_out = r.sent_at.is_some_and(|t| now.duration_since(t).as_secs_f32() > 4.0);
        if no_connection {
            self.read_cfg = None;
            self.status = tr("Could not connect to the device.").to_string();
        } else if timed_out {
            self.finish_read();
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(150));
    }

    pub fn finish_read(&mut self) {
        let Some(r) = self.read_cfg.take() else { return };
        let reading = r.controller.parse_settings(&r.lines);
        if r.kind == ReadKind::Probe {
            let check = reading.check(&self.doc.device);
            if !check.is_clean() {
                self.fw_warn = Some(check);
            }
            // Then ask which options the firmware was built with.
            let now = std::time::Instant::now();
            self.read_cfg = Some(ReadCfg { controller: r.controller, waiting_connect: false, request_at: Some(now), sent_at: None, lines: vec![], started: now, kind: ReadKind::Info });
            return;
        }
        if r.kind == ReadKind::Info {
            self.single_axis_homing = lc_core::controller::grbl_options(&r.lines).is_some_and(|o| lc_core::controller::single_axis_homing(&o));
            self.fw.info = r.lines;
            return;
        }
        if r.kind == ReadKind::Firmware {
            self.fw_got(r.lines);
            return;
        }
        let n = reading.count();
        if n == 0 {
            self.status = tr("The device did not report any settings.").to_string();
            return;
        }
        if let Some(cfg) = &mut self.cfg {
            let d = &mut cfg.draft;
            if let Some(v) = reading.bed_w {
                d.area.bed_w = v;
            }
            if let Some(v) = reading.bed_h {
                d.area.bed_h = v;
            }
            if let Some(v) = reading.s_max {
                d.heads[0].s_max = v;
            }
            if let Some(v) = reading.travel_speed {
                d.speeds.travel_speed = v;
            }
            if let Some(v) = reading.dynamic_power {
                d.gcode.dynamic_power = v;
            }
        }
        if self.wizard.is_some() {
            self.wizard_read_done = true;
        }
        self.status = trf("Read {} values from the device", &[&n]);
    }
}

/// A new, switched-off camera at the end of the list. It is the one being set up, so it becomes the active one.
pub fn add_camera(d: &mut lc_core::Device) {
    d.cameras.push(lc_core::CameraSource { name: trf("Camera {}", &[&(d.cameras.len() + 1)]), enabled: false, kind: lc_core::CameraKind::Url(String::new()), rotation: 0, image: Default::default(), lens: Default::default(), reference: vec![] });
    d.active_camera = d.cameras.len() - 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_camera_becomes_the_active_one() {
        let mut d = lc_core::Device::default();
        add_camera(&mut d);
        assert_eq!((d.cameras.len(), d.active_camera), (1, 0));
        add_camera(&mut d);
        add_camera(&mut d);
        assert_eq!((d.cameras.len(), d.active_camera), (3, 2));
        assert!(!d.cameras[2].enabled, "it stays off until it has a valid address");
    }

    #[test]
    fn diagonal_jogs_move_both_axes_and_have_icons() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.connected = false;
        for (dx, dy) in [(-5.0, -5.0), (5.0, -5.0), (-5.0, 5.0), (5.0, 5.0)] {
            let Some(Action::Lines(l)) = a.jog_action(dx, dy) else { panic!("a jog") };
            // The design's Y runs down the screen and the default zero is at the front: the machine's Y is flipped.
            let (x, y) = (format!("X{}", if dx < 0.0 { "-5" } else { "5" }), format!("Y{}", if dy < 0.0 { "5" } else { "-5" }));
            assert!(l[0].contains(&x) && l[0].contains(&y), "{dx},{dy}: {}", l[0]);
        }
        let names = crate::icons::all_names();
        for n in ["arrow-up-left", "arrow-up-right", "arrow-down-left", "arrow-down-right"] {
            assert!(names.contains(&n), "{n}");
        }
    }

    fn app() -> App {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.profiles = vec![Device::default()];
        a
    }

    #[test]
    fn a_profile_can_be_copied_exported_and_imported_again() {
        let mut a = app();
        a.duplicate_profile(0);
        a.duplicate_profile(0);
        let names: Vec<String> = a.profiles.iter().map(|p| p.name.clone()).collect();
        assert_eq!(names.len(), 3);
        assert_eq!(names[1], format!("{} (copy)", names[0]));
        assert_eq!(names[2], format!("{} (copy 2)", names[0]));
        assert_eq!(a.start_sel, 2);
        // A file made by the program comes back as a profile under a free name (the first name is taken).
        let dir = std::env::temp_dir().join(format!("lc-profile-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("p.json");
        let mut p = a.profiles[0].clone();
        p.connection.port = "/dev/ttyUSB7".into();
        std::fs::write(&file, lc_core::profiles::export_profile(&p, APP_VERSION)).unwrap();
        a.import_profile_from(&file);
        assert_eq!(a.profiles.len(), 4);
        assert!(a.profiles[3].connection.port.is_empty(), "the port is not taken from the file");
        assert!(a.profiles[3].name.contains("(copy"), "{}", a.profiles[3].name);
        // Garbage and missing files change nothing and say why.
        std::fs::write(&file, "not json").unwrap();
        a.import_profile_from(&file);
        assert_eq!(a.profiles.len(), 4);
        a.import_profile_from(&dir.join("missing.json"));
        assert_eq!(a.profiles.len(), 4);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
