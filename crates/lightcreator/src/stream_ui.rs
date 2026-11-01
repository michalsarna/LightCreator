//! The live picture from the device's camera URL. It can open in its own window (which may leave the
//! app window and the screen), sit in the side panel as a tab or be a tab of the work area.
use crate::app::{App, SideTab};
use crate::camera::rotate_rgb;
use crate::camera_stream::{self, Msg};
use crate::i18n::{tr, trf};
use crate::theme;
use eframe::egui::{self, RichText};

/// How long the camera window must keep its size to count as let go.
const CAM_STILL: std::time::Duration = std::time::Duration::from_millis(500);
/// The smallest camera window (the same as its minimum inner size).
const CAM_MIN: [f32; 2] = [320.0, 240.0];
const MAX_ZOOM: f32 = 8.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StreamMode {
    /// A separate operating-system window.
    Detached,
    /// A tab of the side panel.
    Tab,
    /// A tab of the work area, next to the drawings.
    Workspace,
}

impl StreamMode {
    pub fn from_index(i: u8) -> StreamMode {
        match i {
            1 => StreamMode::Detached,
            2 => StreamMode::Tab,
            3 => StreamMode::Workspace,
            // 0 was a window inside the application: it is a tab of the side panel now.
            _ => StreamMode::Tab,
        }
    }
    pub fn index(self) -> u8 {
        match self {
            StreamMode::Detached => 1,
            StreamMode::Tab => 2,
            StreamMode::Workspace => 3,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            StreamMode::Detached => "Separate window",
            StreamMode::Tab => "Side tab",
            StreamMode::Workspace => "Tab in the work area",
        }
    }
}

/// The three ways of showing the camera view, with their icons.
const MODE_ICONS: [(StreamMode, &str); 3] = [(StreamMode::Detached, "square-arrow-out-up-right"), (StreamMode::Tab, "panel-right"), (StreamMode::Workspace, "panel-top")];

impl StreamMode {
    /// The name of the mode, translated (the work-area tab is a text of the shared interface).
    pub fn tip(self) -> String {
        match self {
            StreamMode::Workspace => base_ui::doc_tabs::work_tab_label().to_string(),
            m => tr(m.label()).to_string(),
        }
    }
}

impl App {
    /// Take the way of showing the camera view from the device (when a device is entered).
    pub fn load_camera_view(&mut self) {
        let d = &self.doc.device;
        let mode = d.cameras.get(d.active_camera).and_then(|c| c.view_mode).unwrap_or(d.camera_view.mode);
        self.stream_mode = StreamMode::from_index(mode);
        self.stream_zoom = d.camera_view.zoom.clamp(1.0, MAX_ZOOM);
        self.cam_view_for = d.active_camera;
    }

