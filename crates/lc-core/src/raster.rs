//! Image engraving: sample the bitmap on a scan grid, dither it and turn each row into laser runs.
use crate::doc::*;
use crate::image::ImageData;

/// One stretch of constant laser power along a scan line; `x0 < x1`, power in percent.
#[derive(Clone, Copy, Debug)]
pub struct Run {
    pub x0: f64,
    pub x1: f64,
    pub power: f64,
}

#[derive(Clone, Debug)]
pub struct Row {
    pub y: f64,
    /// Left to right.
    pub runs: Vec<Run>,
}

pub use base_core::raster::scan_rotation;

/// Scan rows for an image shape. `layer` supplies interval, powers and the dither mode.
pub fn rows_for(shape: &Shape, im: &ImageData, layer: &Layer) -> Vec<Row> {
    // The sampling and the dithering are the shared code of baseCreator; here the cells become runs of laser power.
    let Some(g) = base_core::raster::sample(shape, im, layer.interval, layer.angle, layer.dither) else { return vec![] };
    let (cols, rows, step, b_min) = (g.cols, g.rows, g.step, g.min);
    let d = &g.levels;
    let (pmax, pmin) = (layer.power, layer.min_power.min(layer.power));
    let gray = layer.dither == Dither::Grayscale;
    // Power for a level, quantised to 0.5 % so neighbouring cells merge into one run.
    let power_of = |v: f32| -> f64 {
        if v <= 0.004 {
            return 0.0;
        }
        if !gray {
            return pmax;
        }
        let p = pmin + (pmax - pmin) * v.min(1.0) as f64;
        (p * 2.0).round() / 2.0
    };
    let mut out = Vec::with_capacity(rows);
    for r in 0..rows {
        let y = b_min.y + (r as f64 + 0.5) * step;
        let mut runs: Vec<Run> = vec![];
        for c in 0..cols {
            let p = power_of(d[r * cols + c]);
            if p <= 0.0 {
                continue;
            }
            let (x0, x1) = (b_min.x + c as f64 * step, b_min.x + (c + 1) as f64 * step);
            match runs.last_mut() {
                Some(last) if (last.x1 - x0).abs() < 1e-9 && (last.power - p).abs() < 1e-9 => last.x1 = x1,
                _ => runs.push(Run { x0, x1, power: p }),
            }
        }
        if !runs.is_empty() {
            out.push(Row { y, runs });
        }
    }
    out
}
