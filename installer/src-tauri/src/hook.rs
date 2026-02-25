use crate::embedded;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

const PROFILER_GUID: &str = "{D1A6F5A0-1234-4567-89AB-CDEF01234567}";

#[cfg(target_os = "windows")]
const ENV_VARS: &[&str] = &[
    // DOTNET_ prefix (primary — .NET 10+)
    "DOTNET_EnableDiagnostics",
    "DOTNET_ENABLE_PROFILING",
    "DOTNET_PROFILER",
    "DOTNET_PROFILER_PATH",
    "DOTNET_ReadyToRun",
    // CORECLR_ prefix (legacy — .NET 8/9)
    "CORECLR_ENABLE_PROFILING",
    "CORECLR_PROFILER",
    "CORECLR_PROFILER_PATH",
    // Legacy startup hooks
    "DOTNET_STARTUP_HOOKS",
];

#[derive(Serialize, Clone, Default)]
pub struct HookStatus {
    pub profiler_dll: bool,
    pub hook_dll: bool,
    pub hook_deps: bool,
    pub preload_js: bool,
    pub theme_css: bool,
    pub env_enable_profiling: bool,
    pub env_profiler_guid: bool,
    pub env_profiler_path: bool,
    pub env_ready_to_run: bool,
    /// True if all files are deployed
    pub files_ok: bool,
    /// True if all env vars are set correctly
    pub env_ok: bool,
    /// True if env vars are active in the current process environment (Linux only).
    /// On Windows this always matches env_ok since registry changes apply immediately.
    pub env_vars_active: bool,
}

// ==================== Platform-specific: install directory ====================

/// Returns `%LOCALAPPDATA%\Root\uprooted\` on Windows.
#[cfg(target_os = "windows")]
pub fn get_uprooted_dir() -> PathBuf {
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
    PathBuf::from(local_app_data).join("Root").join("uprooted")
}

/// Returns `~/.local/share/uprooted/` on Linux.
#[cfg(target_os = "linux")]
pub fn get_uprooted_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join(".local/share/uprooted")
}

/// Returns `~/Library/Application Support/uprooted/` on macOS.
#[cfg(target_os = "macos")]
pub fn get_uprooted_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join("Library/Application Support/uprooted")
}

// ==================== Platform-specific: profiler filename ====================

#[cfg(target_os = "windows")]
const PROFILER_FILENAME: &str = "uprooted_profiler.dll";
#[cfg(target_os = "linux")]
const PROFILER_FILENAME: &str = "libuprooted_profiler.so";
#[cfg(target_os = "macos")]
const PROFILER_FILENAME: &str = "libuprooted_profiler.dylib";

// ==================== Deploy files ====================

/// Deploy all embedded files to the install directory.
pub fn deploy_files() -> Result<(), String> {
    let dir = get_uprooted_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create {}: {}", dir.display(), e))?;

    let files: &[(&str, &[u8])] = &[
        (PROFILER_FILENAME, embedded::PROFILER),
        ("UprootedHook.dll", embedded::HOOK_DLL),
        ("UprootedHook.deps.json", embedded::HOOK_DEPS_JSON),
        ("UprootedHook.net9.dll", embedded::HOOK_DLL_NET9),
        ("UprootedHook.net9.deps.json", embedded::HOOK_DEPS_JSON_NET9),
        ("uprooted-preload.js", embedded::PRELOAD_JS),
        ("uprooted.css", embedded::THEME_CSS),
        ("nsfw-filter.js", embedded::NSFW_FILTER_JS),
        ("link-embeds.js", embedded::LINK_EMBEDS_JS),
    ];

    for (name, data) in files {
        let path = dir.join(name);
        fs::write(&path, data)
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
    }

    // On Unix, set the profiler shared library as executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let profiler_path = dir.join(PROFILER_FILENAME);
        let perms = std::fs::Permissions::from_mode(0o755);
        let _ = std::fs::set_permissions(&profiler_path, perms);
    }

    Ok(())
}

// ==================== Windows: environment variables via registry ====================

