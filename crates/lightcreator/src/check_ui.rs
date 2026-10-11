//! The check before a job is sent, framed or exported: errors stop it, warnings can be confirmed.
use crate::app::App;
use crate::i18n::{tr, trf};
use crate::theme;
use eframe::egui::{self, Color32, RichText};
use lc_core::check::{self, Issue, IssueKind, Level};

/// What was asked for while the check window is open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pending {
    Start,
    Frame,
    ExportGcode,
    ExportCam,
}

pub struct CheckDlg {
    pub issues: Vec<Issue>,
    pub pending: Pending,
}

/// The text of an issue in the language of the interface; `layers` names the layers of the design.
pub fn describe(i: &Issue, layers: &[String], frame: bool) -> String {
    let layer = i.layer.and_then(|l| layers.get(l)).cloned().unwrap_or_default();
    match i.kind {
        IssueKind::OutsideArea if frame => tr("The frame is outside the usable area (beyond the work area or in a margin).").to_string(),
        IssueKind::OutsideLimits if frame => tr("The frame is outside the software limits.").to_string(),
        IssueKind::OutsideArea => trf("{} object(s) are outside the usable area (beyond the work area or in a margin).", &[&i.shapes.len()]),
        IssueKind::OutsideLimits => trf("{} object(s) are outside the software limits.", &[&i.shapes.len()]),
        IssueKind::MoveOutside => trf("The toolpath of layer {} leaves the allowed area (overscan or travel moves).", &[&layer]),
        IssueKind::ZeroPower => trf("Layer {} burns at 0 % power.", &[&layer]),
        IssueKind::NothingToBurn => trf("Layer {} has objects but no toolpath (open lines on a fill layer, for example).", &[&layer]),
    }
}

impl App {
    fn layer_names(&self) -> Vec<String> {
        self.doc.layers.iter().map(|l| l.name.clone()).collect()
    }

    /// Check the job for `what`. `true` means go ahead now; otherwise the window with the issues is open.
    pub fn guard(&mut self, what: Pending, frame_bounds: Option<lc_core::Rect>) -> bool {
        let issues = match (what, frame_bounds) {
            (Pending::Frame, Some(b)) => check::check_frame(&self.doc.device, b),
            _ => check::job_check(&self.doc),
        };
        if issues.is_empty() {
            return true;
        }
        self.check_dlg = Some(CheckDlg { issues, pending: what });
        false
    }

    pub fn check_window(&mut self, ctx: &egui::Context) {
        let Some(dlg) = &self.check_dlg else { return };
        let names = self.layer_names();
        let frame = dlg.pending == Pending::Frame;
        let blocked = check::has_error(&dlg.issues);
        let mut open = true;
        let mut go = false;
        let mut close = false;
        let mut select: Option<Vec<u64>> = None;
        crate::window_ui::popup(ctx, "job_check", &tr("Check before the job"), [520.0, 260.0], &mut open, |ui| {
            for i in &dlg.issues {
                ui.horizontal_wrapped(|ui| {
                    let (icon, colour) = match i.level {
                        Level::Error => ("✖", Color32::from_rgb(0xc0, 0x30, 0x30)),
                        Level::Warning => ("⚠", Color32::from_rgb(0xe8, 0x8a, 0x1f)),
                    };
                    ui.label(RichText::new(icon).color(colour).strong());
                    ui.label(describe(i, &names, frame));
                    if !i.shapes.is_empty() && ui.small_button(tr("Select objects")).clicked() {
                        select = Some(i.shapes.clone());
                    }
                });
            }
            ui.add_space(8.0);
            if blocked {
                ui.label(RichText::new(tr("Fix the errors first: nothing is sent, framed or exported with them.")).color(theme::text_dim()));
            }
            ui.horizontal(|ui| {
                if !blocked {
                    let b = egui::Button::new(RichText::new(tr("Continue anyway")).color(Color32::WHITE).strong()).fill(Color32::from_rgb(0x2e, 0xa0, 0x4f));
                    if ui.add(b).clicked() {
                        go = true;
                    }
                }
                if ui.button(tr("Close")).clicked() {
                    close = true;
                }
            });
        });
        if let Some(ids) = select {
            self.sel = ids;
        }
        if go {
            if let Some(d) = self.check_dlg.take() {
                match d.pending {
                    Pending::Start => self.send_job_now(),
                    Pending::Frame => self.frame_now(),
                    Pending::ExportGcode => self.export_gcode_now(),
                    Pending::ExportCam => self.export_cam_now(),
                }
            }
        } else if !open || close {
            self.check_dlg = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lc_core::{Kind, Xf};

    fn app_with_rect(x: f64) -> App {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.screen = crate::app::Screen::Editor;
        a.profiles = vec![lc_core::Device::default()];
        a.enter_editor(0);
        a.doc.add(0, Kind::Rect { w: 20.0, h: 10.0 }, Xf::translate(x, 100.0));
        a
    }

    #[test]
    fn start_frame_and_export_are_blocked_by_an_object_outside_the_area() {
        // The default work area is 400 mm wide; the rectangle ends at 410 mm.
        let mut a = app_with_rect(390.0);
        a.connected = true;
        a.send_job();
        let dlg = a.check_dlg.take().expect("the check window opens");
        assert_eq!((dlg.pending, dlg.issues[0].kind), (Pending::Start, IssueKind::OutsideArea));
        a.select_all();
        a.frame();
        let dlg = a.check_dlg.take().expect("frame is checked too");
        assert_eq!(dlg.pending, Pending::Frame);
        assert!(!a.guard(Pending::ExportGcode, None));
        assert!(check::has_error(&a.check_dlg.as_ref().unwrap().issues));
    }

    #[test]
    fn a_job_inside_the_area_goes_ahead() {
        let mut a = app_with_rect(100.0);
        assert!(a.guard(Pending::ExportGcode, None));
        assert!(a.check_dlg.is_none());
    }

    #[test]
    fn issues_are_described_in_words() {
        let names: Vec<String> = (0..30).map(|i| format!("C{i:02}")).collect();
        let i = Issue { level: Level::Warning, kind: IssueKind::ZeroPower, shapes: vec![1], layer: Some(3) };
        assert!(describe(&i, &names, false).contains("C03"));
    }
}
