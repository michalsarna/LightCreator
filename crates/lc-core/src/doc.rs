//! Document model: shapes, layers (LightBurn "cuts"), and machine settings.
use crate::bezier::Contour;
use crate::image::ImageData;
use crate::text::TextData;
use crate::geom::*;
use serde::{Deserialize, Serialize};

/// The 30 layer colours LightBurn uses (C00..C29).
pub const PALETTE: [[u8; 3]; 30] = [
    [0x00, 0x00, 0x00], [0x00, 0x00, 0xFF], [0xFF, 0x00, 0x00], [0x00, 0xE0, 0x00],
    [0xD0, 0xD0, 0x00], [0xFF, 0x80, 0x00], [0x00, 0xE0, 0xE0], [0xFF, 0x00, 0xFF],
    [0xB4, 0xB4, 0xB4], [0x00, 0x00, 0xA0], [0xA0, 0x00, 0x00], [0x00, 0xA0, 0x00],
    [0xA0, 0xA0, 0x00], [0xC0, 0x80, 0x00], [0x00, 0xA0, 0xFF], [0xA0, 0x00, 0xA0],
    [0x80, 0x80, 0x80], [0x7D, 0x87, 0xB9], [0xBB, 0x77, 0x84], [0x4A, 0x6F, 0xE3],
    [0xD3, 0x3F, 0x6A], [0x8C, 0xD7, 0x8C], [0xF0, 0xB9, 0x8D], [0xF6, 0xC4, 0xE1],
    [0xFA, 0x9E, 0xD4], [0x50, 0x0A, 0x78], [0xB4, 0x5A, 0x00], [0x00, 0x47, 0x54],
    [0x86, 0xFA, 0x88], [0xFF, 0xDB, 0x66],
];

/// Index of the palette colour closest to `rgb`.
pub fn nearest_palette(rgb: [u8; 3]) -> usize {
    let d = |c: &[u8; 3]| -> i32 {
        (0..3).map(|i| (c[i] as i32 - rgb[i] as i32).pow(2)).sum()
    };
    (0..PALETTE.len()).min_by_key(|&i| d(&PALETTE[i])).unwrap_or(0)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerMode {
    /// Follow the outlines.
    Line,
    /// Concentric inward rings (offset fill) of closed shapes.
    Offset,
    /// Raster-scan the inside of closed shapes.
    Fill,
    /// Fill the interior, then cut the outline.
    FillAndLine,
}

impl LayerMode {
    pub const ALL: [LayerMode; 4] = [LayerMode::Line, LayerMode::Fill, LayerMode::FillAndLine, LayerMode::Offset];
    pub fn label(self) -> &'static str {
        match self {
            LayerMode::Line => "Line",
            LayerMode::Fill => "Fill",
            LayerMode::Offset => "Offset fill",
            LayerMode::FillAndLine => "Fill + Line",
        }
    }
}

/// How a grayscale image is turned into laser dots.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dither {
    /// Black or white at 50 %.
    Threshold,
    #[default]
    FloydSteinberg,
    Jarvis,
    Stucki,
    Atkinson,
    /// 8x8 Bayer matrix.
    Ordered,
    /// Continuous power between `min_power` and `power`.
    Grayscale,
}

impl Dither {
    pub const ALL: [Dither; 7] = [Dither::Threshold, Dither::FloydSteinberg, Dither::Jarvis, Dither::Stucki, Dither::Atkinson, Dither::Ordered, Dither::Grayscale];
    pub fn label(self) -> &'static str {
        match self {
            Dither::Threshold => "Threshold",
            Dither::FloydSteinberg => "Floyd-Steinberg",
            Dither::Jarvis => "Jarvis",
            Dither::Stucki => "Stucki",
            Dither::Atkinson => "Atkinson",
            Dither::Ordered => "Ordered",
            Dither::Grayscale => "Grayscale",
        }
    }
}

