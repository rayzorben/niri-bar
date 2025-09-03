use crate::config::LoggingConfig;
use chrono::Utc;
use log::{Level, LevelFilter, Log, Metadata, Record};
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Custom logger that honors the niri-bar.yaml logging configuration
pub struct NiriBarLogger {
    pub config: LoggingConfig,
    pub file_handle: Option<Arc<Mutex<File>>>,
}

impl NiriBarLogger {
    /// Create a new logger with the given configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use niri_bar::logger::NiriBarLogger;
    /// use niri_bar::config::LoggingConfig;
    ///
    /// let config = LoggingConfig {
    ///     level: "info".to_string(),
    ///     file: "/tmp/test.log".to_string(),
    ///     console: true,
    ///     format: "iso8601".to_string(),
    ///     include_file: true,
    ///     include_line: true,
    ///     include_class: true,
    /// };
    ///
    /// let logger = NiriBarLogger::new(config).unwrap();
    /// // Logger is ready to use
    /// ```
    pub fn new(config: LoggingConfig) -> Result<Self, io::Error> {
        let file_handle = if !config.file.is_empty() {
            let expanded_path = shellexpand::tilde(&config.file).to_string();
            let path = Path::new(&expanded_path);

            // Create directory if it doesn't exist
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let file = OpenOptions::new().create(true).append(true).open(path)?;

            Some(Arc::new(Mutex::new(file)))
        } else {
            None
        };

        Ok(Self {
            config,
            file_handle,
        })
    }

    /// Initialize the global logger with the given configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use niri_bar::logger::NiriBarLogger;
    /// use niri_bar::config::LoggingConfig;
    ///
    /// let config = LoggingConfig {
    ///     level: "debug".to_string(),
    ///     file: "/tmp/test.log".to_string(),
    ///     console: true,
    ///     format: "iso8601".to_string(),
    ///     include_file: true,
    ///     include_line: true,
    ///     include_class: true,
    /// };
    ///
    /// // Initialize the global logger
    /// NiriBarLogger::init(config).unwrap();
    ///
    /// // Now you can use log macros
    /// log::info!("Logger initialized!");
    /// ```
    pub fn init(config: LoggingConfig) -> Result<(), Box<dyn std::error::Error>> {
        let level_filter = match config.level.to_lowercase().as_str() {
            "trace" => LevelFilter::Trace,
            "debug" => LevelFilter::Debug,
            "info" => LevelFilter::Info,
            "warn" => LevelFilter::Warn,
            "error" => LevelFilter::Error,
            _ => LevelFilter::Info,
        };

        let logger = Self::new(config)?;
        log::set_boxed_logger(Box::new(logger))?;
        log::set_max_level(level_filter);

        Ok(())
    }

    /// Format a log message according to the configuration
    fn format_message(&self, record: &Record) -> String {
        let timestamp = Utc::now();
        let timestamp_str = if self.config.format.to_lowercase() == "iso8601" {
            timestamp.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
        } else {
            timestamp.format("%Y-%m-%d %H:%M:%S").to_string()
        };

        let level_str = record.level().to_string().to_uppercase();

        let mut parts = vec![format!("[{}]", timestamp_str), format!("[{}]", level_str)];

        // Add class information if enabled
        if self.config.include_class {
            let target = record.target();
            if target != "niri_bar" {
                parts.push(format!("[{}]", target));
            }
        }

        // Add file and line information if enabled
        if self.config.include_file || self.config.include_line {
            let mut location_parts = Vec::new();

            if self.config.include_file
                && let Some(file) = record.file()
            {
                location_parts.push(file.to_string());
            }

            if self.config.include_line
                && let Some(line) = record.line()
            {
                location_parts.push(line.to_string());
            }

            if !location_parts.is_empty() {
                parts.push(format!("[{}]", location_parts.join(":")));
            }
        }

        // Add the actual message
        parts.push(record.args().to_string());

        parts.join(" ")
    }

    /// Write a log message to the configured outputs
    fn write_log(&self, message: &str) -> Result<(), io::Error> {
        let message_with_newline = format!("{}\n", message);

        // Write to console if enabled
        if self.config.console {
            io::stdout().write_all(message_with_newline.as_bytes())?;
        }

        // Write to file if configured
        if let Some(file_handle) = &self.file_handle
            && let Ok(mut file) = file_handle.lock()
        {
            file.write_all(message_with_newline.as_bytes())?;
            file.flush()?;
        }

        Ok(())
    }
}

impl Log for NiriBarLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let level_filter = match self.config.level.to_lowercase().as_str() {
            "trace" => LevelFilter::Trace,
            "debug" => LevelFilter::Debug,
            "info" => LevelFilter::Info,
            "warn" => LevelFilter::Warn,
            "error" => LevelFilter::Error,
            _ => LevelFilter::Info,
        };

        metadata.level() <= level_filter
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let message = self.format_message(record);

            // Use stderr for errors, stdout for everything else
            if record.level() == Level::Error {
                let _ = io::stderr().write_all(format!("{}\n", message).as_bytes());
            } else {
                let _ = self.write_log(&message);
            }
        }
    }

    fn flush(&self) {
        // Flush stdout
        let _ = io::stdout().flush();

        // Flush stderr
        let _ = io::stderr().flush();

        // Flush file if open
        if let Some(file_handle) = &self.file_handle
            && let Ok(mut file) = file_handle.lock()
        {
            let _ = file.flush();
        }
    }
}

/// Convenience macros for logging with class context
#[macro_export]
macro_rules! log_debug {
    ($class:expr, $($arg:tt)*) => {
        log::debug!(target: $class, $($arg)*);
    };
}

#[macro_export]
macro_rules! log_info {
    ($class:expr, $($arg:tt)*) => {
        log::info!(target: $class, $($arg)*);
    };
}

#[macro_export]
macro_rules! log_warn {
    ($class:expr, $($arg:tt)*) => {
        log::warn!(target: $class, $($arg)*);
    };
}

#[macro_export]
macro_rules! log_error {
    ($class:expr, $($arg:tt)*) => {
        log::error!(target: $class, $($arg)*);
    };
}
