use crate::process::{BUILD_CANCEL_NOTIFY, BUILD_CONFIG, kill_process_and_children, kill_build_child_process};
use crate::utils::{log_with_timestamp, LogLevel};
use sysinfo::{System, ProcessesToUpdate};
use tauri::{command, Window, Emitter, Manager}; // Added Manager trait
use tokio::sync::MutexGuard;
use std::time::Duration;

#[command] 
pub async fn cancel_build(window: Window) -> Result<(), String> {
    window.emit("build-log", &log_with_timestamp("[DEBUG] Starting cancel_build process", LogLevel::Debug)).ok();

    // First mark as cancelled and notify
    {
        let mut config_guard = BUILD_CONFIG.lock().await;
        if let Some(config) = config_guard.as_mut() {
            config.cancelled = Some(true);
        }
    }
    
    BUILD_CANCEL_NOTIFY.notify_waiters();
    window.emit("build-log", &log_with_timestamp("[DEBUG] Notified cancel waiters", LogLevel::Debug)).ok();

    // Kill the process first
    match kill_build_child_process().await {
        Ok(_) => {
            window.emit("build-log", &log_with_timestamp("[DEBUG] Process killed successfully", LogLevel::Debug)).ok();
        }
        Err(e) => {
            window.emit("build-log", &log_with_timestamp(&format!("[DEBUG] Kill error: {}", e), LogLevel::Error)).ok();
        }
    }

    // Additional wait to ensure process cleanup
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Send confirmation events
    window.emit("build-log", &log_with_timestamp("Build process terminated", LogLevel::Info)).ok();
    window.emit("build-cancelled", true).ok();
    window.emit("build-log", &log_with_timestamp("[DEBUG] Sent build-cancelled event", LogLevel::Debug)).ok();

    Ok(())
}