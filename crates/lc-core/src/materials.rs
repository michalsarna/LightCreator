//! Material library: starting settings per material, operation and laser type.
//! The built-in values are conservative starting points for typical 10 W diode and 40 W CO2 lasers.
//! Every machine differs, so they must always be tested on a scrap piece first.
use crate::doc::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Preset {
    /// Material and thickness, e.g. "Plywood 3 mm".
    pub material: String,
    /// What the setting is for: "Cut", "Engrave", "Score".
    pub operation: String,
    pub laser: LaserKind,
    pub mode: LayerMode,
    /// mm/s
    pub speed: f64,
    /// Percent.
    pub power: f64,
    pub passes: u32,
    /// Fill interval (mm), used by fill modes.
    pub interval: f64,
    #[serde(default)]
    pub user: bool,
}

impl Preset {
    fn new(material: &str, operation: &str, laser: LaserKind, mode: LayerMode, speed: f64, power: f64, passes: u32, interval: f64) -> Preset {
        Preset { material: material.into(), operation: operation.into(), laser, mode, speed, power, passes, interval, user: false }
    }

    /// Copy the preset into a layer (the layer name and colour stay).
    pub fn apply(&self, l: &mut Layer) {
        l.mode = self.mode;
        l.speed = self.speed;
        l.power = self.power;
        l.passes = self.passes.max(1);
        l.interval = self.interval;
        l.min_power = l.min_power.min(self.power);
    }

    /// A user preset taken from a layer's current settings.
    pub fn from_layer(material: &str, operation: &str, laser: LaserKind, l: &Layer) -> Preset {
        Preset { material: material.into(), operation: operation.into(), laser, mode: l.mode, speed: l.speed, power: l.power, passes: l.passes, interval: l.interval, user: true }
    }
}

pub fn builtin() -> Vec<Preset> {
    use LaserKind::{Co2, Diode};
    use LayerMode::{Fill, Line};
    let p = Preset::new;
    vec![
        // ---- diode (about 10 W optical) ----
        p("Plywood 3 mm", "Cut", Diode, Line, 3.0, 100.0, 2, 0.1),
        p("Plywood 3 mm", "Engrave", Diode, Fill, 60.0, 40.0, 1, 0.1),
        p("Plywood 6 mm", "Cut", Diode, Line, 1.5, 100.0, 4, 0.1),
        p("MDF 3 mm", "Cut", Diode, Line, 3.0, 100.0, 3, 0.1),
        p("MDF 3 mm", "Engrave", Diode, Fill, 70.0, 35.0, 1, 0.1),
        p("Cardboard", "Cut", Diode, Line, 15.0, 80.0, 1, 0.1),
        p("Cardboard", "Engrave", Diode, Fill, 100.0, 20.0, 1, 0.1),
        p("Paper", "Cut", Diode, Line, 30.0, 40.0, 1, 0.1),
        p("Leather 2 mm", "Cut", Diode, Line, 5.0, 100.0, 2, 0.1),
        p("Leather 2 mm", "Engrave", Diode, Fill, 80.0, 30.0, 1, 0.1),
        p("Felt 3 mm", "Cut", Diode, Line, 8.0, 100.0, 1, 0.1),
        p("Cork 3 mm", "Cut", Diode, Line, 4.0, 100.0, 2, 0.1),
        p("Acrylic, black or dark", "Engrave", Diode, Fill, 60.0, 40.0, 1, 0.1),
        p("Slate / stone", "Engrave", Diode, Fill, 100.0, 60.0, 1, 0.1),
        p("Anodised aluminium (coated)", "Engrave", Diode, Fill, 40.0, 70.0, 1, 0.08),
        // ---- CO2 (about 40 W) ----
        p("Plywood 3 mm", "Cut", Co2, Line, 15.0, 70.0, 1, 0.1),
        p("Plywood 6 mm", "Cut", Co2, Line, 7.0, 80.0, 1, 0.1),
        p("Plywood 3 mm", "Engrave", Co2, Fill, 200.0, 20.0, 1, 0.1),
        p("MDF 3 mm", "Cut", Co2, Line, 15.0, 65.0, 1, 0.1),
        p("MDF 3 mm", "Engrave", Co2, Fill, 250.0, 20.0, 1, 0.1),
        p("Acrylic 3 mm", "Cut", Co2, Line, 10.0, 60.0, 1, 0.1),
        p("Acrylic 5 mm", "Cut", Co2, Line, 6.0, 70.0, 1, 0.1),
        p("Acrylic 3 mm", "Engrave", Co2, Fill, 250.0, 15.0, 1, 0.1),
        p("Cardboard", "Cut", Co2, Line, 40.0, 30.0, 1, 0.1),
        p("Paper", "Cut", Co2, Line, 60.0, 15.0, 1, 0.1),
        p("Leather 2 mm", "Cut", Co2, Line, 20.0, 40.0, 1, 0.1),
        p("Leather 2 mm", "Engrave", Co2, Fill, 250.0, 15.0, 1, 0.1),
        p("Rubber 2 mm", "Cut", Co2, Line, 8.0, 80.0, 1, 0.1),
        p("Fabric", "Cut", Co2, Line, 40.0, 25.0, 1, 0.1),
        p("Glass", "Engrave", Co2, Fill, 300.0, 20.0, 1, 0.1),
        p("Slate / stone", "Engrave", Co2, Fill, 250.0, 20.0, 1, 0.1),
        p("Wood (hardwood)", "Engrave", Co2, Fill, 300.0, 25.0, 1, 0.1),
    ]
}

