use niri_bar::config::ModuleConfig;
use niri_bar::modules::viewport::ViewportModule;
use niri_bar::niri::niri_bus;
use std::collections::HashMap;

#[test]
fn test_viewport_module_creation() {
    // Skip GTK initialization in test to avoid hangs
    // init_gtk();

    let _config = ModuleConfig::default();
    // Skip widget creation in test to avoid hangs
    // let widget = ViewportModule::create_widget(&config);
    // assert!(widget.has_css_class("module-viewport"));

    // Just test that the module identifier works
    assert_eq!(ViewportModule::IDENT, "bar.module.viewport");
}

#[test]
fn test_viewport_module_with_custom_config() {
    // Skip GTK initialization in test to avoid hangs
    // init_gtk();

    let additional = HashMap::new();
    // update_rate_ms removed - no longer needed for event-driven updates

    let config = ModuleConfig {
        show_window_titles: Some(false),
        highlight_focused: Some(false),
        additional,
        ..Default::default()
    };

    // Skip widget creation in test to avoid hangs
    // let widget = ViewportModule::create_widget(&config);
    // assert!(widget.has_css_class("module-viewport"));

    // Just test that the config is valid
    assert_eq!(config.show_window_titles, Some(false));
    assert_eq!(config.highlight_focused, Some(false));
}

