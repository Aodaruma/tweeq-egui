# Phase 4: 高度入力とワークスペースUI検証記録

## 実装したprimitive

| 分類 | Rust API | 実装内容 |
|---|---|---|
| 高度入力 | `CubicBezier` | 2ハンドルdrag、0〜1制約、curve preview、4つのNumber代替入力 |
| 高度入力 | `Shuffle` | host所有seedと`seeded_unit`による決定的な値生成 |
| 高度入力 | `ComplexInput` | 実部・虚部をNumberから合成 |
| レイアウト | `Tabs` | host所有index、クリック、左右キー |
| pane | `CollapsingPane`、`FloatingPane` | egui標準の永続状態・移動・resizeへ薄く接続 |
| 時間UI | `Ruler`、`Timeline` | 1/2/5系列tick、host所有range/current、scrub |
| 空間UI | `ViewTransform`、`Viewport2D` | renderer非依存変換、pan、zoom、grid |
| 統合adapter | `CodeInput` | egui multiline code editor。外部editorへ置換可能な小さな境界 |
| 統合adapter | `CommandPalette` | host所有command配列をfilter・選択してindexを返す |

`nice_tick_step`、`seeded_unit`、`ViewTransform`は`tweeq-core`に置き、egui描画から
独立したunit testを追加しています。pane、command、codeの内容と永続データは
ホストアプリケーションが所有します。

## native実機確認

2026-07-22にWindows nativeギャラリーで次を確認しました。

- Bézierの第1ハンドルをdragし、`[0.25, 0.10]`から`[0.50, 0.621]`へ更新できる。
- Shuffleを押すとseedが41から42へ進み、表示値が0.4200から0.4831へ変わる。
- Timelineのtickとcurrent markerが同一range上へ描画される。
- Viewportタブへ切り替え、dragでoffsetを`[-4.0, -2.0]`から`[-6.8, -3.7]`へ変更できる。
- Ctrl+PでCommand Paletteを開き、host commandを表示できる。
- 高度入力とworkspace UIが既存の縦スクロールギャラリー内でclipせず表示される。

## adapterとした領域

- Monaco Editorは移植しない。`CodeInput`を最小fallbackとし、アプリ固有editorは
  hostが差し替える。
- PaneZUIのDOM構造は移植しない。座標変換とpan/zoom意味論を`ViewTransform`と
  `Viewport2D`で提供する。
- Command Paletteのcommand model、shortcut競合解決、実行権限はhostが所有する。
- split/dockingはegui標準primitiveまたは任意の`egui_tiles`/`egui_dock`統合とし、
  主クレートの必須依存にはしない。
- modal、popover、tooltipはegui標準のWindow/Popup/Responseを優先する。Tweeq固有の
  配置やfocus規則が必要と確認できた部分だけ共通化する。

## Phase 4 exit gateに対する残件

このコミットはAPIと対話試作の完了であり、ロードマップ上の全品質ゲート通過ではありません。

- selection popup、複合modal、split/dockingの具体的な利用例
- 大量パラメーター、長時間drag、HiDPIの計測
- WASMブラウザーでのPointer/keyboard/focus実機確認
- BézierハンドルdragとSize aspect変更を単一Undo単位へまとめるhost例

これらはPhase 5の互換性表で公開可否を判定し、未達のものをpreview releaseの既知制限へ
引き継ぎます。
