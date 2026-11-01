use base_ui::clipart_ui::ClipGallery;
use base_ui::editor::EditorImages;
use base_ui::editor::EditorCommands;
#[cfg(test)]
use base_ui::editor::EditorCanvas;
use base_ui::editor::EditorHost;
use crate::laser::{self, Cmd, ConsoleLine, Evt, LaserLink};
use crate::units_ui::drag_len;
use crate::i18n::{self, tr, trf, Lang};
use crate::menu::{menus, Act};
use crate::theme::{self, Scheme};
use eframe::egui::{self, Color32, Key, Modifiers, RichText};
use lc_core::{gcode, svg, Document, Pt, Shape, Xf};
use std::path::PathBuf;

pub use base_ui::editor::{Tool, View};

/// Human-facing release label (branch name matches it).
pub const APP_VERSION: &str = "v0.5.2";

/// Tabs of the right-hand panel.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum SideTab {
    Properties,
    Layers,
    Device,
    /// Moving the machine: the position, the jog buttons, the focus beam, the macros (the same tab as in MillCreator and PenCreator).
    Control,
    Console,
    Clipart,
    /// A free text about the project.
    Description,
    /// The camera picture; only listed while the stream is shown as a tab.
    Camera,
    /// The tab of a plugin with settings (which one: `App::plugin_tab`).
    Plugin,
}

impl SideTab {
    pub const ALL: [(SideTab, &'static str); 7] = [
        (SideTab::Properties, "Properties"),
        (SideTab::Layers, "Layers"),
        (SideTab::Device, "Device"),
        (SideTab::Control, "Control"),
        (SideTab::Console, "Console"),
        (SideTab::Clipart, "Clipart"),
        (SideTab::Description, "Description"),
    ];

    pub fn id(self) -> &'static str {
        match self {
            SideTab::Properties => "properties",
            SideTab::Layers => "layers",
            SideTab::Device => "device",
            SideTab::Control => "control",
            SideTab::Console => "console",
            SideTab::Clipart => "clipart",
            SideTab::Description => "description",
            SideTab::Camera => "camera",
            SideTab::Plugin => "plugin",
        }
    }

    /// The ids of the tabs that are always there, for the order the user arranges them in.
    pub const IDS: [&'static str; 7] = ["properties", "layers", "device", "control", "console", "clipart", "description"];

    pub fn from_id(id: &str) -> Option<SideTab> {
        SideTab::ALL.iter().map(|(t, _)| *t).chain([SideTab::Camera]).find(|t| t.id() == id)
    }

    /// The English name of the tab (translated where it is shown).
    pub fn label(self) -> &'static str {
        SideTab::ALL.iter().find(|(t, _)| *t == self).map(|(_, l)| *l).unwrap_or("Camera view")
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Start,
    Editor,
}

pub struct App {
    pub doc: Document,
    /// The state of the editor itself (selection, tool, view, grid, snapping), shared with other applications.
    pub ed: base_ui::editor::EditorState,
    pub undo: Vec<Document>,
    pub redo: Vec<Document>,
    pub path: Option<PathBuf>,
    /// Most recently opened or saved project files, newest first (at most `MAX_RECENT`).
    pub recent: Vec<PathBuf>,
    pub status: String,
    pub revision: u64,
    /// Revision at the last save by hand (or open / new), and at the last auto-save.
    pub saved_revision: u64,
    /// What the Control tab remembers between frames, the arrow keys that jog, and where the last report said the head is.
    pub ctl: base_ui::control_ui::ControlUi,
    pub keyboard_jog: bool,
    pub pos: crate::control_glue::Where,
    /// The jog step the Control tab and the device agreed on at the last frame.
    pub jog_seen_step: Option<f64>,
    /// The last tab was closed: the program closes too.
    pub close_program: bool,
    /// The tabs of open drawings (the shown one is the live fields above), and the number of the recovery file of the shown one.
    pub tabs_state: crate::workspaces::Tabs,
    /// The central area in panes (one pane unless the person split the view).
    pub panes: base_ui::work_panes::Panes,
    pub slot: usize,
    pub autosaved_revision: u64,
    pub autosave_on: bool,
    /// The shared data folder (the recovery file of this program is in it).
    pub data_base: std::path::PathBuf,
    /// An unsaved project left by an earlier run (offered on the start screen).
    pub recovery: Vec<lc_core::autosave::Found>,
    /// The web server and API (off by default): switched on, port, and reachable from the network.
    /// The material header of the Layers tab: adding or renaming.
    pub mat_edit: Option<crate::materials_ui::MatEdit>,
    /// The window that manages materials and their layers.
    pub materials_dlg: Option<crate::materials_win::MaterialsDlg>,
    /// The revision the selected material was last compared with the layers at.
    pub mat_rev: u64,
    pub api_on: bool,
    pub api_port: u16,
    pub api_lan: bool,
    /// The access phrase of the web server (what its pages ask for), and what is typed in its field.
    pub api_access: String,
    pub api_access_edit: String,
    pub api_access_typing: bool,
    pub api: crate::api_ui::ApiState,
    pub autosave_secs: u32,
    pub last_autosave: std::time::Instant,
    /// A short message (text, until when) such as "file saved".
    pub toast: Option<(String, std::time::Instant)>,
    pub show_close_dlg: bool,
    /// What the person has to look at now: an alarm, or an error in the middle of a job (a big window with STOP and UNLOCK).
    pub problem: Option<base_ui::machine_ctl::Problem>,
    /// A question waiting for Yes / No, drawn by the application so it is in the chosen language.
    pub confirm: Option<crate::session_ui::Confirm>,
    /// The issues found before a job was sent, framed or exported.
    pub check_dlg: Option<crate::check_ui::CheckDlg>,
    /// What the controller reported after connecting that does not look right.
    pub fw_warn: Option<lc_core::controller::FwCheck>,
    /// A running job waits for an answer while the machine stands still: seconds and machine state.
    pub stall: Option<(u64, String)>,
    /// A PDF / Illustrator file with layers waits for the choice of how to put them on the layers of the program: the path and the outlines.
    pub ai_choice: Option<(std::path::PathBuf, lc_core::pdf::Outlines)>,
    /// A G-code file shown over the work area.
    pub gcode_view: Option<base_ui::gcode_view_ui::GcodeViewer>,
    /// The box the frame command traces.
    pub frame_bounds: Option<lc_core::Rect>,
    /// The corners of the outline the frame follows, when the device frames along the outline of the shapes.
    pub frame_path: Option<Vec<Pt>>,
    pub allow_close: bool,
    pub show_options: bool,
    /// Look for a newer release on GitHub when the program starts.
    pub check_updates: bool,
    pub update: crate::update_ui::UpdateState,
    /// Draw the no-go zones on the work area (kept between sessions).
    pub zones_visible: bool,
    pub preview_on: bool,
    pub preview: Option<(u64, gcode::Job)>,
    pub show_all_layers: bool,
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
    pub text_default: lc_core::TextData,
    pub font_filter: String,
    pub focus_text: bool,
    pub show_prefs: bool,
    pub clip: crate::clipart_ui::ClipState,
    pub show_guide: bool,
    pub guide_filter: String,
    pub esc_time: f64,
    pub layer_dlg: Option<usize>,
    pub show_stream: bool,
    pub ser2net: Option<crate::ser2net_ui::Ser2NetDlg>,
    pub print: Option<crate::print_ui::PrintDlg>,
    pub cam_test: Option<crate::cam_test_ui::CamTest>,
    pub stream_mode: crate::stream_ui::StreamMode,
    pub tab_seen: bool,
    /// Rotation the current stream texture was made with.
    pub stream_rot: u8,
    /// The size of the separate camera window and when it last changed (the picture is frozen while it does).
    pub cam_win: Option<([f32; 2], std::time::Instant)>,
    /// Magnification of the camera picture in its window (1 = fitted) and its shift in pixels.
    pub stream_zoom: f32,
    pub stream_pan: egui::Vec2,
    pub stream: Option<(String, crate::camera_stream::StreamWorker)>,
    pub stream_tex: Option<egui::TextureHandle>,
    /// The camera window shrinks to the picture.
    pub cam_fit: base_ui::window::FitWindow,
    /// The camera whose way of opening the camera view has now.
    pub cam_view_for: usize,
    /// The camera view is a tab of the work area and it is the one shown (in place of the drawing).
    pub cam_tab_shown: bool,
    /// The preview is a tab of the work area (and not a window) while it is open, and that tab is the shown one.
    pub preview_tab: bool,
    pub pv_tab_shown: bool,
    pub pv_tab_seen: bool,
    pub cam_tab_seen: bool,
    pub stream_err: String,
    /// Photos, video and time-lapse of the camera view (the folder and the interval are remembered).
    pub cam_rec: base_ui::camera_rec::Recorder,
    /// Where the layers of the job being sent end, for a picture after each layer, and the numbers of the program that is being made.
    pub layer_watch: base_ui::camera_rec::LayerWatch,
    pub job_marks: Vec<usize>,
    pub stream_overlay: bool,
    pub read_cfg: Option<crate::device_ui::ReadCfg>,
    pub show_preview: bool,
    pub pv: crate::preview_ui::PreviewState,
    pub overlay: Option<crate::overlay::Overlay>,
    pub overlay_visible: bool,
    pub overlay_edit: bool,
    pub overlay_drag: Option<usize>,
    pub show_overlay: bool,
    pub cam_list: Vec<String>,
    pub cam_index: u32,
    pub cam_error: String,
    pub cam_worker: Option<crate::camera::Worker>,
    /// The order of the tabs of the side panel, and whether it is collapsed.
    pub tabs: base_ui::side_tabs::SideTabs,
    /// Width of the side panel as drawn in the last frame.
    pub side_w: f32,
    pub lang: Lang,
    pub scheme: Scheme,
    pub logo: Option<egui::TextureHandle>,
    #[cfg(target_os = "macos")]
    pub native_menu: Option<crate::native_menu::NativeMenu>,
    // laser
    pub link: LaserLink,
    pub ports: Vec<String>,
    /// The serial ports with their USB ids, refreshed together with `ports`.
    pub port_info: Vec<crate::laser::PortInfo>,
    /// The user asked for a connection (and has not asked to end it): a lost link may be re-opened by itself.
    pub want_link: bool,
    pub reconnect_at: Option<std::time::Instant>,
    /// When the actions the device runs after connecting are due (clear the alarm, home).
    pub connect_actions_at: Option<std::time::Instant>,
    /// The firmware reported that it was built with single-axis homing (`$HX`, `$HY`).
    pub single_axis_homing: bool,
    /// The Firmware ($$) page.
    pub fw: crate::firmware_ui::FwUi,
    /// "Start from line…": the window, the program it works on, and the window of the calibration.
    pub start_from: base_ui::job_ui::StartFrom,
    pub start_lines: Vec<String>,
    pub cal: base_ui::tools_ui::CalUi,
    /// The window with the templates for a new device is open.
    /// The first-run setup is open at this step.
    pub add_dev: Option<usize>,
    /// The add-device window read the controller.
    pub add_dev_read: bool,
    pub add_dev_read_done: bool,
    /// The last read in the add-device window failed (no connection, no answer): the button shows it.
    pub add_dev_read_err: bool,
    /// A read was running in the previous frame, to see how it ended.
    pub add_dev_was_reading: bool,
    /// The user left the first-run setup: do not start it again this session.
    pub add_dev_skipped: bool,
    /// The focus beam is on.
    pub focus_on: bool,
    /// The warning about macros has been answered in this session.
    pub macros_ok: bool,
    /// When the running job started (for the work time of the machine).
    pub job_started: Option<std::time::Instant>,
    /// The estimated time of the job that was sent last, and how long the last finished one really ran (seconds).
    pub job_est: f64,
    pub job_last_secs: f64,
    /// The counters (by index) that have been announced as due in this session.
    pub maintenance_notified: std::collections::HashSet<usize>,
    /// The laser position that is shown on the work area (live position) and when it was taken.
    pub live_shown: Option<(std::time::Instant, (f64, f64))>,
    /// For this session only: aim with the pointer dot (the frame and the dry run move back by its offset).
    pub pointer_align: bool,
    /// Options of a job, in the Control tab (this session only): burn the elements one after the other, and burn only what is
    /// selected when the G-code is saved or the job started.
    pub one_by_one: bool,
    pub selection_only: bool,
    pub connected: bool,
    pub machine: (String, f64, f64),
    /// The Z position the controller last reported (NaN when it does not report one).
    pub machine_z: f64,
    pub progress: (usize, usize),
    pub console: base_ui::console_ui::ConsoleState,
    pub side_tab: SideTab,
    pub plugins: base_ui::plugins_ui::PluginsUi,
    /// The id of the plugin tab that is shown while `side_tab` is `SideTab::Plugin`.
    pub plugin_tab: &'static str,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::build(&cc.egui_ctx, cc.storage, true);
        app.after_plugins_changed();
        if app.check_updates {
            app.start_update_check(&cc.egui_ctx, false);
        }
        app
    }

