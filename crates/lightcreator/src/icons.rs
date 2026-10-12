//! Icons. The store of SVG icons, the tool and menu-title icons live in base-ui; this file gives every action of
//! LightCreator its icon.
use crate::menu::Act;

pub use base_ui::icons::*;

pub fn act_icon(a: Act) -> Option<&'static str> {
    Some(match a {
        Act::New => "file-plus",
        Act::Open => "folder-open",
        Act::OpenRecent(_) => "file",
        Act::ClearRecent => "trash",
        Act::Save | Act::SaveAs => "save",
        Act::ImportSvg | Act::ImportAi => "file-input",
        Act::ImportImage => "image",
        Act::OnlineClipart => "globe",
        Act::ExportSvg | Act::ExportCam => "file-output",
        Act::Print => "printer",
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
        Act::CenterEachOther => "target",
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
        Act::ToggleSnapObjects => "target",
        Act::TogglePreview => "route",
        Act::PreviewWindow => "play",
        Act::FitBed | Act::ToggleAutoFit => "scan",
        Act::FitAll => "expand",
        Act::ZoomIn => "zoom-in",
        Act::ZoomOut => "zoom-out",
        Act::CameraView => "video",
        Act::ToggleOverlay | Act::CameraOverlay => "layers",
        Act::ToggleZones => "eye",
        Act::DeviceSettings => "settings",
        Act::SwitchDevice => "refresh-cw",
        Act::Frame => "square-dashed",
        Act::StartJob => "play",
        Act::ShareSer2net => "wifi",
        Act::GridOptions => "sliders-horizontal",
        Act::ProgramOptions => "settings",
        Act::CheckUpdates => "refresh-cw",
        Act::DownloadBackup => "package",
        Act::RestoreBackup => "package-open",
        Act::About => "info",
        Act::TraceImage => "portrait",
        Act::AdjustImage => "sliders-horizontal",
        Act::SetLang(_) | Act::SetScheme(_) => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every icon the program asks for must exist in the assets folder (a missing one would draw nothing).
    #[test]
    fn all_requested_icons_exist() {
        let have: std::collections::HashSet<&str> = all_names().into_iter().collect();
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
                        crate::menu::Entry::Sep | crate::menu::Entry::Recent => {}
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
