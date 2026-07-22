# Vue / Rust component parity audit

最終更新: 2026-07-22

この表はVue版の公開コンポーネントを基準に、現在の`crates/tweeq-egui`と
`tweeq-demo`を静的に比較したものです。「Rust名が存在する」だけでは移植完了とせず、
Tweeq固有の操作・active時の描画・keyboard操作・disabled/invalidを含めて判定します。

## 判定基準

| 判定 | 意味 |
|---|---|
| 実装済み | 中核の役割と操作が揃い、残件が主に細部または実機検証である |
| 近似 | 対応するRust widget / adapterはあるが、Tweeq固有の操作または見た目に大きな差がある |
| 未対応 | 公開された同等widgetがない、または同名でも役割が異なり代替にならない |

現状はprototype段階です。2026-07-22のparity passで主要入力のactive描画と
keyboard経路を大幅に更新しましたが、複数選択・式入力・OS別pointer制御まで含む
完全互換として「実装済み」と判定できる入力はまだありません。

## 入力コンポーネント

| Vue | Rust | 判定 | 主な不足・確認事項 |
|---|---|---|---|
| `InputNumber` | `Number` | 近似（高） | bar origin、min/max、step目盛り、clamp、動的precision、disabled/invalid、active感度dots、Shift/Alt+矢印を実装。text focus中のscrub、式入力、複数選択の相対式、OS別pointer fallback実測が残る |
| `InputRotary` | `Rotary` | 近似（高） | 中心まわりdrag、A/R、snap帯、周囲目盛り、多回転arc、pointerラベル、cursor非表示を実装。copy/paste、複数選択とhit areaの細部、native/WASM差を要検証 |
| `InputAngle` | `Angle` | 近似（高） | Rotary + Numberが同一`ParamId`を共有し、snap/offset/disabled/invalidを伝播。狭幅でNumberを隠すresponsive挙動とfocus中scrubが残る |
| `InputButton` | `Button` | 近似 | Tweeq配色、hover/focus、disabled/invalidを実装。focus/pressed時のegui expansionを無効化し、横幅が動かないようにした。icon、chevron、tooltip、blink、narrow、連結角の細部が不足 |
| `InputButtonToggle` | `ToggleButton` | 近似 | boolean更新と専用状態描画、disabled/invalidを実装。icon、連結角、transitionの細部が不足 |
| `InputCheckbox` | `Checkbox` | 近似（高） | font glyphを廃止してcheckをPainterで描画。click/swipe、真偽shortcut、disabled/invalidに加え、press/drag中のOFF/ON円overlayを実装。連結角、multi-select表示が残る |
| `InputSwitch` | `Switch` | 近似（高） | pill描画、click/swipe、真偽shortcut、disabled/invalid、およびpress/drag中に横へ伸びるhandleを実装。連結属性とtransition curveの細部が残る |
| `InputRadio` | `Radio` | 近似（高） | segmented indicator、drag選択、矢印navigation、disabled/invalid、選択indicatorの補間animationを実装。`.animated(false)`でanimationを無効化できる。icon-only/縦responsive、tooltip/labelizerが残る |
| `InputDropdown` | `Dropdown` | 近似 | disabled/invalid、上下キー、簡易type-aheadを実装。fuzzy検索、viewport scroll、icons、prefix/suffix、popup visual/focus復帰が残る |
| `InputDrum` | `Drum` | 近似（高） | 中央mark・隣接peek・ticks・fade、40px/step drag、wheel、矢印、type-ahead、prefix/suffixを実装。Pointer Lock、慣性、wrap方針が残る |
| `InputString` | `StringInput` | 近似 | edit session、Enter commit、Escape cancel、width/disabled/invalidを実装。default reset、validator公開API、theme/font/align、IME、multi-edit式が残る |
| `InputGroup` | `InputGroup` | 近似 | horizontal groupingのみ。子へstart/middle/endを付ける連結角、vertical direction、2px group gapが同等でない |
| `InputPosition` | `Position` | 近似（高） | compact `Translate + Vector`合成、min/max/step、disabled/invalidを実装。responsive幅と単一Undo sessionを要検証 |
| `InputTranslate` | `Translate` | 近似（高） | 小型grid button、全画面grid/value label、X/Y・0/1拘束、Q snap、矢印、pointer lockを実装。scale補間animationとOS別fallbackが残る |
| `InputVec` | `Vector<N>` | 近似（高） | X/Y/Z/W label、min/max、step、compact layout、disabled/invalidを実装。成分別stepと複合Undo単位が残る |
| `InputSize` | `Size` | 近似（高） | 左右2ループと中央線でchain iconを描画し、link時のみ中央線を表示。比率維持、ゼロ除算保護、disabled/invalidを実装。外部比率変更と単一edit sessionを要検証 |
| `InputTime` | `Timecode` | 近似（高） | Painter製時計icon、表示文字列から測定したH/M/S/F hit領域、drag中の単位lock、blur、右クリックのSMPTE/Frames切替、単位別drag、Q snap、range/disabled/invalidを実装。式とdrop-frameが残る |
| `InputColor` | `ColorInput` | 近似（高） | 専用SV pad、Hue/Alpha slider、HSV/RGB/HEX切替、hex入力、swatches、checkerboardを実装。dragではS/V、Shift/H/F、S、V、Alt/A、R/G/Bを切替え、foreground HSV overlayを表示する。複数色のVue同等relative edit、preset外部注入、disabled/invalidが残る |
| `InputCubicBezier` | `CubicBezier` | 近似（高） | compact curve + popup picker、非hover時の白curve、control line/handle、Q snap、矢印、4数値代替、disabled/invalidを実装。presetと複合edit sessionの整合が残る |
| `InputShuffle` | `Shuffle` | 近似 | 文字buttonを1〜6のPainter製dice faceへ変更し、seed更新ごとに面も変化。Rustはseed付き数値range専用で、Vueのgeneric `generate(prev)`と任意型の変更eventとはAPIが異なる |
| `InputComplex` | `ComplexInput` | 未対応 | Vueはschema-driven object editor、Rustの同名型は実数/虚数の2 Numberで別用途。Rust側は名称と設計の再検討が必要 |
| `InputCode` | `CodeInput` | 近似 | Rustはegui multiline editor。Monaco、言語別highlight、theme、completion等は意図的にhost adapter候補 |

