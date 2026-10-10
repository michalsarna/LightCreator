//! Saving as the user sees it: a short "file saved" message, automatic saving, the question asked when the
//! program is closed with unsaved work, and the program options window that sets the auto-save interval.
use crate::app::App;
use crate::i18n::{tr, trf};
use crate::theme;
use eframe::egui::{self, Color32, RichText};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// How long the "file saved" message stays on screen.
const TOAST_SECS: u64 = 5;
pub const DEFAULT_AUTOSAVE_SECS: u32 = 120;

/// A temporary file for work that has not been saved under a name yet. Not a fixed name, so two running
/// copies of the program do not overwrite each other's work.
pub fn temp_autosave_path() -> PathBuf {
    std::env::temp_dir().join("lightcreator-autosave").join(format!("untitled-{}.lcr", std::process::id()))
}

impl App {
    /// There is work that is neither saved by hand nor written by the auto-save.
    pub fn is_dirty(&self) -> bool {
        self.revision != self.saved_revision && (!self.doc.shapes.is_empty() || self.path.is_some())
    }

    /// The document now matches the file on disk: forget the temporary copy.
    pub fn mark_saved(&mut self) {
        self.saved_revision = self.revision;
        self.autosaved_revision = self.revision;
        let _ = std::fs::remove_file(temp_autosave_path());
    }

    /// Show `text` for a few seconds.
    pub fn toast(&mut self, text: String) {
        self.toast = Some((text, Instant::now() + Duration::from_secs(TOAST_SECS)));
    }

    /// Write the document now: to its own file when it has one, otherwise to the temporary file. Returns where.
    pub fn autosave_now(&mut self) -> Option<PathBuf> {
        let target = match &self.path {
            Some(p) => p.clone(),
            None => {
                let t = temp_autosave_path();
                let _ = std::fs::create_dir_all(t.parent()?);
                t
            }
        };
        std::fs::write(&target, self.doc.to_json()).ok()?;
        self.autosaved_revision = self.revision;
        if self.path.is_some() {
            // The real file is up to date, so there is nothing left to ask about when closing.
            self.saved_revision = self.revision;
        }
        self.last_autosave = Instant::now();
        Some(target)
    }

