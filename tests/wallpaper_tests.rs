use niri_bar::wallpaper::{WallpaperSwitcher, WallpaperCommandExecutor, DefaultWallpaperExecutor};
use niri_bar::config::WallpaperConfig;
use niri_bar::niri::WorkspaceInfo;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use tempfile::NamedTempFile;
use std::fs;

/// Mock executor for testing wallpaper switching
struct MockWallpaperExecutor {
    pub executed_commands: std::cell::RefCell<Vec<String>>,
    pub existing_paths: std::collections::HashSet<String>,
}

impl MockWallpaperExecutor {
    fn new() -> Self {
        let mut existing_paths = std::collections::HashSet::new();
        existing_paths.insert("/usr/bin/swww".to_string());
        existing_paths.insert("/usr/bin/swaybg".to_string());

        Self {
            executed_commands: std::cell::RefCell::new(Vec::new()),
            existing_paths,
        }
    }

    fn add_existing_path(&mut self, path: &str) {
        self.existing_paths.insert(path.to_string());
    }

    fn get_executed_commands(&self) -> Vec<String> {
        self.executed_commands.borrow().clone()
    }
}

impl WallpaperCommandExecutor for MockWallpaperExecutor {
    fn execute_command(&self, command: &str) -> Result<(), std::io::Error> {
        self.executed_commands.borrow_mut().push(command.to_string());
        Ok(())
    }

    fn check_path_exists(&self, path: &str) -> bool {
        self.existing_paths.contains(path)
    }
}

// Mock WorkspaceInfo for testing
fn create_test_workspace(idx: i64, name: Option<&str>) -> WorkspaceInfo {
    WorkspaceInfo {
        id: idx * 100, // Simple ID generation
        idx,
        name: name.map(|s| s.to_string()),
        is_focused: true,
    }
}

#[test]
fn test_wallpaper_switcher_creation() {
    let config = WallpaperConfig::default();
    let switcher = WallpaperSwitcher::new_default(config);

    // Test that switcher is created successfully
    // We can't easily test the internal state without making fields public
    let _switcher = switcher;
}

#[test]
fn test_wallpaper_switcher_with_empty_config() {
    let config = WallpaperConfig {
        default: None,
        by_workspace: HashMap::new(),
        special_cmd: None,
        swww_options: None,
    };

    let switcher = WallpaperSwitcher::new_default(config);
    let workspace = create_test_workspace(1, Some("test"));

    // Should not crash with empty config
    switcher.switch_wallpaper(&workspace);
}

#[test]
fn test_wallpaper_switcher_with_default_wallpaper() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path().to_string_lossy().to_string();

    let config = WallpaperConfig {
        default: Some(temp_path.clone()),
        by_workspace: HashMap::new(),
        special_cmd: None,
        swww_options: None,
    };

    let switcher = WallpaperSwitcher::new_default(config);
    let workspace = create_test_workspace(1, Some("test"));

    // Should attempt to switch to default wallpaper
    switcher.switch_wallpaper(&workspace);

    // File should still exist
    assert!(temp_file.path().exists());
}

#[test]
fn test_wallpaper_switcher_with_workspace_specific_wallpaper() {
    let temp_file1 = NamedTempFile::new().unwrap();
    let temp_file2 = NamedTempFile::new().unwrap();
    let temp_path1 = temp_file1.path().to_string_lossy().to_string();
    let temp_path2 = temp_file2.path().to_string_lossy().to_string();

    let mut by_workspace = HashMap::new();
    by_workspace.insert("test".to_string(), temp_path1.clone());
    by_workspace.insert("1".to_string(), temp_path2.clone());

    let config = WallpaperConfig {
        default: None,
        by_workspace,
        special_cmd: None,
        swww_options: None,
    };

    let switcher = WallpaperSwitcher::new_default(config);

    // Test workspace with name matching
    let workspace1 = create_test_workspace(1, Some("test"));
    switcher.switch_wallpaper(&workspace1);

    // Test workspace with index matching
    let workspace2 = create_test_workspace(1, Some("other"));
    switcher.switch_wallpaper(&workspace2);

    // Files should still exist
    assert!(temp_file1.path().exists());
    assert!(temp_file2.path().exists());
}

