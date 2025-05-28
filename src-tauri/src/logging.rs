use tauri::{Window, Emitter};
use crate::utils::LogLevel;
use chrono::Local;

pub struct Logger<'a> {
    window: &'a Window,
    logs: Vec<String>,
}

impl<'a> Logger<'a> {
    pub fn new(window: &'a Window) -> Self {
        Logger {
            window,
            logs: Vec::new(),
        }
    }

    pub fn log(&mut self, message: &str, level: LogLevel) -> String {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        // Формируем строку лога только здесь, не допускаем вложенных [DEBUG] и т.п. в message
        let log_message = format!("[{}] [{:?}] {}", timestamp, level, message);

        self.logs.push(log_message.clone());
        self.window.emit("build-log", &log_message).ok();
        log_message
    }

    pub fn info(&mut self, message: &str) -> String {
        self.log(message, LogLevel::Info)
    }

    pub fn error(&mut self, message: &str) -> String {
        self.log(message, LogLevel::Error)
    }

    pub fn debug(&mut self, message: &str) -> String {
        self.log(message, LogLevel::Debug)
    }

    pub fn warning(&mut self, message: &str) -> String {
        self.log(message, LogLevel::Warning)
    }

    pub fn get_logs(&self) -> &Vec<String> {
        &self.logs
    }
}