    /// Remember the mode (with the active camera) and the zoom (with the device) when they changed.
    pub(crate) fn save_camera_view(&mut self) {
        let d = &mut self.doc.device;
        let (mode, zoom) = (self.stream_mode.index(), self.stream_zoom);
        let cam = d.cameras.get(d.active_camera).and_then(|c| c.view_mode);
        let new_cam = d.active_camera < d.cameras.len() && cam != Some(mode);
        let v = &mut d.camera_view;
        if new_cam || v.mode != mode || (v.zoom - zoom).abs() > 0.005 {
            v.mode = mode;
            v.zoom = zoom;
            if let Some(c) = d.cameras.get_mut(d.active_camera) {
                c.view_mode = Some(mode);
            }
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
        // While the camera window is being resized the picture stands still (new frames are dropped); the stream goes on after.
        if self.cam_resizing() {
            latest = None;
        }
        if let Some(f) = latest {
            let rgb = self.cam_process(f.rgb, f.w, f.h);
            let (w, h, rgb) = rotate_rgb(&rgb, f.w, f.h, rotation);
            self.api_publish_frame(w, h, &rgb);
            self.cam_rec.feed(std::time::Instant::now(), w, h, rgb.clone());
            let img = egui::ColorImage::from_rgb([w as usize, h as usize], &rgb);
            match &mut self.stream_tex {
                Some(t) if t.size() == [w as usize, h as usize] => t.set(img.clone(), egui::TextureOptions::LINEAR),
                _ => self.stream_tex = Some(ctx.load_texture("camera-stream", img.clone(), egui::TextureOptions::LINEAR)),
            }
            if self.stream_overlay {
                self.overlay_set_image(ctx, img, &tr("Camera view"));
            }
        }
        self.cam_rec.tick(std::time::Instant::now());
    }

    /// Is the separate camera window being resized (its size changed a moment ago)?
    pub fn cam_resizing(&self) -> bool {
        self.stream_mode == StreamMode::Detached && self.cam_win.is_some_and(|(_, t)| t.elapsed() < CAM_STILL)
    }

    /// Note the size of the separate camera window; once it has stood still for a while (the edge was let go) and is bigger
    /// than its picture needs, give back the empty strip: the window is cut to the scaled picture.
    fn cam_window_settle(&mut self, ctx: &egui::Context, size: [f32; 2]) {
        match self.cam_win {
            Some((s, _)) if s == size => {}
            _ => self.cam_win = Some((size, std::time::Instant::now())),
        }
        if self.cam_resizing() {
            ctx.request_repaint_after(CAM_STILL);
        } else if let Some(w) = self.cam_fit.fitted(egui::vec2(size[0], size[1]), CAM_MIN) {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(w));
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

    /// In the view options: where the camera view of each camera opens.
    pub fn camera_mode_options(&mut self, ui: &mut egui::Ui) {
        if self.doc.device.cameras.is_empty() {
            return;
        }
        ui.add_space(8.0);
        ui.label(RichText::new(tr("Camera view")).strong());
        let rows: Vec<(String, StreamMode)> = self.doc.device.cameras.iter().map(|c| (c.name.clone(), StreamMode::from_index(c.view_mode.unwrap_or(self.doc.device.camera_view.mode)))).collect();
        let mut pick = None;
        egui::Grid::new("camera_modes").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
            for (i, (name, now)) in rows.iter().enumerate() {
                ui.label(name);
                ui.horizontal(|ui| {
                    for (m, icon) in MODE_ICONS {
                        if crate::app::toggle_icon(ui, icon, &m.tip(), *now == m) {
                            pick = Some((i, m));
                        }
                    }
                });
                ui.end_row();
            }
        });
        if let Some((i, m)) = pick {
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
            for (m, icon) in MODE_ICONS {
                if crate::app::toggle_icon(ui, icon, &m.tip(), self.stream_mode == m) {
                    self.stream_mode = m;
                }
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
        // Photos, video and time-lapse, with their folder.
        ui.add_space(4.0);
        ui.separator();
        self.cam_rec.set_project(self.path.as_deref());
        self.cam_rec.set_source(self.doc.device.cameras.get(self.doc.device.active_camera));
        self.cam_rec.controls(ui);
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
            // A recording goes on when the camera view is closed: the stream is kept while something is recorded.
            if self.cam_rec.active() && !url.is_empty() {
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
                let mut builder = egui::ViewportBuilder::default().with_title(tr("Camera view")).with_inner_size(saved.win_size.unwrap_or([720.0, 580.0])).with_min_inner_size([320.0, 240.0]);
                if let Some(p) = saved.win_pos {
                    builder = builder.with_position(p);
                }
                let mut seen: Option<([f32; 2], [f32; 2])> = None;
                let mut close = false;
                ctx.show_viewport_immediate(egui::ViewportId::from_hash_of("lightcreator-camera"), builder, |ui, class| {
                    if class == egui::ViewportClass::EmbeddedWindow {
                        // The backend cannot open real windows: fall back to the side tab.
                        self.stream_mode = StreamMode::Tab;
                        return;
                    }
                    egui::CentralPanel::default().show(ui, |ui| self.stream_view(ui));
                    ui.input(|i| {
                        let vp = i.viewport();
                        if let (Some(o), Some(inner)) = (vp.outer_rect, vp.inner_rect) {
                            seen = Some(([o.min.x.round(), o.min.y.round()], [inner.width().round(), inner.height().round()]));
                        }
                    });
                    if let Some((_, size)) = seen {
                        self.cam_window_settle(ui.ctx(), size);
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
        // The recording buttons under the picture take some of the height, so the picture may be a little narrower than the window.
        assert!(a.cam_fit.slack.x < 60.0, "the picture fills the width: {:?}", a.cam_fit.slack);
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
        a.cam_rec = base_ui::camera_rec::Recorder::new(RecSettings { dir: Some(dir.clone()), interval: 5.0, mode: LapseMode::PerLayer, format: Default::default() });
        let now = std::time::Instant::now();
        a.cam_rec.feed(now, 4, 2, vec![80; 24]);
        a.cam_rec.start_lapse(now);
        // The view is closed, a time-lapse runs: the stream stays.
        a.doc.device.cameras = vec![lc_core::CameraSource::url("c", "http://127.0.0.1:9/none.mjpg", 0)];
        a.doc.device.active_camera = 0;
        a.show_stream = false;
        a.stream = None;
        let mut out = ctx.run_ui(Default::default(), |ui| a.stream_window(ui.ctx()));
        out.textures_delta.clear();
        assert!(a.stream.is_some(), "recorded: the stream is kept");
        a.cam_rec.stop_lapse();
        a.stream = None;
        let mut out = ctx.run_ui(Default::default(), |ui| a.stream_window(ui.ctx()));
        out.textures_delta.clear();
        assert!(a.stream.is_none(), "nothing recorded: no stream");
        // Two layers: one picture when the second begins, one when the job is over; a stopped job (progress 0, 0) does nothing more.
        a.cam_rec.start_lapse(now);
        a.layer_watch.start(vec![6]);
        a.layer_progress(3, 10);
        assert_eq!(a.cam_rec.lapse_shots(), Some(0));
        a.layer_progress(6, 10);
        assert_eq!(a.cam_rec.lapse_shots(), Some(1));
        a.layer_progress(10, 10);
        assert_eq!(a.cam_rec.lapse_shots(), Some(2));
        a.layer_watch.start(vec![2]);
        a.layer_progress(0, 0);
        a.layer_progress(5, 10);
        assert_eq!(a.cam_rec.lapse_shots(), Some(2));
        let _ = std::fs::remove_dir_all(&dir);
    }
}

