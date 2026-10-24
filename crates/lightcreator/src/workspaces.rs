//! Open drawings as tabs (the bar and the closing are in `base_ui::doc_tabs`): each tab holds a `Workspace`, the part of
//! the program that belongs to one drawing. The live fields of the program are the shown tab; the others are parked here
//! and exchanged with the live ones when a tab is picked.
use crate::app::{App, Screen};
use base_ui::doc_tabs::{self, TabAction, TabHost, TabInfo, MAX_TABS};
use base_ui::editor::DocView;
use eframe::egui;
use lc_core::Document;
use std::path::PathBuf;

/// What belongs to one drawing. `clean` and `autosaved_clean` stand for "the revision equals the saved one": the revisions
/// themselves are not kept, because the caches of the program (the preview, the materials) are keyed by the live revision,
/// which is a new number every time a tab is shown.
pub struct Workspace {
    pub doc: Document,
    pub undo: Vec<Document>,
    pub redo: Vec<Document>,
    pub path: Option<PathBuf>,
    pub clean: bool,
    pub autosaved_clean: bool,
    pub view: DocView,
    /// The number of the recovery file of this tab.
    pub slot: usize,
}

impl Workspace {
    fn empty(doc: Document, slot: usize) -> Workspace {
        Workspace { doc, undo: vec![], redo: vec![], path: None, clean: true, autosaved_clean: true, view: DocView::default(), slot }
    }

    /// Unsaved work, as `App::is_dirty` says it for the shown tab.
    pub fn dirty(&self) -> bool {
        !self.clean && (!self.doc.scene.shapes.is_empty() || self.path.is_some())
    }
}

/// The parked tabs: `parked[i]` is the tab `i`, `None` for the shown one.
#[derive(Default)]
pub struct Tabs {
    pub parked: Vec<Option<Workspace>>,
    pub active: usize,
    /// The shown tab is being closed by its cross: the question is on the screen.
    pub closing: bool,
}

impl App {
    /// The shown tab with the parked ones: a swap of every field that belongs to a drawing. The pattern has no `..`, so a
    /// field added to `Workspace` has to be handled here.
    fn swap_ws(&mut self, w: &mut Workspace) {
        let Workspace { doc, undo, redo, path, clean, autosaved_clean, view, slot } = w;
        std::mem::swap(&mut self.doc, doc);
        std::mem::swap(&mut self.undo, undo);
        std::mem::swap(&mut self.redo, redo);
        std::mem::swap(&mut self.path, path);
        std::mem::swap(&mut self.slot, slot);
        self.ed.swap_doc_view(view);
        // The revisions: a new live number (so nothing cached for another tab is taken for this one), and the saved ones
        // either equal to it or not.
        let was_clean = self.revision == self.saved_revision;
        let was_autosaved_clean = self.revision == self.autosaved_revision;
        self.revision += 1;
        self.saved_revision = if *clean { self.revision } else { self.revision.wrapping_sub(1) };
        self.autosaved_revision = if *autosaved_clean { self.revision } else { self.revision.wrapping_sub(1) };
        *clean = was_clean;
        *autosaved_clean = was_autosaved_clean;
        // Whatever refers to the drawing that was shown.
        self.preview = None;
        self.frame_bounds = None;
        self.layer_dlg = None;
        self.mat_edit = None;
        self.check_dlg = None;
        self.gcode_view = None;
        // The device is the profile, edited in the Device tab: every tab uses the profile as it is now.
        if let Some(p) = self.profiles.get(self.active) {
            self.doc.device = p.clone();
        }
    }

    pub fn tab_count(&self) -> usize {
        self.tabs_state.parked.len().max(1)
    }

    fn ensure_tabs(&mut self) {
        if self.tabs_state.parked.is_empty() {
            self.tabs_state.parked.push(None);
        }
    }

    /// The slots in use.
    fn used_slots(&self) -> Vec<usize> {
        let mut v: Vec<usize> = self.tabs_state.parked.iter().flatten().map(|w| w.slot).collect();
        v.push(self.slot);
        v.extend(self.recovery.iter().map(|f| f.slot));
        v
    }

    /// A new, empty tab, shown now. False (with a message) when ten tabs are open.
    pub fn new_tab(&mut self) -> bool {
        if !self.make_room() {
            return false;
        }
        self.new_doc();
        true
    }

