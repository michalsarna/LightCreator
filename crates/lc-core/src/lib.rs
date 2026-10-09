//! LightCreator core: geometry, document model, SVG I/O and G-code generation.
pub mod bezier;
pub mod controller;
pub mod doc;
pub mod export;
pub mod gcode;
pub mod image;
pub mod geom;
pub mod materials;
pub mod ops;
pub mod pdf;
pub mod raster;
pub mod ser2net;
pub mod svg;
pub mod text;
pub mod trace;

pub use bezier::*;
pub use doc::*;
pub use geom::*;
pub use image::{Adjust, ImageData};
pub use text::TextData;

#[cfg(test)]
mod tests;
