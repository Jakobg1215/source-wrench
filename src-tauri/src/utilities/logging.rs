use std::{
    fmt::{self, Display, Formatter},
    ptr::addr_of,
};

use serde::Serialize;
use tauri::{Manager, WebviewWindow};

#[derive(Serialize, Clone)]
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
    println!("[{}] {}", level, log_message);
    if let Some(window) = unsafe { &*addr_of!(LOGGER) } {
        let _ = window.emit("source-wrench-log", LogEvent::new(level, log_message));
    }
}

#[derive(Serialize, Clone)]
struct LogEvent {
    level: LogLevel,
    message: String,
}

impl LogEvent {
    fn new(level: LogLevel, message: String) -> Self {
        Self { level, message }
    }
}

pub static mut LOGGER: Option<WebviewWindow> = None;