    /// Build the app without a window: used by `new` and by the screenshot tests.
    pub fn build(ctx: &egui::Context, storage: Option<&dyn eframe::Storage>, native_menu: bool) -> Self {
        // Only the macOS menu bar needs it.
        #[cfg(not(target_os = "macos"))]
        let _ = native_menu;
        let port_info = laser::list_port_info();
        let ports: Vec<String> = port_info.iter().map(|p| p.name.clone()).collect();
        let saved = |k: &str| storage.and_then(|st| st.get_string(k));
        let lang = saved("lang").and_then(|c| Lang::from_code(&c)).unwrap_or(Lang::En);
        let scheme = Scheme::from_saved(saved("scheme").as_deref());
        let profiles: Vec<lc_core::Device> = saved("profiles").and_then(|j| serde_json::from_str(&j).ok()).unwrap_or_default();
        let active = saved("active_profile").and_then(|v| v.parse::<usize>().ok()).filter(|i| *i < profiles.len()).unwrap_or(0);
        let grid_prefs: crate::prefs::GridPrefs = saved("grid_prefs").and_then(|j| serde_json::from_str(&j).ok()).unwrap_or_default();
        let show_grid = saved("grid_main_on").map(|v| v == "1").unwrap_or(true);
        let grid = saved("grid_main_mm").and_then(|v| v.parse::<f64>().ok()).filter(|g| *g >= 0.1).unwrap_or(10.0);
        i18n::set_lang(lang);
        theme::apply(ctx, scheme);
        base_ui::icons::install(ctx);
        crate::fonts::install(ctx);
        lc_core::text::preload();
        App {
            doc: Document::default(),
            ed: base_ui::editor::EditorState {
                snap_objects: saved("snap_objects").map(|v| v == "1").unwrap_or(false),
                grid,
                show_grid,
                grid_prefs,
                ..base_ui::editor::EditorState::new()
            },
            undo: vec![],
            redo: vec![],
            path: None,
            recent: saved("recent_files").and_then(|j| serde_json::from_str::<Vec<PathBuf>>(&j).ok()).unwrap_or_default().into_iter().take(crate::menu::MAX_RECENT).collect(),
            status: tr("Ready").into(),
            revision: 0,
            saved_revision: 0,
            ctl: Default::default(),
            keyboard_jog: false,
            pos: Default::default(),
            jog_seen_step: None,
            close_program: false,
            tabs_state: Default::default(),
            panes: base_ui::work_panes::Panes::from_json(&saved("panes").unwrap_or_default()),
            slot: 0,
            autosaved_revision: 0,
            data_base: crate::data_paths().base.clone(),
            recovery: lc_core::autosave::find_all(&crate::data_paths().base, "lightcreator"),
            autosave_on: saved("autosave_on").map(|v| v != "0").unwrap_or(true),
            mat_edit: None,
            materials_dlg: None,
            mat_rev: u64::MAX,
            api_on: saved("api_on").map(|v| v == "1").unwrap_or(false),
            api_port: saved("api_port").and_then(|v| v.parse::<u16>().ok()).filter(|p| *p >= 1024).unwrap_or(crate::api_ui::DEFAULT_PORT),
            api_lan: saved("api_lan").map(|v| v == "1").unwrap_or(false),
            api_access: saved("api_access").and_then(|v| crate::api_server::access_from_setting(&v)).unwrap_or_else(|| crate::api_server::DEFAULT_ACCESS.to_string()),
            api_access_edit: String::new(),
            api_access_typing: false,
            api: Default::default(),
            autosave_secs: saved("autosave_secs").and_then(|v| v.parse().ok()).filter(|v| (10..=3600).contains(v)).unwrap_or(crate::session_ui::DEFAULT_AUTOSAVE_SECS),
            last_autosave: std::time::Instant::now(),
            toast: None,
            show_close_dlg: false,
            confirm: None,
            problem: None,
            check_dlg: None,
            fw_warn: None,
            stall: None,
            ai_choice: None,
            gcode_view: None,
            frame_bounds: None,
            frame_path: None,
            allow_close: false,
            show_options: false,
            check_updates: saved("check_updates").map(|v| v != "0").unwrap_or(true),
            update: Default::default(),
            zones_visible: saved("zones_visible").map(|v| v != "0").unwrap_or(true),
            preview_on: false,
            preview: None,
            show_all_layers: false,
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
            text_default: lc_core::TextData::default(),
            font_filter: String::new(),
            focus_text: false,
            show_prefs: false,
            clip: crate::clipart_ui::new_state(lc_core::clipart::Library::open(crate::data_paths().gallery.clone())),
            show_guide: false,
            guide_filter: String::new(),
            esc_time: -10.0,
            layer_dlg: None,
            show_stream: false,
            ser2net: None,
            print: None,
            cam_test: None,
            stream_mode: crate::stream_ui::StreamMode::Tab,
            tab_seen: false,
            stream_rot: 0,
            cam_win: None,
            stream_zoom: 1.0,
            stream_pan: egui::Vec2::ZERO,
            stream: None,
            stream_tex: None,
            cam_fit: Default::default(),
            cam_view_for: 0,
            cam_tab_shown: false,
            preview_tab: false,
            pv_tab_shown: false,
            pv_tab_seen: false,
            cam_tab_seen: false,
            stream_err: String::new(),
            cam_rec: base_ui::camera_rec::Recorder::default(),
            layer_watch: base_ui::camera_rec::LayerWatch::default(),
            job_marks: vec![],
            stream_overlay: false,
            read_cfg: None,
            show_preview: false,
            pv: Default::default(),
            overlay: None,
            overlay_visible: true,
            overlay_edit: false,
            overlay_drag: None,
            show_overlay: false,
            cam_list: vec![],
            cam_index: 0,
            cam_error: String::new(),
            cam_worker: None,
            side_w: 0.0,
            tabs: base_ui::side_tabs::SideTabs::new(&SideTab::IDS, &saved("tab_order").unwrap_or_default(), saved("side_collapsed").is_some_and(|v| v == "1")),
            lang,
            scheme,
            logo: crate::load_logo(ctx),
            #[cfg(target_os = "macos")]
            native_menu: native_menu.then(|| crate::native_menu::install(ctx)),
            link: LaserLink::spawn(ctx.clone()),
            ports,
            port_info,
            want_link: false,
            reconnect_at: None,
            connect_actions_at: None,
            single_axis_homing: false,
            fw: Default::default(),
            start_from: Default::default(),
            start_lines: vec![],
            cal: Default::default(),
            add_dev: None,
            add_dev_read: false,
            add_dev_read_done: false,
            add_dev_read_err: false,
            add_dev_was_reading: false,
            add_dev_skipped: false,
            focus_on: false,
            macros_ok: false,
            job_started: None,
            one_by_one: false,
            selection_only: false,
            job_est: 0.0,
            job_last_secs: 0.0,
            maintenance_notified: Default::default(),
            live_shown: None,
            pointer_align: false,
            connected: false,
            machine: (tr("Disconnected").into(), 0.0, 0.0),
            machine_z: f64::NAN,
            progress: (0, 0),
            console: Default::default(),
            side_tab: SideTab::Properties,
            plugin_tab: "",
            plugins: base_ui::plugins_ui::PluginsUi::new("lightcreator", &crate::data_paths().base),
        }
    }

