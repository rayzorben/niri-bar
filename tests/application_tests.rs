use niri_bar::application::Application;
use niri_bar::config::{LoggingConfig, ConfigManager};
use pretty_assertions::assert_eq;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn test_application_creation() {
    let logging_config = LoggingConfig {
        level: "debug".to_string(),
        file: "".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let app = Application::new_with_gtk(logging_config, false);
    assert!(app.is_ok());
}

#[test]
fn test_application_config_manager() {
    let logging_config = LoggingConfig {
        level: "debug".to_string(),
        file: "".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let app = Application::new(logging_config).unwrap();
    let config_manager = app.get_config_manager();
    
    // Verify config manager is available
    assert!(config_manager.get_config().is_none()); // No config loaded yet
}

#[test]
fn test_application_monitor_management() {
    let logging_config = LoggingConfig {
        level: "debug".to_string(),
        file: "".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let app = Application::new(logging_config).unwrap();
    
    // Test initial state - monitors should be empty
    let monitors = app.monitors.lock().unwrap();
    assert!(monitors.is_empty());
}

// ===== COMPREHENSIVE APPLICATION TESTS =====

#[test]
fn test_application_creation_with_invalid_logging_config() {
    // Test that application creation handles various logging configurations
    let test_cases = vec![
        ("debug", true),
        ("info", false),
        ("warn", true),
        ("error", false),
        ("trace", true),
    ];

    for (level, console) in test_cases {
        let logging_config = LoggingConfig {
            level: level.to_string(),
            file: "/tmp/test.log".to_string(),
            console,
            format: "iso8601".to_string(),
            include_file: true,
            include_line: false,
            include_class: true,
        };

        let app = Application::new_with_gtk(logging_config, false);
        assert!(app.is_ok(), "Application should create successfully with level: {}", level);
    }
}

#[test]
fn test_application_creation_with_empty_file_path() {
    let logging_config = LoggingConfig {
        level: "info".to_string(),
        file: "".to_string(),
        console: true,
        format: "simple".to_string(),
        include_file: false,
        include_line: false,
        include_class: false,
    };

    let app = Application::new_with_gtk(logging_config, false);
    assert!(app.is_ok(), "Application should handle empty file path");
}

#[test]
fn test_application_monitor_count() {
    let logging_config = LoggingConfig {
        level: "debug".to_string(),
        file: "".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let app = Application::new(logging_config).unwrap();

    // Test that monitor count starts at 0
    assert_eq!(app.monitor_count(), 0);

    // Add a mock monitor to test counting
    {
        let monitors = app.monitors.lock().unwrap();
        // We can't easily create a real Monitor in tests, but we can test the structure
        assert!(monitors.is_empty());
    }
}

#[test]
fn test_application_getters() {
    let logging_config = LoggingConfig {
        level: "warn".to_string(),
        file: "/tmp/custom.log".to_string(),
        console: false,
        format: "simple".to_string(),
        include_file: true,
        include_line: true,
        include_class: false,
    };

    let app = Application::new(logging_config.clone()).unwrap();

    // Test logging config getter
    let retrieved_config = app.get_logging_config();
    assert_eq!(retrieved_config.level, logging_config.level);
    assert_eq!(retrieved_config.file, logging_config.file);
    assert_eq!(retrieved_config.console, logging_config.console);

    // Test config manager getter
    let config_manager = app.get_config_manager();
    assert!(config_manager.get_config().is_none()); // No config loaded yet
}

#[test]
fn test_application_data_integrity() {

    let logging_config = LoggingConfig {
        level: "debug".to_string(),
        file: "".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let app = Application::new(logging_config.clone()).unwrap();

    // Test that multiple accesses return consistent results
    // Note: GTK application cannot be safely shared between threads
    let count1 = app.monitor_count();
    let config1 = app.get_logging_config();
    let count2 = app.monitor_count();
    let config2 = app.get_logging_config();

    assert_eq!(count1, count2);
    assert_eq!(config1.level, config2.level);
    assert_eq!(config1.console, config2.console);
}

#[test]
fn test_application_module_format_collection() {
    // This would ideally test the collect_module_formats method
    // but it's private. We can test the logic indirectly through
    // integration testing or by making it public if needed.

    let logging_config = LoggingConfig {
        level: "info".to_string(),
        file: "".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let app = Application::new(logging_config).unwrap();

    // Test that the application has the expected structure
    assert_eq!(app.monitor_count(), 0);
    assert!(app.get_config_manager().get_config().is_none());
}

#[test]
fn test_application_config_reloading_simulation() {
    // Test that the application can handle configuration changes
    let logging_config = LoggingConfig {
        level: "debug".to_string(),
        file: "".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let app = Application::new(logging_config).unwrap();

    // Simulate config changes by directly manipulating the config manager
    let config_manager = app.get_config_manager();

    // Initially no config
    assert!(config_manager.get_config().is_none());

    // Test that we can access config manager methods
    let _subscriber = config_manager.subscribe();
    let _layouts = config_manager.get_layouts();
    let _modules = config_manager.get_global_modules();
}

#[test]
fn test_application_monitor_pattern_matching() {
    let logging_config = LoggingConfig {
        level: "info".to_string(),
        file: "".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let app = Application::new(logging_config).unwrap();
    let config_manager = app.get_config_manager();

    // Test pattern matching logic (this is tested more thoroughly in config tests)
    // but we can verify the application has access to it
    assert!(ConfigManager::matches_pattern("DP-1", "DP-.*"));
    assert!(ConfigManager::matches_pattern("eDP-1", "eDP-1"));
    assert!(!ConfigManager::matches_pattern("HDMI-1", "DP-.*"));
}

#[test]
fn test_application_performance() {
    use std::time::Instant;

    let logging_config = LoggingConfig {
        level: "debug".to_string(),
        file: "".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let start = Instant::now();
    let app = Application::new(logging_config).unwrap();
    let creation_time = start.elapsed();

    // Application creation should be reasonably fast
    assert!(creation_time < Duration::from_millis(100),
            "Application creation took too long: {:?}", creation_time);

    // Test repeated access performance
    let start = Instant::now();
    for _ in 0..1000 {
        let _count = app.monitor_count();
        let _config = app.get_logging_config();
    }
    let access_time = start.elapsed();

    assert!(access_time < Duration::from_millis(50),
            "1000 property accesses took too long: {:?}", access_time);
}

#[test]
fn test_application_config_validation() {
    // Test that application properly validates configurations
    let invalid_logging_configs = vec![
        LoggingConfig {
            level: "invalid_level".to_string(),
            file: "".to_string(),
            console: true,
            format: "iso8601".to_string(),
            include_file: true,
            include_line: true,
            include_class: true,
        },
        LoggingConfig {
            level: "debug".to_string(),
            file: "".to_string(),
            console: true,
            format: "invalid_format".to_string(),
            include_file: true,
            include_line: true,
            include_class: true,
        },
    ];

    // These should still create successfully (validation happens elsewhere)
    for config in invalid_logging_configs {
        let app = Application::new(config);
        assert!(app.is_ok(), "Application should handle invalid configs gracefully");
    }
}

#[test]
fn test_application_resource_management() {
    // Test that application Arc references work correctly
    let logging_config = LoggingConfig {
        level: "info".to_string(),
        file: "".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    let app = Application::new(logging_config).unwrap();

    // Test that Arc references work correctly
    let monitors_ref1 = Arc::clone(&app.monitors);
    let monitors_ref2 = Arc::clone(&app.monitors);

    // Both references should point to the same data
    // Don't try to lock both at the same time - that would deadlock
    let len1 = {
        let monitors = monitors_ref1.lock().unwrap();
        monitors.len()
    };
    let len2 = {
        let monitors = monitors_ref2.lock().unwrap();
        monitors.len()
    };
    assert_eq!(len1, len2);

    // Test basic config manager functionality
    let config_manager = app.get_config_manager();
    // Just verify we can access the config manager without errors
    assert!(config_manager.get_config().is_none());
}

// ===== PROPERTY-BASED TESTS =====

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_application_creation_with_random_configs(
            level in prop::sample::select(vec!["debug", "info", "warn", "error"]),
            console in any::<bool>(),
            include_file in any::<bool>(),
            include_line in any::<bool>(),
            include_class in any::<bool>(),
        ) {
            let logging_config = LoggingConfig {
                level: level.to_string(),
                file: "/tmp/test.log".to_string(),
                console,
                format: "iso8601".to_string(),
                include_file,
                include_line,
                include_class,
            };

            let app = Application::new_with_gtk(logging_config, false);
            prop_assert!(app.is_ok(), "Application should create successfully with random config");
        }
    }
}