fn d_min_power() -> f64 {
    0.0
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    pub color: usize,
    pub mode: LayerMode,
    /// mm/s
    pub speed: f64,
    /// Max power, percent.
    pub power: f64,
    pub passes: u32,
    /// Fill scan-line spacing in mm.
    pub interval: f64,
    /// Fill scan angle in degrees.
    pub angle: f64,
    /// Fill overscan (mm) on each end of a scan line.
    pub overscan: f64,
    pub bidirectional: bool,
    pub output: bool,
    pub visible: bool,
    /// Image engraving: how grays become dots.
    #[serde(default)]
    pub dither: Dither,
    /// Image engraving: power (percent) for the lightest burnt dot in grayscale mode.
    #[serde(default = "d_min_power")]
    pub min_power: f64,
}

impl Layer {
    pub fn new(color: usize) -> Layer {
        Layer {
            name: format!("C{:02}", color),
            color,
            mode: LayerMode::Line,
            speed: 20.0,
            power: 20.0,
            passes: 1,
            interval: 0.1,
            angle: 0.0,
            overscan: 1.0,
            bidirectional: true,
            output: true,
            visible: true,
            dither: Dither::default(),
            min_power: 0.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Kind {
    Rect { w: f64, h: f64 },
    Ellipse { w: f64, h: f64 },
    /// Pre-flattened polylines in local coordinates.
    Path(Vec<Polyline>),
    /// Editable Bézier contours in local coordinates.
    Bezier(Vec<Contour>),
    /// A bitmap engraved line by line. Local size is `w` x `h` millimetres.
    Image(ImageData),
    /// Live text, rendered to outlines on demand.
    Text(TextData),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shape {
    pub id: u64,
    pub layer: usize,
    pub kind: Kind,
    pub xf: Xf,
    /// Shapes with the same group id move and select together. Grouping never changes the layer.
    #[serde(default)]
    pub group: Option<u64>,
}

impl Shape {
    pub fn local_polys(&self) -> Vec<Polyline> {
        match &self.kind {
            Kind::Rect { w, h } => vec![Polyline::new(
                vec![Pt::new(0.0, 0.0), Pt::new(*w, 0.0), Pt::new(*w, *h), Pt::new(0.0, *h)],
                true,
            )],
            Kind::Ellipse { w, h } => {
                let n = 128;
                let pts = (0..n)
                    .map(|i| {
                        let t = i as f64 / n as f64 * std::f64::consts::TAU;
                        Pt::new(w / 2.0 + w / 2.0 * t.cos(), h / 2.0 + h / 2.0 * t.sin())
                    })
                    .collect();
                vec![Polyline::new(pts, true)]
            }
            Kind::Path(p) => p.clone(),
            Kind::Bezier(cs) => cs.iter().map(|c| c.flatten(0.05)).collect(),
            Kind::Image(im) => vec![Polyline::new(vec![Pt::new(0.0, 0.0), Pt::new(im.w, 0.0), Pt::new(im.w, im.h), Pt::new(0.0, im.h)], true)],
            Kind::Text(t) => crate::text::contours(t).iter().map(|c| c.flatten(0.05)).collect(),
        }
    }
    pub fn polys(&self) -> Vec<Polyline> {
        self.local_polys().iter().map(|p| p.transformed(&self.xf)).collect()
    }
    pub fn bounds(&self) -> Option<Rect> {
        self.polys().iter().filter_map(|p| p.bounds()).reduce(Rect::union)
    }
    /// Distance from a point to this shape (0 if inside a closed shape).
    pub fn hit_dist(&self, p: Pt) -> f64 {
        let polys = self.polys();
        let inside = polys.iter().filter(|q| q.closed).fold(false, |acc, q| acc ^ q.contains(p));
        if inside {
            return 0.0;
        }
        polys.iter().map(|q| q.dist_to(p)).fold(f64::INFINITY, f64::min)
    }
    /// Contours of this shape in local coordinates, as editable Bézier data.
    pub fn local_contours(&self) -> Vec<Contour> {
        match &self.kind {
            Kind::Rect { w, h } => vec![Contour::rect(*w, *h)],
            Kind::Ellipse { w, h } => vec![Contour::ellipse(*w, *h)],
            Kind::Path(p) => p.iter().map(Contour::from_polyline).collect(),
            Kind::Bezier(c) => c.clone(),
            Kind::Image(im) => vec![Contour::rect(im.w, im.h)],
            Kind::Text(t) => crate::text::contours(t).as_ref().clone(),
        }
    }
    pub fn is_image(&self) -> bool {
        matches!(self.kind, Kind::Image(_))
    }
    /// Convert into editable Bézier contours, baking the transform into the nodes.
    pub fn to_bezier(&mut self) {
        let xf = self.xf;
        let cs: Vec<Contour> = self.local_contours().iter().map(|c| c.transformed(&xf)).collect();
        self.kind = Kind::Bezier(cs);
        self.xf = Xf::IDENTITY;
    }
    /// Convert any shape into a generic path (baking the transform in).
    pub fn bake(&mut self) {
        if self.is_image() {
            return;
        }
        let polys = self.polys();
        self.kind = Kind::Path(polys);
        self.xf = Xf::IDENTITY;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Origin {
    /// Machine zero at the front-left corner (GRBL default); Y is flipped on output.
    FrontLeft,
    /// Machine zero at the back-left corner.
    BackLeft,
}

/// Unit system used to show and enter lengths in the interface. All stored geometry is in millimetres.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Units {
    #[default]
    Mm,
    Inch,
}

impl Units {
    pub const ALL: [Units; 2] = [Units::Mm, Units::Inch];
    pub fn from_mm(self, mm: f64) -> f64 {
        match self {
            Units::Mm => mm,
            Units::Inch => mm / 25.4,
        }
    }
    pub fn to_mm(self, v: f64) -> f64 {
        match self {
            Units::Mm => v,
            Units::Inch => v * 25.4,
        }
    }
    /// Suffix for lengths, with a leading space.
    pub fn suffix(self) -> &'static str {
        match self {
            Units::Mm => " mm",
            Units::Inch => " in",
        }
    }
    pub fn speed_s_suffix(self) -> &'static str {
        match self {
            Units::Mm => " mm/s",
            Units::Inch => " in/s",
        }
    }
    pub fn speed_min_suffix(self) -> &'static str {
        match self {
            Units::Mm => " mm/min",
            Units::Inch => " in/min",
        }
    }
    /// Decimal places that make sense when entering lengths.
    pub fn decimals(self) -> usize {
        match self {
            Units::Mm => 3,
            Units::Inch => 4,
        }
    }
    /// English label (translated in the UI).
    pub fn label(self) -> &'static str {
        match self {
            Units::Mm => "Millimetres (mm)",
            Units::Inch => "Inches (in)",
        }
    }
}

/// Which controller family a device speaks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Controller {
    /// GRBL 1.1 over a serial port (streamed live).
    #[default]
    Grbl,
    /// Marlin firmware with laser support over a serial port (streamed live).
    Marlin,
    /// Ruida DSP controllers: jobs are exported as `.rd` files.
    Ruida,
    /// Trocen DSP controllers: jobs are exported as HPGL `.plt` files.
    Trocen,
}

