use crate::models::BuildConfig;
use crate::utils::{log_with_timestamp, LogLevel};
use sysinfo::{Pid, ProcessExt, System, SystemExt};
use tauri::{command, Window, Emitter};
use tokio::sync::Mutex;
use tokio::time::{self, Duration};
use std::process::Command;
use lazy_static::lazy_static;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

// Define BUILD_CONFIG as a static Mutex
lazy_static! {
    pub static ref BUILD_CONFIG: Mutex<Option<BuildConfig>> = Mutex::new(None);
}

#[command]
pub async fn kill_process_and_children(
    pid: u32,
    window: Window,
) -> Result<(), String> {
    let mut logs = Vec::new();
    let mut system = System::new_all();
    system.refresh_processes();

    // Проверка, является ли процесс STM32CubeIDE или java
    if let Some(process) = system.process(Pid::from(pid as usize)) {
        let process_name = process.name().to_lowercase();
        if !process_name.contains("stm32cubeide") && !process_name.contains("java") {
            let msg = log_with_timestamp(
                &format!("Process PID {} is not an STM32CubeIDE process (name: {})", pid, process_name),
                LogLevel::Error,
            );
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            return Err(msg);
        }
    } else {
        let msg = log_with_timestamp(
            &format!("Process with PID {} not found", pid),
            LogLevel::Error,
        );
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        return Err(msg);
    }

    // Попытка мягкого завершения процесса
    let soft_termination_msg = log_with_timestamp(
        &format!("Attempting soft termination for PID {}", pid),
        LogLevel::Info,
    );
    logs.push(soft_termination_msg.clone());
    window.emit("build-log", &soft_termination_msg).ok();

    #[cfg(windows)]
    {
        let taskkill_soft = Command::new("taskkill")
            .args(&["/PID", &pid.to_string()])
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        match taskkill_soft {
            Ok(output) if output.status.success() => {
                let msg = log_with_timestamp(
                    &format!("Soft termination successful for PID {}", pid),
                    LogLevel::Info,
                );
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
            }
            Ok(output) => {
                let msg = log_with_timestamp(
                    &format!(
                        "Soft termination failed for PID {}: {}",
                        pid,
                        String::from_utf8_lossy(&output.stderr)
                    ),
                    LogLevel::Error,
                );
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
            }
            Err(e) => {
                let msg = log_with_timestamp(
                    &format!("Error during soft termination for PID {}: {}", pid, e),
                    LogLevel::Error,
                );
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let kill_soft = Command::new("kill")
            .arg("-15") // SIGTERM
            .arg(pid.to_string())
            .output();

        match kill_soft {
            Ok(output) if output.status.success() => {
                let msg = log_with_timestamp(
                    &format!("Soft termination successful for PID {}", pid),
                    LogLevel::Info,
                );
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
            }
            Ok(output) => {
                let msg = log_with_timestamp(
                    &format!(
                        "Soft termination failed for PID {}: {}",
                        pid,
                        String::from_utf8_lossy(&output.stderr)
                    ),
                    LogLevel::Error,
                );
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
            }
            Err(e) => {
                let msg = log_with_timestamp(
                    &format!("Error during soft termination for PID {}: {}", pid, e),
                    LogLevel::Error,
                );
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let kill_soft = Command::new("kill")
            .arg("-15") // SIGTERM
            .arg(pid.to_string())
            .output();

        match kill_soft {
            Ok(output) if output.status.success() => {
                let msg = log_with_timestamp(
                    &format!("Soft termination successful for PID {}", pid),
                    LogLevel::Info,
                );
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
            }
            Ok(output) => {
                let msg = log_with_timestamp(
                    &format!(
                        "Soft termination failed for PID {}: {}",
                        pid,
                        String::from_utf8_lossy(&output.stderr)
                    ),
                    LogLevel::Error,
                );
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
            }
            Err(e) => {
                let msg = log_with_timestamp(
                    &format!("Error during soft termination for PID {}: {}", pid, e),
                    LogLevel::Error,
                );
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
            }
        }
    }

    // Ожидание завершения процесса (таймаут 10 секунд)
    time::sleep(Duration::from_secs(10)).await;

    // Проверка, завершился ли процесс
    system.refresh_processes();
    if system.process(Pid::from(pid as usize)).is_some() {
        let msg = log_with_timestamp(
            &format!("Process PID {} still running, attempting force kill", pid),
            LogLevel::Info,
        );
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();

        #[cfg(windows)]
        {
            let taskkill_force = Command::new("taskkill")
                .args(&["/F", "/PID", &pid.to_string()])
                .creation_flags(CREATE_NO_WINDOW)
                .output();

            match taskkill_force {
                Ok(output) if output.status.success() => {
                    let msg = log_with_timestamp(
                        &format!("Force termination successful for PID {}", pid),
                        LogLevel::Info,
                    );
                    logs.push(msg.clone());
                    window.emit("build-log", &msg).ok();
                }
                Ok(output) => {
                    let msg = log_with_timestamp(
                        &format!(
                            "Force termination failed for PID {}: {}",
                            pid,
                            String::from_utf8_lossy(&output.stderr)
                        ),
                        LogLevel::Error,
                    );
                    logs.push(msg.clone());
                    window.emit("build-log", &msg).ok();
                    return Err(msg);
                }
                Err(e) => {
                    let msg = log_with_timestamp(
                        &format!("Error during force termination for PID {}: {}", pid, e),
                        LogLevel::Error,
                    );
                    logs.push(msg.clone());
                    window.emit("build-log", &msg).ok();
                    return Err(msg);
                }
            }
        }

        #[cfg(unix)]
        {
            let kill_force = Command::new("kill")
                .args(&["-KILL", &pid.to_string()])
                .output();

            match kill_force {
                Ok(output) if output.status.success() => {
                    let msg = log_with_timestamp(
                        &format!("Force termination successful for PID {}", pid),
                        LogLevel::Info,
                    );
                    logs.push(msg.clone());
                    window.emit("build-log", &msg).ok();
                }
                Ok(output) => {
                    let msg = log_with_timestamp(
                        &format!(
                            "Force termination failed for PID {}: {}",
                            pid,
                            String::from_utf8_lossy(&output.stderr)
                        ),
                        LogLevel::Error,
                    );
                    logs.push(msg.clone());
                    window.emit("build-log", &msg).ok();
                    return Err(msg);
                }
                Err(e) => {
                    let msg = log_with_timestamp(
                        &format!("Error during force termination for PID {}: {}", pid, e),
                        LogLevel::Error,
                    );
                    logs.push(msg.clone());
                    window.emit("build-log", &msg).ok();
                    return Err(msg);
                }
            }
        }
    }

    // Проверка дочерних процессов
    system.refresh_processes();
    let children: Vec<Pid> = system
        .processes()
        .iter()
        .filter(|(_, p)| p.parent() == Some(Pid::from(pid as usize)))
        .map(|(pid, _)| *pid)
        .collect();

    if !children.is_empty() {
        let msg = log_with_timestamp(
            &format!("Found {} child processes for PID {}", children.len(), pid),
            LogLevel::Info,
        );
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();

        for child_pid in children {
            let child_result = Box::pin(kill_process_and_children(
                Into::<usize>::into(child_pid) as u32,
                window.clone(),
            )).await;
            if let Err(e) = child_result {
                let msg = log_with_timestamp(
                    &format!("Failed to kill child PID {}: {}", child_pid, e),
                    LogLevel::Error,
                );
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
                return Err(msg);
            }
        }
    }

    let final_msg = log_with_timestamp(
        &format!("Successfully terminated PID {} and its children", pid),
        LogLevel::Info,
    );
    logs.push(final_msg.clone());
    window.emit("build-log", &final_msg).ok();
    Ok(())
}