    // ---------- document editing helpers ----------
    pub fn touch(&mut self) {
        self.revision += 1;
        self.doc.normalize_image_layers();
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
            self.ed.sel.retain(|id| self.doc.shape(*id).is_some());
            self.touch();
        }
    }
    pub fn do_redo(&mut self) {
        if let Some(d) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.doc, d));
            self.ed.sel.retain(|id| self.doc.shape(*id).is_some());
            self.touch();
        }
    }
    pub fn assign_layer(&mut self, layer: usize) {
        self.ed.active_layer = layer;
        if !self.ed.sel.is_empty() {
            self.checkpoint();
            for id in self.ed.sel.clone() {
                if let Some(s) = self.doc.unlocked_mut(id) {
                    s.layer = layer;
                }
            }
        }
    }
    pub fn make_array(&mut self) {
        let (cols, rows, dx, dy) = self.array;
        let Some(b) = self.sel_bounds() else { return };
        self.checkpoint();
        let base: Vec<Shape> = self.ed.sel.iter().filter_map(|i| self.doc.shape(*i).cloned()).collect();
        let mut new_sel = self.ed.sel.clone();
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
        self.ed.sel = new_sel;
    }

    // ---------- file operations ----------
    pub fn new_doc(&mut self) {
        let device = self.doc.device.clone();
        self.doc = Document::default();
        self.doc.device = device;
        self.doc.select_material(self.doc.device.active_material);
        self.sync_profile();
        self.undo.clear();
        self.redo.clear();
        self.ed.sel.clear();
        self.path = None;
        self.touch();
        self.mark_saved();
        self.ed.view.need_fit = true;
    }
    pub fn pick_project() -> Option<PathBuf> {
        rfd::FileDialog::new().add_filter("Project", &[lc_core::project::EXTENSION]).add_filter("SVG", &["svg"]).add_filter("DXF", &["dxf"]).pick_file()
    }
    pub fn open(&mut self) {
        if let Some(p) = Self::pick_project() {
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
    /// Open a project, an SVG or a LightBurn file; false when nothing came of it.
    pub fn open_path(&mut self, p: PathBuf) -> bool {
        if p.extension().is_some_and(|e| e.eq_ignore_ascii_case("lbrn2") || e.eq_ignore_ascii_case("lbrn")) {
            return self.import_lightburn(p);
        }
        if p.extension().is_some_and(|e| e.eq_ignore_ascii_case("ugsd")) {
            return self.import_ugsd(p);
        }
        if p.extension().is_some_and(|e| e.eq_ignore_ascii_case("dxf")) {
            return self.import_dxf_path(p);
        }
        let is_svg = p.extension().is_some_and(|e| e.eq_ignore_ascii_case("svg"));
        if is_svg {
            let ok = self.import_svg_path(p.clone());
            if ok {
                self.add_recent(&p);
            }
            return ok;
        }
        match std::fs::read_to_string(&p).map_err(|e| e.to_string()).and_then(|s| Document::from_json(&s)) {
            Ok(d) => {
                // A project goes into a tab of its own (a blank tab is used as it is); one that is open is shown.
                if !self.tab_for_file(&p) {
                    return self.path.as_ref().is_some_and(|q| q == &p || std::fs::canonicalize(q).ok() == std::fs::canonicalize(&p).ok());
                }
                let incoming = d.device.material().clone();
                let made_for = lc_core::Machine::of(&d.device);
                self.doc = d;
                let mut other_setup = vec![];
                if let Some(p) = self.profiles.get(self.active) {
                    other_setup = made_for.differences(&lc_core::Machine::of(p));
                    self.doc.device = p.clone();
                    // The material the project was made with joins the library of the device.
                    self.adopt_project_material(incoming);
                }
                self.undo.clear();
                self.redo.clear();
                self.ed.sel.clear();
                self.status = trf("Opened {}", &[&p.display()]);
                if !other_setup.is_empty() {
                    // The project was made for a machine that differs where it matters: say so before anything is burned.
                    let what = other_setup.iter().map(|w| tr(w)).collect::<Vec<_>>().join(", ");
                    self.toast(trf("This project was made for a different machine setup ({}). Check the device before you start.", &[&what]));
                }
                self.add_recent(&p);
                self.path = Some(p);
                self.touch();
                self.mark_saved();
                self.ed.view.need_fit = true;
                true
            }
            Err(e) => {
                self.status = trf("Open failed: {}", &[&e]);
                false
            }
        }
    }
    pub fn import_lightburn_dialog(&mut self) {
        if let Some(p) = rfd::FileDialog::new().add_filter("LightBurn", &["lbrn2", "lbrn"]).pick_file() {
            self.import_lightburn(p);
        }
    }
    /// Import a LightBurn project into the drawing: its shapes are added (and selected, and can be undone), and its
    /// layers with their mode, speed and power are taken for the layers that have no shapes yet. LightBurn counts Y up
    /// from the bottom, so the drawing is turned over with the height of the work area.
    pub fn import_lightburn(&mut self, p: PathBuf) -> bool {
        let device = self.profiles.get(self.active).cloned().unwrap_or_else(|| self.doc.device.clone());
        let read = std::fs::read(&p).map_err(|e| e.to_string()).and_then(|b| String::from_utf8(b).map_err(|_| "this is not a LightBurn project".to_string())).and_then(|t| lc_core::lightburn::import(&t, device.work_size().1));
        match read {
            Ok(imp) => {
                let warnings = imp.warnings.join("; ");
                let cut_layers: Vec<usize> = imp.cuts.iter().map(|c| c.index).collect();
                let doc = Document::from_lightburn(imp, device);
                self.add_imported(&p, doc, &cut_layers, &warnings, "LightBurn project")
            }
            Err(e) => {
                self.status = trf("Import failed: {}", &[&e]);
                false
            }
        }
    }
    pub fn import_ugsd_dialog(&mut self) {
        if let Some(p) = rfd::FileDialog::new().add_filter("UGS Designer", &["ugsd"]).pick_file() {
            self.import_ugsd(p);
        }
    }
    /// Import a UGS Designer file into the drawing: its shapes, and a layer for each way of cutting (outlines or fill)
    /// where no shapes are yet. Its milling feeds and depths mean nothing for a laser and are not taken.
    pub fn import_ugsd(&mut self, p: PathBuf) -> bool {
        let device = self.profiles.get(self.active).cloned().unwrap_or_else(|| self.doc.device.clone());
        let layers = self.doc.layers.len().max(1);
        let read = std::fs::read(&p).map_err(|e| e.to_string()).and_then(|b| String::from_utf8(b).map_err(|_| "this is not a UGS design".to_string())).and_then(|t| lc_core::ugsd::import(&t, device.work_size().1, layers));
        match read {
            Ok(imp) => {
                let warnings = imp.warnings.join("; ");
                let cut_layers: Vec<usize> = imp.cuts.iter().map(|c| c.index).collect();
                let doc = Document::from_ugsd(imp, device);
                self.add_imported(&p, doc, &cut_layers, &warnings, "UGS design")
            }
            Err(e) => {
                self.status = trf("Import failed: {}", &[&e]);
                false
            }
        }
    }
    /// Put an imported document into the drawing: the shapes are added and selected, the layers the import describes
    /// replace the ones without shapes.
    fn add_imported(&mut self, p: &std::path::Path, doc: Document, cut_layers: &[usize], warnings: &str, what: &str) -> bool {
        let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        self.checkpoint();
        let used: std::collections::HashSet<usize> = self.doc.scene.shapes.iter().map(|s| s.layer).collect();
        for &i in cut_layers.iter().filter(|i| !used.contains(i)) {
            if let (Some(to), Some(from)) = (self.doc.layers.get_mut(i), doc.layers.get(i)) {
                *to = from.clone();
            }
        }
        let n = doc.scene.shapes.len();
        let last = self.doc.layers.len().saturating_sub(1);
        self.ed.sel = self.doc.scene.append(doc.scene, last);
        self.status = trf("Imported {} paths from {}", &[&n, &p.display()]);
        self.touch();
        self.ed.view.need_fit = true;
        if warnings.is_empty() {
            self.toast(trf("Imported {} paths from {}", &[&n, &name]));
        } else if what == "UGS design" {
            self.toast(trf("Some parts of the UGS design could not be used: {}", &[&warnings]));
        } else {
            self.toast(trf("Some parts of the LightBurn project could not be used: {}", &[&warnings]));
        }
        true
    }
    pub fn save(&mut self, as_new: bool) {
        let path = if as_new || self.path.is_none() {
            rfd::FileDialog::new().add_filter("Project", &[lc_core::project::EXTENSION]).set_file_name("untitled.cre").save_file()
        } else {
            self.path.clone()
        };
        if let Some(p) = path {
            match std::fs::write(&p, self.doc.to_json()) {
                Ok(_) => {
                    self.status = trf("Saved {}", &[&p.display()]);
                    let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                    self.toast(trf("File saved: {}", &[&name]));
                    self.add_recent(&p);
                    self.path = Some(p);
                    self.mark_saved();
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
                self.ed.sel = ids;
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
        if self.guard(crate::check_ui::Pending::ExportGcode, None) {
            self.export_gcode_now();
        }
    }
    /// Are the variables in the very start and end of the file all known? When not, the reason goes to the status line and no
    /// file is made.
    pub fn frame_ok(&mut self) -> bool {
        match self.doc.device.gcode.check_frame() {
            Ok(()) => true,
            Err(e) => {
                self.status = trf("The own G-code of the device: {}", &[&e]);
                false
            }
        }
    }

    pub fn export_gcode_now(&mut self) {
        if !self.frame_ok() {
            return;
        }
        if self.selection_only && self.ed.sel.is_empty() {
            self.status = tr("Nothing to burn: nothing is selected").into();
            return;
        }
        self.stamp_job();
        if let Some(p) = rfd::FileDialog::new().add_filter("G-code", &["gcode", "nc"]).set_file_name("job.gcode").save_file() {
            let job = gcode::generate_with(&self.job_source(), None, &self.job_options());
            let Some(text) = self.postprocessed(job.gcode) else { return };
            self.status = match std::fs::write(&p, text) {
                Ok(_) => trf("Saved G-code {}", &[&p.display()]),
                Err(e) => trf("Export failed: {}", &[&e]),
            };
        }
    }

    // ---------- laser ----------
    /// Export the vector job as HPGL (.plt) or DXF for file-fed controllers (Ruida / Trocen software).
    pub fn export_cam(&mut self) {
        if self.guard(crate::check_ui::Pending::ExportCam, None) {
            self.export_cam_now();
        }
    }
    pub fn export_cam_now(&mut self) {
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
        if self.guard(crate::check_ui::Pending::Start, None) {
            self.send_job_now();
        }
    }
    /// Put the name of the file and the moment (UTC) where hooks can print them.
    pub fn stamp_job(&mut self) {
        let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let ymd = crate::backup_ui::ymd(secs);
        self.doc.job_info = lc_core::JobInfo {
            name: self.path.as_ref().and_then(|p| p.file_stem()).map(|s| s.to_string_lossy().into_owned()).unwrap_or_default(),
            date: format!("{}-{}-{}", &ymd[..4], &ymd[4..6], &ymd[6..]),
            time: format!("{:02}:{:02}", secs / 3600 % 24, secs / 60 % 60),
            path: self.path.as_ref().map(|p| p.display().to_string()).unwrap_or_default(),
        };
    }

    /// How far the job has got: a picture of the time-lapse for every layer that is done (in the "after each layer" mode).
    pub fn layer_progress(&mut self, done: usize, total: usize) {
        let now = std::time::Instant::now();
        if total == 0 {
            self.layer_watch.stop();
            return;
        }
        let mut n = self.layer_watch.progress(done);
        if done >= total {
            n += self.layer_watch.finished();
        }
        for _ in 0..n {
            self.cam_rec.layer_done(now);
        }
    }

    pub fn send_job_now(&mut self) {
        self.send_job_from(0);
    }

    /// The lines of the whole program, or `None` (with the reason in the status bar) when there is nothing to burn.
    pub fn program_lines(&mut self) -> Option<Vec<String>> {
        if !self.frame_ok() {
            return None;
        }
        self.focus_off();
        self.stamp_job();
        // The job is written for the design as it is: the pointer alignment never moves it.
        let (doc, back) = self.job_doc();
        let job = gcode::generate_with(&doc, back, &self.job_options());
        if job.moves.is_empty() {
            self.status = if self.selection_only && self.ed.sel.is_empty() { tr("Nothing to burn: nothing is selected") } else { tr("Nothing to burn: no shapes on output layers") }.into();
            return None;
        }
        self.job_est = job.est_seconds;
        self.job_marks = base_ui::link::layer_marks_in_text(&job.gcode);
        Some(laser::clean_gcode(&job.gcode))
    }

    /// Send the job, starting at line `from` of the program (0-based; 0 is all of it).
    pub fn send_job_from(&mut self, from: usize) {
        let Some(all) = self.program_lines() else { return };
        let lines = match lc_core::resume::plan(&all, from, 5.0, 0.0) {
            Ok(l) => l,
            Err(e) => {
                self.status = trf("Cannot start here: {}", &[&tr(e)]);
                return;
            }
        };
        self.push_console(ConsoleLine::info(trf("Starting job: {} lines, ~{}", &[&lines.len(), &fmt_time(self.job_est)])));
        self.job_last_secs = 0.0;
        // A picture after each layer of the time-lapse of the camera (the numbers fit a job that starts at the first line).
        if from == 0 {
            self.layer_watch.start(self.job_marks.clone());
        } else {
            self.layer_watch.stop();
        }
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
        self.frame_with(false);
    }

    /// Trace the box round the shapes (the selected ones, or all), or, with `outline`, the outline that wraps them like a
    /// rubber band.
    pub fn frame_with(&mut self, outline: bool) {
        if self.job_running() {
            return;
        }
        // The selected shapes, or all of them.
        let ids: Vec<u64> = if self.ed.sel.is_empty() { self.doc.scene.shapes.iter().map(|s| s.id).collect() } else { self.ed.sel.clone() };
        let b = self.sel_bounds().or_else(|| self.doc.bounds_of(&ids));
        let Some(b) = b else { return };
        if !self.connected {
            self.status = tr("Connect to a laser first (Laser panel)").into();
            return;
        }
        let moved = self.aim(self.placed_rect(b));
        // Along the outline: the rubber band round the shapes, moved the same way as the box would be.
        let path = outline.then(|| {
            let (dx, dy) = (moved.min.x - b.min.x, moved.min.y - b.min.y);
            self.doc.outline_of(&ids).into_iter().map(|p| Pt::new(p.x + dx, p.y + dy)).collect::<Vec<_>>()
        });
        let b = moved;
        if self.guard(crate::check_ui::Pending::Frame, Some(b)) {
            self.frame_bounds = Some(b);
            self.frame_path = path.filter(|p| p.len() >= 3);
            self.frame_now();
        }
    }

    /// Where the head stands, in design coordinates, when the position is known.
    pub fn head_point(&self) -> Option<Pt> {
        (self.connected && self.machine.1.is_finite() && self.machine.2.is_finite()).then(|| self.doc.device.from_machine(self.machine.1, self.machine.2))
    }

    /// The document as the laser gets it: as it is, or (start from the current position) moved so that its top-left
    /// corner lies on the head. The second value is where such a job ends, instead of the machine's zero.
    pub fn job_doc(&self) -> (std::borrow::Cow<'_, lc_core::Document>, Option<Pt>) {
        let source = self.job_source();
        match (self.doc.device.start_from, self.head_point()) {
            (lc_core::StartFrom::CurrentPosition, Some(head)) => (std::borrow::Cow::Owned(gcode::placed_at(&source, head)), Some(head)),
            _ => (source, None),
        }
    }

    /// What the job is made of: the whole drawing, or with "Selection only" the shapes that are selected now.
    pub fn job_source(&self) -> std::borrow::Cow<'_, lc_core::Document> {
        if self.selection_only {
            std::borrow::Cow::Owned(self.doc.only_shapes(&self.ed.sel))
        } else {
            std::borrow::Cow::Borrowed(&self.doc)
        }
    }

    /// How the job is made (the options of the Control tab).
    pub fn job_options(&self) -> gcode::JobOptions {
        gcode::JobOptions { one_by_one: self.one_by_one }
    }

    /// Changes whenever the drawing or what the job makes of it changes: the key of the previews of the job.
    pub fn job_stamp(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        (self.revision, self.one_by_one, self.selection_only).hash(&mut h);
        if self.selection_only {
            self.ed.sel.hash(&mut h);
        }
        h.finish()
    }

    /// A frame box moved the same way as the job: its top-left corner on the head when starting from the current position.
    pub fn placed_rect(&self, r: lc_core::Rect) -> lc_core::Rect {
        match (self.doc.device.start_from, self.head_point()) {
            (lc_core::StartFrom::CurrentPosition, Some(head)) => {
                let (dx, dy) = (head.x - r.min.x, head.y - r.min.y);
                lc_core::Rect { min: Pt::new(r.min.x + dx, r.min.y + dy), max: Pt::new(r.max.x + dx, r.max.y + dy) }
            }
            _ => r,
        }
    }

    /// Where the head has to go for the pointer dot to land on `r`: with the pointer alignment on, the box is moved
    /// back by the offset of the dot from the beam. Jobs and ordinary jogging are never moved.
    pub fn aim(&self, r: lc_core::Rect) -> lc_core::Rect {
        let h = self.doc.device.head(0);
        if !self.pointer_align || !h.has_pointer_offset() {
            return r;
        }
        let (dx, dy) = (-h.pointer_dx, -h.pointer_dy);
        lc_core::Rect { min: Pt::new(r.min.x + dx, r.min.y + dy), max: Pt::new(r.max.x + dx, r.max.y + dy) }
    }
    /// Trace the box that was checked (or that is being framed again after the window of issues).
    pub fn frame_now(&mut self) {
        let Some(b) = self.frame_bounds else { return };
        let h = self.doc.device.head(0);
        let g = match &self.frame_path {
            Some(path) => gcode::frame_path_gcode(path, &self.doc.device, h.frame_power, h.frame_speed),
            None => gcode::frame_gcode(b, &self.doc.device, h.frame_power, h.frame_speed),
        };
        for l in laser::clean_gcode(&g) {
            self.link.send(Cmd::Line(l));
        }
    }
    pub fn push_console(&mut self, l: ConsoleLine) {
        self.console.push_capped(l);
    }

    /// Count the time a job really runs: from its start until the machine is idle again after the last line, or until
    /// it is stopped or the link is lost.
    pub fn job_clock_tick(&mut self) {
        let running = self.progress.1 > 0 && self.progress.0 < self.progress.1;
        if running && self.job_started.is_none() {
            self.job_started = Some(std::time::Instant::now());
        }
        let Some(t0) = self.job_started else { return };
        let finished = self.progress.1 > 0 && self.progress.0 >= self.progress.1 && self.machine.0.starts_with("Idle");
        let stopped = self.progress.1 == 0 || !self.connected;
        if finished || stopped {
            self.job_started = None;
            self.job_last_secs = t0.elapsed().as_secs_f64();
            self.maintenance_add(t0.elapsed().as_secs_f64());
        }
    }

    /// A job has run for `secs`: the machine's total and its counters grow, and what has become due is announced.
    pub fn maintenance_add(&mut self, secs: f64) {
        self.doc.device.maintenance.add_run(secs);
        if let Some(p) = self.profiles.get_mut(self.active) {
            p.maintenance = self.doc.device.maintenance.clone();
        }
        let due: Vec<(usize, String)> = self.doc.device.maintenance.counters.iter().enumerate().filter(|(_, c)| c.is_due()).map(|(i, c)| (i, c.name.clone())).collect();
        for (i, name) in due {
            if self.maintenance_notified.insert(i) {
                let msg = trf("Maintenance due: {}", &[&name]);
                self.status = msg.clone();
                self.toast(msg);
            }
        }
    }

    /// The controller reported an alarm, or an error: an alarm always asks for attention, an error only in a running job.
    pub(crate) fn raise_problem(&mut self, alarm: bool, code: u32, explanation: String) {
        if alarm {
            let title = format!("{} {code}", tr("Alarm").to_uppercase());
            self.problem = Some(base_ui::machine_ctl::Problem { alarm: true, title, text: explanation, line: None });
        } else if self.job_running() {
            let text = if explanation.is_empty() { format!("error:{code}") } else { format!("error:{code}: {explanation}") };
            self.problem = Some(base_ui::machine_ctl::Problem { alarm: false, title: tr("The job stopped").to_string(), text, line: None });
        }
    }

    /// The big window for a problem, and what its buttons do.
    pub(crate) fn problem_window(&mut self, ctx: &egui::Context) {
        use base_ui::alert_ui::{forget_cleared_alarm, show, Choice};
        let in_alarm = self.machine.0.split(':').next().is_some_and(|s| s.eq_ignore_ascii_case("alarm"));
        let connected = self.connected;
        forget_cleared_alarm(ctx, &mut self.problem, in_alarm, connected);
        let Some(p) = self.problem.clone() else { return };
        match show(ctx, &p) {
            Choice::Nothing => {}
            Choice::Stop => {
                self.abort_job();
                if !p.alarm {
                    self.problem = None;
                }
            }
            Choice::Unlock => {
                let a = self.doc.device.link_controller().unlock();
                self.run_action(a);
                self.problem = None;
            }
            Choice::Close => self.problem = None,
        }
    }

    fn poll_laser(&mut self) {
        self.job_clock_tick();
        for e in self.link.poll() {
            match e {
                Evt::Log(s) => self.push_console(ConsoleLine::info(s)),
                Evt::Traffic(l) => {
                    let mut explain = None;
                    if l.dir == crate::laser::Dir::Rx && !l.poll {
                        self.read_feed(&l.text);
                        explain = lc_core::grbl_codes::explain(&l.text);
                    }
                    self.push_console(l);
                    // What an alarm or an error means, in a line under it and in the status bar.
                    if let Some(x) = explain {
                        let word = match x.kind {
                            lc_core::grbl_codes::Kind::Alarm => "Alarm",
                            lc_core::grbl_codes::Kind::Error => "Error",
                        };
                        if let Some(t) = x.text {
                            self.push_console(ConsoleLine::info(format!("   ↳ {}", tr(t))));
                            self.status = trf("{} {}: {}", &[&tr(word), &x.code, &tr(t)]);
                        }
                        self.raise_problem(x.kind == lc_core::grbl_codes::Kind::Alarm, x.code, x.text.map(|t| tr(t).to_string()).unwrap_or_default());
                    }
                }
                Evt::Connected(c) => {
                    let lost = self.connected && !c;
                    self.connected = c;
                    // A link that dropped on its own (not by the user's Disconnect) comes back by itself when asked to.
                    if lost && self.want_link && self.doc.device.connection.auto_reconnect {
                        self.reconnect_at = Some(std::time::Instant::now() + std::time::Duration::from_secs(3));
                    }
                    if c {
                        self.reconnect_at = None;
                    }
                    self.read_on_connected(c);
                    if !c {
                        self.live_shown = None;
                        self.pos.forget();
                        self.machine.0 = tr("Disconnected").into();
                        self.fw_warn = None;
                        self.stall = None;
                        self.focus_on = false;
                    }
                }
                Evt::Status { state, x, y, z } => {
                    self.machine = (state, x, y);
                    self.machine_z = z;
                }
                Evt::Progress { done, total } => {
                    self.progress = (done, total);
                    self.layer_progress(done, total);
                    if done >= total {
                        self.stall = None;
                    }
                }
                Evt::Stalled { secs, state } => self.stall = Some((secs, state)),
                // Pauses are left to the controller here, and the full report is not used.
                Evt::Report(r) => self.pos.take(&r),
                Evt::Barrier { .. } => {}
            }
        }
    }

    // ---------- UI ----------
    /// A second Escape within half a second goes back to the select tool.
    pub fn note_escape(&mut self, now: f64) {
        if now - self.esc_time < 0.5 {
            self.ed.tool = Tool::Select;
            self.esc_time = -10.0;
        } else {
            self.esc_time = now;
        }
    }

    #[cfg(test)]
    pub fn shortcuts_for_test(&mut self, ctx: &egui::Context) {
        self.shortcuts(ctx)
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
        // Ctrl+1 … Ctrl+0 show the tabs, so the two fit commands are Ctrl+Shift+0 and Ctrl+Shift+9.
        if cmd_shift(Key::Num0) {
            self.ed.view.need_fit = true;
            self.ed.view.auto_fit = true;
        }
        if cmd_shift(Key::Num9) {
            self.ed.view.auto_fit = false;
            self.ed.fit_all_req = true;
        }
        match base_ui::doc_tabs::shortcut(ctx) {
            Some(base_ui::doc_tabs::TabKey::Select(i)) => self.switch_to(i),
            Some(base_ui::doc_tabs::TabKey::Close) => self.close_tab_request(),
            None => {}
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
                self.new_tab();
            }
            if cmd(Key::Q) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        let plain = |k: Key| ctx.input_mut(|i| i.consume_key(Modifiers::NONE, k));
        if plain(Key::Delete) || plain(Key::Backspace) {
            if self.ed.tool == Tool::Node && !self.ed.node_sel.is_empty() {
                self.delete_nodes();
            } else {
                self.delete_selection();
            }
        }
        for (t, _, k) in Tool::ALL {
            if plain(k) {
                self.ed.tool = t;
                self.ed.pen_pts.clear();
                self.ed.show_shape_popup = matches!(t, Tool::Polygon | Tool::Spiral);
            }
        }
        if plain(Key::Escape) {
            let now = ctx.input(|i| i.time);
            self.note_escape(now);
            self.ed.pen_pts.clear();
            if self.ed.tool == Tool::Node && !self.ed.node_sel.is_empty() {
                self.ed.node_sel.clear();
            } else {
                self.ed.sel.clear();
            }
        }
        let step = if ctx.input(|i| i.modifiers.shift) { 10.0 } else { 1.0 };
        for (k, dx, dy) in [(Key::ArrowLeft, -step, 0.0), (Key::ArrowRight, step, 0.0), (Key::ArrowUp, 0.0, -step), (Key::ArrowDown, 0.0, step)] {
            if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, k)) || ctx.input_mut(|i| i.consume_key(Modifiers::SHIFT, k)) {
                if !self.ed.sel.is_empty() {
                    self.checkpoint();
                    self.transform_selection(Xf::translate(dx, dy));
                }
            }
        }
    }

    /// Is this action's check mark set? (None = not a check item.)
    pub fn act_checked(&self, a: Act) -> Option<bool> {
        if let Some(s) = a.std() {
            return s.checked(self);
        }
        match a {
            Act::TogglePreview => Some(self.preview_on),
            Act::ToggleOverlay => Some(self.overlay.is_some() && self.overlay_visible),
            Act::ToggleZones => Some(self.zones_visible),
            Act::SetLang(l) => Some(self.lang == l),
            Act::SetScheme(s) => Some(self.scheme == s),
            _ => None,
        }
    }

    pub fn act_enabled(&self, a: Act) -> bool {
        if let Some(s) = a.std() {
            return s.enabled(self);
        }
        match a {
            Act::ClearRecent => !self.recent.is_empty(),
            Act::Frame => self.connected && !self.job_running(),
            Act::StartJob => self.can_start() && !self.job_running(),
            Act::StartFrom => self.connected && !self.job_running(),
            Act::ToggleOverlay => self.overlay.is_some(),
            Act::ToggleZones => !self.doc.device.zones.is_empty(),
            _ => true,
        }
    }

    pub fn do_act(&mut self, ctx: &egui::Context, a: Act) {
        // On the start screen only application-level actions are available.
        if self.screen == Screen::Start {
            match a {
                // Starting a project takes the selected device, or the Dummy when there is none.
                Act::New => self.start_project(|app| app.new_tab_from_start()),
                Act::Open => {
                    if let Some(p) = Self::pick_project() {
                        self.start_project(|app| app.open_path(p));
                    }
                }
                Act::OpenRecent(i) => {
                    if let Some(p) = self.recent.get(i as usize).cloned() {
                        if p.is_file() {
                            self.start_project(|app| app.open_path(p));
                        } else {
                            self.status = trf("Open failed: {}", &[&format!("{} {}", p.display(), tr("no longer exists"))]);
                            self.recent.retain(|r| *r != p);
                        }
                    }
                }
                Act::ViewGcode => self.view_gcode_dialog(),
                Act::Quit | Act::About | Act::SetLang(_) | Act::SetScheme(_) | Act::RestoreBackup | Act::CheckUpdates => self.do_app_act(ctx, a),
                _ => {}
            }
            return;
        }
        self.do_app_act(ctx, a);
    }

    fn do_app_act(&mut self, ctx: &egui::Context, a: Act) {
        if let Some(s) = a.std() {
            s.run(self);
            return;
        }
        match a {
            Act::New => {
                self.new_tab();
            }
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
            Act::Print => self.open_print(),
            Act::ExportSvg => self.export_svg(),
            Act::ExportGcode => self.export_gcode(),
            Act::ExportCam => self.export_cam(),
            Act::Quit => self.request_close(ctx),
            Act::ImportAi => self.import_ai(),
            Act::ImportDxf => self.import_dxf(),
            Act::ImportLightburn => self.import_lightburn_dialog(),
            Act::ImportUgsd => self.import_ugsd_dialog(),
            Act::ViewGcode => self.view_gcode_dialog(),
            Act::ImportPlugin => self.import_with_plugin_dialog(),
            Act::Plugins => self.plugins.open = true,
            Act::Materials => self.open_materials_window(),
            Act::GenerateShape => self.plugins.gen_open = true,
            Act::OnlineClipart => {
                self.side_tab = SideTab::Clipart;
                self.open_online_clipart(ctx);
            }
            // The standard commands (also pictures and the operations on shapes) were run above, by baseCreator.
            Act::ToCurves | Act::AutoJoin | Act::ClosePath | Act::DogboneCorners | Act::CircleArray | Act::CopyAlongPath | Act::TextOnPath | Act::TrimByShape | Act::RemoveDuplicates | Act::BreakApart | Act::SimplifyShapes | Act::SerialText | Act::MakeCode | Act::NestShapes | Act::MeasureShapes | Act::ExportSelectionSvg | Act::ExportSelectionDxf | Act::BoolUnion | Act::BoolIntersect | Act::BoolSubtract | Act::BoolXor | Act::OffsetShape | Act::RoundCorners | Act::ImportImage | Act::AdjustImage | Act::TraceImage | Act::Tool(_) | Act::SplitView(_) | Act::SingleView => {}
            Act::QuickGuide => self.show_guide = true,
            Act::OpenGithub => ctx.open_url(egui::OpenUrl::new_tab(crate::help_ui::GITHUB_URL)),
            Act::CameraOverlay => self.show_overlay = true,
            Act::GridOptions => self.show_prefs = true,
            Act::ProgramOptions => self.show_options = true,
            Act::CheckUpdates => self.start_update_check(ctx, true),
            Act::DownloadBackup => self.download_backup(),
            Act::ExportDiagnostics => self.export_diagnostics(),
            Act::RestoreBackup => {
                let confirm = !self.profiles.is_empty();
                self.restore_backup_dialog(ctx, confirm);
            }
            Act::PreviewWindow => {
                self.show_preview = true;
                if self.preview_tab {
                    self.pv_tab_shown = true;
                    self.cam_tab_shown = false;
                }
            }
            Act::ShareSer2net => self.open_ser2net(),
            Act::ToggleOverlay => {
                if self.overlay.is_some() {
                    self.overlay_visible = !self.overlay_visible;
                } else {
                    self.status = tr("There is no camera overlay yet. Load a picture or start the camera view.").to_string();
                }
            }
            Act::CameraView => {
                if self.doc.device.stream_url().is_none() {
                    self.status = tr("Set a camera URL in the device settings first.").to_string();
                } else {
                    self.show_stream = true;
                }
            }
            Act::GridArray => self.show_array = true,
            Act::ToggleZones => self.zones_visible = !self.zones_visible,
            Act::TogglePreview => self.preview_on = !self.preview_on,
            Act::DeviceSettings => self.show_device = true,
            Act::SwitchDevice => {
                self.start_sel = self.active;
                self.confirm_delete = false;
                self.screen = Screen::Start;
            }
            Act::Frame => self.frame(),
            Act::StartJob => self.send_job(),
            Act::StartFrom => {
                if let Some(lines) = self.program_lines() {
                    self.start_lines = lines;
                    self.start_from.open = true;
                }
            }
            Act::Calibration => self.cal.open = true,
            Act::About => self.show_about = true,
            // The standard editing commands were carried out above.
            Act::Undo | Act::Redo | Act::Copy | Act::Paste | Act::Duplicate | Act::Delete | Act::SelectAll | Act::Align(_) | Act::CenterOnBed | Act::CenterEachOther | Act::FlipH | Act::FlipV | Act::RotCw | Act::RotCcw | Act::ToFront | Act::ToBack | Act::ToPath | Act::EditNodes | Act::Group | Act::Ungroup | Act::Lock | Act::Unlock | Act::ToggleGrid | Act::ToggleSnap | Act::ToggleSnapObjects | Act::FitBed | Act::FitAll | Act::ZoomIn | Act::ZoomOut | Act::ToggleAutoFit => {}
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

    #[cfg(target_os = "macos")]
    fn native_menu_frame(&mut self, ctx: &egui::Context) {
        let Some(mut nm) = self.native_menu.take() else { return };
        for a in nm.take_actions() {
            self.do_act(ctx, a);
        }
        nm.sync(|a| self.act_checked(a), |a| self.act_enabled(a), &self.recent);
        self.native_menu = Some(nm);
    }

    fn control_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_centered(|ui| {
            // Name of the current tool in a box of fixed width, so the bar does not jump around.
            base_ui::editor::control_name(self, ui);
            ui.separator();
            let ctx = ui.ctx().clone();
            for (act, icon, tip) in [(Act::New, "file-plus", "New"), (Act::Open, "folder-open", "Open…"), (Act::Save, "save", "Save")] {
                let ink = match act {
                    Act::Save => crate::window_ui::SAVE_GREEN,
                    Act::New => crate::window_ui::NEW_BLUE,
                    Act::Open => crate::window_ui::OPEN_ORANGE,
                    _ => theme::text(),
                };
                if icon_button_ink(ui, icon, &tr(tip), true, 28.0, ink) {
                    self.do_act(&ctx, act);
                }
            }
            ui.separator();
            base_ui::editor::grid_toggles(self, ui);
            ui.separator();
            self.view_buttons(ui);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if crate::window_ui::save_button(ui, &tr("Save G-code")).clicked() {
                    self.export_gcode();
                }
                base_ui::editor::selection_size(self, ui);
            });
        });
    }

    /// Homing needs a live link and no running job (framing counts as a job).
    pub fn can_home(&self) -> bool {
        self.connected && !self.job_running()
    }


    /// Space held down: pan with the hand tool, and go back to the previous tool when it is released.
    pub fn update_space(&mut self, down: bool) {
        if down && self.ed.space_prev.is_none() {
            if self.ed.tool != Tool::Pan {
                self.ed.space_prev = Some(self.ed.tool);
                self.ed.tool = Tool::Pan;
                self.ed.pen_pts.clear();
            }
        } else if !down {
            if let Some(t) = self.ed.space_prev.take() {
                if self.ed.tool == Tool::Pan {
                    self.ed.tool = t;
                }
            }
        }
    }

    /// Zoom by `f` around the middle of the canvas.
    pub fn zoom_about_centre(&mut self, f: f32) {
        let (w, h) = self.ed.canvas_avail;
        let mid = egui::vec2(w / 2.0, h / 2.0);
        let before = (mid - self.ed.view.pan) / self.ed.view.zoom;
        self.ed.view.zoom = (self.ed.view.zoom * f).clamp(0.05, 200.0);
        self.ed.view.pan = mid - before * self.ed.view.zoom;
        self.ed.view.auto_fit = false;
    }

    fn tool_bar(&mut self, ui: &mut egui::Ui) {
        // Shape operations on the selected objects sit between the tools that insert and those that navigate.
        let mut shape_ops = |app: &mut App, ui: &mut egui::Ui| {
            let multi = app.ed.sel.len() >= 2;
            let mut op = None;
            for (a, icon, tip) in [
                (Act::BoolUnion, "squares-unite", "Union"),
                (Act::BoolIntersect, "squares-intersect", "Intersection"),
                (Act::BoolSubtract, "squares-subtract", "Subtract"),
                (Act::BoolXor, "squares-exclude", "Exclusive or"),
            ] {
                if icon_button(ui, icon, &tr(tip), multi, base_ui::editor::palette_button_size(ui)) {
                    op = Some(a);
                }
            }
            if let Some(a) = op {
                let ctx = ui.ctx().clone();
                app.do_act(&ctx, a);
            }
        };
        base_ui::editor::tool_palette(self, ui, Some(&mut shape_ops));
    }

    /// Narrow strip between the work area and the side panel: align, turn, mirror and group the selection.
    fn arrange_bar(&mut self, ui: &mut egui::Ui) {
        base_ui::editor::arrange_bar(self, ui);
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_centered(|ui| {
            ui.label(&self.status);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                base_ui::editor::zoom_and_cursor(self, ui);
                let web = if self.api.server.is_some() { (trf("Web server: on, port {}", &[&self.api.server.as_ref().map(|s| s.addr.port()).unwrap_or(0)]), Color32::from_rgb(0x2e, 0xa0, 0x4f)) } else if self.api_on { (tr("Web server: error").to_string(), Color32::from_rgb(0xc0, 0x30, 0x30)) } else { (tr("Web server: off").to_string(), theme::text_dim()) };
                ui.label(RichText::new(web.0).color(web.1));
                ui.separator();
                let color = if self.connected { Color32::from_rgb(0x2e, 0xa0, 0x4f) } else { theme::text_dim() };
                ui.label(RichText::new(trf("Device: {}", &[&self.machine.0])).color(color));
                let (r, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                ui.painter().circle_filled(r.center(), 4.0, color);
                if self.progress.1 > 0 && self.progress.0 < self.progress.1 {
                    ui.add(egui::ProgressBar::new(self.progress.0 as f32 / self.progress.1 as f32).desired_width(120.0).show_percentage());
                }
            });
        });
    }

    /// Small window of the polygon and spiral tools: how many sides / turns. It sits in the corner of the work
    /// area (not over the bars), looks like the preview window and has no OK button: a change applies at once, to
    /// the shape drawn last as well as to the next ones. It stays while the tool is active.
    fn shape_popup(&mut self, ctx: &egui::Context) {
        let tool = self.ed.tool;
        if !self.ed.show_shape_popup || !matches!(tool, Tool::Polygon | Tool::Spiral) {
            return;
        }
        let polygon = tool == Tool::Polygon;
        let mut open = true;
        let before = if polygon { self.ed.polygon_sides } else { self.ed.spiral_turns };
        let frame = egui::Frame::window(&ctx.global_style()).inner_margin(egui::Margin { left: 8, right: 8, top: 6, bottom: 6 });
        let pos = self.ed.canvas_origin + egui::vec2(12.0, 12.0);
        egui::Window::new("shape_popup").title_bar(false).frame(frame).default_pos(pos).resizable(false).show(ctx, |ui| {
            crate::window_ui::mini_title(ui, &tr(if polygon { "Polygon" } else { "Spiral" }), &mut open, |_| {});
            let (value, range, label): (&mut u32, std::ops::RangeInclusive<u32>, &str) = if polygon {
                (&mut self.ed.polygon_sides, 3..=360, "Number of sides")
            } else {
                (&mut self.ed.spiral_turns, 1..=30, "Number of turns")
            };
            ui.horizontal(|ui| {
                ui.label(tr(label));
                ui.add(egui::DragValue::new(value).range(range.clone()));
            });
            // The usual choice sits on the slider (polygon 5 to 36, spiral 1 to 24); the number box takes more.
            let slider = if polygon { 5..=36 } else { 1..=24 };
            ui.spacing_mut().slider_width = 200.0;
            ui.add(egui::Slider::new(value, slider).clamping(egui::SliderClamping::Never).show_value(false));
            ui.label(RichText::new(tr("Drag on the work area to draw it.")).color(theme::text_dim()));
        });
        self.ed.polygon_sides = self.ed.polygon_sides.clamp(3, 360);
        self.ed.spiral_turns = self.ed.spiral_turns.clamp(1, 30);
        let now = if polygon { self.ed.polygon_sides } else { self.ed.spiral_turns };
        if now != before {
            self.reshape_last(tool);
        }
        self.ed.show_shape_popup = open;
    }

    /// Give the polygon or spiral drawn last the number of sides / turns now set, keeping its place and size.
    pub fn reshape_last(&mut self, tool: Tool) {
        let Some((t, id, w, h)) = self.ed.last_param else { return };
        if t != tool || self.doc.shape(id).is_none() {
            return;
        }
        let contour = if tool == Tool::Polygon { lc_core::Contour::polygon(self.ed.polygon_sides, w, h) } else { lc_core::Contour::spiral(self.ed.spiral_turns, w, h) };
        self.checkpoint();
        if let Some(s) = self.doc.unlocked_mut(id) {
            s.kind = lc_core::Kind::Bezier(vec![contour]);
            self.touch();
        }
    }

    fn dialogs(&mut self, ctx: &egui::Context) {
        self.shape_popup(ctx);
        self.materials_window(ctx);

        let mut open = self.show_array;
        let mut apply = false;
        crate::window_ui::popup(ctx, "array_window", &tr("Grid array"), [260.0, 220.0], &mut open, |ui| {
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

        let about = base_ui::dialogs::AboutInfo {
            title: &tr("About LightCreator"),
            name: "LightCreator",
            version: APP_VERSION,
            lines: vec![
                tr("Open-source design & control software for laser engravers and cutters.").to_string(),
                tr("Origin: written in Rust with egui/eframe.").to_string(),
                tr("Inspired by LightBurn; interface inspired by VectorCraft.").to_string(),
                tr("Author: Michał Sarna").to_string(),
            ],
            url: crate::help_ui::GITHUB_URL,
            licence: "MIT OR Apache-2.0",
        };
        base_ui::dialogs::about_window(ctx, &mut self.show_about, &about, self.logo.as_ref());
        self.config_window(ctx);
        self.layer_dialog(ctx);
        self.guide_window(ctx);
        self.clip_dialogs(ctx);
        self.online_clipart_window(ctx);
        self.plugin_windows(ctx);
        self.check_window(ctx);
        self.fw_window(ctx);
        self.stall_window(ctx);
        self.ai_choice_window(ctx);
        self.problem_window(ctx);
        self.fw_windows(ctx);
        let mut cal = std::mem::take(&mut self.cal);
        base_ui::tools_ui::calibration_window(ctx, &mut cal, self);
        self.cal = cal;
        if let Some(from) = base_ui::job_ui::start_from_window(ctx, &mut self.start_from, &self.start_lines, 5.0, 0.0) {
            self.send_job_from(from);
        }
        self.add_device_window(ctx);
        self.overlay_window(ctx);
        self.prefs_window(ctx);
        self.preview_window(ctx);
        self.image_dialogs(ctx);
        self.stream_window(ctx);
        self.ser2net_window(ctx);
        self.print_window(ctx);
        self.cam_test_window(ctx);
    }
}

/// `hh:mm:ss` with two digits for the hours, for the clock under the camera picture.
pub fn fmt_clock(s: f64) -> String {
    let s = if s.is_finite() && s > 0.0 { s as u64 } else { 0 };
    format!("{:02}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
}

pub fn fmt_time(s: f64) -> String {
    let s = s as u64;
    format!("{}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
}

impl base_ui::editor::EditorHost for App {
    fn panes_mut(&mut self) -> Option<&mut base_ui::work_panes::Panes> {
        Some(&mut self.panes)
    }
    fn pane_candidates(&self) -> Vec<base_ui::work_panes::Content> {
        use base_ui::work_panes::Content;
        let mut v: Vec<Content> = self.tabs_state.parked.iter().flatten().map(|w| Content::Doc(w.slot)).collect();
        if self.preview_work_tab() {
            v.insert(0, Content::Preview);
        }
        if self.gcode_view.is_some() {
            v.insert(0, Content::Gcode);
        }
        v.push(Content::Preview);
        v
    }
    fn view_options_extra(&mut self, ui: &mut egui::Ui) {
        self.camera_mode_options(ui);
    }
    fn ed(&self) -> &base_ui::editor::EditorState {
        &self.ed
    }
    fn ed_mut(&mut self) -> &mut base_ui::editor::EditorState {
        &mut self.ed
    }
    fn scene(&self) -> &lc_core::Scene {
        &self.doc.scene
    }
    fn scene_mut(&mut self) -> &mut lc_core::Scene {
        &mut self.doc.scene
    }
    fn layer_visible(&self, layer: usize) -> bool {
        self.doc.layers.get(layer).is_some_and(|l| l.visible)
    }
    fn checkpoint(&mut self) {
        App::checkpoint(self)
    }
    fn touch(&mut self) {
        App::touch(self)
    }
    fn undo(&mut self) {
        self.do_undo()
    }
    fn redo(&mut self) {
        self.do_redo()
    }
    fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
    fn drop_checkpoint(&mut self) {
        self.undo.pop();
    }
    fn delete_button(&self, ui: &mut egui::Ui, enabled: bool, text: &str) -> egui::Response {
        crate::window_ui::delete_button_enabled(ui, enabled, text)
    }
    fn work_size(&self) -> (f64, f64) {
        self.doc.device.work_size()
    }
    fn units(&self) -> lc_core::Units {
        self.doc.device.units
    }
    fn workspace_fill(&self) -> egui::Color32 {
        self.ed.grid_prefs.workspace_fill()
    }
    fn bed_fill(&self) -> egui::Color32 {
        self.ed.grid_prefs.bed_fill(self.scheme.is_dark())
    }
    fn shape_color(&self, layer: usize) -> Option<egui::Color32> {
        let l = self.doc.layers.get(layer)?;
        if !l.visible {
            return None;
        }
        let mut c = crate::canvas::col(l.color);
        if self.preview_on {
            c = c.gamma_multiply(0.25);
        } else if !l.output {
            c = c.gamma_multiply(0.45);
        }
        Some(c)
    }
    fn layer_color(&self, layer: usize) -> egui::Color32 {
        crate::canvas::col(layer)
    }
    fn dim_pictures(&self) -> bool {
        self.preview_on
    }
    fn paint_work_area(&mut self, v: &base_ui::editor::CanvasView) {
        self.paint_machine_area(v)
    }
    fn paint_under_shapes(&mut self, v: &base_ui::editor::CanvasView) {
        self.paint_camera_overlay(v)
    }
    fn paint_over_shapes(&mut self, v: &base_ui::editor::CanvasView) {
        self.paint_preview(v)
    }
    fn paint_lock(&self, ui: &egui::Ui, rect: egui::Rect) {
        crate::icons::paint(ui, rect, "lock", theme::accent());
    }
    fn pointer_claimed(&mut self, resp: &egui::Response, origin: egui::Pos2, press: Option<egui::Pos2>, hover: Option<egui::Pos2>) -> bool {
        self.overlay_interact(resp, origin, press, hover)
    }
    fn canvas_context_menu(&mut self, ui: &mut egui::Ui, resp: &egui::Response) {
        if self.overlay_edit {
            return;
        }
        base_ui::editor::context_menu(self, ui, resp);
    }
    fn place_text(&mut self, at: lc_core::Pt) {
        self.add_text_at(at)
    }
    fn open_shape_generators(&mut self) {
        self.plugins.gen_open = true;
    }
    fn open_online_clipart_search(&mut self, ctx: &egui::Context) {
        self.side_tab = SideTab::Clipart;
        base_ui::clipart_ui::ClipGallery::open_online_clipart(self, ctx);
    }
    fn say(&mut self, msg: String) {
        self.status = msg;
    }
    fn said(&self) -> &str {
        &self.status
    }
}

impl base_ui::menu::MenuHost for App {
    type Act = Act;
    fn menus(&self) -> base_ui::menu::Menus<Act> {
        menus()
    }
    fn act_enabled(&self, a: Act) -> bool {
        App::act_enabled(self, a)
    }
    fn act_checked(&self, a: Act) -> Option<bool> {
        App::act_checked(self, a)
    }
    fn act_icon(&self, a: Act) -> Option<&'static str> {
        crate::icons::act_icon(a)
    }
    fn recent_files(&self) -> Vec<PathBuf> {
        self.recent.clone()
    }
    fn open_recent_act(&self, i: usize) -> Option<Act> {
        Some(Act::OpenRecent(i as u8))
    }
    fn clear_recent_act(&self) -> Option<Act> {
        Some(Act::ClearRecent)
    }
    fn run_act(&mut self, ctx: &egui::Context, a: Act) {
        self.do_act(ctx, a)
    }
}

impl eframe::App for App {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // The focus beam must not stay on when the program goes away; give the link thread a moment to send the line.
        if self.focus_on {
            self.focus_off();
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        for (k, v) in self.settings_pairs() {
            storage.set_string(k, v);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.draw(ui);
    }
}

pub use base_ui::widgets::{icon_button, icon_button_ink, toggle_icon, window_toggle};

impl App {
    /// Everything the program remembers between runs, as the key / value strings kept in `app.ron`.
    pub fn settings_pairs(&self) -> Vec<(&'static str, String)> {
        let mut v: Vec<(&'static str, String)> = vec![("lang", self.lang.code().to_owned()), ("scheme", self.scheme.id().to_owned())];
        if let Ok(j) = serde_json::to_string(&self.profiles) {
            v.push(("profiles", j));
        }
        v.push(("active_profile", self.active.to_string()));
        v.push(("autosave_on", if self.autosave_on { "1" } else { "0" }.into()));
        v.push(("api_on", if self.api_on { "1" } else { "0" }.into()));
        v.push(("api_port", self.api_port.to_string()));
        v.push(("api_lan", if self.api_lan { "1" } else { "0" }.into()));
        v.push(("api_access", crate::api_server::access_to_setting(&self.api_access)));
        v.push(("autosave_secs", self.autosave_secs.to_string()));
        v.push(("check_updates", if self.check_updates { "1" } else { "0" }.into()));
        if let Ok(j) = serde_json::to_string(&self.recent) {
            v.push(("recent_files", j));
        }
        if let Ok(j) = serde_json::to_string(&self.ed.grid_prefs) {
            v.push(("grid_prefs", j));
        }
        v.push(("grid_main_on", if self.ed.show_grid { "1" } else { "0" }.into()));
        v.push(("snap_objects", if self.ed.snap_objects { "1" } else { "0" }.into()));
        v.push(("zones_visible", if self.zones_visible { "1" } else { "0" }.into()));
        v.push(("grid_main_mm", self.ed.grid.to_string()));
        v.push(("tab_order", self.tabs.order_string()));
        v.push(("panes", self.panes.to_json()));
        v.push(("side_collapsed", if self.tabs.collapsed { "1" } else { "0" }.into()));
        v
    }

    /// Draw one frame of the whole application into `ui`.
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.poll_laser();
        if self.preview_on && self.preview.as_ref().map(|p| p.0) != Some(self.job_stamp()) {
            self.preview = Some((self.job_stamp(), gcode::generate_with(&self.job_source(), None, &self.job_options())));
        }
        #[cfg(target_os = "macos")]
        self.native_menu_frame(&ctx);
        self.title_bar(ui);
        self.window_edges(&ctx);
        self.session_tick(&ctx);
        #[cfg(target_os = "macos")]
        self.window_edge_drag(&ctx);
        if self.screen == Screen::Start {
            // The first-run read from the device runs here too: it sends its request and gives up after a timeout.
            self.read_tick(&ctx);
            self.handle_drops(&ctx);
            if self.gcode_view.is_some() {
                egui::CentralPanel::no_frame().show(ui, |ui| {
                    self.gcode_view_panel(ui);
                });
            } else {
                self.start_screen(ui);
            }
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
        self.reconnect_tick(&ctx);
        self.overlay_poll_camera(&ctx);
        self.shortcuts(&ctx);

        let chrome = egui::Frame::new().fill(theme::panel()).stroke(egui::Stroke::new(1.0, theme::border())).inner_margin(egui::Margin::symmetric(8, 4));
        #[cfg(target_os = "macos")]
        let in_window_menu = self.native_menu.is_none();
        #[cfg(not(target_os = "macos"))]
        let in_window_menu = true;
        if in_window_menu {
            egui::Panel::top("menu").frame(chrome).show(ui, |ui| base_ui::menu::menu_bar(self, ui));
        }
        egui::Panel::top("controls").frame(chrome).exact_size(36.0).show(ui, |ui| self.control_bar(ui));
        egui::Panel::top("doc_tabs").frame(chrome.inner_margin(egui::Margin::symmetric(8, 2))).exact_size(30.0).show(ui, |ui| self.doc_tab_bar(ui));
        egui::Panel::bottom("status").frame(chrome).exact_size(26.0).show(ui, |ui| self.status_bar(ui));
        egui::Panel::left("tools").frame(chrome).exact_size(52.0).resizable(false).show(ui, |ui| self.tool_bar(ui));
        let side = if self.tabs.collapsed {
            egui::Panel::right("side_strip").frame(chrome.inner_margin(egui::Margin::same(3))).exact_size(base_ui::side_tabs::STRIP_W).resizable(false).show(ui, |ui| self.side_strip(ui))
        } else {
            egui::Panel::right("side").frame(chrome.inner_margin(egui::Margin::same(8))).default_size(410.0).resizable(!base_ui::window::pointer_over_window(ui.ctx())).show(ui, |ui| self.side_panel(ui))
        };
        base_ui::widgets::panel_edge_highlight(ui.ctx(), side.response.rect, base_ui::widgets::Edge::Left);
        egui::Panel::right("arrange_bar").frame(chrome.inner_margin(egui::Margin::symmetric(2, 4))).exact_size(40.0).resizable(false).show(ui, |ui| self.arrange_bar(ui));
        egui::Panel::bottom("swatches").frame(chrome).exact_size(36.0).show(ui, |ui| base_ui::layer_bar::layer_bar(self, ui));
        egui::CentralPanel::no_frame().show(ui, |ui| self.central_panes(ui));
        self.dialogs(&ctx);
        self.pane_windows(&ctx);
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
        a.ed.tool = Tool::Rect;
        a.note_escape(10.0);
        assert_eq!(a.ed.tool, Tool::Rect, "one Escape alone keeps the tool");
        a.note_escape(10.3);
        assert_eq!(a.ed.tool, Tool::Select, "the second one, quickly after, selects");
        // Slow presses never count as a pair.
        a.ed.tool = Tool::Star;
        a.note_escape(20.0);
        a.note_escape(21.0);
        assert_eq!(a.ed.tool, Tool::Star);
        a.note_escape(21.2);
        assert_eq!(a.ed.tool, Tool::Select);
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
        assert!(a.ed.view.auto_fit);
        a.do_act(&ctx, crate::menu::Act::ToggleAutoFit);
        assert!(!a.ed.view.auto_fit);
        a.do_act(&ctx, crate::menu::Act::FitBed);
        assert!(a.ed.view.auto_fit && a.ed.view.need_fit);
    }

    #[test]
    fn navigation_tools_are_a_separate_group() {
        let nav: Vec<Tool> = Tool::ALL.iter().map(|t| t.0).filter(|t| t.is_nav()).collect();
        assert_eq!(nav, vec![Tool::Pan, Tool::Zoom, Tool::ZoomOut]);
        assert!(Tool::ALL.iter().filter(|t| !t.0.is_nav()).all(|t| !matches!(t.0, Tool::Pan | Tool::Zoom | Tool::ZoomOut)));
    }

    #[test]
    fn an_alarm_always_raises_the_big_window_and_an_error_only_in_a_running_job() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        // An error with no job running is only a line in the console.
        a.raise_problem(false, 20, "Unsupported command".into());
        assert!(a.problem.is_none());
        // In a running job it stops the job: the window offers STOP and UNLOCK.
        a.progress = (3, 10);
        a.raise_problem(false, 20, "Unsupported command".into());
        let p = a.problem.clone().unwrap();
        assert!(!p.alarm && p.text.contains("error:20") && p.text.contains("Unsupported command"), "{p:?}");
        // An alarm raises it always, and replaces the error.
        a.progress = (0, 0);
        a.raise_problem(true, 2, "Soft limit".into());
        let p = a.problem.clone().unwrap();
        assert!(p.alarm && p.title.contains('2') && p.text == "Soft limit", "{p:?}");
        // The window goes away by itself when the alarm was cleared, but not before the alarm state was seen.
        a.connected = true;
        let frame = |a: &mut App| {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| a.problem_window(ui.ctx()));
            out.textures_delta.clear();
        };
        a.machine.0 = "Alarm".into();
        frame(&mut a);
        assert!(a.problem.is_some());
        a.machine.0 = "Idle".into();
        frame(&mut a);
        assert!(a.problem.is_none(), "cleared alarm: no window");
    }

    #[test]
    fn holding_space_pans_and_releasing_restores_the_tool() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.ed.tool = Tool::Rect;
        a.update_space(true);
        assert_eq!(a.ed.tool, Tool::Pan);
        a.update_space(true); // key repeat changes nothing
        assert_eq!(a.ed.tool, Tool::Pan);
        a.update_space(false);
        assert_eq!(a.ed.tool, Tool::Rect);
        // Already on the hand tool: nothing to restore, the tool stays.
        a.ed.tool = Tool::Pan;
        a.update_space(true);
        a.update_space(false);
        assert_eq!(a.ed.tool, Tool::Pan);
        // Choosing another tool while Space is held wins over the restore.
        a.ed.tool = Tool::Star;
        a.update_space(true);
        a.ed.tool = Tool::Ellipse;
        a.update_space(false);
        assert_eq!(a.ed.tool, Tool::Ellipse);
    }

    #[test]
    fn zoom_buttons_keep_the_middle_of_the_canvas_fixed() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.ed.canvas_avail = (800.0, 600.0);
        a.ed.view.zoom = 2.0;
        a.ed.view.pan = egui::vec2(100.0, 50.0);
        let mid = egui::vec2(400.0, 300.0);
        let world = (mid - a.ed.view.pan) / a.ed.view.zoom;
        a.zoom_about_centre(1.5);
        assert!((a.ed.view.zoom - 3.0).abs() < 1e-6);
        let again = (mid - a.ed.view.pan) / a.ed.view.zoom;
        assert!((again - world).length() < 1e-4);
        assert!(!a.ed.view.auto_fit);
        a.zoom_about_centre(1.0 / 1.5);
        assert!((a.ed.view.zoom - 2.0).abs() < 1e-6);
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
                let f = dir.join(format!("f{i}.cre"));
                std::fs::write(&f, Document::default().to_json()).unwrap();
                f
            })
            .collect();
        for f in &files {
            a.open_path(f.clone());
            // Every project opens in a tab of its own, and there are at most ten: close the one that was opened.
            if a.tab_count() > 1 {
                a.close_shown_tab();
            }
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
mod snap_objects_tests {
    use super::*;

    #[test]
    fn snapping_to_objects_is_a_saved_switch() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.enter_editor(0);
        a.screen = Screen::Editor;
        assert_eq!(a.act_checked(Act::ToggleSnapObjects), Some(false));
        a.do_act(&ctx, Act::ToggleSnapObjects);
        assert_eq!(a.act_checked(Act::ToggleSnapObjects), Some(true));
        assert!(a.settings_pairs().iter().any(|(k, v)| *k == "snap_objects" && v == "1"));
        assert!(a.act_enabled(Act::ToggleSnapObjects));
    }
}

