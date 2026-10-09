//! Document-level operations: boolean shape operations, offsetting, curve conversion, images and text.
use crate::app::App;
use crate::i18n::{tr, trf};
use crate::icons;
use crate::theme;
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
        let ids: Vec<u64> = self.sel.iter().copied().filter(|id| self.doc.shape(*id).is_some_and(|s| !s.is_image() && !s.locked)).collect();
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
            self.doc.shapes.retain(|s| !ids.contains(&s.id) || s.locked);
        }
        for (layer, polys) in created {
            made.push(self.doc.add(layer, Kind::Path(polys), Xf::IDENTITY));
        }
        self.sel = made;
        self.node_sel.clear();
    }

    /// Shapes picked by a selection rectangle: dragged left to right only what lies wholly inside,
    /// right to left everything the rectangle touches. Groups are completed by the caller.
    pub fn marquee_hits(&self, r: &lc_core::Rect, left_to_right: bool) -> Vec<u64> {
        self.doc
            .shapes
            .iter()
            .filter(|s| self.doc.layers[s.layer].visible)
            .filter(|s| {
                if left_to_right {
                    s.bounds().is_some_and(|b| r.contains(b.min) && r.contains(b.max))
                } else {
                    s.intersects_rect(r)
                }
            })
            .map(|s| s.id)
            .collect()
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

    /// Lock or unlock the selected objects (a group is selected as a whole, so it locks as a whole).
    pub fn lock_selection(&mut self, lock: bool) {
        if !self.sel.iter().any(|id| self.doc.shape(*id).is_some_and(|s| s.locked != lock)) {
            return;
        }
        self.checkpoint();
        let mut n = 0;
        for id in self.sel.clone() {
            if let Some(s) = self.doc.shape_mut(id) {
                if s.locked != lock {
                    s.locked = lock;
                    n += 1;
                }
            }
        }
        self.node_sel.clear();
        self.status_is(if lock { trf("Locked {} object(s).", &[&n]) } else { trf("Unlocked {} object(s).", &[&n]) });
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
        let any_unlocked = self.sel.iter().any(|id| self.doc.shape(*id).is_some_and(|s| !s.locked));
        let any_locked = self.sel.iter().any(|id| self.doc.shape(*id).is_some_and(|s| s.locked));
        let grouped = self.sel.iter().any(|id| self.doc.shape(*id).is_some_and(|s| s.group.is_some()));
        let single_image = self.sel.len() == 1 && self.doc.shape(self.sel[0]).is_some_and(|s| s.is_image());
        let chosen: std::cell::Cell<Option<Act>> = std::cell::Cell::new(None);
        // Every entry has its icon from the assets in front of the text.
        let item = |ui: &mut egui::Ui, on: bool, act: Act, label: &'static str| {
            let ink = if on { theme::text() } else { theme::text().gamma_multiply(0.4) };
            let btn = egui::Button::image_and_text(icons::slot(icons::act_icon(act), ink), tr(label));
            if ui.add_enabled(on, btn).clicked() {
                chosen.set(Some(act));
                ui.close();
            }
        };
        item(ui, has_sel, Act::Copy, "Copy");
        item(ui, !self.clipboard.is_empty(), Act::Paste, "Paste");
        item(ui, has_sel, Act::Duplicate, "Duplicate");
        item(ui, any_unlocked, Act::Delete, "Delete");
        item(ui, true, Act::SelectAll, "Select all");
        ui.separator();
        item(ui, multi, Act::Group, "Group");
        item(ui, grouped, Act::Ungroup, "Ungroup");
        item(ui, any_unlocked, Act::Lock, "Lock");
        item(ui, any_locked, Act::Unlock, "Unlock");
        ui.separator();
        // Mirroring and turning, apart from grouping and ordering.
        item(ui, any_unlocked, Act::FlipH, "Flip horizontal");
        item(ui, any_unlocked, Act::FlipV, "Flip vertical");
        item(ui, any_unlocked, Act::RotCw, "Rotate 90° CW");
        item(ui, any_unlocked, Act::RotCcw, "Rotate 90° CCW");
        ui.separator();
        ui.menu_button((icons::slot(Some("shapes"), theme::text()), tr("Arrange")), |ui| {
            item(ui, any_unlocked, Act::ToFront, "Bring to front");
            item(ui, any_unlocked, Act::ToBack, "Send to back");
            ui.separator();
            for (i, n) in ["Align left", "Align centre (H)", "Align right", "Align top", "Align centre (V)", "Align bottom"].into_iter().enumerate() {
                item(ui, any_unlocked, Act::Align(i as u8), n);
            }
            ui.separator();
            item(ui, any_unlocked, Act::CenterOnBed, "Centre on bed");
        });
        ui.menu_button((icons::slot(Some("squares-unite"), theme::text()), tr("Shape operations")), |ui| {
            item(ui, multi, Act::BoolUnion, "Union");
            item(ui, multi, Act::BoolIntersect, "Intersection");
            item(ui, multi, Act::BoolSubtract, "Subtract");
            item(ui, multi, Act::BoolXor, "Exclusive or");
            ui.separator();
            item(ui, has_sel, Act::OffsetShape, "Offset shape…");
            item(ui, any_unlocked, Act::RoundCorners, "Round corners…");
            item(ui, any_unlocked, Act::ToCurves, "Convert to curves");
            item(ui, any_unlocked, Act::ToPath, "Convert to path");
        });
        if single_image {
            ui.separator();
            item(ui, true, Act::AdjustImage, "Adjust image…");
            item(ui, true, Act::TraceImage, "Trace image…");
        }
        chosen.get()
    }

    /// The two selected shapes, if both are single straight lines (end points in design coordinates).
    pub fn selected_line_pair(&self) -> Option<[(u64, (Pt, Pt)); 2]> {
        let ids = self.doc.unlocked_ids(&self.sel);
        if ids.len() != 2 {
            return None;
        }
        let seg = |id: u64| -> Option<(u64, (Pt, Pt))> {
            let s = self.doc.shape(id)?;
            if s.is_image() {
                return None;
            }
            let cs: Vec<_> = s.local_contours().iter().map(|c| c.transformed(&s.xf)).collect();
            lc_core::fillet::as_segment(&cs).map(|l| (id, l))
        };
        Some([seg(ids[0])?, seg(ids[1])?])
    }

    /// Round corners with `radius` mm: two selected straight lines are joined by a rounded corner, otherwise
    /// every straight corner of the selected shapes is rounded.
    pub fn round_corners(&mut self, radius: f64) {
        if radius <= 0.0 {
            return;
        }
        if let Some([(a, la), (b, lb)]) = self.selected_line_pair() {
            let Some(contour) = lc_core::fillet::fillet_lines(la, lb, radius) else {
                self.status_is(tr("These lines cannot be joined (parallel, or the radius does not fit)."));
                return;
            };
            let layer = self.doc.shape(a).map(|s| s.layer).unwrap_or(self.active_layer);
            self.checkpoint();
            self.doc.shapes.retain(|s| s.id != a && s.id != b);
            let id = self.doc.add(layer, Kind::Bezier(vec![contour]), Xf::IDENTITY);
            self.sel = vec![id];
            self.node_sel.clear();
            self.status_is(tr("Joined the two lines with a rounded corner."));
            return;
        }
        let ids: Vec<u64> = self.doc.unlocked_ids(&self.sel).into_iter().filter(|id| self.doc.shape(*id).is_some_and(|s| !s.is_image())).collect();
        if ids.is_empty() {
            self.status_is(tr("Select shapes to round first."));
            return;
        }
        self.checkpoint();
        let mut total = 0;
        for id in ids {
            if let Some(s) = self.doc.unlocked_mut(id) {
                s.to_bezier();
                if let Kind::Bezier(cs) = &mut s.kind {
                    for c in cs.iter_mut() {
                        let (rounded, n) = lc_core::fillet::round_contour(c, radius, None);
                        *c = rounded;
                        total += n;
                    }
                }
            }
        }
        if total == 0 {
            self.undo.pop();
            self.status_is(tr("There are no straight corners to round."));
        } else {
            self.node_sel.clear();
            self.status_is(trf("Rounded {} corner(s).", &[&total]));
        }
    }

    /// Round the selected nodes (node tool) with `radius` mm.
    pub fn round_selected_nodes(&mut self, radius: f64) {
        if self.node_sel.is_empty() || radius <= 0.0 {
            return;
        }
        self.checkpoint();
        let mut by_contour: std::collections::BTreeMap<(u64, usize), std::collections::HashSet<usize>> = Default::default();
        for (id, ci, ni) in &self.node_sel {
            by_contour.entry((*id, *ci)).or_default().insert(*ni);
        }
        let mut total = 0;
        for ((id, ci), only) in by_contour {
            if let Some(c) = self.contour_mut(id, ci) {
                let (rounded, n) = lc_core::fillet::round_contour(c, radius, Some(&only));
                *c = rounded;
                total += n;
            }
        }
        self.node_sel.clear();
        self.active_seg = None;
        if total == 0 {
            self.undo.pop();
            self.status_is(tr("There are no straight corners to round."));
        } else {
            self.status_is(trf("Rounded {} corner(s).", &[&total]));
        }
    }

    pub fn round_window(&mut self, ctx: &egui::Context) {
        if !self.show_round {
            return;
        }
        let mut open = true;
        let mut apply = false;
        let pair = self.selected_line_pair().is_some();
        let units = self.doc.device.units;
        egui::Window::new(tr("Round corners")).open(&mut open).collapsible(false).resizable(false).show(ctx, |ui| {
            egui::Grid::new("round").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                ui.label(tr("Radius"));
                crate::units_ui::drag_len(ui, units, &mut self.fillet_radius, 0.1, Some((0.05, 500.0)));
                ui.end_row();
            });
            ui.label(egui::RichText::new(if pair { tr("Two straight lines are selected: they are joined with a rounded corner.") } else { tr("Every straight corner of the selected shapes is rounded.") }).color(theme::text_dim()));
            apply = ui.button(tr("Round")).clicked();
        });
        if apply {
            self.round_corners(self.fillet_radius);
            open = false;
        }
        self.show_round = open;
    }

    pub fn to_curves(&mut self) {
        if self.sel.is_empty() {
            return;
        }
        self.checkpoint();
        for id in self.sel.clone() {
            if let Some(s) = self.doc.unlocked_mut(id) {
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
        a.sel = vec![id];
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
        assert_eq!(a.doc.shapes.len(), 2);
        let copy = a.doc.shape(a.sel[0]).unwrap();
        assert!(!copy.locked);
        a.transform_selection(Xf::translate(1.0, 0.0));
        assert!(a.doc.shape(a.sel[0]).unwrap().xf != before);
        assert!(a.doc.shape(id).unwrap().locked);
        // Unlock makes it editable again.
        a.sel = vec![id];
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
        a.grid_prefs.bed_color = Some([10, 20, 30, 255]);
        assert_eq!(a.bed_fill(), eframe::egui::Color32::from_rgb(10, 20, 30));
    }

    #[test]
    fn rounding_corners_of_shapes_and_of_two_lines() {
        let mut a = app();
        let r = a.doc.add(0, lc_core::Kind::Rect { w: 20.0, h: 20.0 }, Xf::translate(5.0, 5.0));
        a.sel = vec![r];
        a.round_corners(3.0);
        let before: f64 = 400.0;
        let area = a.doc.shape(r).unwrap().polys()[0].area().abs();
        assert!(area < before && area > before - 4.0 * (1.0 - std::f64::consts::FRAC_PI_4) * 9.0 - 0.5, "{area}");
        // Undo brings the sharp rectangle back.
        a.do_undo();
        assert!(matches!(a.doc.shape(r).unwrap().kind, lc_core::Kind::Rect { .. }));
        // Two lines.
        a.doc.shapes.clear();
        let h = a.doc.add(0, lc_core::Kind::Path(vec![lc_core::Polyline::new(vec![Pt::new(0.0, 10.0), Pt::new(14.0, 10.0)], false)]), Xf::IDENTITY);
        let v = a.doc.add(1, lc_core::Kind::Path(vec![lc_core::Polyline::new(vec![Pt::new(10.0, 0.0), Pt::new(10.0, 25.0)], false)]), Xf::IDENTITY);
        a.sel = vec![h, v];
        assert!(a.selected_line_pair().is_some());
        a.round_corners(3.0);
        assert_eq!(a.doc.shapes.len(), 1, "the two lines became one rounded path");
        let polys = a.doc.shapes[0].polys();
        assert!(polys[0].pts.len() > 4 && !polys[0].closed);
        // Nothing to round: a smooth ellipse leaves the document untouched (and no stray undo step).
        a.doc.shapes.clear();
        let e = a.doc.add(0, lc_core::Kind::Ellipse { w: 10.0, h: 10.0 }, Xf::IDENTITY);
        a.sel = vec![e];
        let undo_len = a.undo.len();
        a.round_corners(1.0);
        assert_eq!(a.undo.len(), undo_len);
        // Locked shapes are skipped.
        let rr = a.doc.add(0, lc_core::Kind::Rect { w: 10.0, h: 10.0 }, Xf::IDENTITY);
        a.sel = vec![rr];
        a.lock_selection(true);
        a.round_corners(1.0);
        assert!(matches!(a.doc.shape(rr).unwrap().kind, lc_core::Kind::Rect { .. }));
    }
}
