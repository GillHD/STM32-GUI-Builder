use chrono::Local;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs;
use std::path::Path;
use tauri::{Error, command};

// Уровни логирования
#[derive(Debug)]
pub enum LogLevel {
    Debug,
    Warning,
    Info,
    Error,
}

#[command]
pub fn validate_path(path: String) -> Result<(), String> {
    let path = Path::new(&path);
    if path.exists() && path.is_dir() {
        Ok(())
    } else {
        Err(format!("Path '{}' does not exist or is not a directory", path.display()))
    }
}
// Проверка, нужно ли логировать сообщение на основе уровня
fn should_log(_level: &LogLevel) -> bool {
    true // Логируем все уровни для отладки
}

pub fn log_with_timestamp(msg: &str, level: LogLevel) -> String {
    if should_log(&level) {
        format!("[{}] [{}] {}", Local::now().format("%Y-%m-%d %H:%M:%S"), format!("{:?}", level), msg)
    } else {
        String::new()
    }
}

// pub fn quote_path(path: &str) -> String {
//     format!("\"{}\"", path)
// }

pub fn get_project_name(project_path: &Path) -> Result<String, Error> {
    let project_file = project_path.join(".project");
    if !project_file.exists() {
        return Err(Error::from(anyhow::anyhow!("Файл .project не найден")));
    }
    let xml_content = fs::read_to_string(&project_file)
        .map_err(|e| Error::from(anyhow::anyhow!(e.to_string())))?;
    let mut reader = Reader::from_str(&xml_content);
    reader.trim_text(true);
    let mut buf = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) if e.name() == b"name" => {
                if let Ok(Event::Text(text)) = reader.read_event(&mut buf) {
                    return Ok(text.unescape_and_decode(&reader)
                        .map_err(|e| Error::from(anyhow::anyhow!(e.to_string())))?);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(Error::from(anyhow::anyhow!(e.to_string()))),
            _ => (),
        }
        buf.clear();
    }
    Err(Error::from(anyhow::anyhow!("Имя проекта не найдено в .project файле")))
}

pub fn get_cproject_configurations(project_path: &Path) -> Result<Vec<String>, Error> {
    let cproject_file = project_path.join(".cproject");
    if !cproject_file.exists() {
        return Err(Error::from(anyhow::anyhow!("Файл .cproject не найден")));
    }
    let xml_content = fs::read_to_string(&cproject_file)
        .map_err(|e| Error::from(anyhow::anyhow!(e.to_string())))?;
    let mut reader = Reader::from_str(&xml_content);
    reader.trim_text(true);
    let mut buf = Vec::new();
    let mut configs = Vec::new();
    let mut _in_configuration = false;
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) if e.name() == b"configuration" => {
                _in_configuration = true;
                for attr in e.attributes() {
                    if let Ok(attr) = attr {
                        if attr.key == b"name" {
                            if let Ok(value) = attr.unescape_and_decode_value(&reader) {
                                configs.push(value);
                            }
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) if e.name() == b"configuration" => {
                _in_configuration = false;
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(Error::from(anyhow::anyhow!(format!("Ошибка парсинга .cproject: {}", e)))),
            _ => (),
        }
        buf.clear();
    }
    Ok(configs)
}

#[command]
pub async fn get_project_configurations(project_path: String) -> Result<Vec<String>, String> {
    let project_path = Path::new(&project_path);
    match get_cproject_configurations(project_path) {
        Ok(configs) => Ok(configs),
        Err(e) => Err(format!("Failed to get project configurations: {}", e))
    }
}

#[command]
pub async fn get_project_name_from_path(project_path: String) -> Result<String, String> {
    let project_path = Path::new(&project_path);
    match get_project_name(project_path) {
        Ok(name) => Ok(name),
        Err(e) => Err(format!("Failed to get project name: {}", e))
    }
}