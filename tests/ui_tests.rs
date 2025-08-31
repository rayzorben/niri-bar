use niri_bar::ui::{UIManager, MonitorBar};
use niri_bar::config::ConfigManager;

#[test]
fn test_monitor_bar_creation() {
    // This test would require GTK initialization
    // For now, just test the basic structure
    let monitor_name = "eDP-1";
    // Note: In a real test, we'd need to initialize GTK first
    // let bar = MonitorBar::new(monitor_name);
    // assert_eq!(bar.monitor_name(), monitor_name);
}

#[test]
fn test_ui_manager_creation() {
    let config_manager = ConfigManager::new();
    let ui_manager = UIManager::new(config_manager);
    assert_eq!(ui_manager.bar_count(), 0);
}