/// Set CLR profiler environment variables (user-scoped) and broadcast WM_SETTINGCHANGE.
#[cfg(target_os = "windows")]
pub fn set_env_vars() -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (env_key, _) = hkcu
        .create_subkey("Environment")
        .map_err(|e| format!("Failed to open HKCU\\Environment: {}", e))?;

    let profiler_path = get_uprooted_dir()
        .join("uprooted_profiler.dll")
        .to_string_lossy()
        .to_string();

    // DOTNET_ prefix (primary — .NET 10+)
    env_key
        .set_value("DOTNET_EnableDiagnostics", &"1")
        .map_err(|e| format!("Failed to set DOTNET_EnableDiagnostics: {}", e))?;
    env_key
        .set_value("DOTNET_ENABLE_PROFILING", &"1")
        .map_err(|e| format!("Failed to set DOTNET_ENABLE_PROFILING: {}", e))?;
    env_key
        .set_value("DOTNET_PROFILER", &PROFILER_GUID)
        .map_err(|e| format!("Failed to set DOTNET_PROFILER: {}", e))?;
    env_key
        .set_value("DOTNET_PROFILER_PATH", &profiler_path)
        .map_err(|e| format!("Failed to set DOTNET_PROFILER_PATH: {}", e))?;
    env_key
        .set_value("DOTNET_ReadyToRun", &"0")
        .map_err(|e| format!("Failed to set DOTNET_ReadyToRun: {}", e))?;

    // CORECLR_ prefix (legacy — .NET 8/9)
    env_key
        .set_value("CORECLR_ENABLE_PROFILING", &"1")
        .map_err(|e| format!("Failed to set CORECLR_ENABLE_PROFILING: {}", e))?;
    env_key
        .set_value("CORECLR_PROFILER", &PROFILER_GUID)
        .map_err(|e| format!("Failed to set CORECLR_PROFILER: {}", e))?;
    env_key
        .set_value("CORECLR_PROFILER_PATH", &profiler_path)
        .map_err(|e| format!("Failed to set CORECLR_PROFILER_PATH: {}", e))?;

    // Remove legacy startup hooks var if present
    let _ = env_key.delete_value("DOTNET_STARTUP_HOOKS");

    broadcast_env_change();
    Ok(())
}

/// Remove all Uprooted-related environment variables (user-scoped).
#[cfg(target_os = "windows")]
pub fn remove_env_vars() -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env_key = hkcu
        .open_subkey_with_flags("Environment", KEY_WRITE)
        .map_err(|e| format!("Failed to open HKCU\\Environment: {}", e))?;

    for var in ENV_VARS {
        let _ = env_key.delete_value(var);
    }

    broadcast_env_change();
    Ok(())
}

/// Check env var status from the registry (DOTNET_ primary, CORECLR_ fallback).
#[cfg(target_os = "windows")]
fn check_env_vars() -> (bool, bool, bool, bool) {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env_key = match hkcu.open_subkey("Environment") {
        Ok(k) => k,
        Err(_) => return (false, false, false, false),
    };

    let enable: bool = env_key
        .get_value::<String, _>("DOTNET_ENABLE_PROFILING")
        .or_else(|_| env_key.get_value::<String, _>("CORECLR_ENABLE_PROFILING"))
        .map(|v| v == "1")
        .unwrap_or(false);
    let guid: bool = env_key
        .get_value::<String, _>("DOTNET_PROFILER")
        .or_else(|_| env_key.get_value::<String, _>("CORECLR_PROFILER"))
        .map(|v| v == PROFILER_GUID)
        .unwrap_or(false);
    let path: bool = env_key
        .get_value::<String, _>("DOTNET_PROFILER_PATH")
        .or_else(|_| env_key.get_value::<String, _>("CORECLR_PROFILER_PATH"))
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    let r2r: bool = env_key
        .get_value::<String, _>("DOTNET_ReadyToRun")
        .map(|v| v == "0")
        .unwrap_or(false);

    (enable, guid, path, r2r)
}

/// Broadcast WM_SETTINGCHANGE so other processes pick up env var changes.
#[cfg(target_os = "windows")]
fn broadcast_env_change() {
    unsafe {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        let env: Vec<u16> = OsStr::new("Environment")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        // HWND_BROADCAST = 0xFFFF, WM_SETTINGCHANGE = 0x001A
        windows_sys::Win32::UI::WindowsAndMessaging::SendMessageTimeoutW(
            0xFFFF_usize as *mut std::ffi::c_void,
            0x001A,
            0,
            env.as_ptr() as isize,
            0x0002, // SMTO_ABORTIFHUNG
            5000,
            std::ptr::null_mut(),
        );
    }
}

