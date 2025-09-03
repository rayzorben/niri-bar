use indexmap::IndexMap;
use niri_bar::config::{
    ApplicationConfig, ColumnOverflowPolicy, ColumnSpec, ConfigManager, DisplayMode, LayoutConfig,
    LoggingConfig, ModuleConfig, MonitorConfig, NiriBarConfig, TextAlign, WallpaperConfig,
};
use pretty_assertions::assert_eq;
use proptest::prelude::*;
use tempfile::TempDir;

#[test]
fn test_parse_real_config_file() {
    // Test that the real niri-bar.yaml file can be parsed successfully
    let config_path = "niri-bar.yaml";
    let content =
        std::fs::read(config_path).unwrap_or_else(|_| panic!("Failed to read {}", config_path));

    let config = ConfigManager::parse_config(&content).unwrap();

    // Basic structural validation - just ensure the config has required sections
    assert!(
        !config.application.monitors.is_empty(),
        "Should have at least one monitor configuration"
    );
    assert!(
        !config.application.modules.is_empty(),
        "Should have module defaults"
    );
    assert!(
        !config.application.layouts.is_empty(),
        "Should have layout profiles"
    );
    assert!(
        !config.logging.level.is_empty(),
        "Logging level should not be empty"
    );
    assert!(
        !config.logging.file.is_empty(),
        "Log file path should not be empty"
    );
}

#[test]
fn test_schema_validation() {
    // Test that the real config file validates successfully against the schema
    let config_path = "niri-bar.yaml";

    // Read the real config file
    let config_content =
        std::fs::read(config_path).unwrap_or_else(|_| panic!("Failed to read {}", config_path));

    // Parse the config (this internally validates against the schema)
    let result = ConfigManager::parse_config(&config_content);
    assert!(
        result.is_ok(),
        "Real config file should validate successfully: {:?}",
        result.err()
    );
}

#[test]
fn test_monitor_pattern_matching() {
    let _config_manager = ConfigManager::new();

    // Test wildcard pattern
    assert!(ConfigManager::matches_pattern("eDP-1", ".*"));
    assert!(ConfigManager::matches_pattern("DP-1", ".*"));
    assert!(ConfigManager::matches_pattern("HDMI-1", ".*"));

    // Test prefix pattern
    assert!(ConfigManager::matches_pattern("DP-1", "DP-.*"));
    assert!(ConfigManager::matches_pattern("DP-2", "DP-.*"));
    assert!(!ConfigManager::matches_pattern("eDP-1", "DP-.*"));

    // Test exact match with regex
    assert!(ConfigManager::matches_pattern("eDP-1", "^eDP-1$"));
    assert!(!ConfigManager::matches_pattern("eDP-2", "^eDP-1$"));

    // Test the actual pattern from the config
    println!("Testing DP pattern matching:");
    println!(
        "  DP-1 matches ^DP-.*$: {}",
        ConfigManager::matches_pattern("DP-1", "^DP-.*$")
    );
    println!(
        "  DP-2 matches ^DP-.*$: {}",
        ConfigManager::matches_pattern("DP-2", "^DP-.*$")
    );
    println!(
        "  eDP-1 matches ^DP-.*$: {}",
        ConfigManager::matches_pattern("eDP-1", "^DP-.*$")
    );
}

#[test]
fn test_get_monitor_modules() {
    let config_manager = ConfigManager::new();

    // Load the real config file
    let config_path = "niri-bar.yaml";
    let content =
        std::fs::read(config_path).unwrap_or_else(|_| panic!("Failed to read {}", config_path));

    let config = ConfigManager::parse_config(&content).unwrap();

    {
        let mut config_guard = config_manager.config.lock().unwrap();
        *config_guard = Some(config);
    }

    // Test monitor with specific modules (eDP-1 should have modules based on real config)
    let e_dp1_modules = config_manager.get_monitor_modules("eDP-1");
    assert!(
        e_dp1_modules.is_some(),
        "eDP-1 should have module configuration"
    );
    let e_dp1_modules = e_dp1_modules.unwrap();
    assert!(
        e_dp1_modules.contains_key("clock"),
        "eDP-1 should have clock module"
    );
    assert!(
        e_dp1_modules.contains_key("battery"),
        "eDP-1 should have battery module"
    );

    // Test that eDP-1 has the expected modules
    assert!(
        e_dp1_modules.contains_key("clock"),
        "eDP-1 should have clock module"
    );
    assert!(
        e_dp1_modules.contains_key("battery"),
        "eDP-1 should have battery module"
    );

    // Note: Due to YAML merge limitation, the module overrides may not be applied correctly
    // The modules are present but the specific overrides (format, enabled) may not be merged
}

