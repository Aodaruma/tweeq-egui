# 公開・切替チェックリスト

## `0.1.0-alpha.1` preview

| 項目 | 2026-07-22 | 備考 |
|---|---|---|
| `cargo fmt --all --check` | 成功 | workspace全体 |
| clippy `-D warnings` | 成功 | all targets/features |
| native tests | 成功 | core 12、egui 5、doc testsを含む |
| WASM check | 成功 | core/egui、同一公開API |
| Windows native操作 | 成功 | Phase 2〜4記録参照 |
| Vue unit test | 成功 | 4件 |
| VuePress build | 警告付き成功 | SSR例外と循環chunk警告あり |
| crate README/rustdoc | 準備済み | 最小Number例をdoc test化 |
| LICENSE/NOTICE/CITATION | 準備済み | 原著作権表示を維持 |
| `tweeq-core` package/verify | 成功 | crate単体で再コンパイル成功 |
| `tweeq-egui` package list | 成功 | core未公開のためpackage/verifyは順序待ち |
| crates.io名称 | 検索結果なし | `cargo search tweeq`。予約は保証しない |
| Linux/macOS native実機 | 未実施 | alpha既知制限 |
| WASMブラウザー実機 | 未実施 | alpha既知制限 |

検証コマンド:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo check -p tweeq-core -p tweeq-egui --target wasm32-unknown-unknown --all-features
cargo package -p tweeq-core --allow-dirty
cargo package -p tweeq-egui --allow-dirty --list
```

`tweeq-egui`のpackage/verifyは、version固定された`tweeq-core`がcrates.ioへ公開された後に行います。
公開順はcore、registry反映確認、eguiです。`tweeq-demo`はpublishしません。

## SemVerとMSRV

- preview: `0.1.0-alpha.N`。API互換を保証せず、変更をCHANGELOGへ記録する。
- 最初の安定候補: 全切替ブロッカーの解消後に`0.1.0`。
- MSRV: 1.92。CIで固定toolchainも追加してから公開する。
- default feature: 現在は追加featureなし。renderer固有機能をdefaultへ入れない。
- `eframe`はdemoのみ。`tweeq-egui`は`egui`と`tweeq-core`だけに依存する。

## Phase 6: main完全Rust化のGo/No-Go

現在は**No-Go**です。次がすべて完了した時点でGoへ変更します。

- [ ] 互換性評価の切替ブロッカーを解消、または非対応として利用者合意を得る。
- [ ] Windows、macOS、Linuxでnative操作表を埋める。
- [ ] Chrome、Firefox、Safari系でWASM操作表を埋める。
- [ ] native/WASMギャラリーをRustドキュメントの正式入口にする。
- [ ] `vue-final-reference`注釈付きtagを切替直前のVue commitへ作成する。
- [ ] Vue削除PRで機能追加を行わない。
- [ ] Node.jsなしのclean checkoutでbuild/test/docs/demoを生成できる。
- [ ] root README、GitHub Pages、release workflowをRustへ切り替える。

削除は`src`、VuePress/Vite、`package.json`、lockfile、Node CIを対象としますが、
`LICENSE`、`NOTICE.md`、`CITATION.cff`、論文・設計背景、Vue参照tagは保持します。

## crates.io公開時の最終手順

1. `cargo search tweeq`とcrates.io画面で名称を再確認する。
2. clean worktreeで上記検証を再実行する。
3. `cargo package --list`でVue資産がcrateへ混入していないことを確認する。
4. `tweeq-core`をdry-run後に公開する。
5. registry反映後、`tweeq-egui`をdry-run・公開する。
6. tag、GitHub Release、docs.rs、WASM galleryを確認する。

publish、tag push、main削除は外部・共有状態を変更するため、実行時に改めて明示的な指示を
受けてから行います。
