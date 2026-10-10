mod app;
mod camera;
mod camera_stream;
mod clip_net;
mod clipart_ui;
mod canvas;
mod device_ui;
mod fonts;
#[cfg(test)]
mod grbl_sim_tests;
mod i18n;
mod guide;
mod help_ui;
mod icons;
mod image_ui;
mod laser;
mod materials_ui;
mod menu;
mod nodes;
#[cfg(target_os = "macos")]
mod native_menu;
mod overlay;
mod panels;
mod preview_ui;
mod prefs;
mod ser2net_ui;
mod shape_ops;
mod stream_ui;
#[cfg(test)]
mod screenshots;
mod theme;
mod session_ui;
mod units_ui;
mod window_ui;

const ICON_256: &[u8] = include_bytes!("../../../assets/icons/icon-256.png");
const ICON_512: &[u8] = include_bytes!("../../../assets/icons/icon-512.png");

/// Decode the bundled PNG logo into a texture for the About window.
pub fn load_logo(ctx: &eframe::egui::Context) -> Option<eframe::egui::TextureHandle> {
    let img = image::load_from_memory_with_format(ICON_256, image::ImageFormat::Png).ok()?.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    let color = eframe::egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw());
    Some(ctx.load_texture("lightcreator-logo", color, eframe::egui::TextureOptions::LINEAR))
}

fn window_icon() -> Option<eframe::egui::IconData> {
    let img = image::load_from_memory_with_format(ICON_512, image::ImageFormat::Png).ok()?.to_rgba8();
    Some(eframe::egui::IconData { width: img.width(), height: img.height(), rgba: img.into_raw() })
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
    let opts = eframe::NativeOptions { viewport, ..Default::default() };
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
