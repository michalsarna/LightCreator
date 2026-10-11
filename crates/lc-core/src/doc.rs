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

    /// A short description of what the method does.
    pub fn hint(self) -> &'static str {
        match self {
            Dither::Threshold => "Black or white only, split at half brightness. Sharp: good for line art and text.",
            Dither::FloydSteinberg => "Spreads the rounding error to the neighbouring dots. A good all-round choice for photos.",
            Dither::Jarvis => "Spreads the error over a wider area: smoother tones, a little slower.",
            Dither::Stucki => "Like Jarvis, with crisper detail.",
            Dither::Atkinson => "Keeps highlights clean and contrast high; loses some of the darkest and lightest detail.",
            Dither::Ordered => "A fixed 8×8 pattern of dots: regular and fast, without noise.",
            Dither::Grayscale => "No dots: every pixel burns with a power between the minimum and the layer's power.",
        }
    }
}

fn d_min_power() -> f64 {
    0.0
}

/// Default speed of every layer in mm/min; the device profile keeps its own value for each layer.
pub const DEFAULT_LAYER_SPEED_MM_MIN: f64 = 1000.0;

fn d_layer_speeds() -> Vec<f64> {
    vec![DEFAULT_LAYER_SPEED_MM_MIN; 30]
}

/// Millimetres by which a flattened curve may differ from the true one, unless the device says otherwise.
pub const DEFAULT_FLATTEN_TOL: f64 = 0.05;
/// Default distance an arc may stray from the path it replaces.
pub const DEFAULT_ARC_TOL: f64 = 0.01;

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
    /// Switch the air assist (air pump) on while this layer burns.
    #[serde(default)]
    pub air_assist: bool,
    /// Constant (`M3`) or dynamic (`M4`) power for this layer; GRBL-like dialects only.
    #[serde(default)]
    pub power_mode: PowerMode,
    /// The laser head that burns this layer (an index into the device's heads; the first one if it is not there).
    #[serde(default)]
    pub head: usize,
}

/// How a layer's laser power follows the speed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PowerMode {
    /// Whatever the device is set to.
    #[default]
    Inherit,
    /// `M3`: the power stays what is set, also in corners and while accelerating.
    Constant,
    /// `M4`: GRBL scales the power with the speed (recommended for corners and short moves).
    Dynamic,
}

impl PowerMode {
    pub const ALL: [PowerMode; 3] = [PowerMode::Inherit, PowerMode::Constant, PowerMode::Dynamic];
    pub fn label(self) -> &'static str {
        match self {
            PowerMode::Inherit => "Like the device",
            PowerMode::Constant => "Constant (M3)",
            PowerMode::Dynamic => "Dynamic (M4)",
        }
    }
}

impl Layer {
    pub fn new(color: usize) -> Layer {
        Layer {
            name: format!("C{:02}", color),
            color,
            mode: LayerMode::Line,
            speed: DEFAULT_LAYER_SPEED_MM_MIN / 60.0,
            power: 100.0,
            passes: 1,
            interval: 0.1,
            angle: 0.0,
            overscan: 1.0,
            bidirectional: true,
            output: true,
            visible: true,
            dither: Dither::default(),
            min_power: 0.0,
            air_assist: false,
            power_mode: PowerMode::Inherit,
            head: 0,
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
    /// A locked shape cannot be moved, edited or deleted; it can still be selected and copied.
    #[serde(default)]
    pub locked: bool,
}

impl Shape {
    pub fn local_polys(&self) -> Vec<Polyline> {
        self.local_polys_tol(DEFAULT_FLATTEN_TOL)
    }
    /// The outlines with curves flattened to within `tol` millimetres.
    pub fn local_polys_tol(&self, tol: f64) -> Vec<Polyline> {
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
            Kind::Bezier(cs) => cs.iter().map(|c| c.flatten(tol)).collect(),
            Kind::Image(im) => vec![Polyline::new(vec![Pt::new(0.0, 0.0), Pt::new(im.w, 0.0), Pt::new(im.w, im.h), Pt::new(0.0, im.h)], true)],
            Kind::Text(t) => crate::text::contours(t).iter().map(|c| c.flatten(tol)).collect(),
        }
    }
    pub fn polys(&self) -> Vec<Polyline> {
        self.polys_tol(DEFAULT_FLATTEN_TOL)
    }
    pub fn polys_tol(&self, tol: f64) -> Vec<Polyline> {
        self.local_polys_tol(tol).iter().map(|p| p.transformed(&self.xf)).collect()
    }
    pub fn bounds(&self) -> Option<Rect> {
        self.polys().iter().filter_map(|p| p.bounds()).reduce(Rect::union)
    }
    /// Does the shape touch the rectangle (crossing selection)?
    pub fn intersects_rect(&self, r: &Rect) -> bool {
        self.polys().iter().any(|p| p.intersects_rect(r))
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

/// The corner of the work area where the machine's zero is. "Front" is the bottom of the design (y down is the back).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Origin {
    /// Machine zero at the front-left corner (GRBL default); Y is flipped on output.
    FrontLeft,
    /// Machine zero at the back-left corner.
    BackLeft,
    FrontRight,
    BackRight,
}

impl Origin {
    pub const ALL: [Origin; 4] = [Origin::FrontLeft, Origin::BackLeft, Origin::FrontRight, Origin::BackRight];
    pub fn label(self) -> &'static str {
        match self {
            Origin::FrontLeft => "Front-left (GRBL default)",
            Origin::BackLeft => "Back-left",
            Origin::FrontRight => "Front-right",
            Origin::BackRight => "Back-right",
        }
    }
    fn right(self) -> bool {
        matches!(self, Origin::FrontRight | Origin::BackRight)
    }
    fn front(self) -> bool {
        matches!(self, Origin::FrontLeft | Origin::FrontRight)
    }
}

/// How the machine's panel sits in front of the user: the work area is shown turned the same way.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Orientation {
    #[default]
    Native,
    /// The panel is turned a quarter turn counter-clockwise.
    RotateLeft,
    /// The panel is turned a quarter turn clockwise.
    RotateRight,
}