    /// New from the start screen: a blank tab is used as it is, otherwise (the start screen was reached with drawings open,
    /// by Switch device) a tab is added, so nothing that is open is replaced.
    pub fn new_tab_from_start(&mut self) -> bool {
        if self.shown_tab_is_blank() {
            self.new_doc();
            true
        } else {
            self.new_tab()
        }
    }

    /// Park the shown tab and show a fresh one (not yet initialised: the caller fills it). False when there is no room.
    fn make_room(&mut self) -> bool {
        self.ensure_tabs();
        if self.tabs_state.parked.len() >= MAX_TABS {
            self.status = doc_tabs::limit_message();
            return false;
        }
        let slot = doc_tabs::free_slot(&self.used_slots());
        let mut w = Workspace::empty(Document { device: self.doc.device.clone(), ..Document::default() }, slot);
        self.swap_ws(&mut w);
        let at = self.tabs_state.active;
        self.tabs_state.parked[at] = Some(w);
        self.tabs_state.parked.push(None);
        self.tabs_state.active = self.tabs_state.parked.len() - 1;
        true
    }

    /// Is the shown tab one that a file can simply replace: no name, nothing in it.
    fn shown_tab_is_blank(&self) -> bool {
        self.path.is_none() && self.doc.scene.shapes.is_empty() && !self.is_dirty()
    }

    /// Before a project file is put into the shown tab: a blank tab is used, otherwise a new one is made. False when the
    /// file is open in another tab already (that tab is shown) or there is no room.
    pub fn tab_for_file(&mut self, file: &std::path::Path) -> bool {
        self.ensure_tabs();
        let same = |p: &PathBuf| std::fs::canonicalize(p).ok() == std::fs::canonicalize(file).ok();
        if self.path.as_ref().is_some_and(same) {
            return false;
        }
        let other = self.tabs_state.parked.iter().position(|w| w.as_ref().is_some_and(|w| w.path.as_ref().is_some_and(same)));
        if let Some(i) = other {
            self.switch_to(i);
            return false;
        }
        if self.shown_tab_is_blank() {
            return true;
        }
        self.make_room()
    }

    pub fn switch_to(&mut self, i: usize) {
        self.ensure_tabs();
        if i == self.tabs_state.active || i >= self.tabs_state.parked.len() {
            return;
        }
        let Some(mut w) = self.tabs_state.parked[i].take() else { return };
        self.swap_ws(&mut w);
        let at = self.tabs_state.active;
        self.tabs_state.parked[at] = Some(w);
        self.tabs_state.active = i;
    }

    /// Close the shown tab (its work is saved or given up by now). The last tab turns into a new, empty drawing and the
    /// start screen is shown.
    pub fn close_shown_tab(&mut self) {
        self.ensure_tabs();
        lc_core::autosave::clear_slot(&self.data_base, "lightcreator", self.slot);
        let n = self.tabs_state.parked.len();
        if n <= 1 {
            self.new_doc();
            self.screen = Screen::Start;
            return;
        }
        let at = self.tabs_state.active;
        let next = if at + 1 < n { at + 1 } else { at - 1 };
        let Some(mut w) = self.tabs_state.parked[next].take() else { return };
        self.swap_ws(&mut w);
        // `w` now holds the closed tab.
        self.tabs_state.parked.remove(at);
        self.tabs_state.active = if next > at { next - 1 } else { next };
    }

    pub fn any_dirty(&self) -> bool {
        self.is_dirty() || self.tabs_state.parked.iter().flatten().any(|w| w.dirty())
    }

    fn tab_infos(&self) -> Vec<TabInfo> {
        let n = self.tab_count();
        (0..n)
            .map(|i| {
                if i == self.tabs_state.active || self.tabs_state.parked.len() <= 1 {
                    TabInfo { title: doc_tabs::title_of(self.path.as_deref()), dirty: self.is_dirty() }
                } else {
                    let w = self.tabs_state.parked[i].as_ref();
                    TabInfo { title: doc_tabs::title_of(w.and_then(|w| w.path.as_deref())), dirty: w.is_some_and(|w| w.dirty()) }
                }
            })
            .collect()
    }

