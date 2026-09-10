#![windows_subsystem = "windows"]
#![allow(dead_code)]

mod config;
mod discord;
mod process_tracker;
mod scanner_bridge;
mod updater;

use std::collections::HashMap;
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use config::{AppConfig, GameConfig};
use discord::{DiscordIpc, DiscordPresence};
use process_tracker::ProcessTracker;
use scanner_bridge::Scanner;

struct XGameStats {
    config: AppConfig,
    tracker: ProcessTracker,
    discord_connections: HashMap<String, DiscordIpc>,
    start_time: i64,
    last_connect_attempt: HashMap<String, std::time::Instant>,
    logged_running: HashMap<String, bool>,
    cached_large_image: HashMap<String, Option<String>>,
}

impl XGameStats {
    fn new(config: AppConfig) -> Self {
        let tracker = ProcessTracker::new(config.scan_interval_ms, config.games.clone());

        XGameStats {
            config,
            tracker,
            discord_connections: HashMap::new(),
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            last_connect_attempt: HashMap::new(),
            logged_running: HashMap::new(),
            cached_large_image: HashMap::new(),
        }
    }

    fn run(&mut self) {
        log::info!("[XGS] Starting engine...");
        log::info!("[XGS] Tracking {} game(s)", self.config.games.len());
        log::info!("[XGS] Scan interval: {}ms", self.config.scan_interval_ms);
        log::info!("[XGS] Loading configs...");
        log::info!("[XGS] Scanning for processes...");
        log::info!("[XGS] Engine ready");

        loop {
            if let Some(process) = self.tracker.scan() {
                if let Some(ref game_config) = process.game_config {
                    self.update_presence(game_config);
                }
            } else {
                self.clear_all_presences();
            }

            thread::sleep(Duration::from_millis(100));
        }
    }

