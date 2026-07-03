# Steam Idle Picker — Tauri 移植仕様書

対象読者: 実装担当のAI/開発者。この文書のみで実装が完結するように書いてある。
既存実装: WPF (.NET 8) 版が本リポジトリにあり、動作仕様の一次ソースとして参照可。

## 1. 概要

選択した Steam ゲーム(最大32本)を同時に「プレイ中」状態にする Windows 専用デスクトップアプリ。
現行の WPF + C# 実装を **Tauri 2 + Rust + Web フロントエンド** に置き換える。

機能は現行版と同一(機能追加なし)。以下が全機能:

1. Steam ライブラリのゲーム一覧取得(インストール済み + プレイ履歴あり)とローカルキャッシュ
2. ゲームのチェック選択(上限32)・検索・ソート
3. 選択ゲームの一括アイドル開始/停止、個別停止
4. 設定の永続化(選択ゲーム)
5. 日本語/英語 UI(OS 設定から自動判定)、ダーク/ライトテーマ(OS 設定から自動判定)
6. 多重起動防止

## 2. 技術スタック

| 層 | 採用技術 |
|---|---|
| シェル | Tauri 2.x(Windows 専用、x64) |
| バックエンド | Rust(stable) |
| フロントエンド | React + TypeScript + Vite(状態管理は React 標準のみ。Redux 等は不要) |
| スタイル | プレーン CSS(CSS 変数でテーマ切替)。UI フレームワーク不使用 |
| アイドル子プロセス | Rust 製 CLI `steam-idle.exe`(`steamworks` crate 使用)— 別バイナリとして同ワークスペースに置く |
| Tauri プラグイン | `tauri-plugin-single-instance` |

Cargo workspace 構成:

```
/                       … リポジトリルート
  src/                  … フロントエンド (Vite)
  src-tauri/            … Tauri 本体 (crate: steam-idle-picker)
  steam-idle/           … アイドル子プロセス (crate: steam-idle, bin)
  Dependencies/steam_api64.dll   … 既存。steam-idle.exe と同じ場所に配置して配布
```

`steam-idle.exe` と `steam_api64.dll` は Tauri の `externalBin` / `resources` としてバンドルし、
実行時はアプリの `engine/` 相当ディレクトリ(リソースディレクトリ)から起動する。

## 3. アイドルの仕組み(コア原理)

- ゲーム1本につき `steam-idle.exe <appid>` を1プロセス起動する。
- `steam-idle.exe` は環境変数 `SteamAppId=<appid>` を設定して `SteamAPI_Init()` を呼び、
  1秒ごとに `SteamAPI_RunCallbacks()` を回し続けるだけ(現行 `SteamIdle/Program.cs` と同じ)。
  Steam クライアントがこのプロセスを「そのゲームをプレイ中」と認識する。
- ウィンドウは出さない(Rust では `windows_subsystem = "windows"` でコンソール非表示)。
- 停止 = プロセス kill。

Rust 版 steam-idle の擬似コード:

```rust
// steam-idle/src/main.rs
fn main() {
    let appid: u32 = std::env::args().nth(1).and_then(|s| s.parse().ok())
        .unwrap_or_else(|| std::process::exit(1));
    std::env::set_var("SteamAppId", appid.to_string());
    let (client, single) = steamworks::Client::init_app(appid)
        .unwrap_or_else(|_| std::process::exit(1));
    loop {
        single.run_callbacks();
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
```

注意: `steamworks` crate が使えない/バージョン問題がある場合は、`steam_api64.dll` を
`libloading` で直接ロードして `SteamAPI_Init` / `SteamAPI_RunCallbacks` を呼ぶ実装でも可。

## 4. Rust バックエンド仕様(src-tauri)

### 4.1 ゲーム一覧取得

2つのソースをマージする(現行 `GameCacheService.FetchLocalLibraryAsync` と同一ロジック):

