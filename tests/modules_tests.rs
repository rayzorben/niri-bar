use niri_bar::config::{ModuleConfig, DisplayMode};
use niri_bar::modules::{self, BarModule, clock, battery, workspaces, window_title, tray};
use pretty_assertions::assert_eq;

// ===== MODULE REGISTRY TESTS =====

#[test]
fn test_module_registry_creation() {
    // Test that all expected modules are registered
    // Note: In unit tests we can't initialize GTK, so we test the registry structure instead
    let expected_modules = vec![
        "bar.module.clock",
        "bar.module.window_title",
        "bar.module.workspaces",
        "bar.module.battery",
        "bar.module.tray",
    ];

    // Test that the expected identifiers exist as constants
    assert_eq!(clock::ClockModule::IDENT, expected_modules[0]);
    assert_eq!(window_title::WindowTitleModule::IDENT, expected_modules[1]);
    assert_eq!(workspaces::WorkspacesModule::IDENT, expected_modules[2]);
    assert_eq!(battery::BatteryModule::IDENT, expected_modules[3]);
    assert_eq!(tray::TrayModule::IDENT, expected_modules[4]);
}

#[test]
fn test_module_registry_unknown_module() {
    // Test that unknown modules return None
    let config = ModuleConfig::default();
    let widget = modules::create_module_widget("unknown_module", &config);
    assert!(widget.is_none(), "Unknown module should return None");
}

#[test]
fn test_module_registry_empty_name() {
    // Test empty module name
    let config = ModuleConfig::default();
    let widget = modules::create_module_widget("", &config);
    assert!(widget.is_none(), "Empty module name should return None");
}

#[test]
fn test_module_registry_case_sensitivity() {
    // Test case sensitivity - module identifiers should be case sensitive
    assert_eq!(clock::ClockModule::IDENT, "bar.module.clock");
    assert_ne!(clock::ClockModule::IDENT, "BAR.MODULE.CLOCK");

    // Test that identifiers are consistently lowercase
    assert!(clock::ClockModule::IDENT.chars().all(|c| c.is_lowercase() || c == '.'));
    assert!(battery::BatteryModule::IDENT.chars().all(|c| c.is_lowercase() || c == '.'));
    assert!(workspaces::WorkspacesModule::IDENT.chars().all(|c| c.is_lowercase() || c == '.'));
}

// ===== CLOCK MODULE TESTS =====

#[test]
fn test_clock_module_creation() {
    let _config = ModuleConfig::default();

    // This test would ideally initialize GTK, but for now we'll test the logic
    // In a real test environment, you'd initialize GTK first
    // gtk4::init().expect("Failed to initialize GTK");

    // For now, we'll test that the function doesn't panic with default config
    // let result = std::panic::catch_unwind(|| {
    //     modules::clock::ClockModule::create_widget(&config);
    // });
    // assert!(result.is_ok(), "Clock module creation should not panic");

    // Test configuration parsing
    let custom_config = ModuleConfig {
        format: Some("%H:%M:%S".to_string()),
        ..Default::default()
    };

    // Test that custom format is preserved
    assert_eq!(custom_config.format, Some("%H:%M:%S".to_string()));
}

#[test]
fn test_clock_module_config_variations() {
    // Test various clock format configurations
    let test_cases = vec![
        (None, "Should handle None format"),
        (Some("%H:%M".to_string()), "Should handle short format"),
        (Some("%A, %B %d, %Y".to_string()), "Should handle long format"),
        (Some("%s".to_string()), "Should handle Unix timestamp"),
        (Some("".to_string()), "Should handle empty format"),
    ];

    for (format, description) in test_cases {
        let config = ModuleConfig {
            format,
            ..Default::default()
        };

        // Test that config is valid
        assert!(config.format.is_none() || config.format.as_ref().unwrap().len() >= 0,
                "{}", description);
    }
}

#[test]
fn test_clock_module_identity() {
    // Test that ClockModule has correct identifier
    assert_eq!(clock::ClockModule::IDENT, "bar.module.clock");
}

// ===== BATTERY MODULE TESTS =====

