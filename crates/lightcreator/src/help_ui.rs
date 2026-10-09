//! Help: the quick-guide window and the link to the project page.
use crate::app::App;
use crate::guide;
use crate::i18n::tr;
use crate::theme;
use eframe::egui::{self, RichText};

pub const GITHUB_URL: &str = "https://github.com/michalsarna/LightCreator";

impl App {
    pub fn guide_window(&mut self, ctx: &egui::Context) {
        if !self.show_guide {
            return;
        }
        let mut open = true;
        egui::Window::new(tr("Quick guide")).open(&mut open).default_size([640.0, 560.0]).resizable(true).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.guide_filter).hint_text(tr("Search")).desired_width(220.0));
                ui.hyperlink_to(tr("Project page on GitHub"), GITHUB_URL);
            });
            if guide::sections().iter().all(|s| s.title.is_empty()) {
                return;
            }
            ui.add_space(4.0);
            let q = self.guide_filter.to_lowercase();
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                for s in guide::sections() {
                    let hits: Vec<&&str> = s.lines.iter().filter(|l| q.is_empty() || l.to_lowercase().contains(&q) || s.title.to_lowercase().contains(&q)).collect();
                    if hits.is_empty() {
                        continue;
                    }
                    egui::CollapsingHeader::new(RichText::new(s.title).strong().size(15.0)).id_salt(s.title).default_open(!q.is_empty() || s.title == "Getting started" || s.title == "Pierwsze kroki").open(if q.is_empty() { None } else { Some(true) }).show(ui, |ui| {
                        for l in hits {
                            ui.label(*l);
                            ui.add_space(3.0);
                        }
                    });
                }
                ui.add_space(6.0);
                ui.label(RichText::new(tr("This guide is available in English and Polish; other languages show the English text.")).color(theme::text_dim()));
            });
        });
        self.show_guide = open;
    }
}
