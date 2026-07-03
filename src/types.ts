export interface SteamGame {
  AppId: number;
  Name: string;
}

export interface GameCache {
  FetchedAt: string;
  Games: SteamGame[];
}

export interface AppSettings {
  Language: string;
  SelectedGames: number[];
}

export interface FetchResult {
  cache: GameCache;
  installedCount: number;
  resolvedCount: number;
  connected: boolean;
}

export type SortMode = "none" | "status" | "name" | "id";