// ==================== Linux: wrapper script + .desktop file ====================

/// Set CLR profiler env vars system-wide on Linux.
///
/// Three mechanisms for maximum compatibility:
/// 1. `~/.config/environment.d/uprooted.conf` -- systemd user session (applies after re-login)
/// 2. Wrapper script `~/.local/share/uprooted/launch-root.sh` -- immediate use from terminal
/// 3. `.desktop` file -- "Root (Uprooted)" app menu entry using the wrapper
#[cfg(target_os = "linux")]
pub fn set_env_vars() -> Result<(), String> {
    let dir = get_uprooted_dir();
    let profiler_path = dir.join(PROFILER_FILENAME);
    let root_path = crate::detection::get_root_exe_path();

    // 1. systemd environment.d -- session-wide env vars (like Windows registry)
    let home = std::env::var("HOME").unwrap_or_default();
    let env_dir = PathBuf::from(&home).join(".config/environment.d");
    fs::create_dir_all(&env_dir)
        .map_err(|e| format!("Failed to create environment.d: {}", e))?;

    let env_conf = format!(
        "# Uprooted CLR profiler -- remove this file or run the uninstaller to disable\n\
# .NET 10+ (DOTNET_ prefix)\n\
DOTNET_EnableDiagnostics=1\n\
DOTNET_ENABLE_PROFILING=1\n\
DOTNET_PROFILER={guid}\n\
DOTNET_PROFILER_PATH={path}\n\
DOTNET_ReadyToRun=0\n\
# Legacy (.NET 8/9)\n\
CORECLR_ENABLE_PROFILING=1\n\
CORECLR_PROFILER={guid}\n\
CORECLR_PROFILER_PATH={path}\n",
        guid = PROFILER_GUID,
        path = profiler_path.display()
    );
    fs::write(env_dir.join("uprooted.conf"), &env_conf)
        .map_err(|e| format!("Failed to write environment.d/uprooted.conf: {}", e))?;

    // 2. Wrapper script -- works immediately from terminal
    let wrapper = dir.join("launch-root.sh");
    let script = format!(
        "#!/bin/bash\n\
# Uprooted launcher - sets CLR profiler env vars for Root only\n\
# .NET 10+ (DOTNET_ prefix)\n\
export DOTNET_EnableDiagnostics=1\n\
export DOTNET_ENABLE_PROFILING=1\n\
export DOTNET_PROFILER='{guid}'\n\
export DOTNET_PROFILER_PATH='{path}'\n\
export DOTNET_ReadyToRun=0\n\
# Legacy (.NET 8/9)\n\
export CORECLR_ENABLE_PROFILING=1\n\
export CORECLR_PROFILER='{guid}'\n\
export CORECLR_PROFILER_PATH='{path}'\n\
exec '{root}' \"$@\"\n",
        guid = PROFILER_GUID,
        path = profiler_path.display(),
        root = root_path.display()
    );
    fs::write(&wrapper, &script)
        .map_err(|e| format!("Failed to write wrapper script: {}", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        let _ = std::fs::set_permissions(&wrapper, perms);
    }

    // 3. .desktop file
    create_desktop_file(&wrapper)?;

    // 4. KDE Plasma env script -- sourced on Plasma session startup
    let plasma_env_dir = PathBuf::from(&home).join(".config/plasma-workspace/env");
    let _ = fs::create_dir_all(&plasma_env_dir);
    let plasma_script = format!(
        "#!/bin/sh\n\
# Uprooted CLR profiler -- remove this file or run the uninstaller to disable\n\
# .NET 10+ (DOTNET_ prefix)\n\
export DOTNET_EnableDiagnostics=1\n\
export DOTNET_ENABLE_PROFILING=1\n\
export DOTNET_PROFILER='{guid}'\n\
export DOTNET_PROFILER_PATH='{path}'\n\
export DOTNET_ReadyToRun=0\n\
# Legacy (.NET 8/9)\n\
export CORECLR_ENABLE_PROFILING=1\n\
export CORECLR_PROFILER='{guid}'\n\
export CORECLR_PROFILER_PATH='{path}'\n",
        guid = PROFILER_GUID,
        path = profiler_path.display()
    );
    let plasma_env_file = plasma_env_dir.join("uprooted.sh");
    let _ = fs::write(&plasma_env_file, &plasma_script);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        let _ = std::fs::set_permissions(&plasma_env_file, perms);
    }

    // 5. ~/.profile fallback -- for non-systemd sessions (X11 login shells, etc.)
    let profile_path = PathBuf::from(&home).join(".profile");
    let profile_content = fs::read_to_string(&profile_path).unwrap_or_default();
    if !profile_content.contains("DOTNET_ENABLE_PROFILING") {
        let block = format!(
            "\n# Uprooted CLR profiler (remove these lines to disable)\n\
# .NET 10+ (DOTNET_ prefix)\n\
export DOTNET_EnableDiagnostics=1\n\
export DOTNET_ENABLE_PROFILING=1\n\
export DOTNET_PROFILER='{guid}'\n\
export DOTNET_PROFILER_PATH='{path}'\n\
export DOTNET_ReadyToRun=0\n\
# Legacy (.NET 8/9)\n\
export CORECLR_ENABLE_PROFILING=1\n\
export CORECLR_PROFILER='{guid}'\n\
export CORECLR_PROFILER_PATH='{path}'\n",
            guid = PROFILER_GUID,
            path = profiler_path.display()
        );
        let mut file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&profile_path)
            .map_err(|e| format!("Failed to append to ~/.profile: {}", e))?;
        use std::io::Write;
        file.write_all(block.as_bytes())
            .map_err(|e| format!("Failed to write to ~/.profile: {}", e))?;
    }

    Ok(())
}