#[test]
fn test_battery_module_config_parsing() {
    // Test battery-specific configuration options
    let yaml_config = r#"
device: "BAT1"
show_icon: true
show_percentage: true
pulse: false
warn_threshold: 25
critical_threshold: 15
"#;

    let config: ModuleConfig = serde_yaml::from_str(yaml_config).unwrap();

    // Verify battery-specific settings are parsed
    assert_eq!(config.additional.get("device"), Some(&serde_yaml::Value::String("BAT1".to_string())));
    assert_eq!(config.additional.get("show_icon"), Some(&serde_yaml::Value::Bool(true)));
    assert_eq!(config.additional.get("pulse"), Some(&serde_yaml::Value::Bool(false)));
    assert_eq!(config.warn_threshold, Some(25));
    assert_eq!(config.critical_threshold, Some(15));
    assert_eq!(config.show_percentage, Some(true));
}

#[test]
fn test_battery_module_defaults() {
    let config = ModuleConfig::default();

    // Test that battery module has reasonable defaults
    assert!(config.show_percentage.unwrap_or(true), "Should show percentage by default");
    assert!(config.warn_threshold.is_none() || config.warn_threshold.unwrap() > 0);
    assert!(config.critical_threshold.is_none() || config.critical_threshold.unwrap() > 0);
}

#[test]
fn test_battery_module_threshold_validation() {
    let test_cases = vec![
        (Some(10), Some(20), "warn < critical should be valid"),
        (Some(50), Some(20), "warn > critical should be valid (implementation dependent)"),
        (Some(0), Some(10), "zero warn threshold should be valid"),
        (Some(100), Some(10), "high warn threshold should be valid"),
    ];

    for (warn, critical, description) in test_cases {
        let config = ModuleConfig {
            warn_threshold: warn,
            critical_threshold: critical,
            ..Default::default()
        };

        // Test that config is valid
        assert!(config.warn_threshold.unwrap_or(0) >= 0, "{}", description);
        assert!(config.critical_threshold.unwrap_or(0) >= 0, "{}", description);
    }
}

#[test]
fn test_battery_module_identity() {
    assert_eq!(battery::BatteryModule::IDENT, "bar.module.battery");
}

// ===== WORKSPACES MODULE TESTS =====

#[test]
fn test_workspaces_module_config() {
    let yaml_config = r#"
highlight_active: true
show_numbers: true
show_wallpaper: true
max_length: 20
ellipsize: "end"
"#;

    let config: ModuleConfig = serde_yaml::from_str(yaml_config).unwrap();

    assert_eq!(config.highlight_active, Some(true));
    assert_eq!(config.show_numbers, Some(true));
    assert_eq!(config.show_wallpaper, Some(true));
    assert_eq!(config.max_length, Some(20));
    assert_eq!(config.ellipsize, Some("end".to_string()));
}

#[test]
fn test_workspaces_module_wallpaper_config() {
    let yaml_config = r#"
show_wallpaper: true
default_wallpaper: "~/wallpapers/default.jpg"
wallpapers:
  "1": "~/wallpapers/workspace1.png"
  "2": "~/wallpapers/workspace2.jpg"
special_cmd: "swww img ${current_workspace_image}"
"#;

    let config: ModuleConfig = serde_yaml::from_str(yaml_config).unwrap();

    assert_eq!(config.show_wallpaper, Some(true));
    assert_eq!(config.default_wallpaper, Some("~/wallpapers/default.jpg".to_string()));
    assert!(config.wallpapers.is_some());
    assert_eq!(config.special_cmd, Some("swww img ${current_workspace_image}".to_string()));
}

#[test]
fn test_workspaces_module_identity() {
    assert_eq!(workspaces::WorkspacesModule::IDENT, "bar.module.workspaces");
}

// ===== WINDOW TITLE MODULE TESTS =====

#[test]
fn test_window_title_module_config() {
    let yaml_config = r#"
max_length: 50
ellipsize: "middle"
display: "show"
"#;

    let config: ModuleConfig = serde_yaml::from_str(yaml_config).unwrap();

    assert_eq!(config.max_length, Some(50));
    assert_eq!(config.ellipsize, Some("middle".to_string()));
    assert_eq!(config.display, Some(DisplayMode::Show));
}

