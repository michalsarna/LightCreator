//! LightCreator core: geometry, document model, SVG I/O and G-code generation.
pub mod controller;
pub mod doc;
pub mod gcode;
pub mod geom;
pub mod svg;

pub use doc::*;
pub use geom::*;

#[cfg(test)]
mod tests;
