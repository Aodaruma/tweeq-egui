use std::collections::HashMap;

use eframe::egui;
use tweeq_egui::{
    Angle, Button, Checkbox, CodeInput, CollapsingPane, ColorInput, ColorMode, CommandPalette,
    ComplexInput, CubicBezier, Dropdown, Drum, EditEvent, EditOperation, EditSessionId,
    FloatingPane, InputGroup, Number, ParamId, ParamSnapshot, ParamValue, PointerPolicy, Position,
    Radio, Rotary, Ruler, Shuffle, Size, StringInput, Switch, Tabs, Timecode, Timeline,
    ToggleButton, Translate, TweeqContext, TweeqTheme, Vector, Viewport2D,
};

const OPACITY: ParamId = ParamId::from_static("demo.opacity");
const ROTATION: ParamId = ParamId::from_static("demo.rotation");
const OFFSET_X: ParamId = ParamId::from_static("demo.offset_x");
const ROTARY: ParamId = ParamId::from_static("demo.rotary");
const ANGLE: ParamId = ParamId::from_static("demo.angle");
const CHECKBOX: ParamId = ParamId::from_static("demo.checkbox");
const SWITCH: ParamId = ParamId::from_static("demo.switch");
const TOGGLE: ParamId = ParamId::from_static("demo.toggle");
const NAME: ParamId = ParamId::from_static("demo.name");
const DROPDOWN: ParamId = ParamId::from_static("demo.dropdown");
const RADIO: ParamId = ParamId::from_static("demo.radio");
const DRUM: ParamId = ParamId::from_static("demo.drum");
const POSITION: ParamId = ParamId::from_static("demo.position");
const TRANSLATE: ParamId = ParamId::from_static("demo.translate");
const VECTOR: ParamId = ParamId::from_static("demo.vector");
const SIZE: ParamId = ParamId::from_static("demo.size");
const TIMECODE: ParamId = ParamId::from_static("demo.timecode");
const COLOR: ParamId = ParamId::from_static("demo.color");
const BEZIER: ParamId = ParamId::from_static("demo.bezier");
const SHUFFLE: ParamId = ParamId::from_static("demo.shuffle");
const COMPLEX: ParamId = ParamId::from_static("demo.complex");
const TIMELINE: ParamId = ParamId::from_static("demo.timeline");
const FRUIT: &[&str] = &["Apple", "Banana", "Cherry", "Dragonfruit"];
const COMMANDS: &[&str] = &["Reset viewport", "Toggle theme", "Export parameters"];

/// Interactive gallery used to develop and verify Tweeq widgets.
#[allow(clippy::struct_excessive_bools)]
pub struct GalleryApp {
    tweeq: TweeqContext,
    mode: ColorMode,
    opacity: f64,
    rotation: f64,
    offset_x: f64,
    pointer_lock: bool,
    rotary: f64,
    angle: f64,
    checkbox: bool,
    switch: bool,
    toggle: bool,
    button_count: u32,
    name: String,
    dropdown: String,
    radio: String,
    drum: String,
    position: [f64; 2],
    translate: [f64; 2],
    vector: [f64; 3],
    size: [f64; 2],
    aspect_locked: bool,
    frames: i64,
    color: [f32; 4],
    bezier: [f64; 4],
    shuffled: f64,
    shuffle_seed: u64,
    complex: [f64; 2],
    active_tab: usize,
    timeline: f64,
    viewport_offset: [f64; 2],
    viewport_scale: f64,
    code: String,
    floating_open: bool,
    palette_open: bool,
    palette_query: String,
    command_status: String,
    captures: HashMap<EditSessionId, Vec<ParamSnapshot>>,
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
            rotary: 30.0,
            angle: -45.0,
            checkbox: true,
            switch: false,
            toggle: false,
            button_count: 0,
            name: "Baby salmon".to_owned(),
            dropdown: "Apple".to_owned(),
            radio: "Banana".to_owned(),
            drum: "Cherry".to_owned(),
            position: [24.0, -12.0],
            translate: [-18.0, 32.0],
            vector: [1.0, 2.0, 3.0],
            size: [1920.0, 1080.0],
            aspect_locked: true,
            frames: 24 * 61 + 12,
            color: [0.16, 0.42, 1.0, 1.0],
            bezier: [0.25, 0.1, 0.25, 1.0],
            shuffled: 0.42,
            shuffle_seed: 41,
            complex: [1.0, -0.5],
            active_tab: 0,
            timeline: 42.0,
            viewport_offset: [-4.0, -2.0],
            viewport_scale: 18.0,
            code: "fn ease(t: f32) -> f32 {\n    t * t * (3.0 - 2.0 * t)\n}".to_owned(),
            floating_open: false,
            palette_open: false,
            palette_query: String::new(),
            command_status: "No command selected".to_owned(),
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

