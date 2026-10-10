use crate::laser::{self, Cmd, ConsoleLine, Evt, LaserLink};
use crate::units_ui::{drag_len, fmt_len};
use crate::i18n::{self, tr, trf, Lang};
use crate::menu::{menus, Act, Entry};
use crate::theme::{self, Scheme};
use eframe::egui::{self, Color32, Key, Modifiers, RichText};
use lc_core::{gcode, svg, Document, Pt, Rect, Shape, Xf};
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tool {
    Select,
    Node,
    Rect,
    Ellipse,
    Triangle,
    Star,
    Polygon,
    Heart,
    Line,
    Pen,
    Text,
    Pan,
    Zoom,
    ZoomOut,
}

impl Tool {
    pub const ALL: [(Tool, &'static str, Key); 14] = [
        (Tool::Select, "Select (V)", Key::V),
        (Tool::Node, "Node edit (N)", Key::N),
        (Tool::Rect, "Rectangle (R)", Key::R),
        (Tool::Ellipse, "Ellipse (E)", Key::E),
        (Tool::Triangle, "Triangle (Y)", Key::Y),
        (Tool::Star, "Star (S)", Key::S),
        (Tool::Polygon, "Polygon (G)", Key::G),
        (Tool::Heart, "Heart (K)", Key::K),
        (Tool::Line, "Line (L)", Key::L),
        (Tool::Pen, "Polyline / pen (P)", Key::P),
        (Tool::Text, "Text (T)", Key::T),
        (Tool::Pan, "Pan (H)", Key::H),
        (Tool::Zoom, "Zoom in (Z)", Key::Z),
        (Tool::ZoomOut, "Zoom out (X)", Key::X),
    ];

    /// Selecting and editing tools, kept apart from the tools that insert new objects.
    pub fn is_edit(self) -> bool {
        matches!(self, Tool::Select | Tool::Node)
    }

    /// View navigation tools, kept apart from the drawing and editing tools.
    pub fn is_nav(self) -> bool {
        matches!(self, Tool::Pan | Tool::Zoom | Tool::ZoomOut)
    }
}

/// Human-facing release label (branch name matches it).
pub const APP_VERSION: &str = "v0.0.6";

/// Tabs of the right-hand panel.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SideTab {
    Properties,
    Layers,
    Device,
    Console,
    Clipart,
    /// The camera picture; only listed while the stream is shown as a tab.
    Camera,
}

impl SideTab {
    pub const ALL: [(SideTab, &'static str); 5] = [
        (SideTab::Properties, "Properties"),
        (SideTab::Layers, "Layers"),
        (SideTab::Device, "Device"),
        (SideTab::Console, "Console"),
        (SideTab::Clipart, "Clipart"),
    ];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Start,
    Editor,
}

pub struct View {
    pub zoom: f32, // screen px per mm
    pub pan: egui::Vec2,
    pub need_fit: bool,
    /// Keep the work area fitted to the window as it is resized. Zooming or panning by hand turns this off.
    pub auto_fit: bool,
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
    /// Most recently opened or saved project files, newest first (at most `MAX_RECENT`).
    pub recent: Vec<PathBuf>,
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
    pub screen: Screen,
    pub profiles: Vec<lc_core::Device>,
    pub active: usize,
    pub start_sel: usize,
    pub confirm_delete: bool,
    pub cfg: Option<crate::device_ui::CfgEdit>,
    pub show_array: bool,
    pub array: (u32, u32, f64, f64),
    pub show_about: bool,
    pub node_sel: Vec<crate::nodes::NodeRef>,
    pub active_seg: Option<crate::nodes::NodeRef>,
    pub text_default: lc_core::TextData,
    pub font_filter: String,
    pub focus_text: bool,
    pub image_tex: std::collections::HashMap<(u64, bool), egui::TextureHandle>,
    pub show_offset: bool,
    pub show_materials: bool,
    pub show_prefs: bool,
    /// The node-editing bar is shown while the canvas, the bar or the tool bars were the last thing clicked.
    pub node_bar_active: bool,
    /// The tool to return to when Space (temporary pan) is released.
    pub space_prev: Option<Tool>,
    /// Size in pixels of the canvas as last drawn.
    pub canvas_avail: (f32, f32),
    /// Fit the view to all objects on the next frame.
    pub fit_all_req: bool,
    pub clip: crate::clipart_ui::ClipState,
    pub show_guide: bool,
    pub guide_filter: String,
    pub show_round: bool,
    pub fillet_radius: f64,
    pub esc_time: f64,
    pub layer_dlg: Option<usize>,
    pub show_stream: bool,
    pub ser2net: Option<crate::ser2net_ui::Ser2NetDlg>,
    pub stream_mode: crate::stream_ui::StreamMode,
    pub tab_seen: bool,
    /// Rotation the current stream texture was made with.
    pub stream_rot: u8,
    /// Magnification of the camera picture in its window (1 = fitted) and its shift in pixels.
    pub stream_zoom: f32,
    pub stream_pan: egui::Vec2,
    pub stream: Option<(String, crate::camera_stream::StreamWorker)>,
    pub stream_tex: Option<egui::TextureHandle>,
    pub stream_err: String,
    pub stream_overlay: bool,
    pub polygon_sides: u32,
    pub show_polygon: bool,
    pub read_cfg: Option<crate::device_ui::ReadCfg>,
    pub img_dlg: Option<crate::image_ui::ImgDlg>,
    pub trace_dlg: Option<crate::image_ui::TraceDlg>,
    pub show_preview: bool,
    pub pv: crate::preview_ui::PreviewState,
    pub grid_prefs: crate::prefs::GridPrefs,
    pub overlay: Option<crate::overlay::Overlay>,
    pub overlay_visible: bool,
    pub overlay_edit: bool,
    pub overlay_drag: Option<usize>,
    pub show_overlay: bool,
    pub cam_list: Vec<String>,
    pub cam_index: u32,
    pub cam_error: String,
    pub cam_worker: Option<crate::camera::Worker>,
    pub user_presets: Vec<lc_core::materials::Preset>,
    pub mat_filter: String,
    pub mat_laser: Option<lc_core::LaserKind>,
    pub mat_sel: Option<usize>,
    pub mat_name: String,
    pub mat_op: String,
    pub offset_dist: f64,
    pub offset_keep: bool,
    pub lang: Lang,
    pub scheme: Scheme,
    pub logo: Option<egui::TextureHandle>,
    #[cfg(target_os = "macos")]
    pub native_menu: Option<crate::native_menu::NativeMenu>,
    // laser
    pub link: LaserLink,
    pub ports: Vec<String>,
    pub connected: bool,
    pub machine: (String, f64, f64),
    pub progress: (usize, usize),
    pub console: Vec<ConsoleLine>,
    pub side_tab: SideTab,
    pub console_polls: bool,
    pub console_autoscroll: bool,
    pub console_input: String,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::build(&cc.egui_ctx, cc.storage, true)
    }

