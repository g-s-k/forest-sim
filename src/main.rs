fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let size_x = forest_sim::SIZE_X as f32 + 15.0;
    let size_y = forest_sim::SIZE_Y as f32 + 60.0;

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([size_x, size_y])
            .with_min_inner_size([size_x, size_y]),
        ..Default::default()
    };
    eframe::run_native(
        "forest sim",
        native_options,
        Box::new(|cc| Ok(Box::new(forest_sim::App::new(cc)))),
    )
}