impl Orientation {
    pub const ALL: [Orientation; 3] = [Orientation::Native, Orientation::RotateLeft, Orientation::RotateRight];
    pub fn label(self) -> &'static str {
        match self {
            Orientation::Native => "Native",
            Orientation::RotateLeft => "Rotated left",
            Orientation::RotateRight => "Rotated right",
        }
    }
}

/// Time basis in which speeds are shown and entered. Stored speeds stay in mm/s (layers) and mm/min (travel, jog)
/// and the G-code always carries mm/min after `G21`, whatever is chosen here.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpeedUnit {
    #[default]
    PerSecond,
    PerMinute,
}

impl SpeedUnit {
    pub const ALL: [SpeedUnit; 2] = [SpeedUnit::PerSecond, SpeedUnit::PerMinute];
    /// English label with the units of length filled in (translated in the UI).
    pub fn label(self) -> &'static str {
        match self {
            SpeedUnit::PerSecond => "Per second (mm/s, in/s)",
            SpeedUnit::PerMinute => "Per minute (mm/min, in/min)",
        }
    }
    /// Suffix for a speed, with a leading space.
    pub fn suffix(self, u: Units) -> &'static str {
        match (u, self) {
            (Units::Mm, SpeedUnit::PerSecond) => " mm/s",
            (Units::Mm, SpeedUnit::PerMinute) => " mm/min",
            (Units::Inch, SpeedUnit::PerSecond) => " in/s",
            (Units::Inch, SpeedUnit::PerMinute) => " in/min",
        }
    }
    /// How many of the stored units (mm per second, or mm per minute when `stored_per_min`) make one displayed unit.
    pub fn factor(self, u: Units, stored_per_min: bool) -> f64 {
        let time = match (self, stored_per_min) {
            (SpeedUnit::PerSecond, false) | (SpeedUnit::PerMinute, true) => 1.0,
            (SpeedUnit::PerSecond, true) => 60.0,
            (SpeedUnit::PerMinute, false) => 1.0 / 60.0,
        };
        u.to_mm(1.0) * time
    }
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
    /// Smoothieware firmware over a serial port (streamed live like GRBL).
    Smoothieware,
}

impl Controller {
    pub const ALL: [Controller; 5] = [Controller::Grbl, Controller::Marlin, Controller::Smoothieware, Controller::Ruida, Controller::Trocen];
    pub fn label(self) -> &'static str {
        match self {
            Controller::Grbl => "GRBL",
            Controller::Marlin => "Marlin",
            Controller::Ruida => "Ruida",
            Controller::Trocen => "Trocen",
            Controller::Smoothieware => "Smoothieware",
        }
    }
    /// Can LightCreator talk to this controller over a serial port?
    pub fn is_serial(self) -> bool {
        matches!(self, Controller::Grbl | Controller::Marlin | Controller::Smoothieware)
    }
}

/// Laser source type; selects matching entries in the material library.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LaserKind {
    #[default]
    Diode,
    Co2,
    /// Fibre laser (marking and engraving metals).
    Fiber,
}

impl LaserKind {
    pub const ALL: [LaserKind; 3] = [LaserKind::Diode, LaserKind::Co2, LaserKind::Fiber];
    pub fn label(self) -> &'static str {
        match self {
            LaserKind::Diode => "Diode",
            LaserKind::Co2 => "CO2",
            LaserKind::Fiber => "Fiber",
        }
    }
    /// The usual wavelength of this kind of laser, nanometres.
    pub fn default_wavelength(self) -> u32 {
        match self {
            LaserKind::Diode => 445,
            LaserKind::Co2 => 10_600,
            LaserKind::Fiber => 1064,
        }
    }
    /// A typical optical power, watts (the libraries assume these for diode and CO2).
    pub fn default_power(self) -> f64 {
        match self {
            LaserKind::Diode => 10.0,
            LaserKind::Co2 => 40.0,
            LaserKind::Fiber => 20.0,
        }
    }
    /// Does this kind of laser have settings of a pulse width modulation (CO2 tubes and fibre sources do)?
    pub fn has_pwm(self) -> bool {
        !matches!(self, LaserKind::Diode)
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

/// Machine axes: where the zero is.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Axes {
    pub origin: Origin,
    /// X counts the other way (the machine's X axis is inverted).
    #[serde(default)]
    pub flip_x: bool,
    /// Y counts the other way.
    #[serde(default)]
    pub flip_y: bool,
    #[serde(default)]
    pub orientation: Orientation,
}

