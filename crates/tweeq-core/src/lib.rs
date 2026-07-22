//! Renderer-independent interaction semantics for Tweeq.
//!
//! The crate is intentionally small during Phase 1. Validation, gestures,
//! selection, and edit sessions are introduced as the Number vertical slice.

#![forbid(unsafe_code)]

/// The workspace package version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_workspace_version() {
        assert!(!super::VERSION.is_empty());
    }
}
