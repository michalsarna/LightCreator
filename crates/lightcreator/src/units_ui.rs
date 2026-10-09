//! Drag-value widgets that store millimetres but show and edit the profile's display units.
use eframe::egui::{self, Response};
use lc_core::Units;

fn drag(ui: &mut egui::Ui, v: &mut f64, k: f64, suffix: &str, decimals: usize, speed_mm: f64, range: Option<(f64, f64)>) -> Response {
    let mut d = *v / k;
    let mut w = egui::DragValue::new(&mut d).speed(speed_mm / k).suffix(suffix).max_decimals(decimals);
    if let Some((lo, hi)) = range {
        w = w.range(lo / k..=hi / k);
    }
    let r = ui.add(w);
    if r.changed() {
        *v = d * k;
    }
    r
}

/// A length stored in millimetres.
pub fn drag_len(ui: &mut egui::Ui, u: Units, v: &mut f64, speed_mm: f64, range: Option<(f64, f64)>) -> Response {
    drag(ui, v, u.to_mm(1.0), u.suffix(), u.decimals(), speed_mm, range)
}

/// A speed stored in mm/s.
pub fn drag_speed_s(ui: &mut egui::Ui, u: Units, v: &mut f64, speed_mm: f64, range: Option<(f64, f64)>) -> Response {
    drag(ui, v, u.to_mm(1.0), u.speed_s_suffix(), u.decimals(), speed_mm, range)
}

/// A speed stored in mm/min.
pub fn drag_speed_min(ui: &mut egui::Ui, u: Units, v: &mut f64, speed_mm: f64, range: Option<(f64, f64)>) -> Response {
    drag(ui, v, u.to_mm(1.0), u.speed_min_suffix(), u.decimals(), speed_mm, range)
}

/// Format a length given in millimetres.
pub fn fmt_len(u: Units, mm: f64, decimals_mm: usize) -> String {
    match u {
        Units::Mm => format!("{:.*}{}", decimals_mm, mm, u.suffix()),
        Units::Inch => format!("{:.3}{}", u.from_mm(mm), u.suffix()),
    }
}

/// Format a speed given in mm/min.
pub fn fmt_speed_min(u: Units, mm_min: f64) -> String {
    format!("{:.0}{}", u.from_mm(mm_min), u.speed_min_suffix())
}
