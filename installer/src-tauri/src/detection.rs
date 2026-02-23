use crate::hook::{self, HookStatus};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

use crate::patcher;

#[derive(Serialize, Clone)]
pub struct DetectionResult {
    pub root_found: bool,
    pub root_path: String,
    pub profile_dir: String,
    pub html_files: Vec<String>,
    pub is_installed: bool,
    pub hook_status: HookStatus,
}

#[cfg(target_os = "windows")]
pub fn get_profile_dir() -> PathBuf {
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
    PathBuf::from(local_app_data)
        .join("Root Communications")
        .join("Root")
        .join("profile")
        .join("default")
}

#[cfg(target_os = "linux")]
pub fn get_profile_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home)
        .join(".local/share/Root Communications/Root/profile/default")
}

#[cfg(target_os = "macos")]
pub fn get_profile_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home)
        .join("Library/Application Support/Root Communications/Root/profile/default")
}

#[cfg(target_os = "windows")]
pub fn get_root_exe_path() -> PathBuf {
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
    PathBuf::from(local_app_data)
        .join("Root")
        .join("current")
        .join("Root.exe")
}

#[cfg(target_os = "linux")]
pub fn get_root_exe_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();

    // 1. Exact well-known paths (fastest)
    let candidates = [
        format!("{}/Applications/Root.AppImage", home),
        format!("{}/Downloads/Root.AppImage", home),
        format!("{}/.local/bin/Root.AppImage", home),
        "/opt/Root.AppImage".to_string(),
        "/usr/bin/Root.AppImage".to_string(),
        format!("{}/.local/bin/Root", home),
    ];
    for c in &candidates {
        let p = PathBuf::from(c);
        if p.exists() {
            return p;
        }
    }

    // 2. Glob for variant filenames (versioned, renamed) in common dirs
    let search_dirs = [
        format!("{}/Applications", home),
        format!("{}/Downloads", home),
        format!("{}/.local/bin", home),
        format!("{}/Desktop", home),
        home.clone(),
        "/opt".to_string(),
        "/usr/bin".to_string(),
        "/usr/local/bin".to_string(),
    ];
    for dir in &search_dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.starts_with("root") && name.ends_with(".appimage") {
                    let p = entry.path();
                    if p.is_file() {
                        return p;
                    }
                }
            }
        }
    }

    // 3. Search .desktop files for Root's Exec= path
    let desktop_dirs = [
        format!("{}/.local/share/applications", home),
        "/usr/share/applications".to_string(),
        "/usr/local/share/applications".to_string(),
    ];
    for dir in &desktop_dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                    continue;
                }
                let fname = entry.file_name().to_string_lossy().to_lowercase();
                if let Ok(content) = fs::read_to_string(&path) {
                    let has_root_name = content.lines().any(|l| {
                        l.starts_with("Name=")
                            && l.to_lowercase().contains("root")
                    });
                    if !has_root_name && !fname.contains("root") {
                        continue;
                    }
                    for line in content.lines() {
                        if let Some(exec) = line.strip_prefix("Exec=") {
                            // Strip field codes like %f, %u, etc.
                            let exec_path = exec
                                .split_whitespace()
                                .next()
                                .unwrap_or("")
                                .to_string();
                            let p = PathBuf::from(&exec_path);
                            if p.is_file() {
                                return p;
                            }
                        }
                    }
                }
            }
        }
    }

    // 4. Check running Root processes via /proc
    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let fname = entry.file_name();
            let fname_str = fname.to_string_lossy();
            if !fname_str.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            let exe_link = entry.path().join("exe");
            if let Ok(exe) = fs::read_link(&exe_link) {
                let exe_str = exe.to_string_lossy().to_lowercase();
                if (exe_str.contains("root") && exe_str.contains("appimage"))
                    || exe_str.ends_with("/root")
                {
                    if exe.is_file() {
                        return exe;
                    }
                }
            }
        }
    }

    // 5. PATH lookup
    if let Ok(output) = std::process::Command::new("which")
        .arg("Root")
        .output()
    {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let p = PathBuf::from(&path_str);
            if p.is_file() {
                return p;
            }
        }
    }

    // Default fallback
    PathBuf::from(format!("{}/Applications/Root.AppImage", home))
}

#[cfg(target_os = "macos")]
pub fn get_root_exe_path() -> PathBuf {
    // Root does not currently ship on macOS.
    // Provide a plausible path for forward compatibility.
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(format!("{}/Applications/Root.app/Contents/MacOS/Root", home))
}

pub fn find_target_html_files() -> Vec<PathBuf> {
    let profile = get_profile_dir();
    let mut targets = Vec::new();

    // WebRtcBundle/index.html
    let webrtc_index = profile.join("WebRtcBundle").join("index.html");
    if webrtc_index.exists() {
        targets.push(webrtc_index);
    }

    // RootApps/*/index.html
    let root_apps_dir = profile.join("RootApps");
    if root_apps_dir.exists() {
        if let Ok(entries) = fs::read_dir(&root_apps_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let app_index = entry.path().join("index.html");
                    if app_index.exists() {
                        targets.push(app_index);
                    }
                }
            }
        }
    }

    targets
}

pub fn check_is_installed(html_files: &[PathBuf]) -> bool {
    for file in html_files {
        if let Ok(content) = fs::read_to_string(file) {
            if patcher::is_patched(&content) {
                return true;
            }
        }
    }
    false
}

pub fn detect() -> DetectionResult {
    let root_exe = get_root_exe_path();
    let profile = get_profile_dir();
    let html_files = find_target_html_files();
    let is_installed = check_is_installed(&html_files);
    let hook_status = hook::check_hook_status();

    DetectionResult {
        root_found: root_exe.exists(),
        root_path: root_exe.to_string_lossy().to_string(),
        profile_dir: profile.to_string_lossy().to_string(),
        html_files: html_files
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
        is_installed,
        hook_status,
    }
}
