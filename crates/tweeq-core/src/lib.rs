//! Renderer-independent interaction semantics for Tweeq.
//!
//! Values remain owned by the host application. Tweeq emits explicit edit
//! sessions so applications can implement preview, multi-edit, and undo without
//! storing mutable references across UI frames.

#![forbid(unsafe_code)]

mod edit;
mod gesture;
mod id;
mod selection;
mod validation;

pub use edit::{EditEvent, EditOperation, EditSessionId, ParamKind, ParamSnapshot, ParamValue};
pub use gesture::{GestureModifiers, GestureUpdate, TweakGesture};
pub use id::ParamId;
pub use selection::Selection;
pub use validation::{NumberConstraints, NumberValidation, quantize};

/// The workspace package version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_workspace_version() {
        assert!(!super::VERSION.is_empty());
    }
}
