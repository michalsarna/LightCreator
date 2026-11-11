//! No-go zones: parts of the work area the machine must never enter (a clamp, a fixture, a mirror). A move that
//! touches an enabled zone stops the job from being sent, framed or exported.
use crate::doc::*;
use crate::geom::*;

pub use base_core::zones::{route, segment_hits};

/// Does the zone apply at these heights? A zone without a Z range always does; so does any zone on a machine
/// without a Z axis. With a range, the move has to be at a height inside it: `levels` are the heights the move may
/// happen at, and when they are not known (empty) the zone is assumed to be in the way.
pub fn applies_at(dev: &Device, zone: &Zone, levels: &[f64]) -> bool {
    match zone.z_range {
        Some((lo, hi)) if dev.axes.has_z && !levels.is_empty() => levels.iter().any(|z| *z >= lo.min(hi) - 1e-9 && *z <= lo.max(hi) + 1e-9),
        _ => true,
    }
}

/// The enabled zones the move touches, by index. `levels` are the Z heights the move may happen at (empty = unknown).
pub fn zones_hit(dev: &Device, a: Pt, b: Pt, levels: &[f64]) -> Vec<usize> {
    dev.zones.iter().enumerate().filter(|(_, z)| z.enabled && applies_at(dev, z, levels) && segment_hits(&z.shape, a, b)).map(|(i, _)| i).collect()
}