impl Controller {
    pub const ALL: [Controller; 4] = [Controller::Grbl, Controller::Marlin, Controller::Ruida, Controller::Trocen];
    pub fn label(self) -> &'static str {
        match self {
            Controller::Grbl => "GRBL",
            Controller::Marlin => "Marlin",
            Controller::Ruida => "Ruida",
            Controller::Trocen => "Trocen",
        }
    }
    /// Can LightCreator talk to this controller over a serial port?
    pub fn is_serial(self) -> bool {
        matches!(self, Controller::Grbl | Controller::Marlin)
    }
}

/// Laser source type; selects matching entries in the material library.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LaserKind {
    #[default]
    Diode,
    Co2,
}

impl LaserKind {
    pub const ALL: [LaserKind; 2] = [LaserKind::Diode, LaserKind::Co2];
    pub fn label(self) -> &'static str {
        match self {
            LaserKind::Diode => "Diode",
            LaserKind::Co2 => "CO2",
        }
    }
}

/// Camera overlay calibration stored with a device profile.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CameraCfg {
    /// Work-area positions (mm) of the picture's corners: top-left, top-right, bottom-right, bottom-left.
    pub corners: [Pt; 4],
    pub opacity: f64,
    /// Which camera to use (index in the system's camera list).
    #[serde(default)]
    pub index: u32,
}

