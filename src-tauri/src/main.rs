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

use builder::{build_project, load_build_settings_schema};
use cancel::cancel_build;
use config::{get_build_settings, watch_build_settings};
use ld_files::get_ld_files;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            build_project, 
            get_ld_files, 
            cancel_build,
            get_build_settings,
            watch_build_settings,
            load_build_settings_schema  // Add this line
        ])
        .run(tauri::generate_context!())
        .expect("Ошибка запуска приложения Tauri");
}