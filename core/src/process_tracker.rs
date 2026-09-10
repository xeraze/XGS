use std::collections::HashSet;
use std::time::{Duration, Instant};

use crate::config::{GameConfig, GameEngine};
use crate::scanner_bridge::Scanner;

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub process_name: String,
    pub game_config: Option<GameConfig>,
    pub start_time: Instant,
}

pub struct ProcessTracker {
    scan_interval: Duration,
    known_games: Vec<GameConfig>,
    active_process: Option<ProcessInfo>,
    last_scan: Instant,
    logged_detection: HashSet<String>,
}

impl ProcessTracker {
    pub fn new(scan_interval_ms: u64, games: Vec<GameConfig>) -> Self {
        ProcessTracker {
            scan_interval: Duration::from_millis(scan_interval_ms),
            known_games: games,
            active_process: None,
            last_scan: Instant::now(),
            logged_detection: HashSet::new(),
        }
    }

    pub fn scan(&mut self) -> Option<ProcessInfo> {
        if self.last_scan.elapsed() < self.scan_interval {
            return self.active_process.clone();
        }

        self.last_scan = Instant::now();

        for game in &self.known_games {
            if Scanner::open(&game.process_name).is_some() {
                let game_was_running = self.active_process.as_ref()
                    .map(|p| p.process_name == game.process_name)
                    .unwrap_or(false);

                let mut game_config = game.clone();
                
                if !game_was_running {
                    self.active_process = None;
                }
                
                if game_config.pointers.is_none() || game_config.pointers.as_ref().map_or(false, |p| p.is_empty()) {
                    let process_path = Self::get_process_path(&game.process_name);
                    if let Some(path) = &process_path {
                        let detected_engine = GameConfig::detect_engine(path);
                        game_config.engine = detected_engine.clone();
                        
                        if !self.logged_detection.contains(&game.process_name) {
                            log::info!("[XGS] Detected game: {} ({:?})", game.process_name, detected_engine);
                        }
                        
                        let engine_str = match detected_engine {
                            GameEngine::Unity => "unity",
                            GameEngine::UnrealEngine4 => "unreal_engine_4",
                            GameEngine::UnrealEngine5 => "unreal_engine_5",
                            _ => "unknown",
                        };

                        if let Some(scanner) = Scanner::open(&game.process_name) {
                            let mut pointers = std::collections::HashMap::new();
                            let mut rpc_template = game_config.rpc_template.clone();
                            
                            if !self.logged_detection.contains(&game.process_name) {
                                log::info!("[XGS] Scanning memory for strings...");
                            }
                            
                            let found_strings = scanner.scan_for_strings(&game.process_name);
                            
                            if !self.logged_detection.contains(&game.process_name) && !found_strings.is_empty() {
                                log::info!("[XGS] Found {} meaningful strings:", found_strings.len());
                                for (i, s) in found_strings.iter().take(20).enumerate() {
                                    log::info!("[XGS]   [{}] {}", i + 1, s);
                                }
                            }
                            
                            if engine_str != "unknown" {
                                if !self.logged_detection.contains(&game.process_name) {
                                    log::info!("[XGS] Scanning memory for {:?} patterns...", detected_engine);
                                }
                                
                                if let Some(found_offsets) = scanner.scan_for_player_data(&game.process_name, engine_str) {
                                    if !self.logged_detection.contains(&game.process_name) {
                                        log::info!("[XGS] Found player data offsets:");
                                    }
                                    
                                    if let Some(health_offset) = found_offsets.health_offset {
                                        pointers.insert("health".to_string(), crate::config::PointerConfig {
                                            base: format!("{}+0x{:X}", game.process_name, health_offset),
                                            offsets: Some(vec![0]),
                                        });
                                        if !self.logged_detection.contains(&game.process_name) {
                                            log::info!("[XGS]   Health: 0x{:X}", health_offset);
                                        }
                                    }
                                    
                                    if let Some(level_offset) = found_offsets.level_offset {
                                        pointers.insert("level".to_string(), crate::config::PointerConfig {
                                            base: format!("{}+0x{:X}", game.process_name, level_offset),
                                            offsets: Some(vec![0]),
                                        });
                                        if !self.logged_detection.contains(&game.process_name) {
                                            log::info!("[XGS]   Level: 0x{:X}", level_offset);
                                        }
                                    }
                                    
                                    if let Some(class_offset) = found_offsets.class_offset {
                                        pointers.insert("character_class".to_string(), crate::config::PointerConfig {
                                            base: format!("{}+0x{:X}", game.process_name, class_offset),
                                            offsets: Some(vec![0]),
                                        });
                                        if !self.logged_detection.contains(&game.process_name) {
                                            log::info!("[XGS]   Class: 0x{:X}", class_offset);
                                        }
                                    }
                                }
                            }
                            
                            if let Some(game_state) = scanner.scan_for_game_state(&game.process_name) {
                                if !self.logged_detection.contains(&game.process_name) {
                                    log::info!("[XGS] Found game state offsets:");
                                }
                                
                                for (state, offset) in &game_state.state_offsets {
                                    pointers.insert(format!("state_{}", state), crate::config::PointerConfig {
                                        base: format!("{}+0x{:X}", game.process_name, offset),
                                        offsets: Some(vec![0]),
                                    });
                                    if !self.logged_detection.contains(&game.process_name) {
                                        log::info!("[XGS]   State '{}': 0x{:X}", state, offset);
                                    }
                                }
                                
                                for (class, offset) in &game_state.class_offsets {
                                    pointers.insert(format!("class_{}", class), crate::config::PointerConfig {
                                        base: format!("{}+0x{:X}", game.process_name, offset),
                                        offsets: Some(vec![0]),
                                    });
                                    if !self.logged_detection.contains(&game.process_name) {
                                        log::info!("[XGS]   Class '{}': 0x{:X}", class, offset);
                                    }
                                }
                            }
                            
                            let orig_large = rpc_template.large_image.clone();
                            let orig_large_text = rpc_template.large_image_text.clone();

                            if !pointers.is_empty() {
                                game_config.pointers = Some(pointers);
                                rpc_template = crate::config::RpcTemplate {
                                    details: "{player_name} - {character_class}".to_string(),
                                    state: "Level {level} | HP {health}".to_string(),
                                    large_image: orig_large,
                                    large_image_text: orig_large_text,
                                    small_image: None,
                                    small_image_text: None,
                                };
                            } else {
                                game_config.pointers = Some(GameConfig::get_engine_pointers(&detected_engine));
                                rpc_template = GameConfig::get_engine_rpc_template(&detected_engine);
                                rpc_template.large_image = orig_large;
                                rpc_template.large_image_text = orig_large_text;
                            }
                            
                            game_config.rpc_template = rpc_template;
                        }
                    }
                } else if !self.logged_detection.contains(&game.process_name) {
                    log::info!("[XGS] Detected game: {} (config loaded)", game.process_name);
                }

                self.logged_detection.insert(game.process_name.clone());

                let info = ProcessInfo {
                    process_name: game.process_name.clone(),
                    game_config: Some(game_config),
                    start_time: Instant::now(),
                };

                self.active_process = Some(info.clone());
                return Some(info);
            }
        }

        if self.active_process.is_some() {
            log::info!("[XGS] Game closed");
            self.active_process = None;
        }

        None
    }

    fn get_process_path(process_name: &str) -> Option<std::path::PathBuf> {
        use std::process::Command;

        let safe_name: String = process_name
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '_' || *c == '-' || *c == ' ')
            .collect();

        let output = Command::new("wmic")
            .args(&["process", "where", &format!("name='{}'", safe_name), "get", "ExecutablePath"])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();

        if lines.len() > 1 {
            let path_str = lines[1].trim();
            if !path_str.is_empty() {
                return Some(std::path::PathBuf::from(path_str));
            }
        }

        None
    }

    pub fn get_active_process(&self) -> Option<&ProcessInfo> {
        self.active_process.as_ref()
    }

    pub fn is_game_active(&self) -> bool {
        self.active_process.is_some()
    }

    pub fn add_game(&mut self, game: GameConfig) {
        self.known_games.push(game);
    }

    pub fn remove_game(&mut self, process_name: &str) {
        self.known_games
            .retain(|g| g.process_name != process_name);
    }

    pub fn get_games(&self) -> &[GameConfig] {
        &self.known_games
    }
}
