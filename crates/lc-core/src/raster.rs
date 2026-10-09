//! Image engraving: sample the bitmap on a scan grid, dither it and turn each row into laser runs.
use crate::doc::*;
use crate::geom::*;
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

const MAX_CELLS: usize = 20_000_000;

/// Error diffusion kernels: (dx, dy, weight) with a common divisor.
fn kernel(d: Dither) -> (&'static [(i32, i32, f32)], f32) {
    match d {
        Dither::Jarvis => (
            &[(1, 0, 7.0), (2, 0, 5.0), (-2, 1, 3.0), (-1, 1, 5.0), (0, 1, 7.0), (1, 1, 5.0), (2, 1, 3.0), (-2, 2, 1.0), (-1, 2, 3.0), (0, 2, 5.0), (1, 2, 3.0), (2, 2, 1.0)],
            48.0,
        ),
        Dither::Stucki => (
            &[(1, 0, 8.0), (2, 0, 4.0), (-2, 1, 2.0), (-1, 1, 4.0), (0, 1, 8.0), (1, 1, 4.0), (2, 1, 2.0), (-2, 2, 1.0), (-1, 2, 2.0), (0, 2, 4.0), (1, 2, 2.0), (2, 2, 1.0)],
            42.0,
        ),
        Dither::Atkinson => (&[(1, 0, 1.0), (2, 0, 1.0), (-1, 1, 1.0), (0, 1, 1.0), (1, 1, 1.0), (0, 2, 1.0)], 8.0),
        _ => (&[(1, 0, 7.0), (-1, 1, 3.0), (0, 1, 5.0), (1, 1, 1.0)], 16.0),
    }
}

const BAYER8: [[u8; 8]; 8] = [
    [0, 32, 8, 40, 2, 34, 10, 42],
    [48, 16, 56, 24, 50, 18, 58, 26],
    [12, 44, 4, 36, 14, 46, 6, 38],
    [60, 28, 52, 20, 62, 30, 54, 22],
    [3, 35, 11, 43, 1, 33, 9, 41],
    [51, 19, 59, 27, 49, 17, 57, 25],
    [15, 47, 7, 39, 13, 45, 5, 37],
    [63, 31, 55, 23, 61, 29, 53, 21],
];

/// Turn darkness values (0..1, row-major) into levels: 0 = off, otherwise a fraction of the power range.
fn dither(d: &mut [f32], cols: usize, rows: usize, mode: Dither) {
    match mode {
        Dither::Grayscale => {}
        Dither::Threshold => d.iter_mut().for_each(|v| *v = if *v >= 0.5 { 1.0 } else { 0.0 }),
        Dither::Ordered => {
            for r in 0..rows {
                for c in 0..cols {
                    let t = (BAYER8[r % 8][c % 8] as f32 + 0.5) / 64.0;
                    let v = &mut d[r * cols + c];
                    *v = if *v > t { 1.0 } else { 0.0 };
                }
            }
        }
        _ => {
            let (k, div) = kernel(mode);
            for r in 0..rows {
                for c in 0..cols {
                    let old = d[r * cols + c];
                    let new = if old >= 0.5 { 1.0 } else { 0.0 };
                    d[r * cols + c] = new;
                    let err = old - new;
                    if err == 0.0 {
                        continue;
                    }
                    for &(dx, dy, w) in k {
                        let (cc, rr) = (c as i32 + dx, r as i32 + dy);
                        if cc >= 0 && (cc as usize) < cols && (rr as usize) < rows {
                            d[rr as usize * cols + cc as usize] += err * w / div;
                        }
                    }
                }
            }
        }
    }
}

/// Scan rows for an image shape. `layer` supplies interval, powers and the dither mode.
pub fn rows_for(shape: &Shape, im: &ImageData, layer: &Layer) -> Vec<Row> {
    let Some(b) = shape.bounds() else { return vec![] };
    let Some(inv) = shape.xf.inverse() else { return vec![] };
    let mut step = layer.interval.max(0.02);
    loop {
        let (c, r) = ((b.width() / step).ceil() as usize, (b.height() / step).ceil() as usize);
        if c.saturating_mul(r) <= MAX_CELLS {
            break;
        }
        step *= 1.5;
    }
    let cols = ((b.width() / step).ceil() as usize).max(1);
    let rows = ((b.height() / step).ceil() as usize).max(1);
    let mut d = vec![0.0f32; cols * rows];
    for r in 0..rows {
        let y = b.min.y + (r as f64 + 0.5) * step;
        for c in 0..cols {
            let x = b.min.x + (c as f64 + 0.5) * step;
            let l = inv.apply(Pt::new(x, y));
            d[r * cols + c] = im.darkness_at(l.x, l.y);
        }
    }
    dither(&mut d, cols, rows, layer.dither);
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
        let y = b.min.y + (r as f64 + 0.5) * step;
        let mut runs: Vec<Run> = vec![];
        for c in 0..cols {
            let p = power_of(d[r * cols + c]);
            if p <= 0.0 {
                continue;
            }
            let (x0, x1) = (b.min.x + c as f64 * step, b.min.x + (c + 1) as f64 * step);
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
