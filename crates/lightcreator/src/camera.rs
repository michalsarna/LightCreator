//! Live camera capture (optional: build with `--features camera`).
use std::sync::mpsc::Receiver;

/// One captured picture: RGB bytes, row-major.
pub struct Frame {
    pub w: u32,
    pub h: u32,
    pub rgb: Vec<u8>,
}

#[cfg(feature = "camera")]
mod imp {
    use super::*;
    use nokhwa::pixel_format::RgbFormat;
    use std::sync::mpsc::{channel, Sender};
    use nokhwa::utils::{ApiBackend, CameraIndex, RequestedFormat, RequestedFormatType};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    pub const SUPPORTED: bool = true;

    /// Names of the cameras the system reports. Asks for permission first on macOS.
    pub fn list() -> Vec<String> {
        nokhwa::nokhwa_initialize(|_| {});
        // Give the permission prompt a moment on first use.
        for _ in 0..20 {
            if nokhwa::nokhwa_check() {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        nokhwa::query(ApiBackend::Auto).map(|v| v.into_iter().map(|c| c.human_name()).collect()).unwrap_or_default()
    }

    pub struct Worker {
        pub rx: Receiver<Result<Frame, String>>,
        stop: Arc<AtomicBool>,
    }

    impl Drop for Worker {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::Relaxed);
        }
    }

    /// Stream frames (about 8 per second) from camera `index` until the worker is dropped.
    pub fn spawn(index: u32, ctx: eframe::egui::Context) -> Worker {
        let (tx, rx): (Sender<Result<Frame, String>>, _) = channel();
        let stop = Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        std::thread::spawn(move || {
            let fmt = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
            let mut cam = match nokhwa::Camera::new(CameraIndex::Index(index), fmt) {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                    return;
                }
            };
            if let Err(e) = cam.open_stream() {
                let _ = tx.send(Err(e.to_string()));
                return;
            }
            while !flag.load(Ordering::Relaxed) {
                match cam.frame().and_then(|f| f.decode_image::<RgbFormat>()) {
                    Ok(img) => {
                        let (w, h) = (img.width(), img.height());
                        if tx.send(Ok(Frame { w, h, rgb: img.into_raw() })).is_err() {
                            break;
                        }
                        ctx.request_repaint();
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e.to_string()));
                        break;
                    }
                }
                std::thread::sleep(Duration::from_millis(120));
            }
            let _ = cam.stop_stream();
        });
        Worker { rx, stop }
    }
}

#[cfg(not(feature = "camera"))]
mod imp {
    use super::*;
    use std::sync::mpsc::channel;

    pub const SUPPORTED: bool = false;

    pub fn list() -> Vec<String> {
        vec![]
    }

    pub struct Worker {
        pub rx: Receiver<Result<Frame, String>>,
    }

    pub fn spawn(_index: u32, _ctx: eframe::egui::Context) -> Worker {
        let (_tx, rx) = channel();
        Worker { rx }
    }
}

pub use imp::{list, spawn, Worker, SUPPORTED};
