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
    Aob,
}

impl Default for ScanType {
    fn default() -> Self { ScanType::Offsets }
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
    #[serde(default)]
    pub scan_type: ScanType,
    pub pointers: Option<HashMap<String, PointerConfig>>,
    pub aob_patterns: Option<HashMap<String, AOBConfig>>,
    pub rpc_template: RpcTemplate,
    #[serde(default)]
    pub requires_elevation: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AppSettings {
    #[serde(default = "default_scan_interval")]
    pub scan_interval_ms: u64,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            scan_interval_ms: default_scan_interval(),
            log_level: default_log_level(),
        }
    }
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

fn default_scan_interval() -> u64 { 3000 }
fn default_log_level() -> String { "info".to_string() }

pub fn xgs_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("XGameStats")
}

impl AppConfig {
    pub fn load(config_dir: Option<&str>) -> Result<Self, String> {
        let data_dir = if let Some(d) = config_dir {
            PathBuf::from(d)
        } else {
            xgs_data_dir()
        };

        fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

        let settings_file = data_dir.join("settings.json");
        let settings: AppSettings = if settings_file.exists() {
            let raw = fs::read_to_string(&settings_file).map_err(|e| e.to_string())?;
            serde_json::from_str(&raw).unwrap_or_default()
        } else {
            let def = AppSettings::default();
            let _ = fs::write(&settings_file, serde_json::to_string_pretty(&def).unwrap());
            def
        };

        let mut games = load_game_configs_from_dir(&data_dir);

        if let Ok(exe_dir) = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .ok_or(())
        {
            let dev_configs = exe_dir.join("configs");
            if dev_configs.exists() && dev_configs != data_dir {
                let dev_games = load_game_configs_from_dir(&dev_configs);
                for g in dev_games {
                    if !games.iter().any(|x: &GameConfig| x.process_name == g.process_name) {
                        games.push(g);
                    }
                }
            }
        }

        Ok(AppConfig {
            scan_interval_ms: settings.scan_interval_ms,
            log_level: settings.log_level,
            games,
        })
    }

    pub fn load_game_config(path: &str) -> Result<GameConfig, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }
}

fn load_game_configs_from_dir(dir: &PathBuf) -> Vec<GameConfig> {
    let mut games = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return games,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if name == "settings.json" || name == "config.json" {
            continue;
        }
        if let Ok(raw) = fs::read_to_string(&path) {
            match serde_json::from_str::<GameConfig>(&raw) {
                Ok(gc) => {
                    log::debug!("Loaded game config: {} ({})", gc.process_name, path.display());
                    games.push(gc);
                }
                Err(e) => {
                    log::warn!("Skipping {}: {}", path.display(), e);
                }
            }
        }
    }
    games
}
