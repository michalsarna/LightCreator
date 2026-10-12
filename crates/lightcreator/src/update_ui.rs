//! Looking for a newer release on GitHub: once when the program starts (can be switched off in the program
//! options) and on demand from Help > Check for updates…. The check and its window are in base-ui; this file
//! says which program and repository they are about.
use crate::app::{App, APP_VERSION};
use base_ui::dialogs::UpdateConfig;
use eframe::egui;

pub use base_ui::dialogs::UpdateState;

pub static CONFIG: UpdateConfig = UpdateConfig { app_name: "LightCreator", repo: "michalsarna/LightCreator", version: APP_VERSION };

impl App {
    /// Ask GitHub for the newest release in the background. `manual` shows the answer even when there is nothing new.
    pub fn start_update_check(&mut self, ctx: &egui::Context, manual: bool) {
        self.update.start(ctx, &CONFIG, manual);
    }

    /// Pick up the answer and draw the window about it.
    pub fn update_tick(&mut self, ctx: &egui::Context) {
        self.update.tick(ctx, &CONFIG);
    }
}
