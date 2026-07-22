# tweeq-egui

Tweeq-style parameter-tuning widgets for egui.

```rust,no_run
use tweeq_egui::{Number, ParamId, TweeqContext};

fn ui(ui: &mut egui::Ui, context: &mut TweeqContext, opacity: &mut f64) {
    Number::new(ParamId::from_static("opacity"), opacity)
        .range(0.0..=1.0)
        .step(0.01)
        .snap(0.1)
        .precision(3)
        .show(ui, context);
}
```

Run `cargo run -p tweeq-demo` from the repository root for the interactive
native gallery. Native and WASM use the same public widget API.

This is an alpha port. Number and the edit lifecycle are the most complete;
Color, Timecode, Drum, multi-turn Rotary overlays, and workspace components
still have documented differences from the Vue reference implementation.

Licensed under MIT. Tweeq was created by Baku Hashimoto; see `NOTICE.md` in the
repository for attribution and port status.
