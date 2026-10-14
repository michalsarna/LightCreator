//! The **Firmware ($$)** page of the device window: the settings of the controller read from the machine, edited one
//! at a time, backed up to a text file and restored from it. GRBL only; for Marlin the report of `M503` is shown.
use lc_core::ControllerSettings;
use crate::app::App;
use crate::i18n::{tr, trf};
use crate::theme;
use eframe::egui::{self, Color32, RichText};
use lc_core::grbl_settings as gs;
use lc_core::Controller;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

/// What the page asks the application to do (it cannot do it itself while it is drawn).
#[derive(Debug, PartialEq)]
pub enum FwAct {
    Read,
    Write(u32, String),
    Export,
    Import,
}

/// A backup file waiting to be written to the controller, with what the controller has now.
pub struct FwImport {
    pub file: Vec<(u32, String)>,
}

#[derive(Default)]
pub struct FwUi {
    /// The settings as the controller reported them last.
    pub values: BTreeMap<u32, String>,
    /// The text typed for each setting, where it differs from the reported one.
    pub typed: BTreeMap<u32, String>,
    /// The firmware's own description (`$I`), or the report of `M503` for Marlin.
    pub info: Vec<String>,
    pub raw: Vec<String>,
    pub reading: bool,
    /// Read the settings again at this time (after a write, to see what the controller made of it).
    pub reread_at: Option<Instant>,
    /// What an import wrote, to be compared with the controller's answer once it has been read back.
    pub verify: Option<Vec<(u32, String)>>,
    pub report: Vec<String>,
    pub import: Option<FwImport>,
    pub reset_open: bool,
    pub reset_text: String,
}

/// The word that has to be typed before the factory settings are restored.
pub const RESET_WORD: &str = "RESET";

/// The page. `connected` says whether there is a link to read from.
pub fn page(ui: &mut egui::Ui, fw: &mut FwUi, connected: bool, controller: Controller, acts: &mut Vec<FwAct>) {
    if controller == Controller::Marlin {
        marlin_page(ui, fw, connected, acts);
        return;
    }
    egui::Frame::new().fill(Color32::from_rgba_unmultiplied(0xe8, 0x8a, 0x1f, 40)).stroke(egui::Stroke::new(1.0, Color32::from_rgb(0xe8, 0x8a, 0x1f))).inner_margin(8.0).corner_radius(4.0).show(ui, |ui| {
        ui.label(RichText::new(tr("These settings live in the controller, not in the profile. Change one at a time and write down the previous value: a wrong value can make the machine move the wrong distance or fire the laser.")).strong());
    });
    ui.add_space(4.0);
    if !connected {
        ui.label(RichText::new(tr("Connect to the machine first (Device tab).")).color(theme::text_dim()));
        return;
    }
    ui.horizontal_wrapped(|ui| {
        let read = if fw.reading { tr("Reading…") } else { tr("Read the settings ($$)") };
        if ui.add_enabled(!fw.reading, egui::Button::new(read)).clicked() {
            acts.push(FwAct::Read);
        }
        if crate::window_ui::save_button_enabled(ui, !fw.values.is_empty(), &tr("Back up to a file…")).clicked() {
            acts.push(FwAct::Export);
        }
        if crate::window_ui::import_button(ui, &tr("Restore from a file…")).clicked() {
            acts.push(FwAct::Import);
        }
        let red = egui::Button::new(RichText::new(tr("Factory settings ($RST=$)…")).color(Color32::WHITE)).fill(Color32::from_rgb(0xc0, 0x30, 0x30));
        if ui.add(red).clicked() {
            reset_open(fw);
        }
    });
    if !fw.info.is_empty() {
        ui.label(RichText::new(fw.info.iter().filter(|l| l.starts_with("[VER") || l.starts_with("[OPT")).cloned().collect::<Vec<_>>().join("  ")).small().color(theme::text_dim()));
    }
    for r in &fw.report {
        ui.label(RichText::new(r).color(Color32::from_rgb(0xe8, 0x8a, 0x1f)));
    }
    if fw.values.is_empty() {
        ui.add_space(6.0);
        ui.label(RichText::new(tr("Nothing read yet.")).color(theme::text_dim()));
        return;
    }
    ui.add_space(4.0);
    egui::Grid::new("fw_table").num_columns(5).striped(true).spacing([10.0, 4.0]).show(ui, |ui| {
        for h in ["Setting", "Value", "Unit", "Typical", ""] {
            ui.label(RichText::new(tr(h)).strong());
        }
        ui.end_row();
        for (id, value) in fw.values.clone() {
            let meta = gs::info(id);
            let name = meta.map(|m| m.name).unwrap_or("?");
            let critical = meta.is_some_and(|m| m.critical);
            let label = RichText::new(format!("${id}  {name}"));
            let r = ui.label(if critical { label.color(Color32::from_rgb(0xe8, 0x8a, 0x1f)).strong() } else { label });
            if let Some(m) = meta {
                r.on_hover_text(m.desc);
            }
            let text = fw.typed.entry(id).or_insert_with(|| value.clone());
            ui.add(egui::TextEdit::singleline(text).desired_width(90.0));
            ui.label(RichText::new(meta.map(|m| m.unit).unwrap_or("")).color(theme::text_dim()));
            ui.label(RichText::new(meta.map(|m| m.typical).unwrap_or("")).color(theme::text_dim()));
            let changed = !gs::same_value(text, &value);
            if changed {
                match gs::validate(id, text) {
                    Ok(v) => {
                        if ui.button(tr("Write")).clicked() {
                            acts.push(FwAct::Write(id, v));
                        }
                    }
                    Err(e) => {
                        ui.label(RichText::new(tr(e)).color(Color32::from_rgb(0xc0, 0x30, 0x30)));
                    }
                }
            } else {
                ui.label("");
            }
            ui.end_row();
        }
    });
}