    /// The bar of the tabs above the work area.
    pub fn doc_tab_bar(&mut self, ui: &mut egui::Ui) {
        self.ensure_tabs();
        let infos = self.tab_infos();
        match doc_tabs::tab_bar(ui, &infos, self.tabs_state.active) {
            Some(TabAction::Select(i)) => self.switch_to(i),
            Some(TabAction::New) => {
                self.new_tab();
            }
            Some(TabAction::Close(i)) => {
                self.switch_to(i);
                if self.is_dirty() {
                    self.tabs_state.closing = true;
                } else {
                    self.close_shown_tab();
                }
            }
            None => {}
        }
    }

    /// Write the work of the tabs that are not shown (the shown one is written by `autosave_now`).
    pub fn autosave_parked(&mut self) {
        let base = self.data_base.clone();
        for w in self.tabs_state.parked.iter_mut().flatten() {
            if !w.dirty() || w.autosaved_clean {
                continue;
            }
            let ok = match &w.path {
                Some(p) => lc_core::autosave::write_atomic(p, &w.doc.to_json()).is_ok(),
                None => lc_core::autosave::save_slot(&base, "lightcreator", w.slot, &w.doc.to_json()).is_ok(),
            };
            if ok {
                w.autosaved_clean = true;
                if w.path.is_some() {
                    w.clean = true;
                }
            }
        }
    }

    pub fn parked_need_autosave(&self) -> bool {
        self.tabs_state.parked.iter().flatten().any(|w| w.dirty() && !w.autosaved_clean)
    }

    /// The question for a tab closed by its cross (the closing of the program asks through `doc_tabs::quit_tick`).
    pub fn close_tab_question(&mut self, ctx: &egui::Context) {
        if !self.tabs_state.closing {
            return;
        }
        match doc_tabs::ask_save(ctx, &doc_tabs::title_of(self.path.as_deref()), crate::window_ui::SAVE_GREEN) {
            Some(doc_tabs::Answer::Save) => {
                self.save(false);
                if !self.is_dirty() {
                    self.tabs_state.closing = false;
                    self.close_shown_tab();
                } else {
                    self.tabs_state.closing = false;
                }
            }
            Some(doc_tabs::Answer::Discard) => {
                self.tabs_state.closing = false;
                self.close_shown_tab();
            }
            Some(doc_tabs::Answer::Cancel) => self.tabs_state.closing = false,
            None => {}
        }
    }
}

impl TabHost for App {
    fn tab_count(&self) -> usize {
        App::tab_count(self)
    }
    fn active_tab(&self) -> usize {
        self.tabs_state.active
    }
    fn tab_dirty(&self, i: usize) -> bool {
        if i == self.tabs_state.active || self.tabs_state.parked.len() <= 1 {
            self.is_dirty()
        } else {
            self.tabs_state.parked[i].as_ref().is_some_and(|w| w.dirty())
        }
    }
    fn tab_title(&self, i: usize) -> String {
        if i == self.tabs_state.active || self.tabs_state.parked.len() <= 1 {
            doc_tabs::title_of(self.path.as_deref())
        } else {
            doc_tabs::title_of(self.tabs_state.parked[i].as_ref().and_then(|w| w.path.as_deref()))
        }
    }
    fn switch_tab(&mut self, i: usize) {
        self.switch_to(i);
    }
    fn save_active(&mut self) {
        self.save(false);
    }
    fn discard_active(&mut self) {
        lc_core::autosave::clear_slot(&self.data_base, "lightcreator", self.slot);
    }
    fn close_active_tab(&mut self) {
        self.close_shown_tab();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base_ui::doc_tabs::{quit_answer, quit_prepare, Answer, Quit};
    use lc_core::{Kind, Xf};

    fn app() -> App {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        a.data_base = std::env::temp_dir().join(format!("lc-tabs-{}-{}", std::process::id(), N.fetch_add(1, std::sync::atomic::Ordering::SeqCst)));
        let _ = std::fs::remove_dir_all(&a.data_base);
        a.screen = Screen::Editor;
        a
    }

    fn draw(a: &mut App, n: usize) {
        for _ in 0..n {
            a.doc.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::IDENTITY);
        }
        a.touch();
    }

