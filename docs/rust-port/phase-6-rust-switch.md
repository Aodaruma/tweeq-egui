# Phase 6: Rust-only switch

## Decision

2026-07-22、Phase 5の対話確認を踏まえ、`0.1.0-alpha.1`では既知差分を公開文書へ
残すことを条件にPhase 6へ進む判断を行いました。InputSizeのconstraint iconには
pixel-levelの差が残りますが、値、constraintのON/OFF、編集eventは機能します。

## Preserved reference

切替直前のVue/TypeScript実装は注釈付きtag `vue-final-reference`に固定しました。
削除されたcomponent、Vue単体比較ページ、VuePress研究ページ、画像assetは次で確認できます。

```sh
git show vue-final-reference:src/InputNumber.vue
git worktree add ../tweeq-vue-reference vue-final-reference
```

upstreamの実行可能な公開版は https://baku89.github.io/tweeq/ に残ります。

## Removed from main

- `src/**/*.vue`、`src/**/*.ts`とshader/styleを含むVueアプリケーション
- Vite、VuePress、TypeScript、ESLint、Prettier設定
- `package.json`、Yarn lockfile、Node CI/job
- Vue componentを実行する旧ドキュメントと比較ページ

LICENSE、NOTICE、CITATION、論文情報、設計原則、移植判断は保持しました。

## Rust replacements

- native gallery: `cargo run -p tweeq-demo`
- WASM gallery: `cd crates/tweeq-demo && trunk serve index.html --open`
- library documentation: `cargo doc -p tweeq-core -p tweeq-egui --all-features --no-deps`
- Pages: Trunk galleryとrustdocを単一のRust-only workflowで生成
- CI: stable、MSRV 1.92、WASM、rustdoc、package dry-run

クリーンcheckoutのbuild/test/docs/demoにNode.js、npm、Yarnは不要です。

## Accepted alpha limitations

- Pointer Lock/Confinedの可否はOS、ブラウザー、window backendに依存する。
- InputSize constraint iconには参照版との小さな形状差が残る。
- Linux/macOS nativeと主要ブラウザーの実機matrixは公開後も継続して埋める。
- `ComplexInput`はVueのschema-driven `InputComplex`ではなく、Rust固有の複素数入力である。
- workspace primitiveはegui hostとの統合点であり、Vue application shellの逐語移植ではない。

これらは安全性・保存形式を損なう切替ブロッカーではなく、alphaで明示する互換性契約とします。