// ---------------------------------------------------------------------------------------------------------------
// Materials with the settings of several layers.
//
// A material holds the settings of every layer it needs (cut on layer 0, engrave on layer 1 and so on), including
// speed, power, mode, passes and air assist. There is one built-in library (below), a global library of materials
// the user added to the program, and one library per device that is kept in the device profile and travels with
// saved projects.
// ---------------------------------------------------------------------------------------------------------------

/// The settings of one layer that make up a material.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayerSpec {
    pub mode: LayerMode,
    /// mm/s
    pub speed: f64,
    /// Percent.
    pub power: f64,
    pub passes: u32,
    /// Fill interval (mm).
    pub interval: f64,
    pub angle: f64,
    pub overscan: f64,
    pub bidirectional: bool,
    pub dither: Dither,
    pub min_power: f64,
    pub air_assist: bool,
    /// Index of the layer colour in `PALETTE`; `NO_COLOUR` for materials saved before colours were kept.
    #[serde(default = "no_colour")]
    pub colour: usize,
    /// Kerf offset (mm), perforation and tabs of line layers.
    #[serde(default)]
    pub kerf: f64,
    #[serde(default)]
    pub perforation: crate::cutopts::Perforation,
    #[serde(default)]
    pub tabs: crate::cutopts::Tabs,
}

/// A material layer that does not say which colour it has.
pub const NO_COLOUR: usize = usize::MAX;

fn no_colour() -> usize {
    NO_COLOUR
}

/// A value of one setting, for showing differences.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Mode(LayerMode),
    /// mm/s
    Speed(f64),
    Percent(f64),
    Count(u32),
    /// mm
    Length(f64),
    /// Degrees.
    Angle(f64),
    Flag(bool),
    Dither(Dither),
    /// Index into `PALETTE`.
    Colour(usize),
    /// A group of settings (on or off, and a fingerprint of the numbers inside, so a change in them shows).
    Group(bool, u64),
}