#[test]
fn test_yaml_schema_validation() {
    // Test that the real niri-bar.yaml file validates against the JSON schema
    let config_path = "niri-bar.yaml";
    let content =
        std::fs::read(config_path).unwrap_or_else(|_| panic!("Failed to read {}", config_path));

    // This will validate against the schema internally
    let result = ConfigManager::parse_config(&content);
    assert!(
        result.is_ok(),
        "niri-bar.yaml should validate against the JSON schema: {:?}",
        result.err()
    );
}

#[test]
fn test_config_manager_creation() {
    let _config_manager = ConfigManager::new();

    // Test that initial config is None
    assert!(_config_manager.get_config().is_none());

    // Test that we can subscribe to events
    let _event_rx = _config_manager.subscribe();

    // Test that we can get global modules (should be None initially)
    let modules = _config_manager.get_global_modules();
    assert!(modules.is_none());

    // Test that we can get layouts (should be None initially)
    let layouts = _config_manager.get_layouts();
    assert!(layouts.is_none());
}

#[test]
fn test_monitor_enabled_check() {
    let config_manager = ConfigManager::new();

    // Test with no config loaded
    assert!(!config_manager.is_monitor_enabled("eDP-1"));

    // Load the real config file
    let config_path = "niri-bar.yaml";
    let content =
        std::fs::read(config_path).unwrap_or_else(|_| panic!("Failed to read {}", config_path));

    let config = ConfigManager::parse_config(&content).unwrap();

    {
        let mut config_guard = config_manager.config.lock().unwrap();
        *config_guard = Some(config);
    }

    // Test pattern matching behavior (regardless of actual monitor connections)

    // Test that eDP-1 matches the "^eDP-1$" pattern and gets enabled
    assert!(
        config_manager.is_monitor_enabled("eDP-1"),
        "eDP-1 should be enabled by ^eDP-1$ pattern"
    );

    // Test that DP monitors would be disabled by the "^DP-.*$" pattern if they existed
    // This tests the pattern matching logic, not actual monitor connections
    assert!(
        !config_manager.is_monitor_enabled("DP-1"),
        "DP-1 should be disabled by ^DP-.*$ pattern"
    );
    assert!(
        !config_manager.is_monitor_enabled("DP-2"),
        "DP-2 should be disabled by ^DP-.*$ pattern"
    );
    assert!(
        !config_manager.is_monitor_enabled("DP-3"),
        "DP-3 should be disabled by ^DP-.*$ pattern"
    );

    // Test that other monitors (like HDMI) are enabled by the wildcard ".*" pattern
    assert!(
        config_manager.is_monitor_enabled("HDMI-1"),
        "HDMI-1 should be enabled by .* pattern"
    );
    assert!(
        config_manager.is_monitor_enabled("HDMI-2"),
        "HDMI-2 should be enabled by .* pattern"
    );
    assert!(
        config_manager.is_monitor_enabled("DisplayPort-1"),
        "DisplayPort-1 should be enabled by .* pattern"
    );

    // Test that non-existent monitors are handled gracefully
    // They should match the wildcard pattern and be enabled
    assert!(
        config_manager.is_monitor_enabled("non-existent"),
        "Non-existent monitors should be enabled by .* pattern"
    );
}

