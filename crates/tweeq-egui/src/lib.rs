//! Tweeq parameter-tuning widgets for egui.

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