/// How LightCreator reaches the controller.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkKind {
    /// A serial (USB) port.
    #[default]
    Serial,
    /// A TCP socket, for example a Raspberry Pi running `ser2net`.
    Tcp,
}

impl LinkKind {
    pub const ALL: [LinkKind; 2] = [LinkKind::Serial, LinkKind::Tcp];
    pub fn label(self) -> &'static str {
        match self {
            LinkKind::Serial => "Serial port",
            LinkKind::Tcp => "Network (TCP, e.g. ser2net)",
        }
    }
}

fn d_tcp_port() -> u16 {
    3333
}

fn d_jog_step() -> f64 {
    5.0
}
fn d_jog_feed() -> f64 {
    3000.0
}

/// A device (machine) profile: everything the Device tab shows and the job generators need.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Device {
    pub name: String,
    /// Display units; profiles saved before this existed load as millimetres.
    #[serde(default)]
    pub units: Units,
    #[serde(default)]
    pub controller: Controller,
    #[serde(default)]
    pub laser: LaserKind,
    pub bed_w: f64,
    pub bed_h: f64,
    pub origin: Origin,
    /// GRBL `$30`, the S value that equals 100% power.
    pub s_max: f64,
    /// Use `M4` dynamic power (recommended) instead of `M3`.
    pub dynamic_power: bool,
    pub travel_speed: f64,
    pub return_home: bool,
    pub baud: u32,
    /// Serial port used for this device (empty = none chosen yet).
    #[serde(default)]
    pub port: String,
    /// Jog step in mm.
    #[serde(default = "d_jog_step")]
    pub jog_step: f64,
    /// Jog feed in mm/min.
    #[serde(default = "d_jog_feed")]
    pub jog_feed: f64,
    /// Laser power (percent) while framing; 0 keeps the laser off.
    #[serde(default)]
    pub frame_power: f64,
    /// Camera overlay alignment for this machine.
    #[serde(default)]
    pub camera: Option<CameraCfg>,
    /// Serial port or TCP socket.
    #[serde(default)]
    pub link: LinkKind,
    /// TCP host (name or address) when `link` is `Tcp`.
    #[serde(default)]
    pub host: String,
    #[serde(default = "d_tcp_port")]
    pub tcp_port: u16,
    /// URL of a live picture of the machine (MJPEG stream or JPEG snapshot). Empty = none.
    #[serde(default)]
    pub camera_url: String,
}

impl Device {
    /// Where the machine's zero is, in design coordinates (millimetres, y down). Jobs start and end here.
    pub fn home_point(&self) -> Pt {
        match self.origin {
            Origin::FrontLeft => Pt::new(0.0, self.bed_h),
            Origin::BackLeft => Pt::new(0.0, 0.0),
        }
    }

    /// Short description of the connection target.
    pub fn target_label(&self) -> String {
        match self.link {
            LinkKind::Serial => self.port.clone(),
            LinkKind::Tcp => format!("{}:{}", self.host, self.tcp_port),
        }
    }

