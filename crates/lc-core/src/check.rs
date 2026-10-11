//! Checks of a job before it is sent, framed or exported. Errors block the action (nothing is clamped silently);
//! warnings can be confirmed.
use crate::doc::*;
use crate::geom::*;

/// Distance (mm) by which something may stick out of an area before it counts as outside.
pub const TOLERANCE: f64 = 0.01;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    /// The job must not be sent, framed or exported.
    Error,
    /// The job can go ahead once the user has confirmed.
    Warning,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IssueKind {
    /// An object (or the framed box) is outside the usable area: beyond the work area or in a margin.
    OutsideArea,
    /// Inside the usable area but outside the software limits.
    OutsideLimits,
    /// The toolpath of a layer leaves the allowed area although its objects do not (overscan, for instance).
    MoveOutside,
    /// An output layer that has objects burns at 0 % power.
    ZeroPower,
    /// The speed of an output layer is above the maximum cutting speed of the machine.
    SpeedAboveMax,
    /// An output layer has objects but they produce no toolpath (open outlines on a fill layer, for example).
    NothingToBurn,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Issue {
    pub level: Level,
    pub kind: IssueKind,
    /// The objects concerned (empty when the issue is about a layer or the framed box).
    pub shapes: Vec<u64>,
    pub layer: Option<usize>,
}

fn outside(r: Rect, area: Rect) -> bool {
    r.min.x < area.min.x - TOLERANCE || r.min.y < area.min.y - TOLERANCE || r.max.x > area.max.x + TOLERANCE || r.max.y > area.max.y + TOLERANCE
}

fn point_outside(p: Pt, area: Rect) -> bool {
    outside(Rect { min: p, max: p }, area)
}

/// Why `r` may not be used, if it may not.
fn rect_issue(dev: &Device, r: Rect) -> Option<IssueKind> {
    if outside(r, dev.usable_rect()) {
        Some(IssueKind::OutsideArea)
    } else if outside(r, dev.allowed_rect()) {
        Some(IssueKind::OutsideLimits)
    } else {
        None
    }
}

/// The check of a frame: the box that is traced must lie inside the allowed area.
pub fn check_frame(dev: &Device, bounds: Rect) -> Vec<Issue> {
    rect_issue(dev, bounds).map(|kind| Issue { level: Level::Error, kind, shapes: vec![], layer: None }).into_iter().collect()
}

/// The check of a whole job: objects and toolpaths of the output layers, and the settings of those layers.
pub fn job_check(doc: &Document) -> Vec<Issue> {
    let dev = &doc.device;
    let mut out = vec![];
    // Objects on output layers that lie outside, one issue per kind with all of them listed.
    for kind in [IssueKind::OutsideArea, IssueKind::OutsideLimits] {
        let ids: Vec<u64> = doc.shapes.iter().filter(|s| doc.layers[s.layer].output).filter(|s| s.bounds().and_then(|b| rect_issue(dev, b)) == Some(kind)).map(|s| s.id).collect();
        if !ids.is_empty() {
            out.push(Issue { level: Level::Error, kind, shapes: ids, layer: None });
        }
    }
    let job = crate::gcode::generate(doc);
    let area = dev.allowed_rect();
    let home = dev.home_point();
    let mut moved_out: Vec<usize> = vec![];
    for m in &job.moves {
        // The way back to the machine's zero may end outside the usable area (the zero is a corner).
        if m.b.dist(home) < 1e-6 && !m.laser {
            continue;
        }
        if point_outside(m.b, area) && !moved_out.contains(&m.layer) {
            moved_out.push(m.layer);
        }
    }
    for li in moved_out {
        // Reported once per layer, and not again when the objects of that layer are already named above.
        let named = out.iter().any(|i| i.level == Level::Error && doc.shapes.iter().any(|s| s.layer == li && i.shapes.contains(&s.id)));
        if !named {
            out.push(Issue { level: Level::Error, kind: IssueKind::MoveOutside, shapes: vec![], layer: Some(li) });
        }
    }
    for li in doc.layer_order() {
        let layer = &doc.layers[li];
        if !layer.output {
            continue;
        }
        let ids: Vec<u64> = doc.shapes.iter().filter(|s| s.layer == li).map(|s| s.id).collect();
        if ids.is_empty() {
            continue;
        }
        let max = dev.speeds.max_cut_speed;
        if max > 0.0 && layer.speed * 60.0 > max + 1e-6 {
            out.push(Issue { level: Level::Warning, kind: IssueKind::SpeedAboveMax, shapes: ids.clone(), layer: Some(li) });
        }
        if layer.power <= 0.0 {
            out.push(Issue { level: Level::Warning, kind: IssueKind::ZeroPower, shapes: ids.clone(), layer: Some(li) });
        }
        if !job.moves.iter().any(|m| m.layer == li && m.laser) && layer.power > 0.0 {
            out.push(Issue { level: Level::Warning, kind: IssueKind::NothingToBurn, shapes: ids, layer: Some(li) });
        }
    }
    out
}

/// Does the list hold an error?
pub fn has_error(issues: &[Issue]) -> bool {
    issues.iter().any(|i| i.level == Level::Error)
}
