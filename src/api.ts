import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, FetchResult, GameCache, IdleFailure } from "./types";

export const api = {
  loadCache: () => invoke<GameCache | null>("load_cache"),
  refreshLibrary: () => invoke<FetchResult>("refresh_library"),
  startIdle: (appId: number) => invoke<boolean>("start_idle", { appId }),
  stopIdle: (appId: number) => invoke<void>("stop_idle", { appId }),
  stopAll: () => invoke<void>("stop_all"),
  getIdlingIds: () => invoke<number[]>("get_idling_ids"),
  takeIdleFailures: () => invoke<IdleFailure[]>("take_idle_failures"),
  loadSettings: () => invoke<AppSettings>("load_settings"),
  saveSettings: (settings: AppSettings) => invoke<void>("save_settings", { settings }),
  openSteamStore: (appId: number) => invoke<void>("open_steam_store", { appId }),
  openSteamLibrary: (appId: number) => invoke<void>("open_steam_library", { appId }),
};
