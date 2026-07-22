# Phase 3: 中核入力ギャラリー検証記録

## 実装範囲

`cargo run -p tweeq-demo`で、次の値を一画面のスクロール可能なギャラリーとして
対話的に確認できます。

- Number、Rotary、Angle
- Button、ToggleButton、Checkbox、Switch、InputGroup
- StringInput、Dropdown、Radio、Drum
- Position、Translate、Vector、Size
- Timecode、ColorInput

すべての入力は`ParamId`で識別され、複合入力は`ParamId::child`で安定した子IDを
生成します。値はアプリケーションが所有し、ウィジェットは編集イベントを
`TweeqContext`へ発行します。nativeとWASMで公開APIと保存形式は共通です。

## 実機確認

2026-07-22にWindows native版をビルドし、コンピュータ操作によって次を確認しました。

- light/darkテーマを実行中に切り替えられる。
- Numberを40pxドラッグすると0.72から0.94へ変化し、累積deltaを二重加算しない。
- Numberへ`0.5`を入力し、Enterで確定できる。
- Positionのパッドをドラッグすると`[24, -12]`から`[44, -2]`へ同時に変化する。
- Drumの次ボタンでCherryからDragonfruitへ移動する。
- Rotaryの横ドラッグで角度が変化する。
- すべてのセクションがスクロール領域内に表示され、致命的なclipがない。

`cargo fmt`、warningをdenyした`cargo clippy`、workspace test、
`wasm32-unknown-unknown`向けcheckも成功しています。

## Vue版との現時点の差異

| 対象 | Phase 3の状態 | Phase 5までの判断対象 |
|---|---|---|
| Rotary/Angle | 相対drag、A絶対モード、Q/Shift snap | 複数回転の軌跡、全画面overlay、合成入力の単一session |
| Switch | 真偽値とイベントは共通 | 専用switch外観とswipe操作 |
| StringInput | egui TextEditによる入力 | focus単位のBegin/Commit/Cancel、式・同時置換規則 |
| Dropdown/Radio | 選択とkeyboard標準操作 | filter/type-ahead、focus復帰、responsive配置 |
| Drum | 前後、wheel、矢印キー | drag、慣性、type-ahead |
| Position/Translate | 2D delta、Shift/Alt、X/Y拘束 | Vue版overlayと座標方向設定、同時編集 |
| Vector/Size | Number合成、aspect lock | 複合入力全体を一つのUndo単位にする |
| Timecode | frame保存、SMPTE表示、Number編集 | H/M/S/F/Qキー、表示モード、drop-frame |
| ColorInput | RGBA保存、egui標準picker | HSVA channel drag、相対同時編集、専用CPU texture |

Phase 3の「完了」は中核入力が同一のRust APIとテーマprimitiveで動くことを意味します。
上表の互換性項目を完了扱いにはせず、Phase 5のVue比較で`移植`、`代替`、`非対応`を
個別に確定します。