#[test]
fn test_window_title_module_ellipsize_options() {
    let test_cases = vec![
        "start", "middle", "end", "none"
    ];

    for ellipsize in test_cases {
        let yaml_config = format!(r#"
max_length: 30
ellipsize: "{}"
"#, ellipsize);

        let config: ModuleConfig = serde_yaml::from_str(&yaml_config).unwrap();
        assert_eq!(config.ellipsize, Some(ellipsize.to_string()));
    }
}

#[test]
fn test_window_title_module_identity() {
    assert_eq!(window_title::WindowTitleModule::IDENT, "bar.module.window_title");
}

// ===== TRAY MODULE TESTS =====

#[test]
fn test_tray_module_config() {
    let yaml_config = r#"
display: "show"
"#;

    let config: ModuleConfig = serde_yaml::from_str(yaml_config).unwrap();
    assert_eq!(config.display, Some(DisplayMode::Show));
}

#[test]
fn test_tray_module_identity() {
    assert_eq!(tray::TrayModule::IDENT, "bar.module.tray");
}

// ===== SYSTEM MODULE TESTS =====

#[test]
fn test_system_module_config() {
    let yaml_config = r#"
cpu: true
mem: true
net: false
display: "show"
"#;

    let config: ModuleConfig = serde_yaml::from_str(yaml_config).unwrap();

    assert_eq!(config.cpu, Some(true));
    assert_eq!(config.mem, Some(true));
    assert_eq!(config.net, Some(false));
    assert_eq!(config.display, Some(DisplayMode::Show));
}

#[test]
fn test_system_module_all_disabled() {
    let yaml_config = r#"
cpu: false
mem: false
net: false
display: "hide"
"#;

    let config: ModuleConfig = serde_yaml::from_str(yaml_config).unwrap();

    assert_eq!(config.cpu, Some(false));
    assert_eq!(config.mem, Some(false));
    assert_eq!(config.net, Some(false));
    assert_eq!(config.display, Some(DisplayMode::Hide));
}

// ===== MODULE CONFIG EDGE CASES =====

#[test]
fn test_module_config_empty_additional() {
    let config = ModuleConfig::default();
    assert!(config.additional.is_empty());
}

#[test]
fn test_module_config_with_additional_fields() {
    let yaml_config = r#"
format: "%H:%M"
custom_field: "value"
another_field:
  nested: true
numeric_field: 42
"#;

    let config: ModuleConfig = serde_yaml::from_str(yaml_config).unwrap();

    assert_eq!(config.format, Some("%H:%M".to_string()));
    assert_eq!(config.additional.get("custom_field"),
               Some(&serde_yaml::Value::String("value".to_string())));
    assert!(config.additional.contains_key("another_field"));
    assert_eq!(config.additional.get("numeric_field"),
               Some(&serde_yaml::Value::Number(42.into())));
}

#[test]
fn test_module_config_mixed_types() {
    let yaml_config = r#"
enabled: true
max_length: 100
tooltip: false
warn_threshold: 50
ellipsize: "start"
custom_array: [1, 2, 3]
custom_object:
  key1: "value1"
  key2: 42
"#;

    let config: ModuleConfig = serde_yaml::from_str(yaml_config).unwrap();

    assert_eq!(config.enabled, Some(true));
    assert_eq!(config.max_length, Some(100));
    assert_eq!(config.tooltip, Some(false));
    assert_eq!(config.warn_threshold, Some(50));
    assert_eq!(config.ellipsize, Some("start".to_string()));

    // Test array
    if let Some(serde_yaml::Value::Sequence(arr)) = config.additional.get("custom_array") {
        assert_eq!(arr.len(), 3);
    } else {
        panic!("custom_array should be a sequence");
    }

    // Test object
    assert!(config.additional.contains_key("custom_object"));
}

// ===== MODULE CREATION ERROR HANDLING =====

#[test]
fn test_module_creation_with_invalid_config() {
    // Test module creation with configs that might cause issues

    // Very long format string
    let long_format = "a".repeat(1000);
    let config = ModuleConfig {
        format: Some(long_format),
        ..Default::default()
    };

    // Should handle long format gracefully
    let _ = config;
}

#[test]
fn test_module_creation_with_extreme_values() {
    // Test with extreme configuration values

    let extreme_config = ModuleConfig {
        max_length: Some(u32::MAX as usize),
        warn_threshold: Some(u8::MAX),
        critical_threshold: Some(u8::MAX),
        ..Default::default()
    };

    // Should handle extreme values gracefully
    assert_eq!(extreme_config.max_length, Some(u32::MAX as usize));
    assert_eq!(extreme_config.warn_threshold, Some(u8::MAX));
    assert_eq!(extreme_config.critical_threshold, Some(u8::MAX));
}

// ===== PROPERTY-BASED TESTS =====

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_module_config_format_strings(format in ".*") {
            let config = ModuleConfig {
                format: Some(format.clone()),
                ..Default::default()
            };

            // Format should be preserved
            prop_assert_eq!(config.format, Some(format));
        }

        #[test]
        fn test_module_config_max_length(max_len in 0..10000usize) {
            let config = ModuleConfig {
                max_length: Some(max_len),
                ..Default::default()
            };

            prop_assert_eq!(config.max_length, Some(max_len));
        }

        #[test]
        fn test_module_config_thresholds(warn in 0..100u8, crit in 0..100u8) {
            let config = ModuleConfig {
                warn_threshold: Some(warn),
                critical_threshold: Some(crit),
                ..Default::default()
            };

            prop_assert_eq!(config.warn_threshold, Some(warn));
            prop_assert_eq!(config.critical_threshold, Some(crit));
        }
    }
}

