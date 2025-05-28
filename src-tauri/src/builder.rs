use crate::{
    build_combinations::generate_build_combinations,
    build_config_gen::generate_build_config_h,
    models::{BuildConfig, BuildResult},
    process::{BUILD_CANCEL_NOTIFY, BUILD_CONFIG, BUILD_CHILD},
    utils::{/* log_with_timestamp, */ get_project_name, get_cproject_configurations, LogLevel, validate_project_file, validate_cproject_file},
    config::{BuildSettingsConfig, parse_range_string, load_build_settings_schema},
    logging::Logger
};
use serde_json;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tauri::{command, Window, Emitter};
use tokio::process::Command;
use tokio::time::{self, Duration};
use tokio::sync::Notify;
use std::sync::Arc;

// Add platform-specific imports
#[cfg(unix)]
use std::os::unix::process::CommandExt;

// Helper function for formatting setting messages
fn format_setting_message(setting_id: &str, value: &serde_json::Value) -> String {
    format!("Setting '{}' with value '{}'", setting_id, value)
}

#[command]
pub async fn build_project(window: Window, config: BuildConfig) -> Result<BuildResult, tauri::Error> {
    let mut logger = Logger::new(&window);
    let mut stages = Vec::new();
    let mut success = true;

    // Load and validate settings configuration
    let settings_config = match BuildSettingsConfig::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            let msg = logger.error(&format!("Configuration error: {}", e));
            return Ok(BuildResult { 
                result: msg, 
                logs: logger.get_logs().clone(), 
                stages, 
                success: false 
            });
        }
    };

    // Log all settings from frontend
    let settings_json = serde_json::to_string_pretty(&config.settings)
        .unwrap_or_else(|_| "<failed to serialize settings>".to_string());
    logger.debug(&format!("Received settings from frontend:\n{}", settings_json));

    // Log build_settings from schema
    let build_settings_json = serde_json::to_string_pretty(&settings_config.build_settings)
        .unwrap_or_else(|_| "<failed to serialize build_settings>".to_string());
    logger.debug(&format!("Loaded build_settings schema:\n{}", build_settings_json));

    // Validate all settings
    for setting in &settings_config.build_settings {
        if let Some(value) = config.settings.get(&setting.id) {
            let msg = logger.debug(&format!("{}", format_setting_message(&setting.id, value)));

            // Explicitly log if array is empty (for checkbox_group/range)
            if (setting.field_type == "checkbox_group" || setting.field_type == "range")
                && value.is_array() && value.as_array().map(|arr| arr.is_empty()).unwrap_or(false)
            {
                let warn_msg = logger.warning(
                    &format!("Setting '{}' is an empty array (may be optional or missing selection)", setting.id)
                );
            }

            if let Err(e) = settings_config.validate_setting(&setting.id, value) {
                let msg = logger.error(&format!("Validation error for {}: {}", setting.id, e));
                return Ok(BuildResult { result: msg, logs: logger.get_logs().clone(), stages, success: false });
            }
        } else {
            // Explicitly log missing value for parameter
            let warn_msg = logger.warning(
                &format!("Setting '{}' is missing in settings object", setting.id)
            );
        }
    }

    // Load settings schema
    let _schema = match load_build_settings_schema().await {
        Ok(s) => s,
        Err(e) => {
            let msg = logger.error(&format!("Build settings schema error: {}", e));
            return Ok(BuildResult { result: msg, logs: logger.get_logs().clone(), stages, success: false });
        }
    };

    // Check required paths
    if config.project_path.trim().is_empty() || config.build_dir.trim().is_empty() ||
       config.cube_ide_exe_path.trim().is_empty() || config.workspace_path.trim().is_empty() {
        let msg = logger.error("One or more required paths are empty in BuildConfig");
        return Ok(BuildResult { result: msg, logs: logger.get_logs().clone(), stages, success: false });
    }

    // Just copy string, without ok_or_else
    let workspace_path = config.workspace_path.clone();
    let workspace_dir = Path::new(&workspace_path).canonicalize()
        .map_err(|e| {
            let msg = logger.error(&format!("Invalid workspace path '{}': {}", workspace_path, e));
            tauri::Error::from(anyhow::anyhow!(msg))
        })?;
    logger.info(&format!("Using workspace: {}", workspace_path));

    // Check if working directory exists
    if !workspace_dir.exists() || !workspace_dir.is_dir() {
        let msg = logger.error(&format!("Error: Workspace '{}' does not exist", workspace_path));
        return Ok(BuildResult { result: msg, logs: logger.get_logs().clone(), stages, success: false });
    }

    // Clone and update build configuration
    let mut build_config = config.clone();
    build_config.cancelled = Some(build_config.cancelled.unwrap_or(false));
    {
        let mut config_guard = BUILD_CONFIG.lock().await;
        *config_guard = Some(build_config.clone());
        logger.debug("Build configuration saved in BUILD_CONFIG");
    }

    // Check cancellation
    if build_config.cancelled.unwrap_or(false) {
        let msg = logger.info("Build was cancelled before starting");
        return Ok(BuildResult { result: msg, logs: logger.get_logs().clone(), stages, success: false });
    }

    // Start build process
    let start_msg = logger.info("Starting project build");
    stages.push(start_msg.clone());

    // Check STM32CubeIDE path
    stages.push("Validating STM32CubeIDE EXE path".to_string());
    let cube_ide_exe = Path::new(&build_config.cube_ide_exe_path).canonicalize()
        .map_err(|e| {
            let msg = logger.error(&format!("Invalid STM32CubeIDE path '{}': {}", build_config.cube_ide_exe_path, e));
            tauri::Error::from(anyhow::anyhow!(msg))
        })?;
    if !cube_ide_exe.exists() || !cube_ide_exe.is_file() {
        let msg = logger.error(&format!("Error: STM32CubeIDE EXE '{}' not found", build_config.cube_ide_exe_path));
        return Ok(BuildResult { result: msg, logs: logger.get_logs().clone(), stages, success: false });
    }

    // Setup paths
    let project_path = Path::new(&build_config.project_path).canonicalize()
        .map_err(|e| {
            let msg = logger.error(&format!("Invalid project path '{}': {}", build_config.project_path, e));
            tauri::Error::from(anyhow::anyhow!(msg))
        })?;
    let build_config_file = project_path.join("Inc/build_config.h");
    let output_dir = Path::new(&build_config.build_dir).canonicalize()
        .map_err(|e| {
            let msg = logger.error(&format!("Invalid build directory '{}': {}", build_config.build_dir, e));
            tauri::Error::from(anyhow::anyhow!(msg))
        })?;
    let log_file_path = output_dir.join("build_log.txt");

    // Check directories
    stages.push("Checking and creating directories".to_string());
    if !project_path.exists() {
        let msg = logger.error(&format!("Error: Project directory '{}' not found", build_config.project_path));
        return Ok(BuildResult { result: msg, logs: logger.get_logs().clone(), stages, success: false });
    }
    if let Err(e) = fs::create_dir_all(&output_dir) {
        let msg = logger.error(&format!("Error creating directory '{}': {}", output_dir.display(), e));
        return Ok(BuildResult { result: msg, logs: logger.get_logs().clone(), stages, success: false });
    }

    // Check project files
    stages.push("Checking project files".to_string());
    validate_project_file(&project_path)?;
    validate_cproject_file(&project_path)?;

    // Check .cproject configurations
    let configs = get_cproject_configurations(&project_path)
        .map_err(|e| {
            let msg = logger.error(&format!("Error reading .cproject: {}", e));
            tauri::Error::from(anyhow::anyhow!(msg))
        })?;
    let expected_config = build_config.config_name.as_deref().unwrap_or("Debug");
    if !configs.contains(&expected_config.to_string()) {
        let msg = logger.error(&format!("Error: Configuration '{}' not found in .cproject", expected_config));
        return Ok(BuildResult { result: msg, logs: logger.get_logs().clone(), stages, success: false });
    }

    // Get project name
    stages.push("Extracting project name".to_string());
    let project_name = match build_config.project_name {
        Some(name) => name,
        None => get_project_name(&project_path)
            .map_err(|e| {
                let msg = logger.error(&format!("Error getting project name: {}", e));
                tauri::Error::from(anyhow::anyhow!(msg))
            })?,
    };

    // Form build parameter
    stages.push("Forming build parameter".to_string());
    let build_target = match &build_config.config_name {
        Some(config_name) => format!("{}/{}", project_name, config_name),
        None => project_name.clone(),
    };
    let build_flag = if build_config.clean_build { "-cleanBuild" } else { "-build" };

    // Collect settings values
    let settings_values = settings_config.build_settings.iter().map(|setting| {
        let values = match setting.field_type.as_str() {
            "range" => {
                // Get range string and parse it into numbers
                if let Some(value) = config.settings.get(&setting.id) {
                    if let Some(str_val) = value.as_str() {
                        // Use parse_range_string to get numbers
                        if let Some(validation) = &setting.validation {
                            match parse_range_string(str_val, validation.min, validation.max) {
                                Ok(numbers) => numbers.into_iter().map(|n| n.to_string()).collect(),
                                Err(_) => Vec::new()
                            }
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            },
            "select" => config.settings.get(&setting.id)
                .and_then(|v| v.as_str().map(|s| vec![s.to_string()]))
                .unwrap_or_default(),
            "checkbox_group" => config.settings.get(&setting.id)
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>())
                .unwrap_or_default(),
            _ => vec![],
        };
        (setting, values)
    }).collect::<Vec<_>>();

    // Log settings_values in detail
    let settings_values_log = settings_values.iter()
        .map(|(setting, values)| format!("{}: {:?}", setting.id, values))
        .collect::<Vec<_>>()
        .join(", ");
    logger.debug(&format!("settings_values for build combinations: {{ {} }}", settings_values_log));

    // Check: if at least one REQUIRED parameter has no values — error
    let missing_required: Vec<String> = settings_config.build_settings.iter()
        .filter_map(|setting| {
            let value = config.settings.get(&setting.id);
            let values_count = match setting.field_type.as_str() {
                "range" | "checkbox_group" => value
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter(|v| {
                        // Ignore empty strings in array
                        if let Some(s) = v.as_str() {
                            !s.trim().is_empty()
                        } else if v.is_number() {
                            true
                        } else {
                            false
                        }
                    }).count())
                    .unwrap_or(0),
                "select" => value
                    .and_then(|v| v.as_str())
                    .map(|s| if s.trim().is_empty() { 0 } else { 1 })
                    .unwrap_or(0),
                _ => 1,
            };
            // Only if parameter is required (min_selected > 0 or for select always 1)
            let min_required: usize = if setting.field_type == "select" {
                1
            } else {
                setting.min_selected.unwrap_or(0) as usize
            };
            if values_count < min_required {
                Some(setting.id.clone())
            } else {
                None
            }
        })
        .collect();

    if !missing_required.is_empty() {
        let debug_settings = settings_values.iter()
            .map(|(setting, values)| format!("{}: {:?}", setting.id, values))
            .collect::<Vec<_>>()
            .join(", ");
        logger.debug(&format!("Debug: settings_values = {{ {} }}", debug_settings));

        let msg = logger.error(
            &format!("No values provided for required build parameters: {}. Please fill all required build settings.", missing_required.join(", "))
        );
        return Ok(BuildResult { result: msg, logs: logger.get_logs().clone(), stages, success: false });
    }

    // Create combinations for build (detailed logging)
    let build_combinations = generate_build_combinations(&settings_config, &config.settings);

    if build_combinations.is_empty() {
        let msg = logger.error(
            "No build combinations generated. This usually means at least one build parameter has no values. Check settings_values and build_settings."
        );
        return Ok(BuildResult { result: msg, logs: logger.get_logs().clone(), stages, success: false });
    }

    let mut any_build_executed = false;

    // Build for each combination
    for combination in build_combinations {
        any_build_executed = true;
        // Check cancellation
        {
            let config_guard = BUILD_CONFIG.lock().await;
            if let Some(conf) = &*config_guard {
                if conf.cancelled.unwrap_or(false) {
                    let msg = logger.info(&format!("Build cancelled for combination {:?}", combination));
                    success = false;
                    return Ok(BuildResult { result: msg, logs: logger.get_logs().clone(), stages, success });
                }
            }
        }

        // Create combination directory
        let mut combo_dir_name = String::new();
        let mut name_parts = vec![project_name.clone()];
        for (setting_id, value) in &combination {
            // Get the setting object to access its 'value' field
            if let Some(setting) = settings_config.build_settings.iter().find(|s| &s.id == setting_id) {
                combo_dir_name.push_str(&format!("{}_{}_", setting.value, value));
                name_parts.push(format!("{}-{}", setting.value, value));
            }
        }
        
        let combo_dir = output_dir.join(combo_dir_name.trim_end_matches('_'));
        
        if let Err(e) = fs::create_dir_all(&combo_dir) {
            let msg = logger.error(&format!("Error creating directory '{}': {}", combo_dir.display(), e));
            success = false;
            return Ok(BuildResult { result: msg, logs: logger.get_logs().clone(), stages, success });
        }

        // Create file names
        let mut name_parts = Vec::new();
        
        // 1. First 6 characters of project name
        let short_project_name = if project_name.len() > 6 {
            project_name[..6].to_string()
        } else {
            project_name.clone()
        };
        name_parts.push(short_project_name);

        // 2. Value from higher blocks + used lower ones
        for (setting_id, value) in &combination {
            if let Some(setting) = settings_config.build_settings.iter().find(|s| &s.id == setting_id) {
                if !value.is_empty() {
                    name_parts.push(format!("{}-{}", setting.value, value));
                }
            }
        }

        // 3. Build configuration first 5 symbols
        let config_name = build_config.config_name.as_deref().unwrap_or("Debug");
        let short_config = if config_name.len() > 5 {
            &config_name[..5]
        } else {
            config_name
        };
        name_parts.push(short_config.to_string());

        let bin_name = format!("{}.bin", name_parts.join("_"));
        let bin_dst = combo_dir.join(&bin_name);
        let txt_log_name = format!("{}.txt", name_parts.join("_"));
        let txt_log_file = combo_dir.join(&txt_log_name);

        // Find and delete .bin
        stages.push(format!("Checking and removing existing .bin file for combination {:?}", combination));
        if bin_dst.exists() {
            if let Err(e) = fs::remove_file(&bin_dst) {
                let msg = logger.error(&format!("Error removing existing file '{}': {}", bin_dst.display(), e));
                success = false;
                return Ok(BuildResult { result: msg, logs: logger.get_logs().clone(), stages, success });
            }
        }

        // Generate file build_config.h
        stages.push(format!("Generating build_config.h for combination {:?}", combination));
        let build_config_content = generate_build_config_h(&settings_config, &combination)
            .map_err(|e: String| tauri::Error::from(anyhow::anyhow!(e)))?;

        // Create Inc folder
        if let Some(parent) = build_config_file.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                let msg = logger.error(&format!("Error creating directory '{}': {}", parent.display(), e));
                success = false;
                return Ok(BuildResult { result: msg, logs: logger.get_logs().clone(), stages, success });
            }
        }

        // Write build_config.h
        if let Err(e) = File::create(&build_config_file).and_then(|mut f| f.write_all(build_config_content.as_bytes())) {
            let msg = logger.error(&format!("Error writing '{}': {}", build_config_file.display(), e));
            success = false;
            return Ok(BuildResult { result: msg, logs: logger.get_logs().clone(), stages, success });
        }

        // Run STM32CubeIDE
        stages.push(format!("Launching build in STM32CubeIDE for combination {:?}", combination));


        // Create parameters for STM32CubeIDE
        let mut headless_args = vec![
            "-nosplash".to_string(),
            "-application".to_string(),
            "org.eclipse.cdt.managedbuilder.core.headlessbuild".to_string(),
            "-include".to_string(),
            "Inc/build_config.h".to_string(),
            build_flag.to_string(),
            build_target.clone(),
            "-data".to_string(),
            workspace_path.clone(),
        ];
        // Add custom arguments if they exist
        if let Some(ref custom_args) = build_config.custom_console_args {
            headless_args.extend(custom_args.split_whitespace().map(|s| s.to_string()));
        }

        // Add command logging (output as string, not array)
        let msg = logger.info(
            &format!(
                "Executing command: {} {}",
                &build_config.cube_ide_exe_path,
                headless_args
                    .iter()
                    .map(|s| {
                        // Add quotes only if there are spaces
                        if s.contains(' ') { format!("\"{}\"", s) } else { s.clone() }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        );

        let mut command = Command::new(&build_config.cube_ide_exe_path);
        command
            .args(&headless_args)
            .kill_on_drop(true)
            .current_dir(&build_config.project_path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        // Platform-specific settings
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // 0x08000000 = CREATE_NO_WINDOW, 0x00000200 = CREATE_NEW_PROCESS_GROUP
            command.creation_flags(0x08000000 | 0x00000200);
        }

        #[cfg(all(unix, target_os = "macos"))]
        unsafe {
            command.pre_exec(|| {
                libc::setpgid(0, 0);
                Ok(())
            });
        }

        #[cfg(all(unix, target_os = "linux"))]
        unsafe {
            command.pre_exec(|| {
                libc::setpgid(0, 0);
                Ok(())
            });
        }

        let child = command.spawn().map_err(|e| {
            let msg = logger.error(&format!("Failed to start STM32CubeIDE process: {}", e));
            tauri::Error::from(anyhow::anyhow!(msg))
        })?;

        // --- Сохраняем handle процесса ---
        {
            let mut child_guard = BUILD_CHILD.lock().await;
            *child_guard = Some(child);
        }
        // --- конец вставки ---

        // После этого используйте child_guard.as_mut().unwrap() если нужно, или продолжайте работу как раньше:
        // let stdout = child.stdout.take().expect("Failed to capture stdout");
        // ...existing code...
        let mut child_guard = BUILD_CHILD.lock().await;
        let child_ref = child_guard.as_mut().unwrap();
        let stdout = child_ref.stdout.take().expect("Failed to capture stdout");
        let stderr = child_ref.stderr.take().expect("Failed to capture stderr");

        use tokio::io::{AsyncBufReadExt, BufReader};
        let window_clone = window.clone();
        let stdout_task = {
            // Не используем logger и не добавляем timestamp, просто собираем строки для файла
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                let mut stdout_lines = Vec::new();
                while let Ok(Some(line)) = lines.next_line().await {
                    stdout_lines.push(line);
                }
                Ok::<Vec<String>, std::io::Error>(stdout_lines)
            })
        };

        let stderr_window_clone = window.clone();
        let stderr_task = tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            let mut stderr_lines = Vec::new();
            while let Ok(Some(line)) = lines.next_line().await {
                // Не добавляем timestamp, просто пишем в файл
                let log = format!("[STDERR] {}", line.trim());
                stderr_lines.push(log);
            }
            Ok::<Vec<String>, std::io::Error>(stderr_lines)
        });

        // --- асинхронное ожидание с возможностью отмены ---
        let child_wait = child_ref.wait();
        let cancel_notify = BUILD_CANCEL_NOTIFY.clone();

        tokio::select! {
            status = child_wait => {
                let status = status.map_err(|e| {
                    let msg = logger.error(&format!("Process wait failed: {}", e));
                    tauri::Error::from(anyhow::anyhow!(msg))
                })?;

                // Wait for stdout/stderr reading tasks to complete
                let stdout_logs = stdout_task.await.map_err(|e| {
                    let msg = logger.error(&format!("stdout task failed: {}", e));
                    tauri::Error::from(anyhow::anyhow!(msg))
                })??;
                let stderr_logs = stderr_task.await.map_err(|e| {
                    let msg = logger.error(&format!("stderr task failed: {}", e));
                    tauri::Error::from(anyhow::anyhow!(msg))
                })??;

                // Write stdout/stderr to txt_log_file
                if let Ok(mut txt_log_writer) = File::create(&txt_log_file) {
                    for log in &stdout_logs {
                        writeln!(txt_log_writer, "{}", log).ok();
                    }
                    for log in &stderr_logs {
                        writeln!(txt_log_writer, "{}", log).ok();
                    }
                    txt_log_writer.flush().ok();
                } else {
                    let msg = logger.warning(
                        &format!("Failed to create log file '{}'", txt_log_file.display())
                    );
                }

                // Check process status
                let exit_code = status.code().unwrap_or(-1);
                let status_msg = logger.log(
                    &format!("Build process exited with code: {}", exit_code),
                    if exit_code == 0 { LogLevel::Info } else { LogLevel::Error }
                );

                if exit_code != 0 {
                    success = false;
                    return Ok(BuildResult {
                        result: format!("Build failed with exit code: {}", exit_code),
                        logs: logger.get_logs().clone(),
                        stages,
                        success
                    });
                }

                // Add build results check
                time::sleep(Duration::from_secs(2)).await;

                // Check build directory contents
                stages.push(format!("Checking build directory contents for combination {:?}", combination));
                let build_dir_name = build_config.config_name.as_deref().unwrap_or("Debug");
                let build_dir = project_path.join(build_dir_name);
                let expected_bin_file = build_dir.join(format!("{}.bin", project_name.to_lowercase()));
                if !build_dir.exists() || !expected_bin_file.exists() {
                    let msg = logger.error(&format!("Error: Output file '{}.bin' not found in '{}'", project_name.to_lowercase(), build_dir.display()));
                    success = false;
                    return Ok(BuildResult { result: msg, logs: logger.get_logs().clone(), stages, success });
                }

                // Check file size
                if let Ok(metadata) = fs::metadata(&expected_bin_file) {
                    let msg = logger.info(&format!("Output file size: {} bytes", metadata.len()));
                } else {
                    let msg = logger.error(&format!("Failed to get output file metadata: {}", expected_bin_file.display()));
                    success = false;
                    return Ok(BuildResult { result: msg, logs: logger.get_logs().clone(), stages, success });
                }

                // Rename bin file
                stages.push(format!("Renaming output file for combination {:?}", combination));
                if let Err(e) = fs::rename(&expected_bin_file, &bin_dst) {
                    let msg = logger.error(&format!("Error moving '{}' to '{}': {}", expected_bin_file.display(), bin_dst.display(), e));
                    success = false;
                    return Ok(BuildResult { result: msg, logs: logger.get_logs().clone(), stages, success });
                }

                // После завершения:
                {
                    let mut child_guard = BUILD_CHILD.lock().await;
                    *child_guard = None;
                }
            }
            _ = cancel_notify.notified() => {
                println!("[CANCEL] Cancel notification received in builder.rs");
                
                // Notify frontend before killing process
                let msg = logger.info("Build cancellation in progress");
                
                // Kill the process and tasks
                let _ = child_ref.kill().await;
                let _ = stdout_task.abort();
                let _ = stderr_task.abort();
                
                // Wait a bit to ensure process is killed
                tokio::time::sleep(Duration::from_millis(300)).await;
                
                // Release handle and update config atomically
                {
                    let mut child_guard = BUILD_CHILD.lock().await;
                    *child_guard = None;
                    
                    let mut config_guard = BUILD_CONFIG.lock().await;
                    if let Some(config) = config_guard.as_mut() {
                        config.cancelled = Some(true);
                    }
                }

                // Send events in order with confirmation
                let msg = logger.info("Build process cancelled");
                
                // Send build-cancelled event and wait for confirmation
                match window.emit("build-cancelled", true) {
                    Ok(_) => println!("[CANCEL] build-cancelled event sent successfully"),
                    Err(e) => println!("[CANCEL] Failed to send build-cancelled event: {}", e),
                }

                success = false;
                return Ok(BuildResult { 
                    result: msg,
                    logs: logger.get_logs().clone(), 
                    stages,
                    success 
                });
            }
        }
        // --- конец асинхронного ожидания ---

        // ...existing code...
        {
            let mut child_guard = BUILD_CHILD.lock().await;
            *child_guard = None;
        }
    }

    if !any_build_executed {
        let msg = logger.error("No build combinations were executed. Check your build settings.");
        return Ok(BuildResult { 
            result: msg, 
            logs: logger.get_logs().clone(), 
            stages, 
            success: false 
        });
    }

    // Write logs
    stages.push("Writing logs".to_string());
    if let Err(e) = File::create(&log_file_path).and_then(|mut f| {
        for log in logger.get_logs() {
            writeln!(f, "{}", log)?;
        }
        Ok(())
    }) {
        let msg = logger.error(&format!("Failed to write logs: {}", e));
        success = false;
        return Ok(BuildResult { 
            result: msg, 
            logs: logger.get_logs().clone(), 
            stages, 
            success 
        });
    }

    // Finalize build result
    stages.push("Build process completed".to_string());
    let last_result = if success {
        logger.info("Build process completed successfully")
    } else {
        logger.error("Build process completed with errors")
    };

    Ok(BuildResult { 
        result: last_result, 
        logs: logger.get_logs().clone(), 
        stages, 
        success 
    })
}