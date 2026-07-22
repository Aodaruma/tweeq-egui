# 移植ロードマップ

> 2026-07-22更新: Phase 6のRust-only切替とPhase 7の公開準備まで完了しました。
> この文書の「残したまま」「削除前」という表現は、各phaseを実行した当時の手順を
> 記録するものです。現在の参照実装は`vue-final-reference` tagにあります。

## 基本方針

Vue版は移植中の実行可能な仕様書です。Rust版の縦切りを一つずつ完成させ、比較試験が
できる期間を保ちます。ディレクトリを先に一括変換したり、Vue版を先に削除したりは
しません。

各phaseは「コードが存在する」ではなく、記載したexit gateを満たした時点で完了です。

## Phase 0: 設計基準の固定

成果物:

- この文書群
- 作業ブランチ`codex/rust-egui-port`
- Vue参照基準コミットの記録
- コンポーネント移植表

Exit gate:

- クレート境界、同時編集方式、Vue削除条件について合意できる。
- 未決事項が実装開始を妨げない粒度になっている。

## Phase 1: Rust workspaceの併設

成果物:

- virtual Cargo workspace
- `tweeq-core`、`tweeq-egui`、`tweeq-demo`の最小crate
- formatter、clippy、unit test、WASM checkを行うCI
- MITライセンス、原作者表記、README、CHANGELOG
- 最小のlight/darkテーマとコンポーネントギャラリー

方針:

- この段階では`package.json`とVueのbuildを変更しない。
- Rust依存は必要になった時点で追加し、default featureを小さく保つ。
- egui/eframeのバージョン、Rust edition、MSRVはこのphaseでCI実測後に固定する。

Exit gate:

- `cargo test --workspace`がnativeで成功する。
- `tweeq-core`と`tweeq-egui`が`wasm32-unknown-unknown`でcheckできる。
- eframeデモに空のTweeq panelをnative/WASMの両方で表示できる。

## Phase 2: Number縦切りと操作基盤

成果物:

- validation、gesture、selection、edit session
- `TweeqContext`、`TweeqTheme`、foreground overlay
- `Number`と必要なtext-edit primitive
- 単一編集・同時編集のホスト例
- 値とイベント列を比較する自動試験

Exit gate:

- [アーキテクチャ](./architecture.md)の数値入力シナリオを満たす。
- drag中の修飾キー切替で編集セッションが分断されない。
- Enter、blur、Escapeの結果とUndo境界が試験されている。
- pointer grabを拒否した環境でも通常dragで編集を完了できる。

このphaseの公開APIレビュー後にのみ、残りの入力へ横展開します。

## Phase 3: 中核入力セット

実装順:

1. `Rotary`、`Angle`
2. Button、Toggle、Checkbox、Switch、Radio
3. String、Dropdown、InputGroup、ParameterGrid
4. Position、Vector、Size、Translate
5. Timecode、Drum
6. ColorInput

Exit gate:

- P1とP2の入力がコンポーネントギャラリーに揃う。
- すべてが同じ編集ライフサイクルとテーマprimitiveを利用する。
- native/WASMでAPIと保存形式が共通である。
- Vue版との差異一覧が各コンポーネントから参照できる。

最初のpreview releaseを行う場合は、`0.1.0-alpha.N`とし、未実装機能と
破壊的変更の可能性を明記します。

## Phase 4: 高度な入力とワークスペースUI

成果物:

- CubicBezier、Shuffle、ComplexInput
- Popover、Tooltip、Tabs、pane、selection popup
- Ruler、Timeline、Viewport helper
- CodeInput、ZUI、command palette向けの拡張interface

Exit gate:

- P3が完了、P4に置換例または明示的な非対応方針がある。
- renderer固有依存なしで既定機能が動作する。
- 大量パラメーター、長時間drag、HiDPIで許容できる性能を確認している。

## Phase 5: 切替準備

Vue版を削除する前に、次をすべて完了します。

- Vue版の比較対象を注釈付きGitタグ（例: `vue-final-reference`）で保存する。
- Rustデモが現行ドキュメントの主要な操作例を置き換える。
- Windows、macOS、Linux、Chrome/Firefox/Safari系WASMの検証表を公開する。
- 主要APIのrustdocと最小例を整備する。
- ライセンス、NOTICE相当の帰属、`CITATION.cff`を確認する。
- 公開クレートのfeature、MSRV、SemVer方針を確定する。
- `cargo package --list`で不要assetやVue成果物が混入しないことを確認する。

切替PRでは、機能追加とVue削除を混在させません。切替直前のRust版を固定し、削除と
README/CI/build入口の切替だけをレビューできるようにします。

## Phase 6: `main`の完全Rust化

状態: 完了（release branch。main反映はmerge時）

削除対象:

- `src/**/*.vue`, `src/**/*.ts`
- VuePress/Vite設定
- `package.json`, lockfile、Node専用CI
- Rustデモ・rustdocで代替済みのVueドキュメントasset

保持・更新対象:

- MIT `LICENSE`と原著作権表示
- 論文情報と`CITATION.cff`
- 設計背景、操作原則、移植差異
- Rust workspace、デモ、release workflow

Exit gate:

- クリーンcheckoutからNode.jsなしでbuild、test、docs、demoを生成できる。
- `main`のREADMEがRustライブラリの導入方法を第一に説明する。
- リポジトリ内に実行時・build時のVue/npm依存が残っていない。

## Phase 7: crates.io公開

状態: metadata、CI、分割release workflow、core dry-runまで完了。registryへの不可逆な
publish、release tag、GitHub Releaseは公開commitの確定後に行う。

主クレート名は`tweeq-egui`を候補とします。
2026-07-22時点の`cargo search tweeq`では
一致するRustクレートは表示されませんでしたが、名前は予約されていません。公開直前に
crates.io APIと`cargo search`で再確認します。

公開順:

1. `tweeq-core`の`cargo publish --dry-run`
2. `tweeq-core`公開
3. crates.io反映を確認
4. `tweeq-egui`の`cargo publish --dry-run`
5. `tweeq-egui`公開
6. tag、GitHub release、docs.rs、WASM demoを確認

公開ゲート:

- `cargo fmt --check`
- warningをdenyした`cargo clippy --workspace --all-targets --all-features`
- `cargo test --workspace --all-features`
- 最小featureと既定featureのWASM check
- READMEの最小コードがcompileするdoc test
- `repository`、`homepage`、`documentation`、`license`、`keywords`、`categories`を確認
- crate package内に`LICENSE`と必要な帰属文書が含まれる
- 依存ライセンスと既知脆弱性を確認する

## ブランチ運用

- 長期統合ブランチ: `codex/rust-egui-port`
- 小さな実装は同ブランチから短命branchを切り、縦切り単位で統合する。
- Vue upstreamの取り込みは、参照基準を更新する意図がある場合だけ行う。
- 参照基準を更新したら、この文書のcommit hashと差異表も同時に更新する。
- `main`への統合はphase単位で行えるが、Vue削除はPhase 5のgate通過後に限定する。

## 最初の実装PR

次のPRはPhase 1だけを対象にします。

1. virtual Cargo workspaceと3 crateを追加する。
2. egui/eframe、Rust edition、MSRVをCIで検証して固定する。
3. `TweeqTheme`の最小tokenと空のeframe galleryを作る。
4. native test、WASM check、Vue既存testを並列実行する。
5. APIやwidget実装は`Number`縦切りのPRへ分離する。
