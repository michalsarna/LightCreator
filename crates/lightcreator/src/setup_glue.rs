//! The project setup of LightCreator: the shared window of baseCreator (`base_ui::project_setup`, as in MillCreator) and what it does
//! here: the piece of material, a cylinder on the rotary attachment, and a job on more than one side (side A and side B of a sheet
//! turned over round the horizontal or the vertical middle line, with alignment marks burned as a job of their own; the four faces of
//! a post; the six faces of a block), and the switch between the sides with the ghosts of the others.
use crate::app::App;
use base_ui::i18n::{tr, trf};
use base_ui::project_setup::{Answer, Kind, Sides};
use eframe::egui;
use lc_core::faces::{Body, Relation, Solid};
use lc_core::geom::{Polyline, Pt, Xf};
use lc_core::setup::FlipAxis;
use lc_core::{Kind as ShapeKind, LayerMode, TwoSides};

/// What LightCreator asks of the shared window.
fn kind() -> Kind {
    Kind {
        zero: false,
        cylinder: true,
        marks_tip: "Four corner marks just outside the material, burned as a job of their own on the sheet under it before the material is laid down: put the material into them for side A and, turned over, for side B.",
        block_note: "The material lies in the corner of the zero of the machine. For a two-sided job lay it in the same place for both sides (the alignment marks show where).",
        cylinder_note: "The workpiece lies on the rotary attachment; Y of the drawing goes round it and the drawing is unrolled from its surface.",
        colour: WOOD,
        paper: false,
        solids: true,
    }
}

/// The colour of the material in the pictures and in the 3D view.
pub const WOOD: [u8; 3] = [0xd8, 0xb0, 0x7c];

/// The words at the head of face `i`: its name and size.
fn face_note(solid: &Solid, i: usize) -> String {
    solid.head_comment(i)
}

/// The words of the stop before face `i`: how to lay the object, and on a post or a block that the laser is focused on its top (the
/// faces lie at different heights).
fn hold_note(solid: &Solid, i: usize) -> String {
    let turn = solid.turn_comment(i);
    if solid.count() > 2 {
        format!("{turn}; focus the laser on its top")
    } else {
        turn
    }
}

/// How far the alignment marks stand off the material, and how long their arms are (mm).
const MARK_GAP: f64 = 2.0;
const MARK_ARM: f64 = 10.0;

impl App {
    /// Open the window with what the project has now (or the work area when it has nothing yet).
    pub fn open_setup(&mut self) {
        // Without a piece of its own the project takes the whole work area of the machine.
        let (aw, ah) = self.machine_area();
        let piece = self.doc.piece.unwrap_or([aw, ah, 3.0]);
        let rot = &self.doc.device.rotary;
        self.setup = base_ui::project_setup::Setup {
            open: true,
            w: piece[0],
            h: piece[1],
            t: piece[2],
            round: rot.enabled,
            diameter: rot.diameter.clamp(1.0, 1000.0),
            length: aw.min(150.0),
            flip: self.doc.two.as_ref().map_or(FlipAxis::SideToSide, |t| t.flip()),
            marks: self.doc.two.as_ref().is_none_or(|t| t.marks),
            max: (aw, ah),
            ..Default::default()
        };
        let solid = self.doc.solid();
        self.setup.take_solid(solid.as_ref());
        self.setup.work_area = self.setup.sides.many();
    }

    pub fn setup_window(&mut self, ctx: &egui::Context) {
        let units = self.doc.device.units;
        if base_ui::project_setup::window(ctx, &mut self.setup, units, kind()) == Some(Answer::Apply) {
            self.apply_setup();
        }
    }