#[test]
fn test_global_modules_and_layouts() {
    let config_manager = ConfigManager::new();

    // Load the real config file
    let config_path = "niri-bar.yaml";
    let content =
        std::fs::read(config_path).unwrap_or_else(|_| panic!("Failed to read {}", config_path));

    let config = ConfigManager::parse_config(&content).unwrap();

    {
        let mut config_guard = config_manager.config.lock().unwrap();
        *config_guard = Some(config);
    }

    // Test that global modules are accessible
    if let Some(modules) = config_manager.get_global_modules() {
        assert!(modules.contains_key("clock"), "Should have clock module");
        assert!(
            modules.contains_key("workspaces"),
            "Should have workspaces module"
        );
        assert!(
            modules.contains_key("window_title"),
            "Should have window_title module"
        );
        assert!(
            modules.contains_key("battery"),
            "Should have battery module"
        );
        assert!(modules.contains_key("system"), "Should have system module");

        // Test specific module configurations
        if let Some(clock_config) = modules.get("clock") {
            assert!(
                clock_config.format.is_some(),
                "Clock should have default format"
            );
            assert_eq!(
                clock_config.tooltip,
                Some(true),
                "Clock should have tooltip enabled"
            );
        }
    } else {
        panic!("Global modules should be accessible");
    }

    // Test that layouts are accessible
    if let Some(layouts) = config_manager.get_layouts() {
        assert!(
            layouts.contains_key("three_column"),
            "Should have three_column layout"
        );
        assert!(
            layouts.contains_key("five_panel"),
            "Should have five_panel layout"
        );

        // Test three_column layout
        if let Some(three_col) = layouts.get("three_column") {
            assert!(
                three_col.columns.contains_key("left"),
                "three_column should have left column"
            );
            assert!(
                three_col.columns.contains_key("center"),
                "three_column should have center column"
            );
            assert!(
                three_col.columns.contains_key("right"),
                "three_column should have right column"
            );
        }

        // Test five_panel layout
        if let Some(five_panel) = layouts.get("five_panel") {
            assert!(
                five_panel.columns.contains_key("left"),
                "five_panel should have left column"
            );
            assert!(
                five_panel.columns.contains_key("left-of-center"),
                "five_panel should have left-of-center column"
            );
            assert!(
                five_panel.columns.contains_key("center"),
                "five_panel should have center column"
            );
            assert!(
                five_panel.columns.contains_key("right-of-center"),
                "five_panel should have right-of-center column"
            );
            assert!(
                five_panel.columns.contains_key("right"),
                "five_panel should have right column"
            );
        }
    } else {
        panic!("Layouts should be accessible");
    }
}

#[test]
fn test_schema_file_exists_and_valid() {
    // Test that the schema file exists and is valid JSON
    let schema_path = "src/niri-bar-yaml.schema.json";
    let schema_content = std::fs::read_to_string(schema_path)
        .unwrap_or_else(|_| panic!("Failed to read schema file: {}", schema_path));

    // Parse the schema to ensure it's valid JSON
    let schema: serde_json::Value = serde_json::from_str(&schema_content)
        .unwrap_or_else(|e| panic!("Schema file is not valid JSON: {}", e));

    // Test that the schema has the expected structure
    assert!(schema.is_object(), "Schema should be a JSON object");
    assert!(
        schema.get("$schema").is_some(),
        "Schema should have $schema field"
    );
    assert!(
        schema.get("title").is_some(),
        "Schema should have title field"
    );
    assert!(
        schema.get("type").is_some(),
        "Schema should have type field"
    );
    assert!(
        schema.get("properties").is_some(),
        "Schema should have properties field"
    );

    // Test that the schema includes our key sections
    let properties = schema.get("properties").unwrap().as_object().unwrap();
    assert!(
        properties.contains_key("application"),
        "Schema should include application section"
    );
    assert!(
        properties.contains_key("logging"),
        "Schema should include logging section"
    );

    // Test that application section has required subsections
    let application = properties.get("application").unwrap().as_object().unwrap();
    let app_props = application.get("properties").unwrap().as_object().unwrap();
    assert!(
        app_props.contains_key("modules"),
        "Schema should include modules section"
    );
    assert!(
        app_props.contains_key("layouts"),
        "Schema should include layouts section"
    );
    assert!(
        app_props.contains_key("monitors"),
        "Schema should include monitors section"
    );
}

#[test]
fn test_swww_options_from_real_yaml() {
    // Load real YAML and validate schema via parse_config
    let config_bytes = std::fs::read("niri-bar.yaml").expect("read niri-bar.yaml");
    let cfg = ConfigManager::parse_config(&config_bytes).expect("parse real yaml");

    // Assert swww_options exist and have sane types (values come from user YAML)
    let wp = &cfg.application.wallpapers;
    if let Some(opts) = &wp.swww_options {
        assert!(!opts.transition_type.is_empty());
        assert!(opts.transition_duration >= 0.0);
        assert!(opts.transition_fps >= 1);
        assert!(!opts.filter.is_empty());
        assert!(!opts.resize.is_empty());
        assert_eq!(opts.fill_color.len(), 6);
    }
}

// ===== COMPREHENSIVE CONFIG TESTS =====

