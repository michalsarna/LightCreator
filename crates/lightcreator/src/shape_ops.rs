//! Document-level operations: boolean shape operations, offsetting, curve conversion, images and text.
use crate::app::App;
use crate::i18n::{tr, trf};
use eframe::egui;
use lc_core::ops::{self, BoolOp};
use lc_core::{Kind, Polyline, Pt, TextData, Xf};
use std::path::PathBuf;

impl App {
    fn status_is(&mut self, s: impl Into<String>) {
        self.status = s.into();
    }

    /// Combine the selected shapes (in selection order) with a boolean operation.
    pub fn bool_op(&mut self, op: BoolOp) {
        let ids: Vec<u64> = self.sel.iter().copied().filter(|id| self.doc.shape(*id).is_some_and(|s| !s.is_image())).collect();
        if ids.len() < 2 {
            self.status_is(tr("Select at least two shapes (not images)."));
            return;
        }
        let polys_of = |app: &App, id: u64| -> Vec<Polyline> { app.doc.shape(id).map(|s| s.polys()).unwrap_or_default() };
        let mut acc = polys_of(self, ids[0]);
        for id in &ids[1..] {
            let next = polys_of(self, *id);
            acc = ops::boolean(&acc, &next, op);
        }
        if acc.is_empty() {
            self.status_is(tr("The result is empty."));
            return;
        }
        let layer = self.doc.shape(ids[0]).map(|s| s.layer).unwrap_or(self.active_layer);
        self.checkpoint();
        self.doc.shapes.retain(|s| !ids.contains(&s.id));
        let id = self.doc.add(layer, Kind::Path(acc), Xf::IDENTITY);
        self.sel = vec![id];
        self.node_sel.clear();
    }

    /// Offset the outlines of the selected shapes by `d` mm (negative shrinks).
    pub fn offset_selected(&mut self, d: f64, keep_original: bool) {
        let ids: Vec<u64> = self.sel.clone();
        let mut made = vec![];
        let mut created: Vec<(usize, Vec<Polyline>)> = vec![];
        for id in &ids {
            let Some(s) = self.doc.shape(*id) else { continue };
            if s.is_image() {
                continue;
            }
            let res = ops::offset(&s.polys(), d);
            if !res.is_empty() {
                created.push((s.layer, res));
            }
        }
        if created.is_empty() {
            self.status_is(tr("The result is empty."));
            return;
        }
        self.checkpoint();
        if !keep_original {
            self.doc.shapes.retain(|s| !ids.contains(&s.id));
        }
        for (layer, polys) in created {
            made.push(self.doc.add(layer, Kind::Path(polys), Xf::IDENTITY));
        }
        self.sel = made;
        self.node_sel.clear();
    }

    /// Group the selected shapes. Layers are untouched; a group moves, scales and selects as one.
    pub fn group_selection(&mut self) {
        if self.sel.len() < 2 {
            self.status_is(tr("Select at least two objects to group."));
            return;
        }
        self.checkpoint();
        let g = self.doc.new_group_id();
        for id in self.sel.clone() {
            if let Some(s) = self.doc.shape_mut(id) {
                s.group = Some(g);
            }
        }
        self.status_is(tr("Grouped. The objects keep their layers."));
    }

