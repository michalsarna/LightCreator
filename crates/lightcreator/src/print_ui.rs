//! File > Print…: the window is in baseCreator (`base_ui::print_ui`), the same in all the programs.
use crate::app::App;
pub use base_ui::print_ui::PrintDlg;
use base_ui::print_ui::window;
use eframe::egui;

impl App {
    pub fn open_print(&mut self) {
        self.print = Some(PrintDlg::open());
    }

    pub fn print_window(&mut self, ctx: &egui::Context) {
        let mut d = self.print.take();
        if d.is_some() {
            window(ctx, &mut d, &lc_core::print::source(&self.doc));
        }
        self.print = d;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_dialog_opens_and_stays_open() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.enter_editor(0);
        a.open_print();
        let mut out = ctx.run_ui(egui::RawInput::default(), |_| {});
        a.print_window(&ctx);
        out.textures_delta.clear();
        assert!(a.print.is_some(), "stays open");
    }
}