    /// Put the choices into the project.
    pub fn apply_setup(&mut self) {
        let mut st = self.setup.clone();
        // A job on more than one side has the material as its work area.
        if st.sides.many() {
            st.work_area = true;
        }
        self.checkpoint();
        if st.round {
            let d = st.diameter.clamp(1.0, 1000.0);
            self.doc.device.rotary.enabled = true;
            self.doc.device.rotary.diameter = d;
            if st.work_area {
                self.set_work_area(st.length, std::f64::consts::PI * d);
            }
            self.sync_profile();
            self.status = trf("Project set up: a cylinder of Ø {} mm and {} mm long on the rotary attachment (switched on), zero at {}", &[&format!("{d:.1}"), &format!("{:.1}", st.length), &tr("Front left")]);
            self.touch();
            return;
        }
        let was_rotary = self.doc.device.rotary.enabled;
        if was_rotary {
            self.doc.device.rotary.enabled = false;
            self.sync_profile();
        }
        if let (Sides::Post | Sides::Block, Some(solid)) = (st.sides, st.solid()) {
            if !self.make_faces(solid) {
                self.status = tr("The faces that would go have shapes: the project stays as it was. Delete them first.").to_string();
                return;
            }
            let s = solid.size;
            let what = if solid.body == Body::Post { tr("a post") } else { tr("a block") };
            self.status = trf("Project set up: {} of {} x {} x {} mm, zero at {}", &[&what, &format!("{:.1}", s[0]), &format!("{:.1}", s[1]), &format!("{:.1}", s[2]), &tr("Front left")]);
        } else {
            let (w, h, t) = (st.w.clamp(1.0, 20_000.0), st.h.clamp(1.0, 20_000.0), st.t.clamp(0.1, base_ui::project_setup::MAX_THICKNESS));
            if st.sides == Sides::Two {
                if !self.make_faces(Solid::sheet(w, h, t, st.flip)) {
                    self.status = tr("The faces that would go have shapes: the project stays as it was. Delete them first.").to_string();
                    return;
                }
                if let Some(two) = self.doc.two.as_mut() {
                    two.marks = st.marks;
                }
                if st.marks {
                    self.ready_marks_layer();
                }
            } else if self.doc.two.is_some() {
                if !self.make_one_sided() {
                    self.status = tr("Side B has shapes: the project stays two-sided. Delete them first to make it one-sided.").to_string();
                    self.touch();
                    return;
                }
            }
            self.doc.piece = Some([w, h, t]);
            if st.work_area {
                self.set_work_area(w, h);
            } else {
                // The work area of the machine again (a job on more than one side, or the material, may have changed it).
                self.restore_machine_area();
            }
            self.status = trf("Project set up: material {} x {} x {} mm, zero at {}", &[&format!("{w:.1}"), &format!("{h:.1}"), &format!("{t:.1}"), &tr("Front left")]);
            if st.sides == Sides::Two {
                self.status = format!("{} — {}", self.status, base_ui::project_setup::flip_name(st.flip));
            }
        }
        if was_rotary {
            self.status = format!("{} {}", self.status, tr("The rotary attachment of the device is off."));
        }
        self.ed.view.need_fit = true;
        self.touch();
    }

    /// Make the project a job on the faces of `solid` (or change its size); the drawings stay. A face that would go must be empty:
    /// false when the project stays as it was.
    pub fn make_faces(&mut self, solid: Solid) -> bool {
        let n = solid.count();
        if let Some(t) = &self.doc.two {
            if (n..t.others.len()).any(|i| !self.doc.face_scene(i).shapes.is_empty()) {
                return false;
            }
            if t.shown >= n {
                self.doc.show_face(0);
            }
        }
        let t = self.doc.two.get_or_insert_with(|| TwoSides::new(solid));
        t.solid = solid;
        t.others.resize(n, lc_core::Scene::default());
        if !t.is_solid() {
            t.marks = false;
        }
        self.doc.apply_face_size();
        self.ed.sel.clear();
        self.ed.ghost_sel.clear();
        true
    }

    /// One side again: only while the other faces are empty (their shapes are never thrown away).
    pub fn make_one_sided(&mut self) -> bool {
        let Some(t) = &self.doc.two else { return true };
        if (1..t.others.len()).any(|i| !self.doc.face_scene(i).shapes.is_empty()) {
            return false;
        }
        self.doc.show_face(0);
        self.doc.two = None;
        true
    }

    /// The work area of the device in this project made as big as the piece.
    fn set_work_area(&mut self, w: f64, h: f64) {
        let a = &mut self.doc.device.area;
        a.bed_w = w.max(10.0);
        a.bed_h = h.max(10.0);
    }

    /// The last layer (64) burns the alignment marks: lines, out of the way of the layers of the job.
    fn ready_marks_layer(&mut self) {
        let Some(layer) = self.doc.layers.len().checked_sub(1) else { return };
        let l = &mut self.doc.layers[layer];
        l.mode = LayerMode::Line;
        l.output = true;
        l.name = tr("Alignment marks").to_string();
    }

    /// Four L-shaped marks just outside the corners of the piece, on the last layer (64): burned as a job of their own on the sheet
    /// under the material before it is laid down, they show where it lies for side A and, turned over, for side B.
    pub fn marks_scene(&self) -> lc_core::Scene {
        let mut sc = lc_core::Scene::default();
        let (w, h) = self.doc.piece_size();
        let o = self.doc.piece_origin();
        let Some(layer) = self.doc.layers.len().checked_sub(1) else { return sc };
        let (g, a) = (MARK_GAP, MARK_ARM);
        let corners = [(-g, -g, 1.0, 1.0), (w + g, -g, -1.0, 1.0), (-g, h + g, 1.0, -1.0), (w + g, h + g, -1.0, -1.0)];
        for (x, y, sx, sy) in corners {
            let (x, y) = (x + o.x, y + o.y);
            let l = Polyline::new(vec![Pt::new(x + sx * a, y), Pt::new(x, y), Pt::new(x, y + sy * a)], false);
            sc.add(layer, ShapeKind::Path(vec![l]), Xf::IDENTITY);
        }
        sc
    }