#[cfg(test)]
mod home_button_tests {
    use super::*;

    #[test]
    fn home_is_off_without_a_link_and_while_a_job_runs() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.connected = false;
        assert!(!a.can_home());
        a.connected = true;
        assert!(a.can_home());
        a.progress = (1, 10);
        assert!(!a.can_home(), "running");
    }
}

#[cfg(test)]
mod version_tests {
    /// Without eframe's `links` feature `open_url` does nothing at all: the download button of the update window,
    /// the GitHub menu entry and every hyperlink would silently do nothing (issue #24).
    #[test]
    fn links_can_be_opened_in_the_browser() {
        let toml = include_str!("../Cargo.toml");
        let eframe = toml.lines().find(|l| l.trim_start().starts_with("eframe")).expect("eframe dependency");
        assert!(eframe.contains("\"links\""), "{eframe}");
    }

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
        a.ed.sel = vec![big, small];
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
        a.ed.sel = vec![big, g1, g2];
        let gap = a.doc.shape(g2).unwrap().bounds().unwrap().min.x - a.doc.shape(g1).unwrap().bounds().unwrap().min.x;
        a.do_act(&ctx, Act::CenterEachOther);
        let gb = a.doc.bounds_of(&[g1, g2]).unwrap().center();
        let bb = a.doc.shape(big).unwrap().bounds().unwrap().center();
        assert!((gb.x - bb.x).abs() < 1e-6 && (gb.y - bb.y).abs() < 1e-6);
        let gap2 = a.doc.shape(g2).unwrap().bounds().unwrap().min.x - a.doc.shape(g1).unwrap().bounds().unwrap().min.x;
        assert!((gap - gap2).abs() < 1e-9);
        // One object alone is not enough.
        a.ed.sel = vec![big];
        assert!(!a.act_enabled(Act::CenterEachOther));
    }
}

