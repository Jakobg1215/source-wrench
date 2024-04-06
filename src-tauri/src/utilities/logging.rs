pub enum LogLevel {
    Log,
    Info,
    Verbose,
    Debug,
    Warn,
    Error,
}

pub fn log<T: Into<String>>(message: T, level: LogLevel) {
    let log_type = match level {
        LogLevel::Log => "LOG",
        LogLevel::Info => "INFO",
        LogLevel::Verbose => "VERBOSE",
        LogLevel::Debug => "DEBUG",
        LogLevel::Warn => "WARN",
        LogLevel::Error => "ERROR",
    };
    println!("[{}] {}", log_type, message.into())
}
