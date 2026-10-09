//! Camera / photo overlay: a picture shown under the design, aligned to the work area by four corner handles.
use crate::app::App;
use crate::camera;
use crate::i18n::{tr, trf};
use crate::theme;
use eframe::egui::{self, RichText};
use lc_core::{CameraCfg, Pt};

pub struct Overlay {
    pub tex: egui::TextureHandle,
    pub size: (u32, u32),
    /// Work-area positions (mm) of the picture's top-left, top-right, bottom-right and bottom-left corners.
    pub corners: [Pt; 4],
    pub opacity: f32,
    pub name: String,
}

/// Map the unit square onto the quad with a perspective transform (Heckbert's square-to-quad).
pub fn quad_map(c: &[Pt; 4], u: f64, v: f64) -> Pt {
    let (x0, y0, x1, y1, x2, y2, x3, y3) = (c[0].x, c[0].y, c[1].x, c[1].y, c[2].x, c[2].y, c[3].x, c[3].y);
    let (dx1, dx2, dx3) = (x1 - x2, x3 - x2, x0 - x1 + x2 - x3);
    let (dy1, dy2, dy3) = (y1 - y2, y3 - y2, y0 - y1 + y2 - y3);
    let det = dx1 * dy2 - dy1 * dx2;
    let (g, h) = if dx3.abs() < 1e-12 && dy3.abs() < 1e-12 || det.abs() < 1e-12 { (0.0, 0.0) } else { ((dx3 * dy2 - dy3 * dx2) / det, (dx1 * dy3 - dy1 * dx3) / det) };
    let (a, b, cc) = (x1 - x0 + g * x1, x3 - x0 + h * x3, x0);
    let (d, e, f) = (y1 - y0 + g * y1, y3 - y0 + h * y3, y0);
    let w = g * u + h * v + 1.0;
    Pt::new((a * u + b * v + cc) / w, (d * u + e * v + f) / w)
}

impl App {
    fn bed_corners(&self) -> [Pt; 4] {
        let (w, h) = (self.doc.device.bed_w, self.doc.device.bed_h);
        [Pt::new(0.0, 0.0), Pt::new(w, 0.0), Pt::new(w, h), Pt::new(0.0, h)]
    }

    /// Show `img` as the overlay. A saved calibration for the device is reused.
    pub fn overlay_set_image(&mut self, ctx: &egui::Context, img: egui::ColorImage, name: &str) {
        let size = (img.width() as u32, img.height() as u32);
        if let Some(ov) = &mut self.overlay {
            if ov.size == size {
                ov.tex.set(img, egui::TextureOptions::LINEAR);
                ov.name = name.to_string();
                return;
            }
        }
        let saved = self.doc.device.camera.clone();
        let corners = saved.as_ref().map(|c| c.corners).unwrap_or_else(|| self.bed_corners());
        let opacity = saved.as_ref().map(|c| c.opacity as f32).unwrap_or(0.6);
        let tex = ctx.load_texture("camera-overlay", img, egui::TextureOptions::LINEAR);
        self.overlay = Some(Overlay { tex, size, corners, opacity, name: name.to_string() });
        self.overlay_visible = true;
    }

