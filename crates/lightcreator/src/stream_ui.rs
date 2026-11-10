//! The live picture from the device's camera URL. It can open in its own window (which may leave the
//! app window and the screen), sit in the side panel as a tab or be a tab of the work area.
use crate::app::{App, SideTab};
use base_ui::editor::EditorBackground;
use crate::camera::rotate_rgb;
use crate::camera_stream::{self, Msg};
use crate::i18n::{tr, trf};
use eframe::egui::{self, RichText};

// The view itself (the three ways, the picture, the window of its own) is the camera view of every program (`base_ui::camera_view`);
// here are the parts of LightCreator: the stream of its URL, the overlay, the recording and the working time.
pub use base_ui::camera_view::StreamMode;
use base_ui::camera_view::{self as cv, MAX_ZOOM};
#[cfg(test)]
use base_ui::camera_view::{MIN as CAM_MIN, STILL as CAM_STILL};

/// The window of its own of the camera view.
const CAM_WINDOW: CamWindowId = CamWindowId;

struct CamWindowId;

impl From<CamWindowId> for egui::ViewportId {
    fn from(_: CamWindowId) -> Self {
        egui::ViewportId::from_hash_of("lightcreator-camera")
    }
}

impl App {
    /// Take the way of showing the camera view from the device (when a device is entered).
    pub fn load_camera_view(&mut self) {
        let d = &self.doc.device;
        self.stream_mode = StreamMode::of(&d.camera_view, d.cameras.get(d.active_camera));
        self.stream_zoom = d.camera_view.zoom.clamp(1.0, MAX_ZOOM);
        self.cam_view_for = d.active_camera;
    }

    /// Remember the mode (with the active camera) and the zoom (with the device) when they changed.
    pub(crate) fn save_camera_view(&mut self) {
        let d = &mut self.doc.device;
        if cv::keep_mode(&mut d.camera_view, &mut d.cameras, d.active_camera, self.stream_mode, self.stream_zoom) {
            self.sync_profile();
        }
    }

    /// The Recording tab: a panel for each camera, the photos aside and the long lines (a laser has no spindle to keep running).
    pub fn recording_tab(&mut self, ui: &mut egui::Ui) {
        let mpos = self.connected.then(|| self.pos.mpos.unwrap_or([self.machine.1, self.machine.2, self.machine_z]));
        let view = self.doc.device.camera_view.clone();
        let d = &mut self.doc.device;
        let out = base_ui::recording_ui::tab(ui, &mut self.rigs, &mut d.cameras, &view, &mut d.photo_park, &mut d.split, &base_ui::recording_ui::MachineCtx { mpos, spindle: false });
        if let Some((i, m)) = out.mode {
            self.set_camera_mode(i, m);
        }
        if out.cameras || out.machine {
            self.sync_profile();
        }
    }

    /// Set how the camera view of camera `i` opens (from the view options). The one in use changes at once.
    pub fn set_camera_mode(&mut self, i: usize, mode: StreamMode) {
        let Some(c) = self.doc.device.cameras.get_mut(i) else { return };
        c.view_mode = Some(mode.index());
        if i == self.doc.device.active_camera {
            self.stream_mode = mode;
            self.show_mode(mode);
        }
        self.sync_profile();
    }

    /// A camera view in the side panel must be seen: the panel opens on its tab.
    fn show_mode(&mut self, mode: StreamMode) {
        match mode {
            StreamMode::Tab => {
                self.side_tab = SideTab::Camera;
                self.tabs.collapsed = false;
            }
            StreamMode::Workspace => self.cam_tab_shown = true,
            _ => {}
        }
    }

    /// When another camera was made the active one, its own way of opening is taken.
    fn follow_active_camera(&mut self) {
        if self.cam_view_for != self.doc.device.active_camera {
            let before = self.stream_mode;
            self.load_camera_view();
            if before != self.stream_mode {
                self.show_mode(self.stream_mode);
            }
        }
    }

