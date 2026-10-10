//! Icons. Every picture in the interface is an SVG file from `assets/ui-icons` (Lucide, plus two drawn in the
//! same style), embedded in the program and tinted with the colour of the current theme.
use crate::app::Tool;
use crate::menu::Act;
use eframe::egui::{self, Color32, Rect, Ui};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

// Every file of `assets/ui-icons`, collected by `build.rs`.
include!(concat!(env!("OUT_DIR"), "/ui_icon_files.rs"));

/// SVG bytes by name; strokes use `currentColor`, which is replaced by white so a tint can recolour the icon.
fn table() -> &'static HashMap<&'static str, Arc<[u8]>> {
    static T: OnceLock<HashMap<&'static str, Arc<[u8]>>> = OnceLock::new();
    T.get_or_init(|| {
        FILES
            .iter()
            .map(|(n, b)| {
                let text = String::from_utf8_lossy(b).replace("currentColor", "#ffffff");
                (*n, Arc::from(text.into_bytes().into_boxed_slice()))
            })
            .collect()
    })
}

/// Names of all embedded icons.
#[cfg(test)]
pub fn all_names() -> Vec<&'static str> {
    FILES.iter().map(|(n, _)| *n).collect()
}

/// The icon as an image of `size` points, tinted with `color`. An unknown name gives `None`.
pub fn image(name: &str, size: f32, color: Color32) -> Option<egui::Image<'static>> {
    let (key, bytes) = table().get_key_value(name)?;
    Some(egui::Image::from_bytes(format!("bytes://lc-icon/{key}.svg"), bytes.clone()).fit_to_exact_size(egui::vec2(size, size)).tint(color))
}

/// Paint the icon filling `rect`.
pub fn paint(ui: &Ui, rect: Rect, name: &str, color: Color32) {
    if let Some(img) = image(name, rect.width().min(rect.height()), color) {
        img.paint_at(ui, rect);
    }
}

pub fn tool_icon(t: Tool) -> &'static str {
    match t {
        Tool::Select => "mouse-pointer-2",
        Tool::Node => "spline",
        Tool::Rect => "square",
        Tool::Ellipse => "circle",
        Tool::Triangle => "triangle",
        Tool::Star => "star",
        Tool::Polygon => "hexagon",
        Tool::Line => "minus",
        Tool::Pen => "pen-tool",
        Tool::Text => "type",
        Tool::Pan => "hand",
        Tool::Zoom => "zoom-in",
        Tool::ZoomOut => "zoom-out",
    }
}

/// Icon of a menu title (File, Edit ...), by its English name.
pub fn menu_title_icon(title: &str) -> Option<&'static str> {
    Some(match title {
        "File" => "file",
        "Edit" => "pencil",
        "Arrange" => "shapes",
        "View" => "eye",
        "Laser" => "zap",
        "Settings" => "settings",
        "Help" => "circle-question-mark",
        _ => return None,
    })
}

/// Icon of a submenu title, by its English name.
pub fn submenu_icon(title: &str) -> Option<&'static str> {
    Some(match title {
        "Import" => "download",
        "Export" => "upload",
        "Language" => "languages",
        "Colour scheme" => "palette",
        _ => return None,
    })
}