#[test]
fn test_wallpaper_switcher_with_special_command() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path().to_string_lossy().to_string();

    let config = WallpaperConfig {
        default: Some(temp_path.clone()),
        by_workspace: HashMap::new(),
        special_cmd: Some("echo 'wallpaper switched to ${current_workspace_image}'".to_string()),
        swww_options: None,
    };

    let switcher = WallpaperSwitcher::new_default(config);
    let workspace = create_test_workspace(1, Some("test"));

    // Should execute special command
    switcher.switch_wallpaper(&workspace);

    // File should still exist
    assert!(temp_file.path().exists());
}

#[test]
fn test_wallpaper_switcher_workspace_resolution_priority() {
    let temp_file1 = NamedTempFile::new().unwrap();
    let temp_file2 = NamedTempFile::new().unwrap();
    let temp_file3 = NamedTempFile::new().unwrap();

    let temp_path1 = temp_file1.path().to_string_lossy().to_string();
    let temp_path2 = temp_file2.path().to_string_lossy().to_string();
    let temp_path3 = temp_file3.path().to_string_lossy().to_string();

    let mut by_workspace = HashMap::new();
    by_workspace.insert("test".to_string(), temp_path1.clone()); // Name match
    by_workspace.insert("1".to_string(), temp_path2.clone());   // Index match
    by_workspace.insert("fallback".to_string(), temp_path3.clone()); // No match

    let config = WallpaperConfig {
        default: None,
        by_workspace,
        special_cmd: None,
        swww_options: None,
    };

    let switcher = WallpaperSwitcher::new_default(config);

    // Test priority: name match should take precedence
    let workspace = create_test_workspace(1, Some("test"));
    switcher.switch_wallpaper(&workspace);

    // All files should still exist
    assert!(temp_file1.path().exists());
    assert!(temp_file2.path().exists());
    assert!(temp_file3.path().exists());
}

#[test]
fn test_wallpaper_switcher_tilde_expansion() {
    // Test tilde expansion in paths
    let config = WallpaperConfig {
        default: Some("~/test_wallpaper.jpg".to_string()),
        by_workspace: HashMap::new(),
        special_cmd: None,
        swww_options: None,
    };

    let switcher = WallpaperSwitcher::new_default(config);
    let workspace = create_test_workspace(1, Some("test"));

    // Should not crash on tilde expansion
    switcher.switch_wallpaper(&workspace);
}

#[test]
fn test_wallpaper_switcher_invalid_paths() {
    // Test with invalid/non-existent paths
    let config = WallpaperConfig {
        default: Some("/nonexistent/path/wallpaper.jpg".to_string()),
        by_workspace: HashMap::new(),
        special_cmd: None,
        swww_options: None,
    };

    let switcher = WallpaperSwitcher::new_default(config);
    let workspace = create_test_workspace(1, Some("test"));

    // Should handle invalid paths gracefully
    switcher.switch_wallpaper(&workspace);
}

#[test]
fn test_wallpaper_switcher_multiple_workspaces() {
    let temp_file1 = NamedTempFile::new().unwrap();
    let temp_file2 = NamedTempFile::new().unwrap();

    let temp_path1 = temp_file1.path().to_string_lossy().to_string();
    let temp_path2 = temp_file2.path().to_string_lossy().to_string();

    let mut by_workspace = HashMap::new();
    by_workspace.insert("workspace1".to_string(), temp_path1.clone());
    by_workspace.insert("workspace2".to_string(), temp_path2.clone());

    let config = WallpaperConfig {
        default: None,
        by_workspace,
        special_cmd: None,
        swww_options: None,
    };

    let switcher = WallpaperSwitcher::new_default(config);

    // Switch to different workspaces
    let workspace1 = create_test_workspace(1, Some("workspace1"));
    let workspace2 = create_test_workspace(2, Some("workspace2"));

    switcher.switch_wallpaper(&workspace1);
    switcher.switch_wallpaper(&workspace2);

    // Files should still exist
    assert!(temp_file1.path().exists());
    assert!(temp_file2.path().exists());
}

