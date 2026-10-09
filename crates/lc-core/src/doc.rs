//! Document model: shapes, layers (LightBurn "cuts"), and machine settings.
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
    /// Raster-scan the inside of closed shapes.
    Fill,
    /// Fill the interior, then cut the outline.
    FillAndLine,
}

impl LayerMode {
    pub const ALL: [LayerMode; 3] = [LayerMode::Line, LayerMode::Fill, LayerMode::FillAndLine];
    pub fn label(self) -> &'static str {
        match self {
            LayerMode::Line => "Line",
            LayerMode::Fill => "Fill",
            LayerMode::FillAndLine => "Fill + Line",
        }
    }
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
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Kind {
    Rect { w: f64, h: f64 },
    Ellipse { w: f64, h: f64 },
    /// Pre-flattened polylines in local coordinates.
    Path(Vec<Polyline>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shape {
    pub id: u64,
    pub layer: usize,
    pub kind: Kind,
    pub xf: Xf,
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
    /// Convert any shape into a generic path (baking the transform in).
    pub fn bake(&mut self) {
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Device {
    pub name: String,
    /// Display units; profiles saved before this existed load as millimetres.
    #[serde(default)]
    pub units: Units,
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
}

impl Default for Device {
    fn default() -> Self {
        Device {
            name: "GRBL 1.1 (diode / CO2)".into(),
            units: Units::Mm,
            bed_w: 400.0,
            bed_h: 400.0,
            origin: Origin::FrontLeft,
            s_max: 1000.0,
            dynamic_power: true,
            travel_speed: 3000.0,
            return_home: true,
            baud: 115_200,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document {
    pub shapes: Vec<Shape>,
    pub layers: Vec<Layer>,
    pub device: Device,
    next_id: u64,
}

impl Default for Document {
    fn default() -> Self {
        Document {
            shapes: vec![],
            layers: (0..30).map(Layer::new).collect(),
            device: Device::default(),
            next_id: 1,
        }
    }
}

impl Document {
    pub fn add(&mut self, layer: usize, kind: Kind, xf: Xf) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.shapes.push(Shape { id, layer, kind, xf });
        id
    }
    pub fn add_shape(&mut self, mut s: Shape) -> u64 {
        s.id = self.next_id;
        self.next_id += 1;
        let id = s.id;
        self.shapes.push(s);
        id
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
