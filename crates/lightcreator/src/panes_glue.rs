//! The central area in panes (the machinery is in `base_ui::work_panes`): every pane has a bar of tabs; a tab is a drawing, the
//! preview, the G-code of the project as text, the analysis of a G-code file or the camera. Nothing is opened twice.
use crate::app::App;
use base_ui::editor::{show_other_doc, EditorCanvas, Look};
use base_ui::gcode_text_ui::{GcodeText, TextAction};
use base_ui::work_panes::{Content, PaneEvent, PaneHost};
use eframe::egui;

impl App {
    /// The slots of all open drawings, in the order of their tabs.
    fn doc_slots(&self) -> Vec<usize> {
        (0..self.tab_count()).map(|i| if i == self.tabs_state.active || self.tabs_state.parked.len() <= 1 { self.slot } else { self.tabs_state.parked[i].as_ref().map_or(self.slot, |w| w.slot) }).collect()
    }

    /// Every tab there is: the drawings, the preview, the camera, the analysis of a G-code file, the G-code texts of the projects.
    fn pane_tabs(&mut self) -> Vec<Content> {
        let slots = self.doc_slots();
        // The G-code of a project that is closed goes with it.
        self.gcode_texts.retain(|g| slots.contains(&g.slot));
        let mut v: Vec<Content> = slots.iter().map(|s| Content::Doc(*s)).collect();
        if self.preview_work_tab() {
            v.push(Content::Preview);
        }
        for g in &self.gcode_texts {
            v.push(Content::GcodeText(g.slot));
        }
        if self.gcode_view.is_some() {
            v.push(Content::Gcode);
        }
        if self.camera_work_tab() {
            v.push(Content::Camera);
        }
        v
    }

    /// The central area: the panes.
    pub fn central_panes(&mut self, ui: &mut egui::Ui) {
        let mut panes = std::mem::take(&mut self.panes);
        panes.normalize(&self.pane_tabs(), self.slot);
        let event = panes.show(self, ui);
        self.panes = panes;
        self.pane_event(event);
    }

    /// The windows of the tabs that were taken out (drawn outside the panels).
    pub fn pane_windows(&mut self, ctx: &egui::Context) {
        if self.panes.windows.is_empty() {
            return;
        }
        let mut panes = std::mem::take(&mut self.panes);
        panes.normalize(&self.pane_tabs(), self.slot);
        let event = panes.show_windows(self, ctx);
        self.panes = panes;
        self.pane_event(event);
    }

    /// What was done in the panes: a drawing clicked becomes the active one, a cross closes, the "+" makes a new drawing.
    pub fn pane_event(&mut self, event: PaneEvent) {
        match event {
            PaneEvent::Activate(slot) => self.activate_slot(slot),
            PaneEvent::Close(c) => self.close_content(c),
            PaneEvent::NewTab(_) => {
                self.new_tab();
            }
            PaneEvent::Nothing => {}
        }
    }

    fn slot_index(&self, slot: usize) -> Option<usize> {
        self.doc_slots().iter().position(|s| *s == slot)
    }

    /// Make the drawing with this recovery slot the active one.
    pub fn activate_slot(&mut self, slot: usize) {
        if slot != self.slot {
            if let Some(i) = self.slot_index(slot) {
                self.switch_to(i);
            }
        }
    }

    /// Close a tab: a drawing (it asks about unsaved work), the preview, the camera, the analysis, the G-code text (it asks when it
    /// has edits).
    pub fn close_content(&mut self, c: Content) {
        match c {
            Content::Doc(slot) => {
                self.activate_slot(slot);
                self.close_tab_request();
            }
            Content::Preview => self.preview_close(),
            Content::Camera => self.stream_closed(),
            Content::Gcode => self.gcode_view = None,
            Content::GcodeText(slot) => {
                if self.gcode_texts.iter().any(|g| g.slot == slot && g.dirty()) {
                    self.closing_text = Some(slot);
                } else {
                    self.gcode_texts.retain(|g| g.slot != slot);
                }
            }
        }
    }

    /// The question about the G-code text with edits whose tab is being closed.
    pub fn closing_text_dialog(&mut self, ctx: &egui::Context) {
        let Some(slot) = self.closing_text else { return };
        let Some(i) = self.gcode_texts.iter().position(|g| g.slot == slot) else {
            self.closing_text = None;
            return;
        };
        let title = self.gcode_texts[i].name.clone();
        match base_ui::doc_tabs::ask_save(ctx, &title, crate::window_ui::SAVE_GREEN) {
            Some(base_ui::doc_tabs::Answer::Save) => match self.gcode_texts[i].save_dialog() {
                Some(Ok(p)) => {
                    self.status = base_ui::i18n::trf("Saved {}", &[&p.display()]);
                    self.gcode_texts.remove(i);
                    self.closing_text = None;
                }
                Some(Err(e)) => self.status = base_ui::i18n::trf("Could not save: {}", &[&e]),
                None => {}
            },
            Some(base_ui::doc_tabs::Answer::Discard) => {
                self.gcode_texts.remove(i);
                self.closing_text = None;
            }
            Some(base_ui::doc_tabs::Answer::Cancel) => self.closing_text = None,
            None => {}
        }
    }

    /// What the G-code of the active project depends on: its saves and its path (the name), the notes, the device (the
    /// G-code settings), the postprocessors that are on. The G-code text tab is made again when this changes.
    pub(crate) fn gcode_stamp(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        (self.saved_revision, &self.path, &self.doc.description, &self.plugins.posts).hash(&mut h);
        serde_json::to_string(&self.doc.device).unwrap_or_default().hash(&mut h);
        h.finish()
    }