impl Default for Axes {
    fn default() -> Self {
        Axes { origin: Origin::FrontLeft, flip_x: false, flip_y: false, orientation: Orientation::Native }
    }
}

/// Strips along the edges of the work area that the machine cannot use (clamps, a frame), in design millimetres.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Margins {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

/// An own rectangle the machine may not leave, in design millimetres (a software limit).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoftLimits {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

/// The work area of the machine, millimetres.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Area {
    pub bed_w: f64,
    pub bed_h: f64,
    #[serde(default)]
    pub margins: Margins,
    /// Optional own limits; they are kept inside the usable area.
    #[serde(default)]
    pub soft_limits: Option<SoftLimits>,
}

impl Default for Area {
    fn default() -> Self {
        Area { bed_w: 400.0, bed_h: 400.0, margins: Margins::default(), soft_limits: None }
    }
}

/// Machine speeds.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Speeds {
    /// Speed of moves with the laser off, mm/min.
    pub travel_speed: f64,
    /// The fastest the machine can cut, mm/min; 0 = no limit.
    #[serde(default)]
    pub max_cut_speed: f64,
    /// Acceleration of the machine, mm/s²; 0 = not known (the time estimate ignores ramps).
    #[serde(default)]
    pub acceleration: f64,
}

impl Default for Speeds {
    fn default() -> Self {
        Speeds { travel_speed: 3000.0, max_cut_speed: 0.0, acceleration: 0.0 }
    }
}

/// The flavour of G-code a job is written in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dialect {
    /// GRBL with constant power (`M3`).
    Grbl,
    /// GRBL with dynamic power (`M4`, recommended).
    GrblDynamic,
    /// Marlin with the laser feature (inline power).
    Marlin,
    /// Smoothieware (laser module; S is a fraction of the full power).
    Smoothieware,
}

impl Dialect {
    pub const ALL: [Dialect; 4] = [Dialect::Grbl, Dialect::GrblDynamic, Dialect::Marlin, Dialect::Smoothieware];
    pub fn label(self) -> &'static str {
        match self {
            Dialect::Grbl => "GRBL (M3)",
            Dialect::GrblDynamic => "GRBL dynamic (M4)",
            Dialect::Marlin => "Marlin",
            Dialect::Smoothieware => "Smoothieware",
        }
    }
    /// The GRBL-like flavours switch between `M3` and `M4`.
    pub fn is_grbl(self) -> bool {
        matches!(self, Dialect::Grbl | Dialect::GrblDynamic)
    }
}

/// How the G-code of a job is written.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GcodeCfg {
    /// Use `M4` dynamic power (recommended) instead of `M3`.
    pub dynamic_power: bool,
    /// Go back to the machine zero when the job is done.
    pub return_home: bool,
    /// The flavour of G-code; `None` follows the controller (and `dynamic_power` for GRBL).
    #[serde(default)]
    pub dialect: Option<Dialect>,
    /// Lines that replace the start of the job (units, absolute mode, plane and the laser mode); empty = built in.
    #[serde(default)]
    pub preamble: String,
    /// Lines that replace the end of the program (`M2` for GRBL); empty = built in. The laser is always switched
    /// off and the head sent home (when set) before these.
    #[serde(default)]
    pub postscript: String,
    /// Command that switches the air assist on / off; empty = built in (`M8` / `M9`, `M106 S255` / `M107`).
    #[serde(default)]
    pub air_on: String,
    #[serde(default)]
    pub air_off: String,
    /// Write curves as arcs (`G2` / `G3`) where they fit. Off by default: not every firmware does arcs well.
    #[serde(default)]
    pub arcs: bool,
    /// How far an arc may stray from the path it replaces, millimetres.
    #[serde(default = "d_arc_tol")]
    pub arc_tolerance: f64,
    /// How far a flattened curve (Bézier, text) may stray from the true curve, millimetres.
    #[serde(default = "d_flatten_tol")]
    pub flatten_tolerance: f64,
    /// Own G-code at the start and end of layers and objects.
    #[serde(default)]
    pub hooks: Hooks,
}

fn d_arc_tol() -> f64 {
    DEFAULT_ARC_TOL
}
fn d_flatten_tol() -> f64 {
    DEFAULT_FLATTEN_TOL
}

impl Default for GcodeCfg {
    fn default() -> Self {
        GcodeCfg {
            dynamic_power: true,
            return_home: true,
            dialect: None,
            preamble: String::new(),
            postscript: String::new(),
            air_on: String::new(),
            air_off: String::new(),
            arcs: false,
            arc_tolerance: DEFAULT_ARC_TOL,
            flatten_tolerance: DEFAULT_FLATTEN_TOL,
            hooks: Hooks::default(),
        }
    }
}

/// A piece of own G-code run at a set point of a job.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Hook {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub code: String,
}

impl Hook {
    /// The lines to write, or none when the hook is off, empty or not fit to be used.
    pub fn lines(&self) -> Vec<&str> {
        if !self.enabled {
            return vec![];
        }
        GcodeCfg::custom(&self.code).unwrap_or_default().into_iter().filter(|l| !l.trim().is_empty()).collect()
    }
}

