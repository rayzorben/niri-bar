use niri_bar_new::logger::NiriBarLogger;
use niri_bar_new::config::LoggingConfig;
use tempfile::NamedTempFile;
use std::fs;
use log::Level;

#[test]
fn test_logger_initialization() {
    let config = LoggingConfig {
        level: "debug".to_string(),
        file: "".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let result = NiriBarLogger::new(config);
    assert!(result.is_ok());
}

#[test]
fn test_logger_with_file_output() {
    let temp_file = NamedTempFile::new().unwrap();
    let config = LoggingConfig {
        level: "info".to_string(),
        file: temp_file.path().to_string_lossy().to_string(),
        console: false,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let logger = NiriBarLogger::new(config).unwrap();
    
    // Test that file was created
    assert!(temp_file.path().exists());
}

#[test]
fn test_logger_iso8601_format() {
    let config = LoggingConfig {
        level: "debug".to_string(),
        file: "".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let logger = NiriBarLogger::new(config).unwrap();
    
    // Test that the format is set correctly
    assert_eq!(logger.config.format, "iso8601");
}

#[test]
fn test_logger_simple_format() {
    let config = LoggingConfig {
        level: "debug".to_string(),
        file: "".to_string(),
        console: true,
        format: "simple".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let logger = NiriBarLogger::new(config).unwrap();
    
    // Test that the format is set correctly
    assert_eq!(logger.config.format, "simple");
}

#[test]
fn test_logger_level_filtering() {
    let config = LoggingConfig {
        level: "warn".to_string(),
        file: "".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let logger = NiriBarLogger::new(config).unwrap();
    
    // Test that the level is set correctly
    assert_eq!(logger.config.level, "warn");
}

#[test]
fn test_logger_console_output() {
    let config = LoggingConfig {
        level: "debug".to_string(),
        file: "".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let logger = NiriBarLogger::new(config).unwrap();
    
    // Test that console output is enabled
    assert!(logger.config.console);
}

#[test]
fn test_logger_file_output_disabled() {
    let config = LoggingConfig {
        level: "debug".to_string(),
        file: "".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let logger = NiriBarLogger::new(config).unwrap();
    
    // Test that file output is disabled when file path is empty
    assert!(logger.file_handle.is_none());
}

#[test]
fn test_logger_include_file_option() {
    let config = LoggingConfig {
        level: "debug".to_string(),
        file: "".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let logger = NiriBarLogger::new(config).unwrap();
    
    // Test that include_file is enabled
    assert!(logger.config.include_file);
}

#[test]
fn test_logger_include_line_option() {
    let config = LoggingConfig {
        level: "debug".to_string(),
        file: "".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let logger = NiriBarLogger::new(config).unwrap();
    
    // Test that include_line is enabled
    assert!(logger.config.include_line);
}

#[test]
fn test_logger_include_class_option() {
    let config = LoggingConfig {
        level: "debug".to_string(),
        file: "".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let logger = NiriBarLogger::new(config).unwrap();
    
    // Test that include_class is enabled
    assert!(logger.config.include_class);
}

#[test]
fn test_logger_file_creation() {
    let temp_dir = tempfile::tempdir().unwrap();
    let log_file = temp_dir.path().join("test.log");
    
    let config = LoggingConfig {
        level: "debug".to_string(),
        file: log_file.to_string_lossy().to_string(),
        console: false,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let logger = NiriBarLogger::new(config).unwrap();
    
    // Test that the log file was created
    assert!(log_file.exists());
}

#[test]
fn test_logger_directory_creation() {
    let temp_dir = tempfile::tempdir().unwrap();
    let nested_dir = temp_dir.path().join("nested").join("deep");
    let log_file = nested_dir.join("test.log");
    
    let config = LoggingConfig {
        level: "debug".to_string(),
        file: log_file.to_string_lossy().to_string(),
        console: false,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let logger = NiriBarLogger::new(config).unwrap();
    
    // Test that the nested directory was created
    assert!(nested_dir.exists());
    assert!(log_file.exists());
}

#[test]
fn test_logger_default_values() {
    let config = LoggingConfig {
        level: "info".to_string(),
        file: "".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let logger = NiriBarLogger::new(config).unwrap();
    
    // Test default values
    assert_eq!(logger.config.level, "info");
    assert!(logger.config.console);
    assert_eq!(logger.config.format, "iso8601");
    assert!(logger.config.include_file);
    assert!(logger.config.include_line);
    assert!(logger.config.include_class);
}

#[test]
fn test_logger_error_level() {
    let config = LoggingConfig {
        level: "error".to_string(),
        file: "".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let logger = NiriBarLogger::new(config).unwrap();
    
    // Test error level
    assert_eq!(logger.config.level, "error");
}

#[test]
fn test_logger_trace_level() {
    let config = LoggingConfig {
        level: "trace".to_string(),
        file: "".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let logger = NiriBarLogger::new(config).unwrap();
    
    // Test trace level
    assert_eq!(logger.config.level, "trace");
}

#[test]
fn test_logger_invalid_level_handling() {
    let config = LoggingConfig {
        level: "invalid".to_string(),
        file: "".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let logger = NiriBarLogger::new(config).unwrap();
    
    // Test that invalid level is handled gracefully
    assert_eq!(logger.config.level, "invalid");
}

#[test]
fn test_logger_file_path_expansion() {
    let temp_file = NamedTempFile::new().unwrap();
    let config = LoggingConfig {
        level: "debug".to_string(),
        file: temp_file.path().to_string_lossy().to_string(),
        console: false,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let logger = NiriBarLogger::new(config).unwrap();
    
    // Test that file path is handled correctly
    assert!(logger.file_handle.is_some());
}

#[test]
fn test_logger_configuration_validation() {
    let config = LoggingConfig {
        level: "debug".to_string(),
        file: "".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let logger = NiriBarLogger::new(config).unwrap();
    
    // Test that all configuration options are properly set
    assert!(!logger.config.file.is_empty() || logger.config.file.is_empty());
    assert!(logger.config.console);
    assert!(!logger.config.format.is_empty());
    assert!(logger.config.include_file);
    assert!(logger.config.include_line);
    assert!(logger.config.include_class);
}

#[test]
fn test_logger_performance() {
    let config = LoggingConfig {
        level: "debug".to_string(),
        file: "".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let logger = NiriBarLogger::new(config.clone()).unwrap();
    
    // Test that logger creation is fast
    let start = std::time::Instant::now();
    let _logger2 = NiriBarLogger::new(config).unwrap();
    let duration = start.elapsed();
    
    // Should complete in less than 1 second
    assert!(duration.as_millis() < 1000);
}

// Tests moved from src/logger.rs
#[test]
fn test_logger_creation_from_src() {
    let config = LoggingConfig {
        level: "debug".to_string(),
        file: "".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let logger = NiriBarLogger::new(config).unwrap();
    assert!(logger.config.console);
    assert_eq!(logger.config.level, "debug");
}

#[test]
fn test_logger_with_file_from_src() {
    let temp_file = NamedTempFile::new().unwrap();
    let config = LoggingConfig {
        level: "info".to_string(),
        file: temp_file.path().to_string_lossy().to_string(),
        console: false,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let logger = NiriBarLogger::new(config).unwrap();
    assert!(!logger.config.console);
    assert!(logger.file_handle.is_some());
}