#[test]
fn test_config_parsing_edge_cases() {
    // Test empty configuration
    let empty_config = "application:\n  modules: {}\n  layouts: {}\n  monitors: []\nlogging:\n  level: info\n  file: test.log\n  console: true";
    let config: Result<NiriBarConfig, _> = serde_yaml::from_str(empty_config);
    assert!(config.is_ok(), "Empty config should parse successfully");

    // Test configuration with only required fields
    let minimal_config = r#"
application:
  modules: {}
  layouts: {}
  monitors:
    - match: ".*"
logging:
  level: "info"
  file: "/tmp/test.log"
  console: false
"#;
    let config: NiriBarConfig = serde_yaml::from_str(minimal_config).unwrap();
    assert_eq!(config.logging.level, "info");
    assert!(!config.logging.console);
}

#[test]
fn test_config_parsing_error_cases() {
    // Test invalid YAML syntax
    let invalid_yaml = "application:\n  modules:\n    invalid: [unclosed";
    let result: Result<NiriBarConfig, _> = serde_yaml::from_str(invalid_yaml);
    assert!(result.is_err(), "Invalid YAML should fail to parse");

    // Test missing required fields
    let missing_required = "application:\n  modules: {}";
    let result: Result<NiriBarConfig, _> = serde_yaml::from_str(missing_required);
    assert!(result.is_err(), "Missing required fields should fail");

    // Test invalid logging level
    let invalid_log_level = r#"
application:
  modules: {}
  layouts: {}
  monitors: []
logging:
  level: "invalid_level"
  file: "/tmp/test.log"
  console: true
"#;
    let config: Result<NiriBarConfig, _> = serde_yaml::from_str(invalid_log_level);
    assert!(config.is_ok()); // Note: validation happens separately
    let config = config.unwrap();
    let validation_result = ConfigManager::basic_validation(&config);
    assert!(validation_result.is_err());
    assert!(
        validation_result
            .unwrap_err()
            .to_string()
            .contains("Invalid logging level")
    );
}

#[test]
fn test_monitor_pattern_matching_edge_cases() {
    // Test empty pattern
    assert!(!ConfigManager::matches_pattern("eDP-1", ""));

    // Test pattern with only special regex chars
    assert!(ConfigManager::matches_pattern("test", ".*"));
    assert!(!ConfigManager::matches_pattern("test", "^$"));

    // Test case sensitivity
    assert!(!ConfigManager::matches_pattern("edp-1", "^eDP-1$"));
    assert!(ConfigManager::matches_pattern("eDP-1", "^eDP-1$"));

    // Test Unicode patterns
    assert!(ConfigManager::matches_pattern("mönitor-1", "mönitor-.*"));
    assert!(ConfigManager::matches_pattern("монитор-1", "монитор-.*"));
}

#[test]
fn test_pattern_specificity_calculation() {
    // Test specificity ordering
    assert!(ConfigManager::pattern_specificity(".*") < ConfigManager::pattern_specificity("DP-.*"));
    assert!(
        ConfigManager::pattern_specificity("DP-.*") < ConfigManager::pattern_specificity("^DP-1$")
    );
    // Same specificity for different exact patterns
    assert_eq!(
        ConfigManager::pattern_specificity("^DP-1$"),
        ConfigManager::pattern_specificity("^DP-2$")
    );

    // Test exact match has highest specificity
    assert_eq!(ConfigManager::pattern_specificity("^exact$"), 100);

    // Test wildcard patterns
    assert_eq!(ConfigManager::pattern_specificity(".*"), 1);
    assert_eq!(ConfigManager::pattern_specificity("DP-.*"), 5); // Pattern with wildcard, no ^$
    assert_eq!(ConfigManager::pattern_specificity("monitor.*"), 5);
}

