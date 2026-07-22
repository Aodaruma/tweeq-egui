#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Tweeq egui gallery")
            .with_inner_size([920.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Tweeq egui gallery",
        options,
        Box::new(|context| Ok(Box::new(tweeq_demo_lib::GalleryApp::new(context)))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {}