    /// Close the program, after asking when there is unsaved work.
    pub fn request_close(&mut self, ctx: &egui::Context) {
        if self.is_dirty() && !self.allow_close {
            self.show_close_dlg = true;
        } else {
            self.allow_close = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    /// Called every frame: auto-save, the close question, the saved message and the options window.
    pub fn session_tick(&mut self, ctx: &egui::Context) {
        // The window's own close button or the system asks to close.
        if ctx.input(|i| i.viewport().close_requested()) && self.is_dirty() && !self.allow_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.show_close_dlg = true;
        }
        // Auto-save.
        if self.autosave_on && self.is_dirty() && self.revision != self.autosaved_revision {
            let every = Duration::from_secs(self.autosave_secs.max(5) as u64);
            if self.last_autosave.elapsed() >= every {
                if let Some(p) = self.autosave_now() {
                    self.status = trf("Auto-saved {}", &[&p.display()]);
                }
            }
            ctx.request_repaint_after(Duration::from_secs(1));
        }
        self.toast_ui(ctx);
        self.close_dialog(ctx);
        self.options_window(ctx);
    }

    fn toast_ui(&mut self, ctx: &egui::Context) {
        let Some((text, until)) = &self.toast else { return };
        let left = until.saturating_duration_since(Instant::now());
        if left.is_zero() {
            self.toast = None;
            return;
        }
        ctx.request_repaint_after(left);
        egui::Area::new(egui::Id::new("saved_toast")).order(egui::Order::Tooltip).anchor(egui::Align2::CENTER_BOTTOM, [0.0, -48.0]).show(ctx, |ui| {
            egui::Frame::popup(ui.style()).fill(Color32::from_rgb(0x2e, 0xa0, 0x4f)).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add(crate::icons::slot(Some("check"), Color32::WHITE));
                    ui.label(RichText::new(text.as_str()).color(Color32::WHITE).strong());
                });
            });
        });
    }

    fn close_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_close_dlg {
            return;
        }
        let mut choice = None;
        egui::Window::new("close_dialog").title_bar(false).collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
            ui.label(RichText::new(tr("Save your work before closing?")).strong());
            ui.label(RichText::new(tr("The design has not been saved to a file.")).color(theme::text_dim()));
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let save = egui::Button::new(RichText::new(tr("Save")).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0x2e, 0xa0, 0x4f));
                if ui.add(save).clicked() {
                    choice = Some(Close::Save);
                }
                let discard = egui::Button::new(RichText::new(tr("Discard")).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0xc0, 0x30, 0x30));
                if ui.add(discard).clicked() {
                    choice = Some(Close::Discard);
                }
                if ui.button(tr("Cancel")).clicked() {
                    choice = Some(Close::Cancel);
                }
            });
        });
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            choice = Some(Close::Cancel);
        }
        match choice {
            Some(Close::Save) => {
                self.save(false);
                // A cancelled file dialog leaves the work unsaved: stay open.
                if !self.is_dirty() {
                    self.show_close_dlg = false;
                    self.allow_close = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
            Some(Close::Discard) => {
                let _ = std::fs::remove_file(temp_autosave_path());
                self.show_close_dlg = false;
                self.allow_close = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            Some(Close::Cancel) => self.show_close_dlg = false,
            None => {}
        }
    }

    fn options_window(&mut self, ctx: &egui::Context) {
        if !self.show_options {
            return;
        }
        let mut open = true;
        egui::Window::new(tr("Program options")).open(&mut open).collapsible(false).resizable(false).show(ctx, |ui| {
            egui::Grid::new("program_options").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
                ui.label(tr("Auto-save"));
                ui.checkbox(&mut self.autosave_on, tr("Save the work automatically")).on_hover_text(tr("Work without a file name goes to a temporary folder; a file with a name is saved in place"));
                ui.end_row();
                ui.label(tr("Auto-save every"));
                ui.add_enabled(self.autosave_on, egui::DragValue::new(&mut self.autosave_secs).range(10..=3600).suffix(" s"));
                ui.end_row();
            });
        });
        self.show_options = open;
    }
}

enum Close {
    Save,
    Discard,
    Cancel,
}

#[cfg(test)]
mod tests {
    use super::*;
    use lc_core::{Kind, Xf};

    #[test]
    fn unsaved_work_goes_to_a_temp_file_and_closing_asks_first() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.screen = crate::app::Screen::Editor;
        // An empty new document is nothing to save.
        a.touch();
        assert!(!a.is_dirty());
        a.doc.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::IDENTITY);
        a.touch();
        assert!(a.is_dirty());
        // Auto-save of work without a name: temporary file, still counted as unsaved.
        let p = a.autosave_now().expect("written");
        assert_eq!(p, temp_autosave_path());
        assert!(p.is_file());
        assert!(a.is_dirty());
        // Closing asks instead of closing.
        a.request_close(&ctx);
        assert!(a.show_close_dlg && !a.allow_close);
        // A manual save forgets the temporary copy and the question.
        a.show_close_dlg = false;
        a.mark_saved();
        assert!(!a.is_dirty() && !p.exists());
        a.request_close(&ctx);
        assert!(!a.show_close_dlg && a.allow_close);
    }

    #[test]
    fn a_named_file_is_autosaved_in_place() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        let dir = std::env::temp_dir().join(format!("lc-autosave-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("work.lcr");
        a.path = Some(file.clone());
        a.doc.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::IDENTITY);
        a.touch();
        assert!(a.is_dirty());
        assert_eq!(a.autosave_now(), Some(file.clone()));
        assert!(file.is_file());
        assert!(!a.is_dirty(), "the real file is up to date");
        a.toast("File saved: work.lcr".into());
        assert!(a.toast.is_some());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
