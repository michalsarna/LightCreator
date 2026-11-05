//! The "Test" button of a camera in the device settings: the window is the shared one.
use crate::app::App;
use base_ui::camera_feed::CamTest;
use eframe::egui;
use lc_core::CameraSource;

impl App {
    /// Open (or restart) the test window for `cam`. Does nothing for a camera that cannot be opened.
    pub fn start_cam_test(&mut self, ctx: &egui::Context, cam: CameraSource) {
        self.cam_test = CamTest::start(cam, ctx);
    }

    pub fn cam_test_window(&mut self, ctx: &egui::Context) {
        CamTest::window(ctx, &mut self.cam_test);
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
    }
}