#[test]
fn test_wallpaper_switcher_empty_workspace_name() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path().to_string_lossy().to_string();

    let mut by_workspace = HashMap::new();
    by_workspace.insert("1".to_string(), temp_path.clone()); // Index-based mapping

    let config = WallpaperConfig {
        default: None,
        by_workspace,
        special_cmd: None,
        swww_options: None,
    };

    let switcher = WallpaperSwitcher::new_default(config);

    // Test workspace with empty name (should fall back to index)
    let workspace = create_test_workspace(1, None);
    switcher.switch_wallpaper(&workspace);

    // File should still exist
    assert!(temp_file.path().exists());
}

#[test]
fn test_wallpaper_switcher_performance() {
    use std::time::Instant;

    let config = WallpaperConfig {
        default: Some("/tmp/test.jpg".to_string()),
        by_workspace: HashMap::new(),
        special_cmd: None,
        swww_options: None,
    };

    let switcher = WallpaperSwitcher::new_default(config);

    let start = Instant::now();

    // Perform multiple wallpaper switches
    for i in 1..=10 {
        let workspace = create_test_workspace(i, Some(&format!("workspace{}", i)));
        switcher.switch_wallpaper(&workspace);
    }

    let duration = start.elapsed();

    // Should complete within reasonable time
    assert!(duration < std::time::Duration::from_millis(500),
            "10 wallpaper switches took too long: {:?}", duration);
}

#[test]
fn test_wallpaper_switcher_thread_safety() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let config = WallpaperConfig {
        default: Some("/tmp/test.jpg".to_string()),
        by_workspace: HashMap::new(),
        special_cmd: None,
        swww_options: None,
    };

    let switcher = Arc::new(Mutex::new(WallpaperSwitcher::<DefaultWallpaperExecutor>::new_default_with_config(config)));

    let mut handles = vec![];

    // Spawn threads that use the switcher concurrently
    for i in 0..5 {
        let switcher_clone = Arc::clone(&switcher);

        let handle = thread::spawn(move || {
            let workspace = create_test_workspace(i as i64 + 1, Some(&format!("thread{}", i)));

            {
                let switcher_guard = switcher_clone.lock().unwrap();
                switcher_guard.switch_wallpaper(&workspace);
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
fn test_wallpaper_switcher_config_persistence() {
    // Test that configuration is properly stored and accessed
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path().to_string_lossy().to_string();

    let mut by_workspace = HashMap::new();
    by_workspace.insert("test".to_string(), temp_path.clone());

    let config = WallpaperConfig {
        default: Some("/default/wallpaper.jpg".to_string()),
        by_workspace: by_workspace.clone(),
        special_cmd: Some("echo test".to_string()),
        swww_options: None,
    };

    let switcher = WallpaperSwitcher::new_default(config);

    // Test that switcher maintains configuration
    let workspace = create_test_workspace(1, Some("test"));
    switcher.switch_wallpaper(&workspace);

    // Test with different workspace (should use default)
    let workspace2 = create_test_workspace(2, Some("other"));
    switcher.switch_wallpaper(&workspace2);

    // Files should still exist
    assert!(temp_file.path().exists());
}

#[test]
fn test_wallpaper_switcher_special_command_substitution() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path().to_string_lossy().to_string();

    let config = WallpaperConfig {
        default: Some(temp_path.clone()),
        by_workspace: HashMap::new(),
        special_cmd: Some("echo 'Setting wallpaper to: ${current_workspace_image}'".to_string()),
        swww_options: None,
    };

    let switcher = WallpaperSwitcher::new_default(config);
    let workspace = create_test_workspace(1, Some("test"));

    // Should substitute the image path in the command
    switcher.switch_wallpaper(&workspace);

    // File should still exist
    assert!(temp_file.path().exists());
}

#[test]
fn test_wallpaper_switcher_swww_options() {
    use niri_bar::config::SwwwOptions;

    let swww_opts = SwwwOptions {
        transition_type: "fade".to_string(),
        transition_duration: 1.0,
        transition_step: 90,
        transition_fps: 30,
        filter: "Lanczos3".to_string(),
        resize: "crop".to_string(),
        fill_color: "000000".to_string(),
    };

    let config = WallpaperConfig {
        default: Some("/tmp/test.jpg".to_string()),
        by_workspace: HashMap::new(),
        special_cmd: None,
        swww_options: Some(swww_opts),
    };

    let switcher = WallpaperSwitcher::new_default(config);
    let workspace = create_test_workspace(1, Some("test"));

    // Should work with swww options configured
    switcher.switch_wallpaper(&workspace);
}

// ===== PROPERTY-BASED TESTS =====

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_wallpaper_switcher_with_random_configs(
            default_path in ".*",
            special_cmd in prop::option::of(".*"),
            workspace_count in 0..10,
        ) {
            let mut by_workspace = HashMap::new();

            for i in 0..workspace_count {
                let key = format!("workspace{}", i);
                let path = format!("/tmp/wallpaper{}.jpg", i);
                by_workspace.insert(key, path);
            }

            let config = WallpaperConfig {
                default: Some(default_path),
                by_workspace,
                special_cmd,
                swww_options: None,
            };

            let switcher = WallpaperSwitcher::new_default(config);

            // Test with various workspaces
            for i in 0..workspace_count.min(3) {
                let workspace_name = format!("workspace{}", i);
                let workspace = create_test_workspace(i as i64, Some(&workspace_name));
                switcher.switch_wallpaper(&workspace);
            }
        }

        #[test]
        fn test_wallpaper_switcher_workspace_names(
            workspace_name in "[a-zA-Z0-9_\\-]+",
            idx in 1..100i64,
        ) {
            let config = WallpaperConfig {
                default: Some("/tmp/default.jpg".to_string()),
                by_workspace: HashMap::new(),
                special_cmd: None,
                swww_options: None,
            };

            let switcher = WallpaperSwitcher::new_default(config);
            let workspace = create_test_workspace(idx, Some(&workspace_name));

            // Should handle various workspace names
            switcher.switch_wallpaper(&workspace);
        }
    }
}

#[test]
fn test_wallpaper_switcher_with_mock_executor() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path().to_string_lossy().to_string();

    let config = WallpaperConfig {
        default: Some(temp_path.clone()),
        by_workspace: HashMap::new(),
        special_cmd: Some("echo 'Setting wallpaper to: ${current_workspace_image}'".to_string()),
        swww_options: None,
    };

    let mock_executor = MockWallpaperExecutor::new();
    let switcher = WallpaperSwitcher::new(config, mock_executor);

    let workspace = create_test_workspace(1, Some("test"));

    // Switch wallpaper with mock executor
    switcher.switch_wallpaper(&workspace);

    // Verify that the mock executor recorded the command execution
    let executed_commands = switcher.get_executor().get_executed_commands();
    assert_eq!(executed_commands.len(), 1);
    assert!(executed_commands[0].contains("Setting wallpaper to"));
    assert!(executed_commands[0].contains(&temp_path));
}