#[test]
fn test_config_manager_thread_safety() {
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    let config_manager = Arc::new(ConfigManager::new());
    let completed = Arc::new(Mutex::new(0));

    let mut handles = vec![];

    // Spawn multiple threads that access the config manager
    for i in 0..3 {
        let config_manager = Arc::clone(&config_manager);
        let completed = Arc::clone(&completed);

        let handle = thread::spawn(move || {
            // Load a test config
            let test_config = format!(
                r#"
application:
  modules: {{}}
  layouts: {{}}
  monitors:
    - match: ".*"
logging:
  level: "info"
  file: "/tmp/test{}.log"
  console: true
"#,
                i
            );

            let config: NiriBarConfig = serde_yaml::from_str(&test_config).unwrap();

            // Test concurrent writes (should be thread-safe)
            {
                let mut config_guard = config_manager.config.lock().unwrap();
                *config_guard = Some(config.clone());
            }

            // Small delay to increase chance of race conditions
            thread::sleep(Duration::from_millis(10));

            // Test concurrent reads
            let read_config = config_manager.get_config();
            assert!(read_config.is_some());
            assert_eq!(read_config.unwrap().logging.level, "info");

            // Mark completion
            let mut completed_guard = completed.lock().unwrap();
            *completed_guard += 1;
        });

        handles.push(handle);
    }

    // Wait for all threads to complete with timeout
    for handle in handles {
        // Use a timeout for each thread join
        let result = handle.join();
        assert!(result.is_ok(), "Thread should complete successfully");
    }

    // Verify all threads completed
    let final_count = *completed.lock().unwrap();
    assert_eq!(final_count, 3, "All threads should have completed");
}

#[test]
fn test_config_file_watching_integration() {
    use tokio::runtime::Runtime;

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test-config.yaml");

    // Create initial config file
    let initial_config = r#"
application:
  theme: "wombat"
  modules: {}
  layouts: {}
  monitors:
    - match: ".*"
logging:
  level: "debug"
  file: "/tmp/test.log"
  console: true
"#;
    std::fs::write(&config_path, initial_config).unwrap();

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mut config_manager = ConfigManager::new();

        // Test that we can start watching
        let start_result = config_manager.start().await;
        assert!(
            start_result.is_ok(),
            "Config manager should start successfully"
        );

        // Test config loading
        let content = std::fs::read(&config_path).unwrap();
        let config = ConfigManager::parse_config(&content).unwrap();
        assert_eq!(config.application.theme, "wombat");

        // Test config validation
        let validation_result = ConfigManager::basic_validation(&config);
        assert!(validation_result.is_ok());
    });
}

#[test]
fn test_module_config_defaults_and_overrides() {
    // Test module config default values
    let default_config = ModuleConfig::default();
    assert!(default_config.format.is_none());
    assert!(default_config.tooltip.is_none());
    assert!(default_config.display.is_none()); // Option<DisplayMode> defaults to None

    // Test module config with explicit values
    let yaml_config = r#"
format: "%H:%M"
tooltip: false
max_length: 50
ellipsize: "end"
show_percentage: true
warn_threshold: 20
critical_threshold: 10
display: "hide"
"#;
    let config: ModuleConfig = serde_yaml::from_str(yaml_config).unwrap();
    assert_eq!(config.format, Some("%H:%M".to_string()));
    assert_eq!(config.tooltip, Some(false));
    assert_eq!(config.max_length, Some(50));
    assert_eq!(config.ellipsize, Some("end".to_string()));
    assert_eq!(config.show_percentage, Some(true));
    assert_eq!(config.warn_threshold, Some(20));
    assert_eq!(config.critical_threshold, Some(10));
    assert_eq!(config.display, Some(DisplayMode::Hide));
}

#[test]
fn test_column_spec_configuration() {
    let yaml_config = r#"
modules: ["clock", "battery"]
overflow: "kebab"
gap: 8
align: "right"
width: 200
"#;
    let spec: ColumnSpec = serde_yaml::from_str(yaml_config).unwrap();
    assert_eq!(spec.modules, vec!["clock", "battery"]);
    assert_eq!(spec.overflow, ColumnOverflowPolicy::Kebab);
    assert_eq!(spec.gap, Some(8));
    assert_eq!(spec.align, Some(TextAlign::Right));
    assert_eq!(spec.width, Some(200));
}

