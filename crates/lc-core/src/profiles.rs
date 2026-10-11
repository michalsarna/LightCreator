//! Device profiles: general starting points, copying, and a file format for moving one profile between computers.
//! A file is data from outside: it is checked strictly and nothing in it is trusted.
use crate::camimg::CameraKind;
use crate::doc::*;

/// A starting point for a new device. Only general machines: the numbers are typical, not those of a brand.
pub struct Template {
    pub id: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
}

pub const TEMPLATES: &[Template] = &[
    Template { id: "grbl-diode-400", name: "GRBL diode laser 400 × 400", desc: "A GRBL 1.1 diode laser on a 400 × 400 mm bed, front-left zero, dynamic power." },
    Template { id: "grbl-co2-300x200", name: "GRBL CO2 laser 300 × 200", desc: "A GRBL 1.1 CO2 laser on a 300 × 200 mm bed, front-left zero." },
    Template { id: "grbl-k40-300x200", name: "K40 conversion (GRBL) 300 × 200", desc: "A K40 CO2 laser with a GRBL controller board: 300 × 200 mm, zero at the back-left (top-left) corner." },
    Template { id: "marlin-diode", name: "Marlin diode laser 400 × 400", desc: "A Marlin firmware diode laser (laser feature, S 0–255) on a 400 × 400 mm bed." },
];

/// The profile of a template, or the default one for an unknown id. These are starting points: check them against the
/// machine's own documentation.
pub fn template(id: &str) -> Device {
    let mut d = Device::default();
    match id {
        "grbl-co2-300x200" => {
            d.name = "GRBL CO2 laser 300 × 200".into();
            d.area = Area { bed_w: 300.0, bed_h: 200.0, ..Area::default() };
            d.heads[0].laser = LaserKind::Co2;
        }
        "grbl-k40-300x200" => {
            d.name = "K40 conversion (GRBL) 300 × 200".into();
            d.area = Area { bed_w: 300.0, bed_h: 200.0, ..Area::default() };
            d.axes.origin = Origin::BackLeft;
            d.heads[0].laser = LaserKind::Co2;
            d.template_notes = "A K40 with a GRBL board usually homes to the top-left corner (here: back-left). Check the work area and the direction of the axes on your machine, and run a test with the laser off.".into();
        }
        "marlin-diode" => {
            d.name = "Marlin diode laser 400 × 400".into();
            d.controller = Controller::Marlin;
            d.heads[0].s_max = 255.0;
        }
        _ => {
            d.name = "GRBL diode laser 400 × 400".into();
        }
    }
    d
}

/// A profile that starts a new life (a copy, an import): the work times begin at zero, the counters keep their names
/// and limits.
pub fn without_history(d: &mut Device) {
    d.maintenance.total_secs = 0.0;
    for c in &mut d.maintenance.counters {
        c.secs = 0.0;
    }
}

/// A name for a copy: "Name (copy)", "Name (copy 2)"... that none of `taken` has.
pub fn copy_name(name: &str, taken: &[String]) -> String {
    let base = name.trim();
    let first = format!("{base} (copy)");
    if !taken.contains(&first) {
        return first;
    }
    (2..).map(|n| format!("{base} (copy {n})")).find(|c| !taken.contains(c)).unwrap_or(first)
}

/// The most characters of the notes of a profile.
pub const MAX_NOTES: usize = 20_000;

/// The version of the file format written now.
pub const PROFILE_FORMAT: u32 = 1;
/// The biggest file read.
pub const MAX_PROFILE_FILE: usize = 600_000;

#[derive(serde::Serialize, serde::Deserialize)]
struct ProfileFile {
    #[serde(default)]
    format: u32,
    #[serde(default)]
    app_version: String,
    profile: Device,
}

