use crate::process::{kill_process_and_children, BUILD_CONFIG};
use crate::utils::{log_with_timestamp, LogLevel};
//use sysinfo::{Pid, ProcessExt, System, SystemExt};
use sysinfo::{ProcessExt, System, SystemExt};
use tauri::{command, Window, Emitter};
use tokio::sync::MutexGuard;

#[command]
pub async fn cancel_build(window: Window) -> Result<(), String> {
    let mut logs = Vec::new();

    // Lock and update BUILD_CONFIG to mark the build as cancelled
    let mut config_guard: MutexGuard<Option<crate::models::BuildConfig>> = BUILD_CONFIG.lock().await;
    if let Some(config) = config_guard.as_mut() {
        config.cancelled = Some(true);
        let msg = log_with_timestamp("Build cancellation requested", LogLevel::Info);
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
    } else {
        let msg = log_with_timestamp("No active build configuration found to cancel", LogLevel::Error);
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        return Err(msg);
    }

    // Find and terminate STM32CubeIDE processes
    let mut system = System::new_all();
    system.refresh_processes();

    let mut terminated_pids = Vec::new();
    for (pid, process) in system.processes() {
        let process_name = process.name().to_lowercase();
        if process_name.contains("stm32cubeide") || process_name.contains("java") {
            let pid_usize = Into::<usize>::into(*pid);
            let result = Box::pin(kill_process_and_children(pid_usize as u32, window.clone())).await;
            match result {
                Ok(()) => {
                    terminated_pids.push(pid_usize);
                    let msg = log_with_timestamp(
                        &format!("Successfully terminated STM32CubeIDE process PID {}", pid_usize),
                        LogLevel::Info,
                    );
                    logs.push(msg.clone());
                    window.emit("build-log", &msg).ok();
                }
                Err(e) => {
                    let msg = log_with_timestamp(
                        &format!("Failed to terminate STM32CubeIDE process PID {}: {}", pid_usize, e),
                        LogLevel::Error,
                    );
                    logs.push(msg.clone());
                    window.emit("build-log", &msg).ok();
                    return Err(msg);
                }
            }
        }
    }

    if terminated_pids.is_empty() {
        let msg = log_with_timestamp("No STM32CubeIDE processes found to terminate", LogLevel::Info);
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
    } else {
        let msg = log_with_timestamp(
            &format!("Terminated {} STM32CubeIDE processes", terminated_pids.len()),
            LogLevel::Info,
        );
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
    }

    // Final log
    let final_msg = log_with_timestamp("Build cancellation completed", LogLevel::Info);
    logs.push(final_msg.clone());
    window.emit("build-log", &final_msg).ok();

    Ok(())
}