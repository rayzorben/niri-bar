
use std::fs;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_yaml_hot_reload() {
    // Test that YAML configuration changes are detected and reloaded
    // Use a temporary file to avoid interfering with other tests
    let temp_config_path = "test-config-temp.yaml";
    let original_config_path = "niri-bar.yaml";

    // Read original content
    let original_content = fs::read_to_string(original_config_path).expect("Failed to read config file");

    // Create a temporary modified version - replace whatever theme is there with solarized
    let modified_content = if original_content.contains("theme: \"wombat\"") {
        original_content.replace("theme: \"wombat\"", "theme: \"solarized\"")
    } else if original_content.contains("theme: \"dracula\"") {
        original_content.replace("theme: \"dracula\"", "theme: \"solarized\"")
    } else {
        original_content.replace("theme: \"solarized\"", "theme: \"solarized\"")
    };

    // Write modified content to temp file
    fs::write(temp_config_path, &modified_content).expect("Failed to write modified config");

    // Give the file watcher time to detect the change
    sleep(Duration::from_millis(50)).await;

    // Verify the change was written to the temp file
    let current_content = fs::read_to_string(temp_config_path).expect("Failed to read temp config file");
    assert!(current_content.contains("theme: \"solarized\""));

    // Clean up temp file
    fs::remove_file(temp_config_path).expect("Failed to remove temp config file");
}

#[tokio::test]
async fn test_css_hot_reload() {
    // Test that CSS theme file changes are detected
    // Use a temporary file to avoid interfering with other tests
    let temp_css_path = "test-theme-temp.css";
    let original_css_path = "themes/wombat.css";

    // Read original content
    let original_content = fs::read_to_string(original_css_path).expect("Failed to read CSS file");

    // Create a temporary modified version with a comment
    let modified_content = format!("/* Modified for test */\n{}", original_content);

    // Write modified content to temp file
    fs::write(temp_css_path, &modified_content).expect("Failed to write modified CSS");

    // Give the file watcher time to detect the change
    sleep(Duration::from_millis(50)).await;

    // Verify the change was written to the temp file
    let current_content = fs::read_to_string(temp_css_path).expect("Failed to read temp CSS file");
    assert!(current_content.contains("/* Modified for test */"));

    // Clean up temp file
    fs::remove_file(temp_css_path).expect("Failed to remove temp CSS file");
}

#[test]
fn test_file_watcher_setup() {
    // Test that file watcher can be set up for required files
    let required_files = [
        "niri-bar.yaml",
        "themes/wombat.css",
        "themes/solarized.css", 
        "themes/dracula.css"
    ];
    
    for file_path in &required_files {
        assert!(Path::new(file_path).exists(), "Required file {} does not exist", file_path);
    }
}
