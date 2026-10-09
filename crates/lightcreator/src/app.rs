use crate::laser::{self, Cmd, Evt, LaserLink};
use crate::theme;
use eframe::egui::{self, Color32, Key, Modifiers, RichText};
use lc_core::{gcode, svg, Document, Pt, Rect, Shape, Xf};
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tool {
    Select,
    Rect,
    Ellipse,
    Line,
    Pen,
    Pan,
    Zoom,
}

impl Tool {
    pub const ALL: [(Tool, &'static str, Key); 7] = [
        (Tool::Select, "Select (V)", Key::V),
        (Tool::Rect, "Rectangle (R)", Key::R),
        (Tool::Ellipse, "Ellipse (E)", Key::E),
        (Tool::Line, "Line (L)", Key::L),
        (Tool::Pen, "Polyline / pen (P)", Key::P),
        (Tool::Pan, "Pan (H)", Key::H),
        (Tool::Zoom, "Zoom (Z)", Key::Z),
    ];
}

/// Human-facing release label (branch name matches it).
pub const APP_VERSION: &str = "v0.01";

pub struct View {
    pub zoom: f32, // screen px per mm
    pub pan: egui::Vec2,
    pub need_fit: bool,
}

pub struct App {
    pub doc: Document,
    pub undo: Vec<Document>,
    pub redo: Vec<Document>,
    pub sel: Vec<u64>,
    pub tool: Tool,
    pub active_layer: usize,
    pub view: View,
    pub drag: Option<crate::canvas::Drag>,
    pub pen_pts: Vec<Pt>,
    pub clipboard: Vec<Shape>,
    pub path: Option<PathBuf>,
    pub status: String,
    pub revision: u64,
    pub show_grid: bool,
    pub snap: bool,
    pub grid: f64,
    pub preview_on: bool,
    pub preview: Option<(u64, gcode::Job)>,
    pub lock_aspect: bool,
    pub rot_input: f64,
    pub show_all_layers: bool,
    pub cursor_mm: Option<Pt>,
    pub last_layer_undo: f64,
    // dialogs
    pub show_device: bool,
    pub show_array: bool,
    pub array: (u32, u32, f64, f64),
    pub show_about: bool,
    // laser
    pub link: LaserLink,
    pub ports: Vec<String>,
    pub port: String,
    pub connected: bool,
    pub machine: (String, f64, f64),
    pub progress: (usize, usize),
    pub console: Vec<String>,
    pub console_input: String,
    pub jog_step: f64,
    pub jog_feed: f64,
    pub frame_power: f64,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ports = laser::list_ports();
        App {
            doc: Document::default(),
            undo: vec![],
            redo: vec![],
            sel: vec![],
            tool: Tool::Select,
            active_layer: 0,
            view: View { zoom: 2.0, pan: egui::vec2(40.0, 40.0), need_fit: true },
            drag: None,
            pen_pts: vec![],
            clipboard: vec![],
            path: None,
            status: "Ready".into(),
            revision: 0,
            show_grid: true,
            snap: false,
            grid: 10.0,
            preview_on: false,
            preview: None,
            lock_aspect: true,
            rot_input: 15.0,
            show_all_layers: false,
            cursor_mm: None,
            last_layer_undo: -10.0,
            show_device: false,
            show_array: false,
            array: (3, 3, 5.0, 5.0),
            show_about: false,
            link: LaserLink::spawn(cc.egui_ctx.clone()),
            port: ports.first().cloned().unwrap_or_default(),
            ports,
            connected: false,
            machine: ("Disconnected".into(), 0.0, 0.0),
            progress: (0, 0),
            console: vec![],
            console_input: String::new(),
            jog_step: 5.0,
            jog_feed: 3000.0,
            frame_power: 0.0,
        }
    }