impl LayerSpec {
    /// The names of the settings, as shown in the interface (they are also the keys of `value` / `take`).
    pub const FIELDS: [&'static str; 15] = ["Colour", "Mode", "Speed", "Power (%)", "Passes", "Interval", "Scan angle", "Overscan", "Bidirectional", "Dithering", "Min power (%)", "Air assist", "Kerf offset", "Perforation", "Tabs / bridges"];

    pub fn from_layer(l: &Layer) -> LayerSpec {
        LayerSpec {
            mode: l.mode,
            speed: l.speed,
            power: l.power,
            passes: l.passes,
            interval: l.interval,
            angle: l.angle,
            overscan: l.overscan,
            bidirectional: l.bidirectional,
            dither: l.dither,
            min_power: l.min_power,
            air_assist: l.air_assist,
            colour: l.color,
            kerf: l.kerf,
            perforation: l.perforation,
            tabs: l.tabs,
        }
    }

    /// Copy the settings (and the colour) into a layer; its name, output and visibility stay.
    pub fn apply(&self, l: &mut Layer) {
        l.mode = self.mode;
        l.speed = self.speed;
        l.power = self.power;
        l.passes = self.passes.max(1);
        l.interval = self.interval;
        l.angle = self.angle;
        l.overscan = self.overscan;
        l.bidirectional = self.bidirectional;
        l.dither = self.dither;
        l.min_power = self.min_power.min(self.power);
        l.air_assist = self.air_assist;
        if self.colour < PALETTE.len() {
            l.color = self.colour;
        }
        l.kerf = if self.kerf.is_finite() { self.kerf.clamp(-50.0, 50.0) } else { 0.0 };
        l.perforation = self.perforation.clamped();
        l.tabs = self.tabs.clamped();
    }

    pub fn value(&self, field: &str) -> Option<Value> {
        Some(match field {
            "Mode" => Value::Mode(self.mode),
            "Speed" => Value::Speed(self.speed),
            "Power (%)" => Value::Percent(self.power),
            "Passes" => Value::Count(self.passes),
            "Interval" => Value::Length(self.interval),
            "Scan angle" => Value::Angle(self.angle),
            "Overscan" => Value::Length(self.overscan),
            "Bidirectional" => Value::Flag(self.bidirectional),
            "Dithering" => Value::Dither(self.dither),
            "Min power (%)" => Value::Percent(self.min_power),
            "Air assist" => Value::Flag(self.air_assist),
            "Colour" => Value::Colour(self.colour),
            "Kerf offset" => Value::Length(self.kerf),
            "Perforation" => Value::Group(self.perforation.enabled, fingerprint(&[self.perforation.cut, self.perforation.skip])),
            "Tabs / bridges" => Value::Group(self.tabs.enabled, fingerprint(&[self.tabs.size, self.tabs.spacing, self.tabs.per_shape.into(), self.tabs.min_tabs.into(), self.tabs.max_tabs.into(), self.tabs.cut_power, f64::from(self.tabs.limit_max), f64::from(self.tabs.skip_inner), f64::from(self.tabs.mode as u8)])),
            _ => return None,
        })
    }

    /// Take one setting from `other`.
    pub fn take(&mut self, field: &str, other: &LayerSpec) {
        match field {
            "Mode" => self.mode = other.mode,
            "Speed" => self.speed = other.speed,
            "Power (%)" => self.power = other.power,
            "Passes" => self.passes = other.passes,
            "Interval" => self.interval = other.interval,
            "Scan angle" => self.angle = other.angle,
            "Overscan" => self.overscan = other.overscan,
            "Bidirectional" => self.bidirectional = other.bidirectional,
            "Dithering" => self.dither = other.dither,
            "Min power (%)" => self.min_power = other.min_power,
            "Air assist" => self.air_assist = other.air_assist,
            "Colour" => self.colour = other.colour,
            "Kerf offset" => self.kerf = other.kerf,
            "Perforation" => self.perforation = other.perforation,
            "Tabs / bridges" => self.tabs = other.tabs,
            _ => {}
        }
    }
}

/// A number that is the same for the same list of settings.
fn fingerprint(v: &[f64]) -> u64 {
    v.iter().fold(0xcbf29ce484222325u64, |h, x| (h ^ x.to_bits()).wrapping_mul(0x100000001b3))
}

/// Two values are the same when they differ by less than a rounding error.
fn same_value(a: Value, b: Value) -> bool {
    match (a, b) {
        (Value::Speed(x), Value::Speed(y)) | (Value::Percent(x), Value::Percent(y)) | (Value::Length(x), Value::Length(y)) | (Value::Angle(x), Value::Angle(y)) => (x - y).abs() < 1e-6,
        (a, b) => a == b,
    }
}

/// The settings of one layer of a material, with the layer it belongs to.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaterialLayer {
    /// Index of the layer in the design (0..30).
    pub index: usize,
    /// What the layer is for: "Cut", "Engrave", "Score" or a name of your own.
    pub label: String,
    pub spec: LayerSpec,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Material {
    /// Identifies the material in every library and file, whatever it is called or in which language. Empty in
    /// materials saved before ids existed: `key` then derives it from the name.
    #[serde(default)]
    pub id: String,
    /// The name without the thickness, e.g. "Plywood". The names that ship with the program are translated.
    pub name: String,
    /// mm; 0 when the thickness does not matter.
    #[serde(default)]
    pub thickness: f64,
    /// The settings are for this type of laser: one material (one id) has a variant per laser type.
    pub laser: LaserKind,
    /// What the layers are for ("Cut", "Engrave"...), to find the material by.
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub note: String,
    /// "Wood", "Metal", a category of your own... Empty: guessed from the name (see `category`).
    #[serde(default)]
    pub category: String,
    pub layers: Vec<MaterialLayer>,
}

/// The categories every library starts with. Materials may use any other name too.
pub const CATEGORIES: [&str; 6] = ["Wood", "Metal", "Plastic", "Paper and leather", "Stone and glass", "Other"];

/// The category a material name suggests ("Plywood 3 mm" is Wood).
pub fn guess_category(name: &str) -> &'static str {
    let n = name.to_lowercase();
    let has = |words: &[&str]| words.iter().any(|w| n.contains(w));
    if has(&["plywood", "mdf", "wood", "cork", "bamboo", "veneer"]) {
        "Wood"
    } else if has(&["alumin", "steel", "metal", "brass", "copper", "titan"]) {
        "Metal"
    } else if has(&["acrylic", "plastic", "rubber", "pvc", "abs", "delrin", "foam"]) {
        "Plastic"
    } else if has(&["paper", "card", "leather", "felt", "fabric", "cloth"]) {
        "Paper and leather"
    } else if has(&["slate", "stone", "glass", "ceramic", "tile", "marble"]) {
        "Stone and glass"
    } else {
        "Other"
    }
}

