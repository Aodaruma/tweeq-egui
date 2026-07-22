use eframe::egui;
use tweeq_egui::{ColorMode, TweeqContext, TweeqTheme};

/// Interactive gallery used to develop and verify Tweeq widgets.
pub struct GalleryApp {
    tweeq: TweeqContext,
    mode: ColorMode,
}

impl Default for GalleryApp {
    fn default() -> Self {
        Self {
            tweeq: TweeqContext::default(),
            mode: ColorMode::Dark,
        }
    }
}

impl GalleryApp {
    /// Creates and installs the initial Tweeq theme.
    #[must_use]
    pub fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        let app = Self::default();
        app.tweeq.theme().install(&creation_context.egui_ctx);
        app
    }
}

impl eframe::App for GalleryApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Tweeq for egui");
                let label = match self.mode {
                    ColorMode::Light => "Dark",
                    ColorMode::Dark => "Light",
                };
                if ui.button(label).clicked() {
                    self.mode = match self.mode {
                        ColorMode::Light => ColorMode::Dark,
                        ColorMode::Dark => ColorMode::Light,
                    };
                    let theme = match self.mode {
                        ColorMode::Light => TweeqTheme::light(),
                        ColorMode::Dark => TweeqTheme::dark(),
                    };
                    theme.install(ui.ctx());
                    self.tweeq.set_theme(theme);
                }
            });
            ui.add_space(16.0);
            ui.label("Phase 1 workspace and theme foundation");
            ui.label(format!("tweeq-core {}", tweeq_egui::core::VERSION));
        });
    }

    #[cfg(target_arch = "wasm32")]
    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use wasm_bindgen::prelude::*;

    /// JavaScript handle for the eframe WASM application.
    #[derive(Clone)]
    #[wasm_bindgen]
    pub struct WebHandle {
        runner: eframe::WebRunner,
    }

    #[wasm_bindgen]
    impl WebHandle {
        /// Creates an inactive web runner.
        #[wasm_bindgen(constructor)]
        #[must_use]
        pub fn new() -> Self {
            Self {
                runner: eframe::WebRunner::new(),
            }
        }

        /// Starts the gallery in the supplied canvas.
        pub async fn start(
            &self,
            canvas: web_sys::HtmlCanvasElement,
        ) -> Result<(), wasm_bindgen::JsValue> {
            self.runner
                .start(
                    canvas,
                    eframe::WebOptions::default(),
                    Box::new(|context| Ok(Box::new(super::GalleryApp::new(context)))),
                )
                .await
        }

        /// Stops the gallery and releases browser resources.
        pub fn destroy(&self) {
            self.runner.destroy();
        }
    }

    impl Default for WebHandle {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use web::WebHandle;
