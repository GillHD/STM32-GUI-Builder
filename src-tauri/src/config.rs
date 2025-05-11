use tauri::{command, Emitter};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use notify::{Watcher, RecursiveMode, Event, RecommendedWatcher};
use std::sync::mpsc::channel;
use std::thread;
use crate::defaults::DEFAULT_BUILD_SETTINGS;  // Fixed import path

#[command]
pub async fn get_build_settings() -> Result<BuildSettingsConfig, String> {
    BuildSettingsConfig::load()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuildSettingOption {
    pub label: String,
    pub value: String,
    pub define: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuildSetting {
    pub id: String,
    pub label: String,  // Changed from name
    pub description: String,
    pub field_type: String,
    pub format: String,  // Added format field
    pub define: Option<String>,
    pub options: Option<Vec<BuildSettingOption>>,
    pub validation: Option<RangeValidation>,
    pub exclusive: Option<bool>,
    pub min_selected: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RangeValidation {
    pub min: i32,
    pub max: i32
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuildSettingsConfig {
    pub version: String,
    pub build_settings: Vec<BuildSetting>,
}

impl BuildSettingsConfig {
    pub fn load() -> Result<Self, String> {
        let config_path = Path::new("build_settings.json");
        if !config_path.exists() {
            if let Err(e) = fs::write(config_path, DEFAULT_BUILD_SETTINGS) {
                return Err(format!("Failed to create default build_settings.json: {}", e));
            }
        }

        let content = fs::read_to_string(config_path)
            .map_err(|e| format!("Error reading config: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Error parsing config: {}", e))
    }

    pub fn validate_setting(&self, id: &str, value: &serde_json::Value) -> Result<(), String> {
        let setting = self.build_settings.iter().find(|s| s.id == id)
            .ok_or_else(|| format!("Setting {} not found in configuration", id))?;

        match setting.field_type.as_str() {
            "range" => {
                if let Some(validation) = &setting.validation {
                    let values = value.as_array()
                        .ok_or_else(|| format!("Expected array for range setting {}", id))?
                        .iter()
                        .filter_map(|v| v.as_i64().map(|n| n as i32))
                        .collect::<Vec<_>>();
                    for &val in &values {
                        if val < validation.min || val > validation.max {
                            return Err(format!(
                                "Value {} for {} is outside valid range [{}, {}]",
                                val, id, validation.min, validation.max
                            ));
                        }
                    }
                }
            }
            "select" => {
                if let Some(options) = &setting.options {
                    let val = value.as_str()
                        .ok_or_else(|| format!("Expected string for select setting {}", id))?;
                    if !options.iter().any(|opt| opt.value == val) {
                        return Err(format!(
                            "Invalid value '{}' for {}. Valid options: {:?}", 
                            val, id, 
                            options.iter().map(|o| &o.value).collect::<Vec<_>>()
                        ));
                    }
                }
            }
            "checkbox_group" => {
                if let Some(options) = &setting.options {
                    let values = value.as_array()
                        .ok_or_else(|| format!("Expected array for checkbox_group setting {}", id))?
                        .iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>();
                    for val in &values {
                        if !options.iter().any(|opt| opt.value == *val) {
                            return Err(format!(
                                "Invalid value '{}' for {}. Valid options: {:?}", 
                                val, id, 
                                options.iter().map(|o| &o.value).collect::<Vec<_>>()
                            ));
                        }
                    }
                    if let Some(min_selected) = setting.min_selected {
                        if (values.len() as i32) < min_selected {
                            return Err(format!(
                                "Too few selections for {}: {}. Minimum required: {}", 
                                id, values.len(), min_selected
                            ));
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

#[tauri::command]
pub async fn watch_build_settings(window: tauri::Window) {
    let (tx, rx) = channel();

    thread::spawn(move || {
        let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |res| {
            if let Ok(Event { .. }) = res {
                tx.send(()).ok();
            }
        }).expect("Failed to create watcher");

        watcher
            .watch(Path::new("build_settings.json"), RecursiveMode::NonRecursive)
            .expect("Failed to watch build_settings.json");

        loop {
            std::thread::park();
        }
    });

    tauri::async_runtime::spawn({
        let window = window.clone();
        async move {
            while rx.recv().is_ok() {
                if let Ok(config) = BuildSettingsConfig::load() {
                    let _ = window.emit("build-settings-changed", config);
                }
            }
        }
    });
}