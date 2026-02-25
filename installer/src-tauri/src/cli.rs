use crate::{detection, hook, patcher};
use std::fs;

// ANSI color codes
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

fn ok(msg: &str) {
    println!("  {GREEN}\u{2713}{RESET} {msg}");
}

fn fail(msg: &str) {
    println!("  {RED}\u{2717}{RESET} {msg}");
}

fn warn(msg: &str) {
    println!("  {YELLOW}\u{26a0}{RESET} {msg}");
}

fn header(step: &str, title: &str) {
    println!();
    println!("{BOLD}{CYAN}[{step}]{RESET} {BOLD}{title}{RESET}");
    println!("{DIM}{}─{RESET}", "─".repeat(50));
}

// ═══════════════════════════════════════════════════════════════════
// Plain-mode install (--plain)
// ═══════════════════════════════════════════════════════════════════

pub fn run_install_plain() {
    println!();
    println!(
        "{BOLD}  Uprooted v{} — Install{RESET}",
        env!("CARGO_PKG_VERSION")
    );
    println!("{DIM}  {}{RESET}", "═".repeat(40));

    // Check for running Root process
    if hook::check_root_running() {
        let killed = hook::kill_root_processes();
        ok(&format!("Closed Root ({} process{})", killed, if killed == 1 { "" } else { "es" }));
    } else {
        ok("Root is not running");
    }

    // Detect
    let detection = detection::detect();
    if detection.root_found {
        ok(&format!("Root found: {}", detection.root_path));
    } else {
        fail(&format!("Root NOT found: {}", detection.root_path));
        return;
    }

    // Deploy files
    match hook::deploy_files() {
        Ok(()) => ok("Files deployed"),
        Err(e) => {
            fail(&format!("Deploy failed: {e}"));
            return;
        }
    }

    // Set env vars
    match hook::set_env_vars() {
        Ok(()) => ok("Environment variables set"),
        Err(e) => {
            fail(&format!("Env var setup failed: {e}"));
            return;
        }
    }

    // Patch HTML (non-fatal: HTML files only exist after Root has been launched once)
    let result = patcher::install();
    if result.success {
        ok(&result.message);
    } else {
        warn(&format!("{} Launch Root once, then re-run installer or --repair.", result.message));
    }

    println!();
    println!("  {GREEN}{BOLD}\u{2713} Installed{RESET} — restart Root to load Uprooted.");
    println!();
}

// ═══════════════════════════════════════════════════════════════════
// Plain-mode uninstall (--uninstall --plain)
// ═══════════════════════════════════════════════════════════════════

pub fn run_uninstall_plain() {
    println!();
    println!(
        "{BOLD}  Uprooted v{} — Uninstall{RESET}",
        env!("CARGO_PKG_VERSION")
    );
    println!("{DIM}  {}{RESET}", "═".repeat(40));

    // Check for running Root process
    if hook::check_root_running() {
        let killed = hook::kill_root_processes();
        ok(&format!("Closed Root ({} process{})", killed, if killed == 1 { "" } else { "es" }));
    } else {
        ok("Root is not running");
    }

    match hook::remove_env_vars() {
        Ok(()) => ok("Environment variables removed"),
        Err(e) => fail(&format!("Failed to remove env vars: {e}")),
    }

    let result = patcher::uninstall();
    if result.success {
        ok(&result.message);
    } else {
        fail(&format!("HTML restore failed: {}", result.message));
    }

    match hook::reset_settings() {
        Ok(n) => ok(&format!("Settings removed ({} file{} deleted)", n, if n == 1 { "" } else { "s" })),
        Err(e) => fail(&format!("Failed to remove settings: {e}")),
    }

    match hook::remove_files() {
        Ok(()) => ok("Files removed"),
        Err(e) => fail(&format!("Failed to remove files: {e}")),
    }

    println!();
    println!("  {GREEN}{BOLD}\u{2713} Uprooted removed.{RESET}");
    println!();
}

// ═══════════════════════════════════════════════════════════════════
// Plain-mode repair (--repair --plain)
// ═══════════════════════════════════════════════════════════════════

