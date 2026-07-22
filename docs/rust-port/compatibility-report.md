# Phase 5: Vue / Rust互換性評価

> この文書の表はPhase 5開始時点のhistorical snapshotです。その後のparity passで
> Number、Angle/Rotary、Boolean、Timecode、Color、choice、geometry、advanced inputを
> 改善しました。2026-07-22、InputSize icon等の既知差分をalpha制限として受け入れ、
> Phase 6のRust-only切替へ進みました。最新判断は
> [Phase 6切替記録](./phase-6-rust-switch.md)と
> [component parity監査](./component-parity-audit.md)を参照してください。

## 結論

- `tweeq-core`と`tweeq-egui`は`0.1.0-alpha.1`のpreview候補としてpackage検証へ進める。
- Numberを中心とする編集基盤とパラメーターギャラリーは、利用者が対話的に評価できる。
- Vue版をmainから削除する判断は**No-Go**。Color、Timecode、Drum、Rotary overlay、
  複合Undo、WASM実機試験が切替ブロッカーである。
- Monaco、ZUI、dock、Command Paletteは完全複製せず、egui/host adapterを正式方針とする。

これは「Phase 5の評価作業が完了した」という意味であり、「完全移植が完了した」という
意味ではありません。

## 比較条件と証拠

| 対象 | 結果 |
|---|---|
| Vue基準 | `82205f396bc8f3b3eb16a2c6202681e6d71e1ae6` |
| Vue unit test | 4件成功 |
| VuePress build | 17ページ生成に成功。`eval`、Viewport循環chunk、SSR `renderSlot`例外を記録 |
| 公開Vue版 | 2026-07-22に`components.html`をブラウザーでDOM・style・入力操作確認 |
| Rust native | WindowsでNumber、Rotary、Position、Drum、Bézier、Shuffle、Viewport、Paletteを操作確認 |
| Rust自動試験 | fmt、warning deny clippy、workspace test、WASM check成功 |

ブラウザーの画像取得では公開ページが背景のみになる制約があったため、Vueの視覚比較は
DOMの実寸、computed style、表示文字、実入力後の状態を併用しました。Numberへの`0.5`
入力とEnter確定、ArrowUpによる`1.5`への増加、DrumのAppleからBananaへの移動、
Color popupのchannel群表示は実操作で確認しています。

## 見た目の実測比較

| 項目 | Vue実測 | Rust Phase 5 | 判定 |
|---|---:|---:|---|
| 共通input高 | 24px | 24pt | 維持 |
| 標準幅 | 240px | Number/Dropdownを240ptへ調整 | 維持 |
| 角丸 | 4px | 4pt | 維持 |
| 文字 | 12px | Numberは12pt monospace | 概ね維持 |
| light input色 | `rgb(239,240,243)` | 同値へ調整 | 維持 |
| Rotary | 24×24px | 24×24ptへ調整 | 維持 |
| Angle | Rotary 24 + Number 207、gap 9 | Rotary 24 + Number 118 | 部分互換 |
| Color | 24 swatch + 164 code + 48 alpha | egui標準swatch + host表示 | ブロッカー |
| Drum | 240×24、隣接項目を覗かせるtrack | 前/中央118/次ボタン | ブロッカー |
| Position | 24 button + 213 vector、全体240×24 | 180×82 drag pad | 意図的差異を再評価 |

eguiはOS scaleに応じるpoint座標なので、pxとptの物理ピクセル一致は要求しません。
ここでは論理寸法と情報密度を比較しています。

## 機能判定

判定は次の4種類です。

- **移植**: alpha利用者へ提供でき、Vueの主要な操作意味論を保つ。
- **代替**: egui/host標準を正式な置換手段とする。
- **preview制限**: API評価はできるが、差異を明記してalpha提供する。
- **切替ブロッカー**: Vue削除前に実装または明示的な合意が必要。

