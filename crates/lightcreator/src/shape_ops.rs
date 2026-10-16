//! Document-level operations: boolean shape operations, offsetting, curve conversion, images and text.
use base_ui::editor::EditorImages;
#[cfg(test)]
use base_ui::editor::{EditorCommands, EditorHost, EditorShapeOps};
use crate::app::App;
use crate::i18n::trf;
use eframe::egui;
use lc_core::{Kind, Pt, TextData, Xf};
use std::path::PathBuf;

impl App {
    fn status_is(&mut self, s: impl Into<String>) {
        self.status = s.into();
    }

    pub fn import_ai(&mut self) {
        if let Some(p) = rfd::FileDialog::new().add_filter("Adobe Illustrator / PDF", &["ai", "pdf"]).pick_file() {
            self.import_ai_path(p);
        }
    }

    pub fn import_ai_path(&mut self, p: PathBuf) {
        let res = std::fs::read(&p).map_err(|e| e.to_string()).and_then(|d| {
            self.checkpoint();
            lc_core::pdf::import(&d, &mut self.doc, None)
        });
        match res {
            Ok(ids) => {
                self.status_is(trf("Imported {} paths from {}", &[&ids.len(), &p.display()]));
                self.ed.sel = ids;
            }
            Err(e) => {
                // Nothing was added: drop the checkpoint taken for the failed import.
                self.undo.pop();
                self.status_is(trf("Import failed: {}", &[&e]));
            }
        }
    }

    pub fn import_image_path(&mut self, p: PathBuf) {
        self.open_image_dialog(p);
    }

    /// Create a text object with its top-left corner at `at`.
    pub fn add_text_at(&mut self, at: Pt) {
        self.checkpoint();
        let t: TextData = self.text_default.clone();
        let id = self.doc.add(self.ed.active_layer, Kind::Text(t), Xf::translate(at.x, at.y));
        self.ed.sel = vec![id];
        self.side_tab = crate::app::SideTab::Properties;
        self.focus_text = true;
    }