/// The four places where a job runs own G-code, in the order they come: a layer starts, an object starts and ends,
/// the layer ends.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Hooks {
    #[serde(default)]
    pub layer_start: Hook,
    #[serde(default)]
    pub layer_end: Hook,
    #[serde(default)]
    pub object_start: Hook,
    #[serde(default)]
    pub object_end: Hook,
}

impl Hooks {
    pub const NAMES: [&'static str; 4] = ["layer_start", "layer_end", "object_start", "object_end"];

    pub fn get(&self, i: usize) -> &Hook {
        [&self.layer_start, &self.layer_end, &self.object_start, &self.object_end][i]
    }

    pub fn get_mut(&mut self, i: usize) -> &mut Hook {
        [&mut self.layer_start, &mut self.layer_end, &mut self.object_start, &mut self.object_end].into_iter().nth(i).unwrap()
    }

    /// Do the objects have hooks? Then a layer is burnt object by object.
    pub fn per_object(&self) -> bool {
        !self.object_start.lines().is_empty() || !self.object_end.lines().is_empty()
    }

    /// The first thing wrong with a hook (text not fit for G-code, an unknown variable), as `(hook, reason)`.
    pub fn problem(&self) -> Option<(&'static str, String)> {
        (0..4).find_map(|i| {
            let h = self.get(i);
            if !snippet_ok(&h.code) {
                return Some((Self::NAMES[i], "a control character or too long".to_string()));
            }
            crate::hooks::check_variables(&h.code, crate::hooks::JOB_VARS).err().map(|e| (Self::NAMES[i], e))
        })
    }
}

/// Longest text and line, and most lines, of an own piece of G-code.
pub const SNIPPET_MAX_LEN: usize = 2000;
pub const SNIPPET_MAX_LINE: usize = 100;
pub const SNIPPET_MAX_LINES: usize = 60;

/// Is `text` fit to be written into a job? Blank lines and comments are fine; control characters and absurd sizes
/// are not.
pub fn snippet_ok(text: &str) -> bool {
    text.len() <= SNIPPET_MAX_LEN
        && text.lines().count() <= SNIPPET_MAX_LINES
        && text.lines().all(|l| l.chars().count() <= SNIPPET_MAX_LINE)
        && !text.chars().any(|c| c.is_control() && c != '\n')
}

impl GcodeCfg {
    /// The names of the own G-code fields that are not fit to be used.
    pub fn invalid_fields(&self) -> Vec<&'static str> {
        [("preamble", &self.preamble), ("postscript", &self.postscript), ("air on", &self.air_on), ("air off", &self.air_off)]
            .into_iter()
            .filter(|(_, t)| !snippet_ok(t))
            .map(|(n, _)| n)
            .collect()
    }

    /// Back to the built-in start, end and air commands.
    pub fn reset_custom(&mut self) {
        self.preamble.clear();
        self.postscript.clear();
        self.air_on.clear();
        self.air_off.clear();
        self.hooks = Hooks::default();
    }

    /// The lines of an own piece of G-code, or `None` when it is empty or not fit to be used.
    pub fn custom<'a>(text: &'a str) -> Option<Vec<&'a str>> {
        let lines: Vec<&str> = text.lines().map(str::trim_end).collect();
        (snippet_ok(text) && lines.iter().any(|l| !l.trim().is_empty())).then_some(lines)
    }
}

/// How a job is fed to the controller.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Streaming {
    /// Several lines are in the controller's buffer at once; the program counts the bytes (fast, the default).
    #[default]
    Counting,
    /// One line at a time: send, wait for `ok`, send the next. Slow, but for firmware that answers wrongly.
    Simple,
}

impl Streaming {
    pub const ALL: [Streaming; 2] = [Streaming::Counting, Streaming::Simple];
    pub fn label(self) -> &'static str {
        match self {
            Streaming::Counting => "Count the buffer",
            Streaming::Simple => "Simple (wait for each ok)",
        }
    }
}

/// How the program reaches the machine.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Connection {
    /// Serial port or TCP socket.
    #[serde(default)]
    pub link: LinkKind,
    /// TCP host (name or address) when `link` is `Tcp`.
    #[serde(default)]
    pub host: String,
    #[serde(default = "d_tcp_port")]
    pub tcp_port: u16,
    /// Serial port used for this device (empty = none chosen yet).
    #[serde(default)]
    pub port: String,
    pub baud: u32,
    #[serde(default)]
    pub streaming: Streaming,
    /// USB vendor and product id of the controller (`0403:6001`); when set the port is found by it, even after its
    /// name changed (empty = bound to the port name only).
    #[serde(default)]
    pub usb_id: String,
    /// Connect again by itself, every few seconds, when the link was lost without the user asking for it.
    #[serde(default)]
    pub auto_reconnect: bool,
    /// Run the homing cycle (`$H`, `G28`) once the link is up.
    #[serde(default)]
    pub home_on_connect: bool,
    /// Clear an alarm (`$X`, `M999`) once the link is up. This skips homing: only for machines whose head is known
    /// to be in a safe place.
    #[serde(default)]
    pub unlock_on_connect: bool,
}