/// Remove all env var mechanisms: environment.d, wrapper script, .desktop file.
#[cfg(target_os = "linux")]
pub fn remove_env_vars() -> Result<(), String> {
    let home = std::env::var("HOME").unwrap_or_default();

    // Remove systemd environment.d config
    let env_conf = PathBuf::from(&home).join(".config/environment.d/uprooted.conf");
    let _ = fs::remove_file(&env_conf);

    // Remove KDE Plasma env script
    let plasma_env_file = PathBuf::from(&home).join(".config/plasma-workspace/env/uprooted.sh");
    let _ = fs::remove_file(&plasma_env_file);

    // Remove wrapper script
    let dir = get_uprooted_dir();
    let wrapper = dir.join("launch-root.sh");
    let _ = fs::remove_file(&wrapper);

    // Remove .desktop file
    let desktop_file = PathBuf::from(&home)
        .join(".local/share/applications/root-uprooted.desktop");
    let _ = fs::remove_file(&desktop_file);

    // Remove env vars from ~/.profile if present
    let profile_path = PathBuf::from(&home).join(".profile");
    if let Ok(content) = fs::read_to_string(&profile_path) {
        if content.contains("DOTNET_ENABLE_PROFILING") || content.contains("CORECLR_ENABLE_PROFILING") {
            // Remove the Uprooted block (comment + 4 export lines + blank line before)
            let cleaned: Vec<&str> = content
                .lines()
                .collect::<Vec<_>>()
                .into_iter()
                .scan(false, |in_block, line| {
                    if line.contains("# Uprooted CLR profiler") {
                        *in_block = true;
                        Some(None) // skip this line
                    } else if *in_block && (line.starts_with("export CORECLR_")
                        || line.starts_with("export DOTNET_")
                        || line.starts_with('#') // skip intermediate comment lines within the block
                        || line.is_empty())
                    {
                        // An empty line signals the end of our block (the block has no trailing
                        // blank line, so an empty line here is the next content's separator)
                        if line.is_empty() {
                            *in_block = false;
                        }
                        Some(None) // skip
                    } else {
                        *in_block = false;
                        Some(Some(line))
                    }
                })
                .flatten()
                .collect();
            let _ = fs::write(&profile_path, cleaned.join("\n") + "\n");
        }
    }

    Ok(())
}

