use crate::{
    models::{BuildConfig, BuildResult},
    process::BUILD_CONFIG,
    utils::{log_with_timestamp, quote_path, get_project_name, get_cproject_configurations},
    config::BuildSettingsConfig,
    defaults::DEFAULT_BUILD_SETTINGS,  // Fixed import path
};
use chrono::Local;
use serde_json;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tauri::{command, Window, Emitter};
use tokio::process::Command;
use tokio::time::{self, Duration};

#[cfg(windows)]
use winapi::um::wincon::FreeConsole;

// Helper function for dynamic parameter handling
fn format_setting_message(setting_id: &str, value: &serde_json::Value) -> String {
    format!("Setting '{}' with value '{}'", setting_id, value)
}

#[command]
pub async fn build_project(window: Window, config: BuildConfig) -> Result<BuildResult, tauri::Error> {
    let mut logs = Vec::new();
    let mut stages = Vec::new();
    let mut success = true;

    // Load and validate settings configuration
    let settings_config = match BuildSettingsConfig::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            let msg = log_with_timestamp(&format!("Configuration error: {}", e));
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            return Ok(BuildResult { result: msg, logs, stages, success: false });
        }
    };

    // Validate all settings using BuildSettingsConfig::validate_setting
    for setting in &settings_config.build_settings {
        if let Some(value) = config.settings.get(&setting.id) {
            let msg = log_with_timestamp(&format!("{}", format_setting_message(&setting.id, value)));
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            
            if let Err(e) = settings_config.validate_setting(&setting.id, value) {
                let msg = log_with_timestamp(&format!("Validation error for {}: {}", setting.id, e));
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
                return Ok(BuildResult { result: msg, logs, stages, success: false });
            }
        }
    }

    // Load settings schema
    let _schema = match load_build_settings_schema() {
        Ok(s) => s,
        Err(e) => {
            let msg = log_with_timestamp(&format!("Build settings schema error: {}", e));
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            return Ok(BuildResult { result: msg, logs, stages, success: false });
        }
    };

    // Log received configuration
    let config_json = serde_json::to_string(&config).unwrap_or_else(|_| "Error serializing config".to_string());
    logs.push(log_with_timestamp(&format!("Received config: {}", config_json)));
    window.emit("build-log", &logs.last().unwrap()).ok();

    // Check workspace_path
    let workspace_path = config.workspace_path.clone().ok_or_else(|| {
        let msg = log_with_timestamp("Error: workspace_path is not specified in the configuration");
        logs.push(msg.clone());
        window.emit("build-log", &msg).unwrap_or(());
        tauri::Error::from(anyhow::anyhow!(msg))
    })?;
    logs.push(log_with_timestamp(&format!("Using workspace: {}", workspace_path)));
    window.emit("build-log", &logs.last().unwrap()).ok();

    // Verify workspace existence
    let workspace_dir = Path::new(&workspace_path);
    if !workspace_dir.exists() || !workspace_dir.is_dir() {
        let msg = log_with_timestamp(&format!("Error: Workspace '{}' does not exist", workspace_path));
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        return Ok(BuildResult { result: msg, logs, stages, success: false });
    }

    // Clone and update build configuration
    let mut build_config = config.clone();
    build_config.cancelled = Some(build_config.cancelled.unwrap_or(false));

    {
        let mut config_guard = BUILD_CONFIG.lock().await;
        *config_guard = Some(build_config.clone());
        logs.push(log_with_timestamp("Build configuration saved in BUILD_CONFIG"));
        window.emit("build-log", &logs.last().unwrap()).ok();
    }

    // Check for cancellation
    if build_config.cancelled.unwrap_or(false) {
        let msg = log_with_timestamp("Build was cancelled before starting");
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        return Ok(BuildResult { result: msg, logs, stages, success: false });
    }

    // Start build process
    let start_msg = log_with_timestamp("Starting project build");
    stages.push(start_msg.clone());
    logs.push(start_msg.clone());
    window.emit("build-log", &start_msg).ok();

    // Verify STM32CubeIDE path
    stages.push("Validating STM32CubeIDE EXE path".to_string());
    let exe_path_msg = log_with_timestamp(&format!("cube_ide_exe_path: {}", build_config.cube_ide_exe_path));
    logs.push(exe_path_msg.clone());
    window.emit("build-log", &exe_path_msg).ok();
    let cube_ide_exe = Path::new(&build_config.cube_ide_exe_path);
    if !cube_ide_exe.exists() || !cube_ide_exe.is_file() {
        let msg = log_with_timestamp(&format!("Error: STM32CubeIDE EXE '{}' not found or is not a file", build_config.cube_ide_exe_path));
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        return Ok(BuildResult { result: msg, logs, stages, success: false });
    }

    // Set up paths
    let project_path = Path::new(&build_config.project_path);
    let build_config_file = project_path.join("Inc/build_config.h");
    let output_dir = Path::new(&build_config.build_dir);
    let log_file_path = output_dir.join("build_log.txt");
    let stm32_log_filename = format!("stm32_build_{}.txt", Local::now().format("%Y%m%d_%H%M%S"));
    let stm32_log_file_path = output_dir.join(&stm32_log_filename);

    // Verify directories
    stages.push("Checking and creating directories".to_string());
    let project_path_msg = log_with_timestamp(&format!("project_path: {}", build_config.project_path));
    logs.push(project_path_msg.clone());
    window.emit("build-log", &project_path_msg).ok();
    let build_dir_msg = log_with_timestamp(&format!("build_dir: {}", build_config.build_dir));
    logs.push(build_dir_msg.clone());
    window.emit("build-log", &build_dir_msg).ok();
    if !project_path.exists() {
        let msg = log_with_timestamp(&format!("Error: Project directory '{}' not found", build_config.project_path));
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        return Ok(BuildResult { result: msg, logs, stages, success: false });
    }
    if let Err(e) = fs::create_dir_all(&output_dir) {
        let msg = log_with_timestamp(&format!("Error creating directory '{}': {}", output_dir.display(), e));
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        return Ok(BuildResult { result: msg, logs, stages, success: false });
    }

    // Verify project files
    stages.push("Checking project files".to_string());
    let project_file = project_path.join(".project");
    let cproject_file = project_path.join(".cproject");
    if !project_file.exists() {
        let msg = log_with_timestamp(&format!("Error: File '{}' not found", project_file.display()));
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        return Ok(BuildResult { result: msg, logs, stages, success: false });
    }
    if !cproject_file.exists() {
        let msg = log_with_timestamp(&format!("Error: File '{}' not found", cproject_file.display()));
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        return Ok(BuildResult { result: msg, logs, stages, success: false });
    }

    // Verify .cproject configurations
    match get_cproject_configurations(project_path) {
        Ok(configs) => {
            let msg = log_with_timestamp(&format!("Found configurations in .cproject: {:?}", configs));
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            let expected_config = build_config.config_name.as_deref().unwrap_or("Debug");
            if !configs.contains(&expected_config.to_string()) {
                let msg = log_with_timestamp(&format!("Error: Configuration '{}' not found in .cproject", expected_config));
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
                return Ok(BuildResult { result: msg, logs, stages, success: false });
            }
        }
        Err(e) => {
            let msg = log_with_timestamp(&format!("Error reading .cproject: {}", e));
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            return Ok(BuildResult { result: msg, logs, stages, success: false });
        }
    }

    // Get project name
    stages.push("Extracting project name".to_string());
    let project_name = match build_config.project_name {
        Some(name) => {
            let msg = log_with_timestamp(&format!("Using specified project name: {}", name));
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            name
        }
        None => {
            match get_project_name(project_path) {
                Ok(name) => {
                    let msg = log_with_timestamp(&format!("Project name from .project: {}", name));
                    logs.push(msg.clone());
                    window.emit("build-log", &msg).ok();
                    name
                }
                Err(e) => {
                    let msg = log_with_timestamp(&format!("Error getting project name: {}", e));
                    logs.push(msg.clone());
                    window.emit("build-log", &msg).ok();
                    return Ok(BuildResult { result: msg, logs, stages, success: false });
                }
            }
        }
    };

    // Form build parameter
    stages.push("Forming build parameter".to_string());
    let build_target = match &build_config.config_name {
        Some(config_name) => {
            let target = format!("{}/{}", project_name, config_name);
            let msg = log_with_timestamp(&format!("Using build configuration: {}", target));
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            target
        }
        None => {
            let msg = log_with_timestamp(&format!("Using project name without configuration: {}", project_name));
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            project_name.clone()
        }
    };
    let build_target_quoted = quote_path(&build_target);
    let build_flag = if build_config.clean_build { "-cleanBuild" } else { "-build" };

    // Collect settings values
    let settings_values = settings_config.build_settings.iter().map(|setting| {
        let values = match setting.field_type.as_str() {
            "range" => {
                config.settings.get(&setting.id)
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_i64().map(|n| n.to_string())).collect::<Vec<_>>())
                    .unwrap_or_default()
            }
            "select" => {
                config.settings.get(&setting.id)
                    .and_then(|v| v.as_str().map(|s| vec![s.to_string()]))
                    .unwrap_or_default()
            }
            "checkbox_group" => {
                config.settings.get(&setting.id)
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>())
                    .unwrap_or_default()
            }
            _ => vec![]
        };
        (setting, values)
    }).collect::<Vec<_>>();

    // Validate settings values (redundant due to validate_setting, but kept for logging)
    for (setting, values) in &settings_values {
        let setting_msg = log_with_timestamp(&format!("Processing setting '{}': {:?}", setting.id, values));
        logs.push(setting_msg.clone());
        window.emit("build-log", &setting_msg).ok();
    }

    // Create build combinations
    let mut build_combinations = vec![vec![]];
    for (setting, values) in &settings_values {
        let mut new_combinations = vec![];
        for value in values {
            for combo in &build_combinations {
                let mut new_combo = combo.clone();
                new_combo.push((setting.id.clone(), value.clone()));
                new_combinations.push(new_combo);
            }
        }
        build_combinations = new_combinations;
    }

    // Build for each combination
    for combination in build_combinations {
        // Check for cancellation
        {
            let config_guard = BUILD_CONFIG.lock().await;
            if let Some(conf) = &*config_guard {
                if conf.cancelled.unwrap_or(false) {
                    let msg = log_with_timestamp(&format!("Build cancelled for combination {:?}", combination));
                    logs.push(msg.clone());
                    window.emit("build-log", &msg).ok();
                    success = false;
                    return Ok(BuildResult { result: msg, logs, stages, success });
                }
            }
        }

        // Form combination directory
        let mut combo_dir_name = String::new();
        let mut name_parts = vec![project_name.clone()];
        for (setting_id, value) in &combination {
            combo_dir_name.push_str(&format!("{}_{}_", setting_id, value));
            name_parts.push(format!("{}-{}", setting_id, value));
        }
        let combo_dir = output_dir.join(combo_dir_name.trim_end_matches('_'));
        
        if let Err(e) = fs::create_dir_all(&combo_dir) {
            let msg = log_with_timestamp(&format!("Error creating directory '{}': {}", combo_dir.display(), e));
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            success = false;
            return Ok(BuildResult { result: msg, logs, stages, success });
        }

        // Form file names
        let bin_name = format!("{}_FBOOT.bin", name_parts.join("_"));
        let bin_dst = combo_dir.join(&bin_name);
        let log_name = format!("{}_FBOOT.log", name_parts.join("_"));
        let stm32_log_file = combo_dir.join(&log_name);

        // Log file information
        let files_msg = log_with_timestamp(&format!(
            "Processing build:\nOutput: {}\nLog: {}",
            bin_dst.display(),
            stm32_log_file.display()
        ));
        logs.push(files_msg.clone());
        window.emit("build-log", &files_msg).ok();

        // Remove existing .bin
        stages.push(format!("Checking and removing existing .bin file for combination {:?}", combination));
        if bin_dst.exists() {
            let msg = log_with_timestamp(&format!("Found existing file '{}', removing it", bin_dst.display()));
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            if let Err(e) = fs::remove_file(&bin_dst) {
                let msg = log_with_timestamp(&format!("Error removing existing file '{}': {}", bin_dst.display(), e));
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
                success = false;
                return Ok(BuildResult { result: msg, logs, stages, success });
            }
            let msg = log_with_timestamp(&format!("File '{}' successfully removed", bin_dst.display()));
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
        }

        // Generate build_config.h
        stages.push(format!("Generating build_config.h for combination {:?}", combination));
        let mut build_config_content = String::new();
        build_config_content.push_str("#ifndef BUILD_CONFIG_H_\n#define BUILD_CONFIG_H_\n\n");

        // Log the path where we're trying to create build_config.h
        let build_config_path_msg = log_with_timestamp(&format!(
            "Attempting to create build_config.h at: {}", 
            build_config_file.display()
        ));
        logs.push(build_config_path_msg.clone());
        window.emit("build-log", &build_config_path_msg).ok();

        // Try to create Inc directory if it doesn't exist
        if let Some(parent) = build_config_file.parent() {
            match fs::create_dir_all(parent) {
                Ok(_) => {
                    let msg = log_with_timestamp(&format!("Created directory: {}", parent.display()));
                    logs.push(msg.clone());
                    window.emit("build-log", &msg).ok();
                }
                Err(e) => {
                    let msg = log_with_timestamp(&format!("Error creating directory '{}': {}", parent.display(), e));
                    logs.push(msg.clone());
                    window.emit("build-log", &msg).ok();
                    success = false;
                    return Ok(BuildResult { result: msg, logs, stages, success });
                }
            }
        }

        // Write build_config.h content with error checking
        match File::create(&build_config_file) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(build_config_content.as_bytes()) {
                    let msg = log_with_timestamp(&format!("Error writing to build_config.h: {}", e));
                    logs.push(msg.clone());
                    window.emit("build-log", &msg).ok();
                    success = false;
                    return Ok(BuildResult { result: msg, logs, stages, success });
                }
                let msg = log_with_timestamp(&format!("Successfully wrote build_config.h to {}", build_config_file.display()));
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
            }
            Err(e) => {
                let msg = log_with_timestamp(&format!("Error creating build_config.h: {}", e));
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
                success = false;
                return Ok(BuildResult { result: msg, logs, stages, success });
            }
        }

        // Verify the file was created
        if !build_config_file.exists() {
            let msg = log_with_timestamp("Error: build_config.h was not created successfully");
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            success = false;
            return Ok(BuildResult { result: msg, logs, stages, success });
        }

        // Dynamic generation of define/undef for all settings
        for setting in &settings_config.build_settings {
            let id = &setting.id;
            let value_opt = combination.iter().find(|(s_id, _)| s_id == id).map(|(_, v)| v);

            match setting.field_type.as_str() {
                "range" => {
                    if let Some(value) = value_opt {
                        if let Ok(num) = value.parse::<i32>() {
                            build_config_content.push_str(&format!(
                                "#ifndef {}\n#define {} {}\n#endif\n",
                                setting.id.to_uppercase(), setting.id.to_uppercase(), num
                            ));
                        }
                    }
                }
                "select" => {
                    if let Some(options) = &setting.options {
                        for opt in options {
                            let is_selected = value_opt.map_or(false, |v| v == &opt.value);
                            if let Some(define) = &opt.define {
                                if is_selected {
                                    build_config_content.push_str(&format!("#define {}\n", define));
                                } else {
                                    build_config_content.push_str(&format!("#undef {}\n", define));
                                }
                            }
                        }
                    }
                }
                "checkbox_group" => {
                    if let Some(options) = &setting.options {
                        for opt in options {
                            let is_selected = value_opt.map_or(false, |v| v == &opt.value);
                            if let Some(define) = &opt.define {
                                if is_selected {
                                    build_config_content.push_str(&format!("#define {}\n", define));
                                } else {
                                    build_config_content.push_str(&format!("#undef {}\n", define));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        build_config_content.push_str("#undef DEBUG_SET\n");
        build_config_content.push_str("\n#endif // BUILD_CONFIG_H_\n");

        // Create and write build_config.h
        if !build_config_file.exists() {
            if let Some(parent) = build_config_file.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    let msg = log_with_timestamp(&format!("Error creating directory '{}': {}", parent.display(), e));
                    logs.push(msg.clone());
                    window.emit("build-log", &msg).ok();
                    success = false;
                    return Ok(BuildResult { result: msg, logs, stages, success });
                }
            }
            if let Err(e) = File::create(&build_config_file) {
                let msg = log_with_timestamp(&format!("Error creating '{}': {}", build_config_file.display(), e));
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
                success = false;
                return Ok(BuildResult { result: msg, logs, stages, success });
            }
            let msg = log_with_timestamp(&format!("File '{}' created", build_config_file.display()));
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
        }

        if let Err(e) = File::create(&build_config_file).and_then(|mut f| f.write_all(build_config_content.as_bytes())) {
            let msg = log_with_timestamp(&format!("Error writing '{}': {}", build_config_file.display(), e));
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            success = false;
            return Ok(BuildResult { result: msg, logs, stages, success });
        }
        let build_config_msg = log_with_timestamp(&format!("File '{}' successfully written with combination {:?}", 
            build_config_file.display(), combination
        ));
        logs.push(build_config_msg.clone());
        window.emit("build-log", &build_config_msg).ok();

        // Launch STM32CubeIDE
        stages.push(format!("Launching build in STM32CubeIDE for combination {:?}", combination));
        logs.push(log_with_timestamp(&format!("Preparing to launch STM32CubeIDE for combination {:?}", combination)));
        window.emit("build-log", &logs.last().unwrap()).ok();

        // Store quoted paths
        let workspace_path_quoted = quote_path(&workspace_path);
        let project_path_quoted = quote_path(&build_config.project_path);
        let build_target_quoted = quote_path(&build_target);

        let mut headless_args = vec![
            "-nosplash",
            "-application",
            "org.eclipse.cdt.managedbuilder.core.headlessbuild",
            "-data",
            &workspace_path_quoted,
            "-importAll",
            &project_path_quoted,
            build_flag,
            &build_target_quoted,
        ];

        // Log command preparation
        let command_msg = log_with_timestamp(&format!(
            "Executing command:\n{} {}",
            build_config.cube_ide_exe_path,
            headless_args.join(" ")
        ));
        logs.push(command_msg.clone());
        window.emit("build-log", &command_msg).ok();

        let mut command = Command::new(&build_config.cube_ide_exe_path);
        command
            .args(&headless_args)
            .kill_on_drop(true)
            .current_dir(&build_config.project_path) // Set working directory
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        #[cfg(windows)]
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW

        // Launch and monitor process
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(e) => {
                let msg = log_with_timestamp(&format!(
                    "Failed to start STM32CubeIDE process: {}. Command: {} {}",
                    e,
                    build_config.cube_ide_exe_path,
                    headless_args.join(" ")
                ));
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
                success = false;
                return Ok(BuildResult { result: msg, logs, stages, success });
            }
        };

        // Process stdout
        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let window_clone = window.clone();
        let stdout_task = tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            while let Ok(n) = reader.read_line(&mut line).await {
                if n == 0 { break; }
                let msg = log_with_timestamp(&format!("[STDOUT] {}", line.trim()));
                window_clone.emit("build-log", &msg).ok();
                line.clear();
            }
        });

        // Process stderr
        let stderr = child.stderr.take().expect("Failed to capture stderr");
        let window_clone = window.clone();
        let stderr_task = tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            while let Ok(n) = reader.read_line(&mut line).await {
                if n == 0 { break; }
                let msg = log_with_timestamp(&format!("[STDERR] {}", line.trim()));
                window_clone.emit("build-log", &msg).ok();
                line.clear();
            }
        });

        // Wait for output handling to complete
        let _ = tokio::try_join!(stdout_task, stderr_task)?;

        // Wait for process completion and check return code
        match child.wait().await {
            Ok(status) => {
                let exit_code = status.code().unwrap_or(-1);
                let status_msg = match exit_code {
                    0 => {
                        let msg = log_with_timestamp("Build process completed successfully (exit code: 0)");
                        logs.push(msg.clone());
                        window.emit("build-log", &msg).ok();
                        true
                    },
                    1 => {
                        let msg = log_with_timestamp("Build failed - Compilation errors (exit code: 1)");
                        logs.push(msg.clone());
                        window.emit("build-log", &msg).ok();
                        false
                    },
                    13 => {
                        let msg = log_with_timestamp("Build failed - Project import failed (exit code: 13)");
                        logs.push(msg.clone());
                        window.emit("build-log", &msg).ok();
                        false
                    },
                    _ => {
                        let msg = log_with_timestamp(&format!(
                            "Build process failed with unexpected exit code: {}", 
                            exit_code
                        ));
                        logs.push(msg.clone());
                        window.emit("build-log", &msg).ok();
                        false
                    }
                };

                if !status_msg {
                    success = false;
                    return Ok(BuildResult { 
                        result: format!("Build failed with exit code: {}", exit_code),
                        logs, 
                        stages, 
                        success 
                    });
                }
            },
            Err(e) => {
                let msg = log_with_timestamp(&format!("Failed to get build process status: {}", e));
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
                success = false;
                return Ok(BuildResult { result: msg, logs, stages, success });
            }
        }

        // Add delay only if build was successful
        time::sleep(Duration::from_secs(2)).await;

        // Check build directory contents
        stages.push(format!("Checking build directory contents for combination {:?}", combination));
        let build_dir_name = build_config.config_name.as_deref().unwrap_or("Debug");
        let build_dir = project_path.join(build_dir_name);
        let build_dir_msg = log_with_timestamp(&format!("Checking directory: {} for combination {:?}", build_dir.display(), combination));
        logs.push(build_dir_msg.clone());
        window.emit("build-log", &build_dir_msg).ok();
        let expected_bin_file = build_dir.join(format!("{}.bin", project_name.to_lowercase()));
        if build_dir.exists() && build_dir.is_dir() {
            match fs::read_dir(&build_dir) {
                Ok(entries) => {
                    let files: Vec<String> = entries
                        .filter_map(|e| e.ok().and_then(|ent| ent.file_name().to_str().map(|s| s.to_string())))
                        .collect();
                    let files_msg = log_with_timestamp(&format!("Contents of directory {}: {:?}", build_dir_name, files));
                    logs.push(files_msg.clone());
                    window.emit("build-log", &files_msg).ok();
                    logs.push(log_with_timestamp(&format!("Expected .bin file: {}", expected_bin_file.display())));
                    window.emit("build-log", &logs.last().unwrap()).ok();
                    if expected_bin_file.exists() {
                        let msg = log_with_timestamp(&format!("Found output file: {}", expected_bin_file.display()));
                        logs.push(msg.clone());
                        window.emit("build-log", &msg).ok();
                    } else {
                        let msg = log_with_timestamp(&format!(
                            "Error: Output file '{}.bin' not found in directory '{}'. Check STM32CubeIDE log '{}'",
                            project_name.to_lowercase(),
                            build_dir.display(),
                            stm32_log_file_path.display()
                        ));
                        logs.push(msg.clone());
                        window.emit("build-log", &msg).ok();
                        success = false;
                        return Ok(BuildResult { result: msg, logs, stages, success });
                    }
                }
                Err(e) => {
                    let msg = log_with_timestamp(&format!("Error reading directory {}: {}", build_dir_name, e));
                    logs.push(msg.clone());
                    window.emit("build-log", &msg).ok();
                    success = false;
                    return Ok(BuildResult { result: msg, logs, stages, success });
                }
            }
        } else {
            let msg = log_with_timestamp(&format!("Error: Directory '{}' does not exist", build_dir.display()));
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            success = false;
            return Ok(BuildResult { result: msg, logs, stages, success });
        }

        // Rename bin file
        stages.push(format!("Renaming output file for combination {:?}", combination));
        let bin_src = expected_bin_file;
        if !bin_src.exists() {
            let msg = log_with_timestamp(&format!(
                "Output file '{}' not found for combination {:?}", 
                bin_src.display(), combination
            ));
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            success = false;
            return Ok(BuildResult { result: msg, logs, stages, success });
        }
        let rename_msg = log_with_timestamp(&format!("Moving '{}' to '{}'", bin_src.display(), bin_dst.display()));
        logs.push(rename_msg.clone());
        window.emit("build-log", &rename_msg).ok();
        if let Err(e) = fs::rename(&bin_src, &bin_dst) {
            let msg = log_with_timestamp(&format!("Error moving '{}': {}", bin_src.display(), e));
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            success = false;
            return Ok(BuildResult { result: msg, logs, stages, success });
        }
    }

    // Write logs
    stages.push("Writing logs".to_string());
    match File::create(&log_file_path).and_then(|mut f| {
        for log in &logs {
            writeln!(f, "{}", log)?;
        }
        Ok(())
    }) {
        Ok(_) => {
            let msg = log_with_timestamp(&format!("Logs written to: {}", log_file_path.display()));
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
        }
        Err(e) => {
            let msg = log_with_timestamp(&format!("Failed to write logs: {}", e));
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            success = false;
            return Ok(BuildResult { result: msg, logs, stages, success });
        }
    }

    // Finalize build result
    stages.push("Build process completed".to_string());
    let last_result = if success {
        let msg = log_with_timestamp("Build process completed successfully");
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        msg
    } else {
        let msg = log_with_timestamp("Build process completed with errors");
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        msg
    };

    Ok(BuildResult { result: last_result, logs, stages, success })
}

#[command]
pub fn load_build_settings_schema() -> Result<BuildSettingsConfig, String> {
    let schema_path = "build_settings.json";
    
    // Create file with default settings if it doesn't exist
    if !Path::new(schema_path).exists() {
        fs::write(schema_path, DEFAULT_BUILD_SETTINGS)
            .map_err(|e| format!("Error creating settings file: {}", e))?;
    }
    
    let content = fs::read_to_string(schema_path)
        .map_err(|e| format!("Error reading build settings schema: {}", e))?;
    
    serde_json::from_str(&content)
        .map_err(|e| format!("Error parsing build settings schema: {}", e))
}