    /// Handle files dropped onto the window.
    pub fn handle_drops(&mut self, ctx: &egui::Context) {
        let files: Vec<PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).collect());
        for p in files {
            let ext = p.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
            let project = matches!(ext.as_str(), "svg" | "cre" | "lbrn2" | "lbrn" | "ugsd") || self.plugin_reads(&p);
            if self.screen == crate::app::Screen::Start && !project {
                continue;
            }
            match ext.as_str() {
                "png" | "jpg" | "jpeg" | "bmp" | "gif" | "webp" => self.import_image_path(p),
                "svg" | "cre" | "lbrn2" | "lbrn" | "ugsd" if self.screen == crate::app::Screen::Start => self.start_project(|app| app.open_path(p)),
                "svg" | "cre" | "lbrn2" | "lbrn" | "ugsd" => {
                    self.open_path(p);
                }
                "ai" | "pdf" => self.import_ai_path(p),
                _ if self.plugin_reads(&p) => self.plugin_import_from_start(p),
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        App::build(&egui::Context::default(), None, false)
    }

    #[test]
    fn grouping_keeps_layers_and_pastes_as_a_new_group() {
        let mut a = app();
        let s1 = a.doc.add(0, lc_core::Kind::Rect { w: 5.0, h: 5.0 }, Xf::IDENTITY);
        let s2 = a.doc.add(3, lc_core::Kind::Rect { w: 5.0, h: 5.0 }, Xf::translate(10.0, 0.0));
        a.ed.sel = vec![s1, s2];
        a.group_selection();
        let g = a.doc.shape(s1).unwrap().group;
        assert!(g.is_some() && g == a.doc.shape(s2).unwrap().group);
        assert_eq!((a.doc.shape(s1).unwrap().layer, a.doc.shape(s2).unwrap().layer), (0, 3));
        assert_eq!(a.doc.group_of(s1).len(), 2);
        // Duplicate: the copies form their own group.
        a.duplicate();
        assert_eq!(a.doc.scene.shapes.len(), 4);
        let groups: std::collections::HashSet<_> = a.doc.scene.shapes.iter().filter_map(|s| s.group).collect();
        assert_eq!(groups.len(), 2);
        // Ungroup the copies only.
        a.ungroup_selection();
        assert!(a.ed.sel.iter().all(|id| a.doc.shape(*id).unwrap().group.is_none()));
        assert_eq!(a.doc.group_of(s1).len(), 2);
        // Undo restores the group.
        a.do_undo();
        assert!(a.ed.sel.iter().all(|id| a.doc.shape(*id).map_or(true, |s| s.group.is_some())));
    }

    #[test]
    fn preview_job_lists_operations() {
        let mut a = app();
        a.doc.layers[1].mode = lc_core::LayerMode::FillAndLine;
        a.doc.layers[1].passes = 2;
        a.doc.add(1, lc_core::Kind::Rect { w: 10.0, h: 10.0 }, Xf::IDENTITY);
        a.doc.add(0, lc_core::Kind::Rect { w: 4.0, h: 4.0 }, Xf::translate(20.0, 0.0));
        let job = lc_core::gcode::generate(&a.doc);
        // Layer 0: line. Layer 1: (fill + line) x 2 passes.
        assert_eq!(job.ops.len(), 5);
        assert!(job.moves.iter().all(|m| m.op < job.ops.len()));
        assert_eq!(job.moves.iter().filter(|m| m.laser).map(|m| m.op).collect::<std::collections::BTreeSet<_>>().len(), 5);
    }

    #[test]
    fn marquee_direction_decides_between_inside_and_touching() {
        let mut a = app();
        let small = a.doc.add(0, lc_core::Kind::Rect { w: 4.0, h: 4.0 }, Xf::translate(2.0, 2.0));
        let big = a.doc.add(0, lc_core::Kind::Rect { w: 30.0, h: 30.0 }, Xf::translate(5.0, 5.0));
        let far = a.doc.add(0, lc_core::Kind::Rect { w: 4.0, h: 4.0 }, Xf::translate(100.0, 100.0));
        let r = lc_core::Rect { min: lc_core::Pt::new(0.0, 0.0), max: lc_core::Pt::new(10.0, 10.0) };
        assert_eq!(a.marquee_hits(&r, true), vec![small], "left to right: only wholly inside");
        let mut touched = a.marquee_hits(&r, false);
        touched.sort();
        assert_eq!(touched, vec![small, big], "right to left: also partly touched");
        assert!(!touched.contains(&far));
    }

    #[test]
    fn locked_objects_do_not_move_or_vanish_but_copy_unlocked() {
        let mut a = app();
        let id = a.doc.add(0, lc_core::Kind::Rect { w: 10.0, h: 10.0 }, Xf::translate(5.0, 5.0));
        a.ed.sel = vec![id];
        a.lock_selection(true);
        assert!(a.doc.shape(id).unwrap().locked);
        let before = a.doc.shape(id).unwrap().xf;
        a.transform_selection(Xf::translate(20.0, 0.0));
        a.flip(true);
        a.rotate_sel(90.0);
        a.center_on_bed();
        a.align(0);
        a.assign_layer(3);
        a.to_path();
        a.delete_selection();
        let s = a.doc.shape(id).expect("still there");
        assert_eq!((s.xf, s.layer), (before, 3 * 0), "unchanged, still on layer 0");
        // Copy and paste gives an editable copy; the original stays locked.
        a.duplicate();
        assert_eq!(a.doc.scene.shapes.len(), 2);
        let copy = a.doc.shape(a.ed.sel[0]).unwrap();
        assert!(!copy.locked);
        a.transform_selection(Xf::translate(1.0, 0.0));
        assert!(a.doc.shape(a.ed.sel[0]).unwrap().xf != before);
        assert!(a.doc.shape(id).unwrap().locked);
        // Unlock makes it editable again.
        a.ed.sel = vec![id];
        a.lock_selection(false);
        a.transform_selection(Xf::translate(2.0, 0.0));
        assert!(a.doc.shape(id).unwrap().xf != before);
    }

    #[test]
    fn work_area_background_follows_the_colour_scheme() {
        let mut a = app();
        a.scheme = crate::theme::Scheme::Light;
        assert_eq!(a.bed_fill(), eframe::egui::Color32::WHITE);
        a.scheme = crate::theme::Scheme::Dark;
        let g = a.bed_fill();
        assert!(g.r() > 0xdd && g.r() < 0xff, "very light grey, not white: {g:?}");
        a.ed.grid_prefs.bed_color = Some([10, 20, 30, 255]);
        assert_eq!(a.bed_fill(), eframe::egui::Color32::from_rgb(10, 20, 30));
    }

    #[test]
    fn rounding_corners_of_shapes_and_of_two_lines() {
        let mut a = app();
        let r = a.doc.add(0, lc_core::Kind::Rect { w: 20.0, h: 20.0 }, Xf::translate(5.0, 5.0));
        a.ed.sel = vec![r];
        a.round_corners(3.0);
        let before: f64 = 400.0;
        let area = a.doc.shape(r).unwrap().polys()[0].area().abs();
        assert!(area < before && area > before - 4.0 * (1.0 - std::f64::consts::FRAC_PI_4) * 9.0 - 0.5, "{area}");
        // Undo brings the sharp rectangle back.
        a.do_undo();
        assert!(matches!(a.doc.shape(r).unwrap().kind, lc_core::Kind::Rect { .. }));
        // Two lines.
        a.doc.scene.shapes.clear();
        let h = a.doc.add(0, lc_core::Kind::Path(vec![lc_core::Polyline::new(vec![Pt::new(0.0, 10.0), Pt::new(14.0, 10.0)], false)]), Xf::IDENTITY);
        let v = a.doc.add(1, lc_core::Kind::Path(vec![lc_core::Polyline::new(vec![Pt::new(10.0, 0.0), Pt::new(10.0, 25.0)], false)]), Xf::IDENTITY);
        a.ed.sel = vec![h, v];
        assert!(a.selected_line_pair().is_some());
        a.round_corners(3.0);
        assert_eq!(a.doc.scene.shapes.len(), 1, "the two lines became one rounded path");
        let polys = a.doc.scene.shapes[0].polys();
        assert!(polys[0].pts.len() > 4 && !polys[0].closed);
        // Nothing to round: a smooth ellipse leaves the document untouched (and no stray undo step).
        a.doc.scene.shapes.clear();
        let e = a.doc.add(0, lc_core::Kind::Ellipse { w: 10.0, h: 10.0 }, Xf::IDENTITY);
        a.ed.sel = vec![e];
        let undo_len = a.undo.len();
        a.round_corners(1.0);
        assert_eq!(a.undo.len(), undo_len);
        // Locked shapes are skipped.
        let rr = a.doc.add(0, lc_core::Kind::Rect { w: 10.0, h: 10.0 }, Xf::IDENTITY);
        a.ed.sel = vec![rr];
        a.lock_selection(true);
        a.round_corners(1.0);
        assert!(matches!(a.doc.shape(rr).unwrap().kind, lc_core::Kind::Rect { .. }));
    }
}
