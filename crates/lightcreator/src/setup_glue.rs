//! The project setup of LightCreator: the shared window of baseCreator (`base_ui::project_setup`, as in MillCreator) and what it does
//! here: the piece of material, a cylinder on the rotary attachment, and a two-sided job (side A and side B, turned over round the
//! horizontal or the vertical middle line, with alignment marks), and the switch between the sides with the ghosts of the other one.
use crate::app::App;
use lc_core::geom::{Polyline, Pt, Xf};
use lc_core::setup::FlipAxis;
use base_ui::i18n::{tr, trf};
use base_ui::project_setup::{Answer, Kind};
use eframe::egui;
use lc_core::{Kind as ShapeKind, LayerMode, TwoSides};

/// What LightCreator asks of the shared window.
fn kind() -> Kind {
    Kind {
        zero: false,
        cylinder: true,
        marks_tip: "Four corner marks just outside the material, burned on the sheet under it before side A: put the turned-over material back into them for side B.",
        block_note: "The material lies in the corner of the zero of the machine. For a two-sided job lay it in the same place for both sides (the alignment marks show where).",
        cylinder_note: "The workpiece lies on the rotary attachment; Y of the drawing goes round it and the drawing is unrolled from its surface.",
        colour: [0xd8, 0xb0, 0x7c],
        paper: false,
    }
}

/// How far the alignment marks stand off the material, and how long their arms are (mm).
const MARK_GAP: f64 = 2.0;
const MARK_ARM: f64 = 10.0;

impl App {
    /// Open the window with what the project has now (or the work area when it has nothing yet).
    pub fn open_setup(&mut self) {
        let (aw, ah) = self.doc.device.work_size();
        let piece = self.doc.piece.unwrap_or([aw.min(300.0), ah.min(200.0), 3.0]);
        let rot = &self.doc.device.rotary;
        self.setup = base_ui::project_setup::Setup {
            open: true,
            w: piece[0],
            h: piece[1],
            t: piece[2],
            round: rot.enabled,
            diameter: rot.diameter.clamp(1.0, 1000.0),
            length: aw.min(150.0),
            two_sided: self.doc.two.is_some(),
            flip: self.doc.two.as_ref().map_or(FlipAxis::SideToSide, |t| t.flip),
            marks: self.doc.two.is_none(),
            ..Default::default()
        };
    }

    pub fn setup_window(&mut self, ctx: &egui::Context) {
        let units = self.doc.device.units;
        if base_ui::project_setup::window(ctx, &mut self.setup, units, kind()) == Some(Answer::Apply) {
            self.apply_setup();
        }
    }

    /// Put the choices into the project.
    pub fn apply_setup(&mut self) {
        let st = self.setup.clone();
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
        let (w, h, t) = (st.w.clamp(1.0, 20_000.0), st.h.clamp(1.0, 20_000.0), st.t.clamp(0.1, base_ui::project_setup::MAX_THICKNESS));
        self.doc.piece = Some([w, h, t]);
        if st.work_area {
            self.set_work_area(w, h);
        }
        self.status = trf("Project set up: material {} x {} x {} mm, zero at {}", &[&format!("{w:.1}"), &format!("{h:.1}"), &format!("{t:.1}"), &tr("Front left")]);
        if was_rotary {
            self.status = format!("{} {}", self.status, tr("The rotary attachment of the device is off."));
        }
        if st.two_sided {
            let first = self.doc.two.is_none();
            match self.doc.two.as_mut() {
                Some(two) => two.flip = st.flip,
                None => self.doc.two = Some(TwoSides { flip: st.flip, ghosts: true, shown_b: false, other: lc_core::Scene::default() }),
            }
            if first && st.marks {
                self.doc.show_side(false);
                self.add_alignment_marks();
            }
            self.status = format!("{} — {}", self.status, base_ui::project_setup::flip_name(st.flip));
        } else if self.doc.two.is_some() {
            if self.doc.sides().1.is_some_and(|b| !b.shapes.is_empty()) {
                self.status = tr("Side B has shapes: the project stays two-sided. Delete them first to make it one-sided.").to_string();
            } else {
                self.doc.show_side(false);
                self.doc.two = None;
            }
        }
        self.ed.view.need_fit = true;
        self.touch();
    }

    /// The work area of the device in this project made as big as the piece.
    fn set_work_area(&mut self, w: f64, h: f64) {
        let a = &mut self.doc.device.area;
        a.bed_w = w.max(10.0);
        a.bed_h = h.max(10.0);
    }

