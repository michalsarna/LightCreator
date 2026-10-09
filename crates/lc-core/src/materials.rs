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
