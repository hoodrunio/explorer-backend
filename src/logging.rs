use chrono::Utc;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::sync::Mutex;
use lazy_static::lazy_static;

// Simple log levels
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
    FATAL,
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::DEBUG => "DEBUG",
            LogLevel::INFO => "INFO",
            LogLevel::WARN => "WARN",
            LogLevel::ERROR => "ERROR",
            LogLevel::FATAL => "FATAL",
        }
    }

    fn terminal_color(&self) -> &'static str {
        match self {
            LogLevel::DEBUG => "\x1b[36m",  // Cyan
            LogLevel::INFO => "\x1b[32m",   // Green
            LogLevel::WARN => "\x1b[33m",   // Yellow
            LogLevel::ERROR => "\x1b[31m",  // Red
            LogLevel::FATAL => "\x1b[35m",  // Magenta
        }
    }
}

// Simple log categories
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum LogCategory {
    Database,
    API,
    Transaction,
    Socket,
}

impl LogCategory {
    fn as_str(&self) -> &'static str {
        match self {
            LogCategory::Database => "database",
            LogCategory::API => "api",
            LogCategory::Transaction => "transaction",
            LogCategory::Socket => "socket",
        }
    }
}

pub struct FileLogger {
    log_dir: String,
}

impl FileLogger {
    pub fn new() -> Self {
        let log_dir = "logs";
        fs::create_dir_all(log_dir).expect("Failed to create log directory");
        
        FileLogger {
            log_dir: log_dir.to_string(),
        }
    }

    pub fn log(&self, category: LogCategory, level: LogLevel, message: &str) {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC");
        let log_entry = format!("[{}] {} [{}] {}\n", timestamp, level.as_str(), category.as_str(), message);

        // Write to category-specific log file
        let category_path = format!("{}/{}.log", self.log_dir, category.as_str());
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&category_path)
        {
            let _ = file.write_all(log_entry.as_bytes());
        }

        // Write to combined log
        let combined_path = format!("{}/combined.log", self.log_dir);
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&combined_path)
        {
            let _ = file.write_all(log_entry.as_bytes());
        }

        // Write errors and fatal errors to error.log
        if level == LogLevel::ERROR || level == LogLevel::FATAL {
            let error_path = format!("{}/error.log", self.log_dir);
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&error_path)
            {
                let _ = file.write_all(log_entry.as_bytes());
            }
        }

        // Print to terminal with color
        let color = level.terminal_color();
        let reset = "\x1b[0m";
        println!("{}{}{}", color, log_entry.trim_end(), reset);
    }
}

// Create a global logger instance
lazy_static! {
    static ref LOGGER: Mutex<FileLogger> = Mutex::new(FileLogger::new());
}

// Convenience functions for logging
pub fn log_db(level: LogLevel, message: &str) {
    if let Ok(logger) = LOGGER.lock() {
        logger.log(LogCategory::Database, level, message);
    }
}

pub fn log_api(level: LogLevel, message: &str) {
    if let Ok(logger) = LOGGER.lock() {
        logger.log(LogCategory::API, level, message);
    }
}

pub fn log_tx(level: LogLevel, message: &str) {
    if let Ok(logger) = LOGGER.lock() {
        logger.log(LogCategory::Transaction, level, message);
    }
}

pub fn log_socket(level: LogLevel, message: &str) {
    if let Ok(logger) = LOGGER.lock() {
        logger.log(LogCategory::Socket, level, message);
    }
}