impl Default for Connection {
    fn default() -> Self {
        Connection { link: LinkKind::Serial, host: String::new(), tcp_port: 3333, port: String::new(), baud: 115_200, streaming: Streaming::Counting, usb_id: String::new(), auto_reconnect: false, home_on_connect: false, unlock_on_connect: false }
    }
}

/// `0403:6001` (hexadecimal vendor and product id) as numbers.
pub fn parse_usb_id(text: &str) -> Option<(u16, u16)> {
    let (v, p) = text.trim().split_once(':')?;
    let hex = |s: &str| (!s.is_empty() && s.len() <= 4 && s.chars().all(|c| c.is_ascii_hexdigit())).then(|| u16::from_str_radix(s, 16).ok()).flatten();
    Some((hex(v.trim())?, hex(p.trim())?))
}

/// One laser head of the machine.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LaserHead {
    #[serde(default)]
    pub name: String,
    /// The tool number (`T<n>`) the controller knows this head by; unique among the heads.
    #[serde(default)]
    pub tool: u32,
    /// GRBL `$30`, the S value that equals 100% power.
    pub s_max: f64,
    /// Laser power (percent) while framing; 0 keeps the laser off.
    #[serde(default)]
    pub frame_power: f64,
    /// Speed of the frame, mm/s.
    #[serde(default = "d_frame_speed")]
    pub frame_speed: f64,
    /// Power (percent) of the focus beam, a weak steady spot to set the height by; 0 = no focus mode.
    #[serde(default)]
    pub focus_power: f64,
    /// Size of the beam on the material, millimetres (0 = not known).
    #[serde(default)]
    pub spot_x: f64,
    #[serde(default)]
    pub spot_y: f64,
    /// The colour of this head in the program.
    #[serde(default = "d_head_colour")]
    pub colour: [u8; 3],
    #[serde(default)]
    pub laser: LaserKind,
    /// Wavelength in nanometres; 0 = the usual one of the laser type.
    #[serde(default)]
    pub wavelength_nm: u32,
    /// Optical power in watts; 0 = a typical one of the laser type.
    #[serde(default)]
    pub optical_power_w: f64,
    /// Pulse width modulation of CO2 and fibre sources. These are kept and offered as variables for macros; they
    /// are not sent to the controller.
    #[serde(default)]
    pub pwm_freq_hz: f64,
    #[serde(default)]
    pub pwm_max_freq_hz: f64,
    #[serde(default)]
    pub pulse_width_us: f64,
    #[serde(default)]
    pub pulse_min_us: f64,
    #[serde(default)]
    pub pulse_max_us: f64,
    /// Where the red pointer dot is relative to the beam, millimetres in the design's directions (x right, y down).
    #[serde(default)]
    pub pointer_dx: f64,
    #[serde(default)]
    pub pointer_dy: f64,
}

fn d_frame_speed() -> f64 {
    40.0
}
fn d_head_colour() -> [u8; 3] {
    [0xff, 0x7a, 0x1a]
}

impl Default for LaserHead {
    fn default() -> Self {
        LaserHead {
            name: String::new(),
            tool: 0,
            s_max: 1000.0,
            frame_power: 0.0,
            frame_speed: 40.0,
            focus_power: 0.0,
            spot_x: 0.0,
            spot_y: 0.0,
            colour: d_head_colour(),
            laser: LaserKind::Diode,
            wavelength_nm: 0,
            optical_power_w: 0.0,
            pwm_freq_hz: 0.0,
            pwm_max_freq_hz: 0.0,
            pulse_width_us: 0.0,
            pulse_min_us: 0.0,
            pulse_max_us: 0.0,
            pointer_dx: 0.0,
            pointer_dy: 0.0,
        }
    }
}

impl LaserHead {
    /// The wavelength, nanometres: the set one or the usual one of the laser type.
    pub fn wavelength(&self) -> u32 {
        if self.wavelength_nm > 0 { self.wavelength_nm } else { self.laser.default_wavelength() }
    }
    /// The optical power, watts: the set one or a typical one of the laser type.
    pub fn power_w(&self) -> f64 {
        if self.optical_power_w > 0.0 { self.optical_power_w } else { self.laser.default_power() }
    }
    /// Does the pointer sit away from the beam?
    pub fn has_pointer_offset(&self) -> bool {
        self.pointer_dx.abs() > 1e-9 || self.pointer_dy.abs() > 1e-9
    }
}

