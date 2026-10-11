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

impl Device {
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
        self.shapes.iter().any(|s| s.layer == li)
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

    /// A new material with a free name, made from the settings the layers have now; it becomes the selected one.
    pub fn add_material(&mut self, name: &str) -> usize {
        self.sync_material();
        let names: Vec<String> = self.device.materials.iter().map(|m| m.name.clone()).collect();
        let mut m = self.device.material().clone();
        m.name = free_name(&names, name);
        self.device.materials.push(m);
        self.device.active_material = self.device.materials.len() - 1;
        self.device.active_material
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
