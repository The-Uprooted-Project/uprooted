use crate::detection::get_profile_dir;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Serialize, Deserialize, Clone)]
pub struct PluginSettings {
    pub enabled: bool,
    pub config: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UprootedSettings {
    pub enabled: bool,
    pub plugins: HashMap<String, PluginSettings>,
    pub custom_css: String,
}

impl Default for UprootedSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            plugins: HashMap::new(),
            custom_css: String::new(),
        }
    }
}

fn settings_path() -> std::path::PathBuf {
    get_profile_dir().join("uprooted-settings.json")
}

pub fn load_settings() -> UprootedSettings {
    let path = settings_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str::<UprootedSettings>(&content) {
                return settings;
            }
        }
    }
    UprootedSettings::default()
}

#[allow(dead_code)]
pub fn save_settings(settings: &UprootedSettings) -> Result<(), String> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    let json =
        serde_json::to_string_pretty(settings).map_err(|e| format!("Failed to serialize: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write settings: {}", e))
}