pub fn run_repair_plain() {
    println!();
    println!(
        "{BOLD}  Uprooted v{} — Repair (resets all settings){RESET}",
        env!("CARGO_PKG_VERSION")
    );
    println!("{DIM}  {}{RESET}", "═".repeat(40));

    // Check for running Root process
    if hook::check_root_running() {
        let killed = hook::kill_root_processes();
        ok(&format!("Closed Root ({} process{})", killed, if killed == 1 { "" } else { "es" }));
    } else {
        ok("Root is not running");
    }

    // Reset settings (plugins, themes, preferences)
    match hook::reset_settings() {
        Ok(n) => ok(&format!("Settings reset ({} file{} removed)", n, if n == 1 { "" } else { "s" })),
        Err(e) => {
            fail(&format!("Settings reset failed: {e}"));
            return;
        }
    }

    match hook::deploy_files() {
        Ok(()) => ok("Files re-deployed"),
        Err(e) => {
            fail(&format!("Deploy failed: {e}"));
            return;
        }
    }

    match hook::set_env_vars() {
        Ok(()) => ok("Environment variables set"),
        Err(e) => {
            fail(&format!("Env var setup failed: {e}"));
            return;
        }
    }

    let result = patcher::repair();
    if result.success {
        ok(&result.message);
    } else {
        warn(&format!("{} Launch Root once, then re-run --repair.", result.message));
    }

    println!();
    println!("  {GREEN}{BOLD}\u{2713} Repair complete{RESET} — restart Root to load Uprooted.");
    println!();
}

// ═══════════════════════════════════════════════════════════════════
// Diagnose (--diagnose) — verbose diagnostic output
// ═══════════════════════════════════════════════════════════════════

