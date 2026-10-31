//! The "Test" button of a camera in the device settings: opens its stream in a small window, with the rotation, picture
//! settings and lens correction of that camera as they are being edited, without changing the saved device.
use crate::app::App;
use crate::camera::{self, rotate_rgb};
use crate::camera_stream::{self, Msg};
use crate::i18n::{tr, trf};
use eframe::egui::{self, Color32, RichText};
use lc_core::{CameraKind, CameraSource};

enum Source {
    Url(camera_stream::StreamWorker),
    Local(camera::Worker),
}

pub struct CamTest {
    cam: CameraSource,
    src: Source,
    tex: Option<egui::TextureHandle>,
    err: String,
}

impl CamTest {
    fn start(cam: CameraSource, ctx: &egui::Context) -> Option<CamTest> {
        let src = match &cam.kind {
            CameraKind::Url(u) if lc_core::camimg::valid_url(u) => Source::Url(camera_stream::spawn(u.trim().to_string(), ctx.clone())),
            CameraKind::Local(i) if camera::SUPPORTED => Source::Local(camera::spawn(*i, cam.image.prefer_yuyv, ctx.clone())),
            _ => return None,
        };
        Some(CamTest { cam, src, tex: None, err: String::new() })
    }

    /// Take the frames that have arrived and turn the newest into a texture.
    fn poll(&mut self, ctx: &egui::Context) {
        let mut latest = None;
        match &self.src {
            Source::Url(w) => {
                while let Ok(m) = w.rx.try_recv() {
                    match m {
                        Msg::Frame(f) => {
                            self.err.clear();
                            latest = Some(f);
                        }
                        Msg::Error(e) => self.err = e,
                    }
                }
            }
            Source::Local(w) => {
                while let Ok(m) = w.rx.try_recv() {
                    match m {
                        Ok(f) => {
                            self.err.clear();
                            latest = Some(f);
                        }
                        Err(e) => self.err = e,
                    }
                }
            }
        }
        if let Some(f) = latest {
            let (w, h) = (f.w as usize, f.h as usize);
            let mut rgb = if self.cam.lens.is_none() { f.rgb } else { lc_core::camimg::undistort(&f.rgb, w, h, &self.cam.lens) };
            lc_core::camimg::adjust(&mut rgb, w, h, &self.cam.image);
            let (w, h, rgb) = rotate_rgb(&rgb, f.w, f.h, self.cam.rotation % 4);
            let img = egui::ColorImage::from_rgb([w as usize, h as usize], &rgb);
            match &mut self.tex {
                Some(t) if t.size() == [w as usize, h as usize] => t.set(img, egui::TextureOptions::LINEAR),
                _ => self.tex = Some(ctx.load_texture("camera-test", img, egui::TextureOptions::LINEAR)),
            }
        }
    }
}

impl App {
    /// Open (or restart) the test window for `cam`. Does nothing for a camera that cannot be opened.
    pub fn start_cam_test(&mut self, ctx: &egui::Context, cam: CameraSource) {
        self.cam_test = CamTest::start(cam, ctx);
    }

    pub fn cam_test_window(&mut self, ctx: &egui::Context) {
        let Some(mut t) = self.cam_test.take() else { return };
        t.poll(ctx);
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
        let mut open = true;
        let title = trf("Camera test: {}", &[&t.cam.name]);
        crate::window_ui::popup(ctx, "cam_test_window", &title, [480.0, 380.0], &mut open, |ui| match &t.tex {
            Some(tex) => {
                let sz = tex.size_vec2();
                let k = (ui.available_width() / sz.x).min((ui.available_height() - 30.0).max(100.0) / sz.y).min(2.0);
                ui.image((tex.id(), sz * k));
            }
            None => {
                ui.add_space(30.0);
                ui.label(tr("Waiting for the picture…"));
                ui.add_space(30.0);
            }
        });
        if !t.err.is_empty() {
            egui::Area::new(egui::Id::unique("cam_test_err")).anchor(egui::Align2::CENTER_BOTTOM, [0.0, -8.0]).show(ctx, |ui| {
                ui.label(RichText::new(trf("Stream error: {}", &[&t.err.replace(match &t.cam.kind { CameraKind::Url(u) => u.as_str(), _ => "-" }, "…")])).color(Color32::from_rgb(0xc0, 0x30, 0x30)));
            });
        }
        if open {
            self.cam_test = Some(t);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_camera_that_can_be_opened_starts_a_test() {
        let ctx = egui::Context::default();
        assert!(CamTest::start(CameraSource::url("x", "ftp://h/x", 0), &ctx).is_none(), "not an address of a camera");
        assert!(CamTest::start(CameraSource::url("x", "rtsp://127.0.0.1:9/x", 0), &ctx).is_some(), "an RTSP stream is tried (through ffmpeg)");
        assert!(CamTest::start(CameraSource::url("x", "", 0), &ctx).is_none());
        assert!(CamTest::start(CameraSource::url("x", "http://127.0.0.1:9/none", 0), &ctx).is_some());
        let mut local = CameraSource::url("x", "", 0);
        local.kind = CameraKind::Local(0);
        assert_eq!(CamTest::start(local, &ctx).is_some(), camera::SUPPORTED);
    }
}