    /// The G-code text of the active project follows the project: saving the file, a new note, a change of a G-code setting or of a
    /// postprocessor makes it again, a moment after the last change and as long as the person has not edited it (the edits are never
    /// thrown away on their own). Not every edit of the drawing: the program is not made at every drag.
    pub(crate) fn refresh_gcode_text(&mut self, ctx: &egui::Context) {
        if self.refresh_gcode_text_after(std::time::Duration::from_millis(500)) {
            ctx.request_repaint_after(std::time::Duration::from_millis(600));
        }
    }

    /// [`refresh_gcode_text`] with the wait given; returns whether a change is still waiting for its moment.
    pub(crate) fn refresh_gcode_text_after(&mut self, wait: std::time::Duration) -> bool {
        let slot = self.slot;
        if !self.gcode_texts.iter().any(|g| g.slot == slot) {
            return false;
        }
        let stamp = self.gcode_stamp();
        let Some(g) = self.gcode_texts.iter_mut().find(|g| g.slot == slot) else { return false };
        if !g.due(stamp, wait) {
            return g.stamp != stamp && !g.dirty();
        }
        // One try for every change, whatever comes of it (a postprocessor that fails says so in the status line).
        g.stamp = stamp;
        self.remake_gcode_text(slot);
        false
    }

    /// Make the G-code text of the project of `slot` again (the project becomes the active one first: the program is made from it).
    fn remake_gcode_text(&mut self, slot: usize) {
        self.activate_slot(slot);
        if let Some(text) = self.gcode_text_now() {
            if let Some(g) = self.gcode_texts.iter_mut().find(|g| g.slot == slot) {
                g.replace(text);
            }
        }
        let stamp = self.gcode_stamp();
        if let Some(g) = self.gcode_texts.iter_mut().find(|g| g.slot == slot && !g.dirty()) {
            g.stamp = stamp;
        }
    }
}

impl PaneHost for App {
    fn tab_title(&self, c: Content) -> String {
        use base_ui::i18n::tr;
        match c {
            Content::Doc(slot) => {
                if slot == self.slot {
                    base_ui::doc_tabs::title_of(self.path.as_deref())
                } else {
                    base_ui::doc_tabs::title_of(self.tabs_state.parked.iter().flatten().find(|w| w.slot == slot).and_then(|w| w.path.as_deref()))
                }
            }
            Content::Preview => format!("{}: {}", tr("Preview"), base_ui::doc_tabs::title_of(self.path.as_deref())),
            Content::GcodeText(slot) => format!("{}: {}", tr("Preview G-code"), self.gcode_texts.iter().find(|g| g.slot == slot).map(|g| g.name.clone()).unwrap_or_default()),
            Content::Gcode => self.gcode_view.as_ref().and_then(|v| v.path.file_name()).map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| tr("G-code analysis").to_string()),
            Content::Camera => tr("Camera view").to_string(),
        }
    }

    fn tab_dirty(&self, c: Content) -> bool {
        match c {
            Content::Doc(slot) => {
                if slot == self.slot {
                    self.is_dirty()
                } else {
                    self.tabs_state.parked.iter().flatten().find(|w| w.slot == slot).is_some_and(|w| w.dirty())
                }
            }
            Content::GcodeText(slot) => self.gcode_texts.iter().any(|g| g.slot == slot && g.dirty()),
            _ => false,
        }
    }

    fn draw_pane(&mut self, ui: &mut egui::Ui, content: Content, view: &mut Option<base_ui::editor::View>) -> PaneEvent {
        match content {
            Content::Doc(slot) if slot == self.slot => self.canvas(ui),
            Content::Doc(slot) => {
                let Some(w) = self.tabs_state.parked.iter().flatten().find(|w| w.slot == slot) else { return PaneEvent::Nothing };
                let colour = |l: usize| w.doc.layers.get(l).and_then(|l| l.visible.then(|| crate::canvas::col(l.color)));
                let look = Look { workspace: self.ed.grid_prefs.workspace_fill(), bed: self.ed.grid_prefs.bed_fill(self.scheme.is_dark()), layer_colour: &colour };
                if show_other_doc(ui, &w.doc.scene, w.doc.device.work_size(), &look, view) {
                    return PaneEvent::Activate(slot);
                }
            }
            Content::Preview => {
                if self.show_preview {
                    self.preview_work_panel(ui);
                }
            }
            Content::GcodeText(slot) => {
                let Some(i) = self.gcode_texts.iter().position(|g| g.slot == slot) else { return PaneEvent::Nothing };
                match base_ui::gcode_text_ui::show(ui, &mut self.gcode_texts[i]) {
                    TextAction::MakeAgain => self.remake_gcode_text(slot),
                    TextAction::Analyse => {
                        let g: &GcodeText = &self.gcode_texts[i];
                        if let Err(e) = base_ui::gcode_view_ui::open_text(&mut self.gcode_view, &g.name, &g.text) {
                            self.status = e;
                        }
                    }
                    TextAction::None => {}
                }
            }
            Content::Gcode => {
                self.gcode_view_panel(ui);
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
        matches!(content, Content::Preview | Content::GcodeText(_) | Content::Camera)
    }
}

#[cfg(test)]
impl App {
    pub fn tab_title_of(&self, c: Content) -> String {
        PaneHost::tab_title(self, c)
    }
    pub fn tab_dirty_of(&self, c: Content) -> bool {
        PaneHost::tab_dirty(self, c)
    }
    /// What the question about a closed drawing leads to, without the window: the drawing is closed.
    pub fn close_shown_tab_for_test(&mut self, slot: usize) {
        self.activate_slot(slot);
        self.close_shown_tab();
    }
}
