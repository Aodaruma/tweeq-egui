# Rust版アーキテクチャ

## 設計原則

Tweeqの移植単位はVueコンポーネントではなく、次の操作意味論です。

- clickとdragの判定
- 絶対・相対調整
- 横または幾何学方向の移動による値変更
- 縦移動、Shift、Altによる感度変更
- snap、quantize、clamp、precision
- テキスト編集とscrubの共存
- 複数パラメーターの選択と同時編集
- `Begin`、`Update`、`Commit`、`Cancel`からなる編集セッション

表示はegui向けに作り直します。DOM、CSS、Pinia、Vueのwatchを模倣する層は
設けません。

## ワークスペース構成

Phase 1ではVue版を残したままルートへCargo workspaceを追加しました。Phase 6以降は
Rust-only workspaceとし、旧実装は`vue-final-reference` tagで参照します。

```text
Cargo.toml
crates/
├─ tweeq-core/
│  └─ src/
│     ├─ edit.rs
│     ├─ expression.rs
│     ├─ gesture.rs
│     ├─ selection.rs
│     ├─ validation.rs
│     └─ value.rs
├─ tweeq-egui/
│  └─ src/
│     ├─ context.rs
│     ├─ overlay.rs
│     ├─ theme.rs
│     ├─ widgets.rs
│     └─ widgets/
│        ├─ number.rs
│        ├─ rotary.rs
│        └─ ...
└─ tweeq-demo/
   └─ src/
      └─ main.rs
```

### `tweeq-core`

- egui、eframe、ウィンドウシステムへ依存しない。
- 値変換、検証、編集イベント、選択、式評価を担当する。
- nativeと`wasm32-unknown-unknown`の両方で動作する依存だけを採用する。
- `tweeq-egui`から公開型をre-exportし、一般利用者が二つのクレートを意識せず
  使えるようにする。

### `tweeq-egui`

- 通常依存は`egui`までとし、`eframe`や特定rendererへ依存しない。
- `Response`、`Sense`、`Painter`、`Area`、viewport commandをTweeqの操作へ変換する。
- ウィジェットごとの一時状態は安定した`egui::Id`で管理する。
- renderer固有のGPUコードは標準機能にしない。

### `tweeq-demo`

- `eframe`を使うコンポーネントギャラリー兼手動試験アプリとする。
- nativeとWASMを同じ画面構成で比較できるようにする。
- 公開対象外（`publish = false`）とする。

## 状態と編集データフロー

即時モードのUIから複数の`&mut T`を長期間保持する設計は採用しません。
アプリケーションが値を所有し、Tweeqは値のスナップショットと編集命令を扱います。

```text
application model
      │ value snapshot + ParamId
      ▼
widget ── interaction ──► TweeqContext
      │                       │ selection/session state
      └──── EditEvent ◄───────┘
                  │
                  ▼
       application applies change
       and records Undo transaction
```

編集イベントの概念形は次のとおりです。phaseとoperationを独立フィールドにせず、
`Begin`や`Commit`に不要なoperationを持たせない形にします。これは実装開始時に
テストと合わせて確定し、公開後はSemVerに従います。

```rust
pub struct ParamId(/* stable application-owned key */);
pub struct EditSessionId(/* unique within a TweeqContext */);

pub struct ParamSnapshot {
    pub id: ParamId,
    pub value: ParamValue,
}

pub enum EditOperation {
    SetNumber(f64),
    AddNumber(f64),
    ScaleNumber(f64),
    AddVector(Vec<f64>),
    SetBoolean(bool),
    SetString(String),
    TransformColor(ColorTransform),
}

pub enum EditEvent {
    Begin {
        session: EditSessionId,
        source: ParamId,
        targets: Vec<ParamSnapshot>,
    },
    Update {
        session: EditSessionId,
        operation: EditOperation,
    },
    Commit { session: EditSessionId },
    Cancel { session: EditSessionId },
}
```

`Begin`は対象値の型付きsnapshotを含み、`Update`はそのsnapshotを基準に適用します。
これにより、加算操作で毎フレーム誤差が蓄積することを防ぎ、`Cancel`では開始値へ
確実に戻せます。また、直接バインドしたsource値がUIフレーム内で先に変化しても、
描画後にイベントを処理するホストが開始値を失いません。

イベントをeguiのレイアウト完了後に適用するホストでもdrag表示が一frame遅れないよう、
active sessionのpreview値は`TweeqContext`にも保持します。ホストへ`Update`を返したら
再描画を要求し、次frameのモデル値とpreviewが一致した時点で同期します。

単一値だけを編集する用途には、`&mut f64`などを受け取る簡便APIも提供できます。
ただし同時編集とUndoを必要とする場合は、ID付きイベントAPIを標準経路とします。

## ID、選択、レイアウト順

- `egui::Id`: テキストバッファ、drag中の感度、popupなどUI内部状態に使う。
- `ParamId`: アプリケーションモデル、同時編集、Undoイベントに使う。
- ラベル文字列や表示順だけから`ParamId`を生成しない。
- Shift範囲選択は、各フレームで登録された`Rect`とレイアウト順を使う。
- Ctrl/Command追加選択とfocusは別状態として扱う。
- 非表示になったIDは選択集合から遅延削除し、瞬間的なレスポンシブ切替で選択が
  不必要に失われないようにする。

## ジェスチャー

共通の`TweakGesture`が次を正規化します。