pub fn run_diagnose() {
    println!();
    println!(
        "{BOLD}  Uprooted v{} — Diagnostics{RESET}",
        env!("CARGO_PKG_VERSION")
    );
    println!("{DIM}  {}{RESET}", "═".repeat(45));
    println!("  Platform: {}", std::env::consts::OS);
    println!("  Arch:     {}", std::env::consts::ARCH);
    println!(
        "  Time:     {}",
        chrono_lite()
    );

    // ── [1/7] Detection ──
    header("1/7", "Detection");
    let detection = detection::detect();

    if detection.root_found {
        ok(&format!("Root found: {}", detection.root_path));
    } else {
        fail(&format!("Root NOT found (expected: {})", detection.root_path));
    }

    let profile_dir = detection::get_profile_dir();
    if profile_dir.exists() {
        ok(&format!("Profile dir: {}", profile_dir.display()));
    } else {
        warn(&format!(
            "Profile dir missing: {} (launch Root once to create it)",
            profile_dir.display()
        ));
    }

    println!("  HTML files found: {}", detection.html_files.len());
    for f in &detection.html_files {
        let short = f
            .rsplit_once(std::path::MAIN_SEPARATOR)
            .and_then(|(parent, _)| {
                parent
                    .rsplit_once(std::path::MAIN_SEPARATOR)
                    .map(|(_, dir)| format!("{dir}/index.html"))
            })
            .unwrap_or_else(|| f.clone());
        println!("    {DIM}{short}{RESET}");
    }

    if detection.is_installed {
        ok("HTML patches detected (installed)");
    } else {
        warn("HTML patches NOT detected (not installed)");
    }

    // Hook status detail
    let hs = &detection.hook_status;
    println!();
    println!("  {BOLD}Hook file status:{RESET}");
    status_line("profiler dll/so", hs.profiler_dll);
    status_line("UprootedHook.dll", hs.hook_dll);
    status_line("UprootedHook.deps.json", hs.hook_deps);
    status_line("uprooted-preload.js", hs.preload_js);
    status_line("uprooted.css", hs.theme_css);

    println!();
    println!("  {BOLD}Env var status:{RESET}");
    status_line("DOTNET_ENABLE_PROFILING", hs.env_enable_profiling);
    status_line("DOTNET_PROFILER", hs.env_profiler_guid);
    status_line("DOTNET_PROFILER_PATH", hs.env_profiler_path);
    status_line("DOTNET_ReadyToRun", hs.env_ready_to_run);

    if hs.env_vars_active {
        ok("Env vars active in current session");
    } else {
        warn("Env vars NOT active in current session (re-login may be needed)");
    }

    // ── [2/7] Process check ──
    header("2/7", "Process check");
    if hook::check_root_running() {
        let killed = hook::kill_root_processes();
        ok(&format!("Closed Root ({} process{}) — file handles released", killed, if killed == 1 { "" } else { "es" }));
    } else {
        ok("Root is not running");
    }

    // ── [3/7] File deployment ──
    header("3/7", "File deployment");
    let deploy_ok = match hook::deploy_files() {
        Ok(()) => {
            ok("Files deployed successfully");
            let dir = hook::get_uprooted_dir();
            let files = [
                "uprooted_profiler.dll",
                "libuprooted_profiler.so",
                "UprootedHook.dll",
                "UprootedHook.deps.json",
                "uprooted-preload.js",
                "uprooted.css",
                "nsfw-filter.js",
                "link-embeds.js",
            ];
            for name in &files {
                let path = dir.join(name);
                if path.exists() {
                    let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                    println!("    {DIM}{name}{RESET} ({} bytes)", format_size(size));
                }
            }
            true
        }
        Err(e) => {
            fail(&format!("File deployment FAILED: {e}"));
            false
        }
    };

    // ── [4/7] Environment variables ──
    header("4/7", "Environment variables");
    match hook::set_env_vars() {
        Ok(()) => {
            ok("Environment variables set");
        }
        Err(e) => {
            fail(&format!("Env var setup FAILED: {e}"));
        }
    }

    // Re-check env vars after setting
    let hs2 = hook::check_hook_status();
    println!("  {BOLD}Post-set verification:{RESET}");
    status_line("DOTNET_ENABLE_PROFILING", hs2.env_enable_profiling);
    status_line("DOTNET_PROFILER", hs2.env_profiler_guid);
    status_line("DOTNET_PROFILER_PATH", hs2.env_profiler_path);
    status_line("DOTNET_ReadyToRun", hs2.env_ready_to_run);

    // ── [5/7] HTML patching ──
    header("5/7", "HTML patching");
    let patch_result = patcher::install();
    if patch_result.success {
        ok(&patch_result.message);
        for f in &patch_result.files_patched {
            println!("    {DIM}{f}{RESET}");
        }
    } else {
        fail(&format!("HTML patching FAILED: {}", patch_result.message));
    }

    // ── [6/7] Post-install verification ──
    header("6/7", "Post-install verification");
    let final_detection = detection::detect();
    let final_hs = &final_detection.hook_status;

    if deploy_ok && final_hs.files_ok {
        ok("All files deployed");
    } else if !deploy_ok {
        fail("File deployment failed (see step 3)");
    } else {
        fail("Some files missing");
    }

    if final_hs.env_ok {
        ok("Env vars configured");
    } else {
        fail("Env vars incomplete");
    }

    if final_detection.is_installed {
        ok("HTML patches applied");
    } else {
        fail("HTML patches missing");
    }

    // Overall verdict
    let all_good = deploy_ok && final_hs.files_ok && final_hs.env_ok && final_detection.is_installed;
    println!();
    if all_good {
        println!(
            "  {GREEN}{BOLD}\u{2713} Installation looks good!{RESET} Restart Root to activate Uprooted."
        );
    } else {
        println!(
            "  {RED}{BOLD}\u{2717} Installation has issues.{RESET} Check the errors above."
        );
    }

    // ── [7/7] Hook log tail ──
    header("7/7", "Hook log (last 50 lines)");
    let log_path = detection::get_profile_dir().join("uprooted-hook.log");
    if log_path.exists() {
        match fs::read_to_string(&log_path) {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                let start = lines.len().saturating_sub(50);
                ok(&format!(
                    "Log file: {} ({} lines total)",
                    log_path.display(),
                    lines.len()
                ));
                for line in &lines[start..] {
                    println!("    {DIM}{line}{RESET}");
                }
            }
            Err(e) => {
                warn(&format!("Could not read log: {e}"));
            }
        }
    } else {
        warn(&format!(
            "No hook log found at {} (hook has never loaded)",
            log_path.display()
        ));
    }

    println!();
}

fn status_line(label: &str, ok_val: bool) {
    if ok_val {
        println!("    {GREEN}\u{2713}{RESET} {label}");
    } else {
        println!("    {RED}\u{2717}{RESET} {label}");
    }
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes}")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Minimal timestamp without pulling in the chrono crate.
fn chrono_lite() -> String {
    use std::time::SystemTime;
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => {
            let secs = d.as_secs();
            let days = secs / 86400;
            let time_secs = secs % 86400;
            let hours = time_secs / 3600;
            let minutes = (time_secs % 3600) / 60;
            let seconds = time_secs % 60;
            let (year, month, day) = days_to_date(days);
            format!(
                "{year:04}-{month:02}-{day:02} {hours:02}:{minutes:02}:{seconds:02} UTC"
            )
        }
        Err(_) => "unknown".to_string(),
    }
}

fn days_to_date(days: u64) -> (u64, u64, u64) {
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}