/// Create a .desktop file that launches Root through the wrapper script.
#[cfg(target_os = "linux")]
fn create_desktop_file(wrapper: &PathBuf) -> Result<(), String> {
    let home = std::env::var("HOME").unwrap_or_default();
    let apps_dir = PathBuf::from(&home).join(".local/share/applications");
    fs::create_dir_all(&apps_dir)
        .map_err(|e| format!("Failed to create applications dir: {}", e))?;

    let desktop_content = format!(
        "[Desktop Entry]\n\
Name=Root (Uprooted)\n\
Comment=Root Communications with Uprooted mods\n\
Exec={}\n\
Type=Application\n\
Categories=Network;Chat;\n\
Terminal=false\n",
        wrapper.display()
    );

    let desktop_file = apps_dir.join("root-uprooted.desktop");
    fs::write(&desktop_file, &desktop_content)
        .map_err(|e| format!("Failed to write .desktop file: {}", e))?;

    // chmod +x on desktop file
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        let _ = std::fs::set_permissions(&desktop_file, perms);
    }

    Ok(())
}

/// Check env var status from wrapper script / ~/.zprofile on macOS.
#[cfg(target_os = "macos")]
fn check_env_vars() -> (bool, bool, bool, bool) {
    let home = std::env::var("HOME").unwrap_or_default();
    let dir = get_uprooted_dir();
    let content = fs::read_to_string(dir.join("launch-root.sh"))
        .or_else(|_| fs::read_to_string(PathBuf::from(&home).join(".zprofile")))
        .or_else(|_| fs::read_to_string(PathBuf::from(&home).join(".profile")))
        .unwrap_or_default();

    let enable = content.contains("DOTNET_ENABLE_PROFILING=1")
        || content.contains("CORECLR_ENABLE_PROFILING=1");
    let guid = content.contains(PROFILER_GUID);
    let path = content.contains("DOTNET_PROFILER_PATH=")
        || content.contains("CORECLR_PROFILER_PATH=");
    let r2r = content.contains("DOTNET_ReadyToRun=0");

    (enable, guid, path, r2r)
}

/// Set CLR profiler env vars on macOS via wrapper script + ~/.zprofile.
#[cfg(target_os = "macos")]
pub fn set_env_vars() -> Result<(), String> {
    let dir = get_uprooted_dir();
    let profiler_path = dir.join(PROFILER_FILENAME);
    let root_path = crate::detection::get_root_exe_path();
    let home = std::env::var("HOME").unwrap_or_default();

    // 1. Wrapper script
    let wrapper = dir.join("launch-root.sh");
    let script = format!(
        "#!/bin/bash\n\
# Uprooted launcher - sets CLR profiler env vars for Root only\n\
export DOTNET_EnableDiagnostics=1\n\
export DOTNET_ENABLE_PROFILING=1\n\
export DOTNET_PROFILER='{guid}'\n\
export DOTNET_PROFILER_PATH='{path}'\n\
export DOTNET_ReadyToRun=0\n\
export CORECLR_ENABLE_PROFILING=1\n\
export CORECLR_PROFILER='{guid}'\n\
export CORECLR_PROFILER_PATH='{path}'\n\
exec '{root}' \"$@\"\n",
        guid = PROFILER_GUID,
        path = profiler_path.display(),
        root = root_path.display()
    );
    fs::write(&wrapper, &script)
        .map_err(|e| format!("Failed to write wrapper script: {}", e))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755));
    }

    // 2. ~/.zprofile fallback (macOS default shell is zsh)
    let zprofile = PathBuf::from(&home).join(".zprofile");
    let content = fs::read_to_string(&zprofile).unwrap_or_default();
    if !content.contains("DOTNET_ENABLE_PROFILING") {
        let block = format!(
            "\n# Uprooted CLR profiler (remove these lines to disable)\n\
export DOTNET_EnableDiagnostics=1\n\
export DOTNET_ENABLE_PROFILING=1\n\
export DOTNET_PROFILER='{guid}'\n\
export DOTNET_PROFILER_PATH='{path}'\n\
export DOTNET_ReadyToRun=0\n\
export CORECLR_ENABLE_PROFILING=1\n\
export CORECLR_PROFILER='{guid}'\n\
export CORECLR_PROFILER_PATH='{path}'\n",
            guid = PROFILER_GUID,
            path = profiler_path.display()
        );
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&zprofile)
            .map_err(|e| format!("Failed to append to ~/.zprofile: {}", e))?;
        file.write_all(block.as_bytes())
            .map_err(|e| format!("Failed to write to ~/.zprofile: {}", e))?;
    }

    Ok(())
}

