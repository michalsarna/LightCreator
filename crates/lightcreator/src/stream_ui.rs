//! Window with the live picture from the device's camera URL.
use crate::app::App;
use crate::camera_stream::{self, Msg};
use crate::i18n::{tr, trf};
use crate::theme;
use eframe::egui::{self, RichText};

impl App {
    pub fn stream_window(&mut self, ctx: &egui::Context) {
        let url = self.doc.device.camera_url.trim().to_string();
        if !self.show_stream || url.is_empty() {
            self.show_stream = false;
            self.stream = None;
            return;
        }
        // (Re)start the worker for the current address.
        if self.stream.as_ref().map(|s| &s.0) != Some(&url) {
            self.stream_err.clear();
            self.stream = Some((url.clone(), camera_stream::spawn(url.clone(), ctx.clone())));
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
            let img = egui::ColorImage::from_rgb([f.w as usize, f.h as usize], &f.rgb);
            match &mut self.stream_tex {
                Some(t) if t.size() == [f.w as usize, f.h as usize] => t.set(img.clone(), egui::TextureOptions::LINEAR),
                _ => self.stream_tex = Some(ctx.load_texture("camera-stream", img.clone(), egui::TextureOptions::LINEAR)),
            }
            if self.stream_overlay {
                self.overlay_set_image(ctx, img, &tr("Camera view"));
            }
        }
        let mut open = true;
        egui::Window::new(tr("Camera view")).open(&mut open).default_size([680.0, 520.0]).resizable(true).show(ctx, |ui| {
            ui.label(RichText::new(&url).monospace().color(theme::text_dim()));
            let avail = ui.available_width().min(900.0);
            match &self.stream_tex {
                Some(t) => {
                    let sz = t.size_vec2();
                    let k = (avail / sz.x).min(640.0 / sz.y).min(2.0);
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
        });
        if !open {
            self.show_stream = false;
            self.stream = None;
        }
    }
}
