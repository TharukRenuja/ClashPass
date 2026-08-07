mod app;
mod export;
mod models;
mod parser;

use eframe::egui;
use egui::IconData;

fn load_icon() -> IconData {
    let png_bytes = include_bytes!("../icons/clashpass_32.png");
    let img = image::load_from_memory(png_bytes)
        .expect("Failed to load icon PNG")
        .into_rgba8();
    let (w, h) = img.dimensions();
    IconData {
        rgba: img.into_raw(),
        width: w,
        height: h,
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("ClashPass — Password Conflict Resolver")
            .with_inner_size([1200.0, 700.0])
            .with_min_inner_size([800.0, 500.0])
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "ClashPass",
        options,
        Box::new(|cc| Ok(Box::new(app::PasswordComparerApp::new(cc)))),
    )
}