/// A device (machine) profile: everything the Device tab shows and the job generators need.
///
/// Profiles written by earlier versions keep the settings of the groups as flat fields (`bed_w`, `origin`,
/// `s_max`, ...); they are read into the groups, the laser head fields into `heads[0]`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(from = "DeviceWire")]
pub struct Device {
    pub name: String,
    /// Display units; profiles saved before this existed load as millimetres.
    pub units: Units,
    /// Whether speeds are shown per second or per minute.
    pub speed_unit: SpeedUnit,
    pub controller: Controller,
    pub axes: Axes,
    pub area: Area,
    pub speeds: Speeds,
    pub gcode: GcodeCfg,
    pub connection: Connection,
    /// The laser heads; there is always at least one.
    pub heads: Vec<LaserHead>,
    /// Jog step in mm.
    pub jog_step: f64,
    /// Jog feed in mm/min.
    pub jog_feed: f64,
    /// Camera overlay alignment for this machine.
    pub camera: Option<CameraCfg>,
    /// URL of a live picture of the machine (MJPEG stream or JPEG snapshot). Empty = none.
    pub camera_url: String,
    /// Quarter turns clockwise applied to the camera picture (0..=3).
    pub camera_rotation: u8,
    /// Default speed of each of the 30 layers in mm/min, kept with the device and used for new documents.
    pub layer_speeds: Vec<f64>,
    /// The material library of this device: materials with the settings of all their layers. It is stored with
    /// the profile and with saved projects.
    pub materials: Vec<crate::materials::Material>,
    /// The library has been filled from the built-in one (once; deleting materials does not refill it).
    pub materials_seeded: bool,
    /// The library follows the main library: the materials above are only this device's own copies of changed ones.
    pub materials_inherit: bool,
    /// Own G-code scripts that are run by hand from the Device tab.
    pub macros: Vec<Macro>,
    /// Parts of the work area the machine must not enter, in design coordinates.
    pub zones: Vec<Zone>,
}

/// The shape of a no-go zone, in design millimetres.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ZoneShape {
    /// A rectangle: top-left corner and size.
    Rect { x: f64, y: f64, w: f64, h: f64 },
    /// A circle: centre and radius.
    Circle { x: f64, y: f64, r: f64 },
}

/// A no-go zone of a device.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Zone {
    pub name: String,
    #[serde(default = "d_true")]
    pub enabled: bool,
    pub shape: ZoneShape,
}

fn d_true() -> bool {
    true
}

/// A named script of own G-code, run by hand.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Macro {
    pub name: String,
    pub code: String,
}

/// What a device profile looks like in a file: the groups, or the flat fields of older versions.
#[derive(Deserialize)]
struct DeviceWire {
    name: String,
    #[serde(default)]
    units: Units,
    #[serde(default)]
    speed_unit: SpeedUnit,
    #[serde(default)]
    controller: Controller,
    axes: Option<Axes>,
    area: Option<Area>,
    speeds: Option<Speeds>,
    gcode: Option<GcodeCfg>,
    connection: Option<Connection>,
    heads: Option<Vec<LaserHead>>,
    #[serde(default = "d_jog_step")]
    jog_step: f64,
    #[serde(default = "d_jog_feed")]
    jog_feed: f64,
    #[serde(default)]
    camera: Option<CameraCfg>,
    #[serde(default)]
    camera_url: String,
    #[serde(default)]
    camera_rotation: u8,
    #[serde(default = "d_layer_speeds")]
    layer_speeds: Vec<f64>,
    #[serde(default)]
    materials: Vec<crate::materials::Material>,
    #[serde(default)]
    materials_seeded: bool,
    #[serde(default)]
    materials_inherit: bool,
    #[serde(default)]
    macros: Vec<Macro>,
    #[serde(default)]
    zones: Vec<Zone>,
    // Flat fields of profiles saved before the groups existed.
    origin: Option<Origin>,
    bed_w: Option<f64>,
    bed_h: Option<f64>,
    travel_speed: Option<f64>,
    dynamic_power: Option<bool>,
    return_home: Option<bool>,
    link: Option<LinkKind>,
    host: Option<String>,
    tcp_port: Option<u16>,
    port: Option<String>,
    baud: Option<u32>,
    s_max: Option<f64>,
    frame_power: Option<f64>,
    laser: Option<LaserKind>,
}

impl From<DeviceWire> for Device {
    fn from(w: DeviceWire) -> Device {
        let d = Device::default();
        let axes = w.axes.unwrap_or(Axes { origin: w.origin.unwrap_or(d.axes.origin), ..Axes::default() });
        let area = w.area.unwrap_or(Area { bed_w: w.bed_w.unwrap_or(d.area.bed_w), bed_h: w.bed_h.unwrap_or(d.area.bed_h), ..Area::default() });
        let speeds = w.speeds.unwrap_or(Speeds { travel_speed: w.travel_speed.unwrap_or(d.speeds.travel_speed), ..Speeds::default() });
        let gcode = w.gcode.unwrap_or(GcodeCfg { dynamic_power: w.dynamic_power.unwrap_or(true), return_home: w.return_home.unwrap_or(true), ..GcodeCfg::default() });
        let connection = w.connection.unwrap_or(Connection {
            link: w.link.unwrap_or_default(),
            host: w.host.unwrap_or_default(),
            tcp_port: w.tcp_port.unwrap_or(3333),
            port: w.port.unwrap_or_default(),
            baud: w.baud.unwrap_or(115_200),
            streaming: Streaming::Counting,
            usb_id: String::new(),
            auto_reconnect: false,
            home_on_connect: false,
            unlock_on_connect: false,
        });
        let mut heads = w.heads.unwrap_or_default();
        if heads.is_empty() {
            heads.push(LaserHead { s_max: w.s_max.unwrap_or(1000.0), frame_power: w.frame_power.unwrap_or(0.0), laser: w.laser.unwrap_or_default(), ..LaserHead::default() });
        }
        Device {
            name: w.name,
            units: w.units,
            speed_unit: w.speed_unit,
            controller: w.controller,
            axes,
            area,
            speeds,
            gcode,
            connection,
            heads,
            jog_step: w.jog_step,
            jog_feed: w.jog_feed,
            camera: w.camera,
            camera_url: w.camera_url,
            camera_rotation: w.camera_rotation,
            layer_speeds: w.layer_speeds,
            materials: w.materials,
            materials_seeded: w.materials_seeded,
            materials_inherit: w.materials_inherit,
            macros: w.macros,
            zones: w.zones,
        }
    }
}

