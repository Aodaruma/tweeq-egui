# コンポーネント移植表

## 優先度

- **P0**: 全入力が依存する基盤
- **P1**: 最初の利用可能リリースに必要な中核入力
- **P2**: クリエイティブツール向けの主要入力
- **P3**: 高度な入力、レイアウト、補助UI
- **P4**: アプリ統合。互換APIではなく置換手段を提供してよい

進捗は`未着手`、`設計中`、`実装中`、`検証中`、`完了`のいずれかで更新します。

Phase 3時点では基盤と`Number`に加え、P1/P2入力を対話型ギャラリーへ配置しました。
この段階の目的はAPI・値保存形式・操作primitiveの共通化であり、Vue版の全ショートカットや
視覚効果との完全一致ではありません。個別の差異とPhase 5での判定は
[Phase 3検証記録](./phase-3-gallery.md)を参照してください。

Phase 5のVue実測後の判定は[互換性評価](./compatibility-report.md)に集約しています。
この表の「実装先が存在する」ことと、main切替に必要なUX互換性の達成は区別します。

## 基盤

| Vue/TypeScript | Rust移植先 | 優先度 | 方針・受入条件 |
|---|---|---:|---|
| `validator.ts` | `tweeq-core::validation` | P0 | clamp/quantize/composeを`f64`境界値とNaN方針込みでテスト |
| `useDrag.ts` | `tweeq-core::gesture` + egui adapter | P0 | click/drag、raw motion、修飾キー、grab fallbackをイベント列で検証 |
| `stores/multiSelect.ts` | `selection` + `EditEvent` | P0 | 参照を保存せずIDで範囲・追加選択、capture、cancelを実現 |
| `InputTextBase` | internal text-edit state | P0 | edit/display分離、IME、Enter/Escape/blurの規則を共通化 |
| `TweakOverlay` | foreground painter | P0 | clip外の軌跡、目盛り、モード表示。pointer eventは奪わない |
| `theme/*`, `stores/theme.ts` | `TweeqTheme` | P0 | light/dark、意味的色、metrics、既存egui Styleとの共存 |
| `TweeqProvider`, `useTweeq` | `TweeqContext` | P0 | UI一時状態とapp値を分離し、複数viewportでID衝突しない |
| `Popover`, `Tooltip`, `Menu` | shared overlay primitives | P1 | focus復帰、outside click、Escape、viewport edge回避 |
| `Icon`, `SvgIcon`, `BindIcon`, `ColorIcon`, `IconIndicator` | icon/painter module | P1 | 埋込assetまたは描画primitive。Web fetchを必須にしない |

## 入力

| Vueコンポーネント | Rust名（仮） | 優先度 | 方針・受入条件 |
|---|---|---:|---|
| `InputNumber` | `Number` | P1 | 絶対/相対scrub、縦感度、step/snap/clamp、text edit、Undo境界 |
| `InputRotary` | `Rotary` | P1 | A/R mode、複数回転、snap、drag軌跡、angle offset |
| `InputAngle` | `Angle` | P1 | 幅に応じたRotary + Number合成、単一編集セッション |
| `InputButton` | `Button` | P1 | label/icon/disabled、ellipsis時のtooltip |
| `InputButtonToggle` | `ToggleButton` | P1 | pointerとSpace/Enterで同じ値・イベントを生成 |
| `InputCheckbox` | `Checkbox` | P1 | click、左右swipe、真偽キー、選択状態 |
| `InputSwitch` | `Switch` | P1 | Checkboxと操作意味論を共有 |
| `InputRadio` | `Radio` | P1 | responsive layout、keyboard navigation、label fallback |
| `InputDropdown` | `Dropdown` | P1 | filter/type-ahead、確定、focus復帰、icon/label |
| `InputString` | `StringInput` | P1 | IME、commit/cancel、同時編集時の式または置換規則 |
| `InputGroup` | `InputGroup` | P1 | 連結位置と共通border/radiusをテーマから描画 |
| `InputPosition` | `Position` | P2 | 2D drag、X/Y拘束、座標系方向、複数値delta |
| `InputVec` | `Vector` | P2 | 2〜4成分、compact切替、成分ごとのfocus/commit |
| `InputSize` | `Size` | P2 | aspect lock、ゼロ・負値方針、単一Undo単位 |
| `InputTranslate` | `Translate` | P2 | Position/Vectorの共通gestureを再利用 |
| `InputTime` | `Timecode` | P2 | frame rate、SMPTE/frame表示、H/M/S/F/Q操作、drop-frame方針明示 |
| `InputDrum` | `Drum` | P2 | drag/wheel/key/type-ahead、慣性の決定的テスト |
| `InputColor`, `GlslCanvas` | `ColorInput` | P2 | HSVA/RGBA、channel shortcut、relative multi-edit、CPU texture |
| `InputCubicBezier` | `CubicBezier` | P3 | handle制約、数値入力、curve preview、keyboard代替 |
| `InputShuffle` | `Shuffle` | P3 | generatorをcoreへ分離し、seed指定で再現可能にする |
| `InputComplex` | `ComplexInput` | P3 | Number等の合成として実装し独自gestureを増やさない |
| `InputCode` | `CodeInput` adapter | P4 | ホスト提供editorとの接続点。Monaco互換を要件にしない |

