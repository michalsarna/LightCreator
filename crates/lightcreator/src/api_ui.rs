//! LightCreator's side of the web server and API of baseCreator (`base_ui::web_api`): what it publishes (the laser, the job, the
//! project, the cameras) and the commands that come back.
use crate::app::App;
use base_ui::web_api::{Command, WebApi};
use eframe::egui;
use serde_json::json;

impl App {
    /// Start, restart or stop the server to match the options, carry out the commands and publish the state.
    pub fn api_tick(&mut self, ctx: &egui::Context) {
        let (cmds, shared) = self.web.tick(ctx);
        // The commands first, so that STOP does not wait for the next publication.
        for c in cmds {
            self.api_command(c);
        }
        let Some(shared) = shared else { return };
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
        let cams = base_ui::web_api::cameras_json(&dev.cameras, dev.active_camera);
        WebApi::publish(&shared, status, cams, self.revision, &|| lc_core::svg::export(&self.doc));
    }

    /// Hand the newest camera picture to the server (only while it runs).
    pub fn api_publish_frame(&self, w: u32, h: u32, rgb: &[u8]) {
        self.web.publish_frame(w, h, rgb);
    }

    /// One command from the API, with the same rules as the buttons it stands for.
    pub fn api_command(&mut self, c: Command) {
        let ctrl = self.doc.device.link_controller();
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
        self.web.options_ui(ui, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base_ui::web_api::Jog;

    fn app() -> (App, egui::Context) {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.screen = crate::app::Screen::Editor;
        a.web.port = 0;
        (a, ctx)
    }

    #[test]
    fn the_server_follows_the_option_and_publishes_the_state() {
        let (mut a, ctx) = app();
        a.api_tick(&ctx);
        assert!(a.web.server.is_none(), "off by default");
        a.web.on = true;
        a.api_tick(&ctx);
        let srv = a.web.server.as_ref().expect("started");
        assert!(srv.addr.ip().is_loopback());
        let shared = srv.shared.clone();
        a.doc.device.name = "My laser".into();
        a.web.publish_soon();
        a.api_tick(&ctx);
        let st = shared.status.lock().unwrap().clone();
        assert_eq!(st["device"]["name"], "My laser");
        assert_eq!(st["app"], "LightCreator");
        assert_eq!(st["theme"], a.scheme.id(), "the page starts with the program's scheme");
        assert_eq!(shared.themes.lock().unwrap().as_array().map(|l| l.len()), Some(crate::theme::Scheme::available().len()));
        assert!(shared.svg.lock().unwrap().1.contains("<svg"));
        // Other computers: the server is started again on every interface.
        a.web.lan = true;
        a.api_tick(&ctx);
        assert!(a.web.server.as_ref().unwrap().addr.ip().is_unspecified());
        // Switched off, the server goes.
        a.web.on = false;
        a.api_tick(&ctx);
        assert!(a.web.server.is_none());
    }

    #[test]
    fn the_phrase_is_kept_as_base64_and_read_back() {
        let (mut a, _ctx) = app();
        a.web.access = "my phrase ü".into();
        let pairs = a.settings_pairs();
        let kept = pairs.iter().find(|(k, _)| *k == "api_access").map(|(_, v)| v.clone()).unwrap();
        assert!(!kept.contains("phrase"), "not in plain text");
        assert_eq!(base_ui::web_api::server::access_from_setting(&kept).as_deref(), Some("my phrase ü"));
        // The phrase a fresh installation starts with is kept the same way.
        let (b, _) = app();
        let fresh = b.settings_pairs().into_iter().find(|(k, _)| *k == "api_access").map(|(_, v)| v).unwrap();
        assert_eq!(fresh, "TGlnaHRDcmVhdG9y");
    }

    #[test]
    fn a_new_access_phrase_reaches_the_running_server() {
        let (mut a, ctx) = app();
        a.web.on = true;
        a.api_tick(&ctx);
        let shared = a.web.server.as_ref().unwrap().shared.clone();
        assert_eq!(*shared.access.lock().unwrap(), "LightCreator", "the default");
        a.web.access = "other".into();
        a.web.publish_soon();
        a.api_tick(&ctx);
        assert_eq!(*shared.access.lock().unwrap(), "other");
    }

    #[test]
    fn the_cameras_carry_their_turn_and_how_much_of_it_is_in_the_snapshot() {
        let (mut a, ctx) = app();
        a.web.on = true;
        let mut own = lc_core::CameraSource::url("Own", "http://a/x", 1);
        own.enabled = true;
        let mut local = lc_core::CameraSource::url("Local", "", 3);
        local.kind = lc_core::CameraKind::Local(0);
        a.doc.device.cameras = vec![own, local];
        a.api_tick(&ctx);
        let shared = a.web.server.as_ref().unwrap().shared.clone();
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
        a.web.on = true;
        a.web.port = 9000;
        a.web.lan = true;
        a.web.access = "my phrase".into();
        let pairs = a.settings_pairs();
        for (k, v) in [("api_on", "1"), ("api_port", "9000"), ("api_lan", "1"), ("api_access", "bXkgcGhyYXNl")] {
            assert!(pairs.iter().any(|(a, b)| *a == k && b == v), "{k}");
        }
    }
}