**(a) インストール済みゲーム — ACF ファイル(常に動作)**
1. レジストリ `HKLM\SOFTWARE\WOW6432Node\Valve\Steam` → `InstallPath`(なければ `HKLM\SOFTWARE\Valve\Steam`)で Steam パス取得。`winreg` crate 使用。
2. `<steam>/steamapps/libraryfolders.vdf` から `"path" "..."` を正規表現で抽出し、各 `<path>/steamapps` をライブラリパスに追加(`\\` → `\` 置換)。
3. 各ライブラリパスの `appmanifest_*.acf` から `"appid" "N"` と `"name" "..."` を正規表現抽出。名前が空ならスキップ。

**(b) プレイ履歴のある未インストールゲーム — steamclient64.dll(Steam 起動時のみ)**
1. `<steam>/userdata/<uid>/config/localconfig.vdf` を VDF パースし、
   `Software > Valve > Steam > apps` 直下のキー(AppID)を列挙。
2. 名前解決は `steamclient64.dll` を `LoadLibraryExW`(`LOAD_WITH_ALTERED_SEARCH_PATH`)+
   事前に `SetDllDirectoryW("<steam>;<steam>/bin")` でロードし、
   `CreateInterface("SteamClient018")` → `CreateSteamPipe` → `ConnectToGlobalUser` →
   `GetISteamApps(user, pipe, "STEAMAPPS_INTERFACE_VERSION001")` → `GetAppData(appid, "name", buf, len)`。
   vtable 呼び出しの詳細(vtable index・シグネチャ)は既存の
   `SteamIdlePicker/SteamClient/` 一式(`SteamClient018.cs`, `SteamApps001.cs`, `NativeWrapper.cs`)を忠実に移植すること。
   Rust では raw pointer + `unsafe` の vtable 呼び出しで実装。
3. 接続失敗(Steam 未起動等)なら (b) をスキップし (a) のみ返す。エラーにしない。
4. 終了時に `ReleaseUser` / `BReleaseSteamPipe` を呼ぶ。

VDF パーサ: `{ }` ネストと `"key" "value"` / `"key" {` を扱う簡易再帰パーサ(現行 `VdfParser.cs` 相当)。crate(`keyvalues-parser` 等)を使ってもよい。

マージ: AppId をキーに (a) 優先で統合し、名前の大文字小文字無視昇順でソート。

### 4.2 キャッシュと設定(JSON ファイル)

保存先: `%APPDATA%/SteamIdlePicker/`(現行版とパス互換にし、既存ファイルをそのまま読めること)

- `games_cache.json` — `{ "FetchedAt": "<ISO日時>", "Games": [{ "AppId": 123, "Name": "..." }] }`
- `settings.json` — `{ "Language": "ja", "SelectedGames": [123, 456] }`

serde のフィールド名は上記 PascalCase に合わせる(`#[serde(rename_all = "PascalCase")]` 等)。
読み込み失敗(不在・破損)は既定値にフォールバックしエラーにしない。

### 4.3 アイドル管理(IdleManager)

- `HashMap<u32, Child>` を `Mutex` で保持する Tauri managed state。
- `start_idle(appid)`: 既に生存中なら true。`steam-idle.exe` をリソースディレクトリから
  引数 `<appid>`、CREATE_NO_WINDOW で spawn。exe 不在なら false。
- `stop_idle(appid)`: kill して map から除去。エラーは握りつぶす。
- `stop_all()`: 全 kill + clear。
- **アプリ終了時(ウィンドウ close / プロセス exit)に必ず `stop_all()`** を呼ぶ
  (Tauri の `on_window_event` の `CloseRequested` / `RunEvent::Exit` でフック)。

### 4.4 Tauri コマンド(IPC API)

```rust
// 一覧: キャッシュ読込(起動時)。なければ None
#[tauri::command] fn load_cache() -> Option<GameCache>;

// 再取得: (a)+(b) を実行しキャッシュ保存。数秒かかるので async
#[tauri::command] async fn refresh_library() -> Result<FetchResult, String>;
// FetchResult { cache: GameCache, installed_count: u32, resolved_count: u32, connected: bool }

#[tauri::command] fn start_idle(app_id: u32) -> bool;
#[tauri::command] fn stop_idle(app_id: u32);
#[tauri::command] fn stop_all();
#[tauri::command] fn is_idling(app_id: u32) -> bool;      // Child の生存確認(try_wait)
#[tauri::command] fn get_idling_ids() -> Vec<u32>;         // 生存中プロセスの appid 一覧

#[tauri::command] fn load_settings() -> AppSettings;
#[tauri::command] fn save_settings(settings: AppSettings);
```

フロント→Rust の型は camelCase/PascalCase 変換に注意(Tauri は引数を camelCase で受ける)。

### 4.5 多重起動防止

`tauri-plugin-single-instance` を使用。2つ目の起動時は既存ウィンドウを前面化(`set_focus`)。

## 5. フロントエンド仕様

### 5.1 画面構成(単一ウィンドウ・単一画面)

現行 `MainWindow.xaml` / `screenshot.png` 参照。レイアウト:

```
┌──────────────────────────────────────────────┐
│ [検索ボックス(placeholder: ゲーム名で検索)]  [🔄更新] │  ← ツールバー
│ ステータス: 3/32 稼働中   選択: 5   [選択解除]        │
│ ┌─ ヘッダ行: [状態▲] [名前] [AppID] ←クリックでソート ─┐ │
│ │ ☑ ▶稼働中  Portal 2               620    [■停止] │ │
│ │ ☑          Terraria               105600         │ │
│ │ ☐          ...                                   │ │
│ └──────────────────────────────────────────┘ │
│              [▶ アイドル開始 / ■ すべて停止](トグル大ボタン)│
└──────────────────────────────────────────────┘
```

要素詳細:

- **検索**: 入力ごとに名前の部分一致(大文字小文字無視)でフィルタ。
- **更新ボタン**: `refresh_library` 呼出し。実行中はスピナー表示 + ボタン無効化。
  完了/失敗でステータスメッセージ表示。初回起動でキャッシュなしの場合
  「更新ボタンでライブラリを読み込んでください」の旨を表示。
- **行**: チェックボックス、稼働中インジケータ(▶ + アクセント色)、ゲーム名、AppID、
  稼働中の行のみ個別停止ボタン。
- **ソートヘッダ**: 状態/名前/AppID の3種。同じヘッダ再クリックで昇降トグル、
  アクティブなヘッダに ↑/↓ を表示。状態ソート昇順=稼働中が先頭。
  初期状態はソートなし(キャッシュ順=名前昇順)。
- **トグルボタン**: 非アイドル時「アイドル開始」(選択0なら無効)、アイドル中「停止」。
- **カウンタ**: 稼働数 `n/32`、選択数。1秒間隔のタイマーで `get_idling_ids` をポーリングし
  表示を同期(子プロセスが外部要因で死んだ場合の反映。アイドル中のみポーリングでよい)。

### 5.1.1 状態管理の必須原則(現行版バグの再発防止)

**現行 WPF 版の既知バグ**: 検索(またはソート)でリストを再構築すると「稼働中 n/32」の
カウントが正しく更新されなくなる。原因は、選択・稼働状態を行ごとの ViewModel フラグ
(`IsSelected`/`IsIdling`)として持ち、フィルタのたびに表示コレクションを clear/re-add して
バインディング経由の書き戻しが状態を壊すため。**この設計を持ち込まないこと。**

Tauri 版では以下を必須とする:

1. **Single source of truth**: 状態は画面リストと分離した3つだけ。
   - `games: Game[]` — 全ゲーム(キャッシュ由来、不変データ)
   - `selectedIds: Set<number>` — 選択中 AppID
   - `idlingIds: Set<number>` — 稼働中 AppID(真実はバックエンド。start/stop の結果と
     ポーリング `get_idling_ids` の返値で**丸ごと置換**する)
2. **表示リストは純粋な派生値**: フィルタ・ソート結果は `useMemo(games, searchText, sortMode, idlingIds)`
   で計算するだけ。表示リストの生成・破棄が `selectedIds` / `idlingIds` を変更してはならない。
3. **カウンタも派生値**: 選択数 = `selectedIds.size`、稼働数 = `idlingIds.size`。
   表示中(フィルタ後)のリストから数えない。
4. チェックボックスの onChange は「`selectedIds` を更新する」以外の副作用を持たない
   (アイドル中の即時 start/stop は `selectedIds` 変更に対する明示的なハンドラで行う)。

受け入れテスト: アイドル中に検索文字列を入力・削除、ソートを何度も切り替えても、
稼働数・選択数・チェック状態が一切変化しないこと。

### 5.2 挙動ルール(現行 MainViewModel と同一)

- 選択上限 32。超過するチェックは即座に取り消す(33個目はチェックできない)。
- チェック変更のたびに `SelectedGames` を settings.json へ保存。
- **アイドル中にチェックを付ける → 即 start_idle、外す → 即 stop_idle。**
- 個別停止 → その行の選択も解除。全稼働が0になったらアイドル状態フラグを解除。
- 「選択解除」ボタン → 全チェック解除(アイドル停止はしない。※現行挙動どおり保存のみ)。
- 更新(refresh)しても現在の選択は AppId ベースで引き継ぐ。
- 起動時: `load_cache` + `load_settings` で一覧と選択状態を復元。

### 5.3 i18n

- 起動時に `navigator.language`(または Tauri の locale API)が `ja` で始まれば日本語、それ以外は英語。
- 文字列リソースは TS のオブジェクト2枚(`ja.ts` / `en.ts`)。キーは現行
  `Resources/Strings.ja.xaml` / `Strings.en.xaml` の全キーを踏襲し、訳文もそこから転記する。

### 5.4 テーマ / ビジュアルデザイン(Windows 11 ライク)

情報量・要素構成は 5.1 のまま変えず、見た目を Windows 11 Fluent Design に寄せる。

- `window.matchMedia("(prefers-color-scheme: dark)")` で自動判定 + 変更追従。手動切替 UI は無し。
- **Mica 風背景**: Tauri の `window-effects`(`applyEffects` / tauri.conf の `windowEffects: ["mica"]`)で
  Mica を適用し、コンテンツ背景は半透明オーバーレイにする。Mica 非対応環境では
  不透明のソリッド背景(ダーク `#202020` / ライト `#f3f3f3`)にフォールバック。
- **タイポグラフィ**: `font-family: "Segoe UI Variable Text", "Segoe UI", "Yu Gothic UI", sans-serif`。
  本文 14px、キャプション/AppID 12px。
- **アイコン**: Segoe Fluent Icons(Win11 同梱フォント)を現行版と同じグリフで使用
  (検索 E721 / 更新 E72C / 再生 E768 / 停止 E71A / クリア E894 / チェック E73A・E739)。
- **形状**: コントロール角丸 4px、リスト・カード等のコンテナ角丸 8px(WinUI 準拠)。
  ボーダーは1pxの低コントラスト線(ダーク `rgba(255,255,255,0.08)` 程度)。
- **配色**: WinUI トークンに準拠。
  - ダーク: 面 `#2b2b2b`(card)、テキスト `#ffffff` / 二次 `rgba(255,255,255,0.55)`
  - ライト: 面 `#ffffff`、テキスト `#1a1a1a` / 二次 `rgba(0,0,0,0.55)`
  - アクセント: Windows 既定ブルー `#0067c0`(hover `#1975c5`)。稼働中インジケータは緑系
    (現行 `IdleGreenBrush` 準拠)。
  - すべて CSS カスタムプロパティ(`--bg-card`, `--text-primary`, `--accent` 等)で定義し
    `@media (prefers-color-scheme)` で切替。
- **インタラクション**: hover で面色をワントーン変化、pressed で不透明度 0.7、
  無効時 0.35。トランジションは 100–150ms 程度の控えめなもの。
  行 hover はハイライト、チェック済み行はうっすらアクセント面。
- **その他 Win11 らしさ**: スリムスクロールバー(6px、hover で濃く)、検索ボックスは
  下辺のみアクセント色になるフォーカスリング、プライマリボタンはアクセント塗り + 角丸4px。
  過剰な影・グラデーションは使わない(Fluent はフラット基調)。

### 5.5 ウィンドウ

- タイトル「Steam Idle Picker」、初期サイズは現行 MainWindow.xaml の Width/Height に合わせる。
- リサイズ可、最小サイズ設定(概ね 480×400)。
- アイコン: `SteamIdlePicker/Resources/app.ico` を流用。

## 6. ビルド・配布

- `tauri.conf.json`: identifier `com.steamidlepicker.app`、Windows ターゲットのみ。
- バンドル: NSIS インストーラ + ポータブル exe。`steam-idle.exe` と `steam_api64.dll` を
  リソースとして同梱(フロントの資産ではなく exe と同階層 or resources 配下)。
- steam-idle は workspace メンバとしてビルドし、`tauri build` 前に成果物を
  `src-tauri` のリソースへコピーするビルドスクリプト(`beforeBuildCommand`)を用意。

## 7. 受け入れ基準

1. Steam 起動中に更新 → インストール済み + プレイ履歴のゲームが一覧表示される。
2. Steam 未起動で更新 → エラーにならず、インストール済みゲームのみ表示される。
3. ゲームを選択して開始 → Steam のフレンド上で該当ゲームが「プレイ中」になる。
4. アプリを閉じる → すべての steam-idle.exe プロセスが終了している。
5. 33本目のチェックができない。
6. 再起動後、前回の選択状態とゲーム一覧(キャッシュ)が復元される。
7. WPF 版が生成した `%APPDATA%/SteamIdlePicker/` の既存 JSON をそのまま読み込める。
8. 2重起動すると既存ウィンドウがフォーカスされる。
9. OS のダーク/ライト・言語設定に応じて表示が切り替わる。
10. **アイドル中に検索・ソートを繰り返しても、稼働数/選択数/チェック状態が変化しない**
    (現行 WPF 版のバグ修正、5.1.1 参照)。

## 8. 実装順序の推奨

1. workspace 雛形 + Tauri 2 セットアップ + 単一画面の静的 UI
2. steam-idle crate(単体で `steam-idle.exe 620` を叩いて Steam 表示確認)
3. ACF スキャン + キャッシュ/設定 IO + IPC 配線(ここで大半の UI が動く)
4. IdleManager + 終了時 kill + ポーリング
5. steamclient64.dll FFI(最難関。既存 C# を1対1移植)
6. i18n / テーマ / single-instance / バンドル設定
