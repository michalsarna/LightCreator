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
    pub fn label(self) -> &'static str {
        match self {
            StreamMode::Floating => "Floating window",
            StreamMode::Detached => "Separate window",
            StreamMode::Tab => "Side tab",
        }
    }
}

impl App {
    /// Stop the stream. A picture it fed into the work area is hidden with it.
    pub fn stream_closed(&mut self) {
        self.show_stream = false;
        self.stream = None;
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
        let rotation = self.doc.device.camera_rotation % 4;
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
            let (w, h, rgb) = rotate_rgb(&f.rgb, f.w, f.h, rotation);
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
        let avail = ui.available_width().min(1400.0);
        match self.stream_tex.clone() {
            Some(t) => {
                let sz = t.size_vec2();
                let k = (avail / sz.x).min((ui.available_height() - 70.0).max(160.0) / sz.y).min(2.0);
                let (resp, painter) = ui.allocate_painter(sz * k, egui::Sense::drag());
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
                ui.add_space(30.0);
                ui.label(tr("Waiting for the picture…"));
                ui.add_space(30.0);
            }
        }
        if !self.stream_err.is_empty() {
            ui.label(RichText::new(trf("Stream error: {}", &[&self.stream_err.replace(self.doc.device.camera_url.trim(), "…")])).color(egui::Color32::from_rgb(0xc0, 0x30, 0x30)));
        }
    }

    pub fn stream_window(&mut self, ctx: &egui::Context) {
        let url = self.doc.device.camera_url.trim().to_string();
        if !self.show_stream || url.is_empty() {
            if self.show_stream {
                self.stream_closed();
            }
            self.stream = None;
            return;
        }
        self.stream_poll(ctx, &url);
        match self.stream_mode {
            StreamMode::Floating => {
                let mut open = true;
                egui::Window::new(RichText::new(tr("Camera view")).size(12.0)).open(&mut open).default_size([680.0, 540.0]).resizable(true).show(ctx, |ui| self.stream_view(ui));
                if !open {
                    self.stream_closed();
                }
            }
            StreamMode::Detached => {
                let builder = egui::ViewportBuilder::default().with_title(tr("Camera view")).with_inner_size([720.0, 580.0]).with_min_inner_size([320.0, 240.0]);
                let mut close = false;
                ctx.show_viewport_immediate(egui::ViewportId::from_hash_of("lightcreator-camera"), builder, |ui, class| {
                    if class == egui::ViewportClass::EmbeddedWindow {
                        // The backend cannot open real windows: fall back to a floating window.
                        self.stream_mode = StreamMode::Floating;
                        return;
                    }
                    egui::CentralPanel::default().show(ui, |ui| self.stream_view(ui));
                    if ui.input(|i| i.viewport().close_requested()) {
                        close = true;
                    }
                });
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
    fn rotation_comes_from_the_device_and_resets_the_picture() {
        let (mut a, ctx) = app_with_overlay();
        a.stream_tex = Some(ctx.load_texture("t", egui::ColorImage::from_rgb([2, 1], &[0u8; 6]), egui::TextureOptions::LINEAR));
        a.doc.device.camera_rotation = 1;
        a.stream_poll(&ctx, "http://127.0.0.1:9/none.mjpg");
        assert!(a.stream_tex.is_none(), "texture is rebuilt with the new rotation");
        assert_eq!(a.stream_rot, 1);
        a.stream = None;
    }
}
