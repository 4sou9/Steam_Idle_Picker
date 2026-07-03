use std::fs;
use std::path::PathBuf;

use crate::models::{AppSettings, FetchResult, GameCache};

use super::steam_library;

fn app_data_dir() -> PathBuf {
    let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
    PathBuf::from(base).join("SteamIdlePicker")
}

fn cache_path() -> PathBuf {
    app_data_dir().join("games_cache.json")
}

fn settings_path() -> PathBuf {
    app_data_dir().join("settings.json")
}

pub fn load_cache() -> Option<GameCache> {
    let content = fs::read_to_string(cache_path()).ok()?;
    serde_json::from_str(&content).ok()
}

fn save_cache(cache: &GameCache) {
    let _ = fs::create_dir_all(app_data_dir());
    if let Ok(json) = serde_json::to_string_pretty(cache) {
        let _ = fs::write(cache_path(), json);
    }
}

pub fn load_settings() -> AppSettings {
    let Ok(content) = fs::read_to_string(settings_path()) else {
        return AppSettings::default();
    };
    serde_json::from_str(&content).unwrap_or_default()
}

pub fn save_settings(settings: &AppSettings) {
    let _ = fs::create_dir_all(app_data_dir());
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = fs::write(settings_path(), json);
    }
}

/// Blocking: scans the local Steam library and persists the result. Intended to be
/// called from a spawn_blocking context since it touches the filesystem/registry
/// and may talk to steamclient64.dll.
pub fn fetch_and_cache_library() -> FetchResult {
    let (games, installed_count, resolved_count, connected) = steam_library::fetch_local_library();
    let cache = GameCache {
        fetched_at: chrono::Local::now(),
        games,
    };
    save_cache(&cache);

    FetchResult {
        cache,
        installed_count: installed_count as u32,
        resolved_count: resolved_count as u32,
        connected,
    }
}
