use std::sync::{LazyLock, Mutex};

pub enum LogLevel {
    #[allow(dead_code)]
    Log,
    Info,
    Verbose,
    Debug,
    Warn,
    Error,
}

pub fn log<T: Into<String>>(message: T, level: LogLevel) {
    let log_message = message.into();
    let mut logger = LOGGER.lock().unwrap();

    let level_string = match &level {
        LogLevel::Log => "LOG",
        LogLevel::Info => "INFO",
        LogLevel::Verbose => "VERBOSE",
        LogLevel::Debug => "DEBUG",
        LogLevel::Warn => "WARN",
        LogLevel::Error => "ERROR",
    };

    logger.logs.push((format!("[{}] {}", level_string, log_message), level));
}

pub struct LoggingData {
    pub allow_verbose: bool,
    pub allow_debug: bool,
    pub logs: Vec<(String, LogLevel)>,
}

pub static LOGGER: LazyLock<Mutex<LoggingData>> = LazyLock::new(|| {
    Mutex::new(LoggingData {
        allow_verbose: true,
        allow_debug: true,
        logs: Vec::new(),
    })
});