    /// Build the app without a window: used by `new` and by the screenshot tests.
    pub fn build(ctx: &egui::Context, storage: Option<&dyn eframe::Storage>, native_menu: bool) -> Self {
        let ports = laser::list_ports();
        let saved = |k: &str| storage.and_then(|st| st.get_string(k));
        let lang = saved("lang").and_then(|c| Lang::from_code(&c)).unwrap_or(Lang::En);
        let scheme = saved("scheme").and_then(|c| Scheme::from_id(&c)).unwrap_or(Scheme::LightDark);
        let profiles: Vec<lc_core::Device> = saved("profiles").and_then(|j| serde_json::from_str(&j).ok()).unwrap_or_default();
        let active = saved("active_profile").and_then(|v| v.parse::<usize>().ok()).filter(|i| *i < profiles.len()).unwrap_or(0);
        let user_presets: Vec<lc_core::materials::Preset> = saved("user_materials").and_then(|j| serde_json::from_str(&j).ok()).unwrap_or_default();
        let grid_prefs: crate::prefs::GridPrefs = saved("grid_prefs").and_then(|j| serde_json::from_str(&j).ok()).unwrap_or_default();
        let show_grid = saved("grid_main_on").map(|v| v == "1").unwrap_or(true);
        let grid = saved("grid_main_mm").and_then(|v| v.parse::<f64>().ok()).filter(|g| *g >= 0.1).unwrap_or(10.0);
        i18n::set_lang(lang);
        theme::apply(ctx, scheme);
        egui_extras::install_image_loaders(ctx);
        crate::fonts::install(ctx);
        lc_core::text::preload();
        App {
            doc: Document::default(),
            undo: vec![],
            redo: vec![],
            sel: vec![],
            tool: Tool::Select,
            active_layer: 0,
            view: View { zoom: 2.0, pan: egui::vec2(40.0, 40.0), need_fit: true, auto_fit: true },
            drag: None,
            pen_pts: vec![],
            clipboard: vec![],
            path: None,
            recent: saved("recent_files").and_then(|j| serde_json::from_str::<Vec<PathBuf>>(&j).ok()).unwrap_or_default().into_iter().take(crate::menu::MAX_RECENT).collect(),
            status: tr("Ready").into(),
            revision: 0,
            show_grid,
            snap: false,
            grid,
            preview_on: false,
            preview: None,
            lock_aspect: true,
            rot_input: 15.0,
            show_all_layers: false,
            cursor_mm: None,
            last_layer_undo: -10.0,
            show_device: false,
            screen: Screen::Start,
            profiles,
            active,
            start_sel: active,
            confirm_delete: false,
            cfg: None,
            show_array: false,
            array: (3, 3, 5.0, 5.0),
            show_about: false,
            node_sel: vec![],
            active_seg: None,
            text_default: lc_core::TextData::default(),
            font_filter: String::new(),
            focus_text: false,
            image_tex: Default::default(),
            show_offset: false,
            show_materials: false,
            show_prefs: false,
            node_bar_active: true,
            space_prev: None,
            canvas_avail: (900.0, 600.0),
            fit_all_req: false,
            clip: crate::clipart_ui::ClipState::new(lc_core::clipart::Library::open(eframe::storage_dir("lightcreator").unwrap_or_else(std::env::temp_dir).join("clipart"))),
            show_guide: false,
            guide_filter: String::new(),
            show_round: false,
            fillet_radius: 2.0,
            esc_time: -10.0,
            layer_dlg: None,
            show_stream: false,
            ser2net: None,
            stream_mode: crate::stream_ui::StreamMode::Floating,
            tab_seen: false,
            stream_rot: 0,
            stream_zoom: 1.0,
            stream_pan: egui::Vec2::ZERO,
            stream: None,
            stream_tex: None,
            stream_err: String::new(),
            stream_overlay: false,
            polygon_sides: 6,
            show_polygon: false,
            read_cfg: None,
            img_dlg: None,
            trace_dlg: None,
            show_preview: false,
            pv: Default::default(),
            grid_prefs,
            overlay: None,
            overlay_visible: true,
            overlay_edit: false,
            overlay_drag: None,
            show_overlay: false,
            cam_list: vec![],
            cam_index: 0,
            cam_error: String::new(),
            cam_worker: None,
            user_presets,
            mat_filter: String::new(),
            mat_laser: None,
            mat_sel: None,
            mat_name: String::new(),
            mat_op: "Cut".into(),
            offset_dist: -1.0,
            offset_keep: true,
            lang,
            scheme,
            logo: crate::load_logo(ctx),
            #[cfg(target_os = "macos")]
            native_menu: native_menu.then(|| crate::native_menu::NativeMenu::install(ctx)),
            link: LaserLink::spawn(ctx.clone()),
            ports,
            connected: false,
            machine: (tr("Disconnected").into(), 0.0, 0.0),
            progress: (0, 0),
            console: vec![],
            side_tab: SideTab::Properties,
            console_polls: false,
            console_autoscroll: true,
            console_input: String::new(),
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
            if let Some(s) = self.doc.unlocked_mut(id) {
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
        // Locked shapes survive and stay selected.
        self.doc.shapes.retain(|s| !sel.contains(&s.id) || s.locked);
        self.sel = sel.into_iter().filter(|id| self.doc.shape(*id).is_some()).collect();
    }
    pub fn copy(&mut self) {
        // A copy of a locked object is a normal, editable object.
        self.clipboard = self.sel.iter().filter_map(|i| self.doc.shape(*i).cloned()).map(|mut s| { s.locked = false; s }).collect();
    }
    pub fn paste(&mut self) {
        if self.clipboard.is_empty() {
            return;
        }
        self.checkpoint();
        self.sel.clear();
        // Pasted groups get their own group ids so they do not merge with the originals.
        let mut gmap: std::collections::HashMap<u64, u64> = Default::default();
        for mut s in self.clipboard.clone() {
            s.xf = s.xf.then(Xf::translate(5.0, 5.0));
            if let Some(g) = s.group {
                let doc = &mut self.doc;
                s.group = Some(*gmap.entry(g).or_insert_with(|| doc.new_group_id()));
            }
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
                if let Some(s) = self.doc.unlocked_mut(id) {
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
            if let Some(s) = self.doc.unlocked_mut(id) {
                s.xf = s.xf.then(Xf::translate(dx, dy));
            }
        }
    }
    /// Move the selected objects so that their centres coincide, at the centre of the whole selection. A group
    /// counts as one object. With the small shape inside the large one, the large one stays where it is.
    pub fn center_on_each_other(&mut self) {
        let Some(all) = self.sel_bounds() else { return };
        let target = all.center();
        // Objects are single shapes or whole groups.
        let mut units: Vec<Vec<u64>> = vec![];
        for id in self.sel.clone() {
            if units.iter().any(|u| u.contains(&id)) {
                continue;
            }
            units.push(self.doc.group_of(id));
        }
        if units.len() < 2 {
            return;
        }
        self.checkpoint();
        for unit in units {
            let Some(b) = self.doc.bounds_of(&unit) else { continue };
            let t = Xf::translate(target.x - b.center().x, target.y - b.center().y);
            for id in unit {
                if let Some(s) = self.doc.unlocked_mut(id) {
                    s.xf = s.xf.then(t);
                }
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
            if let Some(s) = self.doc.unlocked_mut(id) {
                s.bake();
            }
        }
    }
    pub fn reorder(&mut self, front: bool) {
        self.checkpoint();
        let sel = self.doc.unlocked_ids(&self.sel);
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
    /// Put a file at the top of the recent list (without duplicates, at most `MAX_RECENT`).
    pub fn add_recent(&mut self, p: &std::path::Path) {
        let p = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
        self.recent.retain(|r| *r != p);
        self.recent.insert(0, p);
        self.recent.truncate(crate::menu::MAX_RECENT);
    }
    pub fn open_path(&mut self, p: PathBuf) {
        let is_svg = p.extension().is_some_and(|e| e.eq_ignore_ascii_case("svg"));
        if is_svg {
            if self.import_svg_path(p.clone()) {
                self.add_recent(&p);
            }
            return;
        }
        match std::fs::read_to_string(&p).map_err(|e| e.to_string()).and_then(|s| Document::from_json(&s)) {
            Ok(d) => {
                self.doc = d;
                if let Some(p) = self.profiles.get(self.active) {
                    self.doc.device = p.clone();
                }
                self.undo.clear();
                self.redo.clear();
                self.sel.clear();
                self.status = trf("Opened {}", &[&p.display()]);
                self.add_recent(&p);
                self.path = Some(p);
                self.touch();
                self.view.need_fit = true;
            }
            Err(e) => self.status = trf("Open failed: {}", &[&e]),
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
                    self.status = trf("Saved {}", &[&p.display()]);
                    self.add_recent(&p);
                    self.path = Some(p);
                }
                Err(e) => self.status = trf("Save failed: {}", &[&e]),
            }
        }
    }
    pub fn import_svg(&mut self) {
        if let Some(p) = rfd::FileDialog::new().add_filter("SVG", &["svg"]).pick_file() {
            self.import_svg_path(p);
        }
    }
    pub fn import_svg_path(&mut self, p: PathBuf) -> bool {
        match std::fs::read(&p).map_err(|e| e.to_string()).and_then(|d| {
            self.checkpoint();
            svg::import(&d, &mut self.doc, None)
        }) {
            Ok(ids) => {
                self.status = trf("Imported {} paths from {}", &[&ids.len(), &p.display()]);
                self.sel = ids;
                true
            }
            Err(e) => {
                self.status = trf("Import failed: {}", &[&e]);
                false
            }
        }
    }
    pub fn export_svg(&mut self) {
        if let Some(p) = rfd::FileDialog::new().add_filter("SVG", &["svg"]).set_file_name("design.svg").save_file() {
            self.status = match std::fs::write(&p, svg::export(&self.doc)) {
                Ok(_) => trf("Exported {}", &[&p.display()]),
                Err(e) => trf("Export failed: {}", &[&e]),
            };
        }
    }
    pub fn export_gcode(&mut self) {
        if let Some(p) = rfd::FileDialog::new().add_filter("G-code", &["gcode", "nc"]).set_file_name("job.gcode").save_file() {
            let job = gcode::generate(&self.doc);
            self.status = match std::fs::write(&p, job.gcode) {
                Ok(_) => trf("Saved G-code {}", &[&p.display()]),
                Err(e) => trf("Export failed: {}", &[&e]),
            };
        }
    }

    // ---------- laser ----------
    /// Export the vector job as HPGL (.plt) or DXF for file-fed controllers (Ruida / Trocen software).
    pub fn export_cam(&mut self) {
        let Some(p) = rfd::FileDialog::new().add_filter("HPGL plot", &["plt"]).add_filter("DXF", &["dxf"]).set_file_name("job.plt").save_file() else { return };
        let is_dxf = p.extension().is_some_and(|e| e.eq_ignore_ascii_case("dxf"));
        let data = if is_dxf { lc_core::export::dxf(&self.doc) } else { lc_core::export::hpgl(&self.doc) };
        let skipped = lc_core::export::skipped_images(&self.doc);
        self.status = match std::fs::write(&p, data) {
            Ok(_) if skipped > 0 => trf("Exported {}. {} image(s) skipped: this format has no raster.", &[&p.display(), &skipped]),
            Ok(_) => trf("Exported {}", &[&p.display()]),
            Err(e) => trf("Export failed: {}", &[&e]),
        };
    }

    pub fn send_job(&mut self) {
        if !self.doc.device.controller.is_serial() {
            self.export_cam();
            return;
        }
        if !self.connected {
            self.status = tr("Connect to a laser first (Laser panel)").into();
            return;
        }
        let job = gcode::generate(&self.doc);
        if job.moves.is_empty() {
            self.status = tr("Nothing to burn: no shapes on output layers").into();
            return;
        }
        let lines = laser::clean_gcode(&job.gcode);
        self.push_console(ConsoleLine::info(trf("Starting job: {} lines, ~{}", &[&lines.len(), &fmt_time(job.est_seconds)])));
        self.link.send(Cmd::Job(lines));
    }
    /// Start can be pressed: a streaming controller needs a connected laser, the file-based ones (Ruida, Trocen)
    /// just export the job.
    pub fn can_start(&self) -> bool {
        !self.doc.device.controller.is_serial() || self.connected
    }
    /// (shown, available) for the Overlay buttons.
    pub fn overlay_state(&self) -> (bool, bool) {
        (self.overlay.is_some() && self.overlay_visible, self.overlay.is_some() || self.show_stream)
    }
    /// The overlay is fed from the live camera while its window is open, otherwise it is the saved photo.
    pub fn toggle_overlay_button(&mut self) {
        let (on, _) = self.overlay_state();
        if self.show_stream {
            if on && self.stream_overlay {
                self.stream_overlay = false;
                self.overlay_visible = false;
            } else {
                self.stream_overlay = true;
                self.overlay_visible = true;
            }
        } else {
            self.overlay_visible = !self.overlay_visible;
        }
    }
    /// A job is being streamed to the laser.
    pub fn job_running(&self) -> bool {
        self.progress.1 > 0 && self.progress.0 < self.progress.1
    }
    pub fn frame(&mut self) {
        if self.job_running() {
            return;
        }
        let b = self.sel_bounds().or_else(|| self.doc.bounds_of(&self.doc.shapes.iter().map(|s| s.id).collect::<Vec<_>>()));
        let Some(b) = b else { return };
        if !self.connected {
            self.status = tr("Connect to a laser first (Laser panel)").into();
            return;
        }
        let g = gcode::frame_gcode(b, &self.doc.device, self.doc.device.frame_power, 40.0);
        for l in laser::clean_gcode(&g) {
            self.link.send(Cmd::Line(l));
        }
    }
    pub fn push_console(&mut self, l: ConsoleLine) {
        self.console.push(l);
        if self.console.len() > 4000 {
            self.console.drain(..1000);
        }
    }

    fn poll_laser(&mut self) {
        for e in self.link.poll() {
            match e {
                Evt::Log(s) => self.push_console(ConsoleLine::info(s)),
                Evt::Traffic(l) => {
                    if l.dir == crate::laser::Dir::Rx && !l.poll {
                        self.read_feed(&l.text);
                    }
                    self.push_console(l)
                }
                Evt::Connected(c) => {
                    self.connected = c;
                    self.read_on_connected(c);
                    if !c {
                        self.machine.0 = tr("Disconnected").into();
                    }
                }
                Evt::Status { state, x, y } => self.machine = (state, x, y),
                Evt::Progress { done, total } => self.progress = (done, total),
            }
        }
    }

    // ---------- UI ----------
    /// A second Escape within half a second goes back to the select tool.
    pub fn note_escape(&mut self, now: f64) {
        if now - self.esc_time < 0.5 {
            self.tool = Tool::Select;
            self.esc_time = -10.0;
        } else {
            self.esc_time = now;
        }
    }

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
        if cmd_shift(Key::G) {
            self.ungroup_selection();
        } else if cmd(Key::G) {
            self.group_selection();
        }
        if cmd(Key::Num0) {
            self.view.need_fit = true;
            self.view.auto_fit = true;
        }
        if cmd(Key::Num9) {
            self.view.auto_fit = false;
            self.fit_all_req = true;
        }
        if cmd(Key::Equals) || cmd(Key::Plus) {
            self.zoom_about_centre(1.5);
        }
        if cmd(Key::Minus) {
            self.zoom_about_centre(1.0 / 1.5);
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F1)) {
            self.show_guide = true;
        }
        if cmd_shift(Key::L) {
            self.lock_selection(false);
        } else if cmd(Key::L) {
            self.lock_selection(true);
        }
        if cmd(Key::A) {
            self.select_all();
        }
        // On macOS the native menu bar owns New / Open / Save / Save as / Import accelerators.
        #[cfg(not(target_os = "macos"))]
        {
            if cmd_shift(Key::S) {
                self.save(true);
            } else if cmd(Key::S) {
                self.save(false);
            }
            if cmd(Key::O) {
                self.open();
            }
            if cmd(Key::I) {
                self.import_svg();
            }
            if cmd(Key::N) {
                self.new_doc();
            }
            if cmd(Key::Q) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        let plain = |k: Key| ctx.input_mut(|i| i.consume_key(Modifiers::NONE, k));
        if plain(Key::Delete) || plain(Key::Backspace) {
            if self.tool == Tool::Node && !self.node_sel.is_empty() {
                self.delete_nodes();
            } else {
                self.delete_selection();
            }
        }
        for (t, _, k) in Tool::ALL {
            if plain(k) {
                self.tool = t;
                self.pen_pts.clear();
                if t == Tool::Polygon {
                    self.show_polygon = true;
                }
            }
        }
        if plain(Key::Escape) {
            let now = ctx.input(|i| i.time);
            self.note_escape(now);
            self.pen_pts.clear();
            if self.tool == Tool::Node && !self.node_sel.is_empty() {
                self.node_sel.clear();
            } else {
                self.sel.clear();
            }
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

    /// Is this action's check mark set? (None = not a check item.)
    pub fn act_checked(&self, a: Act) -> Option<bool> {
        match a {
            Act::ToggleGrid => Some(self.show_grid),
            Act::ToggleSnap => Some(self.snap),
            Act::TogglePreview => Some(self.preview_on),
            Act::ToggleOverlay => Some(self.overlay.is_some() && self.overlay_visible),
            Act::ToggleAutoFit => Some(self.view.auto_fit),
            Act::SetLang(l) => Some(self.lang == l),
            Act::SetScheme(s) => Some(self.scheme == s),
            _ => None,
        }
    }

    pub fn act_enabled(&self, a: Act) -> bool {
        match a {
            Act::Undo => !self.undo.is_empty(),
            Act::Redo => !self.redo.is_empty(),
            Act::ClearRecent => !self.recent.is_empty(),
            Act::Frame => self.connected && !self.job_running(),
            Act::StartJob => self.can_start() && !self.job_running(),
            Act::CenterEachOther => self.sel.len() >= 2,
            Act::ToggleOverlay => self.overlay.is_some(),
            _ => true,
        }
    }

    pub fn do_act(&mut self, ctx: &egui::Context, a: Act) {
        // On the start screen only application-level actions are available.
        if self.screen == Screen::Start && !matches!(a, Act::Quit | Act::About | Act::SetLang(_) | Act::SetScheme(_)) {
            return;
        }
        match a {
            Act::New => self.new_doc(),
            Act::Open => self.open(),
            Act::OpenRecent(i) => {
                if let Some(p) = self.recent.get(i as usize).cloned() {
                    if p.is_file() {
                        self.open_path(p);
                    } else {
                        self.status = trf("Open failed: {}", &[&format!("{} {}", p.display(), tr("no longer exists"))]);
                        self.recent.retain(|r| *r != p);
                    }
                }
            }
            Act::ClearRecent => self.recent.clear(),
            Act::Save => self.save(false),
            Act::SaveAs => self.save(true),
            Act::ImportSvg => self.import_svg(),
            Act::ExportSvg => self.export_svg(),
            Act::ExportGcode => self.export_gcode(),
            Act::ExportCam => self.export_cam(),
            Act::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            Act::Undo => self.do_undo(),
            Act::Redo => self.do_redo(),
            Act::Copy => self.copy(),
            Act::Paste => self.paste(),
            Act::Duplicate => self.duplicate(),
            Act::Delete => self.delete_selection(),
            Act::SelectAll => self.select_all(),
            Act::Align(m) => self.align(m),
            Act::CenterOnBed => self.center_on_bed(),
            Act::CenterEachOther => self.center_on_each_other(),
            Act::FlipH => self.flip(true),
            Act::FlipV => self.flip(false),
            Act::RotCw => self.rotate_sel(90.0),
            Act::RotCcw => self.rotate_sel(-90.0),
            Act::ToFront => self.reorder(true),
            Act::ToBack => self.reorder(false),
            Act::ToPath => self.to_path(),
            Act::ToCurves => self.to_curves(),
            Act::EditNodes => self.start_node_edit(),
            Act::Group => self.group_selection(),
            Act::Lock => self.lock_selection(true),
            Act::Unlock => self.lock_selection(false),
            Act::Ungroup => self.ungroup_selection(),
            Act::ImportAi => self.import_ai(),
            Act::OnlineClipart => {
                self.side_tab = SideTab::Clipart;
                self.open_online_clipart(ctx);
            }
            Act::TraceImage => self.open_trace(),
            Act::AdjustImage => self.open_adjust(),
            Act::BoolUnion => self.bool_op(lc_core::ops::BoolOp::Union),
            Act::BoolIntersect => self.bool_op(lc_core::ops::BoolOp::Intersect),
            Act::BoolSubtract => self.bool_op(lc_core::ops::BoolOp::Difference),
            Act::BoolXor => self.bool_op(lc_core::ops::BoolOp::Xor),
            Act::OffsetShape => self.show_offset = true,
            Act::RoundCorners => self.show_round = true,
            Act::QuickGuide => self.show_guide = true,
            Act::OpenGithub => ctx.open_url(egui::OpenUrl::new_tab(crate::help_ui::GITHUB_URL)),
            Act::ImportImage => self.import_image(),
            Act::CameraOverlay => self.show_overlay = true,
            Act::GridOptions => self.show_prefs = true,
            Act::PreviewWindow => self.show_preview = true,
            Act::ShareSer2net => self.open_ser2net(),
            Act::ToggleOverlay => {
                if self.overlay.is_some() {
                    self.overlay_visible = !self.overlay_visible;
                } else {
                    self.status = tr("There is no camera overlay yet. Load a picture or start the camera view.").to_string();
                }
            }
            Act::CameraView => {
                if self.doc.device.camera_url.trim().is_empty() {
                    self.status = tr("Set a camera URL in the device settings first.").to_string();
                } else {
                    self.show_stream = true;
                }
            }
            Act::MaterialLibrary => {
                self.mat_laser = Some(self.doc.device.laser);
                self.show_materials = true;
            }
            Act::GridArray => self.show_array = true,
            Act::ToggleGrid => self.show_grid = !self.show_grid,
            Act::ToggleSnap => self.snap = !self.snap,
            Act::TogglePreview => self.preview_on = !self.preview_on,
            Act::FitBed => {
                self.view.need_fit = true;
                self.view.auto_fit = true;
            }
            Act::FitAll => {
                self.view.auto_fit = false;
                self.fit_all_req = true;
            }
            Act::ZoomIn => self.zoom_about_centre(1.5),
            Act::ZoomOut => self.zoom_about_centre(1.0 / 1.5),
            Act::ToggleAutoFit => {
                self.view.auto_fit = !self.view.auto_fit;
                if self.view.auto_fit {
                    self.view.need_fit = true;
                }
            }
            Act::DeviceSettings => self.show_device = true,
            Act::SwitchDevice => {
                self.start_sel = self.active;
                self.confirm_delete = false;
                self.screen = Screen::Start;
            }
            Act::Frame => self.frame(),
            Act::StartJob => self.send_job(),
            Act::About => self.show_about = true,
            Act::SetLang(l) => {
                self.lang = l;
                i18n::set_lang(l);
            }
            Act::SetScheme(sc) => {
                self.scheme = sc;
                theme::apply(ctx, sc);
            }
        }
    }

    fn menu_entries(&mut self, ui: &mut egui::Ui, entries: &[Entry], pending: &mut Option<Act>) {
        for e in entries {
            match e {
                Entry::Sep => {
                    ui.separator();
                }
                Entry::Sub(title, children) => {
                    let ink = theme::text();
                    ui.menu_button((crate::icons::slot(crate::icons::submenu_icon(title), ink), tr(title)), |ui| self.menu_entries(ui, children, pending));
                }
                Entry::Recent => {
                    let ink = theme::text();
                    let recent = self.recent.clone();
                    ui.menu_button((crate::icons::slot(Some("history"), ink), tr("Open recent")), |ui| {
                        if recent.is_empty() {
                            ui.add_enabled(false, egui::Button::new(tr("(empty)")));
                        }
                        for (i, p) in recent.iter().enumerate() {
                            let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| p.display().to_string());
                            let btn = egui::Button::image_and_text(crate::icons::slot(Some("file"), ink), name);
                            if ui.add(btn).on_hover_text(p.display().to_string()).clicked() {
                                *pending = Some(Act::OpenRecent(i as u8));
                                ui.close();
                            }
                        }
                        if !recent.is_empty() {
                            ui.separator();
                            let btn = egui::Button::image_and_text(crate::icons::slot(Some("trash"), ink), tr("Clear list"));
                            if ui.add(btn).clicked() {
                                *pending = Some(Act::ClearRecent);
                                ui.close();
                            }
                        }
                    });
                }
                Entry::Item(act, label, accel) => {
                    let enabled = self.act_enabled(*act);
                    let ink = if enabled { theme::text() } else { theme::text().gamma_multiply(0.4) };
                    // A tick marks switched-on options; other entries show their own icon.
                    let (icon, ink) = match self.act_checked(*act) {
                        Some(true) => (Some("check"), ink),
                        // Switched off: the entry's own icon, faded, so every entry has one.
                        Some(false) => {
                            let own = match act {
                                Act::SetLang(_) => Some("languages"),
                                Act::SetScheme(_) => Some("palette"),
                                _ => crate::icons::act_icon(*act),
                            };
                            (own, ink.gamma_multiply(0.45))
                        }
                        None => (crate::icons::act_icon(*act), ink),
                    };
                    let mut btn = egui::Button::image_and_text(crate::icons::slot(icon, ink), tr(label));
                    if let Some((_, shown)) = accel {
                        btn = btn.shortcut_text(*shown);
                    }
                    if ui.add_enabled(enabled, btn).clicked() {
                        *pending = Some(*act);
                        ui.close();
                    }
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    fn native_menu_frame(&mut self, ctx: &egui::Context) {
        let Some(mut nm) = self.native_menu.take() else { return };
        for a in nm.take_actions() {
            self.do_act(ctx, a);
        }
        nm.sync(|a| self.act_checked(a), |a| self.act_enabled(a), &self.recent);
        self.native_menu = Some(nm);
    }

    /// In-window menu bar (Windows / Linux, and macOS when no native menu is installed).
    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        let mut pending = None;
        egui::MenuBar::new().ui(ui, |ui| {
            for (title, entries) in menus() {
                let icon = crate::icons::menu_title_icon(title).and_then(|n| crate::icons::image(n, 15.0, theme::text()));
                match icon {
                    Some(img) => ui.menu_button((img, tr(title)), |ui| self.menu_entries(ui, &entries, &mut pending)),
                    None => ui.menu_button(tr(title), |ui| self.menu_entries(ui, &entries, &mut pending)),
                };
            }
        });
        if let Some(a) = pending {
            let ctx = ui.ctx().clone();
            self.do_act(&ctx, a);
        }
    }

    fn control_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_centered(|ui| {
            // Name of the current tool in a box of fixed width, so the bar does not jump around.
            let name = Tool::ALL.iter().find(|t| t.0 == self.tool).map(|t| t.1).unwrap_or("");
            ui.allocate_ui_with_layout(egui::vec2(96.0, 22.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.add(egui::Label::new(RichText::new(tr(name).split(" (").next().unwrap_or("")).strong()).truncate());
            });
            ui.separator();
            let ctx = ui.ctx().clone();
            for (act, icon, tip) in [(Act::New, "file-plus", "New"), (Act::Open, "folder-open", "Open…"), (Act::Save, "save", "Save")] {
                if icon_button(ui, icon, &tr(tip), true, 28.0) {
                    self.do_act(&ctx, act);
                }
            }
            ui.separator();
            let minor = self.grid_prefs.minor_on;
            if toggle_icon(ui, "grid-2x2", &tr("Show or hide the grid"), self.show_grid) {
                self.show_grid = !self.show_grid;
            }
            if toggle_icon(ui, "grid-3x3", &tr("Show or hide the secondary grid"), minor) {
                self.grid_prefs.minor_on = !minor;
            }
            if toggle_icon(ui, "magnet", &tr("Snap to grid"), self.snap) {
                self.snap = !self.snap;
            }
            ui.separator();
            // Green while the toolpaths are shown, grey otherwise.
            let on = self.preview_on;
            let (fill, ink) = if on { (Color32::from_rgb(0x2e, 0xa0, 0x4f), Color32::WHITE) } else { (theme::panel_dark(), theme::text_dim()) };
            let btn = egui::Button::image_and_text(crate::icons::slot(Some("route"), ink), RichText::new(tr("Toolpaths")).color(ink)).fill(fill);
            if ui.add(btn).on_hover_text(tr("Show the toolpaths on the work area")).clicked() {
                self.preview_on = !self.preview_on;
            }
            if window_toggle(ui, &tr("Preview…"), &tr("Open the preview window"), self.show_preview, true) {
                self.show_preview = !self.show_preview;
            }
            let has_camera = !self.doc.device.camera_url.trim().is_empty();
            if has_camera && window_toggle(ui, &tr("Camera view"), &tr("Show the live camera picture of this machine"), self.show_stream, true) {
                self.show_stream = !self.show_stream;
            }
            let (overlay_on, can_overlay) = self.overlay_state();
            if window_toggle(ui, &tr("Overlay"), &tr("Show or hide the camera overlay on the work area"), overlay_on, can_overlay) {
                self.toggle_overlay_button();
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let running = self.job_running();
                if running {
                    let stop = egui::Button::new(RichText::new(format!("   {}   ", tr("STOP").to_uppercase())).color(Color32::WHITE).strong())
                        .fill(Color32::from_rgb(0xc0, 0x30, 0x30));
                    if ui.add(stop).on_hover_text(tr("Abort the running job")).clicked() {
                        self.link.send(Cmd::Abort);
                    }
                } else {
                    let go = egui::Button::new(RichText::new(format!("   {}   ", tr("Start").to_uppercase())).color(Color32::WHITE).strong()).fill(theme::accent());
                    let tip = if self.can_start() { tr("Stream the job to the connected laser") } else { tr("Connect to a laser first (Laser panel)") };
                    if ui.add_enabled(self.can_start(), go).on_hover_text(tip).clicked() {
                        self.send_job();
                    }
                }
                let tip = if self.connected { tr("Trace the bounding box") } else { tr("Connect to a laser first (Laser panel)") };
                if ui.add_enabled(self.connected && !running, egui::Button::new(tr("Frame"))).on_hover_text(tip).clicked() {
                    self.frame();
                }
                if ui.button(tr("Save G-code")).clicked() {
                    self.export_gcode();
                }
                let (w, h) = self.sel_bounds().map(|b| (b.width(), b.height())).unwrap_or((0.0, 0.0));
                if !self.sel.is_empty() {
                    let u = self.doc.device.units;
                    ui.label(RichText::new(format!("{} × {}", fmt_len(u, w, 1), fmt_len(u, h, 1))).color(theme::text_dim()));
                }
            });
        });
    }

    /// Space held down: pan with the hand tool, and go back to the previous tool when it is released.
    pub fn update_space(&mut self, down: bool) {
        if down && self.space_prev.is_none() {
            if self.tool != Tool::Pan {
                self.space_prev = Some(self.tool);
                self.tool = Tool::Pan;
                self.pen_pts.clear();
            }
        } else if !down {
            if let Some(t) = self.space_prev.take() {
                if self.tool == Tool::Pan {
                    self.tool = t;
                }
            }
        }
    }

    /// Zoom by `f` around the middle of the canvas.
    pub fn zoom_about_centre(&mut self, f: f32) {
        let (w, h) = self.canvas_avail;
        let mid = egui::vec2(w / 2.0, h / 2.0);
        let before = (mid - self.view.pan) / self.view.zoom;
        self.view.zoom = (self.view.zoom * f).clamp(0.05, 200.0);
        self.view.pan = mid - before * self.view.zoom;
        self.view.auto_fit = false;
    }

    fn tool_bar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        let out = egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| ui.vertical_centered(|ui| {
            // Selecting and node editing.
            for (t, tip, _) in Tool::ALL.into_iter().filter(|t| t.0.is_edit()) {
                self.tool_button(ui, t, tip);
            }
            // Tools that insert new objects.
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);
            for (t, tip, _) in Tool::ALL.into_iter().filter(|t| !t.0.is_edit() && !t.0.is_nav()) {
                self.tool_button(ui, t, tip);
            }
            // Shape operations on the selected objects.
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);
            let multi = self.sel.len() >= 2;
            let mut op = None;
            for (a, icon, tip) in [
                (Act::BoolUnion, "squares-unite", "Union"),
                (Act::BoolIntersect, "squares-intersect", "Intersection"),
                (Act::BoolSubtract, "squares-subtract", "Subtract"),
                (Act::BoolXor, "squares-exclude", "Exclusive or"),
            ] {
                if icon_button(ui, icon, &tr(tip), multi, 36.0) {
                    op = Some(a);
                }
            }
            if let Some(a) = op {
                let ctx = ui.ctx().clone();
                self.do_act(&ctx, a);
            }
            // View navigation, apart from the rest.
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);
            for (t, tip, _) in Tool::ALL.into_iter().filter(|t| t.0.is_nav()) {
                self.tool_button(ui, t, tip);
            }
            let ink = theme::text();
            let mut act = None;
            for (a, icon, tip) in [(Act::FitBed, "scan", "Fit work area (Ctrl+0)"), (Act::FitAll, "expand", "Fit all objects (Ctrl+9)")] {
                let (rect, resp) = ui.allocate_exact_size(egui::vec2(36.0, 36.0), egui::Sense::click());
                if resp.hovered() {
                    ui.painter().rect_filled(rect, 6.0, theme::panel_dark());
                }
                crate::icons::paint(ui, rect.shrink(7.0), icon, ink);
                if resp.on_hover_text(tr(tip)).clicked() {
                    act = Some(a);
                }
                ui.add_space(2.0);
            }
            if let Some(a) = act {
                let ctx = ui.ctx().clone();
                self.do_act(&ctx, a);
            }
        }));
        scroll_hints(ui, &out);
    }

    /// Narrow strip between the work area and the side panel: align, turn, mirror and group the selection.
    fn arrange_bar(&mut self, ui: &mut egui::Ui) {
        let has_sel = !self.sel.is_empty();
        let any_unlocked = self.sel.iter().any(|id| self.doc.shape(*id).is_some_and(|s| !s.locked));
        let any_locked = self.sel.iter().any(|id| self.doc.shape(*id).is_some_and(|s| s.locked));
        let grouped = self.sel.iter().any(|id| self.doc.shape(*id).is_some_and(|s| s.group.is_some()));
        let multi = self.sel.len() >= 2;
        let _ = has_sel;
        let sections: [Vec<(Act, bool, &str)>; 4] = [
            vec![
                (Act::Align(0), any_unlocked, "Align left"),
                (Act::Align(1), any_unlocked, "Align centre (H)"),
                (Act::Align(2), any_unlocked, "Align right"),
                (Act::Align(3), any_unlocked, "Align top"),
                (Act::Align(4), any_unlocked, "Align centre (V)"),
                (Act::Align(5), any_unlocked, "Align bottom"),
                (Act::CenterOnBed, any_unlocked, "Centre on bed"),
                (Act::CenterEachOther, multi, "Centre on each other"),
            ],
            vec![
                (Act::RotCcw, any_unlocked, "Rotate 90° CCW"),
                (Act::RotCw, any_unlocked, "Rotate 90° CW"),
                (Act::FlipH, any_unlocked, "Flip horizontal"),
                (Act::FlipV, any_unlocked, "Flip vertical"),
            ],
            vec![
                (Act::Group, multi, "Group"),
                (Act::Ungroup, grouped, "Ungroup"),
                (Act::Lock, any_unlocked, "Lock"),
                (Act::Unlock, any_locked, "Unlock"),
            ],
            vec![(Act::ToFront, any_unlocked, "Bring to front"), (Act::ToBack, any_unlocked, "Send to back")],
        ];
        let mut chosen = None;
        let out = egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.add_space(6.0);
            for (i, section) in sections.iter().enumerate() {
                if i > 0 {
                    ui.add_space(2.0);
                    ui.separator();
                    ui.add_space(2.0);
                }
                ui.vertical_centered(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(2.0, 2.0);
                    for (act, on, tip) in section {
                        let icon = crate::icons::act_icon(*act).unwrap_or("shapes");
                        if icon_button(ui, icon, &tr(tip), *on, 28.0) {
                            chosen = Some(*act);
                        }
                    }
                });
            }
        });
        scroll_hints(ui, &out);
        if let Some(a) = chosen {
            let ctx = ui.ctx().clone();
            self.do_act(&ctx, a);
        }
    }

    fn tool_button(&mut self, ui: &mut egui::Ui, t: Tool, tip: &'static str) {
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(36.0, 36.0), egui::Sense::click());
        let on = self.tool == t;
        if on {
            ui.painter().rect_filled(rect, 6.0, theme::accent());
        } else if resp.hovered() {
            ui.painter().rect_filled(rect, 6.0, theme::panel_dark());
        }
        crate::icons::paint(ui, rect.shrink(7.0), crate::icons::tool_icon(t), if on { Color32::WHITE } else { theme::text() });
        if resp.on_hover_text(tr(tip)).clicked() {
            self.tool = t;
            self.space_prev = None;
            self.pen_pts.clear();
            if t == Tool::Polygon {
                self.show_polygon = true;
            }
        }
        ui.add_space(2.0);
    }

    fn swatches(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::horizontal().auto_shrink([false, true]).show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(RichText::new(tr("Layers")).color(theme::text_dim()));
                ui.spacing_mut().item_spacing.x = 3.0;
                let font = egui::FontId::proportional(10.5);
                for i in 0..30 {
                    let c = lc_core::PALETTE[i];
                    let fill = Color32::from_rgb(c[0], c[1], c[2]);
                    // Black or white text, whichever reads better on the layer colour.
                    let lum = 0.299 * c[0] as f32 + 0.587 * c[1] as f32 + 0.114 * c[2] as f32;
                    let ink = if lum > 140.0 { Color32::BLACK } else { Color32::WHITE };
                    let name = self.doc.layers[i].name.clone();
                    let galley = ui.painter().layout_no_wrap(name.clone(), font.clone(), ink);
                    let w = (galley.size().x + 12.0).max(30.0);
                    let (r, resp) = ui.allocate_exact_size(egui::vec2(w, 22.0), egui::Sense::click());
                    ui.painter().rect_filled(r, 3.0, fill);
                    ui.painter().galley(r.center() - galley.size() / 2.0, galley, ink);
                    if self.active_layer == i {
                        ui.painter().rect_stroke(r.expand(1.5), 4.0, egui::Stroke::new(2.0, theme::accent()), egui::StrokeKind::Outside);
                    }
                    let resp = resp.on_hover_text(trf("{} — click to assign selection / set active; double-click for options", &[&name]));
                    if resp.double_clicked() {
                        self.active_layer = i;
                        self.layer_dlg = Some(i);
                    } else if resp.clicked() {
                        self.assign_layer(i);
                    }
                }
            });
        });
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_centered(|ui| {
            ui.label(&self.status);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("{}%", (self.view.zoom / 2.0 * 100.0).round()));
                if let Some(p) = self.cursor_mm {
                    let u = self.doc.device.units;
                    let (x, y) = (u.from_mm(p.x), u.from_mm(p.y));
                    let d = if u == lc_core::Units::Mm { 2 } else { 3 };
                    ui.label(format!("X {:.*}  Y {:.*}{}", d, x, d, y, u.suffix()));
                }
                let color = if self.connected { Color32::from_rgb(0x2e, 0xa0, 0x4f) } else { theme::text_dim() };
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
        let mut open = self.show_polygon;
        let mut done = false;
        egui::Window::new(tr("Polygon")).open(&mut open).collapsible(false).resizable(false).anchor(egui::Align2::LEFT_TOP, [64.0, 120.0]).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(tr("Number of sides"));
                ui.add(egui::DragValue::new(&mut self.polygon_sides).range(3..=360));
            });
            ui.horizontal_wrapped(|ui| {
                for n in [3u32, 4, 5, 6, 8, 12, 24, 360] {
                    if ui.selectable_label(self.polygon_sides == n, n.to_string()).clicked() {
                        self.polygon_sides = n;
                    }
                }
            });
            ui.label(RichText::new(tr("Drag on the work area to draw it. At most 360 sides.")).color(theme::text_dim()));
            done = ui.button(tr("OK")).clicked();
        });
        self.polygon_sides = self.polygon_sides.clamp(3, 360);
        self.show_polygon = open && !done;

        let mut open = self.show_offset;
        let mut apply = false;
        egui::Window::new(tr("Offset shape")).open(&mut open).collapsible(false).resizable(false).show(ctx, |ui| {
            egui::Grid::new("offs").num_columns(2).show(ui, |ui| {
                ui.label(tr("Distance"));
                drag_len(ui, self.doc.device.units, &mut self.offset_dist, 0.1, Some((-500.0, 500.0)));
                ui.end_row();
            });
            ui.label(RichText::new(tr("Positive grows the outline, negative shrinks it.")).color(theme::text_dim()));
            ui.checkbox(&mut self.offset_keep, tr("Keep the original"));
            apply = ui.button(tr("Create offset")).clicked();
        });
        if apply {
            self.offset_selected(self.offset_dist, self.offset_keep);
            open = false;
        }
        self.show_offset = open;

        let mut open = self.show_array;
        let mut apply = false;
        egui::Window::new(tr("Grid array")).open(&mut open).collapsible(false).resizable(false).show(ctx, |ui| {
            egui::Grid::new("arr").num_columns(2).show(ui, |ui| {
                ui.label(tr("Columns"));
                ui.add(egui::DragValue::new(&mut self.array.0).range(1..=100));
                ui.end_row();
                ui.label(tr("Rows"));
                ui.add(egui::DragValue::new(&mut self.array.1).range(1..=100));
                ui.end_row();
                ui.label(tr("Gap X"));
                drag_len(ui, self.doc.device.units, &mut self.array.2, 0.5, None);
                ui.end_row();
                ui.label(tr("Gap Y"));
                drag_len(ui, self.doc.device.units, &mut self.array.3, 0.5, None);
                ui.end_row();
            });
            apply = ui.button(tr("Create array")).clicked();
        });
        if apply {
            self.make_array();
            open = false;
        }
        self.show_array = open;

        let mut open = self.show_about;
        egui::Window::new(tr("About LightCreator")).open(&mut open).collapsible(false).resizable(false).show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(logo) = &self.logo {
                    ui.image((logo.id(), egui::vec2(72.0, 72.0)));
                }
                ui.vertical(|ui| {
                    ui.heading("LightCreator");
                    ui.label(trf("Version {}", &[&APP_VERSION]));
                });
            });
            ui.label(tr("Open-source design & control software for laser engravers and cutters."));
            ui.separator();
            ui.label(tr("Origin: written in Rust with egui/eframe."));
            ui.label(tr("Inspired by LightBurn; interface inspired by VectorCraft."));
            ui.label(tr("Author: Michał Sarna"));
            ui.hyperlink(crate::help_ui::GITHUB_URL);
            ui.label(tr("MIT licensed."));
        });
        self.show_about = open;
        self.config_window(ctx);
        self.layer_dialog(ctx);
        self.guide_window(ctx);
        self.clip_dialogs(ctx);
        self.online_clipart_window(ctx);
        self.round_window(ctx);
        self.materials_window(ctx);
        self.overlay_window(ctx);
        self.prefs_window(ctx);
        self.preview_window(ctx);
        self.image_dialogs(ctx);
        self.stream_window(ctx);
        self.ser2net_window(ctx);
    }
}

