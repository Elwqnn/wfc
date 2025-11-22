use wfc::app::App;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([900.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native(
        "WFC - Wave Function Collapse",
        options,
        Box::new(|_| Ok(Box::new(App::new()))),
    )
}
