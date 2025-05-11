use crate::models::BuildConfig;
use crate::utils::log_with_timestamp;
use lazy_static::lazy_static;
use std::sync::Arc;
use tokio::process::Child;
use tokio::sync::Mutex;
use sysinfo::{Pid, System, SystemExt, ProcessExt, PidExt};
use tauri::{Window, Emitter};
use tokio::time::Duration;

#[cfg(windows)]
use winapi::um::{
    processthreadsapi::{OpenProcess, TerminateProcess},
    wincon::{GenerateConsoleCtrlEvent, FreeConsole},
    winnt::PROCESS_TERMINATE,
    handleapi::CloseHandle,
};
#[cfg(unix)]
use nix::{
    sys::signal::{kill, Signal},
    unistd::Pid as NixPid,
};

lazy_static! {
    pub static ref BUILD_PROCESS: Arc<Mutex<Option<(Child, Pid)>>> = Arc::new(Mutex::new(None));
    pub static ref BUILD_CONFIG: Arc<Mutex<Option<BuildConfig>>> = Arc::new(Mutex::new(None));
    pub static ref IS_CANCELLING: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

#[cfg(windows)]
async fn send_soft_terminate(pid: Pid, logs: &mut Vec<String>, window: &Window) -> Result<(), String> {
    // Освобождаем текущую консоль
    unsafe { FreeConsole() };

    // Пытаемся отправить CTRL+C
    let ctrl_c_result = unsafe { GenerateConsoleCtrlEvent(winapi::um::wincon::CTRL_C_EVENT, 0) };
    if ctrl_c_result != 0 {
        logs.push(log_with_timestamp(&format!("Sent CTRL+C to process PID {}", pid)));
        window.emit("build-log", &logs.last().unwrap()).ok();
    } else {
        let error = std::io::Error::last_os_error();
        logs.push(log_with_timestamp(&format!("Failed to send CTRL+C to process PID {}: {}", pid, error)));
        window.emit("build-log", &logs.last().unwrap()).ok();
    }

    // Пытаемся отправить CTRL+BREAK
    let ctrl_break_result = unsafe { GenerateConsoleCtrlEvent(winapi::um::wincon::CTRL_BREAK_EVENT, 0) };
    if ctrl_break_result != 0 {
        logs.push(log_with_timestamp(&format!("Sent CTRL+BREAK to process PID {}", pid)));
        window.emit("build-log", &logs.last().unwrap()).ok();
    } else {
        let error = std::io::Error::last_os_error();
        logs.push(log_with_timestamp(&format!("Failed to send CTRL+BREAK to process PID {}: {}", pid, error)));
        window.emit("build-log", &logs.last().unwrap()).ok();
    }

    if ctrl_c_result != 0 || ctrl_break_result != 0 {
        Ok(())
    } else {
        Err("Both CTRL+C and CTRL+BREAK failed".to_string())
    }
}

#[cfg(unix)]
async fn send_soft_terminate(pid: Pid, logs: &mut Vec<String>, window: &Window) -> Result<(), String> {
    let nix_pid = NixPid::from_raw(pid.as_u32() as i32);
    match kill(nix_pid, Signal::SIGINT) {
        Ok(()) => {
            logs.push(log_with_timestamp(&format!("Sent SIGINT to process PID {}", pid)));
            window.emit("build-log", &logs.last().unwrap()).ok();
            Ok(())
        }
        Err(e) => {
            let msg = format!("Failed to send SIGINT to process PID {}: {}", pid, e);
            logs.push(log_with_timestamp(&msg));
            window.emit("build-log", &msg).ok();
            Err(msg)
        }
    }
}

#[cfg(windows)]
async fn force_terminate_fallback(pid: Pid, logs: &mut Vec<String>, window: &Window) -> Result<(), String> {
    let output = tokio::process::Command::new("taskkill")
        .args(&["/PID", &pid.to_string(), "/F", "/T"])
        .output()
        .await;

    match output {
        Ok(out) if out.status.success() => {
            logs.push(log_with_timestamp(&format!("Force terminated process PID {} and its tree via taskkill", pid)));
            window.emit("build-log", &logs.last().unwrap()).ok();
            Ok(())
        }
        Ok(out) => {
            let err_msg = String::from_utf8_lossy(&out.stderr).to_string();
            let msg = format!("Failed to force terminate process PID {} via taskkill: {}", pid, err_msg);
            logs.push(log_with_timestamp(&msg));
            window.emit("build-log", &msg).ok();
            Err(msg)
        }
        Err(e) => {
            let msg = format!("Error executing taskkill for PID {}: {}", pid, e);
            logs.push(log_with_timestamp(&msg));
            window.emit("build-log", &msg).ok();
            Err(msg)
        }
    }
}

#[cfg(windows)]
fn terminate_process_low_level(pid: Pid, logs: &mut Vec<String>, window: &Window) -> Result<(), String> {
    let handle = unsafe { OpenProcess(PROCESS_TERMINATE, 0, pid.as_u32()) };
    if handle.is_null() {
        let error = std::io::Error::last_os_error();
        let msg = format!("Failed to open process PID {}: {}", pid, error);
        logs.push(log_with_timestamp(&msg));
        window.emit("build-log", &msg).ok();
        return Err(msg);
    }

    let result = unsafe { TerminateProcess(handle, 1) };
    unsafe { CloseHandle(handle) };

    if result != 0 {
        logs.push(log_with_timestamp(&format!("Terminated process PID {} via TerminateProcess", pid)));
        window.emit("build-log", &logs.last().unwrap()).ok();
        Ok(())
    } else {
        let error = std::io::Error::last_os_error();
        let msg = format!("Failed to terminate process PID {}: {}", pid, error);
        logs.push(log_with_timestamp(&msg));
        window.emit("build-log", &msg).ok();
        Err(msg)
    }
}

#[cfg(unix)]
fn terminate_process_low_level(pid: Pid, logs: &mut Vec<String>, window: &Window) -> Result<(), String> {
    let nix_pid = NixPid::from_raw(pid.as_u32() as i32);
    match kill(nix_pid, Signal::SIGKILL) {
        Ok(()) => {
            logs.push(log_with_timestamp(&format!("Terminated process PID {} via SIGKILL", pid)));
            window.emit("build-log", &logs.last().unwrap()).ok();
            Ok(())
        }
        Err(e) => {
            let msg = format!("Failed to terminate process PID {}: {}", pid, e);
            logs.push(log_with_timestamp(&msg));
            window.emit("build-log", &msg).ok();
            Err(msg)
        }
    }
}

async fn terminate_child_processes(
    pid: Pid,
    system: &mut System,
    logs: &mut Vec<String>,
    window: &Window,
) -> Result<(), String> {
    system.refresh_processes();

    // Собираем все дочерние процессы в стек для итеративной обработки
    let mut stack: Vec<Pid> = vec![pid];
    let mut processed: Vec<Pid> = vec![];

    while let Some(current_pid) = stack.pop() {
        if processed.contains(&current_pid) {
            continue;
        }
        processed.push(current_pid);

        // Собираем дочерние процессы для текущего PID
        let child_pids: Vec<Pid> = system
            .processes()
            .iter()
            .filter_map(|(&child_pid, child_process)| {
                if child_process.parent() == Some(current_pid) {
                    Some(child_pid)
                } else {
                    None
                }
            })
            .collect();

        // Добавляем дочерние процессы в стек
        stack.extend(child_pids.iter().copied());

        // Пытаемся завершить текущий процесс
        if let Some(child_process) = system.process(current_pid) {
            logs.push(log_with_timestamp(&format!(
                "Attempting to terminate process PID {}: {} (status: {:?})",
                current_pid, child_process.name(), child_process.status()
            )));
            window.emit("build-log", &logs.last().unwrap()).ok();

            if let Err(e) = terminate_process_low_level(current_pid, logs, window) {
                logs.push(log_with_timestamp(&format!("Failed to terminate process PID {}: {}", current_pid, e)));
                window.emit("build-log", &logs.last().unwrap()).ok();
                #[cfg(windows)]
                if let Err(e) = force_terminate_fallback(current_pid, logs, window).await {
                    logs.push(log_with_timestamp(&format!("Failed to terminate process PID {} via taskkill: {}", current_pid, e)));
                    window.emit("build-log", &logs.last().unwrap()).ok();
                }
            } else {
                logs.push(log_with_timestamp(&format!("Process PID {} terminated", current_pid)));
                window.emit("build-log", &logs.last().unwrap()).ok();
            }
        }
    }

    Ok(())
}

pub async fn kill_process_and_children(
    pid: Pid,
    logs: &mut Vec<String>,
    window: &Window,
) -> Result<(), String> {
    let mut system = System::new_all();
    system.refresh_processes();

    // Логируем состояние процесса
    if let Some(process) = system.process(pid) {
        logs.push(log_with_timestamp(&format!(
            "Process PID {} status: {:?}, name: {}, cmd: {:?}", 
            pid, process.status(), process.name(), process.cmd()
        )));
        window.emit("build-log", &logs.last().unwrap()).ok();
    } else {
        let msg = log_with_timestamp(&format!("Process with PID {} not found in system", pid));
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        return Err(format!("Process with PID {} not found", pid));
    }

    // Завершаем дочерние процессы
    if let Err(e) = terminate_child_processes(pid, &mut system, logs, window).await {
        logs.push(log_with_timestamp(&format!("Error terminating child processes for PID {}: {}", pid, e)));
        window.emit("build-log", &logs.last().unwrap()).ok();
    }

    let mut process_guard = BUILD_PROCESS.lock().await;
    if let Some((mut child, tracked_pid)) = process_guard.take() {
        if tracked_pid != pid {
            let msg = log_with_timestamp(&format!("PID {} does not match tracked PID {}, skipping", pid, tracked_pid));
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            return Err(format!("PID {} does not match tracked PID {}", pid, tracked_pid));
        }

        logs.push(log_with_timestamp(&format!("Attempting to terminate process with PID: {}", pid)));
        window.emit("build-log", &logs.last().unwrap()).ok();

        // Проверяем, жив ли процесс
        if let Ok(Some(status)) = child.try_wait() {
            let msg = log_with_timestamp(&format!("Process PID {} already exited with status: {}", pid, status));
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            return Ok(());
        }

        // Пытаемся мягко завершить процесс
        if let Ok(()) = send_soft_terminate(pid, logs, window).await {
            // Ждём дольше, чтобы процесс успел завершиться
            tokio::time::sleep(Duration::from_secs(5)).await;
            if let Ok(Some(status)) = child.try_wait() {
                let msg = log_with_timestamp(&format!("Process PID {} terminated softly with status: {}", pid, status));
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
                return Ok(());
            } else {
                logs.push(log_with_timestamp(&format!("Process PID {} still running after soft terminate attempt", pid)));
                window.emit("build-log", &logs.last().unwrap()).ok();
            }
        }

        // Если мягкое завершение не сработало, используем принудительное завершение
        if let Err(e) = child.kill().await {
            logs.push(log_with_timestamp(&format!("Failed to terminate process PID {} via tokio: {}", pid, e)));
            window.emit("build-log", &logs.last().unwrap()).ok();
            // Пробуем завершить через низкоуровневый API
            if let Err(e) = terminate_process_low_level(pid, logs, window) {
                logs.push(log_with_timestamp(&format!("Failed to terminate process PID {} via low-level API: {}", pid, e)));
                window.emit("build-log", &logs.last().unwrap()).ok();
                // Пробуем taskkill как запасной вариант (Windows only)
                #[cfg(windows)]
                if let Err(e) = force_terminate_fallback(pid, logs, window).await {
                    logs.push(log_with_timestamp(&format!("Failed to terminate process PID {} via taskkill: {}", pid, e)));
                    window.emit("build-log", &logs.last().unwrap()).ok();
                }
            }
        }

        // Ждём завершения процесса
        match child.wait().await {
            Ok(status) => {
                let msg = log_with_timestamp(&format!("Process with PID {} terminated with status: {}", pid, status));
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
            }
            Err(e) => {
                let msg = log_with_timestamp(&format!("Error waiting for process PID {} to terminate: {}", pid, e));
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
            }
        }

        // Проверяем, завершился ли процесс в системе
        system.refresh_processes();
        if system.process(pid).is_some() {
            logs.push(log_with_timestamp(&format!("Process PID {} is still running after kill attempt", pid)));
            window.emit("build-log", &logs.last().unwrap()).ok();
            // Последняя попытка через низкоуровневый API
            if let Err(e) = terminate_process_low_level(pid, logs, window) {
                logs.push(log_with_timestamp(&format!("Final attempt to terminate process PID {} failed via low-level API: {}", pid, e)));
                window.emit("build-log", &logs.last().unwrap()).ok();
                // Пробуем taskkill как финальный запасной вариант (Windows only)
                #[cfg(windows)]
                if let Err(e) = force_terminate_fallback(pid, logs, window).await {
                    let msg = format!("Final attempt to terminate process PID {} via taskkill failed: {}", pid, e);
                    logs.push(log_with_timestamp(&msg));
                    window.emit("build-log", &msg).ok();
                    return Err(msg);
                }
            }
        }

        // Проверяем оставшиеся процессы, связанные с STM32CubeIDE
        system.refresh_processes();
        for (remaining_pid, process) in system.processes() {
            let name = process.name().to_lowercase();
            if name.contains("stm32cubeide") || name.contains("java") || name.contains("arm-none-eabi") || name.contains("make") {
                logs.push(log_with_timestamp(&format!(
                    "Found remaining process: PID {}, name: {}, status: {:?}", 
                    remaining_pid, name, process.status()
                )));
                window.emit("build-log", &logs.last().unwrap()).ok();
                if let Err(e) = terminate_process_low_level(*remaining_pid, logs, window) {
                    logs.push(log_with_timestamp(&format!("Failed to terminate remaining process PID {}: {}", remaining_pid, e)));
                    window.emit("build-log", &logs.last().unwrap()).ok();
                    #[cfg(windows)]
                    if let Err(e) = force_terminate_fallback(*remaining_pid, logs, window).await {
                        logs.push(log_with_timestamp(&format!("Failed to terminate remaining process PID {} via taskkill: {}", remaining_pid, e)));
                        window.emit("build-log", &logs.last().unwrap()).ok();
                    }
                } else {
                    logs.push(log_with_timestamp(&format!("Terminated remaining process PID {}", remaining_pid)));
                    window.emit("build-log", &logs.last().unwrap()).ok();
                }
            }
        }

        logs.push(log_with_timestamp(&format!("Process with PID {} terminated successfully", pid)));
        window.emit("build-log", &logs.last().unwrap()).ok();
        Ok(())
    } else {
        let msg = log_with_timestamp(&format!("No process found for PID {}", pid));
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        return Err(format!("No process found for PID {}", pid));
    }
}