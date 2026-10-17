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
    /// A colour to tell the material apart in the lists. It is only shown: nothing is computed from it. Not written when there
    /// is none, so older files stay the same.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<[u8; 3]>,
    #[serde(default)]
    pub layers: Vec<MaterialLayer>,
}

impl Material {
    /// Add a layer with the default settings (nothing when it is there already or the number is out of range).
    pub fn add_layer(&mut self, index: usize) -> bool {
        if index >= LAYER_COUNT || self.layer(index).is_some() {
            return false;
        }
        self.put(index, &Layer::new(index));
        true
    }

    /// Take a layer out of the material.
    pub fn remove_layer(&mut self, index: usize) -> bool {
        let n = self.layers.len();
        self.layers.retain(|l| l.index != index);
        self.layers.len() != n
    }

    pub fn new(name: &str) -> Material {
        Material { name: name.to_string(), color: None, layers: vec![] }
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
        color: None,
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
        self.rename_material_at(self.device.active_material, name);
    }

    /// Give material `i` a new name; the name is made free when another material has it.
    pub fn rename_material_at(&mut self, i: usize, name: &str) {
        let others: Vec<String> = self.device.materials.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, m)| m.name.clone()).collect();
        if let Some(m) = self.device.materials.get_mut(i) {
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

impl Document {
    /// A copy of material `i` under a free name ("Name (copy)"); it becomes the selected one.
    pub fn duplicate_material(&mut self, i: usize) -> Option<usize> {
        self.sync_material();
        let src = self.device.materials.get(i)?.clone();
        let taken: Vec<String> = self.device.materials.iter().map(|m| m.name.clone()).collect();
        if taken.len() >= MAX_MATERIALS {
            return None;
        }
        let mut copy = src;
        copy.name = crate::profiles::copy_name(&copy.name, &taken);
        self.device.materials.insert(i + 1, copy);
        self.select_material(i + 1);
        Some(i + 1)
    }

    /// Delete material `i` (never the last one). The selected material stays selected (or the one before it, when it was
    /// the one deleted).
    pub fn delete_material_at(&mut self, i: usize) -> bool {
        if self.device.materials.len() < 2 || i >= self.device.materials.len() {
            return false;
        }
        self.sync_material();
        let active = self.device.active_material;
        self.device.materials.remove(i);
        let next = if i < active {
            active - 1
        } else if i == active {
            active.saturating_sub(1)
        } else {
            active
        };
        self.select_material(next.min(self.device.materials.len() - 1));
        true
    }

    /// Move material `i` one place up or down in the list. Returns its new place. The selected material stays the same one.
    pub fn move_material(&mut self, i: usize, up: bool) -> usize {
        let n = self.device.materials.len();
        let j = if up { i.checked_sub(1) } else { (i + 1 < n).then_some(i + 1) };
        let Some(j) = j.filter(|j| i < n && *j < n) else { return i };
        self.device.materials.swap(i, j);
        let a = self.device.active_material;
        self.device.active_material = if a == i {
            j
        } else if a == j {
            i
        } else {
            a
        };
        j
    }

    /// Put what was edited in material `i` (not the selected one, or the selected one) into the layers of the design.
    pub fn apply_material_edit(&mut self, i: usize) {
        if i == self.device.active_material {
            self.select_material(i);
        }
    }
}

/// The most materials a device keeps.
pub const MAX_MATERIALS: usize = 200;
/// The biggest file of materials that is read.
pub const MAX_MATERIALS_FILE: usize = 2_000_000;
/// The version of the file format written now.
pub const MATERIALS_FORMAT: u32 = 1;
const MATERIALS_KIND: &str = "lightcreator-materials";

#[derive(Serialize, Deserialize)]
struct MaterialsFile {
    #[serde(default)]
    kind: String,
    #[serde(default)]
    format: u32,
    #[serde(default)]
    app_version: String,
    materials: Vec<Material>,
}

/// The text of a file that holds `materials` (all the settings of their layers, the names, the colours).
pub fn export_materials(materials: &[Material], app_version: &str) -> String {
    serde_json::to_string_pretty(&MaterialsFile { kind: MATERIALS_KIND.into(), format: MATERIALS_FORMAT, app_version: app_version.into(), materials: materials.to_vec() }).unwrap_or_default()
}

/// Read a file of materials. It comes from outside, so everything in it is checked: the size, the number of materials, every
/// name and every number of a layer. A file made by a newer version is refused.
pub fn import_materials(text: &str) -> Result<Vec<Material>, String> {
    if text.len() > MAX_MATERIALS_FILE {
        return Err(format!("the file is larger than {} KB", MAX_MATERIALS_FILE / 1000));
    }
    let f: MaterialsFile = serde_json::from_str(text).map_err(|_| "this is not a file of materials".to_string())?;
    if f.kind != MATERIALS_KIND {
        return Err("this is not a file of materials".into());
    }
    if f.format > MATERIALS_FORMAT {
        return Err(format!("the file is of format {}, made by a newer version ({}); update the program", f.format, f.app_version));
    }
    if f.materials.is_empty() || f.materials.len() > MAX_MATERIALS {
        return Err("the number of materials is out of range".into());
    }
    for m in &f.materials {
        crate::profiles::validate_material(m)?;
    }
    Ok(f.materials)
}

/// Put imported materials into the list: a name that is there already is kept apart under a free name, or, with `replace`,
/// the material with that name is replaced. Returns how many were added and how many replaced.
pub fn merge_imported(list: &mut Vec<Material>, imported: Vec<Material>, replace: bool) -> (usize, usize) {
    let (mut added, mut replaced) = (0, 0);
    for mut m in imported {
        let same = list.iter().position(|x| x.name.eq_ignore_ascii_case(&m.name));
        match same {
            Some(i) if replace => {
                list[i] = m;
                replaced += 1;
            }
            _ if list.len() >= MAX_MATERIALS => break,
            _ => {
                let taken: Vec<String> = list.iter().map(|x| x.name.clone()).collect();
                m.name = free_name(&taken, &m.name);
                list.push(m);
                added += 1;
            }
        }
    }
    (added, replaced)
}
