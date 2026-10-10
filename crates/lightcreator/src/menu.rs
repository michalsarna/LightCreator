//! One menu definition shared by the in-window egui menu bar (Windows / Linux)
//! and the native macOS menu bar (`native_menu.rs`).
use crate::i18n::Lang;
use crate::theme::Scheme;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Act {
    New,
    Open,
    Save,
    SaveAs,
    ImportSvg,
    ExportSvg,
    ExportGcode,
    ExportCam,
    Quit,
    Undo,
    Redo,
    Copy,
    Paste,
    Duplicate,
    Delete,
    SelectAll,
    Align(u8),
    CenterOnBed,
    FlipH,
    FlipV,
    RotCw,
    RotCcw,
    ToFront,
    ToBack,
    ToPath,
    ToCurves,
    EditNodes,
    Group,
    Ungroup,
    Lock,
    Unlock,
    ImportAi,
    OnlineClipart,
    TraceImage,
    AdjustImage,
    BoolUnion,
    BoolIntersect,
    BoolSubtract,
    BoolXor,
    OffsetShape,
    RoundCorners,
    QuickGuide,
    OpenGithub,
    ImportImage,
    GridArray,
    ToggleGrid,
    ToggleSnap,
    TogglePreview,
    FitBed,
    FitAll,
    ZoomIn,
    ZoomOut,
    ToggleAutoFit,
    DeviceSettings,
    SwitchDevice,
    MaterialLibrary,
    CameraOverlay,
    GridOptions,
    PreviewWindow,
    CameraView,
    ShareSer2net,
    ToggleOverlay,
    Frame,
    StartJob,
    About,
    SetLang(Lang),
    SetScheme(Scheme),
}

