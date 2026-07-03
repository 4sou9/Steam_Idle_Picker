use std::fs;
use std::path::PathBuf;

use regex::Regex;

use crate::models::SteamGame;
use crate::steam_client::{get_install_path, SteamClientSession};

use super::vdf;

/// ACF-based scan of installed games. Works without Steam running.
pub fn get_installed_games() -> Vec<SteamGame> {
    let Some(steam_path) = get_install_path() else {
        return Vec::new();
    };

    let mut library_paths = vec![steam_path.join("steamapps")];

    let folders_vdf = steam_path.join("steamapps").join("libraryfolders.vdf");
    if let Ok(content) = fs::read_to_string(&folders_vdf) {
        let re = Regex::new(r#""path"\s+"([^"]+)""#).unwrap();
        for cap in re.captures_iter(&content) {
            let dir = cap[1].replace("\\\\", "\\");
            let lib_dir = PathBuf::from(dir).join("steamapps");
            if lib_dir.is_dir() {
                library_paths.push(lib_dir);
            }
        }
    }

    let id_re = Regex::new(r#""appid"\s+"(\d+)""#).unwrap();
    let name_re = Regex::new(r#""name"\s+"([^"]+)""#).unwrap();

    let mut games = Vec::new();
    for lib_path in library_paths {
        let Ok(entries) = fs::read_dir(&lib_path) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(file_name) = path.file_name().and_then(|f| f.to_str()) else {
                continue;
            };
            if !file_name.starts_with("appmanifest_") || !file_name.ends_with(".acf") {
                continue;
            }
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };

            let Some(id_cap) = id_re.captures(&content) else {
                continue;
            };
            let Some(name_cap) = name_re.captures(&content) else {
                continue;
            };
            let Ok(app_id) = id_cap[1].parse::<u32>() else {
                continue;
            };
            let name = name_cap[1].trim().to_string();
            if name.is_empty() {
                continue;
            }

            games.push(SteamGame { app_id, name });
        }
    }

    games
}

/// AppIDs with play history, sourced from every local user's localconfig.vdf.
pub fn get_local_config_app_ids() -> Vec<u32> {
    let Some(steam_path) = get_install_path() else {
        return Vec::new();
    };

    let user_data_path = steam_path.join("userdata");
    let Ok(entries) = fs::read_dir(&user_data_path) else {
        return Vec::new();
    };

    let mut ids = Vec::new();
    for entry in entries.flatten() {
        let config_path = entry.path().join("config").join("localconfig.vdf");
        if !config_path.is_file() {
            continue;
        }
        let Ok(content) = fs::read_to_string(&config_path) else {
            continue;
        };
        ids.extend(parse_local_config_app_ids(&content));
    }

    ids
}

fn parse_local_config_app_ids(content: &str) -> Vec<u32> {
    let root = vdf::parse(content);
    let Some(software) = root.get("Software") else {
        return Vec::new();
    };
    let Some(valve) = software.get("Valve") else {
        return Vec::new();
    };
    let Some(steam) = valve.get("Steam") else {
        return Vec::new();
    };
    let Some(apps) = steam.get("apps") else {
        return Vec::new();
    };

    apps.children.keys().filter_map(|k| k.parse::<u32>().ok()).collect()
}

/// Fetches the full library: ACF scan (always available) merged with play-history
/// games resolved via steamclient64.dll (only when Steam is running).
/// Never errors — Steam being offline simply means fewer resolved names.
pub fn fetch_local_library() -> (Vec<SteamGame>, usize, usize, bool) {
    let mut games: std::collections::BTreeMap<u32, SteamGame> = std::collections::BTreeMap::new();

    for game in get_installed_games() {
        games.insert(game.app_id, game);
    }
    let installed_count = games.len();

    let mut resolved_count = 0usize;
    let mut session = SteamClientSession::new();
    let connected = session.initialize();

    if connected {
        for app_id in get_local_config_app_ids() {
            if games.contains_key(&app_id) {
                continue;
            }
            let Some(name) = session.get_game_name(app_id) else {
                continue;
            };
            if name.trim().is_empty() {
                continue;
            }
            games.insert(app_id, SteamGame { app_id, name });
            resolved_count += 1;
        }
    }

    let mut list: Vec<SteamGame> = games.into_values().collect();
    list.sort_by_key(|g| g.name.to_lowercase());

    (list, installed_count, resolved_count, connected)
}