/// "Plywood 3 mm" -> ("Plywood", 3.0); a name without a thickness comes back as it is, with 0.
pub fn split_thickness(name: &str) -> (String, f64) {
    let n = name.trim();
    if let Some(head) = n.strip_suffix("mm").map(str::trim_end) {
        if let Some((base, num)) = head.rsplit_once(' ') {
            if let Ok(t) = num.replace(',', ".").parse::<f64>() {
                if t > 0.0 && !base.trim().is_empty() {
                    return (base.trim().to_string(), t);
                }
            }
        }
    }
    (n.to_string(), 0.0)
}

/// An id made of a name and a thickness: "plywood-3".
pub fn make_id(name: &str, thickness: f64) -> String {
    let mut s: String = name.to_lowercase().chars().map(|c| if c.is_alphanumeric() { c } else { '-' }).collect();
    while s.contains("--") {
        s = s.replace("--", "-");
    }
    let mut s = s.trim_matches('-').to_string();
    if thickness > 0.0 {
        s.push_str(&format!("-{}", thickness).replace('.', "_"));
    }
    s
}

impl Material {
    /// The id, or one derived from the name for materials saved without.
    pub fn key(&self) -> String {
        if self.id.is_empty() {
            make_id(&self.name, self.thickness)
        } else {
            self.id.clone()
        }
    }

    /// Give the material its id (kept when the name is changed later).
    pub fn freeze_id(&mut self) {
        if self.id.is_empty() {
            self.id = make_id(&self.name, self.thickness);
        }
    }

    /// A material taken from the settings of the given layers (index, layer).
    /// A thickness at the end of the name ("Birch 3 mm") becomes the thickness of the material.
    pub fn from_layers(name: &str, laser: LaserKind, layers: &[(usize, &Layer)]) -> Material {
        let (base, thickness) = split_thickness(name);
        let mut ls: Vec<MaterialLayer> = layers.iter().map(|(i, l)| MaterialLayer { index: *i, label: l.name.clone(), spec: LayerSpec::from_layer(l) }).collect();
        ls.sort_by_key(|l| l.index);
        let tags = ls.iter().map(|l| l.label.clone()).collect();
        Material { id: make_id(&base, thickness), name: base.clone(), thickness, laser, tags, note: String::new(), category: guess_category(&base).to_string(), layers: ls }
    }

