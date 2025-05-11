use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuildConfig {
    pub workspace_path: Option<String>,
    pub project_path: String,
    pub build_dir: String,
    pub cube_ide_exe_path: String,
    pub project_name: Option<String>,
    pub config_name: Option<String>,
    pub settings: HashMap<String, serde_json::Value>,
    pub custom_console_args: Option<String>,
    pub clean_build: bool,
    pub cancelled: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct BuildResult {
    pub result: String,
    pub logs: Vec<String>,
    pub stages: Vec<String>,
    pub success: bool,
}