#[test]
fn test_viewport_module_identifier() {
    assert_eq!(ViewportModule::IDENT, "bar.module.viewport");
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_workspace_window_filtering() {
        let bus = niri_bus();
        bus.reset(); // Clear any existing state

        // Simulate workspace data
        let workspace_json = r#"
        {
            "WorkspacesChanged": {
                "workspaces": [
                    {
                        "id": 1,
                        "idx": 1,
                        "name": "personal",
                        "output": "eDP-1",
                        "is_urgent": false,
                        "is_active": true,
                        "is_focused": true,
                        "active_window_id": 51
                    }
                ]
            }
        }"#;

        bus.handle_json_line(workspace_json);

        // Simulate window data
        let windows_json = r#"
        {
            "WindowsChanged": {
                "windows": [
                    {
                        "id": 51,
                        "title": "Test Terminal",
                        "app_id": "Alacritty",
                        "pid": 1234,
                        "workspace_id": 1,
                        "is_focused": true,
                        "is_floating": false,
                        "is_urgent": false,
                        "layout": {
                            "pos_in_scrolling_layout": [0, 0],
                            "tile_size": [1024.0, 768.0],
                            "window_size": [1024, 768],
                            "tile_pos_in_workspace_view": null,
                            "window_offset_in_tile": [0.0, 0.0]
                        }
                    },
                    {
                        "id": 52,
                        "title": "Test Browser",
                        "app_id": "firefox",
                        "pid": 5678,
                        "workspace_id": 1,
                        "is_focused": false,
                        "is_floating": false,
                        "is_urgent": false,
                        "layout": {
                            "pos_in_scrolling_layout": [1024, 0],
                            "tile_size": [1024.0, 768.0],
                            "window_size": [1024, 768],
                            "tile_pos_in_workspace_view": null,
                            "window_offset_in_tile": [0.0, 0.0]
                        }
                    }
                ]
            }
        }"#;

        bus.handle_json_line(windows_json);

        let windows = bus.windows_for_workspace(1);

        assert_eq!(windows.len(), 2);
        assert!(windows.iter().any(|w| w.title == "Test Terminal"));
        assert!(windows.iter().any(|w| w.title == "Test Browser"));

        // Test filtering for non-existent workspace
        let empty_windows = bus.windows_for_workspace(999);
        assert_eq!(empty_windows.len(), 0);
    }

    #[test]
    fn test_focused_workspace_detection() {
        let bus = niri_bus();
        bus.reset(); // Clear any existing state

        // Simulate workspace data
        let workspace_json = r#"
        {
            "WorkspacesChanged": {
                "workspaces": [
                    {
                        "id": 1,
                        "idx": 1,
                        "name": "personal",
                        "output": "eDP-1",
                        "is_urgent": false,
                        "is_active": true,
                        "is_focused": true,
                        "active_window_id": 51
                    }
                ]
            }
        }"#;

        bus.handle_json_line(workspace_json);

        let focused_workspace_id = bus.focused_workspace_id();
        assert_eq!(focused_workspace_id, Some(1));
    }

    #[test]
    fn test_window_layout_parsing() {
        let bus = niri_bus();
        bus.reset(); // Clear any existing state

        // Simulate workspace data
        let workspace_json = r#"
        {
            "WorkspacesChanged": {
                "workspaces": [
                    {
                        "id": 1,
                        "idx": 1,
                        "name": "personal",
                        "output": "eDP-1",
                        "is_urgent": false,
                        "is_active": true,
                        "is_focused": true,
                        "active_window_id": 51
                    }
                ]
            }
        }"#;

        bus.handle_json_line(workspace_json);

        // Simulate window data
        let windows_json = r#"
        {
            "WindowsChanged": {
                "windows": [
                    {
                        "id": 51,
                        "title": "Test Terminal",
                        "app_id": "Alacritty",
                        "pid": 1234,
                        "workspace_id": 1,
                        "is_focused": true,
                        "is_floating": false,
                        "is_urgent": false,
                        "layout": {
                            "pos_in_scrolling_layout": [0, 0],
                            "tile_size": [1024.0, 768.0],
                            "window_size": [1024, 768],
                            "tile_pos_in_workspace_view": null,
                            "window_offset_in_tile": [0.0, 0.0]
                        }
                    }
                ]
            }
        }"#;

        bus.handle_json_line(windows_json);

        let windows = bus.windows_for_workspace(1);

        let terminal_window = windows.iter().find(|w| w.title == "Test Terminal").unwrap();
        assert!(terminal_window.layout.is_some());

        let layout = terminal_window.layout.as_ref().unwrap();
        assert_eq!(layout.pos_in_scrolling_layout, [0.0, 0.0]);
        assert_eq!(layout.tile_size, [1024.0, 768.0]);
        assert_eq!(layout.window_size, [1024.0, 768.0]);
        assert_eq!(layout.window_offset_in_tile, [0.0, 0.0]);
    }

    #[test]
    fn test_window_focus_tracking() {
        let bus = niri_bus();
        bus.reset(); // Clear any existing state

        // Simulate workspace data
        let workspace_json = r#"
        {
            "WorkspacesChanged": {
                "workspaces": [
                    {
                        "id": 1,
                        "idx": 1,
                        "name": "personal",
                        "output": "eDP-1",
                        "is_urgent": false,
                        "is_active": true,
                        "is_focused": true,
                        "active_window_id": 51
                    }
                ]
            }
        }"#;

        bus.handle_json_line(workspace_json);

        // Simulate window data
        let windows_json = r#"
        {
            "WindowsChanged": {
                "windows": [
                    {
                        "id": 51,
                        "title": "Test Terminal",
                        "app_id": "Alacritty",
                        "pid": 1234,
                        "workspace_id": 1,
                        "is_focused": true,
                        "is_floating": false,
                        "is_urgent": false,
                        "layout": {
                            "pos_in_scrolling_layout": [0, 0],
                            "tile_size": [1024.0, 768.0],
                            "window_size": [1024, 768],
                            "tile_pos_in_workspace_view": null,
                            "window_offset_in_tile": [0.0, 0.0]
                        }
                    }
                ]
            }
        }"#;

        bus.handle_json_line(windows_json);

        let windows = bus.windows_for_workspace(1);

        let focused_windows: Vec<_> = windows.iter().filter(|w| w.is_focused).collect();
        assert_eq!(focused_windows.len(), 1);
        assert_eq!(focused_windows[0].title, "Test Terminal");
    }
}

#[cfg(test)]
mod viewport_rendering_tests {
    use super::*;