## パラメータレイアウト・overlay・基盤

| Vue | Rust | 判定 | 主な不足・確認事項 |
|---|---|---|---|
| `ParameterGrid` | demo内`egui::Grid` helper | 近似 | 公開widgetではない。subgrid相当、狭幅、label/value column、theme gapをライブラリAPI化する必要あり |
| `Parameter` | demo内`row` helper | 近似 | label icon、hint tooltip、slot、focus関係が不足 |
| `ParameterGroup` | `CollapsingPane` | 近似 | group heading、subgrid維持、animated collapse、heading-rightが不足 |
| `ParameterHeading` | heading text | 近似 | right slot、spacing、共通theme treatmentが未公開 |
| `TweakOverlay` | Number/Rotary内foreground描画 | 近似 | 公開共通primitiveではない。複数widgetでのlayer/order、pointer透過、viewport越境、native/Web差を統一する必要あり |
| `TweeqProvider` | `TweeqContext` | 実装済み | providerの役割はhost-owned contextへ置換。複数viewportのID衝突、overlayのframe lifecycleは継続検証 |
| `MultiSelectPopup` | core selection + edit events | 近似 | ID選択とsession基盤は存在するが、popup/pad/button UI、選択範囲表示、型別相対操作が不足 |
| `Popover` | なし | 未対応 | anchor配置、viewport edge回避、outside click、Escape、focus復帰が必要 |
| `Tooltip` / `TooltipRoot` | egui tooltipを一部直接利用 | 近似 | Tweeq共通の構造化title/description、遅延、top layer、styleの公開primitiveがない |
| `Balloon` | なし | 未対応 | Popover/Tooltipと共有する配置・surface primitiveとして未実装 |
| `Menu` | なし | 未対応 | submenu、keyboard navigation、context menu、focus管理が未実装 |
| `Icon` | 文字・emoji・egui painterで代用 | 未対応 | Iconify相当asset strategy、icon名API、offline/native埋込方針が必要 |
| `SvgIcon` | painter shapeを個別実装 | 近似 | 汎用SVG wrapperなし。必要iconをRust primitive/assetへ変換する方針が必要 |
| `BindIcon` | なし | 未対応 | binding状態・表示をselection/edit modelへ接続するUIが未実装 |
| `ColorIcon` | color swatchを個別描画 | 近似 | checkerboard/alpha、contrast、共通size/styleの公開部品がない |
| `IconIndicator` | なし | 未対応 | indicator state APIと描画が未実装 |
| `InputTextBase`（内部基盤） | Number/Stringに個別実装 | 近似 | focus/display/edit分離、IME、Enter/Escape/blur、prefix/icon、reset menuを共通化できていない |
| `GlslCanvas`（内部基盤） | なし | 未対応 | Colorは当面CPU textureまたはegui picker。GPU callbackはoptional backendとして判断が必要 |

## workspace / appコンポーネント

