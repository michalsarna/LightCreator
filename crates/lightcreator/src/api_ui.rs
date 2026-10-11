//! The program's side of the web server and API: the option in "Program options", what is published for the server
//! (status, project, cameras) and the commands that come back.
use crate::api_server::{Command, Server};
use crate::app::App;
use crate::i18n::{tr, trf};
use eframe::egui::{self, Color32, RichText};
use serde_json::json;
use std::time::{Duration, Instant};

pub const DEFAULT_PORT: u16 = 8765;

/// What is running now: the server and the settings it was started with.
pub struct ApiState {
    pub server: Option<Server>,
    started_with: (u16, bool),
    pub error: String,
    last_publish: Option<Instant>,
}

impl Default for ApiState {
    fn default() -> Self {
        ApiState { server: None, started_with: (0, false), error: String::new(), last_publish: None }
    }
}

impl App {
    /// Start, restart or stop the server to match the options, publish the state and carry out the commands.
    pub fn api_tick(&mut self, ctx: &egui::Context) {
        let want = (self.api_port, self.api_lan);
        if !self.api_on {
            if self.api.server.take().is_some() {
                self.api.error.clear();
            }
            return;
        }
        if self.api.server.is_none() || self.api.started_with != want {
            self.api.server = None;
            match Server::start(want.0, want.1) {
                Ok(s) => {
                    self.api.server = Some(s);
                    self.api.error.clear();
                }
                Err(e) => self.api.error = e,
            }
            self.api.started_with = want;
            self.api.last_publish = None;
        }
        let Some(server) = &self.api.server else { return };
        let shared = server.shared.clone();
        // A changed access phrase counts from now on, without restarting the server.
        if let Ok(mut a) = shared.access.lock() {
            if *a != self.api_access {
                *a = self.api_access.clone();
            }
        }
        // The commands first, so that STOP does not wait for the next publication.
        let cmds: Vec<Command> = shared.commands.lock().map(|mut q| q.drain(..).collect()).unwrap_or_default();
        for c in cmds {
            self.api_command(c);
        }
        if self.api.last_publish.is_some_and(|t| t.elapsed() < Duration::from_millis(250)) {
            ctx.request_repaint_after(Duration::from_millis(250));
            return;
        }
        self.api.last_publish = Some(Instant::now());
        let dev = &self.doc.device;
        let elapsed = self.job_started.map_or(self.job_last_secs, |t| t.elapsed().as_secs_f64());
        let status = json!({
            "app": "LightCreator",
            "version": crate::app::APP_VERSION,
            "theme": self.scheme.id(),
            "device": {
                "name": dev.name,
                "controller": format!("{:?}", dev.controller),
                "connected": self.connected,
                "state": self.machine.0,
                "x": self.machine.1,
                "y": self.machine.2,
            },
            "job": {
                "running": self.job_running(),
                "done": self.progress.0,
                "total": self.progress.1,
                "elapsed_s": elapsed,
                "estimate_s": self.job_est,
            },
            "project": {
                "name": self.path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
                "revision": self.revision,
                "description": self.doc.description,
            },
        });
        let cams = json!({
            "active": dev.active_camera,
            "cameras": dev.cameras.iter().enumerate().map(|(i, c)| json!({
                "index": i,
                "name": c.name,
                "enabled": c.enabled,
                "active": i == dev.active_camera,
                // The address of a camera that has one; a camera of this computer only has the snapshot.
                "url": match &c.kind { lc_core::CameraKind::Url(u) if c.enabled && c.is_valid() => json!(u.trim()), _ => json!(null) },
                "snapshot": "/api/camera.jpg",
                // The turn set for this camera in the program, in degrees clockwise ...
                "rotation": (c.rotation % 4) as u32 * 90,
                // ... and how much of it is already in the snapshot (the program turns its picture; a camera's own address is raw).
                "rotated": match &c.kind { lc_core::CameraKind::Url(u) if c.enabled && c.is_valid() && !u.trim().is_empty() => 0, _ => (c.rotation % 4) as u32 * 90 },
            })).collect::<Vec<_>>(),
        });
        if let Ok(mut s) = shared.status.lock() {
            *s = status;
        }
        if let Ok(mut c) = shared.cameras.lock() {
            *c = cams;
        }
        if let Ok(mut t) = shared.themes.lock() {
            if t.is_null() {
                *t = crate::theme::web_themes();
            }
        }
        let rev = self.revision;
        let stale = shared.svg.lock().map(|s| s.0 != rev || s.1.is_empty()).unwrap_or(false);
        if stale {
            let svg = std::sync::Arc::new(lc_core::svg::export(&self.doc));
            if let Ok(mut s) = shared.svg.lock() {
                *s = (rev, svg);
            }
        }
        ctx.request_repaint_after(Duration::from_millis(250));
    }