    pub fn ungroup_selection(&mut self) {
        if !self.sel.iter().any(|id| self.doc.shape(*id).is_some_and(|s| s.group.is_some())) {
            return;
        }
        self.checkpoint();
        for id in self.sel.clone() {
            if let Some(s) = self.doc.shape_mut(id) {
                s.group = None;
            }
        }
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
                self.sel = ids;
            }
            Err(e) => {
                // Nothing was added: drop the checkpoint taken for the failed import.
                self.undo.pop();
                self.status_is(trf("Import failed: {}", &[&e]));
            }
        }
    }

    /// Context menu for the canvas. Returns the chosen action.
    pub fn context_menu(&mut self, ui: &mut egui::Ui) -> Option<crate::menu::Act> {
        use crate::menu::Act;
        let has_sel = !self.sel.is_empty();
        let multi = self.sel.len() >= 2;
        let grouped = self.sel.iter().any(|id| self.doc.shape(*id).is_some_and(|s| s.group.is_some()));
        let single_image = self.sel.len() == 1 && self.doc.shape(self.sel[0]).is_some_and(|s| s.is_image());
        let mut chosen = None;
        let mut item = |ui: &mut egui::Ui, on: bool, act: Act, label: &'static str| {
            if ui.add_enabled(on, egui::Button::new(tr(label))).clicked() {
                chosen = Some(act);
                ui.close();
            }
        };
        item(ui, has_sel, Act::Copy, "Copy");
        item(ui, !self.clipboard.is_empty(), Act::Paste, "Paste");
        item(ui, has_sel, Act::Duplicate, "Duplicate");
        item(ui, has_sel, Act::Delete, "Delete");
        item(ui, true, Act::SelectAll, "Select all");
        ui.separator();
        item(ui, multi, Act::Group, "Group");
        item(ui, grouped, Act::Ungroup, "Ungroup");
        ui.separator();
        ui.menu_button(tr("Arrange"), |ui| {
            item(ui, has_sel, Act::ToFront, "Bring to front");
            item(ui, has_sel, Act::ToBack, "Send to back");
            ui.separator();
            for (i, n) in ["Align left", "Align centre (H)", "Align right", "Align top", "Align centre (V)", "Align bottom"].into_iter().enumerate() {
                item(ui, has_sel, Act::Align(i as u8), n);
            }
            ui.separator();
            item(ui, has_sel, Act::CenterOnBed, "Centre on bed");
            item(ui, has_sel, Act::FlipH, "Flip horizontal");
            item(ui, has_sel, Act::FlipV, "Flip vertical");
            item(ui, has_sel, Act::RotCw, "Rotate 90° CW");
            item(ui, has_sel, Act::RotCcw, "Rotate 90° CCW");
        });
        ui.menu_button(tr("Shape operations"), |ui| {
            item(ui, multi, Act::BoolUnion, "Union");
            item(ui, multi, Act::BoolIntersect, "Intersection");
            item(ui, multi, Act::BoolSubtract, "Subtract");
            item(ui, multi, Act::BoolXor, "Exclusive or");
            ui.separator();
            item(ui, has_sel, Act::OffsetShape, "Offset shape…");
            item(ui, has_sel, Act::ToCurves, "Convert to curves");
            item(ui, has_sel, Act::ToPath, "Convert to path");
        });
        if single_image {
            ui.separator();
            item(ui, true, Act::AdjustImage, "Adjust image…");
            item(ui, true, Act::TraceImage, "Trace image…");
        }
        chosen
    }

    pub fn to_curves(&mut self) {
        if self.sel.is_empty() {
            return;
        }
        self.checkpoint();
        for id in self.sel.clone() {
            if let Some(s) = self.doc.shape_mut(id) {
                if !s.is_image() {
                    s.to_bezier();
                }
            }
        }
        self.node_sel.clear();
    }

    pub fn import_image(&mut self) {
        if let Some(p) = rfd::FileDialog::new().add_filter("Images", &["png", "jpg", "jpeg", "bmp", "gif", "webp"]).pick_file() {
            self.import_image_path(p);
        }
    }

    pub fn import_image_path(&mut self, p: PathBuf) {
        self.open_image_dialog(p);
    }

    /// Create a text object with its top-left corner at `at`.
    pub fn add_text_at(&mut self, at: Pt) {
        self.checkpoint();
        let t: TextData = self.text_default.clone();
        let id = self.doc.add(self.active_layer, Kind::Text(t), Xf::translate(at.x, at.y));
        self.sel = vec![id];
        self.side_tab = crate::app::SideTab::Properties;
        self.focus_text = true;
    }

    /// Handle files dropped onto the window.
    pub fn handle_drops(&mut self, ctx: &egui::Context) {
        let files: Vec<PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).collect());
        for p in files {
            let ext = p.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
            match ext.as_str() {
                "png" | "jpg" | "jpeg" | "bmp" | "gif" | "webp" => self.import_image_path(p),
                "svg" | "lcr" => self.open_path(p),
                "ai" | "pdf" => self.import_ai_path(p),
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
        a.sel = vec![s1, s2];
        a.group_selection();
        let g = a.doc.shape(s1).unwrap().group;
        assert!(g.is_some() && g == a.doc.shape(s2).unwrap().group);
        assert_eq!((a.doc.shape(s1).unwrap().layer, a.doc.shape(s2).unwrap().layer), (0, 3));
        assert_eq!(a.doc.group_of(s1).len(), 2);
        // Duplicate: the copies form their own group.
        a.duplicate();
        assert_eq!(a.doc.shapes.len(), 4);
        let groups: std::collections::HashSet<_> = a.doc.shapes.iter().filter_map(|s| s.group).collect();
        assert_eq!(groups.len(), 2);
        // Ungroup the copies only.
        a.ungroup_selection();
        assert!(a.sel.iter().all(|id| a.doc.shape(*id).unwrap().group.is_none()));
        assert_eq!(a.doc.group_of(s1).len(), 2);
        // Undo restores the group.
        a.do_undo();
        assert!(a.sel.iter().all(|id| a.doc.shape(*id).map_or(true, |s| s.group.is_some())));
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
}
