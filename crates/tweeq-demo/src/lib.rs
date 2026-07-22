use std::collections::HashMap;

use eframe::egui;
use tweeq_egui::{
    ColorMode, EditEvent, EditOperation, EditSessionId, Number, ParamId, ParamValue, PointerPolicy,
    TweeqContext, TweeqTheme,
};

const OPACITY: ParamId = ParamId::from_static("demo.opacity");
const ROTATION: ParamId = ParamId::from_static("demo.rotation");
const OFFSET_X: ParamId = ParamId::from_static("demo.offset_x");

/// Interactive gallery used to develop and verify Tweeq widgets.
pub struct GalleryApp {
    tweeq: TweeqContext,
    mode: ColorMode,
    opacity: f64,
    rotation: f64,
    offset_x: f64,
    pointer_lock: bool,
    captures: HashMap<EditSessionId, Vec<(ParamId, f64)>>,
    event_log: Vec<String>,
}

impl Default for GalleryApp {
    fn default() -> Self {
        Self {
            tweeq: TweeqContext::default(),
            mode: ColorMode::Dark,
            opacity: 0.72,
            rotation: 35.0,
            offset_x: 120.0,
            pointer_lock: false,
            captures: HashMap::new(),
            event_log: Vec::new(),
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
        self.tweeq.begin_frame();
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
            ui.add_space(10.0);
            ui.label("Number vertical slice — click to type, drag to tweak");
            ui.label("Shift: fast · Alt: fine · Q: snap · Enter: commit · Escape: cancel");
            ui.checkbox(
                &mut self.pointer_lock,
                "Try pointer lock for unbounded drag",
            );
            ui.add_space(12.0);

            egui::Grid::new("number-gallery")
                .num_columns(2)
                .spacing([18.0, 9.0])
                .show(ui, |ui| {
                    ui.label("Opacity");
                    Number::new(OPACITY, &mut self.opacity)
                        .range(0.0..=1.0)
                        .step(0.01)
                        .snap(0.1)
                        .precision(3)
                        .default_value(1.0)
                        .show(ui, &mut self.tweeq);
                    ui.end_row();

                    ui.label("Rotation");
                    Number::new(ROTATION, &mut self.rotation)
                        .range(-180.0..=180.0)
                        .step(1.0)
                        .snap(15.0)
                        .precision(1)
                        .suffix("°")
                        .default_value(0.0)
                        .show(ui, &mut self.tweeq);
                    ui.end_row();

                    ui.label("Offset X");
                    Number::new(OFFSET_X, &mut self.offset_x)
                        .step(0.1)
                        .snap(10.0)
                        .precision(2)
                        .suffix(" px")
                        .bar(false)
                        .pointer_policy(if self.pointer_lock {
                            PointerPolicy::TryLocked
                        } else {
                            PointerPolicy::Disabled
                        })
                        .default_value(0.0)
                        .show(ui, &mut self.tweeq);
                    ui.end_row();
                });

            ui.add_space(20.0);
            ui.separator();
            ui.label(format!(
                "opacity={:.3}  rotation={:.1}  offset_x={:.2}",
                self.opacity, self.rotation, self.offset_x
            ));
            ui.weak("Ctrl/Command-click or Shift-click fields to build a simultaneous selection.");
            ui.collapsing("Recent edit events", |ui| {
                for event in self.event_log.iter().rev().take(10) {
                    ui.monospace(event);
                }
            });
        });

        let events: Vec<_> = self.tweeq.drain_events().collect();
        for event in events {
            self.apply_event(event);
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }
}

impl GalleryApp {
    fn apply_event(&mut self, event: EditEvent) {
        self.event_log.push(format!("{event:?}"));
        if self.event_log.len() > 64 {
            self.event_log.remove(0);
        }

        match event {
            EditEvent::Begin {
                session, targets, ..
            } => {
                let captured = targets
                    .into_iter()
                    .filter_map(|snapshot| match snapshot.value {
                        ParamValue::Number(value) => Some((snapshot.id, value)),
                        _ => None,
                    })
                    .collect();
                self.captures.insert(session, captured);
            }
            EditEvent::Update { session, operation } => {
                let Some(captured) = self.captures.get(&session).cloned() else {
                    return;
                };
                for (id, initial) in captured {
                    let next = match &operation {
                        EditOperation::SetNumber(value) => *value,
                        EditOperation::AddNumber(delta) => initial + delta,
                        EditOperation::ScaleNumber(scale) => initial * scale,
                        _ => continue,
                    };
                    self.set_value(id, next);
                }
            }
            EditEvent::Commit { session } => {
                self.captures.remove(&session);
            }
            EditEvent::Cancel { session } => {
                if let Some(captured) = self.captures.remove(&session) {
                    for (id, value) in captured {
                        self.set_value(id, value);
                    }
                }
            }
        }
    }

    fn set_value(&mut self, id: ParamId, value: f64) {
        match id {
            OPACITY => self.opacity = value.clamp(0.0, 1.0),
            ROTATION => self.rotation = value.clamp(-180.0, 180.0),
            OFFSET_X => self.offset_x = value,
            _ => {}
        }
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
