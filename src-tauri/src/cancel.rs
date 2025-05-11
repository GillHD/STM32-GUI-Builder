use crate::process::{BUILD_PROCESS, BUILD_CONFIG, IS_CANCELLING};
use crate::utils::log_with_timestamp;
use sysinfo::{Pid, System, SystemExt, ProcessExt};
use tauri::{command, Window, Emitter};
use tokio::time::{timeout, Duration};

#[command]
pub async fn cancel_build(window: Window) -> Result<(), String> {
    let mut logs = Vec::new();
    let mut is_cancelling = IS_CANCELLING.lock().await;

    if *is_cancelling {
        let msg = log_with_timestamp("Cancellation already in progress, waiting for completion");
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        return Err(msg);
    }

    *is_cancelling = true;
    logs.push(log_with_timestamp("Cancellation started"));
    window.emit("build-log", &logs.last().unwrap()).ok();
    drop(is_cancelling); // Освобождаем блокировку

    // Обновляем конфигурацию для отмены
    {
        let mut config_guard = BUILD_CONFIG.lock().await;
        if let Some(config) = config_guard.as_mut() {
            config.cancelled = Some(true);
            let msg = log_with_timestamp("Build marked as cancelled in BUILD_CONFIG");
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
        } else {
            let msg = log_with_timestamp("No build configuration found in BUILD_CONFIG");
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
        }
    }

    // Получаем информацию о процессе
    let process_guard = BUILD_PROCESS.lock().await;
    let mut pid_to_terminate: Option<Pid> = None;

    if let Some((_, pid)) = process_guard.as_ref() {
        pid_to_terminate = Some(*pid);
        let msg = log_with_timestamp(&format!("Found process PID {} in BUILD_PROCESS", pid));
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
    } else {
        let msg = log_with_timestamp("No process found in BUILD_PROCESS, checking system for STM32CubeIDE processes");
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();

        // Проверяем систему на наличие процессов STM32CubeIDE
        let mut system = System::new_all();
        system.refresh_processes();
        for (pid, process) in system.processes() {
            let name = process.name().to_lowercase();
            if name.contains("stm32cubeide") || name.contains("java") || name.contains("arm-none-eabi") || name.contains("make") {
                pid_to_terminate = Some(*pid);
                let msg = log_with_timestamp(&format!(
                    "Found STM32CubeIDE-related process: PID {}, name: {}, status: {:?}", 
                    pid, name, process.status()
                ));
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
                break; // Завершаем только первый найденный процесс
            }
        }
    }

    drop(process_guard); // Освобождаем блокировку

    // Завершаем процесс с таймаутом
    let result = if let Some(pid) = pid_to_terminate {
        match timeout(
            Duration::from_secs(10),
            crate::process::kill_process_and_children(pid, &mut logs, &window)
        ).await {
            Ok(Ok(())) => {
                let msg = log_with_timestamp(&format!("Process PID {} terminated successfully", pid));
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
                Ok(())
            }
            Ok(Err(e)) => {
                let msg = log_with_timestamp(&format!("Failed to terminate process PID {}: {}", pid, e));
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
                Err(msg)
            }
            Err(_) => {
                let msg = log_with_timestamp(&format!("Timeout while terminating process PID {}", pid));
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
                Err(msg)
            }
        }
    } else {
        let msg = log_with_timestamp("No STM32CubeIDE-related process found to cancel");
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        Err(msg)
    };

    // Повторная проверка системы на наличие оставшихся процессов
    if result.is_ok() {
        let mut system = System::new_all();
        system.refresh_processes();
        for (pid, process) in system.processes() {
            let name = process.name().to_lowercase();
            if name.contains("stm32cubeide") || name.contains("java") || name.contains("arm-none-eabi") || name.contains("make") {
                let msg = log_with_timestamp(&format!(
                    "Found remaining STM32CubeIDE-related process: PID {}, name: {}, status: {:?}", 
                    pid, name, process.status()
                ));
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
                // Пытаемся завершить
                if let Err(e) = crate::process::kill_process_and_children(*pid, &mut logs, &window).await {
                    let msg = log_with_timestamp(&format!("Failed to terminate remaining process PID {}: {}", pid, e));
                    logs.push(msg.clone());
                    window.emit("build-log", &msg).ok();
                } else {
                    let msg = log_with_timestamp(&format!("Remaining process PID {} terminated successfully", pid));
                    logs.push(msg.clone());
                    window.emit("build-log", &msg).ok();
                }
            }
        }
    }

    // Очищаем BUILD_PROCESS
    {
        let mut process_guard = BUILD_PROCESS.lock().await;
        *process_guard = None;
        let msg = log_with_timestamp("BUILD_PROCESS cleared after cancellation attempt");
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
    }

    // Сбрасываем флаг отмены
    let mut is_cancelling = IS_CANCELLING.lock().await;
    *is_cancelling = false;
    let msg = log_with_timestamp("IS_CANCELLING flag reset");
    logs.push(msg.clone());
    window.emit("build-log", &msg).ok();

    // Возвращаем результат
    match result {
        Ok(()) => {
            let msg = log_with_timestamp("Build successfully cancelled");
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            Ok(())
        }
        Err(e) => {
            let msg = log_with_timestamp(&format!("Cancellation failed: {}", e));
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            Err(e)
        }
    }
}