fn marlin_page(ui: &mut egui::Ui, fw: &mut FwUi, connected: bool, acts: &mut Vec<FwAct>) {
    ui.label(RichText::new(tr("Marlin keeps its settings in its own commands (M92, M203, M204, ...): this page only shows what `M503` reports.")).color(theme::text_dim()));
    if !connected {
        ui.label(RichText::new(tr("Connect to the machine first (Device tab).")).color(theme::text_dim()));
        return;
    }
    if ui.add_enabled(!fw.reading, egui::Button::new(if fw.reading { tr("Reading…") } else { tr("Read the settings (M503)") })).clicked() {
        acts.push(FwAct::Read);
    }
    egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
        for l in &fw.raw {
            ui.label(RichText::new(l).monospace());
        }
    });
}

fn reset_open(fw: &mut FwUi) {
    fw.reset_open = true;
    fw.reset_text.clear();
}

impl App {
    /// Do what the page asked for.
    pub fn run_fw_acts(&mut self, acts: Vec<FwAct>) {
        for a in acts {
            match a {
                FwAct::Read => self.fw_read(),
                FwAct::Write(id, v) => {
                    let old = self.fw.values.get(&id).cloned().unwrap_or_default();
                    self.confirm = Some(crate::session_ui::Confirm {
                        title: tr("Write a setting").to_string(),
                        text: trf("Write ${} = {} to the controller? It is now {}. Change one setting at a time and note the old value.", &[&id, &v, &old]),
                        act: crate::session_ui::ConfirmAct::WriteSetting(id, v),
                    });
                }
                FwAct::Export => self.fw_export(),
                FwAct::Import => self.fw_import_pick(),
            }
        }
    }

    /// Ask the controller for its settings (`$$`), and for its build info.
    pub fn fw_read(&mut self) {
        let c = self.doc.device.controller;
        let Some(_) = c.settings_request() else { return };
        if !self.connected || self.read_cfg.is_some() {
            return;
        }
        let now = Instant::now();
        self.fw.reading = true;
        self.read_cfg = Some(crate::device_ui::ReadCfg { controller: c, waiting_connect: false, request_at: Some(now), sent_at: None, lines: vec![], started: now, kind: crate::device_ui::ReadKind::Firmware });
    }

    /// The lines of a finished read of the settings.
    pub fn fw_got(&mut self, lines: Vec<String>) {
        self.fw.reading = false;
        if self.doc.device.controller == Controller::Marlin {
            self.fw.raw = lines;
            return;
        }
        self.fw.values = gs::parse_report(&lines);
        self.fw.typed.clear();
        if let Some(want) = self.fw.verify.take() {
            let bad: Vec<String> = want.iter().filter(|(id, v)| self.fw.values.get(id).map_or(true, |now| !gs::same_value(now, v))).map(|(id, v)| format!("${id}: {} {}, {} {}", tr("File"), v, tr("Controller"), self.fw.values.get(id).cloned().unwrap_or_else(|| "–".into()))).collect();
            self.fw.report = if bad.is_empty() { vec![trf("All {} settings of the file are in the controller now.", &[&want.len()])] } else { std::iter::once(trf("{} of {} settings differ after writing:", &[&bad.len(), &want.len()])).chain(bad).collect() };
        }
        // The build info, for the page and for the options (`$I`).
        let now = Instant::now();
        self.read_cfg = Some(crate::device_ui::ReadCfg { controller: Controller::Grbl, waiting_connect: false, request_at: Some(now), sent_at: None, lines: vec![], started: now, kind: crate::device_ui::ReadKind::Info });
    }

