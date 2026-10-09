//! "Share over network (ser2net)": set up ser2net on this (Linux) computer so that the laser on its USB port
//! can be reached remotely, for example from LightCreator on another machine ("Network (TCP)" connection).
use crate::app::App;
use crate::i18n::{tr, trf};
use crate::laser;
use crate::theme;
use eframe::egui::{self, Color32, RichText};
use lc_core::ser2net::{config, shell_safe, version_from_output, Settings, Version};
use std::sync::mpsc::{channel, Receiver};

pub struct Ser2NetDlg {
    pub version: Version,
    pub serial_port: String,
    pub baud: u32,
    pub tcp_port: u16,
    pub local_only: bool,
    pub kick: bool,
    /// What `ser2net -v` printed, if the program exists.
    pub installed: Option<String>,
    /// Addresses of this computer, for the "connect to" hint.
    pub addresses: Vec<String>,
    pub confirm: bool,
    pub message: String,
    pub busy: Option<Receiver<Result<String, String>>>,
}

impl Ser2NetDlg {
    fn settings(&self) -> Settings {
        Settings { serial_port: self.serial_port.clone(), baud: self.baud, tcp_port: self.tcp_port, bind: self.local_only.then(|| "127.0.0.1".to_string()), kick_old_user: self.kick }
    }
    pub fn text(&self) -> String {
        config(self.version, &self.settings())
    }
}

fn run(cmd: &str, args: &[&str]) -> Option<String> {
    let out = std::process::Command::new(cmd).args(args).output().ok()?;
    let mut s = String::from_utf8_lossy(&out.stdout).to_string();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    Some(s.trim().to_string())
}

