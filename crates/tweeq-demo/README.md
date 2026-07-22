# Tweeq egui gallery

Run the native gallery from the repository root:

```sh
cargo run -p tweeq-demo
```

The same `GalleryApp` is exported as `WebHandle` on
`wasm32-unknown-unknown`. The web packaging entry point will be added when the
gallery replaces the Vue documentation site; CI already checks that target.

The Number vertical slice supports click-to-edit, drag-to-tweak, vertical
sensitivity, Shift/Alt/Q modifiers, explicit commit/cancel events, multi-select,
and an optional pointer-lock request for unbounded parameters.
