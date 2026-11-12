//! Machine > Diagnostics and statistics… (the window is in baseCreator, the same in all the programs), Machine > Check the program, and
//! Help > Export diagnostics (zip)…: one zip file to attach to a bug report. It holds the version and the system, the settings (without
//! the access key of the web server and the list of recent files), the device, the settings of the controller, the program, the project
//! and the console of the machine, and nothing else.
use crate::app::{App, APP_VERSION};
use crate::i18n::{tr, trf};
use crate::laser::Cmd;
use lc_core::Controller;
use std::io::Write;
use std::path::Path;

/// Settings that are left out: a key and paths of files.
const LEFT_OUT: [&str; 2] = ["api_access", "recent_files"];

/// `user:password@` in an address is hidden: `http://user:secret@host/` → `http://***@host/`.
pub fn hide_passwords(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find("://") {
        out.push_str(&rest[..i + 3]);
        rest = &rest[i + 3..];
        let end = rest.find(|c: char| c == '/' || c == '"' || c == '\'' || c.is_whitespace()).unwrap_or(rest.len());
        match rest[..end].rfind('@') {
            Some(at) => {
                out.push_str("***");
                out.push_str(&rest[at..end]);
            }
            None => out.push_str(&rest[..end]),
        }
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

/// A run of the program through the check mode of GRBL.
pub struct CheckRun {
    /// The job that starts when the run finds nothing.
    pub then: Option<Vec<String>>,
    /// How many lines the controller refused, and the line of the first.
    pub issues: usize,
    pub first: usize,
}

impl App {
    /// Machine > Firmware settings ($$)…: the page of the settings window.
    pub fn open_firmware_page(&mut self) {
        self.open_cfg_edit(self.active);
        if let Some(c) = &mut self.cfg {
            c.page = crate::device_ui::CfgPage::Firmware;
        }
    }

    /// Machine > Check the program: GRBL goes through it without moving, and says which lines it does not take.
    pub fn check_program(&mut self) {
        if !self.connected {
            self.status = tr("Connect to the machine first (Device tab)").to_string();
            return;
        }
        if self.doc.device.controller != Controller::Grbl || self.job_running() {
            return;
        }
        let Some(lines) = self.program_lines() else { return };
        self.start_check(lines, None);
        self.status = tr("Checking the program…").to_string();
    }

    pub fn start_check(&mut self, lines: Vec<String>, then: Option<Vec<String>>) {
        self.layer_watch.stop();
        self.trips.stop();
        self.check_run = Some(CheckRun { then, issues: 0, first: 0 });
        self.link.send(Cmd::Line("$C".into()));
        self.link.send(Cmd::Job(lines));
    }

    /// The last line of a check run was answered: GRBL leaves the check mode, and the job waiting for it goes on when all is well.
    pub fn finish_check(&mut self) {
        let Some(run) = self.check_run.take() else { return };
        self.link.send(Cmd::Line("$C".into()));
        if run.issues > 0 {
            self.status = trf("The check found {} problems, the first in line {}. The job was not started.", &[&run.issues, &run.first]);
            return;
        }
        match run.then {
            Some(lines) => {
                self.job_last_secs = 0.0;
                self.send_job_lines(lines, true);
                self.status = tr("The check found nothing. The job is running.").to_string();
            }
            None => self.status = tr("The check found nothing wrong.").to_string(),
        }
    }

    /// The controller reported an alarm or an error: the statistics of the machine, or the result of a check run.
    pub fn count_problem(&mut self, alarm: bool, code: u32) {
        let line = self.progress.0 + 1;
        if let Some(run) = &mut self.check_run {
            if !alarm {
                if run.issues == 0 {
                    run.first = line;
                }
                run.issues += 1;
                return;
            }
        }
        if alarm {
            self.doc.device.stats.alarm(code);
        } else {
            self.doc.device.stats.error(code);
        }
        self.sync_stats();
    }

    /// The statistics are kept with the profile.
    pub fn sync_stats(&mut self) {
        if let Some(p) = self.profiles.get_mut(self.active) {
            p.stats = self.doc.device.stats.clone();
        }
    }

    /// The window of diagnostics and statistics.
    pub fn diagnostics_window(&mut self, ctx: &eframe::egui::Context) {
        use base_ui::tools_ui::{diagnostics_view, DiagData, MaintRow};
        if !self.diag.open {
            return;
        }
        let d = &self.doc.device;
        let info = lc_core::firmware::parse_info(&self.fw.info);
        let stats = d.stats.clone();
        let maintenance = d.maintenance.counters.iter().map(|c| {
            let hours = c.secs / 3600.0;
            MaintRow { name: c.name.clone(), hours_left: c.threshold_hours.map(|t| t - hours), hours }
        }).collect();
        let data = DiagData {
            connected: self.connected,
            pins: (d.controller == Controller::Grbl).then_some(self.pos.pins.as_str()),
            controller: if info.is_known() { Some(info.label()) } else if d.controller == Controller::Grbl { None } else { Some(format!("{:?}", d.controller)) },
            stats: &stats,
            maintenance,
            no_maintenance: "No reminders. Add them in the device settings (Maintenance).",
        };
        let out = diagnostics_view(ctx, &mut self.diag, &data);
        if out.clear_stats {
            self.doc.device.stats = Default::default();
            self.sync_stats();
        }
        if let Some(i) = out.done {
            let name = self.doc.device.maintenance.counters.get(i).map(|c| c.name.clone()).unwrap_or_default();
            self.confirm = Some(crate::session_ui::Confirm {
                title: tr("Reset the counter").to_string(),
                text: trf("Start the counter \"{}\" again from zero?", &[&name]),
                act: crate::session_ui::ConfirmAct::ResetCounter(i),
            });
        }
        if out.export {
            self.export_diagnostics();
        }
    }

    /// Ask where to put the file and write it.
    pub fn export_diagnostics(&mut self) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let name = format!("lightcreator-diagnostics-{}.zip", crate::backup_ui::ymd(now));
        let Some(dest) = rfd::FileDialog::new().add_filter("Zip", &["zip"]).set_file_name(&name).save_file() else { return };
        match self.write_diagnostics(&dest) {
            Ok(()) => {
                let file = dest.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                self.status = trf("Diagnostics saved: {}", &[&dest.display()]);
                self.toast(trf("Diagnostics saved: {}", &[&file]));
            }
            Err(e) => self.status = trf("Could not save the diagnostics: {}", &[&e]),
        }
    }

    /// The files of the zip, as (name, content).
    pub fn diagnostics_files(&mut self) -> Vec<(&'static str, String)> {
        let gcode = {
            let (doc, back) = self.job_doc();
            lc_core::gcode::generate_with(&doc, back, &self.job_options()).gcode
        };
        let d = &self.doc.device;
        let mut info = String::new();
        let mut line = |k: &str, v: String| info.push_str(&format!("{k}: {v}\n"));
        line("Program", format!("LightCreator {APP_VERSION}"));
        line("System", format!("{} {}", std::env::consts::OS, std::env::consts::ARCH));
        line("Language", self.lang.code().to_owned());
        line("Theme", self.scheme.id().to_owned());
        line("Device", d.name.clone());
        line("Controller", format!("{:?}", d.controller));
        line("Work area", format!("{} x {} mm", d.work_size().0, d.work_size().1));
        line("Connected", self.connected.to_string());
        line("Machine", self.machine.0.to_string());
        line("Project", self.path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "(not saved)".into()));
        line("Layers", self.doc.layers.len().to_string());
        line("Active inputs", if self.pos.pins.is_empty() { "none".to_string() } else { self.pos.pins.clone() });
        let st = &d.stats;
        line("Jobs", st.jobs.to_string());
        line("Run time", format!("{:.2} h", st.hours()));
        line("Alarms", format!("{} {:?}", st.alarms, st.alarm_codes));
        line("Errors", format!("{} {:?}", st.errors, st.error_codes));
        let settings: String = self.settings_pairs().into_iter().filter(|(k, _)| !LEFT_OUT.contains(k)).map(|(k, v)| format!("{k}={v}\n")).collect();
        let console: String = self
            .console
            .iter()
            .filter(|l| !l.poll)
            .map(|l| {
                let mark = match l.dir {
                    base_ui::link::Dir::Tx | base_ui::link::Dir::User => ">",
                    base_ui::link::Dir::Rx => "<",
                    base_ui::link::Dir::Info => "#",
                };
                format!("{mark} {}\n", l.text)
            })
            .collect();
        let fw: String = self.fw.values.iter().map(|(k, v)| format!("${k}={v}\n")).collect();
        vec![
            ("info.txt", info),
            ("device.json", hide_passwords(&serde_json::to_string_pretty(d).unwrap_or_default())),
            ("settings.txt", hide_passwords(&settings)),
            ("grbl-settings.txt", fw),
            ("console.txt", console),
            ("program.gcode", gcode),
            ("project.cre", self.doc.to_json()),
        ]
    }

    pub fn write_diagnostics(&mut self, dest: &Path) -> Result<(), String> {
        let file = std::fs::File::create(dest).map_err(|e| e.to_string())?;
        let mut zip = zip::ZipWriter::new(file);
        let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        for (name, text) in self.diagnostics_files() {
            zip.start_file(name, opts).map_err(|e| e.to_string())?;
            zip.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
        }
        zip.finish().map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn passwords_in_addresses_are_hidden() {
        assert_eq!(hide_passwords("\"rtsp://user:secret@192.168.1.5/live\" and http://host:80/x"), "\"rtsp://***@192.168.1.5/live\" and http://host:80/x");
        assert_eq!(hide_passwords("no address"), "no address");
    }

    #[test]
    fn the_zip_holds_the_files_and_no_access_key() {
        let ctx = eframe::egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.web.access = "TOPSECRET".into();
        a.doc.device.cameras = vec![lc_core::CameraSource::url("A", "http://admin:hunter2@10.0.0.2/video", 0)];
        let dest = std::env::temp_dir().join(format!("lc-diag-{}.zip", std::process::id()));
        a.write_diagnostics(&dest).unwrap();
        let mut z = zip::ZipArchive::new(std::fs::File::open(&dest).unwrap()).unwrap();
        let mut all = String::new();
        let mut names = vec![];
        for i in 0..z.len() {
            let mut f = z.by_index(i).unwrap();
            names.push(f.name().unwrap_or_default().to_string());
            f.read_to_string(&mut all).unwrap();
        }
        let _ = std::fs::remove_file(&dest);
        assert_eq!(names, ["info.txt", "device.json", "settings.txt", "grbl-settings.txt", "console.txt", "program.gcode", "project.cre"]);
        assert!(all.contains("LightCreator v") && !all.contains("TOPSECRET") && !all.contains("hunter2"));
    }
}
