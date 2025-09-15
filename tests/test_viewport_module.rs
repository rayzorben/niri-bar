use niri_bar::config::ModuleConfig;
use niri_bar::modules::viewport::ViewportModule;
use niri_bar::niri::niri_bus;
use gtk4::prelude::*;
use std::collections::HashMap;

#[test]
fn test_viewport_module_creation() {
    gtk4::init().expect("Failed to initialize GTK");
    
    let config = ModuleConfig::default();
    let widget = ViewportModule::create_widget(&config);
    
    assert!(widget.has_css_class("module-viewport"));
}

#[test]
fn test_viewport_module_with_custom_config() {
    gtk4::init().expect("Failed to initialize GTK");
    
    let mut additional = HashMap::new();
    additional.insert("update_rate_ms".to_string(), serde_yaml::Value::Number(16.into()));
    
    let config = ModuleConfig {
        show_window_titles: Some(false),
        highlight_focused: Some(false),
        additional,
        ..Default::default()
    };
    
    let widget = ViewportModule::create_widget(&config);
    assert!(widget.has_css_class("module-viewport"));
}

#[test]
fn test_viewport_module_identifier() {
    assert_eq!(ViewportModule::IDENT, "bar.module.viewport");
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn setup_test_workspace_data() {
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
    }

    #[test]
    fn test_workspace_window_filtering() {
        setup_test_workspace_data();
        
        let bus = niri_bus();
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
        setup_test_workspace_data();
        
        let bus = niri_bus();
        let focused_workspace_id = bus.focused_workspace_id();
        
        assert_eq!(focused_workspace_id, Some(1));
    }

    #[test]
    fn test_window_layout_parsing() {
        setup_test_workspace_data();
        
        let bus = niri_bus();
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
        setup_test_workspace_data();
        
        let bus = niri_bus();
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
        gtk4::init().expect("Failed to initialize GTK");
        
        let config = ModuleConfig::default();
        let widget = ViewportModule::create_widget(&config);
        
        // Test that the widget has the correct CSS classes
        assert!(widget.has_css_class("module-viewport"));
        
        // Test that it contains a drawing area
        let box_widget = widget.downcast::<gtk4::Box>().expect("Widget should be a Box");
        assert_eq!(box_widget.orientation(), gtk4::Orientation::Horizontal);
    }

    #[test]
    fn test_viewport_sizing() {
        gtk4::init().expect("Failed to initialize GTK");
        
        let config = ModuleConfig::default();
        let widget = ViewportModule::create_widget(&config);
        
        let box_widget = widget.downcast::<gtk4::Box>().expect("Widget should be a Box");
        
        // The viewport should expand vertically to match bar height
        assert!(box_widget.vexpands());
        // But not expand horizontally (fixed aspect ratio)
        assert!(!box_widget.hexpands());
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
    use serde_yaml;

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
        additional:
          update_rate_ms: 16
        "#;
        
        let config: ModuleConfig = serde_yaml::from_str(yaml).expect("Failed to deserialize config");
        assert_eq!(config.show_window_titles, Some(false));
        assert_eq!(config.highlight_focused, Some(true));
        
        let update_rate = config.additional.get("update_rate_ms")
            .and_then(|v| v.as_u64())
            .expect("update_rate_ms should be present");
        assert_eq!(update_rate, 16);
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
        setup_test_workspace_data();
        
        let bus = niri_bus();
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
        
        let windows = bus.windows_for_workspace(1);
        assert_eq!(windows.len(), 0);
    }

    #[test]
    fn test_viewport_with_malformed_window_data() {
        let bus = niri_bus();
        bus.reset();
        
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
        gtk4::init().expect("Failed to initialize GTK");
        
        let config = ModuleConfig::default();
        
        let start = Instant::now();
        let _widget = ViewportModule::create_widget(&config);
        let duration = start.elapsed();
        
        // Viewport creation should be fast (< 100ms)
        assert!(duration.as_millis() < 100, "Viewport creation took too long: {:?}", duration);
    }

    #[test]
    fn test_window_filtering_performance() {
        setup_test_workspace_data();
        
        let bus = niri_bus();
        
        let start = Instant::now();
        for _ in 0..1000 {
            let _windows = bus.windows_for_workspace(1);
        }
        let duration = start.elapsed();
        
        // 1000 filter operations should complete quickly (< 10ms)
        assert!(duration.as_millis() < 10, "Window filtering too slow: {:?}", duration);
    }
}

/// Helper function to set up test workspace data for integration tests
fn setup_test_workspace_data() {
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
}
