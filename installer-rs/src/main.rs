use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const VERSION: &str = "0.6.0";
const GITHUB_BASE: &str = "https://github.com/user/XGameStats/releases/download/v0.6.0";

fn main() {
    print_header();
    let existing = find_installation();

    match existing {
        Some(dir) => {
            println!("  XGameStats found at: {}", dir.display());
            println!();
            println!("  What would you like to do?");
            println!();
            println!("  1)  Update (reinstall files)");
            println!("  2)  Repair (fix missing files)");
            println!("  3)  Uninstall");
            println!("  4)  Install to another location");
            println!("  5)  Exit");
            println!();

            let choice = ask_choice(5);
            match choice {
                1 => do_update(&dir),
                2 => do_repair(&dir),
                3 => do_uninstall(&dir),
                4 => do_install(),
                _ => {}
            }
        }
        None => {
            println!("  No existing installation found.");
            println!();
            println!("  1)  Install");
            println!("  2)  Exit");
            println!();

            let choice = ask_choice(2);
            match choice {
                1 => do_install(),
                _ => {}
            }
        }
    }
}

fn print_header() {
    println!();
    println!("  XGameStats Installer v{}", VERSION);
    println!("  ================================");
    println!("  Discord Rich Presence Engine");
}

fn find_installation() -> Option<PathBuf> {
    let candidates = vec![
        format!("C:\\Program Files\\XGameStats"),
        format!("C:\\Program Files (x86)\\XGameStats"),
    ];

    for path in &candidates {
        let p = PathBuf::from(path);
        if p.exists() && p.join("xgs.exe").exists() {
            return Some(p);
        }
    }

    if let Ok(pf) = std::env::var("ProgramFiles") {
        let p = PathBuf::from(pf).join("XGameStats");
        if p.exists() && p.join("xgs.exe").exists() {
            return Some(p);
        }
    }

    let app_data = std::env::var("APPDATA").unwrap_or_default();
    let sm_path = format!("{}\\Microsoft\\Windows\\Start Menu\\Programs\\XGameStats", app_data);
    if Path::new(&sm_path).exists() {
        if let Ok(entries) = fs::read_dir(&sm_path) {
            for entry in entries.flatten() {
                let target = fs::read_link(entry.path()).unwrap_or_default();
                if let Some(parent) = target.parent() {
                    if parent.join("xgs.exe").exists() {
                        return Some(parent.to_path_buf());
                    }
                }
            }
        }
    }

    None
}

fn ask_choice(max: u32) -> u32 {
    loop {
        print!("\n  > ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        if let Ok(n) = input.trim().parse::<u32>() {
            if n >= 1 && n <= max {
                return n;
            }
        }
        println!("  Enter a number 1-{}", max);
    }
}

fn ask_path(default: &str) -> PathBuf {
    println!();
    println!("  Install path (default: {}):", default);
    print!("  > ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();

    if input.is_empty() {
        PathBuf::from(default)
    } else {
        PathBuf::from(input)
    }
}

fn do_install() {
    let default = format!("C:\\Program Files\\XGameStats");
    let install_dir = ask_path(&default);

    kill_running();
    download_and_install(&install_dir);
    create_shortcuts(&install_dir);
    print_complete("Installed", &install_dir);
}

fn do_update(dir: &Path) {
    println!();
    println!("  Updating at: {}", dir.display());
    kill_running();
    download_and_install(dir);
    print_complete("Updated", dir);
}

fn do_repair(dir: &Path) {
    println!();
    println!("  Repairing at: {}", dir.display());
    kill_running();

    let required = vec![
        "xgs.exe", "xgamestats-gui.jar", "libscanner.dll",
        "libhooks.dll", "translations.json", "version.json",
    ];

    let mut missing = Vec::new();
    for file in &required {
        if !dir.join(file).exists() {
            missing.push(file.to_string());
        }
    }

    if missing.is_empty() {
        println!("  All files present. No repair needed.");
    } else {
        println!("  Missing files: {}", missing.join(", "));
        println!("  Downloading...");
        download_and_install(dir);
    }

    print_complete("Repaired", dir);
}

fn do_uninstall(dir: &Path) {
    println!();
    println!("  Uninstalling from: {}", dir.display());
    println!();
    print!("  Are you sure? (y/n): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    if input.trim().to_lowercase() != "y" {
        println!("  Cancelled.");
        return;
    }

    kill_running();
    remove_shortcuts();

    let config_dir = dirs_remove();
    if let Some(cd) = config_dir {
        println!("  Removing config: {}", cd.display());
        let _ = fs::remove_dir_all(&cd);
    }

    println!("  Removing: {}", dir.display());
    let _ = fs::remove_dir_all(dir);

    println!();
    println!("  ================================");
    println!("  Uninstalled!");
    println!("  ================================");
    println!();
    wait_for_key();
}

fn dirs_remove() -> Option<PathBuf> {
    let local = std::env::var("LOCALAPPDATA").ok()?;
    Some(PathBuf::from(local).join("XGameStats"))
}

fn kill_running() {
    println!();
    println!("  Stopping running instances...");
    let _ = std::process::Command::new("taskkill")
        .args(["/F", "/IM", "xgs.exe"])
        .output();
    std::thread::sleep(std::time::Duration::from_secs(1));
}

fn download_and_install(install_dir: &Path) {
    println!();
    println!("  Installing to: {}", install_dir.display());
    println!();

    fs::create_dir_all(install_dir).expect("Failed to create install directory");

    let url = format!("{}/XGameStats.zip", GITHUB_BASE);
    println!("  Downloading XGameStats.zip...");

    match download_file(&url) {
        Ok(data) => {
            println!("  Extracting...");
            extract_zip(&data, install_dir);
            println!("  [OK] Done");
        }
        Err(e) => {
            println!("  [ERROR] Download failed: {}", e);
            println!("  Make sure you have internet connection.");
            println!("  Or manually copy files to: {}", install_dir.display());
            wait_for_key();
            std::process::exit(1);
        }
    }
}

fn download_file(url: &str) -> Result<Vec<u8>, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.get(url).send().map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let bytes = resp.bytes().map_err(|e| e.to_string())?;
    Ok(bytes.to_vec())
}

