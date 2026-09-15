use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamGame {
    #[serde(rename = "AppId")]
    pub app_id: u32,
    #[serde(rename = "Name")]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameCache {
    #[serde(rename = "FetchedAt")]
    pub fetched_at: chrono::DateTime<chrono::Local>,
    #[serde(rename = "Games")]
    pub games: Vec<SteamGame>,
}

/// Missing fields fall back to `AppSettings::default()`, so one absent key does not
/// reset the whole file (and lose the selection and favorites on the next save).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    /// Unused (the UI language follows Windows). Still written so that versions up to
    /// 2.2.0, which require this field, can read the file after a downgrade.
    #[serde(rename = "Language")]
    pub language: String,
    #[serde(rename = "SelectedGames")]
    pub selected_games: Vec<u32>,
    /// Independent of the selection; IDs missing from the list are kept.
    #[serde(rename = "Favorites")]
    pub favorites: Vec<u32>,
    /// "all" | "favorites" | "idling"
    #[serde(rename = "Filter")]
    pub filter: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "ja".into(),
            selected_games: Vec::new(),
            favorites: Vec::new(),
            filter: "all".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchResult {
    pub cache: GameCache,
    pub connected: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_keep_present_fields_when_others_are_missing() {
        let s: AppSettings = serde_json::from_str(r#"{"SelectedGames":[1,2],"Favorites":[3]}"#).unwrap();
        assert_eq!(s.selected_games, vec![1, 2]);
        assert_eq!(s.favorites, vec![3]);
        assert_eq!(s.language, "ja");
        assert_eq!(s.filter, "all");
    }
}
