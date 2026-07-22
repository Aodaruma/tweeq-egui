# Release and Rust-only switch checklist

## `0.1.0-alpha.1`

| Gate | 2026-07-22 | Notes |
|---|---|---|
| `cargo fmt --all --check` | Pass | Entire workspace |
| clippy `-D warnings` | Pass | All targets and features |
| native tests | Pass | Core, egui, demo, and doc tests |
| Rust 1.92 MSRV | Pass | check, test, and clippy; fixed CI job added |
| WASM check | Pass | Same public widget API |
| rustdoc `-D warnings` | Pass | `tweeq-core` and `tweeq-egui` |
| native Windows interaction | Pass | Phase 2–5 records |
| native Linux/macOS interaction | Pending | Documented alpha limitation |
| WASM browser matrix | Pending | Documented alpha limitation |
| LICENSE/NOTICE/CITATION | Pass | Original copyright retained |
| `tweeq-core` package/dry-run | Pass | Rebuilds from packaged source |
| `tweeq-egui` package list | Pass | Full dry-run waits for core registry publication |
| crates.io names | Available at check time | Search is not a reservation |

Canonical local gates:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo check --workspace --all-features --locked --target wasm32-unknown-unknown
RUSTDOCFLAGS="-D warnings" cargo doc -p tweeq-core -p tweeq-egui --all-features --no-deps --locked
cargo publish -p tweeq-core --dry-run --locked
cargo package -p tweeq-egui --list --locked
```

## Phase 6: Rust-only main

Status: **Go / implemented on `codex/rust-egui-port`**.

- [x] Phase 5 differences were fixed or accepted as explicit alpha limitations.
- [x] `vue-final-reference` annotated tag records the final bundled Vue commit.
- [x] Vue, Vite, VuePress, package manager files, and Node CI were removed.
- [x] Root README now leads with Rust installation and API usage.
- [x] The native/WASM gallery and rustdoc replace the executable Vue docs.
- [x] Build, test, docs, and gallery generation require only Rust tooling.
- [x] Rust-only CI covers stable, MSRV 1.92, WASM, rustdoc, and package contents.

The remaining Size icon detail and unfilled cross-platform interaction matrix are
documented alpha limitations, not silent compatibility claims. See
[Phase 6 switch record](./phase-6-rust-switch.md).

## Phase 7: crates.io publication

Status: **repository preparation complete; registry publication pending**.

Publication is intentionally split to avoid a partially verified two-crate
release:

1. Merge or otherwise make the release commit available in the public repository.
2. Confirm `tweeq-core` and `tweeq-egui` are still unclaimed.
3. Run all gates on a clean release commit.
4. Publish `tweeq-core` after its dry-run.
5. Wait until `cargo info tweeq-core@0.1.0-alpha.1` resolves.
6. Package and dry-run `tweeq-egui`, then publish it.
7. Push `v0.1.0-alpha.1`, create the GitHub Release, and confirm docs.rs.
8. Confirm the Rust WASM gallery and API docs on GitHub Pages.

`.github/workflows/release.yml` requires an exact version confirmation, defaults
to dry-run, protects real publication with the `crates-io` environment, and
publishes one crate per invocation. Configure `CARGO_REGISTRY_TOKEN` only in
that protected environment.

Cargo packages cannot be overwritten or deleted. If core needs a correction
after publication, bump both workspace crates to the next alpha before
publishing the dependent crate.
