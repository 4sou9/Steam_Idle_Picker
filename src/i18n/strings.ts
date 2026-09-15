export interface Strings {
  RefreshLibrary: string;
  SearchPlaceholder: string;
  ClearSearch: string;
  FilterAll: string;
  FilterFavorites: string;
  FilterIdling: string;
  NoResults: string;
  FavoritesEmpty: string;
  AddFavorite: string;
  RemoveFavorite: string;
  MenuCopyId: string;
  MenuOpenStore: string;
  MenuOpenLibrary: string;
  ErrSteamNotRunning: string;
  ErrLaunchFailed: string;
  ErrSteamClosed: string;
  ErrExited: string;
  ErrStartFailed: string;
  AndOthers: string;
  MaxSelection: string;
  RefreshOffline: string;
  IdleStart: string;
  IdleStop: string;
  LoadingLibrary: string;
  LoadError: string;
  NoCache: string;
  SortName: string;
  SortId: string;
  StatusLabel: string;
  StatusRunning: string;
  Minimize: string;
  Maximize: string;
  Restore: string;
  Close: string;
}

export const ja: Strings = {
  RefreshLibrary: "ライブラリ読み込み",
  SearchPlaceholder: "ゲームを検索...",
  ClearSearch: "検索をクリア",
  FilterAll: "すべて",
  FilterFavorites: "お気に入り",
  FilterIdling: "起動中",
  NoResults: "一致するゲームがありません",
  FavoritesEmpty: "☆ を押すとお気に入りに追加できます",
  AddFavorite: "お気に入りに追加",
  RemoveFavorite: "お気に入りから外す",
  MenuCopyId: "AppID をコピー",
  MenuOpenStore: "Steam ストアを開く",
  MenuOpenLibrary: "Steam ライブラリで開く",
  ErrSteamNotRunning: "Steam が起動していないためアイドルできません",
  ErrLaunchFailed: "開始できませんでした（未所持など）",
  ErrSteamClosed: "Steam が終了したためアイドルを停止しました",
  ErrExited: "アイドルが停止しました",
  ErrStartFailed: "アイドル用のプログラムを起動できませんでした",
  AndOthers: " ほか {n} 本",
  MaxSelection: "同時にアイドルできるのは 32 本までです",
  RefreshOffline: "Steam に接続できないため、前回の一覧を保持しました",
  IdleStart: "▶ アイドル開始",
  IdleStop: "■ アイドル停止",
  LoadingLibrary: "ライブラリを読み込み中...",
  LoadError: "読み込みエラー: ",
  NoCache: "「ライブラリ読み込み」を押してゲーム一覧を取得してください",
  SortName: "名前",
  SortId: "ID",
  StatusLabel: "ステータス: ",
  StatusRunning: " 稼働中",
  Minimize: "最小化",
  Maximize: "最大化",
  Restore: "元に戻す",
  Close: "閉じる",
};

export const en: Strings = {
  RefreshLibrary: "Load Library",
  SearchPlaceholder: "Search games...",
  ClearSearch: "Clear search",
  FilterAll: "All",
  FilterFavorites: "Favorites",
  FilterIdling: "Running",
  NoResults: "No matching games",
  FavoritesEmpty: "Click ☆ to add favorites",
  AddFavorite: "Add to favorites",
  RemoveFavorite: "Remove from favorites",
  MenuCopyId: "Copy AppID",
  MenuOpenStore: "Open Steam store page",
  MenuOpenLibrary: "Open in Steam library",
  ErrSteamNotRunning: "Steam is not running, so the game cannot idle",
  ErrLaunchFailed: "Could not start idling (the game may not be owned)",
  ErrSteamClosed: "Stopped idling because Steam exited",
  ErrExited: "Idling stopped",
  ErrStartFailed: "Could not start the idle helper",
  AndOthers: " and {n} more",
  MaxSelection: "Up to 32 games can idle at the same time",
  RefreshOffline: "Steam is not reachable; kept the previous list",
  IdleStart: "▶ Start Idling",
  IdleStop: "■ Stop Idling",
  LoadingLibrary: "Loading library...",
  LoadError: "Load error: ",
  NoCache: "Click 'Load Library' to fetch your game list",
  SortName: "Name",
  SortId: "ID",
  StatusLabel: "Status: ",
  StatusRunning: " running",
  Minimize: "Minimize",
  Maximize: "Maximize",
  Restore: "Restore Down",
  Close: "Close",
};

export function detectLanguage(): "ja" | "en" {
  return navigator.language.toLowerCase().startsWith("ja") ? "ja" : "en";
}

export function getStrings(lang: "ja" | "en"): Strings {
  return lang === "ja" ? ja : en;
}

/** Strings for the language detected at startup. */
export const t = getStrings(detectLanguage());
