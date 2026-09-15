use std::fs;
use std::path::PathBuf;

use crate::models::{AppSettings, FetchResult, GameCache};

use super::steam_library;

fn appdata() -> PathBuf {
    PathBuf::from(std::env::var("APPDATA").unwrap_or_else(|_| ".".into()))
}

/// Named after the bundle identifier so the uninstaller's "delete app data" option
/// removes it.
fn app_data_dir() -> PathBuf {
    appdata().join("com.steamidlepicker.app")
}

/// Moves settings and cache from `%APPDATA%\SteamIdlePicker` (used up to 2.1.1).
/// Files already present in the new folder win. The old folder is removed only when
/// everything was moved.
pub fn migrate_legacy_data() {
    let legacy = appdata().join("SteamIdlePicker");
    let Ok(entries) = fs::read_dir(&legacy) else {
        return;
    };
    let target = app_data_dir();
    if fs::create_dir_all(&target).is_err() {
        return;
    }
    let mut complete = true;
    for entry in entries.flatten() {
        let from = entry.path();
        if !from.is_file() {
            complete = false;
            continue;
        }
        let to = target.join(entry.file_name());
        if to.exists() {
            let _ = fs::remove_file(&from);
            continue;
        }
        if fs::rename(&from, &to).is_err() && fs::copy(&from, &to).and_then(|_| fs::remove_file(&from)).is_err() {
            complete = false;
        }
    }
    if complete {
        let _ = fs::remove_dir_all(&legacy);
    }
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

/// Writes to a temporary file and renames it over the target, so an interrupted
/// write (crash, power loss) never leaves a truncated JSON file behind.
fn write_atomic(path: &std::path::Path, contents: &str) -> std::io::Result<()> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, contents)?;
    fs::rename(&tmp, path)
}

fn save_cache(cache: &GameCache) {
    let _ = fs::create_dir_all(app_data_dir());
    if let Ok(json) = serde_json::to_string_pretty(cache) {
        let _ = write_atomic(&cache_path(), &json);
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
        let _ = write_atomic(&settings_path(), &json);
    }
}

/// Blocking: scans the local Steam library and persists the result. Intended to be
/// called from a spawn_blocking context since it touches the filesystem/registry
/// and may talk to steamclient64.dll.
pub fn fetch_and_cache_library() -> FetchResult {
    let (mut games, connected) = steam_library::fetch_local_library();

    // Without a Steam connection only installed games can be named. Keep the rest of
    // the previous list instead of shrinking it to the installed games.
    if !connected {
        if let Some(previous) = load_cache() {
            let known: std::collections::HashSet<u32> = games.iter().map(|g| g.app_id).collect();
            games.extend(previous.games.into_iter().filter(|g| !known.contains(&g.app_id)));
            games.sort_by_key(|g| g.name.to_lowercase());
        }
    }
    let cache = GameCache {
        fetched_at: chrono::Local::now(),
        games,
    };
    save_cache(&cache);

    FetchResult { cache, connected }
}
