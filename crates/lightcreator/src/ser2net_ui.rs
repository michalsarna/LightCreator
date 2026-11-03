//! "Share over network (ser2net)": the window is in baseCreator (`base_ui::ser2net_ui`), the same in all the programs.
use crate::app::App;
use eframe::egui;
pub use base_ui::ser2net_ui::Ser2NetDlg;

impl App {
    /// Close the sharing dialog (used by tests and screenshots).
    #[cfg(test)]
    pub fn show_ser2net_off(&mut self) {
        self.ser2net = None;
    }

    pub fn open_ser2net(&mut self) {
        self.refresh_ports();
        let c = &self.doc.device.connection;
        self.ser2net = Some(Ser2NetDlg::open(&c.port, c.baud, c.tcp_port, &self.ports));
    }

    pub fn ser2net_window(&mut self, ctx: &egui::Context) {
        let ports = self.ports.clone();
        if base_ui::ser2net_ui::window(ctx, &mut self.ser2net, &ports) {
            self.refresh_ports();
        }
    }
}