pub fn fmt_time(s: f64) -> String {
    let s = s as u64;
    format!("{}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string("lang", self.lang.code().to_owned());
        storage.set_string("scheme", self.scheme.id().to_owned());
        if let Ok(j) = serde_json::to_string(&self.profiles) {
            storage.set_string("profiles", j);
        }
        storage.set_string("active_profile", self.active.to_string());
        if let Ok(j) = serde_json::to_string(&self.recent) {
            storage.set_string("recent_files", j);
        }
        if let Ok(j) = serde_json::to_string(&self.grid_prefs) {
            storage.set_string("grid_prefs", j);
        }
        storage.set_string("grid_main_on", if self.show_grid { "1" } else { "0" }.into());
        storage.set_string("grid_main_mm", self.grid.to_string());
        if let Ok(j) = serde_json::to_string(&self.user_presets) {
            storage.set_string("user_materials", j);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.draw(ui);
    }
}

/// Double chevrons at the top and / or bottom edge of a scrolled icon strip when more icons lie beyond it.
fn scroll_hints<R>(ui: &egui::Ui, out: &egui::scroll_area::ScrollAreaOutput<R>) {
    let view = out.inner_rect;
    let (up, down) = (out.state.offset.y > 1.0, out.state.offset.y + view.height() < out.content_size.y - 1.0);
    for (show, name, top) in [(up, "chevrons-up", true), (down, "chevrons-down", false)] {
        if !show {
            continue;
        }
        let h = 16.0;
        let rect = if top {
            egui::Rect::from_min_size(view.left_top(), egui::vec2(view.width(), h))
        } else {
            egui::Rect::from_min_size(egui::pos2(view.left(), view.bottom() - h), egui::vec2(view.width(), h))
        };
        let painter = ui.painter().with_clip_rect(view);
        painter.rect_filled(rect, 0.0, theme::panel().gamma_multiply(0.92));
        crate::icons::paint(ui, egui::Rect::from_center_size(rect.center(), egui::vec2(h - 2.0, h - 2.0)), name, theme::accent());
    }
}

/// Text button for a window that can be open or closed: light blue while it is open.
pub fn window_toggle(ui: &mut egui::Ui, text: &str, tip: &str, open: bool, enabled: bool) -> bool {
    let btn = if open {
        egui::Button::new(RichText::new(text).color(Color32::WHITE)).fill(Color32::from_rgb(0x3f, 0xa9, 0xf5))
    } else {
        egui::Button::new(text)
    };
    ui.add_enabled(enabled, btn).on_hover_text(tip).clicked()
}

/// An icon button that stays highlighted while `on`.
pub fn toggle_icon(ui: &mut egui::Ui, icon: &str, tip: &str, on: bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::click());
    if on {
        ui.painter().rect_filled(rect, 6.0, theme::accent().gamma_multiply(0.3));
    } else if resp.hovered() {
        ui.painter().rect_filled(rect, 6.0, theme::panel_dark());
    }
    crate::icons::paint(ui, rect.shrink(5.6), icon, if on { theme::accent() } else { theme::text() });
    resp.on_hover_text(tip).clicked()
}

