use chrono::Local;
use quick_xml::reader::Reader;
use quick_xml::events::Event;
use std::fs;
use std::path::Path;
use tauri::{Error, command};
use quick_xml::name::QName;
// use tauri::Window;
// use crate::utils::{log_with_timestamp, LogLevel};

pub fn validate_project_file(project_path: &Path) -> Result<(), tauri::Error> {
    let project_file = project_path.join(".project");
    let content = fs::read_to_string(&project_file)
        .map_err(|e| tauri::Error::from(anyhow::anyhow!("Error reading '{}': {}", project_file.display(), e)))?;
    if !content.contains("<projectDescription>") {
        return Err(tauri::Error::from(anyhow::anyhow!("File '{}' is not a valid .project file", project_file.display())));
    }
    Ok(())
}

pub fn validate_cproject_file(project_path: &Path) -> Result<(), tauri::Error> {
    let cproject_file = project_path.join(".cproject");
    let content = fs::read_to_string(&cproject_file)
        .map_err(|e| tauri::Error::from(anyhow::anyhow!("Error reading '{}': {}", cproject_file.display(), e)))?;
    if !content.contains("<cproject") {
        return Err(tauri::Error::from(anyhow::anyhow!("File '{}' is not a valid .cproject file", cproject_file.display())));
    }
    Ok(())
}

// Log levels
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
// Check if message should be logged based on level
fn should_log(_level: &LogLevel) -> bool {
    true // Log all levels for debugging
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
        return Err(Error::from(anyhow::anyhow!(".project file not found")));
    }
    let xml_content = fs::read_to_string(&project_file)
        .map_err(|e| Error::from(anyhow::anyhow!(e.to_string())))?;
    let mut reader = Reader::from_str(&xml_content);
    reader.trim_text(true);

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"name" => {
                if let Ok(Event::Text(text)) = reader.read_event() {
                    return Ok(text.unescape().map_err(|e| Error::from(anyhow::anyhow!(e.to_string())))?.into_owned());
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(Error::from(anyhow::anyhow!(e.to_string()))),
            _ => (),
        }
    }
    Err(Error::from(anyhow::anyhow!("Project name not found in .project file")))
}

pub fn get_cproject_configurations(project_path: &Path) -> Result<Vec<String>, Error> {
    let cproject_file = project_path.join(".cproject");
    if !cproject_file.exists() {
        return Err(Error::from(anyhow::anyhow!(".cproject file not found")));
    }
    let xml_content = fs::read_to_string(&cproject_file)
        .map_err(|e| Error::from(anyhow::anyhow!(e.to_string())))?;
    let mut reader = Reader::from_str(&xml_content);
    reader.trim_text(true);
    let mut configs = Vec::new();
    let mut _in_configuration = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"configuration" => {
                _in_configuration = true;
                for attr in e.attributes() {
                    if let Ok(attr) = attr {
                        if attr.key.as_ref() == b"name" {
                            if let Ok(value) = attr.unescape_value() {
                                configs.push(value.into_owned());
                            }
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) if e.name() == QName(b"configuration") => {
                _in_configuration = false;
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(Error::from(anyhow::anyhow!(format!("Error parsing .cproject: {}", e)))),
            _ => (),
        }
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