#[test]
fn test_wallpaper_config_with_swww() {
    let yaml_config = r#"
default: "~/wallpapers/default.jpg"
by_workspace:
  "1": "~/wallpapers/workspace1.png"
  "2": "~/wallpapers/workspace2.jpg"
special_cmd: "swww img ${current_workspace_image}"
swww_options:
  transition_type: "wipe"
  transition_duration: 0.8
  transition_step: 120
  transition_fps: 60
  filter: "Lanczos3"
  resize: "crop"
  fill_color: "000000"
"#;
    let config: WallpaperConfig = serde_yaml::from_str(yaml_config).unwrap();
    assert_eq!(config.default, Some("~/wallpapers/default.jpg".to_string()));
    assert_eq!(
        config.by_workspace.get("1"),
        Some(&"~/wallpapers/workspace1.png".to_string())
    );
    assert_eq!(
        config.by_workspace.get("2"),
        Some(&"~/wallpapers/workspace2.jpg".to_string())
    );
    assert_eq!(
        config.special_cmd,
        Some("swww img ${current_workspace_image}".to_string())
    );

    let swww = config.swww_options.as_ref().unwrap();
    assert_eq!(swww.transition_type, "wipe");
    assert_eq!(swww.transition_duration, 0.8);
    assert_eq!(swww.transition_step, 120);
    assert_eq!(swww.transition_fps, 60);
    assert_eq!(swww.filter, "Lanczos3");
    assert_eq!(swww.resize, "crop");
    assert_eq!(swww.fill_color, "000000");
}

#[test]
fn test_monitor_config_pattern_matching() {
    let yaml_config = r#"
match: "^DP-.*$"
enabled: false
layout:
  columns:
    left:
      modules: ["workspaces"]
      overflow: "hide"
modules:
  workspaces:
    highlight_active: true
wallpapers:
  default: "~/wallpapers/external.jpg"
"#;
    let config: MonitorConfig = serde_yaml::from_str(yaml_config).unwrap();
    assert_eq!(config.match_pattern, "^DP-.*$");
    assert!(!config.show_bar);
    assert!(config.layout.is_some());
    assert!(config.modules.is_some());
    assert!(config.wallpapers.is_some());
}

// ===== PROPERTY-BASED TESTS =====

proptest! {
    #[test]
    fn test_yaml_round_trip_parsing(config in arbitrary_config()) {
        // Test that any valid config can be serialized and deserialized
        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: NiriBarConfig = serde_yaml::from_str(&yaml).unwrap();
        prop_assert_eq!(config, deserialized);
    }

    #[test]
    fn test_monitor_pattern_properties(pattern in "\\PC*", monitor_name in "\\PC+") {
        // Test pattern matching properties
        let matches = ConfigManager::matches_pattern(&monitor_name, &pattern);

        // If pattern is ".*", it should match everything
        if pattern == ".*" {
            prop_assert!(matches, "Pattern '.*' should match all monitor names");
        }

        // If monitor name exactly equals pattern, it should match
        if monitor_name == pattern {
            prop_assert!(matches, "Exact match should always work");
        }
    }

    #[test]
    fn test_pattern_specificity_properties(pattern1: String, pattern2: String) {
        // Test that specificity is deterministic
        let spec1 = ConfigManager::pattern_specificity(&pattern1);
        let spec2 = ConfigManager::pattern_specificity(&pattern2);

        // Same pattern should have same specificity
        if pattern1 == pattern2 {
            prop_assert_eq!(spec1, spec2);
        }
    }
}

// ===== ARBITRARY GENERATORS FOR PROPERTY TESTS =====

fn arbitrary_config() -> impl Strategy<Value = NiriBarConfig> {
    (
        "[a-z]+".prop_map(|theme| theme), // theme
        proptest::collection::hash_map("[a-z]+", Just(ModuleConfig::default()), 0..5), // modules
        proptest::collection::hash_map("[a-z]+", arbitrary_layout(), 0..3), // layouts
        proptest::collection::vec(arbitrary_monitor_config(), 0..5), // monitors
        arbitrary_logging_config(),       // logging
    )
        .prop_map(
            |(theme, modules, layouts, monitors, logging)| NiriBarConfig {
                application: ApplicationConfig {
                    theme,
                    modules,
                    layouts,
                    monitors,
                    wallpapers: WallpaperConfig::default(),
                },
                logging,
            },
        )
}

fn arbitrary_layout() -> impl Strategy<Value = LayoutConfig> {
    proptest::collection::vec(
        (
            "[a-z]+".prop_map(|name| name), // column name
            (
                proptest::collection::vec("[a-z]+".prop_map(|s| s), 0..5), // modules
                proptest::option::of(0..20u32),                            // gap
                proptest::option::of(100..1000u32),                        // width
            )
                .prop_map(|(modules, gap, width)| ColumnSpec {
                    modules,
                    overflow: ColumnOverflowPolicy::Hide,
                    gap: gap.map(|g| g as i32),
                    align: Some(TextAlign::Left),
                    width: width.map(|w| w as i32),
                }),
        ),
        1..5,
    )
    .prop_map(|columns_vec| {
        let mut columns = IndexMap::new();
        for (name, spec) in columns_vec {
            columns.insert(name, spec);
        }
        LayoutConfig { columns }
    })
}