    /// Read the settings again a moment after something was written.
    pub fn fw_tick(&mut self, ctx: &egui::Context) {
        if let Some(at) = self.fw.reread_at {
            if Instant::now() >= at && self.read_cfg.is_none() {
                self.fw.reread_at = None;
                self.fw_read();
            } else {
                ctx.request_repaint_after(Duration::from_millis(200));
            }
        }
    }

    fn fw_export(&mut self) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let date = crate::backup_ui::ymd(now);
        let Some(p) = rfd::FileDialog::new().add_filter("GRBL settings", &["txt"]).set_file_name(format!("grbl-settings-{date}.txt")).save_file() else { return };
        let text = gs::backup_text(&self.fw.values, &self.fw.info, &date);
        self.status = match std::fs::write(&p, text) {
            Ok(_) => trf("Saved the settings to {}", &[&p.display()]),
            Err(e) => trf("Export failed: {}", &[&e]),
        };
    }

    fn fw_import_pick(&mut self) {
        let Some(p) = rfd::FileDialog::new().add_filter("GRBL settings", &["txt"]).pick_file() else { return };
        let text = match std::fs::metadata(&p) {
            Ok(m) if m.len() > 40_000 => Err("the file is too large".to_string()),
            _ => std::fs::read_to_string(&p).map_err(|e| e.to_string()),
        };
        match text.and_then(|t| gs::parse_backup(&t)) {
            Ok(file) => self.fw.import = Some(FwImport { file }),
            Err(e) => self.status = trf("Cannot use the file: {}", &[&e]),
        }
    }

    /// The windows of the page: the settings of a backup file about to be written, and the factory reset.
    pub fn fw_windows(&mut self, ctx: &egui::Context) {
        self.fw_tick(ctx);
        if let Some(imp) = &self.fw.import {
            let mut open = true;
            let mut cancel = false;
            let mut write = false;
            let current = self.fw.values.clone();
            crate::window_ui::popup(ctx, "fw_import", &tr("Restore the settings from a file"), [520.0, 420.0], &mut open, |ui| {
                ui.label(tr("These settings of the file differ from the controller. Nothing is written until you press Write all."));
                egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                    egui::Grid::new("fw_imp").num_columns(3).striped(true).show(ui, |ui| {
                        ui.label(RichText::new(tr("Setting")).strong());
                        ui.label(RichText::new(tr("Controller")).strong());
                        ui.label(RichText::new(tr("File")).strong());
                        ui.end_row();
                        let mut shown = 0;
                        for (id, v) in &imp.file {
                            let now = current.get(id);
                            if now.is_some_and(|n| gs::same_value(n, v)) {
                                continue;
                            }
                            shown += 1;
                            ui.label(format!("${id}  {}", gs::info(*id).map(|m| m.name).unwrap_or("?")));
                            ui.label(now.cloned().unwrap_or_else(|| "–".into()));
                            ui.label(v);
                            ui.end_row();
                        }
                        if shown == 0 {
                            ui.label(tr("The controller already has all of them."));
                            ui.end_row();
                        }
                    });
                });
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    let b = egui::Button::new(RichText::new(tr("Write all")).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0xc0, 0x30, 0x30));
                    if ui.add(b).clicked() {
                        write = true;
                    }
                    if ui.button(tr("Cancel")).clicked() {
                        cancel = true;
                    }
                });
            });
            if write {
                if let Some(imp) = self.fw.import.take() {
                    self.confirm = Some(crate::session_ui::Confirm {
                        title: tr("Restore the settings from a file").to_string(),
                        text: trf("Write {} settings to the controller? The machine will behave as the file says.", &[&imp.file.len()]),
                        act: crate::session_ui::ConfirmAct::ImportSettings(imp.file),
                    });
                }
            } else if !open || cancel {
                self.fw.import = None;
            }
        }
        if self.fw.reset_open {
            let mut open = true;
            let mut cancel = false;
            let mut go = false;
            let mut text = std::mem::take(&mut self.fw.reset_text);
            crate::window_ui::popup(ctx, "fw_reset", &tr("Restore the factory settings"), [460.0, 220.0], &mut open, |ui| {
                ui.label(RichText::new(tr("This writes $RST=$: every setting of the controller goes back to the firmware's defaults. Step counts, travel, speeds, laser mode and homing will all change.")).color(Color32::from_rgb(0xc0, 0x30, 0x30)).strong());
                ui.label(trf("Back the settings up first. To go on, type {}:", &[&RESET_WORD]));
                ui.add(egui::TextEdit::singleline(&mut text).desired_width(160.0));
                let ready = text.trim() == RESET_WORD;
                ui.horizontal(|ui| {
                    let b = egui::Button::new(RichText::new(tr("Restore the factory settings")).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0xc0, 0x30, 0x30));
                    if ui.add_enabled(ready, b).clicked() {
                        go = true;
                    }
                    if ui.button(tr("Cancel")).clicked() {
                        cancel = true;
                    }
                });
            });
            self.fw.reset_text = text;
            if go {
                self.confirm = Some(crate::session_ui::Confirm {
                    title: tr("Restore the factory settings").to_string(),
                    text: tr("Really write $RST=$ to the controller?").to_string(),
                    act: crate::session_ui::ConfirmAct::ResetSettings,
                });
                self.fw.reset_open = false;
            } else if !open || cancel {
                self.fw.reset_open = false;
            }
        }
    }

    /// Write the settings of an import, one line each; they are read back afterwards and compared.
    pub fn fw_write_all(&mut self, settings: Vec<(u32, String)>) {
        for (id, v) in &settings {
            self.link.send(crate::laser::Cmd::Line(gs::write_line(*id, v)));
        }
        self.fw.verify = Some(settings);
        self.fw.reread_at = Some(Instant::now() + Duration::from_millis(1500));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device_ui::ReadKind;

    fn app() -> App {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.screen = crate::app::Screen::Editor;
        a.profiles = vec![lc_core::Device::default()];
        a.enter_editor(0);
        a.connected = true;
        a
    }

    fn lines(l: &[&str]) -> Vec<String> {
        l.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_read_fills_the_table_and_a_write_is_confirmed_before_it_is_sent() {
        let mut a = app();
        a.fw_read();
        assert!(a.fw.reading && a.read_cfg.as_ref().is_some_and(|r| r.kind == ReadKind::Firmware));
        a.read_cfg = None;
        a.fw_got(lines(&["$30=1000.000", "$110=3000.000", "$130=400.000", "ok"]));
        assert!(!a.fw.reading);
        assert_eq!(a.fw.values.len(), 3);
        // The build info is asked for next.
        assert!(a.read_cfg.as_ref().is_some_and(|r| r.kind == ReadKind::Info));
        // A write only opens the question; nothing is sent and nothing changes before the answer.
        a.run_fw_acts(vec![FwAct::Write(110, "5000".into())]);
        let c = a.confirm.take().expect("the question");
        assert!(c.text.contains("$110") && c.text.contains("5000") && c.text.contains("3000.000"));
        assert!(matches!(c.act, crate::session_ui::ConfirmAct::WriteSetting(110, ref v) if v == "5000"));
        assert!(a.fw.reread_at.is_none());
        // Yes: the setting is written and the settings are read again a moment later.
        a.do_confirmed(&egui::Context::default(), c.act);
        assert!(a.fw.reread_at.is_some());
    }

    #[test]
    fn a_restored_file_is_compared_with_what_the_controller_then_reports() {
        let mut a = app();
        a.fw_write_all(vec![(110, "5000".into()), (111, "5000".into()), (130, "300".into())]);
        assert!(a.fw.verify.is_some() && a.fw.reread_at.is_some());
        // The controller took two of the three.
        a.fw_got(lines(&["$110=5000.000", "$111=3000.000", "$130=300.000"]));
        assert!(a.fw.verify.is_none());
        // A heading and one line for the setting that differs (the words depend on the language).
        assert_eq!(a.fw.report.len(), 2, "{:?}", a.fw.report);
        assert!(a.fw.report[1].contains("$111") && !a.fw.report.iter().any(|l| l.contains("$110") || l.contains("$130")));
        let mut b = app();
        b.fw_write_all(vec![(110, "5000".into())]);
        b.fw_got(lines(&["$110=5000.000"]));
        assert_eq!(b.fw.report.len(), 1, "{:?}", b.fw.report);
    }

    #[test]
    fn the_factory_reset_needs_the_word_and_clears_what_was_read() {
        let mut a = app();
        a.fw.values.insert(110, "3000".into());
        a.do_confirmed(&egui::Context::default(), crate::session_ui::ConfirmAct::ResetSettings);
        assert!(a.fw.values.is_empty() && a.fw.reread_at.is_some());
        assert_eq!(RESET_WORD, "RESET");
    }

    #[test]
    fn marlin_shows_the_report_as_text_and_a_disconnected_page_reads_nothing() {
        let mut a = app();
        a.doc.device.controller = lc_core::Controller::Marlin;
        a.fw_read();
        a.read_cfg = None;
        a.fw_got(lines(&["echo:  M203 X500.00 Y500.00"]));
        assert_eq!(a.fw.raw.len(), 1);
        assert!(a.fw.values.is_empty());
        let mut off = app();
        off.connected = false;
        off.fw_read();
        assert!(off.read_cfg.is_none());
    }
}
