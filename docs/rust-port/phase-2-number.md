# Phase 2: Number縦切り検証記録

## 実装済み

- deterministicな`ParamId`
- 型付き開始値を含む`Begin`、`Update`、`Commit`、`Cancel`
- spreadsheet型の単一・追加・範囲選択
- clamp、step quantize、Q snap
- 横dragによる値変更、縦dragによる連続感度変更
- Shift高速化、Alt微調整
- click-to-edit、矢印キー、Enter確定、Escape取消、blur確定
- default値へのcontext-menu reset
- range barとforeground tweak overlay
- unbounded入力向けの任意Pointer Lock要求と通常drag fallback

## GUI確認

Windows native版の`Tweeq egui gallery`で次を確認しました。

1. dark/lightテーマを切り替えられる。
2. Opacityの40px右dragで`0.72`から`0.94`へ変化する。
3. `0.5`をテキスト入力し、Enterで値と`Commit`イベントが確定する。
4. `Begin`のsnapshotを基準に`AddNumber`を適用し、直接バインド値を二重加算しない。
5. range bar、選択border、suffix、イベントログが更新される。

GUI検証中、eguiの累積drag量をフレーム差分として扱っていた問題と、ホストが
描画後に`Begin`を処理するとsourceの開始値を失う問題を検出しました。前者は累積量の
前回値との差分を使い、後者は`Begin`へ型付き`ParamSnapshot`を含めることで解決済みです。

## 自動検証

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo check --workspace --all-features --target wasm32-unknown-unknown`

coreではID、range selection、gesture accumulation、fine/fast sensitivity、縦感度、
quantize、clamp、非有限値方針を単体試験しています。

## Phase 3へ持ち越す事項

- Pointer Lock成功・拒否時のOS別確認
- 同時選択した複数NumberのGUI操作確認
- IMEとWASMのテキスト入力確認
- expression parser
- Number/Rotary/Angle間で共有するgesture primitiveの抽出
