//! The clipart gallery: the tab, the dialogs and the online search are in base-ui; the pictures that ship with
//! the programs are in base-ui too.
use crate::app::App;
use lc_core::clipart::Library;

pub use base_ui::clipart_ui::{ClipHost, ClipState};
#[cfg(test)]
pub use base_ui::clipart_ui::BUILTIN_CLIPART;
#[cfg(test)]
pub use base_ui::clipart_ui::OnlineDlg;

/// The state of the gallery over a library folder, with the pictures that ship with the programs.
pub fn new_state(lib: Library) -> ClipState {
    ClipState::new(lib)
}

impl ClipHost for App {
    fn clip(&self) -> &ClipState {
        &self.clip
    }
    fn clip_mut(&mut self) -> &mut ClipState {
        &mut self.clip
    }
}
