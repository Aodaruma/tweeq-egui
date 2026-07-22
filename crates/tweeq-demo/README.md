# Tweeq egui gallery

Run the native gallery from the repository root:

```sh
cargo run -p tweeq-demo
```

The same `GalleryApp` is exported as `WebHandle` on
`wasm32-unknown-unknown`. The web packaging entry point will be added when the
gallery replaces the Vue documentation site; CI already checks that target.

The gallery contains interactive examples for Number, Rotary, Angle, Boolean,
String, choice, Position, Translate, Vector, Size, Timecode, Drum, and Color
parameters. Number supports click-to-edit, drag-to-tweak, vertical sensitivity,
Shift/Alt/Q modifiers, explicit commit/cancel events, multi-select, and an
optional pointer-lock request for unbounded parameters. Expand “InputNumber
parity controls” to exercise bar origin, min/max, step, clamp, precision,
disabled, and invalid states. Angle exposes snap and angle-offset controls.

Phase 4 examples add CubicBezier, deterministic Shuffle, ComplexInput, tabs,
rulers, a timeline scrubber, a pan/zoom viewport, floating/collapsing panes,
a lightweight code input, and a host-owned command palette.

This gallery proves the shared API and edit lifecycle. It is not yet a pixel-
or shortcut-complete reproduction of the Vue implementation; current
differences are tracked in `docs/rust-port/component-parity-audit.md`.