/// Write the file as administrator (pkexec) and restart the service. Keeps the first backup of an existing file.
fn install(text: &str, version: Version) -> Result<String, String> {
    let tmp = std::env::temp_dir().join("lightcreator-ser2net.conf");
    std::fs::write(&tmp, text).map_err(|e| e.to_string())?;
    let tmp_s = tmp.to_string_lossy().to_string();
    if !shell_safe(&tmp_s) {
        return Err("unsafe temporary path".into());
    }
    let t = version.file();
    let script = format!("if [ -f {t} ]; then cp -n {t} {t}.lightcreator.bak; fi; cp '{tmp_s}' {t} && (systemctl restart ser2net || systemctl restart ser2net.service)");
    let out = std::process::Command::new("pkexec").arg("sh").arg("-c").arg(script).output().map_err(|e| format!("pkexec: {e}"))?;
    if out.status.success() {
        Ok(t.to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

impl App {
    /// Close the sharing dialog (used by tests and screenshots).
    #[cfg(test)]
    pub fn show_ser2net_off(&mut self) {
        self.ser2net = None;
    }

    pub fn open_ser2net(&mut self) {
        let installed = run("ser2net", &["-v"]);
        let version = installed.as_deref().and_then(version_from_output).unwrap_or(Version::V4);
        let addresses: Vec<String> = if cfg!(target_os = "linux") { run("hostname", &["-I"]).unwrap_or_default().split_whitespace().map(|s| s.to_string()).collect() } else { vec![] };
        self.ports = laser::list_ports();
        let dev = &self.doc.device;
        self.ser2net = Some(Ser2NetDlg {
            version,
            serial_port: if dev.port.is_empty() { self.ports.first().cloned().unwrap_or_else(|| "/dev/ttyUSB0".into()) } else { dev.port.clone() },
            baud: dev.baud,
            tcp_port: dev.tcp_port,
            local_only: false,
            kick: true,
            installed,
            addresses,
            confirm: false,
            message: String::new(),
            busy: None,
        });
    }

    pub fn ser2net_window(&mut self, ctx: &egui::Context) {
        let Some(mut d) = self.ser2net.take() else { return };
        // Result of a running install.
        if let Some(rx) = &d.busy {
            if let Ok(r) = rx.try_recv() {
                d.message = match r {
                    Ok(file) => trf("Installed {} and restarted ser2net.", &[&file]),
                    Err(e) => trf("Could not install the configuration: {}", &[&e]),
                };
                d.busy = None;
            } else {
                ctx.request_repaint_after(std::time::Duration::from_millis(200));
            }
        }
        let mut open = true;
        let text = d.text();
        egui::Window::new(tr("Share over network (ser2net)")).open(&mut open).collapsible(false).resizable(false).show(ctx, |ui| {
            match &d.installed {
                Some(v) => ui.label(RichText::new(trf("ser2net found: {}", &[v])).color(Color32::from_rgb(0x2e, 0xa0, 0x4f))),
                None => ui.label(RichText::new(tr("ser2net was not found. Install it first, for example: sudo apt install ser2net")).color(Color32::from_rgb(0xc0, 0x30, 0x30))),
            };
            ui.label(RichText::new(tr("This shares the laser's serial port on this computer over the network, so LightCreator elsewhere can connect with the \"Network (TCP)\" connection type.")).color(theme::text_dim()));
            ui.add_space(4.0);
            egui::Grid::new("s2n").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                ui.label(tr("Port"));
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt("s2n_port").width(170.0).selected_text(d.serial_port.clone()).show_ui(ui, |ui| {
                        for p in &self.ports {
                            ui.selectable_value(&mut d.serial_port, p.clone(), p);
                        }
                    });
                    ui.add(egui::TextEdit::singleline(&mut d.serial_port).desired_width(140.0));
                    if ui.button(tr("Refresh")).clicked() {
                        self.ports = laser::list_ports();
                    }
                });
                ui.end_row();
                ui.label(tr("Baud rate"));
                ui.add(egui::DragValue::new(&mut d.baud).range(1200..=1_000_000));
                ui.end_row();
                ui.label(tr("TCP port"));
                ui.add(egui::DragValue::new(&mut d.tcp_port).range(1..=65535));
                ui.end_row();
                ui.label(tr("ser2net version"));
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut d.version, Version::V4, "4.x (YAML)");
                    ui.selectable_value(&mut d.version, Version::V3, "3.x");
                });
                ui.end_row();
            });
            ui.checkbox(&mut d.local_only, tr("Listen only on this computer (use with an SSH tunnel)"));
            ui.checkbox(&mut d.kick, tr("A new client replaces the connected one"));
            ui.add_space(4.0);
            ui.label(RichText::new(trf("File: {}", &[&d.version.file()])).color(theme::text_dim()));
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_width(460.0);
                ui.label(RichText::new(text.trim_end()).monospace().size(12.0));
            });
            let addr = d.addresses.first().cloned().unwrap_or_else(|| "<this computer>".to_string());
            ui.label(RichText::new(trf("Then connect from the other computer to {}:{}", &[&addr, &d.tcp_port])).strong());
            ui.label(RichText::new(tr("Warning: anyone on the network who reaches this port can drive the laser. Use a trusted network, a firewall, or the local-only option with an SSH tunnel.")).color(Color32::from_rgb(0xc0, 0x80, 0x20)));
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                if ui.button(tr("Copy")).clicked() {
                    ctx.copy_text(text.clone());
                    d.message = tr("Copied.").to_string();
                }
                if ui.button(tr("Save as…")).clicked() {
                    if let Some(p) = rfd::FileDialog::new().set_file_name(d.version.file().rsplit('/').next().unwrap_or("ser2net.yaml")).save_file() {
                        d.message = match std::fs::write(&p, &text) {
                            Ok(_) => trf("Saved {}", &[&p.display()]),
                            Err(e) => trf("Save failed: {}", &[&e]),
                        };
                    }
                }
                if cfg!(target_os = "linux") {
                    if !d.confirm {
                        if ui.add_enabled(d.busy.is_none(), egui::Button::new(tr("Install and restart ser2net…"))).clicked() {
                            d.confirm = true;
                        }
                    } else {
                        ui.label(RichText::new(trf("This replaces {} (a backup is kept) and restarts ser2net. Administrator rights are requested.", &[&d.version.file()])).color(Color32::from_rgb(0xc0, 0x30, 0x30)));
                        if ui.button(tr("Confirm")).clicked() {
                            d.confirm = false;
                            let (tx, rx) = channel();
                            let (t, v) = (text.clone(), d.version);
                            std::thread::spawn(move || {
                                let _ = tx.send(install(&t, v));
                            });
                            d.busy = Some(rx);
                            d.message = tr("Waiting for administrator approval…").to_string();
                        }
                        if ui.button(tr("Cancel")).clicked() {
                            d.confirm = false;
                        }
                    }
                }
            });
            if !d.message.is_empty() {
                ui.label(&d.message);
            }
            if cfg!(target_os = "linux") {
                ui.label(RichText::new(trf("By hand: sudo cp <saved file> {} && sudo systemctl restart ser2net", &[&d.version.file()])).monospace().size(11.0).color(theme::text_dim()));
            }
        });
        if open {
            self.ser2net = Some(d);
        }
    }
}
