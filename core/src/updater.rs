use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, serde::Deserialize)]
pub struct RemoteVersion {
    pub version: String,
    pub min_version: Option<String>,
    pub download_url: String,
    pub changelog: Option<String>,
}

pub fn current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub fn check_for_update(remote_url: &str) -> Result<RemoteVersion, String> {
    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile", "-Command",
            &format!("(Invoke-WebRequest -Uri '{}' -UseBasicParsing).Content", remote_url),
        ])
        .output()
        .map_err(|e| format!("Failed to run PowerShell: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "PowerShell request failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let body = String::from_utf8(output.stdout).map_err(|e| format!("Invalid UTF-8: {}", e))?;
    let remote: RemoteVersion =
        serde_json::from_str(&body).map_err(|e| format!("Failed to parse version.json: {}", e))?;

    Ok(remote)
}

pub fn download_file(url: &str, dest: &Path) -> Result<(), String> {
    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile", "-Command",
            &format!(
                "Invoke-WebRequest -Uri '{}' -OutFile '{}' -UseBasicParsing",
                url,
                dest.to_string_lossy().replace('\\', "\\\\"),
            ),
        ])
        .output()
        .map_err(|e| format!("Failed to download: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Download failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

pub fn extract_zip(zip_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let file = fs::File::open(zip_path).map_err(|e| format!("Cannot open ZIP: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Invalid ZIP: {}", e))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Cannot read ZIP entry: {}", e))?;

        let out_path = dest_dir.join(entry.name());

        if entry.is_dir() {
            fs::create_dir_all(&out_path)
                .map_err(|e| format!("Cannot create dir: {}", e))?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Cannot create dir: {}", e))?;
            }
            let mut out_file = fs::File::create(&out_path)
                .map_err(|e| format!("Cannot create file: {}", e))?;
            std::io::copy(&mut entry, &mut out_file)
                .map_err(|e| format!("Cannot write file: {}", e))?;
        }
    }

    Ok(())
}

pub fn apply_update(zip_path: &Path, app_dir: &Path) -> Result<(), String> {
    let temp_dir = app_dir.join("update_temp");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).map_err(|e| format!("Cannot clean temp: {}", e))?;
    }
    fs::create_dir_all(&temp_dir).map_err(|e| format!("Cannot create temp: {}", e))?;

    extract_zip(zip_path, &temp_dir)?;

    let entries: Vec<PathBuf> = fs::read_dir(&temp_dir)
        .map_err(|e| format!("Cannot read temp: {}", e))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();

    for entry in &entries {
        let file_name = entry.file_name().unwrap();
        let target = app_dir.join(file_name);

        if entry.is_dir() {
            if target.exists() {
                fs::remove_dir_all(&target)
                    .map_err(|e| format!("Cannot remove old dir: {}", e))?;
            }
            fs::rename(entry, &target)
                .map_err(|e| format!("Cannot move dir: {}", e))?;
        } else {
            if target.exists() {
                let _ = fs::remove_file(&target);
            }
            fs::rename(entry, &target)
                .map_err(|e| format!("Cannot move file: {}", e))?;
        }
    }

    let _ = fs::remove_dir_all(&temp_dir);
    let _ = fs::remove_file(zip_path);

    Ok(())
}

pub fn run_updater(zip_path: &Path, app_dir: &Path) {
    println!("[XGS Updater] Waiting for main process to exit...");
    std::thread::sleep(std::time::Duration::from_secs(2));

    println!("[XGS Updater] Applying update...");
    if let Err(e) = apply_update(zip_path, app_dir) {
        eprintln!("[XGS Updater] Update failed: {}", e);
        return;
    }

    println!("[XGS Updater] Update applied successfully");

    let exe_path = app_dir.join("xgs.exe");
    if exe_path.exists() {
        println!("[XGS Updater] Restarting XGameStats...");
        let _ = std::process::Command::new(&exe_path).spawn();
    }
}

