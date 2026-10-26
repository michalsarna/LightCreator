//! The live picture from the device's camera URL. It can float inside the app, open in its own window
//! (which may leave the app window and the screen) or sit in the side panel as a tab.
use crate::app::{App, SideTab};
use crate::camera::rotate_rgb;
use crate::camera_stream::{self, Msg};
use crate::i18n::{tr, trf};
use crate::theme;
use eframe::egui::{self, RichText};

const MAX_ZOOM: f32 = 8.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StreamMode {
    /// A window inside the application.
    Floating,
    /// A separate operating-system window.
    Detached,
    /// A tab of the side panel.
    Tab,
}

impl StreamMode {
    pub fn from_index(i: u8) -> StreamMode {
        match i {
            1 => StreamMode::Detached,
            2 => StreamMode::Tab,
            _ => StreamMode::Floating,
        }
    }
    pub fn index(self) -> u8 {
        match self {
            StreamMode::Floating => 0,
            StreamMode::Detached => 1,
            StreamMode::Tab => 2,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            StreamMode::Floating => "Floating window",
            StreamMode::Detached => "Separate window",
            StreamMode::Tab => "Side tab",
        }
    }
}

impl App {
    /// Take the way of showing the camera view from the device (when a device is entered).
    pub fn load_camera_view(&mut self) {
        let v = &self.doc.device.camera_view;
        self.stream_mode = StreamMode::from_index(v.mode);
        self.stream_zoom = v.zoom.clamp(1.0, MAX_ZOOM);
    }

    /// Remember the mode and zoom with the device when they changed.
    pub(crate) fn save_camera_view(&mut self) {
        let v = &mut self.doc.device.camera_view;
        let (mode, zoom) = (self.stream_mode.index(), self.stream_zoom);
        if v.mode != mode || (v.zoom - zoom).abs() > 0.005 {
            v.mode = mode;
            v.zoom = zoom;
            self.sync_profile();
        }
    }

