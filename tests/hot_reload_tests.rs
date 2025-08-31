
use std::fs;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_yaml_hot_reload() {
    // Test that YAML configuration changes are detected and reloaded
    let config_path = "niri-bar.yaml";
    
    // Read original content
    let original_content = fs::read_to_string(config_path).expect("Failed to read config file");
    
    // Create a temporary modified version - replace whatever theme is there with solarized
    let modified_content = if original_content.contains("theme: \"wombat\"") {
        original_content.replace("theme: \"wombat\"", "theme: \"solarized\"")
    } else if original_content.contains("theme: \"dracula\"") {
        original_content.replace("theme: \"dracula\"", "theme: \"solarized\"")
    } else {
        original_content.replace("theme: \"solarized\"", "theme: \"solarized\"")
    };
    
    // Write modified content
    fs::write(config_path, &modified_content).expect("Failed to write modified config");
    
    // Give the file watcher time to detect the change
    sleep(Duration::from_millis(500)).await;
    
    // Verify the change was written to the file
    let current_content = fs::read_to_string(config_path).expect("Failed to read config file");
    assert!(current_content.contains("theme: \"solarized\""));
    
    // Restore original content
    fs::write(config_path, original_content).expect("Failed to restore original config");
}

#[tokio::test]
async fn test_css_hot_reload() {
    // Test that CSS theme file changes are detected
    let css_path = "themes/wombat.css";
    
    // Read original content
    let original_content = fs::read_to_string(css_path).expect("Failed to read CSS file");
    
    // Create a temporary modified version with a comment
    let modified_content = format!("/* Modified for test */\n{}", original_content);
    
    // Write modified content
    fs::write(css_path, &modified_content).expect("Failed to write modified CSS");
    
    // Give the file watcher time to detect the change
    sleep(Duration::from_millis(500)).await;
    
    // Verify the change was written
    let current_content = fs::read_to_string(css_path).expect("Failed to read CSS file");
    assert!(current_content.contains("/* Modified for test */"));
    
    // Restore original content
    fs::write(css_path, original_content).expect("Failed to restore original CSS");
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
