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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(rename = "Language")]
    pub language: String,
    #[serde(rename = "SelectedGames")]
    pub selected_games: Vec<u32>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "ja".into(),
            selected_games: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchResult {
    pub cache: GameCache,
    pub installed_count: u32,
    pub resolved_count: u32,
    pub connected: bool,
}
