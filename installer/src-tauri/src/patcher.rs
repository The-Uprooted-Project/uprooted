use crate::detection::find_target_html_files;
use crate::hook;
use crate::settings::load_settings;
use serde::Serialize;
use std::fs;
use std::path::Path;

const MARKER_START: &str = "<!-- uprooted:start -->";
const MARKER_END: &str = "<!-- uprooted:end -->";
/// Legacy marker for detection of older installs
const LEGACY_MARKER: &str = "<!-- uprooted -->";
const BACKUP_SUFFIX: &str = ".uprooted.bak";

#[derive(Serialize)]
pub struct PatchResult {
    pub success: bool,
    pub message: String,
    pub files_patched: Vec<String>,
}

/// Check whether a file contains any uprooted injection.
pub fn is_patched(content: &str) -> bool {
    content.contains(MARKER_START)
        || content.contains(LEGACY_MARKER)
        || content.contains("uprooted-preload")
}

pub fn install() -> PatchResult {
    let uprooted_dir = hook::get_uprooted_dir();

    let preload_path = uprooted_dir
        .join("uprooted-preload.js")
        .to_string_lossy()
        .replace('\\', "/");
    let css_path = uprooted_dir
        .join("uprooted.css")
        .to_string_lossy()
        .replace('\\', "/");

    let settings = load_settings();
    // Escape `</` to `<\/` so JSON containing "</script>" can't break out of the script tag
    let settings_json = serde_json::to_string(&settings)
        .unwrap_or_else(|_| "{}".to_string())
        .replace("</", "<\\/");

    // On Linux, paths start with `/` so `file://` + `/home/...` = `file:///home/...` (correct).
    // On Windows, paths start with `C:\` so we need `file:///` to get `file:///C:/...`.
    let file_prefix = if cfg!(target_os = "windows") { "file:///" } else { "file://" };

    let injection = format!(
        "{start}\n    <script>window.__UPROOTED_SETTINGS__={settings};</script>\n    <script src=\"{prefix}{preload}\"></script>\n    <link rel=\"stylesheet\" href=\"{prefix}{css}\">\n    {end}",
        start = MARKER_START,
        end = MARKER_END,
        settings = settings_json,
        prefix = file_prefix,
        preload = preload_path,
        css = css_path,
    );

    let targets = find_target_html_files();
    if targets.is_empty() {
        return PatchResult {
            success: false,
            message: "No target HTML files found in profile directory.".to_string(),
            files_patched: vec![],
        };
    }

    let mut patched = Vec::new();
    for file in &targets {
        let content = match fs::read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                return PatchResult {
                    success: false,
                    message: format!("Failed to read {}: {}", file.display(), e),
                    files_patched: patched,
                };
            }
        };

        // If already patched, strip old injection so we re-patch with current settings
        let content = if is_patched(&content) {
            strip_injection(&content)
        } else {
            content
        };

        // Backup original
        let backup_path_str = format!("{}{}", file.to_string_lossy(), BACKUP_SUFFIX);
        let backup_path = Path::new(&backup_path_str);
        if !backup_path.exists() {
            if let Err(e) = fs::copy(file, backup_path) {
                return PatchResult {
                    success: false,
                    message: format!("Failed to backup {}: {}", file.display(), e),
                    files_patched: patched,
                };
            }
        }

        // Inject before </head>
        let new_content = content.replace("</head>", &format!("    {}\n  </head>", injection));
        // Write atomically: write to temp then rename so a crash mid-write cannot corrupt the file
        let tmp_path = file.with_extension("html.tmp");
        if let Err(e) = fs::write(&tmp_path, &new_content) {
            return PatchResult {
                success: false,
                message: format!("Failed to write temp file {}: {}", tmp_path.display(), e),
                files_patched: patched,
            };
        }
        if let Err(e) = fs::rename(&tmp_path, file) {
            let _ = fs::remove_file(&tmp_path);
            return PatchResult {
                success: false,
                message: format!("Failed to finalize write to {}: {}", file.display(), e),
                files_patched: patched,
            };
        }

        patched.push(file.to_string_lossy().to_string());
    }

    PatchResult {
        success: true,
        message: format!("Uprooted installed. {} files patched.", patched.len()),
        files_patched: patched,
    }
}

pub fn uninstall() -> PatchResult {
    let targets = find_target_html_files();
    let mut restored = Vec::new();

    for file in &targets {
        let content = match fs::read_to_string(file) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if !is_patched(&content) {
            continue;
        }

        // Prefer stripping in-place (preserves current Root HTML)
        let cleaned = strip_injection(&content);
        if cleaned != content {
            let _ = fs::write(file, &cleaned);
            restored.push(file.to_string_lossy().to_string());

            // Clean up backup file if it exists
            let backup_path_str = format!("{}{}", file.to_string_lossy(), BACKUP_SUFFIX);
            let _ = fs::remove_file(Path::new(&backup_path_str));
            continue;
        }

        // Fallback: restore from backup if stripping didn't change anything
        let backup_path_str = format!("{}{}", file.to_string_lossy(), BACKUP_SUFFIX);
        let backup_path = Path::new(&backup_path_str);
        if backup_path.exists() {
            if let Err(e) = fs::copy(backup_path, file) {
                return PatchResult {
                    success: false,
                    message: format!("Failed to restore {}: {}", file.display(), e),
                    files_patched: restored,
                };
            }
            let _ = fs::remove_file(backup_path);
            restored.push(file.to_string_lossy().to_string());
        }
    }

    PatchResult {
        success: true,
        message: format!(
            "Uprooted uninstalled. {} files restored.",
            restored.len()
        ),
        files_patched: restored,
    }
}

/// Strip injected content between start/end markers, legacy markers, and bare uprooted tags
/// (from bash installer which historically didn't use markers).
fn strip_injection(content: &str) -> String {
    // Detect original line ending style so we don't silently convert CRLF to LF
    let line_ending = if content.contains("\r\n") { "\r\n" } else { "\n" };

    let mut result = Vec::new();
    let mut inside_block = false;

    for line in content.lines() {
        if line.contains(MARKER_START) {
            inside_block = true;
            continue;
        }
        if line.contains(MARKER_END) {
            inside_block = false;
            continue;
        }
        if inside_block {
            continue;
        }
        // Legacy: strip lines with old marker
        if line.contains(LEGACY_MARKER) {
            continue;
        }
        // Bare uprooted tags (bash installer without markers)
        if line.contains("uprooted-preload")
            && (line.contains("<script") || line.contains("</script"))
        {
            continue;
        }
        if line.contains("uprooted.css") && line.contains("<link") {
            continue;
        }
        if line.contains("__UPROOTED_SETTINGS__") && line.contains("<script") {
            continue;
        }
        result.push(line);
    }

    result.join(line_ending)
}

pub fn repair() -> PatchResult {
    let targets = find_target_html_files();

    // Strip existing injection in-place (preserves Root's current HTML)
    for file in &targets {
        let content = match fs::read_to_string(file) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if is_patched(&content) {
            let cleaned = strip_injection(&content);
            let _ = fs::write(file, &cleaned);

            // Update backup to current clean state
            let backup_path_str = format!("{}{}", file.to_string_lossy(), BACKUP_SUFFIX);
            let _ = fs::write(Path::new(&backup_path_str), &cleaned);
        }
    }

    // Re-install fresh patches
    install()
}