    /// The G-code of the alignment marks alone (with the postprocessors), when the project has them.
    pub fn marks_text(&mut self) -> Option<String> {
        if !self.doc.two.as_ref().is_some_and(|t| t.marks && !t.is_solid()) {
            return None;
        }
        let mut d = self.doc.clone();
        d.scene = self.marks_scene();
        d.two = None;
        let opts = lc_core::gcode::JobOptions { one_by_one: false, side_note: Some("Alignment marks: burn them on the sheet under the material, then lay the material into them".into()), then: vec![] };
        let text = lc_core::gcode::generate_with(&d, None, &opts).gcode;
        self.postprocessed(text)
    }

    /// Show face `i` (side B of a sheet is 1); undo can take it back.
    pub fn show_face(&mut self, i: usize) {
        if self.doc.two.as_ref().is_some_and(|t| t.shown != i && i < t.others.len()) {
            self.checkpoint();
            let solid = self.doc.two.as_ref().is_some_and(|t| t.is_solid());
            self.doc.show_face(i);
            self.ed.sel.clear();
            self.ed.ghost_sel.clear();
            self.ed.node_sel.clear();
            if solid {
                self.ed.view.need_fit = true;
            }
            self.touch();
        }
    }

    /// Show side B (`true`) or side A of a sheet.
    #[cfg(test)]
    pub fn show_side(&mut self, b: bool) {
        self.show_face(usize::from(b));
    }

    /// The switch over the work area: the side or the face, and the ghosts.
    pub fn sides_bar(&mut self, ui: &mut egui::Ui) {
        let Some(solid) = self.doc.solid() else { return };
        let Some(t) = self.doc.two.as_mut() else { return };
        let shown = t.shown;
        let changed = base_ui::two_sided::faces_bar(ui, &solid, shown, &mut t.ghosts);
        if changed.ghosts {
            self.touch();
        }
        if let Some(i) = changed.face {
            self.show_face(i);
        }
    }

    /// The ghosts of the other faces where they lie to the shown one, while they are shown. On a post or a block each face's ghosts
    /// get ids of their own (the drawings of the faces count their ids apart).
    pub fn ghosts(&self) -> Vec<lc_core::Shape> {
        let Some(t) = self.doc.two.as_ref().filter(|t| t.ghosts) else { return vec![] };
        let many = t.others.len() > 2;
        (0..t.others.len())
            .filter(|j| *j != t.shown)
            .flat_map(|j| {
                let xf = self.doc.ghost_xf(j).unwrap_or(Xf::IDENTITY);
                t.others[j].shapes.iter().map(move |s| lc_core::Shape { xf: s.xf.then(xf), id: if many { s.id + ((j as u64 + 1) << 40) } else { s.id }, ..s.clone() })
            })
            .collect()
    }

    /// The question of Save G-code (or of Preview G-code) in a job on more than one side, and the saving or the showing when it is
    /// answered.
    pub fn sides_save_window(&mut self, ctx: &egui::Context) {
        let Some(solid) = self.doc.solid() else {
            self.sides_save.open = false;
            return;
        };
        let marks = self.doc.two.as_ref().is_some_and(|t| t.marks && !t.is_solid());
        let show = self.sides_save.show;
        if let Some(c) = base_ui::two_sided::save_window_for(ctx, &mut self.sides_save, &solid, marks) {
            if show {
                self.show_sides(c);
            } else {
                self.export_sides(c);
            }
        }
    }

    /// The number of sides or faces of the job (1 without).
    pub fn face_count(&self) -> usize {
        self.doc.two.as_ref().map_or(1, |t| t.others.len())
    }

