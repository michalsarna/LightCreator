//! The overlay as a section of the project of LightCreator. The data itself is in baseCreator (`base_core::overlay_data`), the same in all
//! the programs, so a project brings its picture along to any of them.
use crate::Document;
pub use base_core::overlay_data::{OverlayData, OVERLAY_SECTION};

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
    use crate::geom::Pt;

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
