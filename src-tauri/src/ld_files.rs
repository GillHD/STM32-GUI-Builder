use std::fs;
use std::path::Path;
use tauri::command;

#[command]
pub fn get_ld_files(#[allow(non_snake_case)] projectPath: String) -> Result<Vec<String>, String> {
    if projectPath.trim().is_empty() {
        return Err("Путь к проекту не может быть пустым".into());
    }
    let dir = Path::new(&projectPath);
    if !dir.exists() || !dir.is_dir() {
        return Err(format!(
            "Директория '{}' не найдена или не является папкой",
            projectPath
        ));
    }
    let mut ld_files = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| format!("Ошибка чтения '{}': {}", projectPath, e))? {
        let entry = entry.map_err(|e| format!("Ошибка обработки записи в '{}': {}", projectPath, e))?;
        let path = entry.path();
        if path.is_file() && matches!(path.extension().and_then(|e| e.to_str()), Some("ld")) {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                ld_files.push(name.to_string());
            }
        }
    }
    if ld_files.is_empty() {
        return Err(format!("В '{}' нет .ld файлов", projectPath));
    }
    Ok(ld_files)
}