impl Device {
    /// Default speed of layer `i` in mm/s.
    pub fn layer_speed_mm_s(&self, i: usize) -> f64 {
        self.layer_speeds.get(i).copied().filter(|v| *v > 0.0).unwrap_or(DEFAULT_LAYER_SPEED_MM_MIN) / 60.0
    }
    /// Remember `mm_s` as the default speed of layer `i`.
    pub fn set_layer_speed_mm_s(&mut self, i: usize, mm_s: f64) {
        if self.layer_speeds.len() < 30 {
            self.layer_speeds.resize(30, DEFAULT_LAYER_SPEED_MM_MIN);
        }
        if let Some(v) = self.layer_speeds.get_mut(i) {
            *v = mm_s * 60.0;
        }
    }

    /// Size of the work area as the design sees it (millimetres): the axis ranges, swapped when the panel is turned.
    pub fn work_size(&self) -> (f64, f64) {
        match self.axes.orientation {
            Orientation::Native => (self.area.bed_w, self.area.bed_h),
            Orientation::RotateLeft | Orientation::RotateRight => (self.area.bed_h, self.area.bed_w),
        }
    }

    /// Braking distance (mm) from `speed` (mm/s) with the machine's acceleration: a sensible overscan. `None` when
    /// the acceleration is not known.
    pub fn suggested_overscan(&self, speed: f64) -> Option<f64> {
        let a = self.speeds.acceleration;
        (a > 0.0).then(|| speed * speed / (2.0 * a))
    }

    /// The usable area in design coordinates: the work area minus the margins (never negative in size).
    pub fn usable_rect(&self) -> Rect {
        let (w, h) = self.work_size();
        let m = self.area.margins;
        let x0 = m.left.clamp(0.0, w);
        let y0 = m.top.clamp(0.0, h);
        Rect { min: Pt::new(x0, y0), max: Pt::new((w - m.right).clamp(x0, w), (h - m.bottom).clamp(y0, h)) }
    }

    /// The area jobs and moves may use: the usable area, cut down to the software limits when there are some.
    pub fn allowed_rect(&self) -> Rect {
        let u = self.usable_rect();
        let Some(l) = self.area.soft_limits else { return u };
        let (x0, x1) = (l.x0.min(l.x1).max(u.min.x), l.x0.max(l.x1).min(u.max.x));
        let (y0, y1) = (l.y0.min(l.y1).max(u.min.y), l.y0.max(l.y1).min(u.max.y));
        Rect { min: Pt::new(x0, y0), max: Pt::new(x1.max(x0), y1.max(y0)) }
    }

    /// A move of `(dx, dy)` in the design's directions as the change of the machine's X and Y.
    pub fn machine_delta(&self, dx: f64, dy: f64) -> (f64, f64) {
        let a = self.to_machine(Pt::new(0.0, 0.0));
        let b = self.to_machine(Pt::new(dx, dy));
        (b.0 - a.0, b.1 - a.1)
    }

    /// A jog of `(dx, dy)` in the design's directions, shortened so the head stays inside the allowed area. `at` is the
    /// machine's position when it is known. Returns the change of the machine's X and Y (zero when there is no room).
    pub fn limited_jog(&self, at: Option<(f64, f64)>, dx: f64, dy: f64) -> (f64, f64) {
        let (mut dx, mut dy) = (dx, dy);
        if let Some((mx, my)) = at.filter(|(x, y)| x.is_finite() && y.is_finite()) {
            let p = self.from_machine(mx, my);
            let r = self.allowed_rect();
            let t = Pt::new((p.x + dx).clamp(r.min.x, r.max.x), (p.y + dy).clamp(r.min.y, r.max.y));
            // A head that stands outside (never homed, or limits changed) may always move back in.
            dx = if p.x < r.min.x - 1e-6 || p.x > r.max.x + 1e-6 { if (dx > 0.0) == (p.x < r.min.x) { dx } else { 0.0 } } else { t.x - p.x };
            dy = if p.y < r.min.y - 1e-6 || p.y > r.max.y + 1e-6 { if (dy > 0.0) == (p.y < r.min.y) { dy } else { 0.0 } } else { t.y - p.y };
        }
        if dx.abs() < 1e-9 && dy.abs() < 1e-9 {
            return (0.0, 0.0);
        }
        self.machine_delta(dx, dy)
    }