    /// The G-code of all the sides of the job the way `how` says, with the postprocessors: one text (side A, a stop to turn the
    /// object or none, side B...) or one for each side. Each says at its head which side it is. `None` when a postprocessor failed
    /// (the reason is in the status line).
    pub fn sides_texts(&mut self, how: base_ui::two_sided::SaveSides) -> Option<Vec<String>> {
        use base_ui::two_sided::SaveSides;
        let solid = self.doc.solid()?;
        let n = solid.count();
        let one_by_one = self.one_by_one;
        self.stamp_job();
        let docs: Vec<lc_core::Document> = (0..n).map(|i| self.doc.with_face(i)).collect();
        let opts = |i: usize| lc_core::gcode::JobOptions { one_by_one, side_note: Some(face_note(&solid, i)), then: vec![] };
        let texts = match how {
            SaveSides::Separate => docs.iter().enumerate().map(|(i, d)| lc_core::gcode::generate_with(d, None, &opts(i)).gcode).collect(),
            SaveSides::OneFile | SaveSides::OneFileWithHold => {
                let hold = how == SaveSides::OneFileWithHold;
                // Every face is moved so that the device of face A burns it where the device of its own face would.
                let then = (1..n)
                    .map(|i| {
                        let to_first = if solid.count() > 2 { self.doc.face_to_first(i) } else { Xf::IDENTITY };
                        let mut scene = docs[i].scene.clone();
                        for s in &mut scene.shapes {
                            s.xf = s.xf.then(to_first);
                        }
                        lc_core::gcode::Then { scene, note: face_note(&solid, i), hold: hold.then(|| hold_note(&solid, i)) }
                    })
                    .collect();
                vec![lc_core::gcode::generate_with(&docs[0], None, &lc_core::gcode::JobOptions { then, ..opts(0) }).gcode]
            }
        };
        texts.into_iter().map(|t| self.postprocessed(t)).collect()
    }

    /// The G-code of all the sides of the job: in one file or in one file for each (`_A`, `_B`...), and the alignment marks in a file
    /// of their own (`_marks`) when asked.
    pub fn export_sides(&mut self, how: base_ui::two_sided::SaveSides) {
        use base_ui::two_sided::{face_paths, marks_path, SaveSides};
        let Some(p) = rfd::FileDialog::new().add_filter("G-code", &["gcode", "nc"]).set_file_name(format!("{}.gcode", self.gcode_name())).save_file() else { return };
        let n = self.face_count();
        let Some(mut texts) = self.sides_texts(how) else { return };
        let mut paths = match how {
            SaveSides::Separate => face_paths(&p, n),
            _ => vec![p.clone()],
        };
        if self.sides_save.marks {
            if let Some(m) = self.marks_text() {
                texts.push(m);
                paths.push(marks_path(&p));
            }
        }
        let mut saved = vec![];
        for (path, text) in paths.into_iter().zip(texts) {
            if let Err(e) = std::fs::write(&path, text) {
                self.status = trf("Export failed: {}", &[&e]);
                return;
            }
            saved.push(path);
        }
        self.status = trf("Saved G-code {}", &[&saved.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", ")]);
        if let Some(first) = saved.into_iter().next().filter(|p| self.ed.grid_prefs.gcode_preview_after_save && p.exists()) {
            self.time_gcode_for_device();
            if let Err(e) = base_ui::gcode_view_ui::open(&mut self.gcode_view, first) {
                self.status = e;
            }
        }
    }

    /// The G-code of all the sides of the job in tabs, the way `how` says: one tab with the whole job, or a tab for each side
    /// (`name_A`, `name_B`...), and one for the alignment marks when asked. Nothing is saved.
    pub fn show_sides(&mut self, how: base_ui::two_sided::SaveSides) {
        use base_ui::two_sided::{face_text, text_face, text_slot, SaveSides, MARKS_FACE};
        let Some(texts) = self.sides_texts(how) else { return };
        let stem = self.gcode_name();
        let slot = self.slot;
        let mut tabs: Vec<(usize, String, String)> = if how == SaveSides::Separate {
            texts.into_iter().enumerate().map(|(i, t)| (i, format!("{stem}_{}.gcode", Solid::letter(i)), t)).collect()
        } else {
            texts.into_iter().map(|t| (0, format!("{stem}.gcode"), t)).collect()
        };
        if self.sides_save.marks {
            if let Some(m) = self.marks_text() {
                tabs.push((MARKS_FACE, format!("{stem}_marks.gcode"), m));
            }
        }
        // The tabs of the faces that are not shown now go (unless they have edits).
        let faces: Vec<usize> = tabs.iter().map(|t| t.0).collect();
        self.gcode_texts.retain(|g| text_slot(g.slot) != slot || faces.contains(&text_face(g.slot)) || g.dirty());
        let stamp = self.gcode_stamp();
        for (i, name, text) in tabs {
            let key = face_text(slot, i);
            match self.gcode_texts.iter_mut().find(|g| g.slot == key) {
                Some(g) => {
                    if !g.dirty() {
                        g.replace(text);
                        g.stamp = stamp;
                    }
                    g.name = name;
                    g.sides = Some(how);
                }
                None => {
                    let mut g = base_ui::gcode_text_ui::GcodeText::new(key, &name, text);
                    g.stamp = stamp;
                    g.sides = Some(how);
                    self.gcode_texts.push(g);
                }
            }
        }
        self.panes.select(base_ui::work_panes::Content::GcodeText(slot));
        self.status = tr("The G-code is shown, nothing is saved").to_string();
    }

    /// The piece of material where it lies (a faint dashed outline), the ghosts of the other faces (the one under the shown face on
    /// the work area, those beside it unfolded round it), the net of the faces round it, the alignment marks of a sheet, and the
    /// name of the shown face.
    pub fn paint_sides(&self, v: &base_ui::editor::CanvasView) {
        let to = |p: Pt| base_ui::editor::EditorHost::w2s(self, v.origin, p);
        if self.doc.piece.is_some() {
            let (w, h) = self.doc.piece_size();
            let o = self.doc.piece_origin();
            let pts = [o, Pt::new(o.x + w, o.y), Pt::new(o.x + w, o.y + h), Pt::new(o.x, o.y + h), o].map(to);
            let ink = egui::Color32::from_rgba_unmultiplied(0xb0, 0x7a, 0x30, 200);
            v.painter.add(egui::Shape::dashed_line(&pts, egui::Stroke::new(1.2, ink), 8.0, 4.0));
        }
        let (Some(t), Some(solid)) = (&self.doc.two, self.doc.solid()) else { return };
        let colour = base_ui::two_sided::ghost_colour(self.ed.grid_prefs.bed_fill(self.scheme.is_dark()));
        let outside = base_ui::two_sided::ghost_colour(base_ui::editor::EditorHost::workspace_fill(self));
        if t.ghosts {
            for j in (0..t.others.len()).filter(|j| *j != t.shown) {
                let xf = self.doc.ghost_xf(j).unwrap_or(Xf::IDENTITY);
                let g: Vec<lc_core::Shape> = t.others[j].shapes.iter().map(|s| lc_core::Shape { xf: s.xf.then(xf), ..s.clone() }).collect();
                let c = if solid.relation(j, t.shown) == Relation::Opposite { colour } else { outside };
                base_ui::two_sided::paint_ghosts(v.painter, &to, &g, c);
            }
        }
        if t.others.len() > 2 {
            base_ui::two_sided::paint_net(v.painter, &to, self.doc.piece_origin(), &solid, t.shown, outside);
        }
        if t.marks && !t.is_solid() {
            // The marks are a job of their own: drawn as they will be burned, round the material.
            let ink = egui::Color32::from_rgb(0xd0, 0x40, 0x30);
            for s in self.marks_scene().shapes {
                for p in s.polys() {
                    let pts: Vec<egui::Pos2> = p.pts.iter().map(|q| to(*q)).collect();
                    v.painter.add(egui::Shape::line(pts, egui::Stroke::new(1.6, ink)));
                }
            }
        }
        base_ui::two_sided::paint_face_name(v.painter, v.bed.left_top() + egui::vec2(6.0, 6.0), &solid, t.shown);
    }
}

