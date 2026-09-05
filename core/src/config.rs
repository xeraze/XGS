use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PointerConfig {
    pub base: String,
    pub offsets: Option<Vec<u32>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AOBConfig {
    pub pattern: String,
    pub mask: String,
    pub module: String,
    pub scan_start: Option<u64>,
    pub scan_size: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ScanType {
    Offsets,
    AOB,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RpcTemplate {
    pub details: String,
    pub state: String,
    pub large_image: Option<String>,
    pub large_image_text: Option<String>,
    pub small_image: Option<String>,
    pub small_image_text: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GameConfig {
    pub process_name: String,
    pub discord_app_id: String,
    pub scan_type: ScanType,
    pub pointers: Option<HashMap<String, PointerConfig>>,
    pub aob_patterns: Option<HashMap<String, AOBConfig>>,
    pub rpc_template: RpcTemplate,
    #[serde(default)]
    pub requires_elevation: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    #[serde(default = "default_scan_interval")]
    pub scan_interval_ms: u64,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default)]
    pub games: Vec<GameConfig>,
}

fn default_scan_interval() -> u64 {
    3000
}

fn default_log_level() -> String {
    "info".to_string()
}

impl AppConfig {
    pub fn load(config_dir: Option<&str>) -> Result<Self, String> {
        let base_path = if let Some(dir) = config_dir {
            PathBuf::from(dir)
        } else {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("GamePresenceEngine")
        };

        if !base_path.exists() {
            fs::create_dir_all(&base_path).map_err(|e| e.to_string())?;
        }

        let config_file = base_path.join("config.json");
        if !config_file.exists() {
            let default = AppConfig {
                scan_interval_ms: 3000,
                log_level: "info".to_string(),
                games: vec![],
            };
            let json = serde_json::to_string_pretty(&default).map_err(|e| e.to_string())?;
            fs::write(&config_file, json).map_err(|e| e.to_string())?;
            return Ok(default);
        }

        let content = fs::read_to_string(&config_file).map_err(|e| e.to_string())?;
        let config: AppConfig = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        Ok(config)
    }

    pub fn load_game_config(path: &str) -> Result<GameConfig, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let config: GameConfig = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        Ok(config)
    }
}