    /// A design point (millimetres, y down, within `work_size`) as the machine's X and Y coordinate.
    pub fn to_machine(&self, p: Pt) -> (f64, f64) {
        let (w, h) = (self.area.bed_w, self.area.bed_h);
        // Back to the machine's own view of its work area.
        let q = match self.axes.orientation {
            Orientation::Native => p,
            Orientation::RotateLeft => Pt::new(w - p.y, p.x),
            Orientation::RotateRight => Pt::new(p.y, h - p.x),
        };
        let mut x = if self.axes.origin.right() { w - q.x } else { q.x };
        let mut y = if self.axes.origin.front() { h - q.y } else { q.y };
        if self.axes.flip_x {
            x = w - x;
        }
        if self.axes.flip_y {
            y = h - y;
        }
        (x, y)
    }

    /// The design point of a machine coordinate; the inverse of `to_machine`.
    pub fn from_machine(&self, x: f64, y: f64) -> Pt {
        let (w, h) = (self.area.bed_w, self.area.bed_h);
        let x = if self.axes.flip_x { w - x } else { x };
        let y = if self.axes.flip_y { h - y } else { y };
        let q = Pt::new(if self.axes.origin.right() { w - x } else { x }, if self.axes.origin.front() { h - y } else { y });
        match self.axes.orientation {
            Orientation::Native => q,
            Orientation::RotateLeft => Pt::new(q.y, w - q.x),
            Orientation::RotateRight => Pt::new(h - q.y, q.x),
        }
    }

    /// Where the machine's zero is, in design coordinates (millimetres, y down). Jobs start and end here.
    pub fn home_point(&self) -> Pt {
        self.from_machine(0.0, 0.0)
    }

    /// The flavour of G-code this device gets.
    pub fn dialect(&self) -> Dialect {
        if let Some(d) = self.gcode.dialect {
            return d;
        }
        match self.controller {
            Controller::Marlin => Dialect::Marlin,
            Controller::Smoothieware => Dialect::Smoothieware,
            _ if self.gcode.dynamic_power => Dialect::GrblDynamic,
            _ => Dialect::Grbl,
        }
    }

    /// Laser head `i`, or the first one when there is no such head.
    pub fn head(&self, i: usize) -> &LaserHead {
        self.heads.get(i).unwrap_or(&self.heads[0])
    }

    pub fn head_mut(&mut self, i: usize) -> &mut LaserHead {
        let i = if i < self.heads.len() { i } else { 0 };
        &mut self.heads[i]
    }

    /// Do two heads share a tool number? (Only matters with more than one head.)
    pub fn has_duplicate_tools(&self) -> bool {
        self.heads.iter().enumerate().any(|(i, h)| self.heads[..i].iter().any(|o| o.tool == h.tool))
    }

    /// The laser type of the first head (the one the material library follows).
    pub fn laser(&self) -> LaserKind {
        self.heads[0].laser
    }

    /// `$30` of the first head: the S value that equals 100 % power.
    pub fn s_max(&self) -> f64 {
        self.heads[0].s_max
    }

    /// Short description of the connection target.
    pub fn target_label(&self) -> String {
        match self.connection.link {
            LinkKind::Serial => self.connection.port.clone(),
            LinkKind::Tcp => format!("{}:{}", self.connection.host, self.connection.tcp_port),
        }
    }

    /// Is a connection target configured?
    pub fn has_target(&self) -> bool {
        match self.connection.link {
            LinkKind::Serial => !self.connection.port.is_empty(),
            LinkKind::Tcp => !self.connection.host.trim().is_empty(),
        }
    }
}

impl Default for Device {
    fn default() -> Self {
        Device {
            name: "GRBL 1.1 (diode / CO2)".into(),
            units: Units::Mm,
            speed_unit: SpeedUnit::default(),
            controller: Controller::Grbl,
            axes: Axes::default(),
            area: Area::default(),
            speeds: Speeds::default(),
            gcode: GcodeCfg::default(),
            connection: Connection::default(),
            heads: vec![LaserHead::default()],
            jog_step: 5.0,
            jog_feed: 3000.0,
            camera: None,
            camera_url: String::new(),
            camera_rotation: 0,
            layer_speeds: d_layer_speeds(),
            materials: vec![],
            materials_seeded: false,
            materials_inherit: false,
            macros: vec![],
            zones: vec![],
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
    /// The material whose settings were put into the layers of this design (it travels with the file, so the
    /// design can be repeated on another machine).
    #[serde(default)]
    pub material: Option<crate::materials::Material>,
    /// What hooks may print about the job; set by the program before the G-code is made, never saved.
    #[serde(skip)]
    pub job_info: JobInfo,
}

/// The name of the job and the moment it is made, for the `{job_name}`, `{date}` and `{time}` of hooks.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct JobInfo {
    pub name: String,
    pub date: String,
    pub time: String,
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
            material: None,
            job_info: JobInfo::default(),
        }
    }
}

impl Document {
    pub fn add(&mut self, layer: usize, kind: Kind, xf: Xf) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.shapes.push(Shape { id, layer, kind, xf, group: None, locked: false });
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
    /// Mutable access for editing: `None` when the shape does not exist or is locked.
    pub fn unlocked_mut(&mut self, id: u64) -> Option<&mut Shape> {
        self.shape_mut(id).filter(|s| !s.locked)
    }
    /// The ids in `ids` that may be edited.
    pub fn unlocked_ids(&self, ids: &[u64]) -> Vec<u64> {
        ids.iter().copied().filter(|id| self.shape(*id).is_some_and(|s| !s.locked)).collect()
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