#[cfg(test)]
mod tests {
    use crate::app::{App, Screen};
    use eframe::egui;

    fn app() -> App {
        let mut a = App::build(&egui::Context::default(), None, false);
        a.screen = Screen::Editor;
        a
    }

    /// Preview G-code of a two-sided job asks how, then shows one tab with the whole job or a tab for each side; the tab of side B is
    /// made again with the project like the other one; Save G-code in such a tab asks how to save.
    #[test]
    fn the_g_code_of_a_two_sided_job_is_shown_whole_or_a_tab_for_each_side() {
        use base_ui::two_sided::{SaveSides, SIDE_B_TEXT};
        let mut a = app();
        a.setup.sides = base_ui::project_setup::Sides::Two;
        a.setup.marks = false;
        a.apply_setup();
        a.doc.add(0, lc_core::Kind::Rect { w: 20.0, h: 10.0 }, lc_core::Xf::translate(10.0, 10.0));
        a.show_side(true);
        a.doc.add(0, lc_core::Kind::Rect { w: 5.0, h: 5.0 }, lc_core::Xf::translate(40.0, 30.0));
        a.preview_gcode();
        assert!(a.sides_save.open && a.sides_save.show, "asked first");
        let slot = a.slot;
        a.show_sides(SaveSides::Separate);
        let tab = |a: &App, key: usize| a.gcode_texts.iter().find(|g| g.slot == key).map(|g| (g.name.clone(), g.text.clone(), g.sides));
        let (ta, tb) = (tab(&a, slot).unwrap(), tab(&a, slot | SIDE_B_TEXT).unwrap());
        assert!(ta.0.ends_with("_A.gcode") && tb.0.ends_with("_B.gcode"));
        assert!(ta.1.contains("; Side A") && !ta.1.contains("Side B") && tb.1.contains("; Side B"));
        // The tab of side B follows the project.
        let before = tb.1.clone();
        a.show_side(true);
        a.doc.add(0, lc_core::Kind::Rect { w: 7.0, h: 7.0 }, lc_core::Xf::translate(60.0, 30.0));
        a.touch();
        a.mark_saved();
        a.refresh_gcode_text_after(std::time::Duration::ZERO);
        assert_ne!(tab(&a, slot | SIDE_B_TEXT).unwrap().1, before, "side B made again");
        // One file: one tab with both sides and the stop; the tab of side B goes.
        a.show_sides(SaveSides::OneFileWithHold);
        assert!(tab(&a, slot | SIDE_B_TEXT).is_none());
        let whole = tab(&a, slot).unwrap();
        assert!(whole.1.contains("; Side A") && whole.1.contains("M0 ;") && whole.1.contains("; Side B"), "{}", whole.1);
        assert_eq!(whole.2, Some(SaveSides::OneFileWithHold));
        // A closed project takes the tab of its side B with it, never another project's.
        assert_eq!(base_ui::two_sided::text_slot(slot | SIDE_B_TEXT), slot);
    }

