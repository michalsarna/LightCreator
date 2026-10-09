//! The live picture from the device's camera URL. It can float inside the app, open in its own window
//! (which may leave the app window and the screen) or sit in the side panel as a tab.
use crate::app::{App, SideTab};
use crate::camera::rotate_rgb;
use crate::camera_stream::{self, Msg};
use crate::i18n::{tr, trf};
use crate::theme;
use eframe::egui::{self, RichText};

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
    pub const ALL: [StreamMode; 3] = [StreamMode::Floating, StreamMode::Detached, StreamMode::Tab];
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
            let (w, h, rgb) = rotate_rgb(&f.rgb, f.w, f.h, self.doc.device.camera_rotation);
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
        let url = self.doc.device.camera_url.trim().to_string();
        ui.horizontal_wrapped(|ui| {
            ui.label(tr("Show in"));
            let before = self.stream_mode;
            for m in StreamMode::ALL {
                ui.selectable_value(&mut self.stream_mode, m, tr(m.label()));
            }
            if before != self.stream_mode && self.stream_mode == StreamMode::Tab {
                self.side_tab = SideTab::Camera;
            }
            if before == StreamMode::Tab && self.stream_mode != StreamMode::Tab && self.side_tab == SideTab::Camera {
                self.side_tab = SideTab::Properties;
            }
        });
        ui.horizontal(|ui| {
            ui.label(tr("Rotate"));
            let mut r = self.doc.device.camera_rotation % 4;
            for q in 0..4u8 {
                ui.selectable_value(&mut r, q, format!("{}°", q as u32 * 90));
            }
            if r != self.doc.device.camera_rotation {
                self.doc.device.camera_rotation = r;
                self.sync_profile();
                // The next frame is rotated; refresh the texture size right away.
                self.stream_tex = None;
            }
        });
        ui.label(RichText::new(&url).monospace().color(theme::text_dim()));
        let avail = ui.available_width().min(1400.0);
        match &self.stream_tex {
            Some(t) => {
                let sz = t.size_vec2();
                let k = (avail / sz.x).min((ui.available_height() - 70.0).max(160.0) / sz.y).min(2.0);
                ui.image((t.id(), sz * k));
            }
            None => {
                ui.add_space(30.0);
                ui.label(tr("Waiting for the picture…"));
                ui.add_space(30.0);
            }
        }
        if !self.stream_err.is_empty() {
            ui.label(RichText::new(trf("Stream error: {}", &[&self.stream_err])).color(egui::Color32::from_rgb(0xc0, 0x30, 0x30)));
        }
        ui.checkbox(&mut self.stream_overlay, tr("Show as overlay on the work area"));
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
                egui::Window::new(tr("Camera view")).open(&mut open).default_size([680.0, 540.0]).resizable(true).show(ctx, |ui| self.stream_view(ui));
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
}
