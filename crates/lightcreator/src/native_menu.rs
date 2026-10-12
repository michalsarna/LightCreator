//! The native macOS menu bar of LightCreator: the menus come from `menu.rs`, the building from base-ui.
use crate::menu::{menus, Act, MAX_RECENT};

pub use base_ui::native_menu::set_dock_icon;

pub type NativeMenu = base_ui::native_menu::NativeMenu<Act>;

fn is_check(a: Act) -> bool {
    matches!(a, Act::ToggleGrid | Act::ToggleSnap | Act::ToggleSnapObjects | Act::TogglePreview | Act::SetLang(_) | Act::SetScheme(_))
}

/// Accelerators the system menu owns. Editing shortcuts (copy, paste, delete...) stay with egui so text
/// fields keep working; the egui shortcut handler skips the ones listed here.
pub fn owns_shortcut(a: Act) -> bool {
    matches!(a, Act::New | Act::Open | Act::Save | Act::SaveAs | Act::ImportSvg)
}

pub static CONFIG: base_ui::native_menu::NativeConfig<Act> = base_ui::native_menu::NativeConfig {
    app_name: "LightCreator",
    about_label: "About LightCreator",
    about: Act::About,
    quit: Act::Quit,
    menus,
    act_icon: crate::icons::act_icon,
    is_check,
    owns_shortcut,
    open_recent: |i| Act::OpenRecent(i),
    clear_recent: Act::ClearRecent,
    max_recent: MAX_RECENT as u8,
};

/// Install the menu bar.
pub fn install(ctx: &eframe::egui::Context) -> NativeMenu {
    NativeMenu::install(ctx, &CONFIG)
}
