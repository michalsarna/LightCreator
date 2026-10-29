//! LightCreator core: the laser side of the document (layers, device, jobs, G-code) on top of base-core.
pub use base_core::{autosave, plugin_script, plugins, arcs, backup, bezier, camimg, control, dxf, fillet, geom, grbl_codes, grbl_settings, grbl_status, image, lightburn, ops, paths, project, resume, ser2net, snap, text, trace, ugsd};
pub mod check;
pub mod clipart;
pub mod controller;
pub mod cutopts;
pub mod doc;
pub mod export;
pub mod gcode;
pub mod hooks;
pub mod materials;
pub mod print;
pub mod pdf;
pub mod profiles;
pub mod raster;
pub mod svg;
pub mod zones;

pub use base_core::bezier::*;
pub use base_core::camimg::{CameraKind, CameraSource, CameraView, ImageSettings, Lens, RefPoint};
pub use controller::ControllerSettings;
pub use doc::*;
pub use base_core::geom::*;
pub use base_core::image::{Adjust, ImageData};
pub use base_core::text::TextData;

#[cfg(test)]
mod tests;