| 領域 | 判定 | 現在の内容 | 残件 |
|---|---|---|---|
| Theme/metrics | 移植 | light/dark、24高、4角丸、意味色 | 全semantic tokenとfocus/invalid状態 |
| ParamId/EditEvent | 移植 | Begin/Update/Commit/Cancel、型付きcapture | 複合値を一sessionへ束ねるAPI |
| Number | 移植 | text、drag、縦感度、Shift/Alt/Q、矢印、clamp/step | 制限式入力、blur方針設定、OS別grab |
| Button/Toggle/Checkbox | preview制限 | 値・click・keyboard標準 | swipe、真偽shortcut、icon/tooltip |
| Switch | 切替ブロッカー | 真偽イベントを共有 | 48×24 track外観、swipe/shortcut |
| Radio | 移植 | host所有String、選択 | responsive折返しの調整 |
| Dropdown | preview制限 | egui ComboBox | filter/type-ahead、focus復帰 |
| StringInput | preview制限 | TextEdit、SetString event | focus単位session、Escape、式/複数置換 |
| Rotary/Angle | 切替ブロッカー | A絶対、相対drag、snap、Number合成 | 多回転軌跡、全画面overlay、単一session |
| Position/Translate | preview制限 | 2D pad、X/Y、Shift/Alt | Vueのcompact構成かpad構成かを決定 |
| Vector | 移植 | 2〜4 Number合成 | component label、responsive compact |
| Size | preview制限 | aspect lock | width/height連動を一Undo単位にする |
| Drum | 切替ブロッカー | button/wheel/矢印、wrap | slot-machine外観、drag、慣性、type-ahead |
| Timecode | 切替ブロッカー | frame保存、SMPTE表示、Number編集 | 右click単位、H/M/S/F/Q、drop-frame |
| ColorInput | 切替ブロッカー | RGBA、egui標準picker | code/alpha群、HSVA/RGBA shortcut、相対multi-edit |
| CubicBezier | preview制限 | handle、preview、数値代替 | keyboard focus、複合Undo、overlay |
| Shuffle/Complex | 移植 | 決定的seed、Number合成 | host preset例 |
| Tabs/Ruler/Timeline | preview制限 | 基本描画とhost所有値 | overflow、selection、zoom/scroll |
| Pane/Popover/Modal | 代替 | egui Window/CollapsingHeader/Popup | focus規則の共通テスト |
| ZUI/Viewport | 代替 | ViewTransform、pan/zoom grid | host scene/tool routing例 |
| Monaco/InputCode | 代替 | CodeInput fallback | editor traitと外部editor例 |
| Command Palette | 代替 | host command配列をfilter・選択 | shortcut競合、非同期command |
| Multi-select UI | 切替ブロッカー | core selectionとNumber同時編集 | popup/pad表示、全型の相対編集 |

## 維持するUXと変更を許容するUX

維持するもの:

- 小さい入力高、安定した修飾キー、drag中の連続感度変更。
- click-to-editとdrag-to-tweakの共存。
- 値のpreviewとUndo境界を分離する編集session。
- light/darkに依存しないsemantic token。

Rust/egui向けに変更を許容するもの:

- DOM top-layerはegui foreground layerへ置換する。
- Monaco、dock tree、app command modelはhost統合にする。
- CSS backdrop blurは半透明surfaceとshadowへ退化できる。
- Pointer Lock不能環境では通常dragへ退化し、操作可能性を優先する。

## 次の実装順

1. ColorInputの専用compact fieldとchannel操作。
2. Rotaryのmulti-turn overlayとAngle単一session。
3. Drumのtrack/drag/慣性。
4. Timecodeの単位切替とH/M/S/F/Q。
5. 複合値・複数選択のUndo/Cancel統合。
6. WASMギャラリーの配布入口とChrome/Firefox/Safari系試験。

上記を通過してから`vue-final-reference`タグを作り、Vue削除だけのPhase 6 PRへ進みます。