    #[test]
    fn a_new_tab_is_a_new_drawing_and_each_tab_keeps_its_own() {
        let mut a = app();
        draw(&mut a, 2);
        assert!(a.new_tab());
        assert_eq!(a.tab_count(), 2);
        assert_eq!(a.doc.scene.shapes.len(), 0, "the new tab is empty");
        draw(&mut a, 3);
        assert!(a.new_tab());
        draw(&mut a, 1);
        a.switch_to(0);
        assert_eq!(a.doc.scene.shapes.len(), 2);
        a.switch_to(1);
        assert_eq!(a.doc.scene.shapes.len(), 3);
        a.switch_to(2);
        assert_eq!(a.doc.scene.shapes.len(), 1);
        assert_eq!(a.tabs_state.active, 2);
    }

    #[test]
    fn there_are_at_most_ten_tabs() {
        let mut a = app();
        for _ in 1..MAX_TABS {
            assert!(a.new_tab());
        }
        assert_eq!(a.tab_count(), MAX_TABS);
        assert!(!a.new_tab(), "the eleventh is refused");
        assert_eq!(a.tab_count(), MAX_TABS);
        assert!(a.status.contains("10"), "{}", a.status);
        let mut slots = a.used_slots();
        slots.sort();
        slots.dedup();
        assert_eq!(slots.len(), MAX_TABS, "every tab has a slot of its own");
    }

    #[test]
    fn what_is_cached_for_one_tab_is_not_taken_for_another() {
        let mut a = app();
        draw(&mut a, 1);
        a.new_tab();
        draw(&mut a, 4);
        let r1 = a.revision;
        a.preview = Some((r1, lc_core::gcode::generate(&a.doc)));
        a.switch_to(0);
        assert!(a.preview.is_none(), "the preview of the other tab is dropped");
        assert_ne!(a.revision, r1, "the revision is a new number in every tab shown");
    }

    #[test]
    fn unsaved_work_is_per_tab_and_closing_asks_about_each() {
        let mut a = app();
        draw(&mut a, 1);
        a.new_tab();
        a.new_tab();
        draw(&mut a, 2);
        a.switch_to(0);
        assert!(a.is_dirty() && a.any_dirty());
        assert!(a.tab_dirty(0) && !a.tab_dirty(1) && a.tab_dirty(2));
        // Closing the program asks; the first tab with unsaved work is the one shown.
        let ctx = egui::Context::default();
        a.request_close(&ctx);
        assert!(a.show_close_dlg && !a.allow_close);
        assert_eq!(quit_prepare(&mut a), Quit::Asking);
        assert_eq!(a.tabs_state.active, 0);
        // Discard it: it is closed and the next one with work (the third, now the second) is shown.
        assert_eq!(quit_answer(&mut a, Answer::Discard), Quit::Asking);
        assert_eq!(a.tab_count(), 2);
        assert!(a.is_dirty());
        assert_eq!(a.doc.scene.shapes.len(), 2);
        // Cancel: the program stays, and the tab keeps its work.
        assert_eq!(quit_answer(&mut a, Answer::Cancel), Quit::Stopped);
        assert_eq!(a.tab_count(), 2);
        assert!(a.any_dirty());
    }