    fn toggle_theme(&mut self, context: &egui::Context) {
        self.mode = match self.mode {
            ColorMode::Light => ColorMode::Dark,
            ColorMode::Dark => ColorMode::Light,
        };
        let theme = match self.mode {
            ColorMode::Light => TweeqTheme::light(),
            ColorMode::Dark => TweeqTheme::dark(),
        };
        theme.install(context);
        self.tweeq.set_theme(theme);
    }

    #[allow(clippy::too_many_lines)]
    fn gallery(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        ui.label("Interactive parameter gallery");
        ui.label("Shift: fast · Alt: fine · Q: snap · Enter: commit · Escape: cancel");
        ui.checkbox(
            &mut self.pointer_lock,
            "Try pointer lock for unbounded drag",
        );

        section(ui, "Numbers");
        parameter_grid(ui, "number-gallery", |ui| {
            row(ui, "Opacity", |ui| {
                Number::new(OPACITY, &mut self.opacity)
                    .range(0.0..=1.0)
                    .step(0.01)
                    .snap(0.1)
                    .precision(3)
                    .default_value(1.0)
                    .show(ui, &mut self.tweeq);
            });
            row(ui, "Rotation", |ui| {
                Number::new(ROTATION, &mut self.rotation)
                    .range(-180.0..=180.0)
                    .step(1.0)
                    .snap(15.0)
                    .precision(1)
                    .suffix("°")
                    .default_value(0.0)
                    .show(ui, &mut self.tweeq);
            });
            row(ui, "Offset X", |ui| {
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
            });
        });

        section(ui, "Angles");
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.weak("Rotary");
                Rotary::new(ROTARY, &mut self.rotary).show(ui, &mut self.tweeq);
            });
            ui.add_space(20.0);
            ui.vertical(|ui| {
                ui.weak("Angle composition");
                Angle::new(ANGLE, &mut self.angle).show(ui, &mut self.tweeq);
            });
        });

        section(ui, "Boolean and actions");
        InputGroup::show(ui, |ui| {
            Checkbox::new(CHECKBOX, &mut self.checkbox, "Checkbox").show(ui, &mut self.tweeq);
            Switch::new(SWITCH, &mut self.switch, "Switch").show(ui, &mut self.tweeq);
            ToggleButton::new(TOGGLE, &mut self.toggle, "Toggle").show(ui, &mut self.tweeq);
            if Button::new("Action").show(ui).clicked() {
                self.button_count += 1;
            }
            ui.weak(format!("{} clicks", self.button_count));
        });

        section(ui, "Text and choices");
        parameter_grid(ui, "choice-gallery", |ui| {
            row(ui, "String", |ui| {
                StringInput::new(NAME, &mut self.name)
                    .hint("Name")
                    .show(ui, &mut self.tweeq);
            });
            row(ui, "Dropdown", |ui| {
                Dropdown::new(DROPDOWN, &mut self.dropdown, FRUIT).show(ui, &mut self.tweeq);
            });
            row(ui, "Radio", |ui| {
                Radio::new(RADIO, &mut self.radio, &FRUIT[..3]).show(ui, &mut self.tweeq);
            });
            row(ui, "Drum", |ui| {
                Drum::new(DRUM, &mut self.drum, FRUIT).show(ui, &mut self.tweeq);
            });
        });

        section(ui, "Vectors and geometry");
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.weak("Position (drag pad)");
                Position::new(POSITION, &mut self.position).show(ui, &mut self.tweeq);
            });
            ui.vertical(|ui| {
                ui.weak("Translate (drag pad)");
                Translate::new(TRANSLATE, &mut self.translate).show(ui, &mut self.tweeq);
            });
        });
        parameter_grid(ui, "vector-gallery", |ui| {
            row(ui, "Vector 3", |ui| {
                Vector::new(VECTOR, &mut self.vector).show(ui, &mut self.tweeq);
            });
            row(ui, "Size", |ui| {
                Size::new(SIZE, &mut self.size, &mut self.aspect_locked).show(ui, &mut self.tweeq);
            });
        });

        section(ui, "Time and color");
        parameter_grid(ui, "media-gallery", |ui| {
            row(ui, "Timecode", |ui| {
                Timecode::new(TIMECODE, &mut self.frames)
                    .frame_rate(24)
                    .show(ui, &mut self.tweeq);
            });
            row(ui, "Color", |ui| {
                ColorInput::new(COLOR, &mut self.color).show(ui, &mut self.tweeq);
                ui.monospace(format!(
                    "rgba({:.2}, {:.2}, {:.2}, {:.2})",
                    self.color[0], self.color[1], self.color[2], self.color[3]
                ));
            });
        });

        section(ui, "Advanced inputs");
        ui.weak("Cubic Bézier (drag either handle or edit its four values)");
        CubicBezier::new(BEZIER, &mut self.bezier).show(ui, &mut self.tweeq);
        parameter_grid(ui, "advanced-gallery", |ui| {
            row(ui, "Shuffle", |ui| {
                Shuffle::new(SHUFFLE, &mut self.shuffled, &mut self.shuffle_seed)
                    .range(-1.0..=1.0)
                    .show(ui, &mut self.tweeq);
            });
            row(ui, "Complex", |ui| {
                ComplexInput::new(COMPLEX, &mut self.complex).show(ui, &mut self.tweeq);
            });
        });

        section(ui, "Workspace primitives");
        Tabs::new(
            "workspace-tabs",
            &mut self.active_tab,
            &["Timeline", "Viewport", "Code"],
        )
        .show(ui);
        match self.active_tab {
            0 => {
                Ruler::new(0.0, 0.3)
                    .cursor(self.timeline)
                    .width(560.0)
                    .show(ui);
                Timeline::new(TIMELINE, &mut self.timeline, 0.0..=120.0).show(ui, &mut self.tweeq);
            }
            1 => {
                Viewport2D::new(&mut self.viewport_offset, &mut self.viewport_scale).show(ui);
            }
            _ => {
                CodeInput::new(&mut self.code).rows(6).show(ui);
            }
        }
        CollapsingPane::new("Pane adapter notes")
            .default_open(false)
            .show(ui, |ui| {
                ui.label(
                    "Collapsing and floating panes reuse egui persistence and focus behavior.",
                );
                ui.label("Docking and split trees stay optional host integrations.");
            });
        ui.horizontal(|ui| {
            if ui.button("Open floating pane").clicked() {
                self.floating_open = true;
            }
            if ui.button("Open command palette (Ctrl+P)").clicked() {
                self.palette_open = true;
            }
            ui.weak(&self.command_status);
        });

        ui.add_space(20.0);
        ui.separator();
        ui.weak("Ctrl/Command-click or Shift-click numeric fields for simultaneous selection.");
        ui.collapsing("Recent edit events", |ui| {
            for event in self.event_log.iter().rev().take(10) {
                ui.monospace(event);
            }
        });
    }

    fn apply_event(&mut self, event: EditEvent) {
        self.event_log.push(format!("{event:?}"));
        if self.event_log.len() > 64 {
            self.event_log.remove(0);
        }

        match event {
            EditEvent::Begin {
                session, targets, ..
            } => {
                self.captures.insert(session, targets);
            }
            EditEvent::Update { session, operation } => {
                let Some(captured) = self.captures.get(&session).cloned() else {
                    return;
                };
                for snapshot in captured {
                    let next = match (&snapshot.value, &operation) {
                        (ParamValue::Number(_), EditOperation::SetNumber(value)) => {
                            ParamValue::Number(*value)
                        }
                        (ParamValue::Number(initial), EditOperation::AddNumber(delta)) => {
                            ParamValue::Number(initial + delta)
                        }
                        (ParamValue::Number(initial), EditOperation::ScaleNumber(scale)) => {
                            ParamValue::Number(initial * scale)
                        }
                        (_, EditOperation::SetBoolean(value)) => ParamValue::Boolean(*value),
                        (_, EditOperation::SetString(value)) => ParamValue::String(value.clone()),
                        (
                            ParamValue::Vector { value, dimensions },
                            EditOperation::AddVector {
                                delta,
                                dimensions: operation_dimensions,
                            },
                        ) if dimensions == operation_dimensions => ParamValue::Vector {
                            value: std::array::from_fn(|index| value[index] + delta[index]),
                            dimensions: *dimensions,
                        },
                        (_, EditOperation::SetColor(value)) => ParamValue::Color(*value),
                        _ => continue,
                    };
                    self.set_value(snapshot.id, next);
                }
            }
            EditEvent::Commit { session } => {
                self.captures.remove(&session);
            }
            EditEvent::Cancel { session } => {
                if let Some(captured) = self.captures.remove(&session) {
                    for snapshot in captured {
                        self.set_value(snapshot.id, snapshot.value);
                    }
                }
            }
        }
    }

    fn set_value(&mut self, id: ParamId, value: ParamValue) {
        match value {
            ParamValue::Number(value) => self.set_number(id, value),
            ParamValue::Boolean(value) => match id {
                CHECKBOX => self.checkbox = value,
                SWITCH => self.switch = value,
                TOGGLE => self.toggle = value,
                _ => {}
            },
            ParamValue::String(value) => match id {
                NAME => self.name = value,
                DROPDOWN => self.dropdown = value,
                RADIO => self.radio = value,
                DRUM => self.drum = value,
                _ => {}
            },
            ParamValue::Vector { value, dimensions } if dimensions >= 2 => {
                self.set_vector(id, [value[0], value[1]]);
            }
            ParamValue::Color(value) if id == COLOR => self.color = value,
            _ => {}
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn set_number(&mut self, id: ParamId, value: f64) {
        match id {
            OPACITY => self.opacity = value.clamp(0.0, 1.0),
            ROTATION => self.rotation = value.clamp(-180.0, 180.0),
            OFFSET_X => self.offset_x = value,
            ROTARY => self.rotary = value,
            candidate if candidate == ANGLE.child(1) || candidate == ANGLE.child(2) => {
                self.angle = value;
            }
            candidate if candidate == VECTOR.child(1) => self.vector[0] = value,
            candidate if candidate == VECTOR.child(2) => self.vector[1] = value,
            candidate if candidate == VECTOR.child(3) => self.vector[2] = value,
            candidate if candidate == SIZE.child(1) => self.size[0] = value,
            candidate if candidate == SIZE.child(2) => self.size[1] = value,
            TIMECODE => self.frames = value.round() as i64,
            candidate if candidate == BEZIER.child(1) => self.bezier[0] = value,
            candidate if candidate == BEZIER.child(2) => self.bezier[1] = value,
            candidate if candidate == BEZIER.child(3) => self.bezier[2] = value,
            candidate if candidate == BEZIER.child(4) => self.bezier[3] = value,
            SHUFFLE => self.shuffled = value,
            candidate if candidate == COMPLEX.child(1) => self.complex[0] = value,
            candidate if candidate == COMPLEX.child(2) => self.complex[1] = value,
            TIMELINE => self.timeline = value,
            _ => {}
        }
    }

    fn set_vector(&mut self, id: ParamId, value: [f64; 2]) {
        match id {
            POSITION => self.position = value,
            TRANSLATE => self.translate = value,
            candidate if candidate == BEZIER.child(101) => {
                self.bezier[0] = value[0];
                self.bezier[1] = value[1];
            }
            candidate if candidate == BEZIER.child(102) => {
                self.bezier[2] = value[0];
                self.bezier[3] = value[1];
            }
            _ => {}
        }
    }
}

impl eframe::App for GalleryApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.tweeq.begin_frame();
        if ui.input(|input| input.modifiers.command && input.key_pressed(egui::Key::P)) {
            self.palette_open = true;
        }
        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Tweeq for egui");
                let label = match self.mode {
                    ColorMode::Light => "Dark",
                    ColorMode::Dark => "Light",
                };
                if ui.button(label).clicked() {
                    self.toggle_theme(ui.ctx());
                }
            });
            egui::ScrollArea::vertical().show(ui, |ui| self.gallery(ui));
        });

        let mut floating_open = self.floating_open;
        FloatingPane::new(
            "demo-floating-pane",
            "Floating parameter pane",
            &mut floating_open,
        )
        .show(ui.ctx(), |ui| {
            ui.label("Host-owned content inside an egui window adapter.");
            ui.monospace(format!("timeline: {:.2}", self.timeline));
        });
        self.floating_open = floating_open;

        if let Some(command) =
            CommandPalette::new(&mut self.palette_open, &mut self.palette_query, COMMANDS)
                .show(ui.ctx())
        {
            match command {
                0 => {
                    self.viewport_offset = [-4.0, -2.0];
                    self.viewport_scale = 18.0;
                    "Viewport reset".clone_into(&mut self.command_status);
                }
                1 => {
                    self.toggle_theme(ui.ctx());
                    "Theme toggled".clone_into(&mut self.command_status);
                }
                _ => "Export adapter invoked".clone_into(&mut self.command_status),
            }
            self.palette_query.clear();
        }

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

fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(18.0);
    ui.heading(title);
    ui.separator();
    ui.add_space(5.0);
}

fn parameter_grid(ui: &mut egui::Ui, id: &str, contents: impl FnOnce(&mut egui::Ui)) {
    egui::Grid::new(id)
        .num_columns(2)
        .spacing([18.0, 9.0])
        .show(ui, contents);
}

fn row(ui: &mut egui::Ui, label: &str, contents: impl FnOnce(&mut egui::Ui)) {
    ui.label(label);
    contents(ui);
    ui.end_row();
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