    /// Stop the stream. A picture it fed into the work area is hidden with it.
    pub fn stream_closed(&mut self) {
        self.show_stream = false;
        self.stream = None;
        self.tab_seen = false;
        if self.stream_overlay {
            self.stream_overlay = false;
            self.overlay_visible = false;
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
        if let Some(f) = latest {
            let rgb = self.cam_process(f.rgb, f.w, f.h);
            let (w, h, rgb) = rotate_rgb(&rgb, f.w, f.h, rotation);
            self.api_publish_frame(w, h, &rgb);
            let img = egui::ColorImage::from_rgb([w as usize, h as usize], &rgb);
            match &mut self.stream_tex {
                Some(t) if t.size() == [w as usize, h as usize] => t.set(img.clone(), egui::TextureOptions::LINEAR),
                _ => self.stream_tex = Some(ctx.load_texture("camera-stream", img.clone(), egui::TextureOptions::LINEAR)),
            }
            if self.stream_overlay {
                self.overlay_set_image(ctx, img, &tr("Camera view"));
            }
        }
    }

    /// Apply the active camera's lens correction and picture adjustments to a frame (RGB bytes of `w × h`).
    pub fn cam_process(&self, rgb: Vec<u8>, w: u32, h: u32) -> Vec<u8> {
        let Some(cam) = self.doc.device.cameras.get(self.doc.device.active_camera) else { return rgb };
        let (w, h) = (w as usize, h as usize);
        let mut out = if cam.lens.is_none() { rgb } else { lc_core::camimg::undistort(&rgb, w, h, &cam.lens) };
        lc_core::camimg::adjust(&mut out, w, h, &cam.image);
        out
    }

    /// Picture and controls, shared by the three ways of showing the stream.
    pub fn stream_view(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let before = self.stream_mode;
            for (m, icon) in [(StreamMode::Floating, "picture-in-picture-2"), (StreamMode::Detached, "square-arrow-out-up-right"), (StreamMode::Tab, "panel-right")] {
                if crate::app::toggle_icon(ui, icon, &tr(m.label()), self.stream_mode == m) {
                    self.stream_mode = m;
                }
            }
            if before != self.stream_mode && self.stream_mode == StreamMode::Tab {
                self.side_tab = SideTab::Camera;
            }
            if before == StreamMode::Tab && self.stream_mode != StreamMode::Tab && self.side_tab == SideTab::Camera {
                self.side_tab = SideTab::Properties;
            }
            ui.separator();
            let (on, can) = self.overlay_state();
            if crate::app::window_toggle(ui, &tr("Overlay"), &tr("Show or hide the camera overlay on the work area"), on, can) {
                self.toggle_overlay_button();
            }
            ui.separator();
            if ui.button("+").on_hover_text(tr("Zoom in")).clicked() {
                self.stream_zoom = (self.stream_zoom * 1.25).min(MAX_ZOOM);
            }
            if ui.button("−").on_hover_text(tr("Zoom out")).clicked() {
                self.stream_zoom = (self.stream_zoom / 1.25).max(1.0);
            }
            if ui.button(tr("Default zoom")).on_hover_text(tr("Back to the whole picture")).clicked() {
                self.stream_zoom = 1.0;
            }
            ui.label(RichText::new(format!("{:.0} %", self.stream_zoom * 100.0)).color(theme::text_dim()));
        });
        let mut picture = None;
        match self.stream_tex.clone() {
            Some(t) => {
                let sz = t.size_vec2();
                // 100 % is the picture fitted to the window, scaled up as well as down. What stands under it (the working time and
                // a stream error) is left out of its height: the last frame measured it (`FitWindow`).
                let (k, slack_x) = base_ui::window::fit_picture(ui, sz, self.cam_fit.under(40.0));
                let (resp, painter) = ui.allocate_painter(sz * k, egui::Sense::drag());
                picture = Some((resp.rect, slack_x));
                if resp.hovered() {
                    let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                    if scroll != 0.0 {
                        self.stream_zoom = (self.stream_zoom * (scroll * 0.0015).exp()).clamp(1.0, MAX_ZOOM);
                    }
                }
                if resp.dragged() {
                    self.stream_pan += resp.drag_delta();
                }
                if resp.double_clicked() {
                    self.stream_zoom = 1.0;
                }
                // The picture cannot be pushed out of its frame.
                let room = resp.rect.size() * (self.stream_zoom - 1.0) / 2.0;
                self.stream_pan = self.stream_pan.clamp(-room, room);
                let img = egui::Rect::from_center_size(resp.rect.center() + self.stream_pan, resp.rect.size() * self.stream_zoom);
                painter.with_clip_rect(resp.rect).image(t.id(), img, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
            }
            None => {
                self.cam_fit.slack = egui::Vec2::ZERO;
                ui.add_space(30.0);
                ui.label(tr("Waiting for the picture…"));
                ui.add_space(30.0);
            }
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
        if !self.show_stream || url.is_empty() {
            if self.show_stream {
                self.stream_closed();
            }
            self.stream = None;
            self.tab_seen = false;
            return;
        }
        self.stream_poll(ctx, &url);
        match self.stream_mode {
            StreamMode::Floating => {
                let mut open = true;
                let saved = self.doc.device.camera_view.clone();
                // A window that is bigger than the picture needs is opened again smaller (see `FitWindow`).
                let size = self.cam_fit.size_or(saved.float_size.unwrap_or([680.0, 540.0]));
                let mut win = egui::Window::new(self.cam_fit.id(&format!("camera_window_{}", self.active))).title_bar(false).default_size(size).resizable(true);
                if let Some(p) = self.cam_fit.pos().or(saved.float_pos.map(|p| egui::pos2(p[0], p[1]))) {
                    win = win.default_pos(p);
                }
                let shown = win.show(ctx, |ui| {
                    crate::window_ui::mini_title(ui, &tr("Camera view"), &mut open, |_| {});
                    self.stream_view(ui);
                });
                // The place and size are kept with the device, once the mouse is released.
                if let Some(r) = shown.map(|r| r.response.rect) {
                    self.cam_fit.settle(ctx, r);
                    if !ctx.input(|i| i.pointer.any_down()) {
                        let (pos, size) = ([r.min.x.round(), r.min.y.round()], [r.width().round(), r.height().round()]);
                        let v = &mut self.doc.device.camera_view;
                        if v.float_pos != Some(pos) || v.float_size != Some(size) {
                            v.float_pos = Some(pos);
                            v.float_size = Some(size);
                            self.sync_profile();
                        }
                    }
                }
                if !open {
                    self.stream_closed();
                }
            }
            StreamMode::Detached => {
                let saved = self.doc.device.camera_view.clone();
                let mut builder = egui::ViewportBuilder::default().with_title(tr("Camera view")).with_inner_size(saved.win_size.unwrap_or([720.0, 580.0])).with_min_inner_size([320.0, 240.0]);
                if let Some(p) = saved.win_pos {
                    builder = builder.with_position(p);
                }
                let mut seen: Option<([f32; 2], [f32; 2])> = None;
                let mut close = false;
                ctx.show_viewport_immediate(egui::ViewportId::from_hash_of("lightcreator-camera"), builder, |ui, class| {
                    if class == egui::ViewportClass::EmbeddedWindow {
                        // The backend cannot open real windows: fall back to a floating window.
                        self.stream_mode = StreamMode::Floating;
                        return;
                    }
                    egui::CentralPanel::default().show(ui, |ui| self.stream_view(ui));
                    ui.input(|i| {
                        let vp = i.viewport();
                        if let (Some(o), Some(inner)) = (vp.outer_rect, vp.inner_rect) {
                            seen = Some(([o.min.x.round(), o.min.y.round()], [inner.width().round(), inner.height().round()]));
                        }
                    });
                    // The window gives back the empty strip beside or under the picture, once the mouse is let go.
                    if let Some(inner) = ui.input(|i| i.viewport().inner_rect) {
                        if let Some(w) = self.cam_fit.viewport_size(ui.ctx(), inner.size()) {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::InnerSize(w));
                        }
                    }
                    if ui.input(|i| i.viewport().close_requested()) {
                        close = true;
                    }
                });
                if let Some((pos, size)) = seen {
                    let v = &mut self.doc.device.camera_view;
                    if size[0] >= 100.0 && size[1] >= 100.0 && (v.win_pos != Some(pos) || v.win_size != Some(size)) && !ctx.input(|i| i.pointer.any_down()) {
                        v.win_pos = Some(pos);
                        v.win_size = Some(size);
                        self.sync_profile();
                    }
                }
                if close {
                    self.stream_closed();
                }
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
        a.overlay_set_image(&ctx, egui::ColorImage::from_rgb([4, 2], &[200u8; 24]), "t");
        (a, ctx)
    }

    #[test]
    fn overlay_can_be_hidden_from_the_menu_after_the_camera_window_closed() {
        let (mut a, ctx) = app_with_overlay();
        assert!(a.overlay_visible);
        // The camera view fed the overlay, then its window was closed: the picture goes away with it.
        a.show_stream = true;
        a.stream_overlay = true;
        a.stream_closed();
        assert!(!a.overlay_visible && !a.stream_overlay && !a.show_stream);
        // The menu toggle works without any window open, in both directions.
        assert_eq!(a.act_checked(Act::ToggleOverlay), Some(false));
        a.do_act(&ctx, Act::ToggleOverlay);
        assert!(a.overlay_visible);
        a.do_act(&ctx, Act::ToggleOverlay);
        assert!(!a.overlay_visible);
        // Without an overlay the toggle is disabled.
        a.overlay = None;
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
    fn the_floating_camera_window_shrinks_to_the_shape_of_the_picture() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.doc.device.cameras = vec![lc_core::CameraSource::url("Camera 1", "http://127.0.0.1:9/none.mjpg", 1)];
        a.doc.device.active_camera = 0;
        a.stream_mode = StreamMode::Floating;
        a.show_stream = true;
        for i in 0..14 {
            let input = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1600.0, 1200.0))), ..Default::default() };
            let mut out = ctx.run_ui(input, |ui| a.stream_window(ui.ctx()));
            out.textures_delta.clear();
            if i == 0 {
                a.stream_tex = Some(ctx.load_texture("t", egui::ColorImage::from_rgb([320, 180], &vec![0u8; 320 * 180 * 3]), egui::TextureOptions::LINEAR));
            }
        }
        let [w, h] = a.cam_fit.size_or([0.0, 0.0]);
        assert!(w > 0.0, "the window was never fitted");
        // It opens 680 x 540; a 16:9 picture (and the toolbar and the working time round it) needs less than that in both.
        assert!(h < 520.0 && w <= 680.0, "{w} x {h}");
        // And once fitted it stays so.
        for _ in 0..8 {
            let input = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1600.0, 1200.0))), ..Default::default() };
            let mut out = ctx.run_ui(input, |ui| a.stream_window(ui.ctx()));
            out.textures_delta.clear();
        }
        let [w2, h2] = a.cam_fit.size_or([0.0, 0.0]);
        assert!((w - w2).abs() < 1.0 && (h - h2).abs() < 1.0, "{w} x {h} -> {w2} x {h2}");
    }

    #[test]
    fn the_camera_view_opens_the_way_it_was_left_for_each_device() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        let mut d1 = lc_core::Device::default();
        d1.name = "One".into();
        d1.camera_view = lc_core::CameraView { mode: 2, zoom: 3.0, ..Default::default() };
        let mut d2 = lc_core::Device::default();
        d2.name = "Two".into();
        a.profiles = vec![d1, d2];
        a.enter_editor(0);
        assert_eq!((a.stream_mode, a.stream_zoom), (StreamMode::Tab, 3.0));
        a.enter_editor(1);
        assert_eq!((a.stream_mode, a.stream_zoom), (StreamMode::Floating, 1.0), "the other device has its own");
        // A change is kept with the device and in the profile list.
        a.stream_mode = StreamMode::Detached;
        a.stream_zoom = 2.0;
        a.save_camera_view();
        assert_eq!((a.doc.device.camera_view.mode, a.doc.device.camera_view.zoom), (1, 2.0));
        assert_eq!(a.profiles[1].camera_view.mode, 1);
        a.enter_editor(0);
        assert_eq!(a.stream_mode, StreamMode::Tab);
        a.enter_editor(1);
        assert_eq!((a.stream_mode, a.stream_zoom), (StreamMode::Detached, 2.0));
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
}
