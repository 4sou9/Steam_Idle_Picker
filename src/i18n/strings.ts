export interface Strings {
  RefreshLibrary: string;
  SearchPlaceholder: string;
  IdlingPrefix: string;
  IdlingGames: string;
  IdleStart: string;
  IdleStop: string;
  LoadingLibrary: string;
  LoadError: string;
  NoCache: string;
  SortName: string;
  SortId: string;
  StatusLabel: string;
  StatusRunning: string;
}

export const ja: Strings = {
  RefreshLibrary: "ライブラリ読み込み",
  SearchPlaceholder: "ゲームを検索...",
  IdlingPrefix: "起動中: ",
  IdlingGames: "ゲーム",
  IdleStart: "▶ アイドル開始",
  IdleStop: "■ アイドル停止",
  LoadingLibrary: "ライブラリを読み込み中...",
  LoadError: "読み込みエラー: ",
  NoCache: "「ライブラリ読み込み」を押してゲーム一覧を取得してください",
  SortName: "名前",
  SortId: "ID",
  StatusLabel: "ステータス: ",
  StatusRunning: " 稼働中",
};

export const en: Strings = {
  RefreshLibrary: "Load Library",
  SearchPlaceholder: "Search games...",
  IdlingPrefix: "Running: ",
  IdlingGames: " games",
  IdleStart: "▶ Start Idling",
  IdleStop: "■ Stop Idling",
  LoadingLibrary: "Loading library...",
  LoadError: "Load error: ",
  NoCache: "Click 'Load Library' to fetch your game list",
  SortName: "Name",
  SortId: "ID",
  StatusLabel: "Status: ",
  StatusRunning: " running",
};

export function detectLanguage(): "ja" | "en" {
  return navigator.language.toLowerCase().startsWith("ja") ? "ja" : "en";
}

export function getStrings(lang: "ja" | "en"): Strings {
  return lang === "ja" ? ja : en;
}
