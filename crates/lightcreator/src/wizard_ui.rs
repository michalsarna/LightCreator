//! First-run setup: when there is no profile yet, a few steps lead to one: a template (or a machine read from its
//! controller), the connection, a name and units, and a summary with the things worth checking.
use crate::app::App;
use crate::device_ui::CfgEdit;
use crate::i18n::tr;
use eframe::egui::{self, Color32, RichText};
use lc_core::{profiles, Controller, Device, LinkKind, Units};

impl App {
    /// Start the first-run setup on a profile made from the default template.
    pub fn start_wizard(&mut self) {
        self.cfg = Some(CfgEdit { idx: None, draft: profiles::template("grbl-diode-400"), page: crate::device_ui::CfgPage::General, head: 0 });
        self.wizard = Some(0);
    }

    /// The things to look at before a new profile is used, from what is known now.
    pub fn wizard_warnings(d: &Device, read: bool) -> Vec<String> {
        let mut w = vec![];
        if d.controller == Controller::Grbl && !read {
            w.push(tr("Laser mode ($32) and the maximum S value ($30) of the controller were not read: they are checked when you connect.").to_string());
        }
        if d.controller.is_serial() && !d.has_target() {
            w.push(tr("No port or host is set yet: choose it in the device settings before connecting.").to_string());
        }
        if d.area.margins == lc_core::Margins::default() && d.area.soft_limits.is_none() {
            w.push(tr("No margins or software limits are set: the whole axis range is the work area. Set them in the Hardware page if a clamp or a frame is in the way.").to_string());
        }
        w.push(tr("Nothing here has been tried on your machine: run a first test with the laser off or at the lowest power.").to_string());
        w
    }