    /// A click on a ghost of side A, where it lies under side B (turned over round the middle of the piece, which lies at the zero of
    /// the machine), selects it as the reference; the shape of side B is aligned to it, and the ghost stays where it is.
    #[test]
    fn a_ghost_is_picked_where_it_lies_and_the_shape_aligns_to_it() {
        use base_ui::editor::{EditorCommands, EditorHost};
        let mut a = app();
        a.setup.w = 120.0;
        a.setup.h = 80.0;
        a.setup.sides = base_ui::project_setup::Sides::Two;
        a.setup.marks = false;
        a.apply_setup();
        let o = a.doc.piece_origin();
        a.doc.add(0, lc_core::Kind::Rect { w: 20.0, h: 10.0 }, lc_core::Xf::translate(o.x + 10.0, o.y + 10.0));
        a.show_side(true);
        let on_b = a.doc.add(0, lc_core::Kind::Rect { w: 5.0, h: 5.0 }, lc_core::Xf::translate(o.x + 50.0, o.y + 50.0));
        let ghost = a.ghosts()[0].clone();
        let gb = ghost.bounds().unwrap();
        assert!((gb.max.x - (o.x + 110.0)).abs() < 1e-6, "turned over round the middle of the piece: {gb:?} {o:?}");
        let ctx = egui::Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1200.0, 800.0));
        let mut frame = |a: &mut App, events: Vec<egui::Event>| {
            let mut out = ctx.run_ui(egui::RawInput { screen_rect: Some(screen), events, ..Default::default() }, |ui| {
                egui::CentralPanel::no_frame().show(ui, |ui| base_ui::editor::EditorCanvas::canvas(a, ui));
            });
            out.textures_delta.clear();
        };
        frame(&mut a, vec![]);
        frame(&mut a, vec![]);
        let at = a.w2s(a.ed.canvas_origin, gb.center());
        let button = |down: bool| egui::Event::PointerButton { pos: at, button: egui::PointerButton::Primary, pressed: down, modifiers: Default::default() };
        frame(&mut a, vec![egui::Event::PointerMoved(at), button(true), button(false)]);
        frame(&mut a, vec![]);
        assert_eq!(a.ed.ghost_sel, vec![ghost.id], "the ghost is picked");
        assert!(a.ed.sel.is_empty());
        a.ed.sel = vec![on_b];
        a.align(0);
        assert!((a.doc.scene.shape(on_b).unwrap().bounds().unwrap().min.x - gb.min.x).abs() < 1e-6, "left edges together");
        assert_eq!(a.ghosts()[0].bounds(), Some(gb), "the ghost stays");
    }

    /// A two-sided job: the work area is the material; the alignment marks are a job of their own (four corners just outside the
    /// material, on the last layer), saved with the project and in a file of their own beside the files of the sides.
    #[test]
    fn a_two_sided_setup_makes_side_b_and_the_marks_are_a_job_of_their_own() {
        let mut a = app();
        a.open_setup();
        assert!(a.setup.open && a.setup.sides == base_ui::project_setup::Sides::One);
        a.setup.w = 200.0;
        a.setup.h = 100.0;
        a.setup.sides = base_ui::project_setup::Sides::Two;
        a.setup.flip = lc_core::setup::FlipAxis::EndToEnd;
        a.apply_setup();
        assert_eq!(a.doc.piece, Some([200.0, 100.0, a.setup.t]));
        assert_eq!(a.doc.device.work_size(), (200.0, 100.0), "the work area is the material");
        assert!(a.doc.scene.shapes.is_empty(), "the marks are not shapes of side A");
        let t = a.doc.two.as_ref().unwrap();
        assert!(t.marks && t.flip() == lc_core::setup::FlipAxis::EndToEnd);
        let marks = a.marks_scene();
        assert_eq!(marks.shapes.len(), 4);
        assert!(marks.shapes.iter().all(|s| s.layer == a.doc.layers.len() - 1), "on the last layer");
        let o = a.doc.piece_origin();
        let b = marks.shapes.iter().filter_map(|s| s.bounds()).reduce(lc_core::Rect::union).unwrap();
        assert!((b.min.x - (o.x - 2.0)).abs() < 1e-6 && (b.max.y - (o.y + 102.0)).abs() < 1e-6, "{b:?} {o:?}");
        let g = a.marks_text().expect("the marks job");
        assert!(g.contains("; Alignment marks") && g.matches("G1").count() >= 8, "{g}");
        a.show_side(true);
        assert!(a.doc.scene.shapes.is_empty() && a.doc.two.as_ref().unwrap().shown_b());
        a.doc.add(0, lc_core::Kind::Rect { w: 10.0, h: 10.0 }, lc_core::Xf::IDENTITY);
        // Undo takes the change of side back (with what was drawn since): side A again.
        a.do_undo();
        assert!(!a.doc.two.as_ref().unwrap().shown_b() && a.doc.scene.shapes.is_empty());
        a.show_side(true);
        a.doc.add(0, lc_core::Kind::Rect { w: 10.0, h: 10.0 }, lc_core::Xf::IDENTITY);
        // Saved and read back: side A is the drawing of the file, side B and the marks in its section.
        let back = lc_core::Document::from_json(&a.doc.to_json()).unwrap();
        let (sa, sb) = back.sides();
        assert!(sa.shapes.is_empty() && sb.is_some_and(|b| b.shapes.len() == 1));
        assert!(back.others.sections.keys().all(|k| k != lc_core::setup::SECTION));
        assert!(back.two.as_ref().unwrap().marks);
        assert_eq!(back.piece, a.doc.piece);
        // Saved apart: a file for each side and one for the marks.
        a.sides_save.marks = true;
        a.show_sides(base_ui::two_sided::SaveSides::Separate);
        let names: Vec<&str> = a.gcode_texts.iter().map(|g| g.name.as_str()).collect();
        assert!(names.iter().any(|n| n.ends_with("_marks.gcode")) && names.iter().any(|n| n.ends_with("_B.gcode")), "{names:?}");
        // One-sided again only while side B is empty.
        a.setup.sides = base_ui::project_setup::Sides::One;
        a.apply_setup();
        assert!(a.doc.two.is_some());
    }

    /// A post: four faces, each a work area as big as the face with the piece as thick as the post is deep there; the faces beside
    /// the shown one show unfolded round it; in one file each face comes after a stop that says how to roll the post, and is burned
    /// from its own corner of the zero.
    #[test]
    fn a_post_has_four_faces_with_their_own_work_area_and_one_file_rolls_it_between_them() {
        use base_ui::two_sided::SaveSides;
        let mut a = app();
        a.open_setup();
        a.setup.sides = base_ui::project_setup::Sides::Post;
        a.setup.post = [60.0, 40.0, 250.0];
        a.apply_setup();
        assert_eq!((a.doc.device.work_size(), a.doc.piece), ((250.0, 60.0), Some([250.0, 60.0, 40.0])));
        a.doc.add(0, lc_core::Kind::Rect { w: 20.0, h: 10.0 }, lc_core::Xf::translate(10.0, 10.0));
        a.show_face(1);
        assert_eq!((a.doc.device.work_size(), a.doc.piece), ((250.0, 40.0), Some([250.0, 40.0, 60.0])));
        a.doc.add(0, lc_core::Kind::Rect { w: 20.0, h: 10.0 }, lc_core::Xf::translate(10.0, 10.0));
        // On face B face A lies at the back, above the work area.
        let g = a.ghosts();
        assert_eq!(g.len(), 1);
        assert!(g[0].bounds().unwrap().max.y < 0.0, "{:?}", g[0].bounds());
        a.show_face(0);
        let one = a.sides_texts(SaveSides::OneFileWithHold).unwrap();
        let t = &one[0];
        let at = |s: &str| t.find(s).unwrap_or_else(|| panic!("no {s:?} in\n{t}"));
        assert!(at("; Face A of 4") < at("M0 ; Roll it a quarter turn back: face B up") && at("M0 ; Roll it a quarter turn back: face B up") < at("; Face B of 4"));
        assert!(t.contains("M0 ; Roll it a quarter turn back: face B up, face A at the back; focus the laser on its top"), "{t}");
        // Face B in the one file is burned where its own file burns it (from the corner of the zero of its own work area).
        let sep = a.sides_texts(SaveSides::Separate).unwrap();
        assert_eq!(sep.len(), 4);
        let burns = |text: &str| -> Vec<String> { text.lines().filter(|l| l.starts_with("G1 ")).map(|l| l.split(" S").next().unwrap_or(l).to_string()).collect() };
        let in_one = burns(&t[at("; Face B of 4")..]);
        assert!(!in_one.is_empty());
        assert_eq!(in_one, burns(&sep[1]), "face B from its own corner");
        // Saved and read back with the four faces; it opens on face A.
        a.show_face(1);
        let back = lc_core::Document::from_json(&a.doc.to_json()).unwrap();
        let bt = back.two.as_ref().unwrap();
        assert_eq!((bt.shown, bt.others.len()), (0, 4));
        assert_eq!(back.device.work_size(), (250.0, 60.0));
        assert_eq!(back.face_scene(1).shapes.len(), 1);
    }

    #[test]
    fn a_cylinder_switches_the_rotary_on() {
        let mut a = app();
        a.open_setup();
        a.setup.round = true;
        a.setup.diameter = 60.0;
        a.apply_setup();
        assert!(a.doc.device.rotary.enabled && a.doc.device.rotary.diameter == 60.0);
        a.open_setup();
        assert!(a.setup.round);
        a.setup.round = false;
        a.apply_setup();
        assert!(!a.doc.device.rotary.enabled);
    }

    /// A click beside the work area, on a face of the post unfolded there, shows that face.
    #[test]
    fn a_click_on_a_face_unfolded_beside_the_work_area_shows_it() {
        use base_ui::editor::EditorHost;
        let mut a = app();
        a.open_setup();
        a.setup.sides = base_ui::project_setup::Sides::Post;
        a.setup.post = [60.0, 40.0, 250.0];
        a.apply_setup();
        let ctx = egui::Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1200.0, 800.0));
        let mut frame = |a: &mut App, events: Vec<egui::Event>| {
            let mut out = ctx.run_ui(egui::RawInput { screen_rect: Some(screen), events, ..Default::default() }, |ui| {
                egui::CentralPanel::no_frame().show(ui, |ui| base_ui::editor::EditorCanvas::canvas(a, ui));
            });
            out.textures_delta.clear();
        };
        frame(&mut a, vec![]);
        frame(&mut a, vec![]);
        // Face D lies above face A (at its back), 40 mm deep.
        let o = a.doc.piece_origin();
        let at = a.w2s(a.ed.canvas_origin, lc_core::Pt::new(o.x + 100.0, o.y - 15.0));
        let button = |down: bool| egui::Event::PointerButton { pos: at, button: egui::PointerButton::Primary, pressed: down, modifiers: Default::default() };
        frame(&mut a, vec![egui::Event::PointerMoved(at), button(true), button(false)]);
        frame(&mut a, vec![]);
        assert_eq!(a.doc.two.as_ref().unwrap().shown, 3, "face D");
        assert_eq!(a.doc.device.work_size(), (250.0, 40.0));
    }

    /// The work area of the machine stays its own: a post makes the work area of the project as big as a face, but the profile of the
    /// device keeps its size, a new project gets it back, and the setup of one side starts from it.
    #[test]
    fn the_work_area_of_the_machine_stays_its_own_and_one_side_starts_from_it() {
        let mut a = app();
        let mut dev = a.doc.device.clone();
        dev.area.bed_w = 400.0;
        dev.area.bed_h = 300.0;
        a.profiles = vec![dev.clone()];
        a.active = 0;
        a.doc.device = dev;
        a.open_setup();
        assert_eq!((a.setup.w, a.setup.h), (400.0, 300.0), "the whole work area of the machine");
        a.setup.sides = base_ui::project_setup::Sides::Post;
        a.setup.post = [60.0, 40.0, 250.0];
        a.apply_setup();
        assert_eq!(a.doc.device.work_size(), (250.0, 60.0));
        a.sync_profile();
        assert_eq!((a.profiles[0].area.bed_w, a.profiles[0].area.bed_h), (400.0, 300.0), "the device keeps its own");
        a.new_doc();
        assert_eq!(a.doc.device.work_size(), (400.0, 300.0), "a new project has the machine's");
    }
}
