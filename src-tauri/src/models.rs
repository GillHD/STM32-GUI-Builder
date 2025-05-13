// use serde::{Deserialize, Serialize};
// use std::collections::HashMap;
use serde::{Serialize};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct BuildConfig {
    #[serde(rename = "projectPath")]
    pub project_path: String,
    #[serde(rename = "buildDir")]
    pub build_dir: String,
    #[serde(rename = "cubeIdeExePath")]
    pub cube_ide_exe_path: String,
    #[serde(rename = "workspacePath")]
    pub workspace_path: String,
    #[serde(rename = "projectName")]
    pub project_name: Option<String>,
    #[serde(rename = "configName")]
    pub config_name: Option<String>,
    #[serde(rename = "cleanBuild")]
    pub clean_build: bool,
    #[serde(rename = "customConsoleArgs")]
    pub custom_console_args: Option<String>,
    pub settings: serde_json::Map<String, serde_json::Value>,
    pub cancelled: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct BuildResult {
    pub result: String,
    pub logs: Vec<String>,
    pub stages: Vec<String>,
    pub success: bool,
}