    /// The category of the material: the one set, or a guess from its name.
    pub fn category(&self) -> String {
        if self.category.trim().is_empty() {
            guess_category(&self.name).to_string()
        } else {
            self.category.trim().to_string()
        }
    }

    pub fn layer(&self, index: usize) -> Option<&MaterialLayer> {
        self.layers.iter().find(|l| l.index == index)
    }

    /// Put the settings of every layer of the material into `layers`: values, colour and name (the label).
    /// Returns how many layers were set.
    pub fn apply(&self, layers: &mut [Layer]) -> usize {
        let mut n = 0;
        for ml in &self.layers {
            if let Some(l) = layers.get_mut(ml.index) {
                ml.spec.apply(l);
                if !ml.label.trim().is_empty() {
                    l.name = ml.label.trim().to_string();
                }
                n += 1;
            }
        }
        n
    }

    /// Id and laser type identify a material within a library.
    pub fn same_key(&self, other: &Material) -> bool {
        self.key() == other.key() && self.laser == other.laser
    }
}

/// The index a layer gets for an operation in the built-in materials.
fn operation_index(op: &str) -> usize {
    match op {
        "Cut" => 0,
        "Engrave" => 1,
        "Score" => 2,
        _ => 3,
    }
}

/// Group single-layer presets (material + operation) into materials: cut on layer 0, engrave on layer 1, score on 2.
pub fn materials_from_presets(presets: &[Preset]) -> Vec<Material> {
    let mut out: Vec<Material> = vec![];
    for p in presets {
        let spec = LayerSpec { mode: p.mode, speed: p.speed, power: p.power, passes: p.passes.max(1), interval: p.interval, angle: 0.0, overscan: 1.0, bidirectional: true, dither: Dither::default(), min_power: 0.0, air_assist: false, colour: NO_COLOUR, kerf: 0.0, perforation: Default::default(), tabs: Default::default() };
        let mut index = operation_index(&p.operation);
        let pos = match out.iter().position(|m| (m.name.clone(), m.thickness) == split_thickness(&p.material) && m.laser == p.laser) {
            Some(i) => i,
            None => {
                let (base, thickness) = split_thickness(&p.material);
                out.push(Material { id: make_id(&base, thickness), category: guess_category(&base).to_string(), name: base, thickness, laser: p.laser, tags: vec![], note: String::new(), layers: vec![] });
                out.len() - 1
            }
        };
        // A custom operation name takes the next free layer.
        while out[pos].layer(index).is_some() {
            index += 1;
        }
        if !out[pos].tags.contains(&p.operation) {
            out[pos].tags.push(p.operation.clone());
        }
        let index = index.min(29);
        let spec = LayerSpec { colour: index, ..spec };
        out[pos].layers.push(MaterialLayer { index, label: p.operation.clone(), spec });
    }
    for m in &mut out {
        m.layers.sort_by_key(|l| l.index);
    }
    out
}

/// The built-in materials: the single-layer starting values of `builtin()` grouped into materials.
pub fn builtin_materials() -> Vec<Material> {
    materials_from_presets(&builtin())
}

/// The materials of the main library as this computer sees them: the ones that ship with the program, where
/// `local` holds another version of one (same id and laser type) that one, and then the ones only `local` has.
pub fn effective_library(builtin: &[Material], local: &[Material]) -> Vec<Material> {
    let mut out: Vec<Material> = builtin.iter().map(|b| local.iter().find(|l| l.same_key(b)).unwrap_or(b).clone()).collect();
    out.extend(local.iter().filter(|l| !builtin.iter().any(|b| b.same_key(l))).cloned());
    out
}

/// Copy the materials of `source` that suit `laser` and are not yet in `library` into it. Returns how many.
pub fn add_missing(library: &mut Vec<Material>, source: &[Material], laser: LaserKind) -> usize {
    let mut n = 0;
    for m in source.iter().filter(|m| m.laser == laser) {
        if !library.iter().any(|l| l.same_key(m)) {
            library.push(m.clone());
            n += 1;
        }
    }
    n
}