    pub fn overlay_load_file(&mut self, ctx: &egui::Context) {
        let Some(p) = rfd::FileDialog::new().add_filter("Images", &["png", "jpg", "jpeg", "bmp", "webp"]).pick_file() else { return };
        match image::open(&p) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let ci = egui::ColorImage::from_rgba_unmultiplied([rgba.width() as usize, rgba.height() as usize], rgba.as_raw());
                let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                self.overlay_set_image(ctx, ci, &name);
                self.status = trf("Overlay: {}", &[&name]);
            }
            Err(e) => self.status = trf("Import failed: {}", &[&e]),
        }
    }

    /// Persist the calibration with the device profile.
    pub fn overlay_save_cfg(&mut self) {
        if let Some(ov) = &self.overlay {
            self.doc.device.camera = Some(CameraCfg { corners: ov.corners, opacity: ov.opacity as f64, index: self.cam_index });
            self.sync_profile();
        }
    }

    /// Take frames from the camera thread into the overlay texture.
    pub fn overlay_poll_camera(&mut self, ctx: &egui::Context) {
        let mut latest = None;
        let mut error = None;
        if let Some(w) = &self.cam_worker {
            while let Ok(f) = w.rx.try_recv() {
                match f {
                    Ok(fr) => latest = Some(fr),
                    Err(e) => error = Some(e),
                }
            }
        }
        if let Some(e) = error {
            self.cam_error = e;
            self.cam_worker = None;
        }
        if let Some(fr) = latest {
            let ci = egui::ColorImage::from_rgb([fr.w as usize, fr.h as usize], &fr.rgb);
            self.overlay_set_image(ctx, ci, &tr("Camera"));
        }
    }

    pub fn overlay_window(&mut self, ctx: &egui::Context) {
        if !self.show_overlay {
            return;
        }
        let mut open = true;
        let mut changed = false;
        egui::Window::new(tr("Camera overlay")).open(&mut open).collapsible(false).resizable(false).show(ctx, |ui| {
            ui.label(RichText::new(tr("Show a photo or the live camera under the design to position work on the material.")).color(theme::text_dim()));
            ui.horizontal(|ui| {
                if ui.button(tr("Load picture…")).clicked() {
                    self.overlay_load_file(ctx);
                    changed = true;
                }
                if self.overlay.is_some() && ui.button(tr("Remove")).clicked() {
                    self.overlay = None;
                    self.cam_worker = None;
                }
            });
            ui.separator();
            ui.label(RichText::new(tr("Live camera")).strong());
            if !camera::SUPPORTED {
                ui.label(RichText::new(tr("This build has no camera support. Build with: cargo run --release --features camera")).color(theme::text_dim()));
            } else {
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt("cam_sel")
                        .width(220.0)
                        .selected_text(self.cam_list.get(self.cam_index as usize).cloned().unwrap_or_else(|| tr("No camera").to_string()))
                        .show_ui(ui, |ui| {
                            for (i, n) in self.cam_list.iter().enumerate() {
                                ui.selectable_value(&mut self.cam_index, i as u32, n);
                            }
                        });
                    if ui.button(tr("Find cameras")).clicked() {
                        self.cam_list = camera::list();
                        self.cam_index = self.cam_index.min(self.cam_list.len().saturating_sub(1) as u32);
                    }
                });
                ui.horizontal(|ui| {
                    if self.cam_worker.is_none() {
                        if ui.add_enabled(!self.cam_list.is_empty(), egui::Button::new(tr("Start camera"))).clicked() {
                            self.cam_error.clear();
                            self.cam_worker = Some(camera::spawn(self.cam_index, ctx.clone()));
                        }
                    } else if ui.button(tr("Freeze frame")).clicked() {
                        self.cam_worker = None;
                    }
                });
                if !self.cam_error.is_empty() {
                    ui.label(RichText::new(&self.cam_error).color(egui::Color32::from_rgb(0xc0, 0x30, 0x30)));
                }
            }
            if let Some(ov) = &mut self.overlay {
                ui.separator();
                ui.label(RichText::new(trf("Overlay: {}", &[&ov.name])).strong());
                ui.checkbox(&mut self.overlay_visible, tr("Show overlay"));
                changed |= ui.add(egui::Slider::new(&mut ov.opacity, 0.05..=1.0).text(tr("Opacity"))).changed();
                ui.checkbox(&mut self.overlay_edit, tr("Align: drag the four corners on the work area")).on_hover_text(tr("Drag each corner to the matching corner of the material or work area; perspective is corrected."));
                ui.horizontal_wrapped(|ui| {
                    if ui.button(tr("Reset to work area")).clicked() {
                        let (w, h) = (self.doc.device.bed_w, self.doc.device.bed_h);
                        ov.corners = [Pt::new(0.0, 0.0), Pt::new(w, 0.0), Pt::new(w, h), Pt::new(0.0, h)];
                        changed = true;
                    }
                    if ui.button(tr("Flip H")).clicked() {
                        ov.corners.swap(0, 1);
                        ov.corners.swap(3, 2);
                        changed = true;
                    }
                    if ui.button(tr("Flip V")).clicked() {
                        ov.corners.swap(0, 3);
                        ov.corners.swap(1, 2);
                        changed = true;
                    }
                    if ui.button(tr("Rotate 90°")).clicked() {
                        ov.corners.rotate_right(1);
                        changed = true;
                    }
                });
            }
        });
        if changed {
            self.overlay_save_cfg();
        }
        self.show_overlay = open;
        if !open {
            self.overlay_edit = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quad_map_hits_corners_and_handles_perspective() {
        let c = [Pt::new(10.0, 10.0), Pt::new(110.0, 20.0), Pt::new(100.0, 90.0), Pt::new(0.0, 80.0)];
        for (u, v, k) in [(0.0, 0.0, 0), (1.0, 0.0, 1), (1.0, 1.0, 2), (0.0, 1.0, 3)] {
            assert!(quad_map(&c, u, v).dist(c[k]) < 1e-9, "corner {k}");
        }
        // Axis-aligned rectangle: plain linear interpolation.
        let r = [Pt::new(0.0, 0.0), Pt::new(200.0, 0.0), Pt::new(200.0, 100.0), Pt::new(0.0, 100.0)];
        assert!(quad_map(&r, 0.5, 0.5).dist(Pt::new(100.0, 50.0)) < 1e-9);
        // A trapezoid keeps the centre inside the quad.
        let t = [Pt::new(20.0, 0.0), Pt::new(80.0, 0.0), Pt::new(100.0, 100.0), Pt::new(0.0, 100.0)];
        let m = quad_map(&t, 0.5, 0.5);
        assert!(m.x > 40.0 && m.x < 60.0 && m.y > 0.0 && m.y < 100.0);
    }
}
