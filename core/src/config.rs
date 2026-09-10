use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GameEngine {
    Unknown,
    Unity,
    UnrealEngine4,
    UnrealEngine5,
}

impl Default for GameEngine {
    fn default() -> Self { GameEngine::Unknown }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GameConfig {
    pub process_name: String,
    pub discord_app_id: String,
    #[serde(default)]
    pub scan_type: ScanType,
    #[serde(default)]
    pub engine: GameEngine,
    pub steam_app_id: Option<u32>,
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
    #[serde(default = "default_discord_app_id")]
    pub discord_app_id: String,
    #[serde(default = "default_language")]
    pub language: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            scan_interval_ms: default_scan_interval(),
            log_level: default_log_level(),
            discord_app_id: default_discord_app_id(),
            language: default_language(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    #[serde(default = "default_scan_interval")]
    pub scan_interval_ms: u64,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_discord_app_id")]
    pub discord_app_id: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub games: Vec<GameConfig>,
}

fn default_scan_interval() -> u64 { 3000 }
fn default_log_level() -> String { "info".to_string() }
fn default_discord_app_id() -> String { String::new() }
fn default_language() -> String { "en".to_string() }

pub fn xgs_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("XGameStats")
}

impl GameConfig {
    pub fn detect_engine(process_path: &Path) -> GameEngine {
        let dir = process_path.parent().unwrap_or(Path::new(""));
        
        if dir.join("UnityPlayer.dll").exists() || dir.join("UnityPlayer_w64_s.dll").exists() {
            log::info!("Detected Unity engine game");
            return GameEngine::Unity;
        }
        
        if dir.join("UE4Editor.exe").exists() || 
           dir.join("UE5Editor.exe").exists() ||
           dir.join("Engine\\Binaries\\ThirdParty\\Windows\\64\\d3d11.dll").exists() {
            log::info!("Detected Unreal Engine 5 game");
            return GameEngine::UnrealEngine5;
        }
        
        if dir.join("Engine\\Binaries\\ThirdParty\\Windows\\64\\d3d11.dll").exists() ||
           dir.join("UE4Game.exe").exists() ||
           dir.join("Engine\\Binaries\\Win64\\UE4Editor.exe").exists() {
            log::info!("Detected Unreal Engine 4 game");
            return GameEngine::UnrealEngine4;
        }
        
        GameEngine::Unknown
    }

    pub fn get_engine_pointers(engine: &GameEngine) -> HashMap<String, PointerConfig> {
        let mut pointers = HashMap::new();
        
        match engine {
            GameEngine::Unity => {
                pointers.insert("player_name".to_string(), PointerConfig {
                    base: "UnityPlayer.dll+0x01A0E830".to_string(),
                    offsets: Some(vec![0x0, 0x18, 0x28, 0x10]),
                });
                pointers.insert("level".to_string(), PointerConfig {
                    base: "UnityPlayer.dll+0x01A0E830".to_string(),
                    offsets: Some(vec![0x0, 0x18, 0x30, 0x20]),
                });
                pointers.insert("health".to_string(), PointerConfig {
                    base: "UnityPlayer.dll+0x01A0E830".to_string(),
                    offsets: Some(vec![0x0, 0x18, 0x38, 0x18]),
                });
                pointers.insert("character_class".to_string(), PointerConfig {
                    base: "UnityPlayer.dll+0x01A0E830".to_string(),
                    offsets: Some(vec![0x0, 0x18, 0x40, 0x10]),
                });
            }
            GameEngine::UnrealEngine4 | GameEngine::UnrealEngine5 => {
                pointers.insert("player_name".to_string(), PointerConfig {
                    base: "Engine.dll+0x0389F1A0".to_string(),
                    offsets: Some(vec![0x30, 0x28, 0x0, 0x18]),
                });
                pointers.insert("level".to_string(), PointerConfig {
                    base: "Engine.dll+0x0389F1A0".to_string(),
                    offsets: Some(vec![0x30, 0x28, 0x0, 0x20]),
                });
                pointers.insert("health".to_string(), PointerConfig {
                    base: "Engine.dll+0x0389F1A0".to_string(),
                    offsets: Some(vec![0x30, 0x28, 0x0, 0x28]),
                });
                pointers.insert("character_class".to_string(), PointerConfig {
                    base: "Engine.dll+0x0389F1A0".to_string(),
                    offsets: Some(vec![0x30, 0x28, 0x0, 0x30]),
                });
            }
            _ => {}
        }
        
        pointers
    }

