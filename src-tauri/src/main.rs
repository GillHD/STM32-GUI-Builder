#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod builder;
mod cancel;
mod config;
mod defaults;
mod models;
mod process;
mod utils;
mod build_combinations;
mod build_config_gen;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            crate::builder::build_project,
            crate::config::load_build_settings_schema, // Fixed: changed from builder to config
            crate::cancel::cancel_build,
            crate::utils::validate_path,
            crate::utils::get_project_configurations,
            crate::utils::get_project_name_from_path,
            crate::config::check_project_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}