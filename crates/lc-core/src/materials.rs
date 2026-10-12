//! The materials of a device: each has a name and the settings of the layers it uses. The device keeps a list of
//! them and says which one is selected; the layers of the project follow the selected material.
use crate::doc::*;
use serde::{Deserialize, Serialize};

/// The name of the material every device starts with. Shown in the language of the interface.
pub const DEFAULT_MATERIAL: &str = "Default material";

/// The settings of one layer of a material.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaterialLayer {
    /// Index of the layer in the design (0..30).
    pub index: usize,
    pub settings: Layer,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Material {
    pub name: String,
    #[serde(default)]
    pub layers: Vec<MaterialLayer>,
}

impl Material {
    pub fn new(name: &str) -> Material {
        Material { name: name.to_string(), layers: vec![] }
    }

    pub fn layer(&self, index: usize) -> Option<&MaterialLayer> {
        self.layers.iter().find(|l| l.index == index)
    }

    fn put(&mut self, index: usize, settings: &Layer) {
        match self.layers.iter_mut().find(|l| l.index == index) {
            Some(l) => l.settings = settings.clone(),
            None => {
                self.layers.push(MaterialLayer { index, settings: settings.clone() });
                self.layers.sort_by_key(|l| l.index);
            }
        }
    }
}

impl Layer {
    /// Take the cutting settings of `other` (mode, speed, power, passes, fill, image, head, Z, rotary, kerf,
    /// perforation, tabs); the name, colour, output and visibility stay.
    pub fn take_settings(&mut self, other: &Layer) {
        let (name, color, output, visible) = (std::mem::take(&mut self.name), self.color, self.output, self.visible);
        *self = other.clone();
        self.name = name;
        self.color = color;
        self.output = output;
        self.visible = visible;
    }

    /// Do the two layers have the same cutting settings?
    pub fn same_settings(&self, other: &Layer) -> bool {
        let mut a = self.clone();
        a.take_settings(other);
        a == *self
    }
}

/// Names must be told apart; `base` becomes "base (2)", "base (3)"... when taken.
pub fn free_name(taken: &[String], base: &str) -> String {
    let base = base.trim();
    let base = if base.is_empty() { "Material" } else { base };
    if !taken.iter().any(|t| t.eq_ignore_ascii_case(base)) {
        return base.to_string();
    }
    (2..).map(|n| format!("{base} ({n})")).find(|c| !taken.iter().any(|t| t.eq_ignore_ascii_case(c))).unwrap_or_default()
}

/// One layer of a starter material: mode, speed in mm/min, power in percent, passes, interval in mm.
fn starter_layer(index: usize, mode: LayerMode, speed_mm_min: f64, power: f64, passes: u32, interval: f64) -> MaterialLayer {
    let mut l = Layer::new(index);
    l.mode = mode;
    l.speed = speed_mm_min / 60.0;
    l.power = power;
    l.passes = passes;
    l.interval = interval;
    MaterialLayer { index, settings: l }
}

/// A material with an engraving layer (layer 0, fill) and a cutting layer (layer 1, line).
fn starter(name: &str, engrave: (f64, f64), cut: (f64, f64, u32)) -> Material {
    Material {
        name: name.to_string(),
        layers: vec![starter_layer(0, LayerMode::Fill, engrave.0, engrave.1, 1, 0.1), starter_layer(1, LayerMode::Line, cut.0, cut.1, cut.2, 0.1)],
    }
}

/// The materials a new device offers, besides the empty default one: generic starting points for a typical diode
/// (about 10 W) or CO2 (about 40 W) laser. Always test on scrap. Their names are translated.
pub fn starter_materials(laser: LaserKind) -> Vec<Material> {
    match laser {
        LaserKind::Diode => vec![
            starter("Plywood 3 mm", (6000.0, 35.0), (180.0, 100.0, 3)),
            starter("Cardboard", (6000.0, 25.0), (600.0, 100.0, 1)),
            starter("Leather 2 mm", (5000.0, 30.0), (250.0, 100.0, 2)),
            starter("Paper", (6000.0, 12.0), (1500.0, 70.0, 1)),
        ],
        LaserKind::Co2 => vec![
            starter("Plywood 3 mm", (12000.0, 25.0), (600.0, 70.0, 1)),
            starter("Acrylic 3 mm", (12000.0, 20.0), (480.0, 75.0, 1)),
            starter("Cardboard", (15000.0, 15.0), (1500.0, 40.0, 1)),
            starter("Leather 2 mm", (12000.0, 18.0), (900.0, 45.0, 1)),
            starter("Paper", (15000.0, 10.0), (3000.0, 20.0, 1)),
        ],
        LaserKind::Fiber => vec![],
    }
}