fn extract_zip(data: &[u8], target: &Path) {
    let cursor = io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor).expect("Invalid ZIP file");

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).expect("Failed to read ZIP entry");
        let path = entry.mangled_name();

        let out_path = target.join(&path);

        if entry.is_dir() {
            fs::create_dir_all(&out_path).ok();
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).ok();
            }
            let mut out_file = fs::File::create(&out_path).expect("Failed to create file");
            io::copy(&mut entry, &mut out_file).expect("Failed to write file");
        }
    }
}

fn create_shortcuts(install_dir: &Path) {
    println!();
    println!("  Creating shortcuts...");

    let desktop = std::env::var("USERPROFILE").unwrap_or_default();
    let link_path = format!("{}\\Desktop\\XGameStats.lnk", desktop);
    let target = install_dir.join("xgs.exe");

    let ps = format!(
        "$s=(New-Object -COM WScript.Shell);\
         $lnk=$s.CreateShortcut('{}');\
         $lnk.TargetPath='{}';\
         $lnk.WorkingDirectory='{}';\
         $lnk.Description='XGameStats';\
         $lnk.Save();",
        link_path, target.display(), install_dir.display()
    );
    let _ = std::process::Command::new("powershell")
        .args(["-Command", &ps])
        .output();

    let app_data = std::env::var("APPDATA").unwrap_or_default();
    let sm_dir = format!("{}\\Microsoft\\Windows\\Start Menu\\Programs\\XGameStats", app_data);
    fs::create_dir_all(&sm_dir).ok();

    let sm_link = format!("{}\\XGameStats.lnk", sm_dir);
    let ps2 = format!(
        "$s=(New-Object -COM WScript.Shell);\
         $lnk=$s.CreateShortcut('{}');\
         $lnk.TargetPath='{}';\
         $lnk.WorkingDirectory='{}';\
         $lnk.Description='XGameStats';\
         $lnk.Save();",
        sm_link, target.display(), install_dir.display()
    );
    let _ = std::process::Command::new("powershell")
        .args(["-Command", &ps2])
        .output();

    println!("  [OK] Shortcuts created");
}

fn remove_shortcuts() {
    println!("  Removing shortcuts...");

    let desktop = std::env::var("USERPROFILE").unwrap_or_default();
    let _ = fs::remove_file(format!("{}\\Desktop\\XGameStats.lnk", desktop));

    let app_data = std::env::var("APPDATA").unwrap_or_default();
    let sm_dir = format!("{}\\Microsoft\\Windows\\Start Menu\\Programs\\XGameStats", app_data);
    let _ = fs::remove_dir_all(&sm_dir);
}

fn print_complete(action: &str, install_dir: &Path) {
    println!();
    println!("  ================================");
    println!("  {} successfully!", action);
    println!("  ================================");
    println!();
    println!("  Location: {}", install_dir.display());
    println!();
    println!("  To start: run xgs.exe or use desktop shortcut");
    println!();
    wait_for_key();
}

fn wait_for_key() {
    println!("  Press Enter to exit...");
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).ok();
}