## レイアウト・アプリ部品

| Vueコンポーネント | Rust移植先 | 優先度 | 方針・受入条件 |
|---|---|---:|---|
| `ParameterGrid`, `ParameterHeading`, `ParameterGroup`, `Parameter` | parameter layout | P2 | 狭幅で崩れず、labelと入力のfocus関係を維持 |
| `Tabs` | `Tabs` | P3 | keyboard navigation、overflow、永続ID |
| `PaneExpandable` | `CollapsingPane` | P3 | egui標準挙動を土台にTweeqテーマを適用 |
| `PaneFloating` | `FloatingPane` | P3 | viewport内制約、移動、resize、永続位置 |
| `PaneModal`, `PaneModalComplex`, `PaneModalTabs` | modal primitives | P3 | focus trap相当、Escape、重なり順、small viewport |
| `PaneSplit` | split adapter | P3 | まずegui primitiveで実装し、dock依存は任意featureで検討 |
| `PaneZUI` | ZUI adapter | P4 | pan/zoom意味論と埋込APIを提供。完全なDOM互換は不要 |
| `Ruler` | `Ruler` | P3 | zoom/offset/tick選択の純粋関数をcoreでテスト |
| `Timeline` | `Timeline` | P3 | selection、scrub、zoom、scrollを段階実装。データモデルはホスト所有 |
| `Viewport` | viewport helpers | P3 | coordinate transform、overlay、input routingだけを担当 |
| `CommandPalette` | command-palette adapter | P4 | 検索・実行interfaceを提供し、app command modelは所有しない |
| `TitleBar` | demo/app shell | P4 | ライブラリ中核へ含めず、custom decoration例として提供 |
| `Balloon` | overlay primitive | P3 | Popover/Tooltipと配置エンジンを共有 |
| `MultiSelectPopup`, `MultiSelectPad`, `MultiSelectButton` | selection UI | P3 | core selectionの表示・操作に限定し、値参照を保持しない |
| `Markdown` | demo documentation | P4 | Rustデモの説明表示へ置換。汎用Markdown widgetは提供しない |
| `MonacoEditor` | external integration | P4 | Rust再実装しない。CodeInput adapterの利用例を用意 |
| `App` | `tweeq-demo` | P4 | コンポーネントギャラリー、試験設定、native/WASM入口 |

## 移植判定の共通チェック

各行を`完了`へ変更するには、該当する項目を満たす必要があります。

- 通常、hover、focus、active、disabled、invalidを確認した。
- pointer、keyboard、必要な場合はtouchで操作できる。
- `Begin`と`Commit`が一操作につき一度だけ発生し、Escapeで`Cancel`できる。
- responsive/compact表示で安定したIDとfocusを保つ。
- light/dark、100%/150%/200% scaleで致命的なclipがない。
- nativeとWASMの差異を試験または既知制限として記録した。
- Vue版と異なる操作は、理由と代替操作を文書化した。