impl Device {
    /// A device that has only what it started with (the empty default material, or the starters of some laser type)
    /// gets the starter materials of its laser type. A device whose materials were changed is left alone.
    pub fn add_starter_materials(&mut self) {
        let empty_default = |m: &[Material]| m.len() == 1 && m[0].name == DEFAULT_MATERIAL && m[0].layers.is_empty();
        let with_starters = |k: LaserKind| {
            let mut v = vec![Material::new(DEFAULT_MATERIAL)];
            v.extend(starter_materials(k));
            v
        };
        let pristine = empty_default(&self.materials) || [LaserKind::Diode, LaserKind::Co2, LaserKind::Fiber].into_iter().any(|k| self.materials == with_starters(k));
        if pristine {
            self.materials = with_starters(self.laser());
            self.active_material = 0;
        }
    }

    /// A device always has a material: older profiles get the default one.
    pub fn ensure_materials(&mut self) {
        if self.materials.is_empty() {
            self.materials.push(Material::new(DEFAULT_MATERIAL));
        }
        self.materials.truncate(200);
        if self.active_material >= self.materials.len() {
            self.active_material = 0;
        }
    }

    pub fn material(&self) -> &Material {
        &self.materials[self.active_material.min(self.materials.len().saturating_sub(1))]
    }
}

impl Document {
    /// Is the layer used by the design?
    fn layer_used(&self, li: usize) -> bool {
        self.scene.shapes.iter().any(|s| s.layer == li)
    }

    /// Select a material: the layers it defines get its settings; the other layers go back to the default
    /// settings. A layer the design uses that the material lacks is added to it with those defaults.
    pub fn select_material(&mut self, i: usize) {
        self.device.ensure_materials();
        if i >= self.device.materials.len() {
            return;
        }
        self.device.active_material = i;
        for li in 0..self.layers.len() {
            let from = self.device.materials[i].layer(li).map(|m| m.settings.clone()).unwrap_or_else(|| Layer::new(li));
            self.layers[li].take_settings(&from);
        }
        self.sync_material();
    }

    /// Keep the selected material in step with the layers of the design: a layer that was edited, or that the
    /// design uses and the material lacks, is written into the material. Returns whether the material changed.
    pub fn sync_material(&mut self) -> bool {
        self.device.ensure_materials();
        let a = self.device.active_material;
        let mut changed = false;
        for li in 0..self.layers.len() {
            let used = self.layer_used(li);
            let layer = self.layers[li].clone();
            let same = self.device.materials[a].layer(li).map(|m| layer.same_settings(&m.settings));
            let wanted = match same {
                Some(true) => false,
                Some(false) => true,
                // Not in the material: it joins when the design uses it or when it differs from the defaults.
                None => used || !layer.same_settings(&Layer::new(li)),
            };
            if wanted {
                self.device.materials[a].put(li, &layer);
                changed = true;
            }
        }
        changed
    }

    /// A new, empty material with a free name; it becomes the selected one. The layers go back to the default
    /// settings, and the layers the design uses join the material with them.
    pub fn add_material(&mut self, name: &str) -> usize {
        self.sync_material();
        let names: Vec<String> = self.device.materials.iter().map(|m| m.name.clone()).collect();
        self.device.materials.push(Material::new(&free_name(&names, name)));
        let i = self.device.materials.len() - 1;
        self.select_material(i);
        i
    }

    pub fn rename_material(&mut self, name: &str) {
        let a = self.device.active_material;
        let others: Vec<String> = self.device.materials.iter().enumerate().filter(|(i, _)| *i != a).map(|(_, m)| m.name.clone()).collect();
        if let Some(m) = self.device.materials.get_mut(a) {
            m.name = free_name(&others, name);
        }
    }

    /// Delete the selected material (never the last one); the one before it is selected and applied.
    pub fn delete_material(&mut self) -> bool {
        if self.device.materials.len() < 2 {
            return false;
        }
        self.device.materials.remove(self.device.active_material);
        let next = self.device.active_material.saturating_sub(1).min(self.device.materials.len() - 1);
        self.select_material(next);
        true
    }
}