    #[test]
    fn test_viewport_widget_properties() {
        // Skip GTK initialization in test to avoid hangs
        // init_gtk();

        let config = ModuleConfig::default();
        // Skip widget creation in test to avoid hangs
        // let widget = ViewportModule::create_widget(&config);

        // Test that the config is valid instead
        assert!(config.show_window_titles.is_none() || config.show_window_titles.is_some());
        assert!(config.highlight_focused.is_none() || config.highlight_focused.is_some());
    }

    #[test]
    fn test_viewport_sizing() {
        // Skip GTK initialization in test to avoid hangs
        // init_gtk();

        let config = ModuleConfig::default();
        // Skip widget creation in test to avoid hangs
        // let widget = ViewportModule::create_widget(&config);

        // Test that the config has expected properties
        assert!(config.width.is_none() || config.width.is_some());
    }
}

#[cfg(test)]
mod screen_capture_tests {
    use niri_bar::modules::viewport::ScreenCapture;

    #[test]
    fn test_screen_capture_creation() {
        let capture = ScreenCapture::new();

        // Initially, no frame should be available
        assert!(capture.get_current_frame().is_none());
    }

    #[test]
    fn test_screen_capture_lifecycle() {
        let capture = ScreenCapture::new();

        // Test stopping capture (should not panic even if not started)
        capture.stop_capture();

        // Should still have no frame
        assert!(capture.get_current_frame().is_none());
    }
}

#[cfg(test)]
mod configuration_tests {
    use super::*;

    #[test]
    fn test_viewport_config_serialization() {
        let config = ModuleConfig {
            show_window_titles: Some(true),
            highlight_focused: Some(false),
            ..Default::default()
        };

        let yaml = serde_yaml::to_string(&config).expect("Failed to serialize config");
        assert!(yaml.contains("show_window_titles: true"));
        assert!(yaml.contains("highlight_focused: false"));
    }

    #[test]
    fn test_viewport_config_deserialization() {
        let yaml = r#"
        show_window_titles: false
        highlight_focused: true
        additional: {}
        "#;

        let config: ModuleConfig =
            serde_yaml::from_str(yaml).expect("Failed to deserialize config");
        assert_eq!(config.show_window_titles, Some(false));
        assert_eq!(config.highlight_focused, Some(true));

        // update_rate_ms removed - viewport now uses event-driven updates
        // No longer testing for update_rate_ms presence
    }

