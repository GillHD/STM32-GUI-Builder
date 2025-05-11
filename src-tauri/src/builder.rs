use crate::{
    models::{BuildConfig, BuildResult},
    process::BUILD_CONFIG,
    utils::{log_with_timestamp, quote_path, get_project_name, get_cproject_configurations, LogLevel},
    config::BuildSettingsConfig,
    defaults::DEFAULT_BUILD_SETTINGS,
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

// Вспомогательная функция для форматирования сообщений о настройках
fn format_setting_message(setting_id: &str, value: &serde_json::Value) -> String {
    format!("Setting '{}' with value '{}'", setting_id, value)
}

// Проверка валидности .project файла
fn validate_project_file(project_path: &Path) -> Result<(), tauri::Error> {
    let project_file = project_path.join(".project");
    let content = fs::read_to_string(&project_file)
        .map_err(|e| tauri::Error::from(anyhow::anyhow!("Ошибка чтения '{}': {}", project_file.display(), e)))?;
    if !content.contains("<projectDescription>") {
        return Err(tauri::Error::from(anyhow::anyhow!("Файл '{}' не является валидным .project файлом", project_file.display())));
    }
    Ok(())
}

// Проверка валидности .cproject файла
fn validate_cproject_file(project_path: &Path) -> Result<(), tauri::Error> {
    let cproject_file = project_path.join(".cproject");
    let content = fs::read_to_string(&cproject_file)
        .map_err(|e| tauri::Error::from(anyhow::anyhow!("Ошибка чтения '{}': {}", cproject_file.display(), e)))?;
    if !content.contains("<cproject") {
        return Err(tauri::Error::from(anyhow::anyhow!("Файл '{}' не является валидным .cproject файлом", cproject_file.display())));
    }
    Ok(())
}

#[command]
pub async fn build_project(window: Window, config: BuildConfig) -> Result<BuildResult, tauri::Error> {
    let mut logs = Vec::new();
    let mut stages = Vec::new();
    let mut success = true;

    // Загрузка и валидация конфигурации настроек
    let settings_config = match BuildSettingsConfig::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            let msg = log_with_timestamp(&format!("Configuration error: {}", e), LogLevel::Error);
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            return Ok(BuildResult { result: msg, logs, stages, success: false });
        }
    };

    // Логируем все settings, которые пришли с фронта
    let settings_json = serde_json::to_string_pretty(&config.settings).unwrap_or_else(|_| "<failed to serialize settings>".to_string());
    let msg = log_with_timestamp(&format!("Received settings from frontend:\n{}", settings_json), LogLevel::Debug);
    logs.push(msg.clone());
    window.emit("build-log", &msg).ok();

    // Логируем build_settings из схемы
    let build_settings_json = serde_json::to_string_pretty(&settings_config.build_settings).unwrap_or_else(|_| "<failed to serialize build_settings>".to_string());
    let msg = log_with_timestamp(&format!("Loaded build_settings schema:\n{}", build_settings_json), LogLevel::Debug);
    logs.push(msg.clone());
    window.emit("build-log", &msg).ok();

    // Валидация всех настроек
    for setting in &settings_config.build_settings {
        if let Some(value) = config.settings.get(&setting.id) {
            let msg = log_with_timestamp(&format!("{}", format_setting_message(&setting.id, value)), LogLevel::Debug);
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();

            // Явно логируем если массив пустой (для checkbox_group/range)
            if (setting.field_type == "checkbox_group" || setting.field_type == "range")
                && value.is_array() && value.as_array().map(|arr| arr.is_empty()).unwrap_or(false)
            {
                let warn_msg = log_with_timestamp(
                    &format!("Warning: Setting '{}' is an empty array (may be optional or missing selection)", setting.id),
                    LogLevel::Debug
                );
                logs.push(warn_msg.clone());
                window.emit("build-log", &warn_msg).ok();
            }

            if let Err(e) = settings_config.validate_setting(&setting.id, value) {
                let msg = log_with_timestamp(&format!("Validation error for {}: {}", setting.id, e), LogLevel::Error);
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
                return Ok(BuildResult { result: msg, logs, stages, success: false });
            }
        } else {
            // Явно логируем отсутствие значения для параметра
            let warn_msg = log_with_timestamp(
                &format!("Warning: Setting '{}' is missing in settings object", setting.id),
                LogLevel::Debug
            );
            logs.push(warn_msg.clone());
            window.emit("build-log", &warn_msg).ok();
        }
    }

    // Загрузка схемы настроек
    let _schema = match load_build_settings_schema() {
        Ok(s) => s,
        Err(e) => {
            let msg = log_with_timestamp(&format!("Build settings schema error: {}", e), LogLevel::Error);
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            return Ok(BuildResult { result: msg, logs, stages, success: false });
        }
    };

    // Проверка обязательных путей
    if config.project_path.trim().is_empty() || config.build_dir.trim().is_empty() ||
       config.cube_ide_exe_path.trim().is_empty() || config.workspace_path.trim().is_empty() {
        let msg = log_with_timestamp("One or more required paths are empty in BuildConfig", LogLevel::Error);
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        return Ok(BuildResult { result: msg, logs, stages, success: false });
    }

    // Просто копируем строку, без ok_or_else
    let workspace_path = config.workspace_path.clone();
    let workspace_dir = Path::new(&workspace_path).canonicalize()
        .map_err(|e| {
            let msg = log_with_timestamp(&format!("Invalid workspace path '{}': {}", workspace_path, e), LogLevel::Error);
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            tauri::Error::from(anyhow::anyhow!(msg))
        })?;
    logs.push(log_with_timestamp(&format!("Using workspace: {}", workspace_path), LogLevel::Info));
    window.emit("build-log", &logs.last().unwrap()).ok();

    // Проверка существования рабочей директории
    if !workspace_dir.exists() || !workspace_dir.is_dir() {
        let msg = log_with_timestamp(&format!("Error: Workspace '{}' does not exist", workspace_path), LogLevel::Error);
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        return Ok(BuildResult { result: msg, logs, stages, success: false });
    }

    // Клонирование и обновление конфигурации сборки
    let mut build_config = config.clone();
    build_config.cancelled = Some(build_config.cancelled.unwrap_or(false));
    {
        let mut config_guard = BUILD_CONFIG.lock().await;
        *config_guard = Some(build_config.clone());
        logs.push(log_with_timestamp("Build configuration saved in BUILD_CONFIG", LogLevel::Debug));
        window.emit("build-log", &logs.last().unwrap()).ok();
    }

    // Проверка отмены
    if build_config.cancelled.unwrap_or(false) {
        let msg = log_with_timestamp("Build was cancelled before starting", LogLevel::Info);
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        return Ok(BuildResult { result: msg, logs, stages, success: false });
    }

    // Начало процесса сборки
    let start_msg = log_with_timestamp("Starting project build", LogLevel::Info);
    stages.push(start_msg.clone());
    logs.push(start_msg.clone());
    window.emit("build-log", &start_msg).ok();

    // Проверка пути к STM32CubeIDE
    stages.push("Validating STM32CubeIDE EXE path".to_string());
    let cube_ide_exe = Path::new(&build_config.cube_ide_exe_path).canonicalize()
        .map_err(|e| {
            let msg = log_with_timestamp(&format!("Invalid STM32CubeIDE path '{}': {}", build_config.cube_ide_exe_path, e), LogLevel::Error);
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            tauri::Error::from(anyhow::anyhow!(msg))
        })?;
    if !cube_ide_exe.exists() || !cube_ide_exe.is_file() {
        let msg = log_with_timestamp(&format!("Error: STM32CubeIDE EXE '{}' not found", build_config.cube_ide_exe_path), LogLevel::Error);
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        return Ok(BuildResult { result: msg, logs, stages, success: false });
    }

    // Настройка путей
    let project_path = Path::new(&build_config.project_path).canonicalize()
        .map_err(|e| {
            let msg = log_with_timestamp(&format!("Invalid project path '{}': {}", build_config.project_path, e), LogLevel::Error);
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            tauri::Error::from(anyhow::anyhow!(msg))
        })?;
    let build_config_file = project_path.join("Inc/build_config.h");
    let output_dir = Path::new(&build_config.build_dir).canonicalize()
        .map_err(|e| {
            let msg = log_with_timestamp(&format!("Invalid build directory '{}': {}", build_config.build_dir, e), LogLevel::Error);
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            tauri::Error::from(anyhow::anyhow!(msg))
        })?;
    let log_file_path = output_dir.join("build_log.txt");
    let stm32_log_filename = format!("stm32_build_{}.txt", Local::now().format("%Y%m%d_%H%M%S"));
    let stm32_log_file_path = output_dir.join(&stm32_log_filename);

    // Проверка директорий
    stages.push("Checking and creating directories".to_string());
    if !project_path.exists() {
        let msg = log_with_timestamp(&format!("Error: Project directory '{}' not found", build_config.project_path), LogLevel::Error);
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        return Ok(BuildResult { result: msg, logs, stages, success: false });
    }
    if let Err(e) = fs::create_dir_all(&output_dir) {
        let msg = log_with_timestamp(&format!("Error creating directory '{}': {}", output_dir.display(), e), LogLevel::Error);
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        return Ok(BuildResult { result: msg, logs, stages, success: false });
    }

    // Проверка проектных файлов
    stages.push("Checking project files".to_string());
    validate_project_file(&project_path)?;
    validate_cproject_file(&project_path)?;

    // Проверка конфигураций .cproject
    let configs = get_cproject_configurations(&project_path)
        .map_err(|e| {
            let msg = log_with_timestamp(&format!("Error reading .cproject: {}", e), LogLevel::Error);
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            tauri::Error::from(anyhow::anyhow!(msg))
        })?;
    let expected_config = build_config.config_name.as_deref().unwrap_or("Debug");
    if !configs.contains(&expected_config.to_string()) {
        let msg = log_with_timestamp(&format!("Error: Configuration '{}' not found in .cproject", expected_config), LogLevel::Error);
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        return Ok(BuildResult { result: msg, logs, stages, success: false });
    }

    // Получение имени проекта
    stages.push("Extracting project name".to_string());
    let project_name = match build_config.project_name {
        Some(name) => name,
        None => get_project_name(&project_path)
            .map_err(|e| {
                let msg = log_with_timestamp(&format!("Error getting project name: {}", e), LogLevel::Error);
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
                tauri::Error::from(anyhow::anyhow!(msg))
            })?,
    };

    // Формирование параметра сборки
    stages.push("Forming build parameter".to_string());
    let build_target = match &build_config.config_name {
        Some(config_name) => format!("{}/{}", project_name, config_name),
        None => project_name.clone(),
    };
    let build_flag = if build_config.clean_build { "-cleanBuild" } else { "-build" };

    // Сбор значений настроек
    let settings_values = settings_config.build_settings.iter().map(|setting| {
        let values = match setting.field_type.as_str() {
            "range" => config.settings.get(&setting.id)
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_i64().map(|n| n.to_string())).collect::<Vec<_>>())
                .unwrap_or_default(),
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

    // Подробно логируем settings_values
    let settings_values_log = settings_values.iter()
        .map(|(setting, values)| format!("{}: {:?}", setting.id, values))
        .collect::<Vec<_>>()
        .join(", ");
    let msg = log_with_timestamp(
        &format!("settings_values for build combinations: {{ {} }}", settings_values_log),
        LogLevel::Debug
    );
    logs.push(msg.clone());
    window.emit("build-log", &msg).ok();

    // Проверка: если хотя бы для одного ОБЯЗАТЕЛЬНОГО параметра нет значений — ошибка
    let missing_required: Vec<String> = settings_config.build_settings.iter()
        .filter_map(|setting| {
            let value = config.settings.get(&setting.id);
            let values_count = match setting.field_type.as_str() {
                "range" | "checkbox_group" => value
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter(|v| {
                        // Игнорируем пустые строки в массиве
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
            // Только если параметр обязательный (min_selected > 0 или для select всегда 1)
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
        let debug_msg = log_with_timestamp(
            &format!("Debug: settings_values = {{ {} }}", debug_settings),
            LogLevel::Debug
        );
        logs.push(debug_msg.clone());
        window.emit("build-log", &debug_msg).ok();

        let msg = log_with_timestamp(
            &format!("No values provided for required build parameters: {}. Please fill all required build settings.", missing_required.join(", ")),
            LogLevel::Error
        );
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        return Ok(BuildResult { result: msg, logs, stages, success: false });
    }

    // Создание комбинаций сборки (подробное логирование)
    let mut build_combinations = vec![vec![]];
    for (setting, values) in &settings_values {
        let mut new_combinations = vec![];
        // Если параметр необязательный и массив пустой — используем [None] для декартова произведения
        let is_optional = setting.min_selected.unwrap_or(0) == 0;
        let values_for_comb = if values.is_empty() && is_optional {
            vec![None]
        } else {
            values.iter().map(|v| Some(v.clone())).collect()
        };
        for value_opt in values_for_comb {
            for combo in &build_combinations {
                let mut new_combo = combo.clone();
                if let Some(ref value) = value_opt {
                    new_combo.push((setting.id.clone(), value.clone()));
                }
                new_combinations.push(new_combo);
            }
        }
        build_combinations = new_combinations;
        // Логируем после каждого шага
        let msg = log_with_timestamp(
            &format!(
                "After processing '{}', build_combinations count: {}. Example: {:?}", 
                setting.id, 
                build_combinations.len(), 
                build_combinations.get(0)
            ),
            LogLevel::Debug
        );
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
    }

    // Если build_combinations пустой, логируем причину
    if build_combinations.is_empty() {
        let msg = log_with_timestamp(
            "No build combinations generated. This usually means at least one build parameter has no values. Check settings_values and build_settings.",
            LogLevel::Error
        );
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        return Ok(BuildResult { result: msg, logs, stages, success: false });
    }

    let mut any_build_executed = false;

    // Сборка для каждой комбинации
    for combination in build_combinations {
        any_build_executed = true;
        // Проверка отмены
        {
            let config_guard = BUILD_CONFIG.lock().await;
            if let Some(conf) = &*config_guard {
                if conf.cancelled.unwrap_or(false) {
                    let msg = log_with_timestamp(&format!("Build cancelled for combination {:?}", combination), LogLevel::Info);
                    logs.push(msg.clone());
                    window.emit("build-log", &msg).ok();
                    success = false;
                    return Ok(BuildResult { result: msg, logs, stages, success });
                }
            }
        }

        // Формирование директории комбинации
        let mut combo_dir_name = String::new();
        let mut name_parts = vec![project_name.clone()];
        for (setting_id, value) in &combination {
            combo_dir_name.push_str(&format!("{}_{}_", setting_id, value));
            name_parts.push(format!("{}-{}", setting_id, value));
        }
        let combo_dir = output_dir.join(combo_dir_name.trim_end_matches('_'));
        
        if let Err(e) = fs::create_dir_all(&combo_dir) {
            let msg = log_with_timestamp(&format!("Error creating directory '{}': {}", combo_dir.display(), e), LogLevel::Error);
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            success = false;
            return Ok(BuildResult { result: msg, logs, stages, success });
        }

        // Формирование имен файлов
        let bin_name = format!("{}_FBOOT.bin", name_parts.join("_"));
        let bin_dst = combo_dir.join(&bin_name);
        let log_name = format!("{}_FBOOT.log", name_parts.join("_"));
        let stm32_log_file = combo_dir.join(&log_name);

        // Новый файл для stdout/stderr
        let txt_log_name = format!("{}_FBOOT.txt", name_parts.join("_"));
        let txt_log_file = combo_dir.join(&txt_log_name);

        // Проверка и удаление существующего .bin
        stages.push(format!("Checking and removing existing .bin file for combination {:?}", combination));
        if bin_dst.exists() {
            if let Err(e) = fs::remove_file(&bin_dst) {
                let msg = log_with_timestamp(&format!("Error removing existing file '{}': {}", bin_dst.display(), e), LogLevel::Error);
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
                success = false;
                return Ok(BuildResult { result: msg, logs, stages, success });
            }
        }

        // Генерация build_config.h
        stages.push(format!("Generating build_config.h for combination {:?}", combination));
        let mut build_config_content = String::new();
        build_config_content.push_str("#ifndef BUILD_CONFIG_H_\n#define BUILD_CONFIG_H_\n\n");

        // Создание директории Inc
        if let Some(parent) = build_config_file.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                let msg = log_with_timestamp(&format!("Error creating directory '{}': {}", parent.display(), e), LogLevel::Error);
                logs.push(msg.clone());
                window.emit("build-log", &msg).ok();
                success = false;
                return Ok(BuildResult { result: msg, logs, stages, success });
            }
        }

        // Динамическая генерация define/undef
        for setting in &settings_config.build_settings {
            let id = &setting.id;
            // Clone the value to avoid lifetime issues (fixes previous `value_opt.0` error)
            let value_opt = combination.iter().find(|(s_id, _)| s_id == id).map(|(_, v)| v.clone());

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
                "select" | "checkbox_group" => {
                    if let Some(options) = &setting.options {
                        for opt in options {
                            let is_selected = if let Some(v) = &value_opt {
                                v == &opt.value // Direct String comparison
                            } else {
                                false
                            };
                            
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

        // Запись build_config.h
        if let Err(e) = File::create(&build_config_file).and_then(|mut f| f.write_all(build_config_content.as_bytes())) {
            let msg = log_with_timestamp(&format!("Error writing '{}': {}", build_config_file.display(), e), LogLevel::Error);
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            success = false;
            return Ok(BuildResult { result: msg, logs, stages, success });
        }

        // Запуск STM32CubeIDE
        stages.push(format!("Launching build in STM32CubeIDE for combination {:?}", combination));
        let workspace_path_quoted = quote_path(&workspace_path);
        let project_path_quoted = quote_path(&build_config.project_path);

        // Формирование параметров командной строки для STM32CubeIDE
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
        // Добавляем пользовательские аргументы, если они есть
        if let Some(ref custom_args) = build_config.custom_console_args {
            headless_args.extend(custom_args.split_whitespace().map(|s| s.to_string()));
        }

        // Добавляем логирование команды (выводим строкой, а не массивом)
        let msg = log_with_timestamp(
            &format!(
                "Executing command: {} {}",
                &build_config.cube_ide_exe_path,
                headless_args
                    .iter()
                    .map(|s| {
                        // Добавляем кавычки только если есть пробелы
                        if s.contains(' ') { format!("\"{}\"", s) } else { s.clone() }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            LogLevel::Info
        );
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();

        let mut command = Command::new(&build_config.cube_ide_exe_path);
        command
            .args(&headless_args)
            .kill_on_drop(true)
            .current_dir(&build_config.project_path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        #[cfg(windows)]
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW

        let mut child = command.spawn().map_err(|e| {
            let msg = log_with_timestamp(&format!("Failed to start STM32CubeIDE process: {}", e), LogLevel::Error);
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            tauri::Error::from(anyhow::anyhow!(msg))
        })?;

        // Обработка stdout
        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);
        let tx_stdout = tx.clone();
        let mut stdout_lines = Vec::new();
        let stdout_task = tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(line) = lines.next_line().await {
                if let Some(line) = line {
                    let log = format!("[STDOUT] {}", line.trim());
                    // Сохраняем вектор строк, не отправляем в канал
                    stdout_lines.push(log);
                } else {
                    break;
                }
            }
            Ok::<Vec<String>, std::io::Error>(stdout_lines)
        });

        // Обработка stderr
        let stderr = child.stderr.take().expect("Failed to capture stderr");
        let mut stderr_lines = Vec::new();
        let stderr_task = tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(line) = lines.next_line().await {
                if let Some(line) = line {
                    let log = format!("[STDERR] {}", line.trim());
                    stderr_lines.push(log);
                } else {
                    break;
                }
            }
            Ok::<Vec<String>, std::io::Error>(stderr_lines)
        });

        // Ожидаем завершения процесса
        let status = child.wait().await.map_err(|e| {
            let msg = log_with_timestamp(&format!("Process wait failed: {}", e), LogLevel::Error);
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            tauri::Error::from(anyhow::anyhow!(msg))
        })?;

        // Дожидаемся завершения задач чтения stdout/stderr
        let stdout_logs = stdout_task.await.map_err(|e| {
            let msg = log_with_timestamp(&format!("stdout task failed: {}", e), LogLevel::Error);
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            tauri::Error::from(anyhow::anyhow!(msg))
        })??;
        let stderr_logs = stderr_task.await.map_err(|e| {
            let msg = log_with_timestamp(&format!("stderr task failed: {}", e), LogLevel::Error);
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            tauri::Error::from(anyhow::anyhow!(msg))
        })??;

        // Записываем stdout/stderr в txt_log_file
        if let Ok(mut txt_log_writer) = File::create(&txt_log_file) {
            for log in &stdout_logs {
                writeln!(txt_log_writer, "{}", log).ok();
            }
            for log in &stderr_logs {
                writeln!(txt_log_writer, "{}", log).ok();
            }
            txt_log_writer.flush().ok();
        } else {
            let msg = log_with_timestamp(
                &format!("Failed to create log file '{}'", txt_log_file.display()),
                LogLevel::Warning
            );
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
        }

        // Проверяем статус процесса
        let exit_code = status.code().unwrap_or(-1);
        let status_msg = log_with_timestamp(
            &format!("Build process exited with code: {}", exit_code),
            if exit_code == 0 { LogLevel::Info } else { LogLevel::Error }
        );
        logs.push(status_msg.clone());
        window.emit("build-log", &status_msg).ok();

        if exit_code != 0 {
            success = false;
            return Ok(BuildResult {
                result: format!("Build failed with exit code: {}", exit_code),
                logs,
                stages,
                success
            });
        }

        // Добавляем проверку результатов сборки
        time::sleep(Duration::from_secs(2)).await;

        // Проверка содержимого директории сборки
        stages.push(format!("Checking build directory contents for combination {:?}", combination));
        let build_dir_name = build_config.config_name.as_deref().unwrap_or("Debug");
        let build_dir = project_path.join(build_dir_name);
        let expected_bin_file = build_dir.join(format!("{}.bin", project_name.to_lowercase()));
        if !build_dir.exists() || !expected_bin_file.exists() {
            let msg = log_with_timestamp(&format!("Error: Output file '{}.bin' not found in '{}'", project_name.to_lowercase(), build_dir.display()), LogLevel::Error);
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            success = false;
            return Ok(BuildResult { result: msg, logs, stages, success });
        }

        // Проверяем размер файла
        if let Ok(metadata) = fs::metadata(&expected_bin_file) {
            let msg = log_with_timestamp(
                &format!("Output file size: {} bytes", metadata.len()),
                LogLevel::Info
            );
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
        } else {
            let msg = log_with_timestamp(
                &format!("Failed to get output file metadata: {}", expected_bin_file.display()),
                LogLevel::Error
            );
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            success = false;
            return Ok(BuildResult { result: msg, logs, stages, success });
        }

        // Переименование bin файла
        stages.push(format!("Renaming output file for combination {:?}", combination));
        if let Err(e) = fs::rename(&expected_bin_file, &bin_dst) {
            let msg = log_with_timestamp(&format!("Error moving '{}': {}", expected_bin_file.display(), e), LogLevel::Error);
            logs.push(msg.clone());
            window.emit("build-log", &msg).ok();
            success = false;
            return Ok(BuildResult { result: msg, logs, stages, success });
        }
    }

    if !any_build_executed {
        let msg = log_with_timestamp("No build combinations were executed. Check your build settings.", LogLevel::Error);
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        return Ok(BuildResult { result: msg, logs, stages, success: false });
    }

    // Запись логов
    stages.push("Writing logs".to_string());
    if let Err(e) = File::create(&log_file_path).and_then (|mut f| {
        for log in &logs {
            writeln!(f, "{}", log)?;
        }
        Ok(())
    }) {
        let msg = log_with_timestamp(&format!("Failed to write logs: {}", e), LogLevel::Error);
        logs.push(msg.clone());
        window.emit("build-log", &msg).ok();
        success = false;
        return Ok(BuildResult { result: msg, logs, stages, success });
    }

    // Финализация результата сборки
    stages.push("Build process completed".to_string());
    let last_result = if success {
        log_with_timestamp("Build process completed successfully", LogLevel::Info)
    } else {
        log_with_timestamp("Build process completed with errors", LogLevel::Error)
    };
    logs.push(last_result.clone());
    window.emit("build-log", &last_result).ok();

    Ok(BuildResult { result: last_result, logs, stages, success })
}

#[command]
pub fn load_build_settings_schema() -> Result<BuildSettingsConfig, String> {
    let schema_path = "build_settings.json";
    
    if !Path::new(schema_path).exists() {
        fs::write(schema_path, DEFAULT_BUILD_SETTINGS)
            .map_err(|e| format!("Error creating settings file: {}", e))?;
    }
    
    let content = fs::read_to_string(schema_path)
        .map_err(|e| format!("Error reading build settings schema: {}", e))?;
    
    serde_json::from_str(&content)
        .map_err(|e| format!("Error parsing build settings schema: {}. Line: {}, Column: {}", e, e.line(), e.column()))
}