#[cfg(test)]
mod layer_speed_tests {
    use super::*;

    #[test]
    fn layer_settings_belong_to_the_project_not_to_the_device() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.profiles = vec![lc_core::Device::default()];
        a.enter_editor(0);
        let device_before = a.profiles[0].clone();
        // An edit in the layer panel changes the project's layer and leaves the device profile alone.
        let before = a.doc.layers.clone();
        a.doc.layers[2].speed = 30.0;
        a.commit_layer_edit(10.0, before);
        assert_eq!(a.doc.layers[2].speed, 30.0);
        assert_eq!(a.profiles[0], device_before);
        assert_eq!(a.doc.device.layer_speeds, device_before.layer_speeds);
        // A new document starts from the defaults, not from what was typed before.
        a.new_doc();
        assert!((a.doc.layers[2].speed - 1000.0 / 60.0).abs() < 1e-9);
        // The layers travel with the project file.
        a.doc.layers[2].speed = 12.0;
        let back = Document::from_json(&a.doc.to_json()).unwrap();
        assert_eq!(back.layers[2].speed, 12.0);
    }
}

#[cfg(test)]
mod tool_name_tests {
    use super::*;

    /// The control bar must not move when the tool changes: the box with the tool name has one width.
    #[test]
    fn tool_name_box_has_a_fixed_width() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.screen = Screen::Editor;
        let mut xs = vec![];
        for tool in [Tool::Select, Tool::Node, Tool::Polygon, Tool::Text] {
            a.ed.tool = tool;
            let mut x = 0.0;
            let mut out = ctx.run_ui(egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1400.0, 900.0))), ..Default::default() }, |ui| {
                a.draw(ui);
                x = ctx.data(|d| d.get_temp::<f32>(egui::Id::unique("tool_name_right"))).unwrap_or(-1.0);
            });
            // Debug builds insist that texture updates are consumed before the output is dropped.
            out.textures_delta.clear();
            xs.push(x);
        }
        assert!(xs.iter().all(|x| *x > 0.0), "{xs:?}");
        assert!(xs.windows(2).all(|w| (w[0] - w[1]).abs() < 0.01), "{xs:?}");
    }
}

