//! Raster images for engraving: stored as 8-bit grayscale (white = no burn).
use serde::{Deserialize, Serialize};

mod b64 {
    use base64::Engine;
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S: Serializer>(v: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&base64::engine::general_purpose::STANDARD.encode(v))
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let s = String::deserialize(d)?;
        base64::engine::general_purpose::STANDARD.decode(s).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImageData {
    pub name: String,
    pub px_w: u32,
    pub px_h: u32,
    /// Row-major luminance, `px_w * px_h` bytes.
    #[serde(with = "b64")]
    pub gray: Vec<u8>,
    /// Size on the work area, millimetres.
    pub w: f64,
    pub h: f64,
    /// Swap black and white (engrave the light parts).
    #[serde(default)]
    pub invert: bool,
}

impl ImageData {
    /// Decode a PNG / JPEG / BMP / GIF / WebP file. Transparent pixels become white. The longer side is
    /// `longest_mm` millimetres.
    pub fn from_bytes(bytes: &[u8], name: &str, longest_mm: f64) -> Result<ImageData, String> {
        let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?.to_rgba8();
        let (w, h) = (img.width(), img.height());
        if w == 0 || h == 0 {
            return Err("empty image".into());
        }
        let gray: Vec<u8> = img
            .pixels()
            .map(|p| {
                let [r, g, b, a] = p.0;
                let l = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
                let a = a as f32 / 255.0;
                (l * a + 255.0 * (1.0 - a)).round().clamp(0.0, 255.0) as u8
            })
            .collect();
        let k = longest_mm / w.max(h) as f64;
        Ok(ImageData { name: name.to_string(), px_w: w, px_h: h, gray, w: w as f64 * k, h: h as f64 * k, invert: false })
    }

    /// Darkness (0 = white / nothing, 1 = black) at the local point `(x, y)` millimetres, bilinear.
    pub fn darkness_at(&self, x: f64, y: f64) -> f32 {
        if x < 0.0 || y < 0.0 || x >= self.w || y >= self.h {
            return 0.0;
        }
        let fx = (x / self.w * self.px_w as f64 - 0.5).clamp(0.0, self.px_w as f64 - 1.0);
        let fy = (y / self.h * self.px_h as f64 - 0.5).clamp(0.0, self.px_h as f64 - 1.0);
        let (x0, y0) = (fx.floor() as usize, fy.floor() as usize);
        let (x1, y1) = ((x0 + 1).min(self.px_w as usize - 1), (y0 + 1).min(self.px_h as usize - 1));
        let (tx, ty) = ((fx - x0 as f64) as f32, (fy - y0 as f64) as f32);
        let g = |x: usize, y: usize| self.gray[y * self.px_w as usize + x] as f32;
        let v = g(x0, y0) * (1.0 - tx) * (1.0 - ty) + g(x1, y0) * tx * (1.0 - ty) + g(x0, y1) * (1.0 - tx) * ty + g(x1, y1) * tx * ty;
        let d = 1.0 - v / 255.0;
        if self.invert {
            1.0 - d
        } else {
            d
        }
    }
}