| Vue | Rust | 判定 | 主な不足・確認事項 |
|---|---|---|---|
| `Tabs` / `Tab` | `Tabs` | 近似 | 選択と左右キーのみ。Vueのslot、indicator、overflow、focus navigation、disabled状態が不足 |
| `PaneExpandable` | `CollapsingPane` | 近似 | egui標準collapse。Tweeqのpane visual、resize/collapse event、animationが異なる |
| `PaneFloating` | `FloatingPane` | 近似 | egui Window adapter。viewport margin、独自title/resize handles、永続layout、z-orderの差がある |
| `PaneModal` | なし | 未対応 | top-layer modal、focus trap相当、Escape、outside click、small viewport対応が必要 |
| `PaneModalComplex` | なし | 未対応 | `InputComplex`自体もVue schema editorと非同等 |
| `PaneModalTabs` | なし | 未対応 | modal primitiveとTabs統合後に実装 |
| `PaneSplit` | なし | 未対応 | egui/egui_tiles/egui_dock adapterの選定が必要 |
| `PaneZUI` | `Viewport2D` | 近似 | pan/zoom gridだけの概念実証。pane content transform、gesture routing、fit/reset等が不足 |
| `Ruler` | `Ruler` | 近似 | nice tick描画は存在。orientation、zoom/offset追従、interactive cursor、Vueのvisual詳細が不足 |
| `Timeline` | `Timeline` | 近似 | drag/click開始時に単独edit selectionとfocusを取得し、直前のNumber等へ同時適用されないようにした。なお1本のscrubberのみで、tracks/keyframes/selection/zoom/scroll/edit modelが不足 |
| `Viewport` | `Viewport2D` | 近似 | Vueのroot reset/theme/scroll containerとは役割が異なる。Rust側のviewport helper境界を明文化する必要あり |
| `CommandPalette` | `CommandPalette` | 近似 | query filterと実行は存在。fuzzy ranking、keyboard selection、command metadata、modal visual/focusが不足 |
| `TitleBar` | demo shellのみ | 未対応 | custom decoration、window controls、menu統合はライブラリ中核外としてexample化する方針 |
| `App` | `tweeq-demo::GalleryApp` | 近似 | gallery shellは存在。Vue referenceとのside-by-side state coverage、WASM entry、automated screenshot baselineが不足 |
| `Markdown` | なし | 未対応 | 汎用Markdownは移植対象外候補。demo docsは別crate/host責務にする |
| `MonacoEditor` | `CodeInput` adapter | 近似 | Monaco完全移植は対象外候補。host editor接続interfaceとexampleを用意する |

## Vue単体リファレンスページ

`port-reference.html`はVuePressの説明・navigationを含まず、既存Vueコンポーネントだけを
並べた比較用ページです。次で起動します。

```sh
npm run dev:reference
```

`InputNumber`では次を同一画面で変更・比較できます。

- `bar`とbar origin、`min` / `max`
- `step`、`precision`
- `clampMin` / `clampMax`
- `disabled` / `invalid`
- stepped + clamped、範囲外、unrangedの固定比較例

`InputAngle`ではwide compositionとcompact rotaryを同時表示し、`snap`、
`angleOffset`、`disabled`、`invalid`を確認できます。主要なboolean、choice、vector、
time、color、cubic Bézier、shuffle、schema-based complex inputも同じページに配置しています。

## `InputComplex`という名前の相違

Vue版の`InputComplex`は複素数入力ではありません。schemaに並べた異種fieldを辿り、
`InputNumber`、`InputAngle`、`InputColor`、`InputVec`などを組み合わせて任意objectを編集する
schema-driven editorです。一方、現在のRust `ComplexInput`は`Re` / `Im`という2個の`Number`を
横に並べただけの複素数専用controlであり、原版と用途もデータモデルも一致しません。

prototype中は互換性のため型名を残しますが、stable前には次の整理を推奨します。

- 現在の型を`ComplexNumber`へrenameする。
- `ComplexInput`または`SchemaInput`を、host側のschema / value adapterを受け取る原版相当APIへ予約する。
- demoでも「Complex number (Rust-only)」と明示し、原版相当として数えない。

## 2026-07-22追加フィードバックの実機確認

- 公式galleryでInputColor popup、InputCheckbox / Switch / Size / Timeの構造と操作説明を確認。
- Windows native galleryでColor popup、SV/Hue/Alpha、swatch保持、相対dragを確認。
- TimeのSMPTE/Frames右クリック切替、他control clickによるblurを確認。
- Numberを編集状態にしてからTimelineをdragし、Numberが変化せずfocusが外れることを確認。
- Checkboxのcheck、Sizeのchain、Cubic Bézierの白curve、Shuffle diceをnative描画で確認。
- active中だけ見えるoverlay/handle伸長は実装コードとstate testを基準に確認。自動化でpress中の
  screenshotを固定するinteraction harnessは今後追加する。

## 次の優先順位

1. Numberのfocus中scrub・式入力と、Rotary/Angleの複数選択event列を固定する。
2. Colorの複数選択relative edit、外部preset API、disabled/invalidを追加する。
3. `ParameterGrid`とPopover/Tooltip/Menuを公開primitiveとして実装する。
4. input群の単一Undo session、OS/WASM pointer fallback、screenshot baselineを検証する。
5. workspace系はexact portとhost adapterを分け、移植対象外をrelease前に確定する。
