mod app;
mod export;
mod models;
mod parser;

use eframe::egui;
use egui::IconData;

fn load_icon() -> IconData {
    let png_bytes = include_bytes!("../icons/clashpass_32.png");
    let decoder = png::Decoder::new(std::io::Cursor::new(png_bytes));
    let mut reader = decoder.read_info().expect("Failed to decode icon PNG");
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).expect("Failed to read icon frame");
    buf.truncate(info.buffer_size());
    IconData {
        rgba: buf,
        width: info.width,
        height: info.height,
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
