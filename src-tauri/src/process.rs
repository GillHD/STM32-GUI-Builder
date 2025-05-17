use crate::models::BuildConfig;
use crate::utils::{log_with_timestamp, LogLevel};
use sysinfo::{Pid, System, ProcessesToUpdate};
use tauri::{command, Window, Emitter};
use tokio::sync::{Mutex, Notify};
use tokio::time::{self, Duration};
use std::process::Command;
use tokio::process::Child;
use lazy_static::lazy_static;
use winapi::um::wincon::GenerateConsoleCtrlEvent;
use std::sync::Arc;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

// Define BUILD_CONFIG as a static Mutex
lazy_static! {
    pub static ref BUILD_CONFIG: Mutex<Option<BuildConfig>> = Mutex::new(None);
    pub static ref BUILD_CHILD: Mutex<Option<Child>> = Mutex::new(None); // Новый глобальный процесс
    pub static ref BUILD_CANCEL_NOTIFY: Arc<Notify> = Arc::new(Notify::new()); // Add this line
}

#[command]
pub async fn kill_process_and_children(
    pid: u32,
    window: Window,
) -> Result<(), String> {
    let mut logs = Vec::new();
    let mut system = System::new_all();
    system.refresh_all();

    // Проверяем только, что процесс существует (убираем фильтр по имени)
    if system.process(Pid::from(pid as usize)).is_none() {
        let msg = log_with_timestamp(
            &format!("Process with PID {} not found", pid),
            LogLevel::Error,
        );
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        return Err(msg);
    }

    // Attempt soft process termination
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

    // Wait for process termination (10 seconds timeout)
    time::sleep(Duration::from_secs(10)).await;

    // Check if process has terminated
    system.refresh_processes(ProcessesToUpdate::All, true);
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

    // Check child processes
    system.refresh_processes(ProcessesToUpdate::All, true);
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
            // Рекурсивно убиваем всех потомков, независимо от имени
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

// Новый метод для завершения процесса по handle:
#[command]
pub async fn kill_build_child_process() -> Result<(), String> {
    // Use a timeout for the lock acquisition
    let mut child_guard = match tokio::time::timeout(
        Duration::from_secs(1),
        BUILD_CHILD.lock()
    ).await {
        Ok(guard) => guard,
        Err(_) => return Ok(()) // Return OK if we can't get lock
    };

    if let Some(child) = child_guard.as_mut() {
        println!("[KILL] Found active build process");

        #[cfg(windows)]
        {
            // Run taskkill in a separate task to avoid blocking
            let kill_task = tokio::spawn(async {
                Command::new("taskkill")
                    .args(&["/F", "/T", "/IM", "stm32cubeidec.exe"])
                    .creation_flags(CREATE_NO_WINDOW)
                    .output()
            });

            // Wait for taskkill with timeout
            match tokio::time::timeout(Duration::from_secs(2), kill_task).await {
                Ok(result) => match result {
                    Ok(output) => if let Ok(output) = output {
                        println!("[KILL] taskkill result: {}", output.status.success());
                    },
                    Err(e) => println!("[KILL] taskkill task failed: {}", e),
                },
                Err(_) => println!("[KILL] taskkill timeout"),
            }

            // Kill child process without waiting
            let _ = child.kill().await;
            println!("[KILL] Child process kill signal sent");

            // Force drop the handle
            drop(child);
            *child_guard = None;
            println!("[KILL] Process handle released");
        }

        #[cfg(unix)]
        {
            // On Unix, send SIGTERM to process group
            use nix::sys::signal::{self, Signal};
            use nix::unistd::Pid;
            
            let pid = Pid::from_raw(-(child.id().unwrap_or(0) as i32));
            if let Err(e) = signal::kill(pid, Signal::SIGTERM) {
                // Fallback to regular process kill
                if let Err(e2) = child.kill().await {
                    return Err(format!(
                        "Failed to kill process group ({}), and process kill failed: {}", 
                        e, e2
                    ));
                }
            }
            *child_guard = None;
            return Ok(());
        }
    } else {
        println!("[KILL] No active build process found");
    }
    Ok(())
}