    #[test]
    fn a_file_that_is_open_is_shown_not_opened_again() {
        let mut a = app();
        let dir = std::env::temp_dir().join(format!("lc-tabs-files-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mk = |n: &str| {
            let f = dir.join(n);
            std::fs::write(&f, Document::default().to_json()).unwrap();
            f
        };
        let (f1, f2) = (mk("one.cre"), mk("two.cre"));
        assert!(a.open_path(f1.clone()));
        assert_eq!(a.tab_count(), 1, "a blank tab is used for the first file");
        assert!(a.open_path(f2.clone()));
        assert_eq!(a.tab_count(), 2);
        assert!(a.open_path(f1.clone()));
        assert_eq!(a.tab_count(), 2, "one.cre is open already");
        assert_eq!(a.tabs_state.active, 0);
        assert_eq!(a.path.as_deref(), Some(f1.as_path()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn closing_the_last_tab_returns_to_the_start_screen() {
        let mut a = app();
        a.new_tab();
        a.close_shown_tab();
        assert_eq!(a.tab_count(), 1);
        assert_eq!(a.screen, Screen::Editor);
        a.close_shown_tab();
        assert_eq!(a.tab_count(), 1);
        assert_eq!(a.screen, Screen::Start);
    }

    #[test]
    fn tabs_that_are_not_shown_are_autosaved_to_their_own_files() {
        let mut a = app();
        draw(&mut a, 1);
        a.new_tab();
        draw(&mut a, 2);
        let (s0, s1) = (a.tabs_state.parked[0].as_ref().unwrap().slot, a.slot);
        assert_ne!(s0, s1);
        assert!(a.parked_need_autosave());
        a.autosave_parked();
        assert!(!a.parked_need_autosave());
        assert!(lc_core::autosave::slot_path(&a.data_base, "lightcreator", s0).is_file(), "the parked tab has its file");
        assert!(!lc_core::autosave::slot_path(&a.data_base, "lightcreator", s1).exists(), "the shown one is written by autosave_now");
        // Every unsaved tab is found again after a crash; closing a tab removes its file.
        a.autosave_now();
        assert_eq!(lc_core::autosave::find_all(&a.data_base, "lightcreator").len(), 2);
        a.close_shown_tab();
        assert_eq!(lc_core::autosave::find_all(&a.data_base, "lightcreator").len(), 1);
    }

    #[test]
    fn every_unsaved_tab_is_recovered_in_a_tab_of_its_own() {
        let mut a = app();
        draw(&mut a, 1);
        a.new_tab();
        draw(&mut a, 2);
        a.autosave_parked();
        a.autosave_now();
        let base = a.data_base.clone();
        let mut b = App::build(&egui::Context::default(), None, false);
        b.data_base = base.clone();
        b.recovery = lc_core::autosave::find_all(&base, "lightcreator");
        assert_eq!(b.recovery.len(), 2);
        b.recovery_choice(base_ui::autosave_ui::Recovery::Recover);
        assert_eq!(b.screen, Screen::Editor);
        assert_eq!(b.tab_count(), 2);
        let mut shapes = vec![b.doc.scene.shapes.len()];
        b.switch_to(0);
        shapes.push(b.doc.scene.shapes.len());
        shapes.sort();
        assert_eq!(shapes, vec![1, 2]);
        assert!(b.any_dirty(), "still unsaved work");
        assert_eq!(lc_core::autosave::find_all(&base, "lightcreator").len(), 2, "the files stay until the work is saved");
        let _ = std::fs::remove_dir_all(&base);
    }
}

#[cfg(test)]
mod start_screen_tests {
    use super::*;
    use crate::menu::Act;
    use lc_core::{Kind, Xf};

    #[test]
    fn from_the_start_screen_new_and_open_never_replace_a_drawing_that_is_open() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.data_base = std::env::temp_dir().join(format!("lc-tabs-start-{}", std::process::id()));
        a.screen = Screen::Editor;
        a.doc.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::IDENTITY);
        a.touch();
        a.new_tab();
        a.doc.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::IDENTITY);
        a.doc.add(0, Kind::Rect { w: 10.0, h: 10.0 }, Xf::translate(20.0, 0.0));
        a.touch();
        // Switch device goes back to the start screen with both drawings open.
        a.do_act(&ctx, Act::SwitchDevice);
        assert_eq!(a.screen, Screen::Start);
        a.do_act(&ctx, Act::New);
        assert_eq!((a.screen, a.tab_count()), (Screen::Editor, 3), "New adds a tab");
        assert_eq!(a.doc.scene.shapes.len(), 0);
        a.switch_to(0);
        assert_eq!(a.doc.scene.shapes.len(), 1);
        a.switch_to(1);
        assert_eq!(a.doc.scene.shapes.len(), 2);
        // The same with Open.
        let dir = std::env::temp_dir().join(format!("lc-tabs-start-files-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("three.cre");
        std::fs::write(&f, lc_core::Document::default().to_json()).unwrap();
        a.close_shown_tab(); // the second drawing is given up
        a.switch_to(0);
        assert_eq!(a.doc.scene.shapes.len(), 1);
        a.do_act(&ctx, Act::SwitchDevice);
        assert_eq!(a.screen, Screen::Start);
        let before = a.tab_count();
        a.start_project(|app| app.open_path(f.clone()));
        assert_eq!(a.tab_count(), before + 1, "Open adds a tab too");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
