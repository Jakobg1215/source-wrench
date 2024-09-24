use std::{
    fmt::{self, Display, Formatter},
    sync::OnceLock,
};

use serde::Serialize;
use tauri::{Manager, WebviewWindow};

#[derive(Clone, Serialize)]
pub enum LogLevel {
    Log,
    Info,
    Verbose,
    Debug,
    Warn,
    Error,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let output = match self {
            LogLevel::Log => "LOG",
            LogLevel::Info => "INFO",
            LogLevel::Verbose => "VERBOSE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        };
        write!(f, "{}", output)
    }
}

pub fn log<T: Into<String>>(message: T, level: LogLevel) {
    let log_message = message.into();
    if tauri::dev() {
        println!("[{}] {}", level, log_message);
    }
    if let Some(window) = LOGGER.get() {
        let _ = window.emit("source-wrench-log", LogEvent::new(level, log_message));
    }
}

#[derive(Clone, Serialize)]
struct LogEvent {
    level: LogLevel,
    message: String,
}

impl LogEvent {
    fn new(level: LogLevel, message: String) -> Self {
        Self { level, message }
    }
}

pub static LOGGER: OnceLock<WebviewWindow> = OnceLock::new();
