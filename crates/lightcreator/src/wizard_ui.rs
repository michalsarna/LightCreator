//! First-run setup: when there is no profile yet, a few steps lead to one: a template (or a machine read from its
//! controller), the connection, a name and units, and a summary with the things worth checking.
use crate::app::App;
use crate::device_ui::CfgEdit;
use crate::i18n::{tr, trf};
use crate::theme;
use crate::units_ui::fmt_len;
use eframe::egui::{self, Color32, RichText};
use lc_core::{profiles, Controller, Device, LinkKind, Units};

pub const STEPS: usize = 4;

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

    pub fn wizard_window(&mut self, ctx: &egui::Context) {
        let Some(step) = self.wizard else { return };
        let Some(mut cfg) = self.cfg.take() else {
            self.wizard = None;
            return;
        };
        let mut next = step;
        let mut finish = false;
        let mut by_hand = false;
        let mut read = false;
        let reading = self.read_cfg.is_some();
        let was_read = self.wizard_read;
        let mut open = true;
        let mut close = false;
        crate::window_ui::popup(ctx, "first_setup", &trf("Set up your machine ({}/{})", &[&(step + 1), &STEPS]), [540.0, 460.0], &mut open, |ui| {
            let d = &mut cfg.draft;
            match step {
                0 => {
                    ui.label(RichText::new(tr("What machine do you have?")).strong());
                    ui.label(RichText::new(tr("Pick the closest starting point. You can change everything later.")).color(theme::text_dim()));
                    for t in profiles::TEMPLATES {
                        let selected = d.name == t.name;
                        if ui.add_sized([ui.available_width(), 34.0], egui::Button::selectable(selected, tr(t.name))).clicked() {
                            *d = profiles::template(t.id);
                        }
                        ui.label(RichText::new(tr(t.desc)).small().color(theme::text_dim()));
                    }
                }
                1 => {
                    ui.label(RichText::new(tr("Connection (optional)")).strong());
                    ui.label(RichText::new(tr("If the machine is connected now, its work area and speed can be read from the controller.")).color(theme::text_dim()));
                    ui.horizontal(|ui| {
                        ui.label(tr("Connection type"));
                        egui::ComboBox::from_id_salt("wiz_link").selected_text(tr(d.connection.link.label())).show_ui(ui, |ui| {
                            for k in LinkKind::ALL {
                                ui.selectable_value(&mut d.connection.link, k, tr(k.label()));
                            }
                        });
                    });
                    if d.connection.link == LinkKind::Serial {
                        ui.horizontal(|ui| {
                            ui.label(tr("Port"));
                            egui::ComboBox::from_id_salt("wiz_port").width(240.0).selected_text(if d.connection.port.is_empty() { tr("No port") } else { d.connection.port.as_str() }).show_ui(ui, |ui| {
                                for p in &self.port_info {
                                    ui.selectable_value(&mut d.connection.port, p.name.clone(), p.label());
                                }
                            });
                            if ui.button(tr("Refresh")).clicked() {
                                self.port_info = crate::laser::list_port_info();
                                self.ports = self.port_info.iter().map(|p| p.name.clone()).collect();
                            }
                        });
                    } else {
                        ui.horizontal(|ui| {
                            ui.label(tr("Host"));
                            ui.add(egui::TextEdit::singleline(&mut d.connection.host).hint_text("raspberrypi.local").desired_width(200.0));
                            ui.label(tr("TCP port"));
                            ui.add(egui::DragValue::new(&mut d.connection.tcp_port).range(1..=65535));
                        });
                    }
                    let label = if reading { tr("Reading…") } else { tr("Read from device") };
                    if ui.add_enabled(!reading && d.has_target() && d.controller.settings_request().is_some(), egui::Button::new(label)).clicked() {
                        read = true;
                    }
                    if was_read {
                        ui.label(RichText::new(tr("The settings were read from the controller.")).color(Color32::from_rgb(0x2e, 0xa0, 0x4f)));
                    }
                    ui.label(RichText::new(self.status.as_str()).small().color(theme::text_dim()));
                }
                2 => {
                    ui.label(RichText::new(tr("Name and units")).strong());
                    egui::Grid::new("wiz_name").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                        ui.label(tr("Name"));
                        ui.add(egui::TextEdit::singleline(&mut d.name).hint_text(tr("e.g. My diode laser")));
                        ui.end_row();
                        ui.label(tr("Units"));
                        egui::ComboBox::from_id_salt("wiz_units").selected_text(tr(d.units.label())).show_ui(ui, |ui| {
                            for u in Units::ALL {
                                ui.selectable_value(&mut d.units, u, tr(u.label()));
                            }
                        });
                        ui.end_row();
                    });
                }
                _ => {
                    ui.label(RichText::new(tr("Summary")).strong());
                    egui::Grid::new("wiz_sum").num_columns(2).spacing([12.0, 4.0]).show(ui, |ui| {
                        for (k, v) in [
                            (tr("Name"), d.name.clone()),
                            (tr("Controller"), d.controller.label().to_string()),
                            (tr("Laser type"), d.heads[0].laser.label().to_string()),
                            (tr("Work area"), format!("{} × {}", fmt_len(d.units, d.area.bed_w, 0), fmt_len(d.units, d.area.bed_h, 0))),
                            (tr("Machine zero (0,0)"), tr(d.axes.origin.label()).to_string()),
                            (tr("S-value max ($30)"), format!("{}", d.heads[0].s_max)),
                            (tr("Connection"), if d.has_target() { d.target_label() } else { tr("not set").to_string() }),
                        ] {
                            ui.label(k);
                            ui.label(RichText::new(v).strong());
                            ui.end_row();
                        }
                    });
                    ui.add_space(6.0);
                    ui.label(RichText::new(tr("Worth checking")).strong());
                    for w in Self::wizard_warnings(d, was_read) {
                        ui.label(RichText::new(format!("• {w}")).color(Color32::from_rgb(0xe8, 0x8a, 0x1f)));
                    }
                }
            }
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if step > 0 && ui.button(tr("Back")).clicked() {
                    next = step - 1;
                }
                if step + 1 < STEPS {
                    let go = egui::Button::new(RichText::new(tr("Next")).color(Color32::WHITE).strong()).fill(theme::accent());
                    if ui.add_enabled(step != 2 || !d.name.trim().is_empty(), go).clicked() {
                        next = step + 1;
                    }
                    if step == 1 && ui.button(tr("Skip")).clicked() {
                        next = step + 1;
                    }
                } else {
                    let go = egui::Button::new(RichText::new(tr("Finish")).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0x2e, 0xa0, 0x4f));
                    if ui.add_enabled(!d.name.trim().is_empty(), go).clicked() {
                        finish = true;
                    }
                }
                if ui.button(tr("Set up by hand…")).on_hover_text(tr("Skip these steps and open the device settings")).clicked() {
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
            // Closing the setup leaves the start screen without a profile: the plain device window comes back.
            self.wizard = None;
            self.wizard_skipped = true;
            self.cfg = Some(cfg);
            return;
        }
        self.wizard = Some(next);
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
}