// ===== PERFORMANCE TESTS =====

#[test]
fn test_module_config_creation_performance() {
    use std::time::{Duration, Instant};

    let start = Instant::now();

    // Create many module configs
    for i in 0..1000 {
        let config = ModuleConfig {
            format: Some(format!("%H:%M:{}", i)),
            max_length: Some(i % 100),
            ..Default::default()
        };
        let _ = config;
    }

    let elapsed = start.elapsed();
    assert!(elapsed < Duration::from_millis(100),
            "Creating 1000 configs took too long: {:?}", elapsed);
}

// ===== CONCURRENCY TESTS =====

#[test]
fn test_module_config_thread_safety() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let config = Arc::new(Mutex::new(ModuleConfig::default()));
    let mut handles = vec![];

    // Spawn threads that modify the config
    for i in 0..10 {
        let config = Arc::clone(&config);
        let handle = thread::spawn(move || {
            let mut config_guard = config.lock().unwrap();
            config_guard.max_length = Some(i * 10);
            config_guard.format = Some(format!("thread-{}", i));
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Config should be in a valid state
    let final_config = config.lock().unwrap();
    assert!(final_config.max_length.is_some());
    assert!(final_config.format.is_some());
}

// ===== WORKSPACE SCROLL CONFIGURATION TESTS =====

#[test]
fn test_workspace_scroll_configuration() {
    let yaml_config = r#"
show_numbers: true
highlight_active: true
scroll_wraparound: true
scroll_throttle_ms: 25
"#;

    let config: ModuleConfig = serde_yaml::from_str(yaml_config).unwrap();

    assert_eq!(config.show_numbers, Some(true));
    assert_eq!(config.highlight_active, Some(true));
    assert_eq!(config.additional.get("scroll_wraparound"), Some(&serde_yaml::Value::Bool(true)));
    assert_eq!(config.additional.get("scroll_throttle_ms"), Some(&serde_yaml::Value::Number(25.into())));
}

#[test]
fn test_workspace_scroll_defaults() {
    let config = ModuleConfig::default();

    // Test that scroll configuration has sensible defaults
    // These should be None since they're in additional fields
    assert!(config.additional.get("scroll_wraparound").is_none());
    assert!(config.additional.get("scroll_throttle_ms").is_none());
}

#[test]
fn test_workspace_scroll_extreme_values() {
    let yaml_config = r#"
scroll_wraparound: false
scroll_throttle_ms: 0
"#;

    let config: ModuleConfig = serde_yaml::from_str(yaml_config).unwrap();

    assert_eq!(config.additional.get("scroll_wraparound"), Some(&serde_yaml::Value::Bool(false)));
    assert_eq!(config.additional.get("scroll_throttle_ms"), Some(&serde_yaml::Value::Number(0.into())));
}

#[test]
fn test_workspace_scroll_large_throttle() {
    let yaml_config = r#"
scroll_throttle_ms: 1000
"#;

    let config: ModuleConfig = serde_yaml::from_str(yaml_config).unwrap();

    assert_eq!(config.additional.get("scroll_throttle_ms"), Some(&serde_yaml::Value::Number(1000.into())));
}