#[derive(Clone)]
pub enum Entry {
    /// Action, English label, accelerator in muda syntax (`CmdOrCtrl+O`) and display text (`Ctrl+O`).
    Item(Act, &'static str, Option<(&'static str, &'static str)>),
    Sep,
    Sub(&'static str, Vec<Entry>),
}

use Entry::{Item, Sep, Sub};

const fn key(accel: &'static str, shown: &'static str) -> Option<(&'static str, &'static str)> {
    Some((accel, shown))
}

/// Top-level menus: (English title, entries). The macOS app menu is added by `native_menu`.
pub fn menus() -> Vec<(&'static str, Vec<Entry>)> {
    let mut align = vec![];
    for (i, n) in ["Align left", "Align centre (H)", "Align right", "Align top", "Align centre (V)", "Align bottom"].into_iter().enumerate() {
        align.push(Item(Act::Align(i as u8), n, None));
    }
    let mut file = vec![
        Item(Act::New, "New", key("CmdOrCtrl+N", "Ctrl+N")),
        Item(Act::Open, "Open…", key("CmdOrCtrl+O", "Ctrl+O")),
        Item(Act::Save, "Save", key("CmdOrCtrl+S", "Ctrl+S")),
        Item(Act::SaveAs, "Save as…", key("CmdOrCtrl+Shift+S", "Ctrl+Shift+S")),
        Sep,
        Sub(
            "Import",
            vec![
                Item(Act::ImportSvg, "Import SVG…", key("CmdOrCtrl+I", "Ctrl+I")),
                Item(Act::ImportImage, "Import image…", None),
                Item(Act::ImportAi, "Import Adobe Illustrator / PDF…", None),
                Item(Act::OnlineClipart, "Online clipart…", None),
            ],
        ),
        Sub(
            "Export",
            vec![
                Item(Act::ExportSvg, "Export SVG…", None),
                Item(Act::ExportGcode, "Export G-code…", None),
                Item(Act::ExportCam, "Export PLT / DXF…", None),
            ],
        ),
    ];
    if !cfg!(target_os = "macos") {
        file.push(Sep);
        file.push(Item(Act::Quit, "Quit", key("CmdOrCtrl+Q", "Ctrl+Q")));
    }
    let mut help = vec![Item(Act::QuickGuide, "Quick guide", key("F1", "F1")), Item(Act::OpenGithub, "Project page on GitHub", None)];
    if !cfg!(target_os = "macos") {
        help.push(Sep);
        help.push(Item(Act::About, "About", None));
    }
    let mut laser = vec![
        Item(Act::SwitchDevice, "Switch device…", None),
        Item(Act::MaterialLibrary, "Material library…", None),
        Item(Act::DeviceSettings, "Device settings…", None),
        Item(Act::Frame, "Frame", None),
        Item(Act::StartJob, "Start job", None),
    ];
    // Sharing a local serial port with ser2net is a Linux feature.
    if cfg!(target_os = "linux") {
        laser.push(Sep);
        laser.push(Item(Act::ShareSer2net, "Share over network (ser2net)…", None));
    }
    vec![
        ("File", file),
        (
            "Edit",
            vec![
                Item(Act::Undo, "Undo", key("CmdOrCtrl+Z", "Ctrl+Z")),
                Item(Act::Redo, "Redo", key("CmdOrCtrl+Shift+Z", "Ctrl+Y")),
                Sep,
                Item(Act::Copy, "Copy", key("CmdOrCtrl+C", "Ctrl+C")),
                Item(Act::Paste, "Paste", key("CmdOrCtrl+V", "Ctrl+V")),
                Item(Act::Duplicate, "Duplicate", key("CmdOrCtrl+D", "Ctrl+D")),
                Item(Act::Delete, "Delete", key("Backspace", "Del")),
                Item(Act::SelectAll, "Select all", key("CmdOrCtrl+A", "Ctrl+A")),
            ],
        ),
        (
            "Arrange",
            [
                align,
                vec![
                    Sep,
                    Item(Act::CenterOnBed, "Centre on bed", None),
                    Item(Act::FlipH, "Flip horizontal", None),
                    Item(Act::FlipV, "Flip vertical", None),
                    Item(Act::RotCw, "Rotate 90° CW", None),
                    Item(Act::RotCcw, "Rotate 90° CCW", None),
                    Sep,
                    Item(Act::Group, "Group", key("CmdOrCtrl+G", "Ctrl+G")),
                    Item(Act::Ungroup, "Ungroup", key("CmdOrCtrl+Shift+G", "Ctrl+Shift+G")),
                    Item(Act::Lock, "Lock", key("CmdOrCtrl+L", "Ctrl+L")),
                    Item(Act::Unlock, "Unlock", key("CmdOrCtrl+Shift+L", "Ctrl+Shift+L")),
                    Sep,
                    Item(Act::ToFront, "Bring to front", None),
                    Item(Act::ToBack, "Send to back", None),
                    Item(Act::ToPath, "Convert to path", None),
                    Item(Act::EditNodes, "Edit nodes", key("N", "N")),
                    Item(Act::ToCurves, "Convert to curves", None),
                    Sep,
                    Item(Act::BoolUnion, "Union", None),
                    Item(Act::BoolIntersect, "Intersection", None),
                    Item(Act::BoolSubtract, "Subtract", None),
                    Item(Act::BoolXor, "Exclusive or", None),
                    Item(Act::OffsetShape, "Offset shape…", None),
                    Item(Act::RoundCorners, "Round corners…", None),
                    Item(Act::AdjustImage, "Adjust image…", None),
                    Item(Act::TraceImage, "Trace image…", None),
                    Sep,
                    Item(Act::GridArray, "Grid array…", None),
                ],
            ]
            .concat(),
        ),
        (
            "View",
            vec![
                Item(Act::ToggleGrid, "Grid", None),
                Item(Act::ToggleSnap, "Snap to grid", None),
                Item(Act::TogglePreview, "Toolpath preview", None),
                Item(Act::PreviewWindow, "Preview window…", None),
                Item(Act::ZoomIn, "Zoom in", key("CmdOrCtrl+=", "Ctrl++")),
                Item(Act::ZoomOut, "Zoom out", key("CmdOrCtrl+-", "Ctrl+-")),
                Item(Act::FitBed, "Fit bed to window", key("CmdOrCtrl+0", "Ctrl+0")),
                Item(Act::FitAll, "Fit all objects", key("CmdOrCtrl+9", "Ctrl+9")),
                Item(Act::ToggleAutoFit, "Auto-fit work area to window", None),
                Item(Act::CameraView, "Camera view", None),
                Item(Act::ToggleOverlay, "Show camera overlay", None),
                Item(Act::CameraOverlay, "Camera overlay…", None),
            ],
        ),
        ("Laser", laser),
        (
            "Settings",
            vec![
                Item(Act::GridOptions, "View options…", None),
                Sep,
                Sub("Language", Lang::ALL.iter().map(|l| Item(Act::SetLang(*l), l.name(), None)).collect()),
                Sub("Colour scheme", Scheme::ALL.iter().map(|s| Item(Act::SetScheme(*s), s.label(), None)).collect()),
            ],
        ),
        ("Help", help),
    ]
}