/// A square icon-only button with a tooltip; greyed out and inert when `on` is false.
pub fn icon_button(ui: &mut egui::Ui, icon: &str, tip: &str, on: bool, size: f32) -> bool {
    let sense = if on { egui::Sense::click() } else { egui::Sense::hover() };
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(size, size), sense);
    if on && resp.hovered() {
        ui.painter().rect_filled(rect, 6.0, theme::panel_dark());
    }
    let ink = if on { theme::text() } else { theme::text().gamma_multiply(0.35) };
    crate::icons::paint(ui, rect.shrink(size * 0.2), icon, ink);
    let resp = resp.on_hover_text(tip);
    ui.add_space(2.0);
    on && resp.clicked()
}

impl App {
    /// Draw one frame of the whole application into `ui`.
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.poll_laser();
        if self.preview_on && self.preview.as_ref().map(|p| p.0) != Some(self.revision) {
            self.preview = Some((self.revision, gcode::generate(&self.doc)));
        }
        #[cfg(target_os = "macos")]
        self.native_menu_frame(&ctx);
        self.title_bar(ui);
        self.window_edges(&ctx);
        #[cfg(target_os = "macos")]
        self.window_edge_drag(&ctx);
        if self.screen == Screen::Start {
            self.start_screen(ui);
            self.dialogs(&ctx);
            return;
        }
        self.handle_drops(&ctx);
        // Space held (and not typing): the hand tool, until it is released.
        let typing = ctx.egui_wants_keyboard_input();
        let space = self.screen == Screen::Editor && !typing && ctx.input(|i| i.key_down(Key::Space) && i.focused);
        if space {
            // Keep a focused button from reacting to the same key.
            ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Space));
        }
        self.update_space(space);
        self.read_tick(&ctx);
        self.overlay_poll_camera(&ctx);
        self.shortcuts(&ctx);

        let chrome = egui::Frame::new().fill(theme::panel()).stroke(egui::Stroke::new(1.0, theme::border())).inner_margin(egui::Margin::symmetric(8, 4));
        #[cfg(target_os = "macos")]
        let in_window_menu = self.native_menu.is_none();
        #[cfg(not(target_os = "macos"))]
        let in_window_menu = true;
        if in_window_menu {
            egui::Panel::top("menu").frame(chrome).show(ui, |ui| self.menu_bar(ui));
        }
        egui::Panel::top("controls").frame(chrome).exact_size(36.0).show(ui, |ui| self.control_bar(ui));
        egui::Panel::bottom("status").frame(chrome).exact_size(26.0).show(ui, |ui| self.status_bar(ui));
        egui::Panel::bottom("swatches").frame(chrome).exact_size(36.0).show(ui, |ui| self.swatches(ui));
        egui::Panel::left("tools").frame(chrome).exact_size(52.0).resizable(false).show(ui, |ui| self.tool_bar(ui));
        egui::Panel::right("side").frame(chrome.inner_margin(egui::Margin::same(8))).default_size(330.0).show(ui, |ui| self.side_panel(ui));
        egui::Panel::right("arrange_bar").frame(chrome.inner_margin(egui::Margin::symmetric(2, 4))).exact_size(40.0).resizable(false).show(ui, |ui| self.arrange_bar(ui));
        egui::CentralPanel::no_frame().show(ui, |ui| self.canvas(ui));
        self.dialogs(&ctx);
        if self.connected || self.progress.0 < self.progress.1 {
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }
        }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_quick_escapes_select_the_select_tool() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.tool = Tool::Rect;
        a.note_escape(10.0);
        assert_eq!(a.tool, Tool::Rect, "one Escape alone keeps the tool");
        a.note_escape(10.3);
        assert_eq!(a.tool, Tool::Select, "the second one, quickly after, selects");
        // Slow presses never count as a pair.
        a.tool = Tool::Star;
        a.note_escape(20.0);
        a.note_escape(21.0);
        assert_eq!(a.tool, Tool::Star);
        a.note_escape(21.2);
        assert_eq!(a.tool, Tool::Select);
    }

    #[test]
    fn tool_shortcuts_are_unique() {
        let mut keys: Vec<_> = Tool::ALL.iter().map(|t| t.2).collect();
        keys.sort_by_key(|k| format!("{k:?}"));
        keys.dedup();
        assert_eq!(keys.len(), Tool::ALL.len());
    }

    #[test]
    fn view_fits_the_window_and_manual_zoom_turns_auto_fit_off() {
        let (z, pan) = crate::canvas::fit_view((900.0, 700.0), (400.0, 400.0), 50.0);
        assert!((z - 1.5).abs() < 1e-6);
        assert!((pan.x - 150.0).abs() < 1e-4 && (pan.y - 50.0).abs() < 1e-4);
        // A tiny window never gives a zero or negative zoom.
        assert!(crate::canvas::fit_view((60.0, 60.0), (400.0, 400.0), 50.0).0 >= 0.05);
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.screen = Screen::Editor;
        assert!(a.view.auto_fit);
        a.do_act(&ctx, crate::menu::Act::ToggleAutoFit);
        assert!(!a.view.auto_fit);
        a.do_act(&ctx, crate::menu::Act::FitBed);
        assert!(a.view.auto_fit && a.view.need_fit);
    }

    #[test]
    fn navigation_tools_are_a_separate_group() {
        let nav: Vec<Tool> = Tool::ALL.iter().map(|t| t.0).filter(|t| t.is_nav()).collect();
        assert_eq!(nav, vec![Tool::Pan, Tool::Zoom, Tool::ZoomOut]);
        assert!(Tool::ALL.iter().filter(|t| !t.0.is_nav()).all(|t| !matches!(t.0, Tool::Pan | Tool::Zoom | Tool::ZoomOut)));
    }

    #[test]
    fn holding_space_pans_and_releasing_restores_the_tool() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.tool = Tool::Rect;
        a.update_space(true);
        assert_eq!(a.tool, Tool::Pan);
        a.update_space(true); // key repeat changes nothing
        assert_eq!(a.tool, Tool::Pan);
        a.update_space(false);
        assert_eq!(a.tool, Tool::Rect);
        // Already on the hand tool: nothing to restore, the tool stays.
        a.tool = Tool::Pan;
        a.update_space(true);
        a.update_space(false);
        assert_eq!(a.tool, Tool::Pan);
        // Choosing another tool while Space is held wins over the restore.
        a.tool = Tool::Star;
        a.update_space(true);
        a.tool = Tool::Ellipse;
        a.update_space(false);
        assert_eq!(a.tool, Tool::Ellipse);
    }

    #[test]
    fn zoom_buttons_keep_the_middle_of_the_canvas_fixed() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.canvas_avail = (800.0, 600.0);
        a.view.zoom = 2.0;
        a.view.pan = egui::vec2(100.0, 50.0);
        let mid = egui::vec2(400.0, 300.0);
        let world = (mid - a.view.pan) / a.view.zoom;
        a.zoom_about_centre(1.5);
        assert!((a.view.zoom - 3.0).abs() < 1e-6);
        let again = (mid - a.view.pan) / a.view.zoom;
        assert!((again - world).length() < 1e-4);
        assert!(!a.view.auto_fit);
        a.zoom_about_centre(1.0 / 1.5);
        assert!((a.view.zoom - 2.0).abs() < 1e-6);
        let (z, pan) = crate::canvas::fit_region((900.0, 700.0), (100.0, 50.0), (200.0, 100.0), 50.0);
        assert!((z - 4.0).abs() < 1e-6, "{z}");
        // The region's centre lands in the middle of the area.
        assert!(((200.0f32 * z + pan.x) - 450.0).abs() < 1e-3 && ((100.0f32 * z + pan.y) - 350.0).abs() < 1e-3);
    }
}