    /// Stop the stream. A picture it fed into the work area is hidden with it.
    pub fn stream_closed(&mut self) {
        self.show_stream = false;
        self.stream = None;
        self.tab_seen = false;
        self.cam_tab_shown = false;
        if self.stream_overlay {
            self.stream_overlay = false;
            self.overlay_show(false);
        }
        if self.side_tab == SideTab::Camera {
            self.side_tab = SideTab::Properties;
        }
    }

    /// Keep the worker running for the current URL and turn new frames into textures.
    fn stream_poll(&mut self, ctx: &egui::Context, url: &str) {
        if self.stream.as_ref().map(|s| s.0.as_str()) != Some(url) {
            self.stream_err.clear();
            self.stream = Some((url.to_string(), camera_stream::spawn(url.to_string(), ctx.clone())));
        }
        // The rotation is a device setting: a change shows up at once.
        let rotation = self.doc.device.camera_source().map_or(0, |c| c.rotation % 4);
        if self.stream_rot != rotation {
            self.stream_rot = rotation;
            self.stream_tex = None;
        }
        let mut latest = None;
        if let Some((_, w)) = &self.stream {
            while let Ok(m) = w.rx.try_recv() {
                match m {
                    Msg::Frame(f) => {
                        self.stream_err.clear();
                        latest = Some(f);
                    }
                    Msg::Error(e) => self.stream_err = e,
                }
            }
        }
        // While the camera window is being resized the picture stands still (new frames are dropped); the stream goes on after.
        if self.cam_resizing() {
            latest = None;
        }
        if let Some(f) = latest {
            let rgb = self.cam_process(f.rgb, f.w, f.h);
            let (w, h, rgb) = rotate_rgb(&rgb, f.w, f.h, rotation);
            self.api_publish_frame(w, h, &rgb);
            self.rigs.feed(self.doc.device.active_camera, std::time::Instant::now(), w, h, rgb.clone());
            let img = egui::ColorImage::from_rgb([w as usize, h as usize], &rgb);
            match &mut self.stream_tex {
                Some(t) if t.size() == [w as usize, h as usize] => t.set(img.clone(), egui::TextureOptions::LINEAR),
                _ => self.stream_tex = Some(ctx.load_texture("camera-stream", img.clone(), egui::TextureOptions::LINEAR)),
            }
            if self.stream_overlay {
                self.set_background(ctx, img, &tr("Camera view"));
            }
        }
    }

    /// Is the separate camera window being resized (its size changed a moment ago)?
    pub fn cam_resizing(&self) -> bool {
        self.stream_mode == StreamMode::Detached && cv::resizing(&self.cam_win)
    }

    /// Note the size of the separate camera window and cut it to its picture once it was let go (`camera_view::settle`).
    fn cam_window_settle(&mut self, ctx: &egui::Context, size: [f32; 2]) {
        cv::settle(ctx, CAM_WINDOW.into(), &mut self.cam_win, &self.cam_fit, size);
    }

    /// Apply the active camera's lens correction and picture adjustments to a frame (RGB bytes of `w × h`).
    pub fn cam_process(&self, rgb: Vec<u8>, w: u32, h: u32) -> Vec<u8> {
        let Some(cam) = self.doc.device.cameras.get(self.doc.device.active_camera) else { return rgb };
        let (w, h) = (w as usize, h as usize);
        let mut out = if cam.lens.is_none() { rgb } else { lc_core::camimg::undistort(&rgb, w, h, &cam.lens) };
        lc_core::camimg::adjust(&mut out, w, h, &cam.image);
        out
    }

    /// In the view options: where the camera view of each camera opens.
    pub fn camera_mode_options(&mut self, ui: &mut egui::Ui) {
        let d = &self.doc.device;
        if let Some((i, m)) = cv::mode_options(ui, &d.camera_view, &d.cameras) {
            self.set_camera_mode(i, m);
        }
    }