pub fn act_icon(a: Act) -> Option<&'static str> {
    Some(match a {
        Act::New => "file-plus",
        Act::Open => "folder-open",
        Act::Save | Act::SaveAs => "save",
        Act::ImportSvg | Act::ImportAi => "file-input",
        Act::ImportImage => "image",
        Act::OnlineClipart => "globe",
        Act::ExportSvg | Act::ExportCam => "file-output",
        Act::ExportGcode => "file-code",
        Act::Quit => "x",
        Act::Undo => "undo-2",
        Act::Redo => "redo-2",
        Act::Copy => "copy",
        Act::Paste => "clipboard-paste",
        Act::Duplicate => "copy-plus",
        Act::Delete => "trash",
        Act::SelectAll => "lasso-select",
        Act::Align(0) => "align-start-vertical",
        Act::Align(1) => "align-center-vertical",
        Act::Align(2) => "align-end-vertical",
        Act::Align(3) => "align-start-horizontal",
        Act::Align(4) => "align-center-horizontal",
        Act::Align(_) => "align-end-horizontal",
        Act::CenterOnBed => "focus",
        Act::FlipH => "flip-horizontal",
        Act::FlipV => "flip-vertical",
        Act::RotCw => "rotate-cw",
        Act::RotCcw => "rotate-ccw",
        Act::ToFront => "bring-to-front",
        Act::ToBack => "send-to-back",
        Act::ToPath | Act::ToCurves | Act::EditNodes => "spline",
        Act::Group => "group",
        Act::Ungroup => "ungroup",
        Act::Lock => "lock",
        Act::Unlock => "lock-open",
        Act::BoolUnion => "squares-unite",
        Act::BoolIntersect => "squares-intersect",
        Act::BoolSubtract => "squares-subtract",
        Act::BoolXor => "squares-exclude",
        Act::OffsetShape => "maximize",
        Act::RoundCorners => "square-round-corner",
        Act::QuickGuide => "book-open",
        Act::OpenGithub => "globe",
        Act::GridArray => "grid-3x3",
        Act::ToggleGrid => "grid-3x3",
        Act::ToggleSnap => "magnet",
        Act::TogglePreview => "route",
        Act::PreviewWindow => "play",
        Act::FitBed | Act::ToggleAutoFit => "scan",
        Act::FitAll => "expand",
        Act::ZoomIn => "zoom-in",
        Act::ZoomOut => "zoom-out",
        Act::CameraView => "video",
        Act::ToggleOverlay | Act::CameraOverlay => "layers",
        Act::DeviceSettings => "settings",
        Act::SwitchDevice => "refresh-cw",
        Act::MaterialLibrary => "package",
        Act::Frame => "square-dashed",
        Act::StartJob => "play",
        Act::ShareSer2net => "wifi",
        Act::GridOptions => "sliders-horizontal",
        Act::About => "info",
        Act::TraceImage => "spline",
        Act::AdjustImage => "sliders-horizontal",
        Act::SetLang(_) | Act::SetScheme(_) => return None,
    })
}

/// A leading icon for menu entries: the icon, or a transparent placeholder so that texts line up.
pub fn slot(name: Option<&str>, color: Color32) -> egui::Image<'static> {
    match name.and_then(|n| image(n, 16.0, color)) {
        Some(img) => img,
        None => image("check", 16.0, Color32::TRANSPARENT).unwrap_or_else(|| egui::Image::new(egui::include_image!("../../../assets/icons/icon-16.png"))),
    }
}

/// The original SVG text of an icon (strokes still `currentColor`), for the native macOS menu bar.
#[cfg(target_os = "macos")]
pub fn svg_text(name: &str) -> Option<String> {
    FILES.iter().find(|(n, _)| *n == name).map(|(_, b)| String::from_utf8_lossy(b).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every icon the program asks for must exist in the assets folder (a missing one would draw nothing).
    #[test]
    fn all_requested_icons_exist() {
        let have: std::collections::HashSet<&str> = all_names().into_iter().collect();
        for t in Tool::ALL {
            assert!(have.contains(tool_icon(t.0)), "tool icon {}", tool_icon(t.0));
        }
        for (_, entries) in crate::menu::menus() {
            fn walk(es: &[crate::menu::Entry], have: &std::collections::HashSet<&str>) {
                for e in es {
                    match e {
                        crate::menu::Entry::Item(a, _, _) => {
                            if let Some(n) = act_icon(*a) {
                                assert!(have.contains(n), "act icon {n}");
                            }
                        }
                        crate::menu::Entry::Sub(t, c) => {
                            if let Some(n) = submenu_icon(t) {
                                assert!(have.contains(n), "submenu icon {n}");
                            }
                            walk(c, have);
                        }
                        crate::menu::Entry::Sep => {}
                    }
                }
            }
            walk(&entries, &have);
        }
        for t in ["File", "Edit", "Arrange", "View", "Laser", "Settings", "Help"] {
            assert!(have.contains(menu_title_icon(t).unwrap()), "{t}");
        }
        for n in ["check", "lock", "lock-open", "chevron-up", "chevron-down", "star", "rotate-cw", "rotate-ccw", "scan", "expand", "zoom-in", "zoom-out", "shapes", "squares-unite", "flip-horizontal", "flip-vertical"] {
            assert!(have.contains(n), "{n}");
        }
    }
}