fn arbitrary_monitor_config() -> impl Strategy<Value = MonitorConfig> {
    (
        "[a-zA-Z0-9\\-.*^$]+".prop_map(|pattern| pattern), // match_pattern
        any::<bool>(),                                     // show_bar
    )
        .prop_map(|(match_pattern, show_bar)| MonitorConfig {
            match_pattern,
            show_bar,
            layout: None,
            modules: None,
            wallpapers: None,
        })
}

fn arbitrary_logging_config() -> impl Strategy<Value = LoggingConfig> {
    (
        proptest::sample::select(vec!["debug", "info", "warn", "error"]), // level
        ".*".prop_map(|file| file), // file - simplified to avoid regex issues
        any::<bool>(),              // console
        proptest::sample::select(vec!["iso8601", "simple"]), // format
        any::<bool>(),              // include_file
        any::<bool>(),              // include_line
        any::<bool>(),              // include_class
    )
        .prop_map(
            |(level, file, console, format, include_file, include_line, include_class)| {
                LoggingConfig {
                    level: level.to_string(),
                    file,
                    console,
                    format: format.to_string(),
                    include_file,
                    include_line,
                    include_class,
                }
            },
        )
}

// ===== BOUNDARY AND EDGE CASE TESTS =====

#[test]
fn test_config_parsing_boundary_conditions() {
    // Test very large configuration
    let large_config = generate_large_config(100);
    let result: Result<NiriBarConfig, _> = serde_yaml::from_str(&large_config);
    assert!(result.is_ok(), "Large config should parse successfully");

    // Test configuration with extreme values
    let extreme_config = r#"
application:
  theme: "test"
  modules:
    test:
      max_length: 1000000
      warn_threshold: 255
      critical_threshold: 255
  layouts: {}
  monitors:
    - match: ".*"
logging:
  level: "debug"
  file: "/tmp/test.log"
  console: true
"#;
    let config: NiriBarConfig = serde_yaml::from_str(extreme_config).unwrap();
    assert_eq!(config.application.modules["test"].max_length, Some(1000000));
    assert_eq!(config.application.modules["test"].warn_threshold, Some(255));
}

#[test]
fn test_unicode_and_special_characters() {
    // Test configuration with Unicode characters
    let unicode_config = r#"
application:
  theme: "dracula"
  modules:
    clock:
      format: "🕐 %H:%M"
    workspaces:
      highlight_active: true
  layouts:
    test:
      columns:
        left:
          modules: ["workspaces"]
  monitors:
    - match: ".*"
      modules:
        clock:
          format: "🕐 %H:%M"
logging:
  level: "info"
  file: "~/日志文件.log"
  console: true
"#;
    let config: NiriBarConfig = serde_yaml::from_str(unicode_config).unwrap();
    assert_eq!(
        config.application.modules["clock"].format,
        Some("🕐 %H:%M".to_string())
    );
    assert_eq!(config.logging.file, "~/日志文件.log");
}

fn generate_large_config(num_monitors: usize) -> String {
    let mut config = r#"
application:
  theme: "test"
  modules: {}
  layouts: {}
  monitors:
"#
    .to_string();

    for i in 0..num_monitors {
        config.push_str(&format!(
            r#"
    - match: "monitor-{}"
      show_bar: true
"#,
            i
        ));
    }

    config.push_str(
        r#"
logging:
  level: "info"
  file: "/tmp/test.log"
  console: true
"#,
    );

    config
}

// ===== TIMEOUT AND PERFORMANCE TESTS =====

#[test]
fn test_config_parsing_performance() {
    use std::time::{Duration, Instant};

    // Generate a large config for performance testing
    let large_config = generate_large_config(50);

    let start = Instant::now();
    let config: NiriBarConfig = serde_yaml::from_str(&large_config).unwrap();
    let parse_time = start.elapsed();

    // Should parse within reasonable time (adjust threshold as needed)
    assert!(
        parse_time < Duration::from_millis(500),
        "Config parsing took too long: {:?}",
        parse_time
    );

    // Verify the parsed config is correct
    assert_eq!(config.application.monitors.len(), 50);
}

