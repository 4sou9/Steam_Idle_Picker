use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::models::SteamGame;
use crate::steam_client::{get_active_user, get_install_path, SteamClientSession};

use super::vdf;

/// ACF-based scan of installed games. Works without Steam running.
pub fn get_installed_games() -> Vec<SteamGame> {
    let Some(steam_path) = get_install_path() else {
        return Vec::new();
    };

    let mut games = Vec::new();
    for lib_path in library_paths(&steam_path) {
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
            games.extend(parse_app_manifest(&content));
        }
    }

    games
}

/// `steamapps` folders of every library, without duplicates. libraryfolders.vdf
/// usually lists the Steam install folder itself as well.
fn library_paths(steam_path: &Path) -> Vec<PathBuf> {
    let mut paths = vec![steam_path.join("steamapps")];
    let folders_vdf = steam_path.join("steamapps").join("libraryfolders.vdf");
    if let Ok(content) = fs::read_to_string(&folders_vdf) {
        paths.extend(
            parse_library_folders(&content)
                .into_iter()
                .map(|dir| dir.join("steamapps"))
                .filter(|dir| dir.is_dir()),
        );
    }

    let mut seen = HashSet::new();
    paths.retain(|p| seen.insert(p.to_string_lossy().trim_end_matches(['\\', '/']).to_lowercase()));
    paths
}

/// Library root folders listed in libraryfolders.vdf.
fn parse_library_folders(content: &str) -> Vec<PathBuf> {
    let root = vdf::parse(content);
    let mut folders: Vec<(&String, &vdf::VdfNode)> = root.children.iter().collect();
    // Keys are "0", "1", ...; keep the file's order for a stable scan order.
    folders.sort_by_key(|(key, _)| key.parse::<u32>().unwrap_or(u32::MAX));
    folders
        .into_iter()
        .filter_map(|(_, folder)| folder.value_of("path"))
        .map(PathBuf::from)
        .collect()
}

/// AppID and name from an appmanifest_*.acf file.
fn parse_app_manifest(content: &str) -> Option<SteamGame> {
    let root = vdf::parse(content);
    let app_id = root.value_of("appid")?.parse::<u32>().ok()?;
    let name = root.value_of("name")?.trim();
    if name.is_empty() {
        return None;
    }
    Some(SteamGame {
        app_id,
        name: name.to_string(),
    })
}

/// AppIDs with play history from localconfig.vdf. Only the logged-in account is read,
/// so games of other accounts on this PC (which cannot be idled) are not listed. Falls
/// back to every local account when the active one cannot be determined.
pub fn get_local_config_app_ids() -> Vec<u32> {
    let Some(steam_path) = get_install_path() else {
        return Vec::new();
    };
    let user_data_path = steam_path.join("userdata");

    if let Some(user) = get_active_user() {
        let config_path = user_data_path.join(user.to_string()).join("config").join("localconfig.vdf");
        if let Ok(content) = fs::read_to_string(&config_path) {
            return parse_local_config_app_ids(&content);
        }
    }

    let Ok(entries) = fs::read_dir(&user_data_path) else {
        return Vec::new();
    };

    let mut ids = Vec::new();
    for entry in entries.flatten() {
        let config_path = entry.path().join("config").join("localconfig.vdf");
        let Ok(content) = fs::read_to_string(&config_path) else {
            continue;
        };
        ids.extend(parse_local_config_app_ids(&content));
    }

    ids
}

fn parse_local_config_app_ids(content: &str) -> Vec<u32> {
    let root = vdf::parse(content);
    let apps = root
        .get("Software")
        .and_then(|n| n.get("Valve"))
        .and_then(|n| n.get("Steam"))
        .and_then(|n| n.get("apps"));
    let Some(apps) = apps else {
        return Vec::new();
    };

    apps.children.keys().filter_map(|k| k.parse::<u32>().ok()).collect()
}

/// Fetches the full library: ACF scan (always available) merged with play-history
/// games resolved via steamclient64.dll (only when Steam is running).
/// Never errors — Steam being offline simply means fewer resolved names.
/// Returns the games sorted by name and whether Steam was reachable.
pub fn fetch_local_library() -> (Vec<SteamGame>, bool) {
    let mut games: BTreeMap<u32, SteamGame> = BTreeMap::new();

    for game in get_installed_games() {
        games.insert(game.app_id, game);
    }

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
        }
    }

    let mut list: Vec<SteamGame> = games.into_values().collect();
    list.sort_by_key(|g| g.name.to_lowercase());

    (list, connected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_app_manifest() {
        let game = parse_app_manifest(
            r#""AppState"
            {
                "appid"     "431730"
                "universe"  "1"
                "name"      " Aseprite \"Pro\" "
                "UserConfig"
                {
                    "name"  "should not be used"
                }
            }"#,
        )
        .unwrap();
        assert_eq!(game.app_id, 431730);
        assert_eq!(game.name, r#"Aseprite "Pro""#);
    }

    #[test]
    fn skips_manifest_without_name_or_id() {
        assert!(parse_app_manifest(r#""AppState" { "appid" "1" "name" "  " }"#).is_none());
        assert!(parse_app_manifest(r#""AppState" { "name" "Game" }"#).is_none());
        assert!(parse_app_manifest(r#""AppState" { "appid" "abc" "name" "Game" }"#).is_none());
    }

    #[test]
    fn parses_library_folders_in_order() {
        let folders = parse_library_folders(
            r#""libraryfolders"
            {
                "1" { "path" "D:\\SteamLibrary" "apps" { "10" "123" } }
                "0" { "path" "C:\\Program Files (x86)\\Steam" }
            }"#,
        );
        assert_eq!(
            folders,
            vec![PathBuf::from(r"C:\Program Files (x86)\Steam"), PathBuf::from(r"D:\SteamLibrary")]
        );
    }

    #[test]
    fn parses_local_config_app_ids() {
        let mut ids = parse_local_config_app_ids(
            r#""UserLocalConfigStore"
            {
                "Software" { "valve" { "Steam" { "apps" {
                    "440" { "LastPlayed" "1" }
                    "570" { }
                    "not-an-id" { }
                } } } }
            }"#,
        );
        ids.sort();
        assert_eq!(ids, vec![440, 570]);
        assert!(parse_local_config_app_ids(r#""UserLocalConfigStore" { }"#).is_empty());
    }
}