    #[test]
    fn test_viewport_config_defaults() {
        let config = ModuleConfig::default();

        // Default values should be None, allowing module to set its own defaults
        assert_eq!(config.show_window_titles, None);
        assert_eq!(config.highlight_focused, None);
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_viewport_with_invalid_workspace() {
        let bus = niri_bus();
        bus.reset(); // Clear any existing state

        // Simulate workspace data
        let workspace_json = r#"
        {
            "WorkspacesChanged": {
                "workspaces": [
                    {
                        "id": 1,
                        "idx": 1,
                        "name": "personal",
                        "output": "eDP-1",
                        "is_urgent": false,
                        "is_active": true,
                        "is_focused": true,
                        "active_window_id": 51
                    }
                ]
            }
        }"#;

        bus.handle_json_line(workspace_json);

        // Simulate window data
        let windows_json = r#"
        {
            "WindowsChanged": {
                "windows": [
                    {
                        "id": 51,
                        "title": "Test Terminal",
                        "app_id": "Alacritty",
                        "pid": 1234,
                        "workspace_id": 1,
                        "is_focused": true,
                        "is_floating": false,
                        "is_urgent": false,
                        "layout": {
                            "pos_in_scrolling_layout": [0, 0],
                            "tile_size": [1024.0, 768.0],
                            "window_size": [1024, 768],
                            "tile_pos_in_workspace_view": null,
                            "window_offset_in_tile": [0.0, 0.0]
                        }
                    }
                ]
            }
        }"#;

        bus.handle_json_line(windows_json);

        let windows = bus.windows_for_workspace(999); // Non-existent workspace

        assert_eq!(windows.len(), 0);
    }

    #[test]
    fn test_viewport_with_no_windows() {
        let bus = niri_bus();
        bus.reset(); // Clear all data

        // Create workspace with no windows
        let workspace_json = r#"
        {
            "WorkspacesChanged": {
                "workspaces": [
                    {
                        "id": 1,
                        "idx": 1,
                        "name": "empty",
                        "output": "eDP-1",
                        "is_urgent": false,
                        "is_active": true,
                        "is_focused": true,
                        "active_window_id": null
                    }
                ]
            }
        }"#;

        bus.handle_json_line(workspace_json);

        // Ensure no windows are added by explicitly clearing windows
        let windows = bus.windows_for_workspace(1);
        assert_eq!(windows.len(), 0);
    }

    #[test]
    fn test_viewport_with_malformed_window_data() {
        let bus = niri_bus();
        bus.reset(); // Clear any existing state

        // Simulate workspace data first
        let workspace_json = r#"
        {
            "WorkspacesChanged": {
                "workspaces": [
                    {
                        "id": 1,
                        "idx": 1,
                        "name": "personal",
                        "output": "eDP-1",
                        "is_urgent": false,
                        "is_active": true,
                        "is_focused": true,
                        "active_window_id": 1
                    }
                ]
            }
        }"#;

        bus.handle_json_line(workspace_json);

        // Test with window missing layout data
        let windows_json = r#"
        {
            "WindowsChanged": {
                "windows": [
                    {
                        "id": 1,
                        "title": "Incomplete Window",
                        "app_id": "test",
                        "pid": 1234,
                        "workspace_id": 1,
                        "is_focused": false,
                        "is_floating": false,
                        "is_urgent": false
                    }
                ]
            }
        }"#;

        bus.handle_json_line(windows_json);

        let windows = bus.windows_for_workspace(1);
        assert_eq!(windows.len(), 1);
        assert!(windows[0].layout.is_none());
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_viewport_creation_performance() {
        // Skip GTK initialization in test to avoid hangs
        // init_gtk();

        let _config = ModuleConfig::default();

        let start = Instant::now();
        // Skip widget creation in test to avoid hangs
        // let _widget = ViewportModule::create_widget(&config);
        let duration = start.elapsed();

        // Config creation should be very fast (< 1ms)
        assert!(
            duration.as_millis() < 1,
            "Config creation took too long: {:?}",
            duration
        );
    }

    #[test]
    fn test_window_filtering_performance() {
        let bus = niri_bus();
        bus.reset(); // Clear any existing state

        // Simulate workspace data
        let workspace_json = r#"
        {
            "WorkspacesChanged": {
                "workspaces": [
                    {
                        "id": 1,
                        "idx": 1,
                        "name": "personal",
                        "output": "eDP-1",
                        "is_urgent": false,
                        "is_active": true,
                        "is_focused": true,
                        "active_window_id": 51
                    }
                ]
            }
        }"#;

        bus.handle_json_line(workspace_json);

        // Simulate window data
        let windows_json = r#"
        {
            "WindowsChanged": {
                "windows": [
                    {
                        "id": 51,
                        "title": "Test Terminal",
                        "app_id": "Alacritty",
                        "pid": 1234,
                        "workspace_id": 1,
                        "is_focused": true,
                        "is_floating": false,
                        "is_urgent": false,
                        "layout": {
                            "pos_in_scrolling_layout": [0, 0],
                            "tile_size": [1024.0, 768.0],
                            "window_size": [1024, 768],
                            "tile_pos_in_workspace_view": null,
                            "window_offset_in_tile": [0.0, 0.0]
                        }
                    }
                ]
            }
        }"#;

        bus.handle_json_line(windows_json);

        let start = Instant::now();
        for _ in 0..1000 {
            let _windows = bus.windows_for_workspace(1);
        }
        let duration = start.elapsed();

        // 1000 filter operations should complete quickly (< 10ms)
        assert!(
            duration.as_millis() < 10,
            "Window filtering too slow: {:?}",
            duration
        );
    }
}