pub fn check_and_update(app_dir: &Path, remote_url: &str) -> bool {
    let current = current_version();
    println!("[XGS] Current version: {}", current);
    println!("[XGS] Checking for updates...");

    let remote = match check_for_update(remote_url) {
        Ok(r) => r,
        Err(e) => {
            println!("[XGS] Update check failed: {}", e);
            return false;
        }
    };

    println!("[XGS] Latest version: {}", remote.version);

    if remote.version == current {
        println!("[XGS] Already up to date");
        return false;
    }

    if let Some(ref min) = remote.min_version {
        if &current < min {
            println!(
                "[XGS] Version {} is too old, minimum required: {}. Please update manually.",
                current, min
            );
            return false;
        }
    }

    println!("[XGS] New version available: {}", remote.version);
    if let Some(ref cl) = remote.changelog {
        println!("[XGS] Changelog: {}", cl);
    }

    let temp_zip = app_dir.join(format!("update-{}.zip", remote.version));
    println!("[XGS] Downloading update...");

    if let Err(e) = download_file(&remote.download_url, &temp_zip) {
        eprintln!("[XGS] Download failed: {}", e);
        return false;
    }

    println!("[XGS] Update downloaded: {}", temp_zip.display());

    let updater_exe = app_dir.join("xgs.exe");
    let _ = std::process::Command::new(&updater_exe)
        .args(["update", &temp_zip.to_string_lossy()])
        .spawn();

    true
}

pub fn default_install_dir() -> PathBuf {
    let program_files = std::env::var("ProgramFiles")
        .or_else(|_| std::env::var("ProgramFiles(x86)"))
        .unwrap_or_else(|_| "C:\\Program Files".to_string());
    PathBuf::from(program_files).join("XGameStats")
}

fn start_menu_dir() -> PathBuf {
    let app_data = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(app_data)
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("XGameStats")
}

fn create_shortcuts(install_dir: &Path) -> Result<(), String> {
    let sm = start_menu_dir();
    fs::create_dir_all(&sm).map_err(|e| format!("Cannot create Start Menu dir: {}", e))?;

    let exe = install_dir.join("xgs.exe");

    let bat_content = format!(
        "@echo off\r\nstart \"\" \"{}\" start\r\n",
        exe.to_string_lossy().replace('\\', "\\\\")
    );
    let bat_path = sm.join("XGameStats.bat");
    fs::write(&bat_path, bat_content).map_err(|e| format!("Cannot create shortcut: {}", e))?;

    let vbs_content = format!(
        "Set oWS = WScript.CreateObject(\"WScript.Shell\")\r\n\
         Set oLink = oWS.CreateShortcut(\"{}\\XGameStats.lnk\")\r\n\
         oLink.TargetPath = \"{}\"\r\n\
         oLink.Arguments = \"start\"\r\n\
         oLink.WorkingDirectory = \"{}\"\r\n\
         oLink.Save\r\n",
        sm.to_string_lossy().replace('\\', "\\\\"),
        exe.to_string_lossy().replace('\\', "\\\\"),
        install_dir.to_string_lossy().replace('\\', "\\\\")
    );
    let vbs_path = sm.join("create_shortcut.vbs");
    fs::write(&vbs_path, vbs_content).map_err(|e| format!("Cannot create VBS: {}", e))?;

    let _ = std::process::Command::new("cscript")
        .arg("//NoLogo")
        .arg(&vbs_path)
        .output();
    let _ = fs::remove_file(&vbs_path);

    Ok(())
}

fn remove_shortcuts() -> Result<(), String> {
    let sm = start_menu_dir();
    if sm.exists() {
        fs::remove_dir_all(&sm).map_err(|e| format!("Cannot remove shortcuts: {}", e))?;
    }
    Ok(())
}

