//! LightCreator core: geometry, document model, SVG I/O and G-code generation.
pub mod bezier;
pub mod controller;
pub mod doc;
pub mod gcode;
pub mod image;
pub mod geom;
pub mod ops;
pub mod raster;
pub mod svg;
pub mod text;

pub use bezier::*;
pub use doc::*;
pub use geom::*;
pub use image::ImageData;
pub use text::TextData;

#[cfg(test)]
mod tests;
