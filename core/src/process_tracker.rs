use std::time::{Duration, Instant};

use crate::config::GameConfig;
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
}

impl ProcessTracker {
    pub fn new(scan_interval_ms: u64, games: Vec<GameConfig>) -> Self {
        ProcessTracker {
            scan_interval: Duration::from_millis(scan_interval_ms),
            known_games: games,
            active_process: None,
            last_scan: Instant::now(),
        }
    }

    pub fn scan(&mut self) -> Option<ProcessInfo> {
        if self.last_scan.elapsed() < self.scan_interval {
            return self.active_process.clone();
        }

        self.last_scan = Instant::now();

        for game in &self.known_games {
            if Scanner::open(&game.process_name).is_some() {
                let info = ProcessInfo {
                    process_name: game.process_name.clone(),
                    game_config: Some(game.clone()),
                    start_time: Instant::now(),
                };

                self.active_process = Some(info.clone());
                log::info!("Detected game: {}", game.process_name);
                return Some(info);
            }
        }

        if self.active_process.is_some() {
            log::info!("Game closed");
            self.active_process = None;
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
