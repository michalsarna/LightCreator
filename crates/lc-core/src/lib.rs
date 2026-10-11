//! LightCreator core: geometry, document model, SVG I/O and G-code generation.
pub mod arcs;
pub mod bezier;
pub mod camimg;
pub mod check;
pub mod clipart;
pub mod controller;
pub mod doc;
pub mod export;
pub mod gcode;
pub mod hooks;
pub mod image;
pub mod fillet;
pub mod geom;
pub mod grbl_settings;
pub mod materials;
pub mod ops;
pub mod pdf;
pub mod profiles;
pub mod raster;
pub mod ser2net;
pub mod svg;
pub mod text;
pub mod trace;
pub mod zones;

pub use bezier::*;
pub use camimg::{CameraKind, CameraSource, ImageSettings, Lens, RefPoint};
pub use doc::*;
pub use geom::*;
pub use image::{Adjust, ImageData};
pub use text::TextData;

#[cfg(test)]
mod tests;