    /// Hand the newest camera picture to the server (only while it runs).
    pub fn api_publish_frame(&self, w: u32, h: u32, rgb: &[u8]) {
        if let Some(s) = &self.api.server {
            if let Ok(mut f) = s.shared.frame.lock() {
                *f = Some((w, h, std::sync::Arc::new(rgb.to_vec())));
            }
        }
    }

    /// One command from the API, with the same rules as the buttons it stands for.
    pub fn api_command(&mut self, c: Command) {
        let ctrl = self.doc.device.controller;
        match c {
            Command::Stop => self.abort_job(),
            Command::Home => {
                if self.can_home() {
                    self.run_action(ctrl.home());
                }
            }
            Command::CancelJog => {
                if self.connected {
                    self.run_action(ctrl.cancel_jog());
                }
            }
            Command::Jog(d) => {
                if self.connected && !self.job_running() {
                    let (dx, dy) = d.delta();
                    let s = self.doc.device.jog_step;
                    if let Some(a) = self.jog_action(dx * s, dy * s) {
                        self.run_action(a);
                    }
                }
            }
            Command::SelectCamera(i) => {
                if i < self.doc.device.cameras.len() {
                    self.doc.device.activate_camera(i);
                    self.sync_profile();
                }
            }
        }
    }

    /// The part of "Program options" about the web server and API.
    pub fn api_options_ui(&mut self, ui: &mut egui::Ui) {
        ui.label(tr("Web server and API"));
        ui.vertical(|ui| {
            ui.checkbox(&mut self.api_on, tr("Switch on the web server and the API")).on_hover_text(tr("A web page with the state of the machine, the project and the camera, and an API for other programs. Every request needs the access phrase; there is no encryption."));
            ui.add_enabled_ui(self.api_on, |ui| {
                ui.horizontal(|ui| {
                    ui.label(tr("Port"));
                    ui.add(egui::DragValue::new(&mut self.api_port).range(1024..=65535));
                });
                ui.checkbox(&mut self.api_lan, tr("Reachable from other computers on the network"));
            });
            ui.add_enabled_ui(self.api_on, |ui| {
                if self.api_access_edit.is_empty() && !self.api_access_typing {
                    self.api_access_edit = self.api_access.clone();
                }
                let ok = crate::api_server::access_ok(&self.api_access_edit);
                ui.horizontal(|ui| {
                    ui.label(tr("Access phrase"));
                    let r = ui.add(egui::TextEdit::singleline(&mut self.api_access_edit).password(true).desired_width(140.0).char_limit(crate::api_server::MAX_ACCESS));
                    if r.changed() {
                        self.api_access_typing = true;
                    }
                    let changed = self.api_access_edit != self.api_access;
                    if crate::window_ui::save_button_enabled(ui, ok && changed, &tr("Save")).on_hover_text(tr("Use this access phrase from now on")).clicked() {
                        self.api_access = self.api_access_edit.clone();
                        self.api_access_typing = false;
                    }
                });
                if !ok {
                    ui.label(RichText::new(tr("The access phrase cannot be empty or have control characters.")).color(Color32::from_rgb(0xc0, 0x30, 0x30)));
                }
            });
            if self.api_on && self.api_lan {
                ui.label(RichText::new(tr("Anyone on the network who knows the access phrase can see the machine and stop or move it. The phrase is sent without encryption: use only on a network you trust.")).color(Color32::from_rgb(0xc0, 0x80, 0x20)));
            }
            if let Some(s) = &self.api.server {
                ui.label(RichText::new(trf("Listening on {}", &[&format!("http://{}:{}", if self.api_lan { "<this computer>".to_string() } else { "localhost".to_string() }, s.addr.port())])).color(Color32::from_rgb(0x2e, 0xa0, 0x4f)));
            } else if self.api_on && !self.api.error.is_empty() {
                ui.label(RichText::new(trf("The server could not start: {}", &[&self.api.error])).color(Color32::from_rgb(0xc0, 0x30, 0x30)));
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api_server::Jog;

    fn app() -> (App, egui::Context) {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.screen = crate::app::Screen::Editor;
        a.api_port = 0;
        (a, ctx)
    }

    #[test]
    fn the_server_follows_the_option_and_publishes_the_state() {
        let (mut a, ctx) = app();
        a.api_tick(&ctx);
        assert!(a.api.server.is_none(), "off by default");
        a.api_on = true;
        a.api_tick(&ctx);
        let srv = a.api.server.as_ref().expect("started");
        assert!(srv.addr.ip().is_loopback());
        let shared = srv.shared.clone();
        a.doc.device.name = "My laser".into();
        a.api.last_publish = None;
        a.api_tick(&ctx);
        let st = shared.status.lock().unwrap().clone();
        assert_eq!(st["device"]["name"], "My laser");
        assert_eq!(st["app"], "LightCreator");
        assert_eq!(st["theme"], a.scheme.id(), "the page starts with the program's scheme");
        assert_eq!(shared.themes.lock().unwrap().as_array().map(|l| l.len()), Some(crate::theme::Scheme::ALL.len()));
        assert!(shared.svg.lock().unwrap().1.contains("<svg"));
        // Other computers: the server is started again on every interface.
        a.api_lan = true;
        a.api_tick(&ctx);
        assert!(a.api.server.as_ref().unwrap().addr.ip().is_unspecified());
        // Switched off, the server goes.
        a.api_on = false;
        a.api_tick(&ctx);
        assert!(a.api.server.is_none());
    }

    #[test]
    fn the_phrase_is_kept_as_base64_and_read_back() {
        let (mut a, _ctx) = app();
        a.api_access = "my phrase ü".into();
        let pairs = a.settings_pairs();
        let kept = pairs.iter().find(|(k, _)| *k == "api_access").map(|(_, v)| v.clone()).unwrap();
        assert!(!kept.contains("phrase"), "not in plain text");
        assert_eq!(crate::api_server::access_from_setting(&kept).as_deref(), Some("my phrase ü"));
        // The phrase a fresh installation starts with is kept the same way.
        let (b, _) = app();
        let fresh = b.settings_pairs().into_iter().find(|(k, _)| *k == "api_access").map(|(_, v)| v).unwrap();
        assert_eq!(fresh, "TGlnaHRDcmVhdG9y");
    }

    #[test]
    fn a_new_access_phrase_reaches_the_running_server() {
        let (mut a, ctx) = app();
        a.api_on = true;
        a.api_tick(&ctx);
        let shared = a.api.server.as_ref().unwrap().shared.clone();
        assert_eq!(*shared.access.lock().unwrap(), crate::api_server::DEFAULT_ACCESS, "the default");
        a.api_access = "other".into();
        a.api.last_publish = None;
        a.api_tick(&ctx);
        assert_eq!(*shared.access.lock().unwrap(), "other");
    }

    #[test]
    fn the_cameras_carry_their_turn_and_how_much_of_it_is_in_the_snapshot() {
        let (mut a, ctx) = app();
        a.api_on = true;
        let mut own = lc_core::CameraSource::url("Own", "http://a/x", 1);
        own.enabled = true;
        let mut local = lc_core::CameraSource::url("Local", "", 3);
        local.kind = lc_core::CameraKind::Local(0);
        a.doc.device.cameras = vec![own, local];
        a.api_tick(&ctx);
        let shared = a.api.server.as_ref().unwrap().shared.clone();
        let c = shared.cameras.lock().unwrap().clone();
        assert_eq!((c["cameras"][0]["rotation"].as_u64(), c["cameras"][0]["rotated"].as_u64()), (Some(90), Some(0)), "the camera's own address is raw");
        assert_eq!((c["cameras"][1]["rotation"].as_u64(), c["cameras"][1]["rotated"].as_u64()), (Some(270), Some(270)), "the program turns its own picture");
    }

    #[test]
    fn commands_follow_the_rules_of_the_buttons() {
        let (mut a, _ctx) = app();
        a.doc.device.cameras = vec![lc_core::CameraSource::url("A", "http://a/x", 0), lc_core::CameraSource::url("B", "http://b/y", 0)];
        a.api_command(Command::SelectCamera(1));
        assert_eq!(a.doc.device.active_camera, 1);
        a.api_command(Command::SelectCamera(7));
        assert_eq!(a.doc.device.active_camera, 1, "no such camera");
        // Without a connection nothing moves: no panic, no state change.
        a.connected = false;
        a.api_command(Command::Jog(Jog::Up));
        a.api_command(Command::Home);
        a.api_command(Command::CancelJog);
        a.api_command(Command::Stop);
    }

    #[test]
    fn the_option_is_saved_with_the_settings() {
        let (mut a, _ctx) = app();
        a.api_on = true;
        a.api_port = 9000;
        a.api_lan = true;
        a.api_access = "my phrase".into();
        let pairs = a.settings_pairs();
        for (k, v) in [("api_on", "1"), ("api_port", "9000"), ("api_lan", "1"), ("api_access", "bXkgcGhyYXNl")] {
            assert!(pairs.iter().any(|(a, b)| *a == k && b == v), "{k}");
        }
    }
}