    // ---------- document editing helpers ----------
    pub fn touch(&mut self) {
        self.revision += 1;
    }
    pub fn checkpoint(&mut self) {
        self.undo.push(self.doc.clone());
        if self.undo.len() > 100 {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.touch();
    }
    pub fn do_undo(&mut self) {
        if let Some(d) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.doc, d));
            self.sel.retain(|id| self.doc.shape(*id).is_some());
            self.touch();
        }
    }
    pub fn do_redo(&mut self) {
        if let Some(d) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.doc, d));
            self.sel.retain(|id| self.doc.shape(*id).is_some());
            self.touch();
        }
    }
    pub fn sel_bounds(&self) -> Option<Rect> {
        self.doc.bounds_of(&self.sel)
    }
    pub fn transform_selection(&mut self, xf: Xf) {
        for id in self.sel.clone() {
            if let Some(s) = self.doc.shape_mut(id) {
                s.xf = s.xf.then(xf);
            }
        }
        self.touch();
    }
    pub fn delete_selection(&mut self) {
        if self.sel.is_empty() {
            return;
        }
        self.checkpoint();
        let sel = std::mem::take(&mut self.sel);
        self.doc.shapes.retain(|s| !sel.contains(&s.id));
    }
    pub fn copy(&mut self) {
        self.clipboard = self.sel.iter().filter_map(|i| self.doc.shape(*i).cloned()).collect();
    }
    pub fn paste(&mut self) {
        if self.clipboard.is_empty() {
            return;
        }
        self.checkpoint();
        self.sel.clear();
        for mut s in self.clipboard.clone() {
            s.xf = s.xf.then(Xf::translate(5.0, 5.0));
            let id = self.doc.add_shape(s);
            self.sel.push(id);
        }
        // Next paste lands a bit further away.
        self.clipboard = self.sel.iter().filter_map(|i| self.doc.shape(*i).cloned()).collect();
    }
    pub fn duplicate(&mut self) {
        self.copy();
        self.paste();
    }
    pub fn select_all(&mut self) {
        let vis: Vec<u64> = self.doc.shapes.iter().filter(|s| self.doc.layers[s.layer].visible).map(|s| s.id).collect();
        self.sel = vis;
    }
    pub fn assign_layer(&mut self, layer: usize) {
        self.active_layer = layer;
        if !self.sel.is_empty() {
            self.checkpoint();
            for id in self.sel.clone() {
                if let Some(s) = self.doc.shape_mut(id) {
                    s.layer = layer;
                }
            }
        }
    }
    pub fn align(&mut self, mode: u8) {
        let Some(b) = self.sel_bounds() else { return };
        // A single object aligns to the bed, several align to each other.
        let target = if self.sel.len() == 1 {
            Rect { min: Pt::new(0.0, 0.0), max: Pt::new(self.doc.device.bed_w, self.doc.device.bed_h) }
        } else {
            b
        };
        self.checkpoint();
        for id in self.sel.clone() {
            let Some(sb) = self.doc.shape(id).and_then(|s| s.bounds()) else { continue };
            let (dx, dy) = match mode {
                0 => (target.min.x - sb.min.x, 0.0),
                1 => (target.center().x - sb.center().x, 0.0),
                2 => (target.max.x - sb.max.x, 0.0),
                3 => (0.0, target.min.y - sb.min.y),
                4 => (0.0, target.center().y - sb.center().y),
                _ => (0.0, target.max.y - sb.max.y),
            };
            if let Some(s) = self.doc.shape_mut(id) {
                s.xf = s.xf.then(Xf::translate(dx, dy));
            }
        }
    }
    pub fn flip(&mut self, horizontal: bool) {
        if let Some(b) = self.sel_bounds() {
            self.checkpoint();
            let c = b.center();
            self.transform_selection(if horizontal { Xf::scale_about(-1.0, 1.0, c) } else { Xf::scale_about(1.0, -1.0, c) });
        }
    }
    pub fn rotate_sel(&mut self, deg: f64) {
        if let Some(b) = self.sel_bounds() {
            self.checkpoint();
            self.transform_selection(Xf::rotate_about(deg.to_radians(), b.center()));
        }
    }
    pub fn center_on_bed(&mut self) {
        if let Some(b) = self.sel_bounds() {
            self.checkpoint();
            let (c, d) = (b.center(), &self.doc.device);
            let t = Xf::translate(d.bed_w / 2.0 - c.x, d.bed_h / 2.0 - c.y);
            self.transform_selection(t);
        }
    }
    pub fn to_path(&mut self) {
        if self.sel.is_empty() {
            return;
        }
        self.checkpoint();
        for id in self.sel.clone() {
            if let Some(s) = self.doc.shape_mut(id) {
                s.bake();
            }
        }
    }
    pub fn reorder(&mut self, front: bool) {
        self.checkpoint();
        let sel = self.sel.clone();
        let (a, b): (Vec<Shape>, Vec<Shape>) = std::mem::take(&mut self.doc.shapes).into_iter().partition(|s| sel.contains(&s.id));
        self.doc.shapes = if front { [b, a].concat() } else { [a, b].concat() };
    }
    pub fn make_array(&mut self) {
        let (cols, rows, dx, dy) = self.array;
        let Some(b) = self.sel_bounds() else { return };
        self.checkpoint();
        let base: Vec<Shape> = self.sel.iter().filter_map(|i| self.doc.shape(*i).cloned()).collect();
        let mut new_sel = self.sel.clone();
        for r in 0..rows {
            for c in 0..cols {
                if r == 0 && c == 0 {
                    continue;
                }
                let t = Xf::translate(c as f64 * (b.width() + dx), r as f64 * (b.height() + dy));
                for s in &base {
                    let mut s = s.clone();
                    s.xf = s.xf.then(t);
                    new_sel.push(self.doc.add_shape(s));
                }
            }
        }
        self.sel = new_sel;
    }

    // ---------- file operations ----------
    pub fn new_doc(&mut self) {
        let device = self.doc.device.clone();
        self.doc = Document::default();
        self.doc.device = device;
        self.undo.clear();
        self.redo.clear();
        self.sel.clear();
        self.path = None;
        self.touch();
        self.view.need_fit = true;
    }
    pub fn open(&mut self) {
        if let Some(p) = rfd::FileDialog::new().add_filter("LightCreator project", &["lcr"]).add_filter("SVG", &["svg"]).pick_file() {
            self.open_path(p);
        }
    }
    pub fn open_path(&mut self, p: PathBuf) {
        let is_svg = p.extension().is_some_and(|e| e.eq_ignore_ascii_case("svg"));
        if is_svg {
            self.import_svg_path(p);
            return;
        }
        match std::fs::read_to_string(&p).map_err(|e| e.to_string()).and_then(|s| Document::from_json(&s)) {
            Ok(d) => {
                self.doc = d;
                self.undo.clear();
                self.redo.clear();
                self.sel.clear();
                self.status = format!("Opened {}", p.display());
                self.path = Some(p);
                self.touch();
                self.view.need_fit = true;
            }
            Err(e) => self.status = format!("Open failed: {e}"),
        }
    }
    pub fn save(&mut self, as_new: bool) {
        let path = if as_new || self.path.is_none() {
            rfd::FileDialog::new().add_filter("LightCreator project", &["lcr"]).set_file_name("untitled.lcr").save_file()
        } else {
            self.path.clone()
        };
        if let Some(p) = path {
            match std::fs::write(&p, self.doc.to_json()) {
                Ok(_) => {
                    self.status = format!("Saved {}", p.display());
                    self.path = Some(p);
                }
                Err(e) => self.status = format!("Save failed: {e}"),
            }
        }
    }
    pub fn import_svg(&mut self) {
        if let Some(p) = rfd::FileDialog::new().add_filter("SVG", &["svg"]).pick_file() {
            self.import_svg_path(p);
        }
    }
    pub fn import_svg_path(&mut self, p: PathBuf) {
        match std::fs::read(&p).map_err(|e| e.to_string()).and_then(|d| {
            self.checkpoint();
            svg::import(&d, &mut self.doc, None)
        }) {
            Ok(ids) => {
                self.status = format!("Imported {} paths from {}", ids.len(), p.display());
                self.sel = ids;
            }
            Err(e) => self.status = format!("Import failed: {e}"),
        }
    }
    pub fn export_svg(&mut self) {
        if let Some(p) = rfd::FileDialog::new().add_filter("SVG", &["svg"]).set_file_name("design.svg").save_file() {
            self.status = match std::fs::write(&p, svg::export(&self.doc)) {
                Ok(_) => format!("Exported {}", p.display()),
                Err(e) => format!("Export failed: {e}"),
            };
        }
    }
    pub fn export_gcode(&mut self) {
        if let Some(p) = rfd::FileDialog::new().add_filter("G-code", &["gcode", "nc"]).set_file_name("job.gcode").save_file() {
            let job = gcode::generate(&self.doc);
            self.status = match std::fs::write(&p, job.gcode) {
                Ok(_) => format!("Saved G-code {}", p.display()),
                Err(e) => format!("Export failed: {e}"),
            };
        }
    }

    // ---------- laser ----------
    pub fn send_job(&mut self) {
        if !self.connected {
            self.status = "Connect to a laser first (Laser panel)".into();
            return;
        }
        let job = gcode::generate(&self.doc);
        if job.moves.is_empty() {
            self.status = "Nothing to burn: no shapes on output layers".into();
            return;
        }
        let lines = laser::clean_gcode(&job.gcode);
        self.console.push(format!("Starting job: {} lines, ~{}", lines.len(), fmt_time(job.est_seconds)));
        self.link.send(Cmd::Job(lines));
    }
    pub fn frame(&mut self) {
        let b = self.sel_bounds().or_else(|| self.doc.bounds_of(&self.doc.shapes.iter().map(|s| s.id).collect::<Vec<_>>()));
        let Some(b) = b else { return };
        if !self.connected {
            self.status = "Connect to a laser first (Laser panel)".into();
            return;
        }
        let g = gcode::frame_gcode(b, &self.doc.device, self.frame_power, 40.0);
        for l in laser::clean_gcode(&g) {
            self.link.send(Cmd::Line(l));
        }
    }
    fn poll_laser(&mut self) {
        for e in self.link.poll() {
            match e {
                Evt::Log(s) => {
                    self.console.push(s);
                    if self.console.len() > 500 {
                        self.console.remove(0);
                    }
                }
                Evt::Connected(c) => {
                    self.connected = c;
                    if !c {
                        self.machine.0 = "Disconnected".into();
                    }
                }
                Evt::Status { state, x, y } => self.machine = (state, x, y),
                Evt::Progress { done, total } => self.progress = (done, total),
            }
        }
    }

    // ---------- UI ----------
    fn shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        let cmd = |k: Key| ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, k));
        let cmd_shift = |k: Key| ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, k));
        if cmd_shift(Key::Z) || cmd(Key::Y) {
            self.do_redo();
        } else if cmd(Key::Z) {
            self.do_undo();
        }
        if cmd(Key::C) {
            self.copy();
        }
        if cmd(Key::V) {
            self.paste();
        }
        if cmd(Key::D) {
            self.duplicate();
        }
        if cmd(Key::A) {
            self.select_all();
        }
        if cmd(Key::S) {
            self.save(false);
        }
        if cmd(Key::O) {
            self.open();
        }
        if cmd(Key::I) {
            self.import_svg();
        }
        let plain = |k: Key| ctx.input_mut(|i| i.consume_key(Modifiers::NONE, k));
        if plain(Key::Delete) || plain(Key::Backspace) {
            self.delete_selection();
        }
        for (t, _, k) in Tool::ALL {
            if plain(k) {
                self.tool = t;
                self.pen_pts.clear();
            }
        }
        if plain(Key::Escape) {
            self.pen_pts.clear();
            self.sel.clear();
        }
        let step = if ctx.input(|i| i.modifiers.shift) { 10.0 } else { 1.0 };
        for (k, dx, dy) in [(Key::ArrowLeft, -step, 0.0), (Key::ArrowRight, step, 0.0), (Key::ArrowUp, 0.0, -step), (Key::ArrowDown, 0.0, step)] {
            if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, k)) || ctx.input_mut(|i| i.consume_key(Modifiers::SHIFT, k)) {
                if !self.sel.is_empty() {
                    self.checkpoint();
                    self.transform_selection(Xf::translate(dx, dy));
                }
            }
        }
    }

    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New").clicked() {
                    self.new_doc();
                    ui.close();
                }
                if ui.button("Open…        Ctrl+O").clicked() {
                    self.open();
                    ui.close();
                }
                if ui.button("Save          Ctrl+S").clicked() {
                    self.save(false);
                    ui.close();
                }
                if ui.button("Save as…").clicked() {
                    self.save(true);
                    ui.close();
                }
                ui.separator();
                if ui.button("Import SVG…   Ctrl+I").clicked() {
                    self.import_svg();
                    ui.close();
                }
                if ui.button("Export SVG…").clicked() {
                    self.export_svg();
                    ui.close();
                }
                if ui.button("Export G-code…").clicked() {
                    self.export_gcode();
                    ui.close();
                }
            });
            ui.menu_button("Edit", |ui| {
                if ui.add_enabled(!self.undo.is_empty(), egui::Button::new("Undo   Ctrl+Z")).clicked() {
                    self.do_undo();
                    ui.close();
                }
                if ui.add_enabled(!self.redo.is_empty(), egui::Button::new("Redo   Ctrl+Y")).clicked() {
                    self.do_redo();
                    ui.close();
                }
                ui.separator();
                if ui.button("Copy        Ctrl+C").clicked() {
                    self.copy();
                    ui.close();
                }
                if ui.button("Paste       Ctrl+V").clicked() {
                    self.paste();
                    ui.close();
                }
                if ui.button("Duplicate   Ctrl+D").clicked() {
                    self.duplicate();
                    ui.close();
                }
                if ui.button("Delete      Del").clicked() {
                    self.delete_selection();
                    ui.close();
                }
                if ui.button("Select all  Ctrl+A").clicked() {
                    self.select_all();
                    ui.close();
                }
            });
            ui.menu_button("Arrange", |ui| {
                for (i, n) in ["Align left", "Align centre (H)", "Align right", "Align top", "Align centre (V)", "Align bottom"].iter().enumerate() {
                    if ui.button(*n).clicked() {
                        self.align(i as u8);
                        ui.close();
                    }
                }
                ui.separator();
                if ui.button("Centre on bed").clicked() {
                    self.center_on_bed();
                    ui.close();
                }
                if ui.button("Flip horizontal").clicked() {
                    self.flip(true);
                    ui.close();
                }
                if ui.button("Flip vertical").clicked() {
                    self.flip(false);
                    ui.close();
                }
                if ui.button("Rotate 90° CW").clicked() {
                    self.rotate_sel(90.0);
                    ui.close();
                }
                if ui.button("Rotate 90° CCW").clicked() {
                    self.rotate_sel(-90.0);
                    ui.close();
                }
                ui.separator();
                if ui.button("Bring to front").clicked() {
                    self.reorder(true);
                    ui.close();
                }
                if ui.button("Send to back").clicked() {
                    self.reorder(false);
                    ui.close();
                }
                if ui.button("Convert to path").clicked() {
                    self.to_path();
                    ui.close();
                }
                if ui.button("Grid array…").clicked() {
                    self.show_array = true;
                    ui.close();
                }
            });
            ui.menu_button("View", |ui| {
                ui.checkbox(&mut self.show_grid, "Grid");
                ui.checkbox(&mut self.snap, "Snap to grid");
                ui.checkbox(&mut self.preview_on, "Toolpath preview");
                if ui.button("Fit bed to window").clicked() {
                    self.view.need_fit = true;
                    ui.close();
                }
            });
            ui.menu_button("Laser", |ui| {
                if ui.button("Device settings…").clicked() {
                    self.show_device = true;
                    ui.close();
                }
                if ui.button("Frame").clicked() {
                    self.frame();
                    ui.close();
                }
                if ui.button("Start job").clicked() {
                    self.send_job();
                    ui.close();
                }
            });
            ui.menu_button("Help", |ui| {
                if ui.button("About").clicked() {
                    self.show_about = true;
                    ui.close();
                }
            });
        });
    }

    fn control_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_centered(|ui| {
            let (name, _, _) = Tool::ALL.iter().find(|t| t.0 == self.tool).map(|t| (t.1, 0, 0)).unwrap();
            ui.label(RichText::new(name).strong());
            ui.separator();
            let c = lc_core::PALETTE[self.active_layer];
            let (r, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
            ui.painter().rect_filled(r, 3.0, Color32::from_rgb(c[0], c[1], c[2]));
            ui.label(format!("Layer {}", self.doc.layers[self.active_layer].name));
            ui.separator();
            ui.checkbox(&mut self.show_grid, "Grid");
            ui.checkbox(&mut self.snap, "Snap");
            ui.add(egui::DragValue::new(&mut self.grid).range(0.5..=100.0).suffix(" mm"));
            ui.separator();
            if ui.selectable_label(self.preview_on, "Preview").clicked() {
                self.preview_on = !self.preview_on;
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let go = egui::Button::new(RichText::new("   Start   ").color(Color32::WHITE).strong()).fill(theme::ACCENT);
                if ui.add(go).on_hover_text("Stream the job to the connected laser").clicked() {
                    self.send_job();
                }
                if ui.button("Frame").on_hover_text("Trace the bounding box").clicked() {
                    self.frame();
                }
                if ui.button("Save G-code").clicked() {
                    self.export_gcode();
                }
                let (w, h) = self.sel_bounds().map(|b| (b.width(), b.height())).unwrap_or((0.0, 0.0));
                if !self.sel.is_empty() {
                    ui.label(RichText::new(format!("{:.1} × {:.1} mm", w, h)).color(theme::TEXT_DIM));
                }
            });
        });
    }

    fn tool_bar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            for (t, tip, _) in Tool::ALL {
                let (rect, resp) = ui.allocate_exact_size(egui::vec2(36.0, 36.0), egui::Sense::click());
                let on = self.tool == t;
                if on {
                    ui.painter().rect_filled(rect, 6.0, theme::ACCENT);
                } else if resp.hovered() {
                    ui.painter().rect_filled(rect, 6.0, theme::PANEL_DARK);
                }
                crate::icons::paint(ui.painter(), rect.shrink(4.0), t, if on { Color32::WHITE } else { theme::TEXT });
                if resp.on_hover_text(tip).clicked() {
                    self.tool = t;
                    self.pen_pts.clear();
                }
                ui.add_space(2.0);
            }
        });
    }

    fn swatches(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_centered(|ui| {
            ui.label(RichText::new("Layers").color(theme::TEXT_DIM));
            ui.spacing_mut().item_spacing.x = 3.0;
            for i in 0..30 {
                let c = lc_core::PALETTE[i];
                let (r, resp) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::click());
                ui.painter().rect_filled(r, 3.0, Color32::from_rgb(c[0], c[1], c[2]));
                if self.active_layer == i {
                    ui.painter().rect_stroke(r.expand(1.5), 4.0, egui::Stroke::new(2.0, theme::ACCENT), egui::StrokeKind::Outside);
                }
                if resp.on_hover_text(format!("{} — click to assign selection / set active", self.doc.layers[i].name)).clicked() {
                    self.assign_layer(i);
                }
            }
        });
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_centered(|ui| {
            ui.label(&self.status);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("{}%", (self.view.zoom / 2.0 * 100.0).round()));
                if let Some(p) = self.cursor_mm {
                    ui.label(format!("X {:.2}  Y {:.2} mm", p.x, p.y));
                }
                let color = if self.connected { Color32::from_rgb(0x2e, 0xa0, 0x4f) } else { theme::TEXT_DIM };
                ui.label(RichText::new(self.machine.0.clone()).color(color));
                let (r, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                ui.painter().circle_filled(r.center(), 4.0, color);
                if self.progress.1 > 0 && self.progress.0 < self.progress.1 {
                    ui.add(egui::ProgressBar::new(self.progress.0 as f32 / self.progress.1 as f32).desired_width(120.0).show_percentage());
                }
            });
        });
    }

    fn dialogs(&mut self, ctx: &egui::Context) {
        let mut open = self.show_device;
        egui::Window::new("Device settings").open(&mut open).collapsible(false).resizable(false).show(ctx, |ui| {
            let d = &mut self.doc.device;
            egui::Grid::new("dev").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                ui.label("Name");
                ui.text_edit_singleline(&mut d.name);
                ui.end_row();
                ui.label("Work area X (mm)");
                ui.add(egui::DragValue::new(&mut d.bed_w).range(10.0..=5000.0));
                ui.end_row();
                ui.label("Work area Y (mm)");
                ui.add(egui::DragValue::new(&mut d.bed_h).range(10.0..=5000.0));
                ui.end_row();
                ui.label("Machine zero (0,0)");
                egui::ComboBox::from_id_salt("origin")
                    .selected_text(if d.origin == lc_core::Origin::FrontLeft { "Front-left (GRBL default)" } else { "Back-left" })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut d.origin, lc_core::Origin::FrontLeft, "Front-left (GRBL default)");
                        ui.selectable_value(&mut d.origin, lc_core::Origin::BackLeft, "Back-left");
                    });
                ui.end_row();
                ui.label("S-value max ($30)");
                ui.add(egui::DragValue::new(&mut d.s_max).range(1.0..=100000.0));
                ui.end_row();
                ui.label("Dynamic power (M4)");
                ui.checkbox(&mut d.dynamic_power, "");
                ui.end_row();
                ui.label("Travel speed (mm/min)");
                ui.add(egui::DragValue::new(&mut d.travel_speed).range(100.0..=60000.0));
                ui.end_row();
                ui.label("Return to origin");
                ui.checkbox(&mut d.return_home, "");
                ui.end_row();
                ui.label("Baud rate");
                ui.add(egui::DragValue::new(&mut d.baud).range(1200..=1_000_000));
                ui.end_row();
            });
        });
        self.show_device = open;
        if open {
            self.touch();
        }

        let mut open = self.show_array;
        let mut apply = false;
        egui::Window::new("Grid array").open(&mut open).collapsible(false).resizable(false).show(ctx, |ui| {
            egui::Grid::new("arr").num_columns(2).show(ui, |ui| {
                ui.label("Columns");
                ui.add(egui::DragValue::new(&mut self.array.0).range(1..=100));
                ui.end_row();
                ui.label("Rows");
                ui.add(egui::DragValue::new(&mut self.array.1).range(1..=100));
                ui.end_row();
                ui.label("Gap X (mm)");
                ui.add(egui::DragValue::new(&mut self.array.2));
                ui.end_row();
                ui.label("Gap Y (mm)");
                ui.add(egui::DragValue::new(&mut self.array.3));
                ui.end_row();
            });
            apply = ui.button("Create array").clicked();
        });
        if apply {
            self.make_array();
            open = false;
        }
        self.show_array = open;

        let mut open = self.show_about;
        egui::Window::new("About LightCreator").open(&mut open).collapsible(false).resizable(false).show(ctx, |ui| {
            ui.heading("LightCreator");
            ui.label(format!("Version {APP_VERSION}"));
            ui.label("Open-source design & control software for laser engravers and cutters.");
            ui.separator();
            ui.label("Origin: written in Rust with egui/eframe.");
            ui.label("Inspired by LightBurn; interface inspired by VectorCraft.");
            ui.label("Author: Michał Sarna");
            ui.hyperlink("https://github.com/michalsarna/LightCreator");
            ui.label("MIT licensed.");
        });
        self.show_about = open;
    }
}

