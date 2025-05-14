
use tauri::{command};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::defaults::DEFAULT_BUILD_SETTINGS;  


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
    pub label: String,
    pub value: String,  // Added this field
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
        let config_path = Path::new("build_settings.yaml");
        if !config_path.exists() {
            if let Err(e) = fs::write(config_path, DEFAULT_BUILD_SETTINGS) {
                return Err(format!("Failed to create default build_settings.yaml: {}", e));
            }
        }

        let content = fs::read_to_string(config_path)
            .map_err(|e| format!("Error reading config: {}", e))?;

        serde_yaml::from_str(&content)
            .map_err(|e| format!("Error parsing config: {}", e))
    }

    pub fn validate_setting(&self, id: &str, value: &serde_json::Value) -> Result<(), String> {
        let setting = self.build_settings.iter().find(|s| s.id == id)
            .ok_or_else(|| format!("Setting {} not found in configuration", id))?;

        match setting.field_type.as_str() {
            "range" => {
                if let Some(validation) = &setting.validation {
                    let range_str = value.as_str().ok_or_else(|| format!("Expected string for range setting {}", id))?;
                    let numbers = parse_range_string(range_str, validation.min, validation.max)?;
                    // Можно добавить проверку на пустой массив, если нужно
                    if numbers.is_empty() {
                        return Err(format!("No values provided for range '{}'", id));
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

// Сделать функцию публичной для использования в других модулях
pub fn parse_range_string(range_str: &str, min: i32, max: i32) -> Result<Vec<i32>, String> {
    let mut result = Vec::new();
    for part in range_str.split(',') {
        let part = part.trim();
        if part.is_empty() { continue; }
        if let Some((start, end)) = part.split_once('-') {
            let start: i32 = start.trim().parse().map_err(|_| format!("Invalid number '{}'", start))?;
            let end: i32 = end.trim().parse().map_err(|_| format!("Invalid number '{}'", end))?;
            if start > end { return Err(format!("Range start {} > end {}", start, end)); }
            if start < min || end > max { return Err(format!("Range {}-{} out of bounds [{}, {}]", start, end, min, max)); }
            for n in start..=end { result.push(n); }
        } else {
            let n: i32 = part.parse().map_err(|_| format!("Invalid number '{}'", part))?;
            if n < min || n > max { return Err(format!("Value {} out of bounds [{}, {}]", n, min, max)); }
            result.push(n);
        }
    }
    Ok(result)
}

// Добавляем новую команду для проверки наличия build_settings.yaml в проекте
#[command]
pub async fn check_project_settings(project_path: String) -> Result<bool, String> {
    let settings_path = Path::new(&project_path).join("build_settings.yaml");
    Ok(settings_path.exists())
}

// Make load_settings_schema async and rename it
#[command]
pub async fn load_build_settings_schema() -> Result<BuildSettingsConfig, String> {
    let schema_path = "build_settings.yaml";
    
    if !Path::new(schema_path).exists() {
        fs::write(schema_path, DEFAULT_BUILD_SETTINGS)
            .map_err(|e| format!("Error creating settings file: {}", e))?;
    }
    
    let content = tokio::fs::read_to_string(schema_path)
        .await
        .map_err(|e| format!("Error reading build settings schema: {}", e))?;
    
    serde_yaml::from_str(&content)
        .map_err(|e| format!("Error parsing build settings schema: {}", e))
}