#[cfg(test)]
mod shape_popup_tests {
    use super::*;

    #[test]
    fn changing_the_number_of_sides_reshapes_the_last_polygon() {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.screen = Screen::Editor;
        let ct = lc_core::Contour::polygon(6, 30.0, 20.0);
        let id = a.doc.add(0, lc_core::Kind::Bezier(vec![ct]), Xf::translate(10.0, 10.0));
        a.ed.last_param = Some((Tool::Polygon, id, 30.0, 20.0));
        a.ed.tool = Tool::Polygon;
        a.ed.polygon_sides = 9;
        a.reshape_last(Tool::Polygon);
        let nodes = match &a.doc.shape(id).unwrap().kind {
            lc_core::Kind::Bezier(cs) => cs[0].nodes.len(),
            _ => 0,
        };
        assert_eq!(nodes, 9);
        // The place is kept.
        assert_eq!(a.doc.shape(id).unwrap().xf.e, 10.0);
        // A spiral change does not touch a polygon.
        a.ed.spiral_turns = 5;
        a.reshape_last(Tool::Spiral);
        assert!(matches!(&a.doc.shape(id).unwrap().kind, lc_core::Kind::Bezier(cs) if cs[0].closed));
        // Spiral as the last shape: its turns change.
        let sp = a.doc.add(0, lc_core::Kind::Bezier(vec![lc_core::Contour::spiral(3, 30.0, 30.0)]), Xf::IDENTITY);
        a.ed.last_param = Some((Tool::Spiral, sp, 30.0, 30.0));
        a.ed.spiral_turns = 6;
        a.reshape_last(Tool::Spiral);
        assert!(matches!(&a.doc.shape(sp).unwrap().kind, lc_core::Kind::Bezier(cs) if cs[0].nodes.len() == 6 * 12 + 1));
    }

