use niri_bar_new::application::Application;
use niri_bar_new::config::LoggingConfig;

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

    let app = Application::new(logging_config);
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
