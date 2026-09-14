use tauri::State;

use crate::idle_manager::{IdleFailure, IdleManager};
use crate::models::{AppSettings, FetchResult, GameCache};
use crate::services::storage;

#[tauri::command]
pub fn load_cache() -> Option<GameCache> {
    storage::load_cache()
}

#[tauri::command]
pub async fn refresh_library() -> Result<FetchResult, String> {
    tauri::async_runtime::spawn_blocking(storage::fetch_and_cache_library)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn start_idle(state: State<IdleManager>, app_id: u32) -> bool {
    state.start_idle(app_id)
}

#[tauri::command]
pub fn stop_idle(state: State<IdleManager>, app_id: u32) {
    state.stop_idle(app_id);
}

#[tauri::command]
pub fn stop_all(state: State<IdleManager>) {
    state.stop_all();
}

#[tauri::command]
pub fn is_idling(state: State<IdleManager>, app_id: u32) -> bool {
    state.is_idling(app_id)
}

#[tauri::command]
pub fn get_idling_ids(state: State<IdleManager>) -> Vec<u32> {
    state.get_idling_ids()
}

/// Helpers that ended on their own since the last call (failed to start, Steam closed).
#[tauri::command]
pub fn take_idle_failures(state: State<IdleManager>) -> Vec<IdleFailure> {
    state.take_failures()
}

#[tauri::command]
pub fn load_settings() -> AppSettings {
    storage::load_settings()
}

#[tauri::command]
pub fn save_settings(settings: AppSettings) {
    storage::save_settings(&settings);
}

/// Opens a URL built here from the AppID, so arbitrary URLs cannot be opened.
fn open_with_shell(url: String) -> Result<(), String> {
    std::process::Command::new("explorer.exe")
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_steam_store(app_id: u32) -> Result<(), String> {
    open_with_shell(format!("https://store.steampowered.com/app/{app_id}/"))
}

#[tauri::command]
pub fn open_steam_library(app_id: u32) -> Result<(), String> {
    open_with_shell(format!("steam://nav/games/details/{app_id}"))
}
