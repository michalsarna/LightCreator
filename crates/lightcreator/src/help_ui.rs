//! Help: the quick-guide window and the link to the project page.
use crate::app::App;
use crate::guide;
use crate::i18n::tr;
use eframe::egui;

pub const GITHUB_URL: &str = "https://github.com/michalsarna/LightCreator";

impl App {
    pub fn guide_window(&mut self, ctx: &egui::Context) {
        let footer = tr("This guide is available in English and Polish; other languages show the English text.");
        base_ui::dialogs::guide_window(ctx, &mut self.show_guide, &mut self.guide_filter, guide::sections(), &["Getting started", "Pierwsze kroki"], GITHUB_URL, footer);
    }
}