pub fn fmt_time(s: f64) -> String {
    let s = s as u64;
    format!("{}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.poll_laser();
        if self.preview_on && self.preview.as_ref().map(|p| p.0) != Some(self.revision) {
            self.preview = Some((self.revision, gcode::generate(&self.doc)));
        }
        self.shortcuts(&ctx);

        let chrome = egui::Frame::new().fill(theme::PANEL).stroke(egui::Stroke::new(1.0, theme::BORDER)).inner_margin(egui::Margin::symmetric(8, 4));
        egui::Panel::top("menu").frame(chrome).show(ui, |ui| self.menu_bar(ui));
        egui::Panel::top("controls").frame(chrome).exact_size(36.0).show(ui, |ui| self.control_bar(ui));
        egui::Panel::bottom("status").frame(chrome).exact_size(26.0).show(ui, |ui| self.status_bar(ui));
        egui::Panel::bottom("swatches").frame(chrome).exact_size(36.0).show(ui, |ui| self.swatches(ui));
        egui::Panel::left("tools").frame(chrome).exact_size(52.0).resizable(false).show(ui, |ui| self.tool_bar(ui));
        egui::Panel::right("side").frame(chrome.inner_margin(egui::Margin::same(8))).default_size(330.0).show(ui, |ui| self.side_panel(ui));
        egui::CentralPanel::no_frame().show(ui, |ui| self.canvas(ui));
        self.dialogs(&ctx);
        if self.connected || self.progress.0 < self.progress.1 {
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }
    }
}
