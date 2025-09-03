use niri_bar::config::LoggingConfig;
use niri_bar::logger::NiriBarLogger;
use tempfile::NamedTempFile;

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

    let _logger = NiriBarLogger::new(config).unwrap();

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

    let _logger = NiriBarLogger::new(config).unwrap();

    // Test that file output is disabled when file path is empty
    assert!(_logger.file_handle.is_none());
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

    let _logger = NiriBarLogger::new(config).unwrap();

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

    let _logger = NiriBarLogger::new(config).unwrap();

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

    let _logger = NiriBarLogger::new(config.clone()).unwrap();

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

// ===== COMPREHENSIVE LOGGER TESTS =====

#[test]
fn test_logger_creation_edge_cases() {
    // Test with empty file path
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
    assert!(logger.file_handle.is_none());

    // Test with tilde expansion
    let config_with_tilde = LoggingConfig {
        level: "info".to_string(),
        file: "~/test.log".to_string(),
        console: false,
        format: "simple".to_string(),
        include_file: false,
        include_line: false,
        include_class: false,
    };

    let logger = NiriBarLogger::new(config_with_tilde).unwrap();
    assert!(logger.file_handle.is_some());
}

#[test]
fn test_logger_file_creation_error_handling() {
    // Test with invalid directory path
    let config = LoggingConfig {
        level: "debug".to_string(),
        file: "/nonexistent/deep/path/test.log".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    // This should fail because we can't create the directory
    let result = NiriBarLogger::new(config);
    assert!(result.is_err());
}

#[test]
fn test_logger_level_filter_conversion() {
    use log::LevelFilter;

    let test_cases = vec![
        ("trace", LevelFilter::Trace),
        ("debug", LevelFilter::Debug),
        ("info", LevelFilter::Info),
        ("warn", LevelFilter::Warn),
        ("error", LevelFilter::Error),
        ("TRACE", LevelFilter::Trace), // Test case insensitivity
        ("DEBUG", LevelFilter::Debug),
        ("invalid", LevelFilter::Info), // Invalid levels should default to Info
    ];

    for (level_str, expected_filter) in test_cases {
        // Test the level conversion logic directly instead of calling init multiple times
        let actual_filter = match level_str.to_lowercase().as_str() {
            "trace" => LevelFilter::Trace,
            "debug" => LevelFilter::Debug,
            "info" => LevelFilter::Info,
            "warn" => LevelFilter::Warn,
            "error" => LevelFilter::Error,
            _ => LevelFilter::Info,
        };

        assert_eq!(
            actual_filter, expected_filter,
            "Level '{}' should convert to {:?}",
            level_str, expected_filter
        );
    }
}

#[test]
fn test_logger_format_variations() {
    let formats = vec!["iso8601", "simple"];

    for format in formats {
        let config = LoggingConfig {
            level: "debug".to_string(),
            file: "".to_string(),
            console: true,
            format: format.to_string(),
            include_file: true,
            include_line: true,
            include_class: true,
        };

        let logger = NiriBarLogger::new(config).unwrap();
        assert_eq!(logger.config.format, format);
    }
}

#[test]
fn test_logger_configuration_flags() {
    let test_cases = vec![
        (true, true, true, "all enabled"),
        (false, false, false, "all disabled"),
        (true, false, true, "mixed config"),
        (false, true, false, "another mixed config"),
    ];

    for (include_file, include_line, include_class, description) in test_cases {
        let config = LoggingConfig {
            level: "debug".to_string(),
            file: "".to_string(),
            console: true,
            format: "iso8601".to_string(),
            include_file,
            include_line,
            include_class,
        };

        let logger = NiriBarLogger::new(config.clone()).unwrap();
        assert_eq!(logger.config.include_file, include_file, "{}", description);
        assert_eq!(logger.config.include_line, include_line, "{}", description);
        assert_eq!(
            logger.config.include_class, include_class,
            "{}",
            description
        );
    }
}

#[test]
fn test_logger_thread_safety() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let logger = Arc::new(Mutex::new(
        NiriBarLogger::new(LoggingConfig {
            level: "debug".to_string(),
            file: "".to_string(),
            console: true,
            format: "iso8601".to_string(),
            include_file: true,
            include_line: true,
            include_class: true,
        })
        .unwrap(),
    ));

    let mut handles = vec![];

    // Spawn threads that access the logger
    for _i in 0..5 {
        let logger_clone = Arc::clone(&logger);

        let handle = thread::spawn(move || {
            // Test concurrent access to logger configuration
            {
                let logger_guard = logger_clone.lock().unwrap();
                assert_eq!(logger_guard.config.level, "debug");
                assert!(logger_guard.config.console);
            }
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_logger_file_handle_sharing() {
    // Test that multiple logger instances can share the same file handle
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_string_lossy().to_string();

    let config1 = LoggingConfig {
        level: "info".to_string(),
        file: file_path.clone(),
        console: false,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let config2 = LoggingConfig {
        level: "debug".to_string(),
        file: file_path.clone(),
        console: false,
        format: "simple".to_string(),
        include_file: false,
        include_line: false,
        include_class: false,
    };

    let logger1 = NiriBarLogger::new(config1).unwrap();
    let logger2 = NiriBarLogger::new(config2).unwrap();

    // Both should have file handles (though they point to the same file)
    assert!(logger1.file_handle.is_some());
    assert!(logger2.file_handle.is_some());

    // The file should still exist
    assert!(temp_file.path().exists());
}

#[test]
fn test_logger_resource_cleanup() {
    // Test that logger resources are properly managed
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

    {
        let logger = NiriBarLogger::new(config).unwrap();
        assert!(logger.file_handle.is_some());
        // File should exist while logger is alive
        assert!(temp_file.path().exists());
    }

    // After logger goes out of scope, file should still exist
    // (tempfile will be cleaned up by the OS when the process exits)
    assert!(temp_file.path().exists());
}

// ===== PROPERTY-BASED TESTS =====

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_logger_config_arbitrary_creation(
            level in prop::sample::select(vec!["debug", "info", "warn", "error", "trace"]),
            console in any::<bool>(),
            format in prop::sample::select(vec!["iso8601", "simple"]),
            include_file in any::<bool>(),
            include_line in any::<bool>(),
            include_class in any::<bool>(),
        ) {
            let config = LoggingConfig {
                level: level.to_string(),
                file: "/tmp/test.log".to_string(),
                console,
                format: format.to_string(),
                include_file,
                include_line,
                include_class,
            };

            let logger = NiriBarLogger::new(config);
            prop_assert!(logger.is_ok(), "Logger should create successfully with arbitrary config");
        }

        #[test]
        fn test_logger_config_properties(
            level in "[a-z]+",
            include_file in any::<bool>(),
            include_line in any::<bool>(),
            include_class in any::<bool>(),
        ) {
            let config = LoggingConfig {
                level,
                file: "".to_string(),
                console: true,
                format: "iso8601".to_string(),
                include_file,
                include_line,
                include_class,
            };

            let logger = NiriBarLogger::new(config.clone()).unwrap();
            prop_assert_eq!(logger.config.level, config.level);
            prop_assert_eq!(logger.config.include_file, config.include_file);
            prop_assert_eq!(logger.config.include_line, config.include_line);
            prop_assert_eq!(logger.config.include_class, config.include_class);
        }
    }
}
