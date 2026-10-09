mod app;
mod canvas;
mod icons;
mod laser;
mod panels;
mod theme;

fn main() -> eframe::Result {
    let opts = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("LightCreator")
            .with_inner_size([1360.0, 860.0])
            .with_min_inner_size([900.0, 560.0]),
        ..Default::default()
    };
    eframe::run_native(
        "LightCreator",
        opts,
        Box::new(|cc| {
            theme::apply(&cc.egui_ctx);
            let mut app = app::App::new(cc);
            if let Some(p) = std::env::args().nth(1) {
                app.open_path(p.into());
            }
            Ok(Box::new(app))
        }),
    )
}
