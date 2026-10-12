//! The window frame and the buttons come from base-ui; this file adds the title of LightCreator's window.
use crate::app::{App, APP_VERSION};
use eframe::egui;

pub use base_ui::window::*;

impl App {
    /// The bar across the top of the window.
    pub fn title_bar(&mut self, ui: &mut egui::Ui) {
        let mut title = format!("LightCreator {APP_VERSION}");
        if let Some(name) = self.path.as_ref().and_then(|p| p.file_name()) {
            title.push_str(&format!("  —  {}", name.to_string_lossy()));
        }
        if base_ui::window::title_bar(ui, &title, self.logo.as_ref()) {
            let ctx = ui.ctx().clone();
            self.request_close(&ctx);
        }
    }

    /// Resizing by the window edges, which the missing system frame no longer offers.
    pub fn window_edges(&mut self, ctx: &egui::Context) {
        base_ui::window::window_edges(ctx);
    }

    /// Continue a resize started on the right or bottom edge (macOS).
    #[cfg(target_os = "macos")]
    pub fn window_edge_drag(&mut self, ctx: &egui::Context) {
        base_ui::window::window_edge_drag(ctx);
    }
}
