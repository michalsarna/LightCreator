//! The picture of the overlay (a photo or a frame of the camera, shown under the design) as the project file keeps it: the picture as a
//! JPEG, where its corners are on the work area, how see-through it is, and whether it is shown. It is a section of the project
//! (`overlay`), so the file brings the picture back to the machine and the person that opens it.
use crate::geom::Pt;
use crate::Document;
use base64::Engine;
use serde::{Deserialize, Serialize};

/// The name of the section of the project file.
pub const OVERLAY_SECTION: &str = "overlay";
const FORMAT: u32 = 1;
/// The quality of the JPEG that is kept: a photo to look at while placing work, not an archive.
const JPEG_QUALITY: u8 = 88;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OverlayData {
    pub format: u32,
    pub name: String,
    /// Work-area positions (mm) of the picture's top-left, top-right, bottom-right and bottom-left corners.
    pub corners: [Pt; 4],
    pub opacity: f64,
    /// Is it shown on the work area?
    pub visible: bool,
    /// The picture: a JPEG in base64.
    pub jpeg: String,
}

impl OverlayData {
    /// The data of a picture given as RGB bytes (`w * h * 3`).
    pub fn new(name: &str, corners: [Pt; 4], opacity: f64, visible: bool, rgb: &[u8], w: u32, h: u32) -> Result<OverlayData, String> {
        if w == 0 || h == 0 || rgb.len() != (w as usize) * (h as usize) * 3 {
            return Err("the picture of the overlay has no size".into());
        }
        let mut out = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, JPEG_QUALITY).encode(rgb, w, h, image::ExtendedColorType::Rgb8).map_err(|e| e.to_string())?;
        Ok(OverlayData { format: FORMAT, name: name.to_string(), corners, opacity, visible, jpeg: base64::engine::general_purpose::STANDARD.encode(out) })
    }

    /// The picture as RGBA bytes with its size.
    pub fn decode(&self) -> Result<(u32, u32, Vec<u8>), String> {
        let bytes = base64::engine::general_purpose::STANDARD.decode(self.jpeg.as_bytes()).map_err(|e| e.to_string())?;
        let img = image::load_from_memory_with_format(&bytes, image::ImageFormat::Jpeg).map_err(|e| e.to_string())?.to_rgba8();
        Ok((img.width(), img.height(), img.into_raw()))
    }

    /// Does this program read it (a newer file may have another format)?
    pub fn is_readable(&self) -> bool {
        self.format <= FORMAT && self.corners.iter().all(|c| c.x.is_finite() && c.y.is_finite()) && self.opacity.is_finite()
    }
}

impl Document {
    /// The overlay kept in this project, when there is one this program can read.
    pub fn overlay(&self) -> Option<OverlayData> {
        let v = self.others.sections.get(OVERLAY_SECTION)?;
        serde_json::from_value::<OverlayData>(v.clone()).ok().filter(OverlayData::is_readable)
    }

    /// Keep an overlay in the project (or none).
    pub fn set_overlay(&mut self, data: Option<OverlayData>) {
        match data.and_then(|d| serde_json::to_value(d).ok()) {
            Some(v) => {
                self.others.sections.insert(OVERLAY_SECTION.to_string(), v);
            }
            None => {
                self.others.sections.remove(OVERLAY_SECTION);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgb(w: u32, h: u32) -> Vec<u8> {
        (0..w * h * 3).map(|i| (i % 251) as u8).collect()
    }

    #[test]
    fn a_picture_goes_into_the_project_file_and_comes_back_with_its_corners() {
        let corners = [Pt::new(1.0, 2.0), Pt::new(300.0, 3.0), Pt::new(298.0, 200.0), Pt::new(0.5, 199.0)];
        let data = OverlayData::new("photo.jpg", corners, 0.4, true, &rgb(64, 48), 64, 48).unwrap();
        let mut d = Document::default();
        assert!(d.overlay().is_none(), "none to begin with");
        d.set_overlay(Some(data.clone()));
        // Through the file, as a person's save and open do it.
        let back = Document::from_json(&d.to_json()).unwrap();
        let got = back.overlay().expect("the overlay is in the file");
        assert_eq!((got.name.as_str(), got.corners, got.opacity, got.visible), ("photo.jpg", corners, 0.4, true));
        let (w, h, px) = got.decode().unwrap();
        assert_eq!((w, h, px.len()), (64, 48, 64 * 48 * 4));
        // A project that has none stays as it was: no section at all.
        d.set_overlay(None);
        assert!(!d.to_json().contains("\"overlay\""));
        assert!(Document::from_json(&d.to_json()).unwrap().overlay().is_none());
    }

    #[test]
    fn a_picture_without_size_or_a_newer_format_or_broken_data_is_not_taken() {
        assert!(OverlayData::new("x", [Pt::default(); 4], 0.5, true, &[1, 2, 3], 4, 4).is_err());
        assert!(OverlayData::new("x", [Pt::default(); 4], 0.5, true, &[], 0, 0).is_err());
        let mut data = OverlayData::new("x", [Pt::default(); 4], 0.5, true, &rgb(8, 8), 8, 8).unwrap();
        data.jpeg = "not base64!!".into();
        assert!(data.decode().is_err());
        let mut d = Document::default();
        let mut newer = OverlayData::new("x", [Pt::default(); 4], 0.5, true, &rgb(8, 8), 8, 8).unwrap();
        newer.format = 99;
        d.set_overlay(Some(newer));
        assert!(d.overlay().is_none(), "a format this program does not know is left alone");
        assert!(d.to_json().contains("\"overlay\""), "and is kept for the next save");
    }
}
