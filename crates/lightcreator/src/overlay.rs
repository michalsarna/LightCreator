//! The camera / photo overlay of LightCreator. The picture, its window, its corners and drawing are the shared ones
//! (`base_ui::editor::EditorBackground`); here is what is the laser's own: the corners kept with the device, the USB camera that can be
//! chosen in the window, and the project kept in the document.
use crate::app::App;
use crate::camera;
use crate::i18n::tr;
use crate::theme;
use base_ui::editor::EditorBackground;
use eframe::egui::{self, RichText};
use lc_core::CameraCfg;

impl App {
    /// Put the overlay into the project (call it before the project is written, or put away in its tab). An overlay that was just read
    /// from the file and is not on the screen yet is left as it is.
    pub fn overlay_sync_to_doc(&mut self) {
        if self.overlay_reload || self.ed.background_pending.is_some() {
            return;
        }
        let data = self.ed.background.as_ref().and_then(|b| b.data());
        self.doc.set_overlay(data);
    }

    /// Show the overlay of the project that is shown now (none: the overlay goes, unless the camera view feeds it). Called when the project
    /// changed: a new one, an opened file, another tab.
    pub fn overlay_from_doc(&mut self) {
        self.overlay_reload = false;
        self.ed.background_live = false;
        self.ed.background_pending = self.doc.overlay();
        if self.ed.background_pending.is_some() {
            self.ed.background = None;
            self.stream_overlay = false;
        } else if !self.stream_overlay {
            self.ed.background = None;
        }
    }

    /// Persist the corners and the opacity with the device profile.
    pub fn overlay_save_cfg(&mut self) {
        if let Some(b) = &self.ed.background {
            self.doc.device.camera = Some(CameraCfg { corners: b.corners, opacity: b.opacity as f64, index: self.cam_index });
            self.sync_profile();
        }
        // The corners, the opacity and the picture are kept in the project too.
        self.touch();
    }

    /// Every frame: the project that changed, and a USB camera that is no longer asked for stops.
    pub fn overlay_poll_camera(&mut self, ctx: &egui::Context) {
        if self.overlay_reload {
            self.overlay_from_doc();
        }
        self.background_tick(ctx);
        if !self.ed.background_live {
            self.cam_worker = None;
        }
    }

    /// The USB camera runs and its frames are taken; the number of the last one, or `None` when it failed.
    pub fn overlay_live_camera(&mut self, ctx: &egui::Context) -> Option<u64> {
        if !camera::SUPPORTED {
            return None;
        }
        if self.cam_worker.is_none() {
            self.cam_error.clear();
            let yuyv = self.doc.device.cameras.get(self.doc.device.active_camera).is_some_and(|c| c.image.prefer_yuyv);
            self.cam_worker = Some(camera::spawn(self.cam_index, yuyv, ctx.clone()));
        }
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
            return None;
        }
        if let Some(fr) = latest {
            let rgb = self.cam_process(fr.rgb, fr.w, fr.h);
            self.cam_frame = Some((fr.w as usize, fr.h as usize, rgb));
            self.cam_frames += 1;
        }
        Some(self.cam_frames)
    }

    /// The choice of the camera, in the shared window of the overlay.
    pub fn overlay_camera_choice(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.label(RichText::new(tr("Live camera")).strong());
        if !camera::SUPPORTED {
            ui.label(RichText::new(tr("This build has no camera support. Build with: cargo run --release --features camera")).color(theme::text_dim()));
            return;
        }
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("cam_sel").width(220.0).selected_text(self.cam_list.get(self.cam_index as usize).cloned().unwrap_or_else(|| tr("No camera").to_string())).show_ui(ui, |ui| {
                for (i, n) in self.cam_list.iter().enumerate() {
                    ui.selectable_value(&mut self.cam_index, i as u32, n);
                }
            });
            if ui.button(tr("Find cameras")).clicked() {
                self.cam_list = camera::list();
                self.cam_index = self.cam_index.min(self.cam_list.len().saturating_sub(1) as u32);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lc_core::Pt;

    /// An aligned laser stays aligned: corners dragged in an older version are in the profile, and the picture takes them.
    #[test]
    fn the_corners_kept_in_an_older_profile_are_used_and_a_drag_keeps_them() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        let kept = [Pt::new(3.0, 4.0), Pt::new(90.0, 5.0), Pt::new(91.0, 70.0), Pt::new(2.0, 71.0)];
        let old = serde_json::json!({ "corners": kept, "opacity": 0.35 });
        a.doc.device.camera = Some(serde_json::from_value(old).expect("the file of an older version has no camera number"));
        a.set_background(&ctx, egui::ColorImage::from_rgb([4, 2], &[200u8; 24]), "photo");
        let b = a.ed.background.as_mut().expect("the picture");
        assert_eq!((b.corners, b.opacity), (kept, 0.35));
        b.corners[0] = Pt::new(7.0, 8.0);
        b.opacity = 0.5;
        a.overlay_save_cfg();
        let saved = a.doc.device.camera.as_ref().unwrap();
        assert_eq!((saved.corners[0], saved.opacity), (Pt::new(7.0, 8.0), 0.5));
        // Removing the picture keeps what the device knows.
        a.ed.background = None;
        a.overlay_save_cfg();
        assert_eq!(a.doc.device.camera.as_ref().unwrap().corners[0], Pt::new(7.0, 8.0));
    }

    /// The hooks of the window: it is a camera overlay even with no camera set up, and with a picture from the USB camera.
    #[test]
    fn the_window_is_the_one_of_a_camera_overlay_and_the_usb_frame_goes_under_the_drawing() {
        use base_ui::editor::EditorHost;
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        assert!(a.overlay_is_camera());
        assert!(a.background_camera_image().is_none(), "no frame yet");
        a.cam_frame = Some((2, 1, vec![9u8; 6]));
        let (img, _) = a.background_camera_image().expect("the frame");
        assert_eq!((img.width(), img.height()), (2, 1));
    }
}
