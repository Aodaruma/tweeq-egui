# tweeq-core

Renderer-independent interaction semantics for the Rust/egui port of Tweeq.

This crate contains stable parameter IDs, explicit edit sessions, selection,
numeric validation, tweak gesture math, reproducible shuffle values, ruler
intervals, and viewport transforms. It does not depend on egui.

The API is currently an alpha preview. See the repository's
`docs/rust-port/compatibility-report.md` before adopting it in a persistent
project format.

Licensed under MIT. Tweeq was created by Baku Hashimoto; see `NOTICE.md` in the
repository for attribution and port status.