pub fn install(source_dir: &Path, install_dir: &Path) -> Result<(), String> {
    println!("[XGS] Installing to: {}", install_dir.display());

    fs::create_dir_all(install_dir).map_err(|e| format!("Cannot create install dir: {}", e))?;

    let files_to_copy = [
        "xgs.exe",
        "xgamestats-gui.jar",
        "libscanner.dll",
        "libhooks.dll",
        "translations.json",
        "version.json",
        "README.md",
        "SECURITY.md",
    ];

    for file in &files_to_copy {
        let src = source_dir.join(file);
        let dst = install_dir.join(file);
        if src.exists() {
            fs::copy(&src, &dst).map_err(|e| format!("Cannot copy {}: {}", file, e))?;
            println!("[XGS]   {}", file);
        }
    }

    let dirs_to_copy = ["javafx-sdk", "configs", "assets"];
    for dir in &dirs_to_copy {
        let src = source_dir.join(dir);
        let dst = install_dir.join(dir);
        if src.exists() {
            if dst.exists() {
                fs::remove_dir_all(&dst).map_err(|e| format!("Cannot remove old {}: {}", dir, e))?;
            }
            copy_dir_recursive(&src, &dst)
                .map_err(|e| format!("Cannot copy {}/: {}", dir, e))?;
            println!("[XGS]   {}/", dir);
        }
    }

    create_shortcuts(install_dir)?;

    let config_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("XGameStats");
    fs::create_dir_all(&config_dir).ok();

    let uninstall_bat = format!(
        "@echo off\r\n\
         taskkill /F /IM xgs.exe 2>nul\r\n\
         timeout /t 2 /nobreak >nul\r\n\
         rmdir /s /q \"{}\"\r\n\
         rmdir /s /q \"{}\"\r\n\
         echo XGameStats has been uninstalled.\r\n\
         pause\r\n",
        install_dir.to_string_lossy(),
        config_dir.to_string_lossy()
    );
    let uninstall_path = install_dir.join("uninstall.bat");
    fs::write(&uninstall_path, uninstall_bat)
        .map_err(|e| format!("Cannot create uninstaller: {}", e))?;

    println!("[XGS] Installation complete!");
    println!("[XGS] Run: {}\\xgs.exe start", install_dir.display());
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub fn uninstall(install_dir: &Path) -> Result<(), String> {
    println!("[XGS] Uninstalling from: {}", install_dir.display());

    let _ = std::process::Command::new("taskkill")
        .args(["/F", "/IM", "xgs.exe"])
        .output();

    std::thread::sleep(std::time::Duration::from_secs(2));

    remove_shortcuts()?;

    let config_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("XGameStats");
    if config_dir.exists() {
        fs::remove_dir_all(&config_dir)
            .map_err(|e| format!("Cannot remove config dir: {}", e))?;
        println!("[XGS]   Config removed");
    }

    if install_dir.exists() {
        fs::remove_dir_all(install_dir)
            .map_err(|e| format!("Cannot remove install dir: {}", e))?;
        println!("[XGS]   Installation removed");
    }

    println!("[XGS] Uninstall complete!");
    Ok(())
}

pub fn repair(install_dir: &Path, source_dir: &Path) -> Result<(), String> {
    println!("[XGS] Repairing installation...");

    let required_files = [
        "xgs.exe",
        "xgamestats-gui.jar",
        "libscanner.dll",
        "libhooks.dll",
        "translations.json",
    ];

    let mut missing = Vec::new();
    for file in &required_files {
        let path = install_dir.join(file);
        if !path.exists() {
            missing.push(file.to_string());
        }
    }

    if missing.is_empty() {
        println!("[XGS] All files present, no repair needed");
        return Ok(());
    }

    println!("[XGS] Missing files: {:?}", missing);

    for file in &missing {
        let src = source_dir.join(file);
        let dst = install_dir.join(file);
        if src.exists() {
            fs::copy(&src, &dst).map_err(|e| format!("Cannot restore {}: {}", file, e))?;
            println!("[XGS]   Restored: {}", file);
        } else {
            println!("[XGS]   WARNING: {} not found in source", file);
        }
    }

    create_shortcuts(install_dir)?;

    println!("[XGS] Repair complete!");
    Ok(())
}
