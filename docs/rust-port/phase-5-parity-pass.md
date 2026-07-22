# Phase 5 follow-up: input parity pass

実施日: 2026-07-22

Phase 5のVue比較で判明した「型はあるがTweeq固有のactive UIがない」状態を解消する
ため、入力コンポーネントを原Vueソースと公式galleryに照らして更新しました。

## InputNumber

- 原版にないpointer追従線・値ラベルを削除した。
- `bar`をbooleanだけでなく原点値として指定可能にした。
- `min` / `max`、`clamp_min` / `clamp_max`、`invalid`を独立API化した。
- 常時のstep目盛り、値位置handle、範囲外indicatorを追加した。
- drag中は3層の感度dotsをfield内に描き、縦移動に応じて間隔を変える。
- step、range幅、gesture speedから表示precisionを求め、drag中のみ固定桁表示する。
- focus中の矢印操作をVueに合わせた。stepありではShiftが高速化しAltはstep未満へ
  変更しない。stepなしではShift/Altをそれぞれ高速・微調整として扱う。
- range barのabsolute dragではpress originを基準にし、drag開始閾値分の二重加算を防ぐ。

残件は、text focus中にhandle/gripだけをscrubできるhit area、制限付き式入力、複数
選択の相対式、OS/WASM別pointer grabの実測です。

## InputRotary / InputAngle

- 横方向scrubを廃止し、knob中心まわりの符号付き角度差で値を更新する。
- pointerがtip側から始まった場合のabsolute modeと、それ以外のrelative modeを実装した。
- `A` / `R`によるmode上書き、Shift / Q、および96〜160pxのringによるsnapを実装した。
- foreground layerへ360°のsnap meter、active meter、absolute radial line、relativeの
  multi-turn arcとarrowを描画する。
- cursorをdrag中に隠し、drag originから画面内へclampした値labelと方向chevronを描く。
- `Angle`内のRotaryとNumberが同じ`ParamId`を使い、snap、angle offset、disabled、
  invalidを共有するようにした。

残件は狭幅時にNumberを隠すresponsive composition、copy/paste、複数選択と単一Undo
sessionの実機確認です。

## 横展開

同じ比較passで次も更新しました。

- Button / Toggle / Checkbox / Switch / String
- Radio / Dropdown / Drum / Timecode
- Position / Translate / Vector / Size / CubicBezier

各実装済み範囲と残件は[component parity audit](./component-parity-audit.md)を参照します。
Color、Popover/Menu、workspace全体は引き続き切替ブロッカーです。

## 比較用アプリ

Rust galleryにはInputNumberの全propsとInputAngleのsnap/offset/stateを変更できる欄を
追加しました。また、VuePressのnavigationや説明を除いたVue単体ページを追加しました。

```sh
cargo run -p tweeq-demo
npm run dev:reference
```

Vue単体ページは`http://localhost:5173/port-reference.html`で開きます。

## 検証

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- Windows native galleryの起動、Number absolute drag（`0.75`から`1.50`）、
  focus中の通常/Alt/Shift + 矢印、Angle中心回転drag、各sectionの描画
- in-app browserでVue単体ページをdesktop幅で目視

active overlayはdrag中だけ表示されるため、自動screenshot baselineは今後、合成
`RawInput`または専用interaction harnessで固定します。
