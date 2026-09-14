export interface SteamGame {
  AppId: number;
  Name: string;
}

export interface GameCache {
  FetchedAt: string;
  Games: SteamGame[];
}

export type FilterMode = "all" | "favorites" | "idling";

export interface AppSettings {
  Language: string;
  SelectedGames: number[];
  Favorites: number[];
  Filter: FilterMode;
}

export interface FetchResult {
  cache: GameCache;
  installedCount: number;
  resolvedCount: number;
  connected: boolean;
}

export type FailureReason = "steamNotRunning" | "launchFailed" | "steamClosed" | "exited";

export interface IdleFailure {
  appId: number;
  reason: FailureReason;
}

export type SortMode = "none" | "status" | "name" | "id";