    /// Four L-shaped marks just outside the corners of the piece, on the last layer (64) as lines: burned on the sheet under the
    /// material before side A, they show where to put it back after it is turned over.
    pub fn add_alignment_marks(&mut self) {
        let (w, h) = self.doc.piece_size();
        let o = self.doc.piece_origin();
        // The last layer (64), out of the way of the layers of the job.
        let Some(layer) = self.doc.layers.len().checked_sub(1) else { return };
        let (g, a) = (MARK_GAP, MARK_ARM);
        let corners = [(-g, -g, 1.0, 1.0), (w + g, -g, -1.0, 1.0), (-g, h + g, 1.0, -1.0), (w + g, h + g, -1.0, -1.0)];
        for (x, y, sx, sy) in corners {
            let (x, y) = (x + o.x, y + o.y);
            let l = Polyline::new(vec![Pt::new(x + sx * a, y), Pt::new(x, y), Pt::new(x, y + sy * a)], false);
            self.doc.add(layer, ShapeKind::Path(vec![l]), Xf::IDENTITY);
        }
        let l = &mut self.doc.layers[layer];
        l.mode = LayerMode::Line;
        l.output = true;
        l.name = tr("Alignment marks").to_string();
        self.status = tr("Alignment marks").to_string();
    }

    /// Show side B (`true`) or side A; undo can take it back.
    pub fn show_side(&mut self, b: bool) {
        if self.doc.two.as_ref().is_some_and(|t| t.shown_b != b) {
            self.checkpoint();
            self.doc.show_side(b);
            self.ed.sel.clear();
            self.ed.ghost_sel.clear();
            self.ed.node_sel.clear();
            self.touch();
        }
    }

    /// The switch over the work area: side A or B, and the ghosts.
    pub fn sides_bar(&mut self, ui: &mut egui::Ui) {
        let Some(t) = self.doc.two.as_mut() else { return };
        let (shown_b, flip) = (t.shown_b, t.flip);
        let changed = base_ui::two_sided::bar(ui, shown_b, &mut t.ghosts, flip);
        if changed.ghosts {
            self.touch();
        }
        if changed.side {
            self.show_side(!shown_b);
        }
    }

    /// The ghosts of the other side as they lie under the shown one (turned over), while they are shown.
    pub fn ghosts(&self) -> Vec<lc_core::Shape> {
        match &self.doc.two {
            Some(t) if t.ghosts => {
                let turn = self.doc.side_turn().unwrap_or(Xf::IDENTITY);
                t.other.shapes.iter().map(|s| lc_core::Shape { xf: s.xf.then(turn), ..s.clone() }).collect()
            }
            _ => vec![],
        }
    }

    /// The question of Save G-code in a two-sided job, and the saving when it is answered.
    pub fn sides_save_window(&mut self, ctx: &egui::Context) {
        let Some(flip) = self.doc.two.as_ref().map(|t| t.flip) else {
            self.sides_save.open = false;
            return;
        };
        if let Some(c) = base_ui::two_sided::save_window(ctx, &mut self.sides_save, flip) {
            self.export_sides(c);
        }
    }

