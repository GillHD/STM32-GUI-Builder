use crate::process::{BUILD_CANCEL_NOTIFY, BUILD_CONFIG, kill_process_and_children, kill_build_child_process};
use crate::utils::{LogLevel};
use crate::logging::Logger;
use sysinfo::{System, ProcessesToUpdate};
use tauri::{command, Window, Emitter, Manager};
use tokio::sync::MutexGuard;
use std::time::Duration;

#[command] 
pub async fn cancel_build(window: Window) -> Result<(), String> {
    let mut logger = Logger::new(&window);
    logger.debug("Starting cancel_build process");

    // First mark as cancelled and notify
    {
        let mut config_guard = BUILD_CONFIG.lock().await;
        if let Some(config) = config_guard.as_mut() {
            config.cancelled = Some(true);
        }
    }
    
    BUILD_CANCEL_NOTIFY.notify_waiters();
    logger.debug("Notified cancel waiters");

    // Kill the process first
    match kill_build_child_process().await {
        Ok(_) => {
            logger.debug("Process killed successfully");
        }
        Err(e) => {
            logger.error(&format!("Kill error: {}", e));
        }
    }

    // Additional wait to ensure process cleanup
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Send confirmation events
    logger.info("Build process terminated");
    window.emit("build-cancelled", true).ok();
    logger.debug("Sent build-cancelled event");

    Ok(())
}