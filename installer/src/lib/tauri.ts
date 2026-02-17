const { invoke } = (window as any).__TAURI__.core;

export interface HookStatus {
  profiler_dll: boolean;
  hook_dll: boolean;
  hook_deps: boolean;
  preload_js: boolean;
  theme_css: boolean;
  env_enable_profiling: boolean;
  env_profiler_guid: boolean;
  env_profiler_path: boolean;
  env_ready_to_run: boolean;
  files_ok: boolean;
  env_ok: boolean;
  env_vars_active: boolean;
}

export interface DetectionResult {
  root_found: boolean;
  root_path: string;
  profile_dir: string;
  html_files: string[];
  is_installed: boolean;
  hook_status: HookStatus;
}

export interface PatchResult {
  success: boolean;
  message: string;
  files_patched: string[];
}

export interface PreviewColors {
  background: string;
  text: string;
  accent: string;
  border: string;
}

export interface ThemeDefinition {
  name: string;
  display_name: string;
  description: string;
  author: string;
  variables: Record<string, string>;
  preview_colors: PreviewColors;
}

export interface PluginSettings {
  enabled: boolean;
  config: Record<string, unknown>;
}

export interface UprootedSettings {
  enabled: boolean;
  plugins: Record<string, PluginSettings>;
  customCss: string;
}

export async function detectRoot(): Promise<DetectionResult> { return invoke("detect_root"); }
export async function installUprooted(): Promise<PatchResult> { return invoke("install_uprooted"); }
export async function uninstallUprooted(): Promise<PatchResult> { return invoke("uninstall_uprooted"); }
export async function repairUprooted(): Promise<PatchResult> { return invoke("repair_uprooted"); }
export async function loadSettings(): Promise<UprootedSettings> { return invoke("load_settings"); }
export async function saveSettings(settings: UprootedSettings): Promise<void> { return invoke("save_settings", { settings }); }
export async function listThemes(): Promise<ThemeDefinition[]> { return invoke("list_themes"); }
export async function applyTheme(name: string): Promise<void> { return invoke("apply_theme", { name }); }
export async function getUprootedVersion(): Promise<string> { return invoke("get_uprooted_version"); }
export async function openProfileDir(): Promise<void> { return invoke("open_profile_dir"); }
export async function checkHookStatus(): Promise<HookStatus> { return invoke("check_hook_status"); }
export async function checkRootRunning(): Promise<boolean> { return invoke("check_root_running"); }
export async function killRoot(): Promise<number> { return invoke("kill_root"); }
