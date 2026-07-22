# Tweeq Rust / egui 移植設計

## 現在地

この文書群は、TweeqのVue実装をRustへ再設計し、最終的に
`tweeq-egui`としてcrates.ioへ公開するための基準を定めるものです。
現時点では設計段階であり、Rust実装はまだ開始していません。

- 作業ブランチ: `codex/rust-egui-port`
- 参照実装: Vue版 `main`
- 参照基準コミット: `82205f396bc8f3b3eb16a2c6202681e6d71e1ae6`
- 基準日: 2026-07-22
- ライセンス: MIT（Baku Hashimotoによる原著作権表示を維持する）

## 進捗

| Phase | 状態 | 主な成果 |
|---:|---|---|
| 0 | 完了 | 設計基準、移植表、ロードマップ |
| 1 | 完了 | Cargo workspace、3クレート、CI、最小native/WASMデモ |
| 2 | 未着手 | Number縦切りと操作基盤 |
| 3 | 未着手 | P1/P2入力とパラメーターギャラリー |
| 4 | 未着手 | 高度入力、ワークスペースUI、拡張点 |
| 5 | 未着手 | Vue比較、互換性判断、切替準備 |

## 目標

1. Tweeq固有の高速・高精度なパラメーター調整操作をRustで再現する。
2. ライブラリ本体は`egui`だけに依存し、eframe、Bevyなどのeguiホストで
   利用できるようにする。
3. nativeとWASMで同じ公開APIを提供し、機能差は明示的なフォールバックで
   吸収する。
4. 同時編集、Undo境界、式入力を、Rustの所有権と安全性に合うイベント方式へ
   再設計する。
5. Rust版の互換性・ドキュメント・公開準備が整った段階で、`main`からVue、
   Vite、npm依存を削除する。
6. `tweeq-egui`をcrates.ioへ公開する。

## 非目標

- Vueのコンポーネント構造やDOM処理を逐語的に翻訳すること。
- `eframe`をライブラリ利用者へ必須依存として課すこと。
- 全OSでPointer Lockの挙動を完全に同一にすること。
- Monaco EditorそのものをRustで再実装すること。
- 移植完了前にVue版を削除して比較対象を失うこと。

## 決定事項

| 項目 | 方針 |
|---|---|
| 公開クレート | `tweeq-egui`を主クレート、`tweeq-core`をUI非依存クレートとする |
| eframe | デモ、native/WASM統合試験にのみ使用する |
| 値の精度 | 意味論・公開値は原則`f64`、描画時のみeguiの`f32`へ変換する |
| UI状態 | 安定した`egui::Id`と`ParamId`で保持し、値への`&mut`をフレーム間に保存しない |
| 同時編集 | 値参照の登録ではなく、ID付き編集イベントをアプリケーションへ返す |
| Pointer Lock | `Response::drag_motion()`を優先し、grab不能時は通常ドラッグへ退化する |
| オーバーレイ | eguiのforeground layerへ描画し、DOM top layerは移植しない |
| Color描画 | 最初はCPU生成テクスチャ、必要性を計測後にGPU callbackを検討する |
| 式入力 | JavaScript `eval`は移植せず、制限付きパーサーを任意機能として実装する |
| Vue削除 | [ロードマップ](./roadmap.md)の切替ゲート通過後にのみ行う |

## 文書

- [アーキテクチャ](./architecture.md): クレート境界、状態、イベント、入力、描画、API
- [コンポーネント移植表](./component-matrix.md): Vue版各機能の移植先と優先順位
- [ロードマップ](./roadmap.md): 段階、完了条件、ブランチ運用、公開手順

## 「完全移植」の定義

完全移植とは、ソースファイル数を一致させることではありません。次をすべて満たす
ことを完了条件とします。

- P0〜P3の対象について、操作意味論または文書化した代替操作が存在する。
- Windows、macOS、Linux、WASMの検証結果と既知の差異が公開されている。
- キーボードだけで主要入力を確定・取消できる。
- 編集の`Begin`、`Update`、`Commit`、`Cancel`がUndo単位として利用できる。
- ライブラリの通常利用でunsafeコードを必要としない。
- RustのデモとAPI文書がVue版ドキュメントの役割を代替する。
- `cargo fmt`、`cargo clippy`、native/WASMテスト、`cargo publish --dry-run`が通る。
- Vue版を参照できるGitタグを作成したうえで、`main`からNode系資産を削除する。

Monaco、Markdown、アプリシェルなどのP4統合機能は、同一実装ではなく、egui向けの
置換手段と拡張点が用意されていれば完全移植に含めます。