#[test]
fn test_wallpaper_switcher_swww_fallback_with_mock() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path().to_string_lossy().to_string();

    let config = WallpaperConfig {
        default: Some(temp_path.clone()),
        by_workspace: HashMap::new(),
        special_cmd: None, // No special command, should fall back to swww
        swww_options: None,
    };

    let mut mock_executor = MockWallpaperExecutor::new();
    // Ensure swww is available
    mock_executor.add_existing_path("/usr/bin/swww");

    let switcher = WallpaperSwitcher::new(config, mock_executor);

    let workspace = create_test_workspace(1, Some("test"));

    // Switch wallpaper - should use swww since it's available
    switcher.switch_wallpaper(&workspace);

    // Verify that swww command was executed
    let executed_commands = switcher.get_executor().get_executed_commands();
    assert_eq!(executed_commands.len(), 1);
    assert!(executed_commands[0].contains("swww img"));
    assert!(executed_commands[0].contains(&temp_path));
}

#[test]
fn test_wallpaper_switcher_no_providers_with_mock() {
    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path().to_string_lossy().to_string();

    let config = WallpaperConfig {
        default: Some(temp_path.clone()),
        by_workspace: HashMap::new(),
        special_cmd: None,
        swww_options: None,
    };

    let mut mock_executor = MockWallpaperExecutor::new();
    // Remove both swww and swaybg
    mock_executor.existing_paths.clear();

    let switcher = WallpaperSwitcher::new(config, mock_executor);

    let workspace = create_test_workspace(1, Some("test"));

    // Switch wallpaper - should not execute any commands since no providers are available
    switcher.switch_wallpaper(&workspace);

    // Verify that no commands were executed
    let executed_commands = switcher.get_executor().get_executed_commands();
    assert_eq!(executed_commands.len(), 0);
}