    #[test]
    fn spiral_tool_is_in_the_insert_group_with_its_own_key() {
        assert!(Tool::ALL.iter().any(|t| t.0 == Tool::Spiral && t.2 == egui::Key::I));
        assert!(!Tool::Spiral.is_edit() && !Tool::Spiral.is_nav());
    }
}

#[cfg(test)]
mod live_position_tests {
    use super::*;

    fn app() -> App {
        let ctx = egui::Context::default();
        let mut a = App::build(&ctx, None, false);
        a.connected = true;
        a.machine = ("Idle".into(), 10.0, 20.0);
        a
    }

    #[test]
    fn it_is_off_by_default_and_shows_nothing() {
        let mut a = app();
        assert!(!a.doc.device.live_position);
        assert_eq!(a.live_point(), None);
    }

    #[test]
    fn it_shows_the_head_in_design_coordinates_only_with_a_link() {
        let mut a = app();
        a.doc.device.live_position = true;
        let expected = a.doc.device.from_machine(10.0, 20.0);
        assert_eq!(a.live_point(), Some(expected));
        a.connected = false;
        assert_eq!(a.live_point(), None);
        a.connected = true;
        a.machine.1 = f64::NAN;
        assert_eq!(a.live_point(), None, "no position known yet");
    }

    #[test]
    fn homing_only_when_it_is_connected_and_the_machine_stands_still() {
        let mut a = app();
        assert!(a.live_position_homes());
        a.machine.0 = "Run".into();
        assert!(!a.live_position_homes(), "frame or jog under way");
        a.machine.0 = "Idle".into();
        a.progress = (1, 10);
        assert!(!a.live_position_homes(), "job running");
        a.progress = (0, 0);
        a.connected = false;
        assert!(!a.live_position_homes(), "no link");
        a.set_live_position(true);
        assert!(a.doc.device.live_position);
        assert!(a.live_shown.is_none());
    }