    /// The G-code of both sides of a two-sided job: in one file (side A, a stop to turn the material over or none, side B) or in
    /// two files named `_A` and `_B`. Each side says at its head which one it is.
    pub fn export_sides(&mut self, how: base_ui::two_sided::SaveSides) {
        use base_ui::two_sided::{hold_comment, side_comment, side_paths, SaveSides};
        let Some(flip) = self.doc.two.as_ref().map(|t| t.flip) else { return };
        let side = |b: bool| {
            let mut d = self.doc.clone();
            d.show_side(b);
            d
        };
        let (a, b) = (side(false), side(true));
        let one_by_one = self.one_by_one;
        let opts = |b: bool| lc_core::gcode::JobOptions { one_by_one, side_note: Some(side_comment(b, flip)), then: None };
        self.stamp_job();
        let Some(p) = rfd::FileDialog::new().add_filter("G-code", &["gcode", "nc"]).set_file_name(format!("{}.gcode", self.gcode_name())).save_file() else { return };
        let files: Vec<(std::path::PathBuf, String)> = match how {
            SaveSides::Separate => {
                let (pa, pb) = side_paths(&p);
                vec![(pa, lc_core::gcode::generate_with(&a, None, &opts(false)).gcode), (pb, lc_core::gcode::generate_with(&b, None, &opts(true)).gcode)]
            }
            SaveSides::OneFile | SaveSides::OneFileWithHold => {
                let hold = (how == SaveSides::OneFileWithHold).then(|| hold_comment(flip));
                let then = lc_core::gcode::Then { scene: b.scene.clone(), note: side_comment(true, flip), hold };
                vec![(p, lc_core::gcode::generate_with(&a, None, &lc_core::gcode::JobOptions { then: Some(then), ..opts(false) }).gcode)]
            }
        };
        let mut saved = vec![];
        for (path, text) in files {
            let Some(text) = self.postprocessed(text) else { return };
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

    /// The piece of material where it lies (a faint dashed outline), the ghosts of the other side under the shapes of the shown one,
    /// and the name of the shown side.
    pub fn paint_sides(&self, v: &base_ui::editor::CanvasView) {
        let to = |p: Pt| base_ui::editor::EditorHost::w2s(self, v.origin, p);
        if self.doc.piece.is_some() {
            let (w, h) = self.doc.piece_size();
            let o = self.doc.piece_origin();
            let pts = [o, Pt::new(o.x + w, o.y), Pt::new(o.x + w, o.y + h), Pt::new(o.x, o.y + h), o].map(to);
            let ink = egui::Color32::from_rgba_unmultiplied(0xb0, 0x7a, 0x30, 200);
            v.painter.add(egui::Shape::dashed_line(&pts, egui::Stroke::new(1.2, ink), 8.0, 4.0));
        }
        let Some(t) = &self.doc.two else { return };
        if t.ghosts {
            let ghosts = self.ghosts();
            let colour = base_ui::two_sided::ghost_colour(self.ed.grid_prefs.bed_fill(self.scheme.is_dark()));
            base_ui::two_sided::paint_ghosts(v.painter, &to, &ghosts, colour);
        }
        base_ui::two_sided::paint_side_name(v.painter, v.bed.left_top() + egui::vec2(6.0, 6.0), t.shown_b);
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

    /// A click on a ghost of side A, where it lies under side B (turned over round the middle of the piece, which lies at the zero of
    /// the machine), selects it as the reference; the shape of side B is aligned to it, and the ghost stays where it is.
    #[test]
    fn a_ghost_is_picked_where_it_lies_and_the_shape_aligns_to_it() {
        use base_ui::editor::{EditorCommands, EditorHost};
        let mut a = app();
        a.setup.w = 120.0;
        a.setup.h = 80.0;
        a.setup.two_sided = true;
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

    #[test]
    fn a_two_sided_setup_makes_side_b_with_marks_on_side_a_and_they_are_saved() {
        let mut a = app();
        a.open_setup();
        assert!(a.setup.open && !a.setup.two_sided);
        a.setup.w = 200.0;
        a.setup.h = 100.0;
        a.setup.two_sided = true;
        a.setup.flip = lc_core::setup::FlipAxis::EndToEnd;
        a.apply_setup();
        assert_eq!(a.doc.piece, Some([200.0, 100.0, a.setup.t]));
        assert_eq!(a.doc.scene.shapes.len(), 4, "four corner marks on side A");
        assert!(a.doc.scene.shapes.iter().all(|s| s.layer == a.doc.layers.len() - 1), "on the last layer");
        // The piece lies in the corner of the zero of the machine, and the marks round it.
        let o = a.doc.piece_origin();
        let home = a.doc.device.home_point();
        let (aw, ah) = a.doc.device.work_size();
        let near = |h: f64, lo: f64, len: f64| (h - lo).abs().min((h - (lo + len)).abs()) <= lc_core::PIECE_MARGIN + 1e-6;
        assert!(near(home.x, o.x, 200.0) || aw < 200.0, "{home:?} {o:?}");
        assert!(near(home.y, o.y, 100.0) || ah < 100.0, "{home:?} {o:?}");
        let inside = |v: f64, len: f64| v >= -1e-6 && v <= len + 1e-6;
        let marks = a.doc.scene.shapes.iter().filter_map(|s| s.bounds()).reduce(lc_core::Rect::union).unwrap();
        assert!((marks.min.x - (o.x - 2.0)).abs() < 1e-6 && (marks.max.y - (o.y + 102.0)).abs() < 1e-6, "{marks:?} {o:?}");
        assert!(inside(marks.min.x, aw) && inside(marks.max.x, aw) && inside(marks.min.y, ah) && inside(marks.max.y, ah), "the marks are on the work area");
        a.show_side(true);
        assert!(a.doc.scene.shapes.is_empty() && a.doc.two.as_ref().unwrap().shown_b);
        a.doc.add(0, lc_core::Kind::Rect { w: 10.0, h: 10.0 }, lc_core::Xf::IDENTITY);
        // Undo takes the change of side back (with what was drawn since): side A with its marks again.
        a.do_undo();
        assert!(!a.doc.two.as_ref().unwrap().shown_b && a.doc.scene.shapes.len() == 4);
        a.show_side(true);
        a.doc.add(0, lc_core::Kind::Rect { w: 10.0, h: 10.0 }, lc_core::Xf::IDENTITY);
        // Saved and read back: side A is the drawing of the file, side B its section.
        let back = lc_core::Document::from_json(&a.doc.to_json()).unwrap();
        let (sa, sb) = back.sides();
        assert!(sa.shapes.len() == 4 && sb.is_some_and(|b| b.shapes.len() == 1), "{} {:?}", sa.shapes.len(), sb.map(|b| b.shapes.len()));
        assert!(back.others.sections.keys().all(|k| k != lc_core::setup::SECTION));
        assert_eq!(back.piece, a.doc.piece);
        // One-sided again only while side B is empty.
        a.setup.two_sided = false;
        a.apply_setup();
        assert!(a.doc.two.is_some());
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
}