/// The text of a file that holds `dev` (without what only makes sense on this computer: the port).
pub fn export_profile(dev: &Device, app_version: &str) -> String {
    let mut d = dev.clone();
    d.connection.port.clear();
    serde_json::to_string_pretty(&ProfileFile { format: PROFILE_FORMAT, app_version: app_version.to_string(), profile: d }).unwrap_or_default()
}

fn text_ok(s: &str, max: usize) -> bool {
    s.chars().count() <= max && !s.chars().any(|c| c.is_control() && c != '\n')
}

fn between(v: f64, lo: f64, hi: f64) -> bool {
    v.is_finite() && v >= lo && v <= hi
}

/// Is the profile within sane limits? The first thing wrong, in words.
pub fn validate(d: &Device) -> Result<(), String> {
    let bad = |what: &str| Err(format!("{what} is out of range"));
    if d.name.trim().is_empty() || !text_ok(&d.name, 80) {
        return Err("the name is empty, too long or has control characters".into());
    }
    if !between(d.area.bed_w, 10.0, 5000.0) || !between(d.area.bed_h, 10.0, 5000.0) {
        return bad("the work area");
    }
    let m = d.area.margins;
    if ![m.left, m.top, m.right, m.bottom].iter().all(|v| between(*v, 0.0, 5000.0)) {
        return bad("a margin");
    }
    if let Some(l) = d.area.soft_limits {
        if ![l.x0, l.y0, l.x1, l.y1].iter().all(|v| between(*v, -5000.0, 10000.0)) {
            return bad("a software limit");
        }
    }
    if !between(d.speeds.travel_speed, 1.0, 100_000.0) || !between(d.speeds.max_cut_speed, 0.0, 100_000.0) || !between(d.speeds.acceleration, 0.0, 100_000.0) {
        return bad("a speed");
    }
    if !between(d.jog_step, 0.001, 1000.0) || !between(d.jog_feed, 1.0, 100_000.0) {
        return bad("the jog setting");
    }
    if d.heads.is_empty() || d.heads.len() > 8 {
        return bad("the number of laser heads");
    }
    for h in &d.heads {
        if !text_ok(&h.name, 60) || h.tool > 99 || !between(h.s_max, 1.0, 1_000_000.0) || !between(h.frame_power, 0.0, 100.0) || !between(h.frame_speed, 0.1, 10_000.0) || !between(h.focus_power, 0.0, 100.0) {
            return bad("a laser head setting");
        }
        if ![h.spot_x, h.spot_y, h.optical_power_w, h.pwm_freq_hz, h.pwm_max_freq_hz, h.pulse_width_us, h.pulse_min_us, h.pulse_max_us].iter().all(|v| between(*v, 0.0, 1e7)) || h.wavelength_nm > 100_000 || !between(h.pointer_dx, -1000.0, 1000.0) || !between(h.pointer_dy, -1000.0, 1000.0) {
            return bad("a laser head value");
        }
    }
    if d.has_duplicate_tools() {
        return Err("two laser heads share a tool number".into());
    }
    let c = &d.connection;
    if c.baud < 300 || c.baud > 4_000_000 || !text_ok(&c.host, 253) || !text_ok(&c.port, 200) || !text_ok(&c.usb_id, 9) || (!c.usb_id.is_empty() && parse_usb_id(&c.usb_id).is_none()) {
        return bad("a connection setting");
    }
    if d.axes.z_min > d.axes.z_max || !between(d.axes.z_min, -10_000.0, 10_000.0) || !between(d.axes.z_max, -10_000.0, 10_000.0) {
        return bad("the Z travel");
    }
    if !d.camera_view.is_valid() {
        return bad("the camera view");
    }
    if d.cameras.len() > 8 || d.active_camera > 8 {
        return bad("the number of cameras");
    }
    for c in &d.cameras {
        let kind_ok = match &c.kind {
            CameraKind::Url(u) => text_ok(u, 500),
            CameraKind::Local(i) => *i < 64,
        };
        let i = c.image.clamped();
        if !text_ok(&c.name, 60) || !kind_ok || c.rotation > 3 || i != c.image || !c.lens.is_sane() || c.reference.len() > 32 {
            return bad("a camera setting");
        }
        if c.reference.iter().any(|r| ![r.px.0, r.px.1, r.mm.0, r.mm.1].iter().all(|v| between(*v, -1e7, 1e7))) {
            return bad("a reference point");
        }
        // An address that is not usable cannot be switched on.
        if c.enabled && !c.is_valid() {
            return Err("a camera is switched on but its address is not valid".into());
        }
    }
    if d.layer_speeds.len() > 30 || !d.layer_speeds.iter().all(|v| between(*v, 0.0, 100_000.0)) {
        return bad("a layer speed");
    }
    if d.macros.len() > 50 || d.zones.len() > 100 || d.materials.len() > 5000 {
        return Err("too many macros, zones or materials".into());
    }
    if d.maintenance.counters.len() > 50 || !d.maintenance.counters.iter().all(|c| text_ok(&c.name, 80) && between(c.secs, 0.0, 1e10) && c.threshold_hours.map_or(true, |h| between(h, 0.0, 1e7))) || !between(d.maintenance.total_secs, 0.0, 1e10) {
        return bad("a maintenance counter");
    }
    if !text_ok(&d.notes, MAX_NOTES) {
        return Err("the notes are too long or have control characters".into());
    }
    if !text_ok(&d.template_notes, 5000) || !between(d.rotary.diameter, 0.0, 5000.0) || !between(d.rotary.roller_diameter, 0.0, 5000.0) || !between(d.rotary.microsteps_per_rev, 0.0, 10_000_000.0) {
        return bad("a note or the rotary");
    }
    if let Some((n, why)) = d.gcode.hooks.problem().map(|(n, w)| (n.to_string(), w)) {
        return Err(format!("hook {n}: {why}"));
    }
    if !d.gcode.invalid_fields().is_empty() {
        return Err("own G-code is not fit to be used".into());
    }
    if let Some((n, why)) = crate::hooks::validate_macros(d) {
        return Err(format!("macro {n}: {why}"));
    }
    if !between(d.gcode.arc_tolerance, 0.0001, 10.0) || !between(d.gcode.flatten_tolerance, 0.001, 10.0) {
        return bad("a tolerance");
    }
    for z in &d.zones {
        let ok = match z.shape {
            ZoneShape::Rect { x, y, w, h } => [x, y, w, h].iter().all(|v| between(*v, -100_000.0, 100_000.0)),
            ZoneShape::Circle { x, y, r } => [x, y, r].iter().all(|v| between(*v, -100_000.0, 100_000.0)),
        };
        if !ok || !text_ok(&z.name, 80) {
            return bad("a zone");
        }
    }
    Ok(())
}

/// Read a profile file. The format version must not be newer than this program knows; a bare profile (an older
/// export) is accepted. The port name is never taken from the file.
pub fn import_profile(text: &str) -> Result<Device, String> {
    if text.len() > MAX_PROFILE_FILE {
        return Err(format!("the file is larger than {} KB", MAX_PROFILE_FILE / 1000));
    }
    let value: serde_json::Value = serde_json::from_str(text).map_err(|_| "this is not a profile file".to_string())?;
    let mut dev: Device = if value.get("profile").is_some() {
        let f: ProfileFile = serde_json::from_value(value).map_err(|e| format!("the profile cannot be read: {e}"))?;
        if f.format > PROFILE_FORMAT {
            return Err(format!("the file is of format {}, made by a newer version ({}); update the program", f.format, f.app_version));
        }
        f.profile
    } else {
        serde_json::from_value(value).map_err(|e| format!("the profile cannot be read: {e}"))?
    };
    dev.connection.port.clear();
    validate(&dev)?;
    without_history(&mut dev);
    Ok(dev)
}