#[cfg(test)]
mod recent_tests {
    use super::*;

    #[test]
    fn recent_files_are_unique_newest_first_and_capped() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        let dir = std::env::temp_dir().join(format!("lc-recent-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let files: Vec<PathBuf> = (0..12)
            .map(|i| {
                let f = dir.join(format!("f{i}.lcr"));
                std::fs::write(&f, Document::default().to_json()).unwrap();
                f
            })
            .collect();
        for f in &files {
            a.open_path(f.clone());
        }
        assert_eq!(a.recent.len(), crate::menu::MAX_RECENT);
        assert_eq!(a.recent[0], std::fs::canonicalize(&files[11]).unwrap());
        // Opening an older one again moves it to the top without a duplicate.
        a.open_path(files[5].clone());
        assert_eq!(a.recent[0], std::fs::canonicalize(&files[5]).unwrap());
        assert_eq!(a.recent.len(), crate::menu::MAX_RECENT);
        assert_eq!(a.recent.iter().filter(|r| **r == a.recent[0]).count(), 1);
        // The menu action opens entry n; a file that vanished is dropped from the list.
        std::fs::remove_file(&files[11]).unwrap();
        let gone = std::fs::canonicalize(&files[10]).unwrap();
        let idx = a.recent.iter().position(|r| *r == gone).unwrap() as u8;
        a.screen = Screen::Editor;
        a.do_act(&ctx, Act::OpenRecent(idx));
        assert_eq!(a.recent[0], gone);
        a.do_act(&ctx, Act::ClearRecent);
        assert!(a.recent.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod version_tests {
    /// The version shown in About, the Cargo version, the newest VERSIONS.md section and the supported
    /// version in SECURITY.md must all name the same release.
    #[test]
    fn version_is_the_same_everywhere() {
        let cargo = env!("CARGO_PKG_VERSION");
        assert_eq!(super::APP_VERSION, format!("v{cargo}"), "APP_VERSION in app.rs vs Cargo.toml");
        let versions = include_str!("../../../VERSIONS.md");
        let top = versions.lines().find_map(|l| l.strip_prefix("## ")).expect("a section in VERSIONS.md");
        assert_eq!(top.trim(), super::APP_VERSION, "newest section of VERSIONS.md");
        let security = include_str!("../../../SECURITY.md");
        let latest = security.lines().find(|l| l.contains("(latest)")).expect("a (latest) row in SECURITY.md");
        assert!(latest.contains(super::APP_VERSION), "SECURITY.md latest row: {latest}");
    }
}

#[cfg(test)]
mod centre_tests {
    use super::*;

    #[test]
    fn objects_are_centred_on_each_other_and_groups_stay_together() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.screen = Screen::Editor;
        let big = a.doc.add(0, lc_core::Kind::Rect { w: 100.0, h: 60.0 }, Xf::translate(10.0, 10.0));
        let small = a.doc.add(0, lc_core::Kind::Rect { w: 10.0, h: 10.0 }, Xf::translate(20.0, 15.0));
        a.sel = vec![big, small];
        a.do_act(&ctx, Act::CenterEachOther);
        let (cb, cs) = (a.doc.shape(big).unwrap().bounds().unwrap().center(), a.doc.shape(small).unwrap().bounds().unwrap().center());
        assert!((cb.x - 60.0).abs() < 1e-6 && (cb.y - 40.0).abs() < 1e-6, "the large one stays: {cb:?}");
        assert!((cs.x - cb.x).abs() < 1e-6 && (cs.y - cb.y).abs() < 1e-6, "{cs:?} vs {cb:?}");
        // A group of two shapes moves as one object and keeps its inner layout.
        let g1 = a.doc.add(0, lc_core::Kind::Rect { w: 4.0, h: 4.0 }, Xf::translate(200.0, 200.0));
        let g2 = a.doc.add(0, lc_core::Kind::Rect { w: 4.0, h: 4.0 }, Xf::translate(210.0, 200.0));
        let gid = a.doc.new_group_id();
        for id in [g1, g2] {
            a.doc.shape_mut(id).unwrap().group = Some(gid);
        }
        a.sel = vec![big, g1, g2];
        let gap = a.doc.shape(g2).unwrap().bounds().unwrap().min.x - a.doc.shape(g1).unwrap().bounds().unwrap().min.x;
        a.do_act(&ctx, Act::CenterEachOther);
        let gb = a.doc.bounds_of(&[g1, g2]).unwrap().center();
        let bb = a.doc.shape(big).unwrap().bounds().unwrap().center();
        assert!((gb.x - bb.x).abs() < 1e-6 && (gb.y - bb.y).abs() < 1e-6);
        let gap2 = a.doc.shape(g2).unwrap().bounds().unwrap().min.x - a.doc.shape(g1).unwrap().bounds().unwrap().min.x;
        assert!((gap - gap2).abs() < 1e-9);
        // One object alone is not enough.
        a.sel = vec![big];
        assert!(!a.act_enabled(Act::CenterEachOther));
    }
}