    fn update_presence(&mut self, game_config: &GameConfig) {
        let app_id = if game_config.discord_app_id.is_empty() {
            self.config.discord_app_id.clone()
        } else {
            game_config.discord_app_id.clone()
        };

        if app_id.is_empty() {
            log::warn!("No Discord App ID configured for {}", game_config.process_name);
            return;
        }

        if !self.discord_connections.contains_key(&app_id) {
            if let Some(last_attempt) = self.last_connect_attempt.get(&app_id) {
                if last_attempt.elapsed() < Duration::from_secs(5) {
                    return;
                }
            }
            log::info!("[XGS] Connecting to Discord (app_id: {})...", app_id);
            self.last_connect_attempt.insert(app_id.clone(), std::time::Instant::now());
            let mut ipc = DiscordIpc::new(&app_id);
            match ipc.connect() {
                Ok(()) => {
                    log::info!("[XGS] IPC pipe connected, sending handshake...");
                    match ipc.send_handshake() {
                        Ok(()) => {
                            self.discord_connections.insert(app_id.clone(), ipc);
                            log::info!("[XGS] Connected to Discord for {}", game_config.process_name);
                        }
                        Err(e) => {
                            log::error!("[XGS] Handshake failed for {}: {}", game_config.process_name, e);
                            return;
                        }
                    }
                }
                Err(e) => {
                    log::error!("[XGS] Failed to connect to Discord for {}: {}", game_config.process_name, e);
                    return;
                }
            }
        }

        let details = self.render_template(&game_config.rpc_template.details, game_config);
        let state = self.render_template(&game_config.rpc_template.state, game_config);
        let large_image_text = game_config
            .rpc_template
            .large_image_text
            .as_ref()
            .map(|t| self.render_template(t, game_config));
        let small_image_text = game_config
            .rpc_template
            .small_image_text
            .as_ref()
            .map(|t| self.render_template(t, game_config));

        let large_image = if let Some(ref img) = game_config.rpc_template.large_image {
            if is_discord_large_image_key(img) {
                log::info!("[XGS] Using image from config: {}", img);
                Some(img.clone())
            } else {
                log::warn!(
                    "[XGS] Ignoring local image path as large_image (Discord can't display local files): {}",
                    img
                );
                self.cached_large_image.get(&game_config.process_name).cloned().flatten()
            }
        } else {
            self.cached_large_image.get(&game_config.process_name).cloned().flatten()
        };

        if large_image.is_none() {
            if let Some(path) = self.get_process_path(&game_config.process_name) {
                let steam_app_id = game_config.steam_app_id.or_else(|| GameConfig::find_steam_app_id(&path));

                let icon = if let Some(app_id) = steam_app_id {
                    let url = GameConfig::get_steam_capsule_url(app_id);
                    log::info!("[XGS] Using Steam capsule art (App ID {}): {}", app_id, url);
                    Some(url)
                } else if let Some(icon_path) = GameConfig::find_game_icon(&path) {
                    // Локальный файл Discord показать не может — только лог.
                    log::warn!(
                        "[XGS] Found local game icon '{}' but Discord cannot display local files; skipping",
                        icon_path.display()
                    );
                    None
                } else {
                    None
                };
                self.cached_large_image.insert(game_config.process_name.clone(), icon);
            }
        }

        // После сканирования в кэше мог появиться Steam-арт — перечитываем,
        // чтобы использовать его уже в текущем обновлении.
        let large_image = if large_image.is_none() {
            self.cached_large_image.get(&game_config.process_name).cloned().flatten()
        } else {
            large_image
        };

        // small_image: тоже только asset-ключ или удалённый URL.
        let small_image = match &game_config.rpc_template.small_image {
            Some(img) if is_discord_large_image_key(img) => Some(img.clone()),
            Some(img) => {
                log::warn!(
                    "[XGS] Ignoring local image path as small_image (Discord can't display local files): {}",
                    img
                );
                None
            }
            None => None,
        };

        let presence = DiscordPresence {
            name: game_config.process_name.replace(".exe", ""),
            details,
            state,
            large_image,
            large_image_text,
            small_image,
            small_image_text,
            start_timestamp: Some(self.start_time),
        };

        if let Some(ipc) = self.discord_connections.get_mut(&app_id) {
            if let Err(e) = ipc.set_activity(&presence) {
                log::error!(
                    "Failed to set activity for {}: {}",
                    game_config.process_name,
                    e
                );
                self.discord_connections.remove(&app_id);
                self.logged_running.remove(&app_id);
                log::info!("[XGS] Removed broken connection for {}, will retry next cycle", app_id);
            } else if !self.logged_running.get(&app_id).unwrap_or(&false) {
                log::info!("[XGS] Running");
                self.logged_running.insert(app_id.clone(), true);
            }
        }
    }

    fn clear_all_presences(&mut self) {
        for (app_id, ipc) in &mut self.discord_connections {
            if let Err(e) = ipc.clear_activity() {
                log::warn!("Failed to clear activity for {}: {}", app_id, e);
            }
        }
    }

