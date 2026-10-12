//! The clipart gallery: the tab, the dialogs and the online search are in base-ui; the pictures that ship with
//! the program are embedded here.
use crate::app::App;
use lc_core::clipart::Library;

pub use base_ui::clipart_ui::{ClipHost, ClipState};
#[cfg(test)]
pub use base_ui::clipart_ui::OnlineDlg;

include!(concat!(env!("OUT_DIR"), "/clipart_files.rs"));

/// The state of the gallery over a library folder, with the pictures that ship with the program.
pub fn new_state(lib: Library) -> ClipState {
    ClipState::new(lib, BUILTIN_CLIPART)
}

impl ClipHost for App {
    fn clip(&self) -> &ClipState {
        &self.clip
    }
    fn clip_mut(&mut self) -> &mut ClipState {
        &mut self.clip
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lc_core::{Document, Pt};
    use std::collections::HashSet;

    #[test]
    fn every_shipped_picture_places_cleanly() {
        assert!(BUILTIN_CLIPART.len() >= 300, "{} pictures", BUILTIN_CLIPART.len());
        let cats: HashSet<&str> = BUILTIN_CLIPART.iter().map(|c| c.0).collect();
        assert_eq!(cats.len(), 9, "{cats:?}");
        for (cat, name, svg) in BUILTIN_CLIPART {
            let mut d = Document::default();
            let ids = lc_core::clipart::import_clip(svg, &mut d, 2, 40.0, Pt::new(100.0, 100.0)).unwrap_or_else(|e| panic!("{cat}/{name}: {e}"));
            let b = d.bounds_of(&ids).unwrap();
            assert!((b.width().max(b.height()) - 40.0).abs() < 1e-6, "{cat}/{name}");
        }
    }
}