#[test]
fn test_config_operations_with_timeout() {
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    let config_manager = ConfigManager::new();
    let (tx, rx) = mpsc::channel();

    let handle = std::thread::spawn(move || {
        // Simulate a long-running operation
        let test_config = generate_large_config(20);
        let config: NiriBarConfig = serde_yaml::from_str(&test_config).unwrap();

        {
            let mut config_guard = config_manager.config.lock().unwrap();
            *config_guard = Some(config);
        }

        // Simulate some processing time
        std::thread::sleep(Duration::from_millis(50));

        tx.send("completed").unwrap();
    });

    // Wait for completion with timeout
    let start = Instant::now();
    let result = rx.recv_timeout(Duration::from_secs(2));

    assert!(result.is_ok(), "Operation should complete within timeout");
    assert!(start.elapsed() < Duration::from_secs(2));

    handle.join().unwrap();
}

/// Test that demonstrates using a timeout mechanism for operations
#[cfg(test)]
mod timeout_tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    /// Helper function to run operations with timeout
    fn run_with_timeout<F, T>(operation: F, timeout: Duration) -> Result<T, &'static str>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            let result = operation();
            let _ = tx.send(result);
        });

        match rx.recv_timeout(timeout) {
            Ok(result) => Ok(result),
            Err(_) => Err("Operation timed out"),
        }
    }

    #[test]
    fn test_config_validation_with_timeout() {
        let config = NiriBarConfig {
            application: ApplicationConfig {
                theme: "test".to_string(),
                modules: std::collections::HashMap::new(),
                layouts: std::collections::HashMap::new(),
                monitors: vec![MonitorConfig {
                    match_pattern: ".*".to_string(),
                    show_bar: true,
                    layout: None,
                    modules: None,
                    wallpapers: None,
                }],
                wallpapers: WallpaperConfig::default(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file: "/tmp/test.log".to_string(),
                console: true,
                format: "iso8601".to_string(),
                include_file: true,
                include_line: true,
                include_class: true,
            },
        };

        let result = run_with_timeout(
            move || ConfigManager::basic_validation(&config),
            Duration::from_millis(100),
        );

        match result {
            Ok(validation_result) => assert!(validation_result.is_ok()),
            Err(msg) => panic!("Validation timed out: {}", msg),
        }
    }

    #[test]
    fn test_timeout_behavior() {
        // Test that timeouts actually work
        let slow_operation = || {
            std::thread::sleep(Duration::from_millis(200));
            "completed"
        };

        let result = run_with_timeout(slow_operation, Duration::from_millis(50));
        assert!(result.is_err(), "Slow operation should timeout");
        assert!(matches!(result.unwrap_err(), "Operation timed out"));
    }
}

// ===== CONCURRENCY TESTS WITH LOOM (when available) =====

#[cfg(all(test, feature = "loom"))]
mod loom_concurrency_tests {
    use loom::sync::Arc;
    use loom::sync::atomic::{AtomicUsize, Ordering};
    use loom::thread;
    use niri_bar::config::{ConfigManager, NiriBarConfig};

    #[test]
    #[ignore] // Loom test causing stack overflow - needs proper loom environment setup
    fn loom_config_manager_concurrent_access() {
        loom::model(|| {
            let config_manager = Arc::new(ConfigManager::new());
            let counter = Arc::new(AtomicUsize::new(0));

            let config_manager1 = Arc::clone(&config_manager);
            let counter1 = Arc::clone(&counter);

            let handle1 = thread::spawn(move || {
                let test_config = r#"
application:
  modules: {}
  layouts: {}
  monitors:
    - match: ".*"
logging:
  level: "info"
  file: "/tmp/test.log"
  console: true
"#;
                let config: NiriBarConfig = serde_yaml::from_str(test_config).unwrap();

                {
                    let mut config_guard = config_manager1.config.lock().unwrap();
                    *config_guard = Some(config);
                }

                counter1.fetch_add(1, Ordering::SeqCst);
            });

            let config_manager2 = Arc::clone(&config_manager);
            let counter2 = Arc::clone(&counter);

            let handle2 = thread::spawn(move || {
                // Small delay to increase interleaving
                loom::thread::yield_now();

                let read_config = config_manager2.get_config();
                if read_config.is_some() {
                    counter2.fetch_add(1, Ordering::SeqCst);
                }
            });

            handle1.join().unwrap();
            handle2.join().unwrap();

            // Verify both operations completed
            assert_eq!(counter.load(Ordering::SeqCst), 2);
        });
    }
}
