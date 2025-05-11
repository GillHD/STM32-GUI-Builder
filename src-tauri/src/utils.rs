use chrono::Local;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs;
use std::path::Path;

pub fn log_with_timestamp(msg: &str) -> String {
    format!("[{}] {}", Local::now().format("%Y-%m-%d %H:%M:%S"), msg)
}

pub fn quote_path(path: &str) -> String {
    format!("\"{}\"", path)
}

pub fn get_project_name(project_path: &Path) -> Result<String, String> {
    let project_file = project_path.join(".project");
    if !project_file.exists() {
        return Err("Файл .project не найден".to_string());
    }
    let xml_content = fs::read_to_string(&project_file).map_err(|e| e.to_string())?;
    let mut reader = Reader::from_str(&xml_content);
    reader.trim_text(true);
    let mut buf = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) if e.name() == b"name" => {
                if let Ok(Event::Text(text)) = reader.read_event(&mut buf) {
                    return Ok(text.unescape_and_decode(&reader).map_err(|e| e.to_string())?);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(e.to_string()),
            _ => (),
        }
        buf.clear();
    }
    Err("Имя проекта не найдено в .project файле".to_string())
}

pub fn get_cproject_configurations(project_path: &Path) -> Result<Vec<String>, String> {
    let cproject_file = project_path.join(".cproject");
    if !cproject_file.exists() {
        return Err("Файл .cproject не найден".to_string());
    }
    let xml_content = fs::read_to_string(&cproject_file).map_err(|e| e.to_string())?;
    let mut reader = Reader::from_str(&xml_content);
    reader.trim_text(true);
    let mut buf = Vec::new();
    let mut configs = Vec::new();
    let mut _in_configuration = false; // Renamed to `_in_configuration` to suppress unused variable warning
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) if e.name() == b"configuration" => {
                _in_configuration = true; // Updated variable name
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
                _in_configuration = false; // Updated variable name
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("Ошибка парсинга .cproject: {}", e)),
            _ => (),
        }
        buf.clear();
    }
    Ok(configs)
}