    #[test]
    fn while_the_machine_works_the_dot_moves_once_a_second() {
        let mut a = app();
        a.doc.device.live_position = true;
        a.machine.0 = "Run".into();
        let first = a.live_point().unwrap();
        a.machine.1 = 50.0;
        assert_eq!(a.live_point(), Some(first), "less than a second later");
        a.live_shown = a.live_shown.map(|(t, p)| (t - std::time::Duration::from_millis(1100), p));
        assert_eq!(a.live_point(), Some(a.doc.device.from_machine(50.0, 20.0)));
        // Standing still, it follows at once.
        a.machine.0 = "Idle".into();
        a.machine.1 = 60.0;
        assert_eq!(a.live_point(), Some(a.doc.device.from_machine(60.0, 20.0)));
    }

    #[test]
    fn the_job_options_of_the_control_tab_change_the_gcode() {
        let mut a = app();
        // Two groups, each a rectangle on layer 0 and a circle on layer 1.
        let mut ids = vec![];
        for x in [100.0, 10.0] {
            let r = a.doc.add(0, lc_core::Kind::Rect { w: 10.0, h: 10.0 }, lc_core::Xf::translate(x, 10.0));
            let c = a.doc.add(1, lc_core::Kind::Ellipse { w: 4.0, h: 4.0 }, lc_core::Xf::translate(x + 3.0, 13.0));
            let g = a.doc.scene.new_group_id();
            for s in a.doc.scene.shapes.iter_mut().filter(|s| s.id == r || s.id == c) {
                s.group = Some(g);
            }
            ids.push((r, c));
        }
        let layer_lines = |a: &mut App| -> usize { gcode::generate_with(&a.job_doc().0, None, &a.job_options()).gcode.lines().filter(|l| l.starts_with("; layer")).count() };
        // As before: a layer after a layer, two of them; one by one: both layers of each of the two groups.
        assert_eq!(layer_lines(&mut a), 2);
        a.one_by_one = true;
        assert_eq!(layer_lines(&mut a), 4);
        // Selection only: the second group (the one at x = 10), burnt completely; nothing selected burns nothing.
        a.one_by_one = false;
        a.selection_only = true;
        a.ed.sel = vec![ids[1].0, ids[1].1];
        assert_eq!(layer_lines(&mut a), 2);
        let all = { a.selection_only = false; a.program_lines().unwrap().len() };
        a.selection_only = true;
        assert!(a.program_lines().unwrap().len() < all, "only half of the drawing is burnt");
        // The previews of the job are rebuilt when an option or the selection changes.
        let with = a.job_stamp();
        a.selection_only = false;
        assert_ne!(with, a.job_stamp());
        a.selection_only = true;
        a.ed.sel.clear();
        assert!(a.program_lines().is_none());
        assert_eq!(a.status, "Nothing to burn: nothing is selected");
    }

    #[test]
    fn a_job_from_the_current_position_is_placed_on_the_head() {
        let mut a = app();
        a.doc.add(0, lc_core::Kind::Rect { w: 10.0, h: 10.0 }, lc_core::Xf::translate(50.0, 60.0));
        let head = a.doc.device.from_machine(10.0, 20.0);
        let (doc, back) = a.job_doc();
        assert!(back.is_none() && matches!(doc, std::borrow::Cow::Borrowed(_)), "from home nothing moves");
        drop(doc);
        a.doc.device.start_from = lc_core::StartFrom::CurrentPosition;
        let (doc, back) = a.job_doc();
        assert_eq!(back, Some(head));
        let c = gcode::burn_corner(&doc).unwrap();
        assert!(c.dist(head) < 1e-6, "{c:?} {head:?}");
        let r = a.placed_rect(lc_core::Rect { min: Pt::new(50.0, 60.0), max: Pt::new(60.0, 70.0) });
        assert!(r.min.dist(head) < 1e-6 && ((r.max.x - r.min.x) - 10.0).abs() < 1e-9);
        // Without a position (no link) the design stays where it is.
        a.connected = false;
        assert!(a.job_doc().1.is_none());
    }

    #[test]
    fn the_switch_is_saved_with_the_device_and_old_profiles_load_with_it_off() {
        let mut d = lc_core::Device::default();
        assert!(!d.live_position);
        d.live_position = true;
        let back: lc_core::Device = serde_json::from_str(&serde_json::to_string(&d).unwrap()).unwrap();
        assert!(back.live_position);
        let mut v: serde_json::Value = serde_json::to_value(&d).unwrap();
        v.as_object_mut().unwrap().remove("live_position");
        let old: lc_core::Device = serde_json::from_value(v).unwrap();
        assert!(!old.live_position);
    }
}

#[cfg(test)]
mod project_open_tests {
    use super::*;

    #[test]
    fn a_project_made_for_another_machine_setup_warns_when_opened() {
        let mut a = App::build(&egui::Context::default(), None, false);
        a.profiles = vec![lc_core::Device::default()];
        a.active = 0;
        let dir = std::env::temp_dir().join(format!("lc-open-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // The same machine: no warning.
        let same = dir.join("same.cre");
        std::fs::write(&same, Document::default().to_json()).unwrap();
        a.open_path(same);
        assert!(a.toast.is_none());
        // Another work area and controller: the warning names both.
        let mut d = Document::default();
        d.device.area.bed_w += 150.0;
        d.device.controller = lc_core::Controller::Marlin;
        let other = dir.join("other.cre");
        std::fs::write(&other, d.to_json()).unwrap();
        a.open_path(other);
        let (text, _) = a.toast.clone().expect("a warning");
        assert!(text.contains("Controller") && text.contains("Work area"), "{text}");
        // The device in use is still the local one.
        assert_eq!(a.doc.device.area.bed_w, lc_core::Device::default().area.bed_w);
        std::fs::remove_dir_all(dir).ok();
    }
}

#[cfg(test)]
mod lightburn_open_tests {
    use super::*;

    const LB: &str = r#"<LightBurnProject DeviceName="No Machine">
        <CutSetting type="Scan"><index Value="3"/><name Value="C03"/><maxPower Value="42"/><speed Value="25"/></CutSetting>
        <Shape Type="Rect" CutIndex="3" W="30" H="10" Cr="0"><XForm>1 0 0 1 50 20</XForm></Shape>
        <Shape Type="Mystery" CutIndex="3"/></LightBurnProject>"#;

    #[test]
    fn a_lightburn_project_opens_as_a_new_document_on_the_selected_device() {
        let mut a = App::build(&egui::Context::default(), None, false);
        a.profiles = vec![lc_core::Device::default()];
        a.active = 0;
        let dir = std::env::temp_dir().join(format!("lc-lb-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("job.lbrn2");
        std::fs::write(&file, LB).unwrap();
        let recent = a.recent.clone();
        a.import_lightburn(file);
        assert_eq!(a.recent, recent, "an import is not a project to reopen");
        assert_eq!(a.doc.scene.shapes.len(), 1);
        assert_eq!((a.doc.layers[3].mode, a.doc.layers[3].power, a.doc.layers[3].speed), (lc_core::LayerMode::Fill, 42.0, 25.0));
        assert!(a.path.is_none(), "the next save asks for a name");
        assert!(a.is_dirty(), "an imported project is unsaved work");
        // The unknown shape is named in the message.
        let (text, _) = a.toast.clone().expect("a message");
        assert!(text.contains("Mystery"), "{text}");
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn a_dxf_drawing_is_imported_into_the_document_with_its_layers_in_order_and_the_lines_joined() {
        let mut a = App::build(&egui::Context::default(), None, false);
        a.profiles = vec![lc_core::Device::default()];
        a.active = 0;
        let dir = std::env::temp_dir().join(format!("lc-dxf-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("part.dxf");
        let dxf = "0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n4\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLINE\n8\nOUTLINE\n10\n0\n20\n0\n11\n40\n21\n0\n0\nLINE\n8\nOUTLINE\n10\n40\n20\n0\n11\n40\n21\n20\n0\nLINE\n8\nOUTLINE\n10\n40\n20\n20\n11\n0\n21\n20\n0\nLINE\n8\nOUTLINE\n10\n0\n20\n20\n11\n0\n21\n0\n0\nCIRCLE\n8\nHOLES\n10\n20\n20\n10\n40\n3\n0\nENDSEC\n0\nEOF\n";
        std::fs::write(&file, dxf).unwrap();
        let recent = a.recent.clone();
        assert!(a.import_dxf_path(file));
        assert_eq!(a.recent, recent, "an import is not a project to reopen");
        let on = |l: usize| a.doc.scene.shapes.iter().filter(|s| s.layer == l).count();
        assert_eq!((on(0), on(1)), (1, 1), "the four lines are one path on layer 1, the circle is on layer 2: {}", a.status);
        assert!(a.status.contains("Imported 2 paths"), "{}", a.status);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn a_ugs_design_opens_as_a_new_document_with_a_layer_for_each_cut() {
        let mut a = App::build(&egui::Context::default(), None, false);
        a.profiles = vec![lc_core::Device::default()];
        a.active = 0;
        let dir = std::env::temp_dir().join(format!("lc-ugsd-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("job.ugsd");
        std::fs::write(&file, r#"{"version":"1","entities":[
          {"type":"RECTANGLE","cutType":"POCKET","cutDepth":4,"feedRate":700,"transform":{"m00":30,"m10":0,"m01":0,"m11":10,"m02":50,"m12":20}},
          {"type":"ELLIPSE","cutType":"OUTSIDE_PATH","cutDepth":4,"feedRate":700,"transform":{"m00":3,"m10":0,"m01":0,"m11":3,"m02":5,"m12":5}}]}"#).unwrap();
        a.doc.add(2, lc_core::Kind::Rect { w: 5.0, h: 5.0 }, lc_core::Xf::IDENTITY);
        a.doc.layers[2].mode = lc_core::LayerMode::Fill;
        let recent = a.recent.clone();
        assert!(a.open_path(file));
        assert_eq!(a.recent, recent, "an import is not a project to reopen");
        assert_eq!(a.doc.scene.shapes.len(), 3, "added to the drawing");
        assert_eq!(a.ed.sel.len(), 2);
        assert_eq!(a.doc.layers[0].mode, lc_core::LayerMode::Fill);
        assert_eq!(a.doc.layers[1].mode, lc_core::LayerMode::Line);
        a.do_undo();
        assert_eq!(a.doc.scene.shapes.len(), 1);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn a_broken_lightburn_file_changes_nothing() {
        let mut a = App::build(&egui::Context::default(), None, false);
        a.doc.add(0, lc_core::Kind::Rect { w: 5.0, h: 5.0 }, lc_core::Xf::IDENTITY);
        let dir = std::env::temp_dir().join(format!("lc-lb-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("bad.lbrn2");
        std::fs::write(&file, "not xml at all").unwrap();
        a.open_path(file);
        assert_eq!(a.doc.scene.shapes.len(), 1);
        assert!(a.status.starts_with("Import failed"), "{}", a.status);
        std::fs::remove_dir_all(dir).ok();
    }
}

impl base_ui::tools_ui::CalHost for App {
    fn connected(&self) -> bool {
        self.connected
    }
    fn setting(&self, id: u32) -> Option<f64> {
        self.fw.values.get(&id).and_then(|v| v.trim().parse::<f64>().ok())
    }
    fn reading(&self) -> bool {
        self.fw.reading
    }
    fn read_settings(&mut self) {
        self.fw_read();
    }
    fn job_running(&self) -> bool {
        App::job_running(self)
    }
    fn send(&mut self, lines: Vec<String>) {
        for l in lines {
            self.link.send(Cmd::Line(l));
        }
    }
    fn ask_write(&mut self, id: u32, value: String) {
        self.run_fw_acts(vec![crate::firmware_ui::FwAct::Write(id, value)]);
    }
}

impl base_ui::layer_bar::LayerBarHost for App {
    fn layer_label(&self, layer: usize) -> String {
        // The description follows the name, so that the tip of the layer bar tells what the layer is for.
        self.doc.layers.get(layer).map(|l| if l.description.trim().is_empty() { l.name.clone() } else { format!("{} — {}", l.name, l.description.trim()) }).unwrap_or_default()
    }
    fn open_layer_options(&mut self, layer: usize) {
        self.layer_dlg = Some(layer);
    }
    fn click_layer(&mut self, layer: usize) {
        self.assign_layer(layer);
    }
}