    fn get_process_path(&self, process_name: &str) -> Option<std::path::PathBuf> {
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

    fn render_template(&self, template: &str, game_config: &GameConfig) -> String {
        let mut result = template.to_string();

        if let Some(ref pointers) = game_config.pointers {
            let scanner = Scanner::open(&game_config.process_name);
            for (key, pointer) in pointers {
                let placeholder = format!("{{{}}}", key);
                let value = self.read_game_value(&scanner, game_config, pointer);
                result = result.replace(&placeholder, &value);
            }
        }

        result
    }

    fn read_game_value(&self, scanner: &Option<Scanner>, game_config: &GameConfig, pointer: &config::PointerConfig) -> String {
        if let Some(scanner) = scanner {
            let value_any = match game_config.scan_type {
                config::ScanType::Offsets => {
                    let base_offset = parse_base_offset(&pointer.base);
                    let offsets = pointer.offsets.as_deref().unwrap_or(&[]);
                    scanner.read_by_config(&game_config.process_name, base_offset, offsets, 0)
                }
                config::ScanType::Aob => {
                    if let Some(aob_map) = &game_config.aob_patterns {
                        if let Some(aob_cfg) = aob_map.get(&pointer.base) {
                            let start = aob_cfg.scan_start.unwrap_or(4194304) as usize;
                            let size = aob_cfg.scan_size.unwrap_or(536870912);

                            let mut pattern_bytes = Vec::new();
                            for byte_str in aob_cfg.pattern.split_whitespace() {
                                if byte_str == "?" || byte_str == "??" {
                                    pattern_bytes.push(0);
                                } else {
                                    pattern_bytes.push(u8::from_str_radix(byte_str, 16).unwrap_or(0));
                                }
                            }

                            if let Some(addr) = scanner.aob_scan(&pattern_bytes, &aob_cfg.mask, start, size) {
                                let offsets = pointer.offsets.as_deref().unwrap_or(&[]);
                                if offsets.is_empty() {
                                    scanner.read_int(addr).map(|v| Box::new(v) as Box<dyn std::any::Any>)
                                } else {
                                    if let Some(resolved) = scanner.resolve_pointer_chain(addr, &offsets[..offsets.len()-1]) {
                                        let final_addr = resolved + *offsets.last().unwrap_or(&0) as usize;
                                        scanner.read_int(final_addr).map(|v| Box::new(v) as Box<dyn std::any::Any>)
                                    } else {
                                        None
                                    }
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            };

            if let Some(value) = value_any {
                if let Some(int_val) = value.downcast_ref::<i32>() {
                    return int_val.to_string();
                }
                if let Some(uint_val) = value.downcast_ref::<u32>() {
                    return uint_val.to_string();
                }
                if let Some(float_val) = value.downcast_ref::<f32>() {
                    return format!("{:.1}", float_val);
                }
                if let Some(str_val) = value.downcast_ref::<String>() {
                    return str_val.clone();
                }
            } else if game_config.requires_elevation {
                log::warn!("Could not read memory for {}. Does the engine need Administrator privileges?", game_config.process_name);
            }
        }

        "N/A".to_string()
    }
}

fn parse_base_offset(base: &str) -> usize {
    if let Some(offset_str) = base.split('+').last() {
        if offset_str.starts_with("0x") {
            usize::from_str_radix(&offset_str[2..], 16).unwrap_or(0)
        } else {
            offset_str.parse::<usize>().unwrap_or(0)
        }
    } else {
        0
    }
}

/// Discord умеет показывать только удалённые URL (http/https, mp:external).
fn is_remote_image_url(img: &str) -> bool {
    img.starts_with("http://")
        || img.starts_with("https://")
        || img.starts_with("mp:")
}

/// Валидный ключ для large/small image: либо удалённый URL, либо asset-ключ
/// из Developer Portal (без слешей, двоеточий и т.п. — не локальный путь).
fn is_discord_large_image_key(img: &str) -> bool {
    is_remote_image_url(img) || (!img.contains('\\') && !img.contains('/') && !img.contains(':'))
}

const VERSION_URL: &str = "https://raw.githubusercontent.com/user/XGameStats/main/version.json";

fn find_exe_dir() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

fn find_java() -> Option<String> {
    let exe_dir = find_exe_dir();
    let scoop_java = exe_dir.join("..\\..\\scoop\\apps\\openjdk21\\current\\bin\\java.exe");
    if scoop_java.exists() {
        return Some(scoop_java.canonicalize().ok()?.to_string_lossy().to_string());
    }

    let candidates = [
        "java",
        r"C:\Program Files\Eclipse Adoptium\jdk-21.0.2.13-hotspot\bin\java.exe",
        r"C:\Program Files\Microsoft\jdk-21.0.2.13-hotspot\bin\java.exe",
    ];

    for candidate in &candidates {
        if Command::new(candidate).arg("--version").output().is_ok() {
            return Some(candidate.to_string());
        }
    }

    let paths = std::env::var("PATH").unwrap_or_default();
    for dir in paths.split(';') {
        let java_path = std::path::Path::new(dir).join("java.exe");
        if java_path.exists() {
            return Some(java_path.to_string_lossy().to_string());
        }
    }

    None
}

fn find_jar() -> Option<String> {
    let exe_dir = find_exe_dir();

    let candidates = [
        exe_dir.join("gui\\build\\xgamestats-gui.jar"),
        exe_dir.join("xgamestats-gui.jar"),
        exe_dir.join("..\\gui\\build\\xgamestats-gui.jar"),
        exe_dir.join("..\\..\\gui\\build\\xgamestats-gui.jar"),
        std::path::PathBuf::from("gui\\build\\xgamestats-gui.jar"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return Some(candidate.to_string_lossy().to_string());
        }
    }

    None
}

fn find_javafx_lib() -> Option<String> {
    let exe_dir = find_exe_dir();

    let candidates = [
        exe_dir.join("gui\\javafx-sdk\\lib"),
        exe_dir.join("..\\gui\\javafx-sdk\\lib"),
        exe_dir.join("..\\..\\gui\\javafx-sdk\\lib"),
        std::path::PathBuf::from("gui\\javafx-sdk\\lib"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return Some(candidate.to_string_lossy().to_string());
        }
    }

    None
}

fn launch_gui() {
    let java = find_java();
    let jar = find_jar();
    let javafx = find_javafx_lib();

    if java.is_none() {
        eprintln!("[XGS] Java not found. Install JDK 21+");
        return;
    }
    if jar.is_none() {
        eprintln!("[XGS] GUI not found. Run build_all.bat first");
        return;
    }

    let java = java.unwrap();
    let jar = jar.unwrap();

    let mut cmd = Command::new(&java);

    if let Some(fx) = &javafx {
        cmd.arg("--module-path").arg(fx);
        cmd.arg("--add-modules").arg("javafx.controls,javafx.fxml");
    }

    cmd.arg("-jar").arg(&jar);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    match cmd.spawn() {
        Ok(_) => println!("[XGS] GUI launched"),
        Err(e) => eprintln!("[XGS] Failed to launch GUI: {}", e),
    }
}

fn print_usage() {
    println!("XGameStats v0.7.0 - Discord Rich Presence Engine");
    println!();
    println!("Usage: xgs.exe <command>");
    println!();
    println!("Commands:");
    println!("  start     Launch the control panel");
    println!("  engine    Run the Discord RPC engine");
    println!("  version   Show current version");
    println!("  help      Show this help message");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let command = if args.len() > 1 {
        args[1].as_str()
    } else {
        "start"
    };

    match command {
        "engine" | "e" => {
            if std::env::var("RUST_LOG").is_err() {
                std::env::set_var("RUST_LOG", "info");
            }
            env_logger::init();

            let config = AppConfig::load(None).unwrap_or_else(|e| {
                log::error!("Failed to load config: {}", e);
                AppConfig {
                    scan_interval_ms: 3000,
                    log_level: "info".to_string(),
                    discord_app_id: String::new(),
                    language: "en".to_string(),
                    games: vec![],
                }
            });
            let mut engine = XGameStats::new(config);
            engine.run();
        }
        "gui" | "g" => {
            let app_dir = find_exe_dir();
            updater::check_and_update(&app_dir, VERSION_URL);
            launch_gui();
        }
        "start" | "s" | "" => {
            let app_dir = find_exe_dir();
            updater::check_and_update(&app_dir, VERSION_URL);
            launch_gui();
        }
        "version" | "v" => {
            println!("XGameStats v{}", updater::current_version());
        }
        "help" | "h" | "--help" | "-h" => {
            print_usage();
        }
        _ => {
            eprintln!("[XGS] Unknown command: {}", command);
            print_usage();
            std::process::exit(1);
        }
    }
}