    /// Is a connection target configured?
    pub fn has_target(&self) -> bool {
        match self.link {
            LinkKind::Serial => !self.port.is_empty(),
            LinkKind::Tcp => !self.host.trim().is_empty(),
        }
    }
}

impl Default for Device {
    fn default() -> Self {
        Device {
            name: "GRBL 1.1 (diode / CO2)".into(),
            units: Units::Mm,
            controller: Controller::Grbl,
            laser: LaserKind::Diode,
            bed_w: 400.0,
            bed_h: 400.0,
            origin: Origin::FrontLeft,
            s_max: 1000.0,
            dynamic_power: true,
            travel_speed: 3000.0,
            return_home: true,
            baud: 115_200,
            port: String::new(),
            jog_step: 5.0,
            jog_feed: 3000.0,
            frame_power: 0.0,
            camera: None,
            link: LinkKind::Serial,
            host: String::new(),
            tcp_port: 3333,
            camera_url: String::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document {
    pub shapes: Vec<Shape>,
    pub layers: Vec<Layer>,
    pub device: Device,
    next_id: u64,
    /// Order in which layers are output (a permutation of the layer indices).
    #[serde(default = "identity_order")]
    pub order: Vec<usize>,
    #[serde(default)]
    next_group: u64,
}

fn identity_order() -> Vec<usize> {
    (0..30).collect()
}

impl Default for Document {
    fn default() -> Self {
        Document {
            shapes: vec![],
            layers: (0..30).map(Layer::new).collect(),
            device: Device::default(),
            next_id: 1,
            order: identity_order(),
            next_group: 1,
        }
    }
}

impl Document {
    pub fn add(&mut self, layer: usize, kind: Kind, xf: Xf) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.shapes.push(Shape { id, layer, kind, xf, group: None });
        id
    }
    pub fn add_shape(&mut self, mut s: Shape) -> u64 {
        s.id = self.next_id;
        self.next_id += 1;
        let id = s.id;
        self.shapes.push(s);
        id
    }
    /// Layer indices in output order, repaired so every layer appears exactly once.
    pub fn layer_order(&self) -> Vec<usize> {
        let n = self.layers.len();
        let mut seen = vec![false; n];
        let mut out: Vec<usize> = self.order.iter().copied().filter(|&i| i < n && !std::mem::replace(&mut seen[i], true)).collect();
        out.extend((0..n).filter(|&i| !seen[i]));
        out
    }
    /// Move a layer one place earlier (`up`) or later in the output order.
    pub fn move_layer(&mut self, layer: usize, up: bool) {
        let mut o = self.layer_order();
        if let Some(i) = o.iter().position(|&l| l == layer) {
            let j = if up { i.checked_sub(1) } else { (i + 1 < o.len()).then_some(i + 1) };
            if let Some(j) = j {
                o.swap(i, j);
            }
        }
        self.order = o;
    }
    pub fn new_group_id(&mut self) -> u64 {
        self.next_group = self.next_group.max(1);
        let g = self.next_group;
        self.next_group += 1;
        g
    }
    /// All shape ids in the same group as `id` (just `id` when ungrouped).
    pub fn group_of(&self, id: u64) -> Vec<u64> {
        match self.shape(id).and_then(|s| s.group) {
            Some(g) => self.shapes.iter().filter(|s| s.group == Some(g)).map(|s| s.id).collect(),
            None => vec![id],
        }
    }
    pub fn shape(&self, id: u64) -> Option<&Shape> {
        self.shapes.iter().find(|s| s.id == id)
    }
    pub fn shape_mut(&mut self, id: u64) -> Option<&mut Shape> {
        self.shapes.iter_mut().find(|s| s.id == id)
    }
    pub fn bounds_of(&self, ids: &[u64]) -> Option<Rect> {
        ids.iter().filter_map(|i| self.shape(*i)).filter_map(|s| s.bounds()).reduce(Rect::union)
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
    pub fn from_json(s: &str) -> Result<Document, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}