```rust
pub struct GestureFrame {
    pub motion: [f32; 2],
    pub modifiers: Modifiers,
    pub mode: TweakMode,
    pub speed: f64,
    pub snap_enabled: bool,
}
```

実装順序は次のとおりです。

1. `Sense::click_and_drag()`でclickとdragを区別する。
2. `Response::drag_motion()`からraw motionが得られる場合はそれを使う。
3. 必要な間だけカーソル非表示・grabをviewportへ要求する。
4. grab不能または拒否時は、通常の`drag_delta`へ自動的に退化する。
5. タッチではPointer Lockを前提にせず、画面内dragと明示的ハンドルを使う。

OSやホストごとにカーソル制御能力が異なるため、値変換ロジックとカーソル制御を
分離します。無限ドラッグの成否は公開APIの成功条件に含めず、`PointerPolicy`と
実行時の能力を結果に含めます。

## 数値入力の縦切り

`InputNumber`相当を最初の縦切り実装にします。この一つで、次を検証できます。

- 範囲ありの絶対dragと、範囲なしの相対scrub
- フォーカス前は領域全体、フォーカス中はgripだけをdrag対象にすること
- 横移動による値変更と、縦移動による連続的な感度変更
- Shift高速化、Alt微調整、Q snap
- `step`、`snap`、`min/max`、clamp、表示precision
- Enter/blurでcommit、Escapeでcancel
- 右クリックからdefault値へreset
- 同時編集とUndo境界

内部状態は少なくとも`display_buffer`、`captured_value`、`accumulated_delta`、
`gesture_speed`、`editing_text`を分離します。表示用の丸め値をモデルへ毎フレーム
書き戻してはなりません。

## テキスト入力と式

テキスト編集中はモデル値と文字列バッファを分離します。

- Enter: parse、validate、commit
- Escape: capture値へ戻してcancel
- blur: ウィジェット設定に従いcommitまたはcancel
- 矢印キー: step単位の変更
- double click: 全選択
- parse失敗: モデルを変更せずinvalid状態を表示

式入力は任意feature `expressions`とし、JavaScript `eval`は使いません。許可するのは
`x`、`i`、四則演算、括弧、明示した数学関数・定数だけです。ファイル、ネットワーク、
アプリケーションの任意関数にはアクセスできない評価器とします。

## 描画とテーマ

`TweeqTheme`はeguiの`Visuals`だけでは表せない意味的トークンを保持します。

- accent、accent hover、accent soft
- background、surface、input、input hover
- text、muted、subtle、border
- error、warning、success、info、recording
- input height、popup width、radius、4段階のgap、pane margin

アプリ全体へ適用する`install_style`は任意の補助APIとし、ウィジェットは個別テーマを
受け取れるようにします。これにより既存eguiアプリのStyleを強制的に上書きしません。

drag中の目盛り、軌跡、モード表示は`Order::Foreground`のlayerへ描きます。CSSの
`backdrop-filter`は半透明surfaceとshadowで近似し、背景ぼかしを必須要件にしません。

Color pickerはまずCPUで`ColorImage`を生成し、色相やサイズが変わったときだけ
textureを更新します。GPU callbackはプロファイル結果が必要性を示した場合に限り、
renderer別の任意featureとして検討します。

## 公開APIの方向性

eguiのbuilder形式に合わせつつ、標準の`Response`に編集情報を追加して返します。

```rust
let result = tweeq_egui::Number::new(
    ParamId::from_static("opacity"),
    opacity,
)
.range(0.0..=1.0)
.step(0.01)
.snap(0.1)
.precision(3)
.show(ui, &mut self.tweeq);

for event in result.events() {
    self.parameters.apply(event);
}
```

正確な型名は最初の`Number`実装で決定しますが、次の性質は維持します。

- builderは一時オブジェクトで、フレームをまたぐ参照を保持しない。
- `TweakResponse`から元の`egui::Response`へアクセスできる。
- changedとcommittedを区別できる。
- Widget内からアプリケーションのUndo実装を直接呼ばない。
- popupやoverlayを使わない最小構成でも動作する。

## テスト戦略

| 層 | 試験 |
|---|---|
| core | clamp、quantize、snap、precision、式、色変換の単体・property test |
| gesture | Begin/Update/Commit/Cancel、修飾キー切替、capture基準の再現試験 |
| egui | 合成`RawInput`によるclick、drag、keyboard、focus試験 |
| visual | 固定フォント・固定scaleで主要状態のsnapshot比較 |
| integration | eframe nativeとWASMで同じ操作シナリオを実行 |
| manual | Windows/macOS/Linux/browserのPointer Lock、IME、touch、HiDPI確認 |

snapshot差分だけで操作互換性を判定せず、値とイベント列を主な検証対象にします。

## アクセシビリティ

- 主要操作にはdrag以外のキーボード経路を用意する。
- focus順を安定させ、状態を`WidgetInfo`へ伝える。
- 色だけでfocus、invalid、selectedを表現しない。
- tooltipを唯一のラベルにしない。
- WASMのIME、スクリーンリーダー、ソフトウェアキーボードの制約を既知差異として
  検証表へ記録する。

## 参照資料

- [egui `Response`](https://docs.rs/egui/latest/egui/response/struct.Response.html)
- [egui `Sense`](https://docs.rs/egui/latest/egui/struct.Sense.html)
- [eframe](https://docs.rs/eframe/latest/eframe/)
- Vue参照実装: `vue-final-reference` tagの`src/`と`docs/`、およびupstream