    /// Is the camera view a tab of the work area (so that the bar of the tabs has one for it)?
    pub fn camera_work_tab(&self) -> bool {
        self.show_stream && self.stream_mode == StreamMode::Workspace && self.doc.device.stream_url().is_some()
    }

    /// The camera view in the work area, in place of the drawing.
    pub fn camera_work_panel(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().id_salt("camera_work_area").auto_shrink([false, false]).show(ui, |ui| {
            egui::Frame::new().inner_margin(8).show(ui, |ui| self.stream_view(ui));
        });
    }

    /// Picture and controls, shared by the three ways of showing the stream.
    pub fn stream_view(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let before = self.stream_mode;
            if let Some(m) = cv::mode_buttons(ui, self.stream_mode) {
                self.stream_mode = m;
            }
            if before != self.stream_mode {
                self.show_mode(self.stream_mode);
            }
            if before == StreamMode::Tab && self.stream_mode != StreamMode::Tab && self.side_tab == SideTab::Camera {
                self.side_tab = SideTab::Properties;
            }
            ui.separator();
            let (on, can) = self.overlay_state();
            if crate::app::window_toggle(ui, &tr("Overlay"), &tr("Show or hide the camera overlay on the work area"), on, can) {
                self.toggle_stream_overlay();
            }
        });
        let tex = self.stream_tex.clone();
        let picture = cv::picture(ui, tex.as_ref(), &mut self.stream_zoom, &mut self.stream_pan, &mut self.cam_fit, 40.0);
        // Photos, video and time-lapse, with their folder.
        ui.add_space(4.0);
        ui.separator();
        // A photo, a video and the time-lapse of this camera; their settings are on the Recording tab.
        if let Some(r) = self.rigs.rec(self.doc.device.active_camera) {
            r.quick(ui);
        }
        // The working time of the job, over the estimate: 00:04:12 / 00:13:00.
        let elapsed = self.job_started.map_or(self.job_last_secs, |t| t.elapsed().as_secs_f64());
        ui.label(RichText::new(format!("{} {} / {}", tr("Working time"), crate::app::fmt_clock(elapsed), crate::app::fmt_clock(self.job_est))).monospace().strong());
        if self.job_started.is_some() {
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(500));
        }
        if !ui.input(|i| i.pointer.any_down()) {
            self.save_camera_view();
        }
        if !self.stream_err.is_empty() {
            ui.label(RichText::new(trf("Stream error: {}", &[&self.stream_err.replace(self.doc.device.stream_url().unwrap_or("-"), "…")])).color(egui::Color32::from_rgb(0xc0, 0x30, 0x30)));
        }
        if let Some((rect, slack_x)) = picture {
            self.cam_fit.measure(ui, rect, slack_x);
        }
    }

    pub fn stream_window(&mut self, ctx: &egui::Context) {
        let url = self.doc.device.stream_url().unwrap_or("").to_string();
        // Every camera records on its own; the one of the view gets its pictures from the view's stream, the others read their own.
        self.rigs.sync(&self.doc.device.cameras, self.path.as_deref());
        let view = (!url.is_empty()).then_some(self.doc.device.active_camera);
        self.rigs.pump(ctx, &self.doc.device.cameras, view, std::time::Instant::now());
        if self.rigs.reading() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        if !self.show_stream || url.is_empty() {
            if self.show_stream {
                self.stream_closed();
            }
            // A recording goes on when the camera view is closed: the stream is kept while something is recorded.
            if self.rigs.active(self.doc.device.active_camera) && !url.is_empty() {
                self.stream_poll(ctx, &url);
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
                self.tab_seen = false;
                return;
            }
            self.stream = None;
            self.tab_seen = false;
            return;
        }
        self.follow_active_camera();
        self.stream_poll(ctx, &url);
        match self.stream_mode {
            StreamMode::Detached => {
                let saved = self.doc.device.camera_view.clone();
                let logo = self.logo.clone();
                let seen = cv::window(ctx, CAM_WINDOW.into(), &tr("Camera view"), logo.as_ref(), &saved, |ui| self.stream_view(ui));
                if seen.embedded {
                    // The backend cannot open real windows: fall back to the side tab.
                    self.stream_mode = StreamMode::Tab;
                }
                if let Some((_, size)) = seen.place {
                    self.cam_window_settle(ctx, size);
                }
                if cv::keep_place(ctx, &mut self.doc.device.camera_view, seen.place) {
                    self.sync_profile();
                }
                let close = seen.close;
                if close {
                    self.stream_closed();
                }
            }
            StreamMode::Workspace => {
                // Drawn in the work area, as a tab next to the drawings (`App::camera_work_tab`); keep frames flowing.
                if !self.cam_tab_seen {
                    self.cam_tab_shown = true;
                }
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
            }
            StreamMode::Tab => {
                // Drawn by the side panel; make sure the tab is there and keep frames flowing.
                if self.side_tab != SideTab::Camera && !self.tab_seen {
                    self.side_tab = SideTab::Camera;
                }
                self.tab_seen = true;
            }
        }
        if self.stream_mode != StreamMode::Tab {
            self.tab_seen = false;
        }
        self.cam_tab_seen = self.stream_mode == StreamMode::Workspace;
        if !self.cam_tab_seen {
            self.cam_tab_shown = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::Act;

    fn app_with_overlay() -> (App, egui::Context) {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.screen = crate::app::Screen::Editor;
        a.set_background(&ctx, egui::ColorImage::from_rgb([4, 2], &[200u8; 24]), "t");
        (a, ctx)
    }

    #[test]
    fn overlay_can_be_hidden_from_the_menu_after_the_camera_window_closed() {
        let (mut a, ctx) = app_with_overlay();
        assert!(a.overlay_shown());
        // The camera view fed the overlay, then its window was closed: the picture goes away with it.
        a.show_stream = true;
        a.stream_overlay = true;
        a.stream_closed();
        assert!(!a.overlay_shown() && !a.stream_overlay && !a.show_stream);
        // The menu toggle works without any window open, in both directions.
        assert_eq!(a.act_checked(Act::ToggleOverlay), Some(false));
        a.do_act(&ctx, Act::ToggleOverlay);
        assert!(a.overlay_shown());
        a.do_act(&ctx, Act::ToggleOverlay);
        assert!(!a.overlay_shown());
        // Without an overlay the toggle is disabled.
        a.ed.background = None;
        assert!(!a.act_enabled(Act::ToggleOverlay));
    }

    #[test]
    fn closing_the_camera_tab_returns_to_properties() {
        let (mut a, _ctx) = app_with_overlay();
        a.stream_mode = StreamMode::Tab;
        a.show_stream = true;
        a.side_tab = SideTab::Camera;
        a.stream_closed();
        assert_eq!(a.side_tab, SideTab::Properties);
    }

    #[test]
    fn camera_tab_is_selected_again_after_the_view_was_switched_off_and_on() {
        let (mut a, ctx) = app_with_overlay();
        a.doc.device.cameras = vec![lc_core::CameraSource::url("Camera 1", "http://127.0.0.1:9/none.mjpg", 0)];
        a.stream_mode = StreamMode::Tab;
        a.show_stream = true;
        a.stream_window(&ctx);
        assert_eq!(a.side_tab, SideTab::Camera);
        a.side_tab = SideTab::Properties;
        a.show_stream = false;
        a.stream_window(&ctx);
        a.show_stream = true;
        a.stream_window(&ctx);
        assert_eq!(a.side_tab, SideTab::Camera);
        a.stream = None;
    }

    #[test]
    fn rotation_comes_from_the_device_and_resets_the_picture() {
        let (mut a, ctx) = app_with_overlay();
        a.stream_tex = Some(ctx.load_texture("t", egui::ColorImage::from_rgb([2, 1], &[0u8; 6]), egui::TextureOptions::LINEAR));
        a.doc.device.cameras = vec![lc_core::CameraSource::url("Camera 1", "http://127.0.0.1:9/none.mjpg", 1)];
        a.stream_poll(&ctx, "http://127.0.0.1:9/none.mjpg");
        assert!(a.stream_tex.is_none(), "texture is rebuilt with the new rotation");
        assert_eq!(a.stream_rot, 1);
        a.stream = None;
    }

    #[test]
    fn the_picture_is_fitted_to_the_window_up_as_well_as_down_and_the_window_keeps_its_size() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.stream_tex = Some(ctx.load_texture("t", egui::ColorImage::from_rgb([320, 180], &vec![0u8; 320 * 180 * 3]), egui::TextureOptions::LINEAR));
        let mut sizes = vec![];
        for size in [[1000.0, 800.0], [1000.0, 800.0], [1000.0, 800.0], [1000.0, 800.0], [1000.0, 800.0], [1000.0, 800.0]] {
            let input = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1600.0, 1200.0))), ..Default::default() };
            let mut shown = None;
            let mut out = ctx.run_ui(input, |ui| {
                shown = egui::Window::new("cam").default_size(size).resizable(true).show(ui.ctx(), |ui| a.stream_view(ui)).map(|r| r.response.rect);
            });
            out.textures_delta.clear();
            sizes.push(shown.expect("the window"));
        }
        let (first, last) = (sizes[2], sizes[5]);
        assert!((first.height() - last.height()).abs() < 1.0, "the window grew: {sizes:?}");
        assert!(last.width() > 900.0 && last.height() > 500.0, "{last:?}");
    }

    #[test]
    fn the_picture_of_a_panel_that_fills_the_window_is_not_made_smaller_by_the_empty_strip_under_it() {
        let (mut a, ctx) = app_with_overlay();
        a.doc.device.cameras = vec![lc_core::CameraSource::url("A", "http://127.0.0.1:9/x", 0)];
        a.show_stream = true;
        a.stream_tex = Some(ctx.load_texture("t", egui::ColorImage::from_rgb([640, 360], &vec![0u8; 640 * 360 * 3]), egui::TextureOptions::LINEAR));
        for i in 0..6 {
            let input = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(790.0, 590.0))), ..Default::default() };
            let mut r = None;
            let mut out = ctx.run_ui(input, |ui| {
                egui::CentralPanel::default().show(ui, |ui| {
                    a.stream_view(ui);
                    r = Some((ui.min_rect(), ui.available_height()));
                });
            });
            out.textures_delta.clear();
            let _ = (i, r);
        }
        // Only the working time and the buttons of the recording stand under the picture (no empty strip); the picture fills the width of
        // the window (640 x 360 in 774 x 574).
        assert!(a.cam_fit.under(0.0) < 170.0, "under {}", a.cam_fit.under(0.0));
        // The recording buttons (the video format, the photo and video, the time-lapse, the folder) under the picture take some of the height,
        // so the picture may be narrower than the window: 25 points of height are 45 of width in a 16:9 picture.
        assert!(a.cam_fit.slack.x < 100.0, "the picture fills the width: {:?}", a.cam_fit.slack);
    }

    #[test]
    fn the_camera_picture_stands_still_while_the_window_is_resized_and_the_window_is_cut_to_it_afterwards() {
        let (mut a, ctx) = app_with_overlay();
        a.stream_mode = StreamMode::Detached;
        a.cam_window_settle(&ctx, [800.0, 600.0]);
        assert!(a.cam_resizing(), "a new size counts as a resize");
        a.cam_window_settle(&ctx, [790.0, 600.0]);
        assert!(a.cam_resizing());
        std::thread::sleep(CAM_STILL + std::time::Duration::from_millis(100));
        a.cam_window_settle(&ctx, [790.0, 600.0]);
        assert!(!a.cam_resizing(), "the same size for a while: the edge was let go");
        // 150 points of empty width beside the picture: the window would be cut by that much, but not below its smallest size.
        a.cam_fit.slack = egui::vec2(150.0, 0.0);
        assert_eq!(a.cam_fit.fitted(egui::vec2(790.0, 600.0), CAM_MIN), Some(egui::vec2(640.0, 600.0)));
        a.cam_fit.slack = egui::vec2(2.0, 1.0);
        assert_eq!(a.cam_fit.fitted(egui::vec2(790.0, 600.0), CAM_MIN), None);
    }

    #[test]
    fn each_camera_opens_its_view_its_own_way_and_a_tab_opens_the_collapsed_panel() {
        let (mut a, _ctx) = app_with_overlay();
        a.doc.device.cameras = vec![lc_core::CameraSource::url("A", "http://a/x", 0), lc_core::CameraSource::url("B", "http://b/y", 0)];
        a.doc.device.active_camera = 0;
        a.load_camera_view();
        a.set_camera_mode(1, StreamMode::Tab);
        assert_eq!(a.doc.device.cameras[1].view_mode, Some(2));
        assert_eq!(a.stream_mode, StreamMode::Tab, "camera 0 is the one in use: it keeps its way");
        a.set_camera_mode(0, StreamMode::Detached);
        assert_eq!(a.stream_mode, StreamMode::Detached);
        // Another camera is made the active one: its own way is taken, and the hidden panel shows its tab.
        a.tabs.collapsed = true;
        a.doc.device.active_camera = 1;
        a.follow_active_camera();
        assert_eq!((a.stream_mode, a.side_tab, a.tabs.collapsed), (StreamMode::Tab, SideTab::Camera, false));
        a.doc.device.active_camera = 0;
        a.follow_active_camera();
        assert_eq!(a.stream_mode, StreamMode::Detached);
    }

    #[test]
    fn the_camera_can_be_a_tab_of_the_work_area_that_a_drawing_tab_replaces() {
        let (mut a, ctx) = app_with_overlay();
        let frame = |a: &mut App, ctx: &egui::Context| {
            let mut out = ctx.run_ui(Default::default(), |ui| a.stream_window(ui.ctx()));
            out.textures_delta.clear();
        };
        a.doc.device.cameras = vec![lc_core::CameraSource::url("A", "http://127.0.0.1:9/x", 0)];
        a.show_stream = true;
        assert!(!a.camera_work_tab());
        a.set_camera_mode(0, StreamMode::Workspace);
        assert!(a.camera_work_tab() && a.cam_tab_shown);
        assert_eq!(a.doc.device.cameras[0].view_mode, Some(3));
        frame(&mut a, &ctx);
        assert!(a.cam_tab_shown, "it stays shown while the mode is the tab");
        // A drawing tab is picked: the drawing is back, the camera tab stays in the bar.
        a.new_tab();
        a.switch_to(0);
        assert!(!a.cam_tab_shown && a.camera_work_tab());
        // Another mode takes the tab away.
        a.set_camera_mode(0, StreamMode::Tab);
        frame(&mut a, &ctx);
        assert!(!a.camera_work_tab() && !a.cam_tab_shown);
    }

    #[test]
    fn the_camera_view_opens_the_way_it_was_left_for_each_device() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        let mut d1 = lc_core::Device::default();
        d1.name = "One".into();
        d1.camera_view = lc_core::CameraView { mode: 1, zoom: 3.0, ..Default::default() };
        let mut d2 = lc_core::Device::default();
        d2.name = "Two".into();
        a.profiles = vec![d1, d2];
        a.enter_editor(0);
        assert_eq!((a.stream_mode, a.stream_zoom), (StreamMode::Detached, 3.0));
        a.enter_editor(1);
        assert_eq!((a.stream_mode, a.stream_zoom), (StreamMode::Tab, 1.0), "the other device has its own");
        // A change is kept with the device and in the profile list.
        a.stream_mode = StreamMode::Workspace;
        a.stream_zoom = 2.0;
        a.save_camera_view();
        assert_eq!((a.doc.device.camera_view.mode, a.doc.device.camera_view.zoom), (3, 2.0));
        assert_eq!(a.profiles[1].camera_view.mode, 3);
        a.enter_editor(0);
        assert_eq!(a.stream_mode, StreamMode::Detached);
        a.enter_editor(1);
        assert_eq!((a.stream_mode, a.stream_zoom), (StreamMode::Workspace, 2.0));
    }

    #[test]
    fn the_clock_under_the_picture_counts_hours_minutes_seconds() {
        use crate::app::fmt_clock;
        assert_eq!(fmt_clock(0.0), "00:00:00");
        assert_eq!(fmt_clock(780.0), "00:13:00");
        assert_eq!(fmt_clock(3.0 * 3600.0 + 61.9), "03:01:01");
        assert_eq!(fmt_clock(f64::NAN), "00:00:00");
        assert_eq!(fmt_clock(-5.0), "00:00:00");
        // While a job runs the elapsed time comes from its start; afterwards the last one stays.
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.job_est = 780.0;
        a.job_started = Some(std::time::Instant::now() - std::time::Duration::from_secs(252));
        let e = a.job_started.map_or(a.job_last_secs, |t| t.elapsed().as_secs_f64());
        assert_eq!(fmt_clock(e), "00:04:12");
    }

    #[test]
    fn a_recording_keeps_the_stream_when_the_view_is_closed_and_a_picture_comes_after_each_layer() {
        use base_ui::camera_rec::{LapseMode, RecSettings};
        let (mut a, ctx) = app_with_overlay();
        let dir = std::env::temp_dir().join(format!("lc-lapse-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        a.rigs.sync(&[lc_core::CameraSource::url("t", "http://127.0.0.1:1/x", 0)], None);
        *a.rigs.rec(0).unwrap() = base_ui::camera_rec::Recorder::new(RecSettings { dir: Some(dir.clone()), interval: 5.0, mode: LapseMode::PerLayer, format: Default::default() });
        let now = std::time::Instant::now();
        a.rigs.rec(0).unwrap().feed(now, 4, 2, vec![80; 24]);
        a.rigs.rec(0).unwrap().start_lapse(now);
        // The view is closed, a time-lapse runs: the stream stays.
        a.doc.device.cameras = vec![lc_core::CameraSource::url("c", "http://127.0.0.1:9/none.mjpg", 0)];
        a.doc.device.active_camera = 0;
        a.show_stream = false;
        a.stream = None;
        let mut out = ctx.run_ui(Default::default(), |ui| a.stream_window(ui.ctx()));
        out.textures_delta.clear();
        assert!(a.stream.is_some(), "recorded: the stream is kept");
        a.rigs.rec(0).unwrap().stop_lapse();
        a.stream = None;
        let mut out = ctx.run_ui(Default::default(), |ui| a.stream_window(ui.ctx()));
        out.textures_delta.clear();
        assert!(a.stream.is_none(), "nothing recorded: no stream");
        // Two layers: one picture when the second begins, one when the job is over; a stopped job (progress 0, 0) does nothing more.
        a.rigs.rec(0).unwrap().start_lapse(now);
        a.layer_watch.start(vec![6]);
        a.layer_progress(3, 10);
        assert_eq!(a.rigs.rec(0).unwrap().lapse_shots(), Some(0));
        a.layer_progress(6, 10);
        assert_eq!(a.rigs.rec(0).unwrap().lapse_shots(), Some(1));
        a.layer_progress(10, 10);
        assert_eq!(a.rigs.rec(0).unwrap().lapse_shots(), Some(2));
        a.layer_watch.start(vec![2]);
        a.layer_progress(0, 0);
        a.layer_progress(5, 10);
        assert_eq!(a.rigs.rec(0).unwrap().lapse_shots(), Some(2));
        let _ = std::fs::remove_dir_all(&dir);
    }
}

