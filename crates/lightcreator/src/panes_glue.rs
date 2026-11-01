//! The central area in panes (the machinery is in `base_ui::work_panes`): the active drawing, another open drawing to look
//! at, the preview, the analysis of a G-code file and the camera. With one pane (the default) the area is drawn as before.
use crate::app::App;
use base_ui::editor::{show_other_doc, EditorCanvas, Look};
use base_ui::work_panes::{content_name, Content, PaneEvent, PaneHost};
use eframe::egui;

impl App {
    /// The slots of all open drawings (the shown one included).
    fn pane_docs(&self) -> Vec<usize> {
        let mut v: Vec<usize> = self.tabs_state.parked.iter().flatten().map(|w| w.slot).collect();
        v.push(self.slot);
        v
    }

    /// The central area: the panes.
    pub fn central_panes(&mut self, ui: &mut egui::Ui) {
        let mut panes = std::mem::take(&mut self.panes);
        panes.normalize(&self.pane_docs(), self.slot, self.camera_work_tab());
        let event = panes.show(self, ui);
        self.panes = panes;
        self.pane_event(event);
    }

    /// The windows of the panes that were taken out (drawn outside the panels).
    pub fn pane_windows(&mut self, ctx: &egui::Context) {
        if self.panes.windows.is_empty() {
            return;
        }
        let mut panes = std::mem::take(&mut self.panes);
        panes.normalize(&self.pane_docs(), self.slot, self.camera_work_tab());
        let event = panes.show_windows(self, ctx);
        self.panes = panes;
        self.pane_event(event);
    }

    /// A click in a pane with another drawing: that drawing becomes the active one, and the one that was active takes its place.
    pub fn pane_event(&mut self, event: PaneEvent) {
        if let PaneEvent::Activate(slot) = event {
            if let Some(i) = self.tabs_state.parked.iter().position(|w| w.as_ref().is_some_and(|w| w.slot == slot)) {
                self.switch_to(i);
            }
        }
    }

    /// "Show beside" in the menu of the tab `i`.
    pub fn show_tab_beside(&mut self, i: usize) {
        if let Some(slot) = self.tabs_state.parked.get(i).and_then(|w| w.as_ref()).map(|w| w.slot) {
            if self.panes.is_single() {
                self.panes.dir = base_ui::work_panes::Dir::Beside;
            }
            if !self.panes.split(Content::Doc(slot)) {
                self.status = base_ui::i18n::tr("The view has three panes at most").to_string();
            }
        }
    }
}

impl PaneHost for App {
    fn pane_choices(&self) -> Vec<(Content, String)> {
        let mut v = vec![(Content::Active, content_name(Content::Active))];
        for w in self.tabs_state.parked.iter().flatten() {
            v.push((Content::Doc(w.slot), base_ui::doc_tabs::title_of(w.path.as_deref())));
        }
        v.push((Content::Preview, content_name(Content::Preview)));
        v.push((Content::Gcode, content_name(Content::Gcode)));
        if self.camera_work_tab() {
            v.push((Content::Camera, content_name(Content::Camera)));
        }
        v
    }

    fn draw_pane(&mut self, ui: &mut egui::Ui, content: Content, view: &mut Option<base_ui::editor::View>) -> PaneEvent {
        match content {
            Content::Active => {
                let panes = &self.panes;
                let (cam, pv, gv) = (panes.shows(Content::Camera), panes.shows(Content::Preview), panes.shows(Content::Gcode));
                if !cam && self.cam_tab_shown && self.camera_work_tab() {
                    self.camera_work_panel(ui);
                } else if !pv && self.pv_tab_shown && self.preview_work_tab() {
                    self.preview_work_panel(ui);
                } else if gv || !self.gcode_view_panel(ui) {
                    self.canvas(ui)
                }
            }
            Content::Doc(slot) => {
                let Some(w) = self.tabs_state.parked.iter().flatten().find(|w| w.slot == slot) else { return PaneEvent::Nothing };
                let colour = |l: usize| w.doc.layers.get(l).and_then(|l| l.visible.then(|| crate::canvas::col(l.color)));
                let look = Look { workspace: self.ed.grid_prefs.workspace_fill(), bed: self.ed.grid_prefs.bed_fill(self.scheme.is_dark()), layer_colour: &colour };
                if show_other_doc(ui, &w.doc.scene, w.doc.device.work_size(), &look, view) {
                    return PaneEvent::Activate(slot);
                }
            }
            Content::Preview => {
                if self.preview_work_tab() {
                    if let Some(job) = self.preview_prepare(ui.ctx()) {
                        egui::Frame::new().inner_margin(egui::Margin::same(6)).show(ui, |ui| self.preview_body(ui, &job));
                        self.pv.job = Some((self.job_stamp(), job));
                    }
                } else {
                    ui.add_space(8.0);
                    if ui.button(base_ui::i18n::tr("Show the preview here")).clicked() {
                        self.show_preview = true;
                        self.preview_pin(true);
                        self.pv_tab_shown = false;
                    }
                }
            }
            Content::Gcode => {
                if !self.gcode_view_panel(ui) {
                    ui.add_space(8.0);
                    ui.label(base_ui::i18n::tr("No G-code file is open: File → View G-code file…"));
                }
            }
            Content::Camera => {
                if self.camera_work_tab() {
                    self.camera_work_panel(ui);
                }
            }
        }
        PaneEvent::Nothing
    }

    fn can_pop_out(&self, content: Content) -> bool {
        matches!(content, Content::Doc(_) | Content::Preview | Content::Gcode | Content::Camera)
    }
}