// ---- Comparing two libraries ----

/// One difference between a material in the local library and the same one coming from a file.
#[derive(Clone, Debug, PartialEq)]
pub struct DiffRow {
    pub layer: usize,
    /// One of `LayerSpec::FIELDS`, or "" when the whole layer is missing locally.
    pub field: &'static str,
    pub local: Option<Value>,
    pub incoming: Option<Value>,
}

/// What a library coming from a file does to the local one.
#[derive(Debug, Default)]
pub struct MergePlan {
    /// Materials the local library does not have: added at once.
    pub added: Vec<Material>,
    /// Materials with the same name and laser type but other settings: the user decides.
    pub conflicts: Vec<Conflict>,
    /// Materials that are the same on both sides.
    pub unchanged: usize,
}

#[derive(Debug)]
pub struct Conflict {
    pub name: String,
    pub incoming: Material,
    pub rows: Vec<DiffRow>,
}

/// The settings of the layers of `incoming` that differ from `local` (layers only `local` has are left alone).
pub fn diff(local: &Material, incoming: &Material) -> Vec<DiffRow> {
    let mut rows = vec![];
    for il in &incoming.layers {
        match local.layer(il.index) {
            None => rows.push(DiffRow { layer: il.index, field: "", local: None, incoming: None }),
            Some(ll) => {
                for f in LayerSpec::FIELDS {
                    let (a, b) = (ll.spec.value(f), il.spec.value(f));
                    if let (Some(a), Some(b)) = (a, b) {
                        if !same_value(a, b) {
                            rows.push(DiffRow { layer: il.index, field: f, local: Some(a), incoming: Some(b) });
                        }
                    }
                }
            }
        }
    }
    rows
}

/// Sort the materials of an incoming library into new, equal and different ones.
pub fn plan_merge(local: &[Material], incoming: &[Material]) -> MergePlan {
    let mut plan = MergePlan::default();
    for m in incoming {
        match local.iter().find(|l| l.same_key(m)) {
            None => plan.added.push(m.clone()),
            Some(l) => {
                let rows = diff(l, m);
                if rows.is_empty() {
                    plan.unchanged += 1;
                } else {
                    plan.conflicts.push(Conflict { name: l.name.clone(), incoming: m.clone(), rows });
                }
            }
        }
    }
    plan
}

/// Apply the chosen rows of a conflict to the local material: `take[i]` true takes row `i` from the file.
pub fn apply_rows(local: &mut Material, conflict: &Conflict, take: &[bool]) {
    for (row, take) in conflict.rows.iter().zip(take) {
        if !*take {
            continue;
        }
        let Some(il) = conflict.incoming.layer(row.layer) else { continue };
        if row.field.is_empty() {
            if local.layer(row.layer).is_none() {
                local.layers.push(il.clone());
                local.layers.sort_by_key(|l| l.index);
            }
        } else if let Some(ll) = local.layers.iter_mut().find(|l| l.index == row.layer) {
            ll.spec.take(row.field, &il.spec);
        }
    }
}

// ---- The library file (.lcb) ----

/// What `LightCreator library` files start with.
pub const LIBRARY_FORMAT: &str = "lightcreator-library";

/// The private library of a user in one file: the materials added for every device and the devices, each with
/// the materials of its own library.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LibraryFile {
    pub format: String,
    pub version: u32,
    #[serde(default)]
    pub global: Vec<Material>,
    #[serde(default)]
    pub devices: Vec<Device>,
}

impl LibraryFile {
    pub fn new(global: Vec<Material>, devices: Vec<Device>) -> LibraryFile {
        LibraryFile { format: LIBRARY_FORMAT.into(), version: 1, global, devices }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Read a library file; `None` when the text is not one.
    pub fn from_json(text: &str) -> Option<LibraryFile> {
        serde_json::from_str::<LibraryFile>(text).ok().filter(|f| f.format == LIBRARY_FORMAT)
    }
}
