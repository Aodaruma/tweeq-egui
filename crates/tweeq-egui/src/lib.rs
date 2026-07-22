//! Tweeq parameter-tuning widgets for egui.

#![forbid(unsafe_code)]

mod context;
mod theme;
mod widgets;

pub use context::TweeqContext;
pub use theme::{ColorMode, TweeqTheme};
pub use tweeq_core as core;
pub use tweeq_core::{
    EditEvent, EditOperation, EditSessionId, ParamId, ParamKind, ParamSnapshot, ParamValue,
};
pub use widgets::{
    Angle, Button, Checkbox, ColorInput, Dropdown, Drum, InputGroup, Number, PointerPolicy,
    Position, Radio, Rotary, Size, StringInput, Switch, Timecode, ToggleButton, Translate,
    TweakResponse, Vector,
};
