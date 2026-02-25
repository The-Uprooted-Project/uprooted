/// Embedded binary artifacts for deployment.
///
/// These files are compiled into the installer binary via `include_bytes!()`.
/// The build pipeline stages real builds into `installer/src-tauri/artifacts/`
/// before `cargo tauri build`.

#[cfg(target_os = "windows")]
pub const PROFILER: &[u8] = include_bytes!("../artifacts/uprooted_profiler.dll");
#[cfg(target_os = "linux")]
pub const PROFILER: &[u8] = include_bytes!("../artifacts/libuprooted_profiler.so");
#[cfg(target_os = "macos")]
pub const PROFILER: &[u8] = include_bytes!("../artifacts/libuprooted_profiler.dylib");

pub const HOOK_DLL: &[u8] = include_bytes!("../artifacts/UprootedHook.dll");
pub const HOOK_DEPS_JSON: &[u8] = include_bytes!("../artifacts/UprootedHook.deps.json");
pub const HOOK_DLL_NET9: &[u8] = include_bytes!("../artifacts/UprootedHook.net9.dll");
pub const HOOK_DEPS_JSON_NET9: &[u8] = include_bytes!("../artifacts/UprootedHook.net9.deps.json");
pub const PRELOAD_JS: &[u8] = include_bytes!("../artifacts/uprooted-preload.js");
pub const THEME_CSS: &[u8] = include_bytes!("../artifacts/uprooted.css");
pub const NSFW_FILTER_JS: &[u8] = include_bytes!("../artifacts/nsfw-filter.js");
pub const LINK_EMBEDS_JS: &[u8] = include_bytes!("../artifacts/link-embeds.js");
