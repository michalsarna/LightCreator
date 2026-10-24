mod api_server;
mod api_ui;
mod app;
mod check_ui;
mod cam_test_ui;
mod clipart_ui;
mod canvas;
mod device_ui;
mod firmware_ui;
mod focus_ui;
mod gcode_view_ui;
#[cfg(test)]
mod grbl_sim_tests;
mod i18n;
mod guide;
mod help_ui;
mod icons;
mod laser;
mod materials_ui;
mod materials_win;
mod menu;
mod nodes;
#[cfg(target_os = "macos")]
mod native_menu;
mod overlay;
mod panels;
mod plugins_glue;
mod preview_ui;
mod prefs;
mod print_ui;
mod ser2net_ui;
mod shape_ops;
mod stream_ui;
#[cfg(test)]
mod screenshots;
mod session_ui;
mod workspaces;
mod update_ui;
mod backup_ui;
mod window_ui;
mod add_device_ui;

pub use base_ui::{camera, camera_stream, fonts, markdown_ui, theme, units_ui};

const ICON_256: &[u8] = include_bytes!("../../../assets/icons/icon-256.png");
const ICON_512: &[u8] = include_bytes!("../../../assets/icons/icon-512.png");

/// Decode the bundled PNG logo into a texture for the title bar, the start screen and the About window.
pub fn load_logo(ctx: &eframe::egui::Context) -> Option<eframe::egui::TextureHandle> {
    base_ui::start::load_logo(ctx, ICON_256, "lightcreator-logo")
}

fn window_icon() -> Option<eframe::egui::IconData> {
    base_ui::start::window_icon(ICON_512)
}

/// Where this program keeps its data: the settings in its own folder, the gallery shared with the other programs
/// (see `base_core::paths`).
pub fn data_paths() -> &'static lc_core::paths::DataPaths {
    static PATHS: std::sync::OnceLock<lc_core::paths::DataPaths> = std::sync::OnceLock::new();
    PATHS.get_or_init(|| {
        if cfg!(test) {
            // Tests never touch the real folders.
            return lc_core::paths::prepare_in(&std::env::temp_dir().join(format!("lc-test-data-{}", std::process::id())), "lightcreator");
        }
        lc_core::paths::prepare("lightcreator")
    })
}

fn main() -> eframe::Result {
    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_title(format!("LightCreator {}", app::APP_VERSION))
        .with_app_id("lightcreator")
        .with_inner_size([1360.0, 860.0])
        .with_min_inner_size([900.0, 560.0])
        // The application draws its own flat, square frame (see window_ui.rs).
        .with_decorations(false);
    if let Some(icon) = window_icon() {
        viewport = viewport.with_icon(icon);
    }
    let opts = eframe::NativeOptions { viewport, persistence_path: Some(data_paths().settings.clone()), ..Default::default() };
    eframe::run_native(
        "LightCreator",
        opts,
        Box::new(|cc| {
            #[cfg(target_os = "macos")]
            native_menu::set_dock_icon(ICON_512);
            let mut app = app::App::new(cc);
            if let Some(p) = std::env::args().nth(1) {
                app.open_path(p.into());
            }
            Ok(Box::new(app))
        }),
    )
}
