#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod builder;
mod cancel;
mod config;
mod defaults;
mod ld_files;
mod models;
mod process;
mod utils;

// use builder::{build_project, load_build_settings_schema};
// use cancel::cancel_build;
// use config::{get_build_settings, watch_build_settings};
// use ld_files::get_ld_files;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            crate::builder::build_project,
            crate::builder::load_build_settings_schema,
            crate::cancel::cancel_build,
            crate::utils::validate_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}