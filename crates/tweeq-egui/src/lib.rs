//! Tweeq parameter-tuning widgets for egui.
//!
//! The host owns parameter values and keeps one [`TweeqContext`]. Widgets use
//! stable [`ParamId`] values and emit edit sessions suitable for Undo/Redo.
//!
//! ```no_run
//! use tweeq_egui::{Number, ParamId, TweeqContext};
//!
//! fn opacity_ui(ui: &mut egui::Ui, context: &mut TweeqContext, opacity: &mut f64) {
//!     Number::new(ParamId::from_static("opacity"), opacity)
//!         .range(0.0..=1.0)
//!         .step(0.01)
//!         .snap(0.1)
//!         .precision(3)
//!         .show(ui, context);
//! }
//! ```

#![forbid(unsafe_code)]

mod context;
mod theme;
mod widgets;
mod workspace;

pub use context::TweeqContext;
pub use theme::{ColorMode, TweeqTheme};
pub use tweeq_core as core;
pub use tweeq_core::{
    EditEvent, EditOperation, EditSessionId, ParamId, ParamKind, ParamSnapshot, ParamValue,
};
pub use widgets::{
    Angle, Button, Checkbox, CodeInput, ColorInput, ComplexInput, CubicBezier, Dropdown, Drum,
    InputGroup, Number, PointerPolicy, Position, Radio, Rotary, Shuffle, Size, StringInput, Switch,
    Timecode, ToggleButton, Translate, TweakResponse, Vector,
};
pub use workspace::{
    CollapsingPane, CommandPalette, FloatingPane, Ruler, Tabs, Timeline, Viewport2D,
};