/// Remove env var mechanisms on macOS.
#[cfg(target_os = "macos")]
pub fn remove_env_vars() -> Result<(), String> {
    let home = std::env::var("HOME").unwrap_or_default();
    let dir = get_uprooted_dir();

    // Remove wrapper script
    let _ = fs::remove_file(dir.join("launch-root.sh"));

    // Clean ~/.zprofile
    let zprofile = PathBuf::from(&home).join(".zprofile");
    if let Ok(content) = fs::read_to_string(&zprofile) {
        if content.contains("DOTNET_ENABLE_PROFILING") || content.contains("CORECLR_ENABLE_PROFILING") {
            let cleaned: Vec<&str> = content
                .lines()
                .collect::<Vec<_>>()
                .into_iter()
                .scan(false, |in_block, line| {
                    if line.contains("# Uprooted CLR profiler") {
                        *in_block = true;
                        Some(None)
                    } else if *in_block && (line.starts_with("export CORECLR_")
                        || line.starts_with("export DOTNET_")
                        || line.starts_with('#')
                        || line.is_empty())
                    {
                        if line.is_empty() { *in_block = false; }
                        Some(None)
                    } else {
                        *in_block = false;
                        Some(Some(line))
                    }
                })
                .flatten()
                .collect();
            let _ = fs::write(&zprofile, cleaned.join("\n") + "\n");
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn check_env_vars_active() -> bool {
    let enable = std::env::var("DOTNET_ENABLE_PROFILING")
        .or_else(|_| std::env::var("CORECLR_ENABLE_PROFILING"))
        .map(|v| v == "1")
        .unwrap_or(false);
    let guid = std::env::var("DOTNET_PROFILER")
        .or_else(|_| std::env::var("CORECLR_PROFILER"))
        .map(|v| v == PROFILER_GUID)
        .unwrap_or(false);
    let path = std::env::var("DOTNET_PROFILER_PATH")
        .or_else(|_| std::env::var("CORECLR_PROFILER_PATH"))
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    enable && guid && path
}

/// Check env var status from environment.d config (falls back to wrapper script).
#[cfg(target_os = "linux")]
fn check_env_vars() -> (bool, bool, bool, bool) {
    let home = std::env::var("HOME").unwrap_or_default();

    // Check environment.d first (primary mechanism)
    let env_conf = PathBuf::from(&home).join(".config/environment.d/uprooted.conf");
    let content = fs::read_to_string(&env_conf)
        .or_else(|_| {
            // Fallback: check wrapper script
            let dir = get_uprooted_dir();
            fs::read_to_string(dir.join("launch-root.sh"))
        })
        .or_else(|_| {
            // Fallback: check ~/.profile
            fs::read_to_string(PathBuf::from(&home).join(".profile"))
        })
        .unwrap_or_default();

    let enable = content.contains("DOTNET_ENABLE_PROFILING=1")
        || content.contains("CORECLR_ENABLE_PROFILING=1");
    let guid = content.contains(PROFILER_GUID);
    let path = content.contains("DOTNET_PROFILER_PATH=")
        || content.contains("CORECLR_PROFILER_PATH=");
    let r2r = content.contains("DOTNET_ReadyToRun=0");

    (enable, guid, path, r2r)
}

// ==================== Common: runtime env var check ====================

/// Check if CLR profiler env vars are active in the current process environment.
/// On Windows, registry-based env vars propagate to new processes, so this mirrors
/// the config check. On Linux, environment.d only takes effect after re-login,
/// so this detects the gap between "configured" and "actually active".
#[cfg(target_os = "windows")]
fn check_env_vars_active() -> bool {
    // On Windows, if the config (registry) says env vars are set, they'll be
    // active for any newly launched process. Return true if configured.
    let (enable, guid, path, _) = check_env_vars();
    enable && guid && path
}

#[cfg(target_os = "linux")]
fn check_env_vars_active() -> bool {
    let enable = std::env::var("DOTNET_ENABLE_PROFILING")
        .or_else(|_| std::env::var("CORECLR_ENABLE_PROFILING"))
        .map(|v| v == "1")
        .unwrap_or(false);
    let guid = std::env::var("DOTNET_PROFILER")
        .or_else(|_| std::env::var("CORECLR_PROFILER"))
        .map(|v| v == PROFILER_GUID)
        .unwrap_or(false);
    let path = std::env::var("DOTNET_PROFILER_PATH")
        .or_else(|_| std::env::var("CORECLR_PROFILER_PATH"))
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    enable && guid && path
}

// ==================== Common: file operations ====================

/// Delete Uprooted settings files from the profile directory.
/// Removes both uprooted-settings.ini (C# hook) and uprooted-settings.json (TypeScript),
/// plus the message log file. This resets all plugin states, themes, and preferences.
pub fn reset_settings() -> Result<u32, String> {
    let profile = crate::detection::get_profile_dir();
    let files = [
        "uprooted-settings.ini",
        "uprooted-settings.json",
        "uprooted-message-log.dat",
    ];
    let mut deleted = 0u32;
    for name in &files {
        let path = profile.join(name);
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| format!("Failed to delete {}: {}", name, e))?;
            deleted += 1;
        }
    }
    Ok(deleted)
}

/// Delete the uprooted install directory.
pub fn remove_files() -> Result<(), String> {
    let dir = get_uprooted_dir();
    if dir.exists() {
        fs::remove_dir_all(&dir)
            .map_err(|e| format!("Failed to remove {}: {}", dir.display(), e))?;
    }
    Ok(())
}

/// Check per-file and per-env-var status.
pub fn check_hook_status() -> HookStatus {
    let dir = get_uprooted_dir();

    let profiler_dll = dir.join(PROFILER_FILENAME).exists();
    let hook_dll = dir.join("UprootedHook.dll").exists();
    let hook_deps = dir.join("UprootedHook.deps.json").exists();
    let preload_js = dir.join("uprooted-preload.js").exists();
    let theme_css = dir.join("uprooted.css").exists();

    let (env_enable, env_guid, env_path, env_r2r) = check_env_vars();

    let files_ok = profiler_dll && hook_dll && hook_deps && preload_js && theme_css;
    let env_ok = env_enable && env_guid && env_path;

    let env_vars_active = check_env_vars_active();

    HookStatus {
        profiler_dll,
        hook_dll,
        hook_deps,
        preload_js,
        theme_css,
        env_enable_profiling: env_enable,
        env_profiler_guid: env_guid,
        env_profiler_path: env_path,
        env_ready_to_run: env_r2r,
        files_ok,
        env_ok,
        env_vars_active,
    }
}

// ==================== Process management ====================

/// Check if Root is currently running.
pub fn check_root_running() -> bool {
    #[cfg(target_os = "windows")]
    {
        !find_root_pids().is_empty()
    }
    #[cfg(unix)]
    {
        std::process::Command::new("pgrep")
            .arg("-x")
            .arg("Root")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

/// Terminate all Root processes. Returns the number of processes killed.
pub fn kill_root_processes() -> u32 {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::System::Threading::{
            OpenProcess, TerminateProcess, WaitForSingleObject,
            PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
        };

        let pids = find_root_pids();
        let mut killed = 0u32;
        for pid in &pids {
            unsafe {
                let handle = OpenProcess(PROCESS_TERMINATE | PROCESS_SYNCHRONIZE, 0, *pid);
                if !handle.is_null() {
                    if TerminateProcess(handle, 1) != 0 {
                        // Wait up to 5s for the process to fully exit and release file handles
                        WaitForSingleObject(handle, 5000);
                        killed += 1;
                    }
                    CloseHandle(handle);
                }
            }
        }
        killed
    }
    #[cfg(unix)]
    {
        std::process::Command::new("pkill")
            .arg("-x")
            .arg("Root")
            .output()
            .map(|o| if o.status.success() { 1 } else { 0 })
            .unwrap_or(0)
    }
}

/// Find all PIDs for Root.exe (Windows only).
#[cfg(target_os = "windows")]
fn find_root_pids() -> Vec<u32> {
    use std::mem::MaybeUninit;
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::*;

    let mut pids = Vec::new();
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return pids;
        }

        let mut entry: PROCESSENTRY32W = MaybeUninit::zeroed().assume_init();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                let name_len = entry
                    .szExeFile
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(entry.szExeFile.len());
                let name = String::from_utf16_lossy(&entry.szExeFile[..name_len]);
                if name.eq_ignore_ascii_case("Root.exe") {
                    pids.push(entry.th32ProcessID);
                }
                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snapshot);
    }
    pids
}