    /// The one window for adding a device: a starting point, the basic settings collected from the device window, the
    /// connection (with a read from the controller) and the things worth checking. "More settings…" opens the full
    /// device window with what is entered here.
    pub fn wizard_window(&mut self, ctx: &egui::Context) {
        if self.wizard.is_none() {
            return;
        }
        let Some(mut cfg) = self.cfg.take() else {
            self.wizard = None;
            return;
        };
        let mut finish = false;
        let mut by_hand = false;
        let mut read = false;
        let reading = self.read_cfg.is_some();
        let was_read = self.wizard_read;
        let mut open = true;
        let mut close = false;
        crate::window_ui::popup(ctx, "first_setup", &tr("Add a device"), [520.0, 560.0], &mut open, |ui| {
            let d = &mut cfg.draft;
            egui::ScrollArea::vertical().max_height((ui.ctx().content_rect().height() - 190.0).clamp(200.0, 520.0)).auto_shrink([false, true]).show(ui, |ui| {
                egui::Grid::new("add_dev").num_columns(2).min_col_width(140.0).spacing([12.0, 6.0]).show(ui, |ui| {
                    ui.label(tr("Starting point"));
                    let current = profiles::TEMPLATES.iter().find(|t| d.template_notes == tr(t.desc) || d.name == t.name).map(|t| tr(t.name).to_string());
                    egui::ComboBox::from_id_salt("add_template").width(260.0).selected_text(current.unwrap_or_else(|| tr("A blank profile").to_string())).show_ui(ui, |ui| {
                        for t in profiles::TEMPLATES {
                            if ui.selectable_label(false, tr(t.name)).on_hover_text(tr(t.desc)).clicked() {
                                // The name that was typed stays.
                                let typed = d.name.clone();
                                let keep = !typed.trim().is_empty() && !profiles::TEMPLATES.iter().any(|x| x.name == typed);
                                *d = profiles::template(t.id);
                                if keep {
                                    d.name = typed;
                                }
                            }
                        }
                    });
                    ui.end_row();
                    ui.label(tr("Name"));
                    ui.add(egui::TextEdit::singleline(&mut d.name).hint_text(tr("e.g. My diode laser")));
                    ui.end_row();
                    ui.label(tr("Controller"));
                    egui::ComboBox::from_id_salt("add_controller").selected_text(d.controller.label()).show_ui(ui, |ui| {
                        for c in lc_core::Controller::ALL {
                            ui.selectable_value(&mut d.controller, c, c.label());
                        }
                    });
                    ui.end_row();
                    ui.label(tr("Laser type"));
                    egui::ComboBox::from_id_salt("add_laser").selected_text(tr(d.heads[0].laser.label())).show_ui(ui, |ui| {
                        for k in lc_core::LaserKind::ALL {
                            ui.selectable_value(&mut d.heads[0].laser, k, tr(k.label()));
                        }
                    });
                    ui.end_row();
                    ui.label(tr("Units"));
                    egui::ComboBox::from_id_salt("add_units").selected_text(tr(d.units.label())).show_ui(ui, |ui| {
                        for u in Units::ALL {
                            ui.selectable_value(&mut d.units, u, tr(u.label()));
                        }
                    });
                    ui.end_row();
                    let u = d.units;
                    ui.label(tr("Work area X"));
                    crate::units_ui::drag_len(ui, u, &mut d.area.bed_w, 1.0, Some((10.0, 5000.0)));
                    ui.end_row();
                    ui.label(tr("Work area Y"));
                    crate::units_ui::drag_len(ui, u, &mut d.area.bed_h, 1.0, Some((10.0, 5000.0)));
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
                    ui.label(tr("Connection type"));
                    egui::ComboBox::from_id_salt("add_link").selected_text(tr(d.connection.link.label())).show_ui(ui, |ui| {
                        for k in LinkKind::ALL {
                            ui.selectable_value(&mut d.connection.link, k, tr(k.label()));
                        }
                    });
                    ui.end_row();
                    if d.connection.link == LinkKind::Serial {
                        ui.label(tr("Port"));
                        ui.horizontal(|ui| {
                            egui::ComboBox::from_id_salt("add_port").width(220.0).selected_text(if d.connection.port.is_empty() { tr("No port") } else { d.connection.port.as_str() }).show_ui(ui, |ui| {
                                for p in &self.port_info {
                                    ui.selectable_value(&mut d.connection.port, p.name.clone(), p.label());
                                }
                            });
                            if ui.button(tr("Refresh")).clicked() {
                                self.port_info = crate::laser::list_port_info();
                                self.ports = self.port_info.iter().map(|p| p.name.clone()).collect();
                            }
                        });
                        ui.end_row();
                    } else {
                        ui.label(tr("Host"));
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut d.connection.host).hint_text("raspberrypi.local").desired_width(160.0));
                            ui.label(tr("TCP port"));
                            ui.add(egui::DragValue::new(&mut d.connection.tcp_port).range(1..=65535));
                        });
                        ui.end_row();
                    }
                    ui.label("");
                    let label = if reading { tr("Reading…") } else { tr("Read from device") };
                    if ui.add_enabled(!reading && d.has_target() && d.controller.settings_request().is_some(), egui::Button::new(label)).on_hover_text(tr("If the machine is connected now, its work area and speed can be read from the controller.")).clicked() {
                        read = true;
                    }
                    ui.end_row();
                });
                if was_read {
                    ui.label(RichText::new(tr("The settings were read from the controller.")).color(Color32::from_rgb(0x2e, 0xa0, 0x4f)));
                }
                ui.add_space(6.0);
                ui.label(RichText::new(tr("Worth checking")).strong());
                for w in Self::wizard_warnings(d, was_read) {
                    ui.label(RichText::new(format!("• {w}")).small().color(Color32::from_rgb(0xe8, 0x8a, 0x1f)));
                }
            });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let go = egui::Button::new(RichText::new(tr("Add device")).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0x2e, 0xa0, 0x4f));
                if ui.add_enabled(!d.name.trim().is_empty(), go).clicked() {
                    finish = true;
                }
                if ui.button(tr("More settings…")).on_hover_text(tr("Open the whole device window with what is entered here")).clicked() {
                    by_hand = true;
                }
                if ui.button(tr("Cancel")).clicked() {
                    close = true;
                }
            });
        });
        if read {
            let draft = cfg.draft.clone();
            self.wizard_read = false;
            self.start_read(&draft);
        }
        if self.wizard_read_done {
            self.wizard_read = true;
            self.wizard_read_done = false;
        }
        if finish {
            let mut dev = cfg.draft;
            dev.name = dev.name.trim().to_string();
            dev.add_starter_materials();
            self.profiles.push(dev);
            self.start_sel = self.profiles.len() - 1;
            self.wizard = None;
            self.wizard_read = false;
            self.status = tr("The device was added. Press Continue to start.").to_string();
            return;
        }
        if by_hand {
            self.wizard = None;
            self.wizard_skipped = true;
            self.cfg = Some(cfg);
            return;
        }
        if close || !open {
            self.wizard = None;
            self.wizard_read = false;
            if self.profiles.is_empty() {
                // Without any profile the program cannot start: the plain device window comes back.
                self.wizard_skipped = true;
                self.cfg = Some(cfg);
            }
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
        a.start_wizard();
        assert_eq!(a.wizard, Some(0));
        let d = a.cfg.as_ref().unwrap().draft.clone();
        assert_eq!((d.controller, d.area.bed_w), (Controller::Grbl, 400.0));
        // A GRBL machine that was not read, with no port and no margins: three things, and the safety reminder last.
        let w = App::wizard_warnings(&d, false);
        assert_eq!(w.len(), 4, "{w:?}");
        // After reading, with a port and a margin, only the reminder is left.
        let mut set = d.clone();
        set.connection.port = "/dev/ttyUSB0".into();
        set.area.margins.left = 5.0;
        assert_eq!(App::wizard_warnings(&set, true).len(), 1);
        // A machine that is not streamed (Ruida) has no port to ask for.
        let mut ruida = d.clone();
        ruida.controller = Controller::Ruida;
        assert_eq!(App::wizard_warnings(&ruida, false).len(), 2);
    }

    #[test]
    fn leaving_the_setup_brings_the_plain_device_window_and_does_not_ask_again() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.profiles.clear();
        a.start_wizard();
        // The device window stays out of the way while the setup is open.
        a.config_window(&ctx);
        assert!(a.cfg.is_some());
        a.wizard = None;
        a.wizard_skipped = true;
        assert!(a.cfg.is_some() && a.wizard.is_none());
    }

    #[test]
    fn adding_a_device_opens_one_window_and_cancelling_it_keeps_the_profiles() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.profiles = vec![Device::default()];
        a.start_wizard();
        assert_eq!(a.wizard, Some(0));
        let mut out = ctx.run_ui(egui::RawInput::default(), |_| {});
        a.wizard_window(&ctx);
        out.textures_delta.clear();
        assert!(a.wizard.is_some() && a.cfg.is_some(), "one window, on the draft");
        // The draft carries what the device window has: controller, laser type, work area, zero, S max, connection.
        let d = &a.cfg.as_ref().unwrap().draft;
        assert!(d.area.bed_w > 0.0 && d.heads[0].s_max > 0.0 && !d.name.is_empty());
        assert_eq!(a.profiles.len(), 1);
    }
}