    pub fn get_engine_rpc_template(engine: &GameEngine) -> RpcTemplate {
        match engine {
            GameEngine::Unity => RpcTemplate {
                details: "{character_class}".to_string(),
                state: "{game_state}".to_string(),
                large_image: None,
                large_image_text: Some("Strike Force Heroes".to_string()),
                small_image: None,
                small_image_text: None,
            },
            GameEngine::UnrealEngine4 | GameEngine::UnrealEngine5 => RpcTemplate {
                details: "{character_class}".to_string(),
                state: "{game_state}".to_string(),
                large_image: None,
                large_image_text: Some("Strike Force Heroes".to_string()),
                small_image: None,
                small_image_text: None,
            },
            _ => RpcTemplate {
                details: "Playing {game}".to_string(),
                state: "{game_state}".to_string(),
                large_image: None,
                large_image_text: None,
                small_image: None,
                small_image_text: None,
            }
        }
    }

    pub fn find_game_icon(process_path: &Path) -> Option<PathBuf> {
        let dir = process_path.parent()?;
        let exe_stem = process_path.file_stem()?.to_str()?;
        
        let icon_candidates = [
            format!("{}.ico", exe_stem),
            format!("{}.png", exe_stem),
            format!("{}.jpg", exe_stem),
            format!("{}.jpeg", exe_stem),
            "icon.ico".to_string(),
            "icon.png".to_string(),
            "logo.ico".to_string(),
            "logo.png".to_string(),
            "favicon.ico".to_string(),
            "favicon.png".to_string(),
        ];
        
        for candidate in &icon_candidates {
            let icon_path = dir.join(candidate);
            if icon_path.exists() {
                log::info!("[XGS] Found game icon: {}", icon_path.display());
                return Some(icon_path);
            }
        }
        
        log::debug!("[XGS] No local icon found in {}", dir.display());
        None
    }

    pub fn find_steam_app_id(process_path: &Path) -> Option<u32> {
        let dir = process_path.parent()?;
        let exe_name = process_path.file_stem()?.to_str()?;
        
        let mut steam_library_paths: Vec<PathBuf> = Vec::new();
        
        let default_steam = PathBuf::from("C:\\Program Files (x86)\\Steam\\steamapps");
        if default_steam.exists() {
            steam_library_paths.push(default_steam);
        }
        
        let local_app_data = dirs::data_local_dir().unwrap_or_default();
        let config_path = local_app_data.join("Steam\\config\\libraryfolders.vdf");
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                let stripped = trimmed.strip_prefix('"').unwrap_or(trimmed);
                if let Some(key_end) = stripped.find('"') {
                    let key = &stripped[..key_end];
                    if key == "path" {
                        let after_key = stripped[key_end + 1..].trim();
                        let after_key = after_key.strip_prefix('"').unwrap_or(after_key);
                        if let Some(val_end) = after_key.find('"') {
                            let path = &after_key[..val_end];
                            let steamapps = PathBuf::from(path).join("steamapps");
                            if steamapps.exists() {
                                steam_library_paths.push(steamapps);
                            }
                        }
                    }
                }
            }
        }
        
        let known_steam_paths = [
            "D:\\Steam\\steamapps",
            "D:\\Games\\Steam\\steamapps",
            "D:\\SteamLibrary\\steamapps",
            "E:\\Steam\\steamapps",
            "E:\\SteamLibrary\\steamapps",
            "F:\\Steam\\steamapps",
            "F:\\SteamLibrary\\steamapps",
        ];
        
        for path in &known_steam_paths {
            let p = PathBuf::from(path);
            if p.exists() && !steam_library_paths.contains(&p) {
                steam_library_paths.push(p);
            }
        }
        
        for steam_path in &steam_library_paths {
            if let Ok(entries) = std::fs::read_dir(steam_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("acf") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Some(id) = Self::parse_steam_app_id(&content) {
                                if let Some(install_dir) = Self::parse_steam_install_dir(&content) {
                                    if exe_name.to_lowercase() == install_dir.to_lowercase() ||
                                       dir.to_string_lossy().to_lowercase().contains(&install_dir.to_lowercase()) {
                                        log::info!("[XGS] Found Steam App ID: {} for {}", id, install_dir);
                                        return Some(id);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        None
    }

    fn parse_steam_app_id(content: &str) -> Option<u32> {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("\"appid\"") {
                if let Some(value) = trimmed.split_once("\"") {
                    if let Some(id_str) = value.1.trim().strip_prefix("\"").and_then(|s| s.strip_suffix("\"")) {
                        if let Ok(id) = id_str.parse::<u32>() {
                            return Some(id);
                        }
                    }
                }
            }
        }
        None
    }

    fn parse_steam_install_dir(content: &str) -> Option<String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("\"installdir\"") {
                if let Some(value) = trimmed.split_once("\"") {
                    if let Some(dir) = value.1.trim().strip_prefix("\"").and_then(|s| s.strip_suffix("\"")) {
                        return Some(dir.to_string());
                    }
                }
            }
        }
        None
    }

    pub fn get_steam_capsule_url(app_id: u32) -> String {
        format!("https://cdn.akamai.steamstatic.com/steam/apps/{}/header.jpg", app_id)
    }
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
            discord_app_id: settings.discord_app_id,
            language: settings.language,
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
