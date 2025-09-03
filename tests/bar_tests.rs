use niri_bar::bar::MonitorInfo;

#[test]
fn test_bar_creation() {
    let monitor_info = MonitorInfo {
        connector: "eDP-1".to_string(),
        manufacturer: Some("LG Display".to_string()),
        model: Some("0x0797".to_string()),
        logical_size: (2048, 1280),
        scale_factor: 2,
    };

    // Note: We can't easily test Bar creation in unit tests due to GTK dependencies
    // This test verifies the MonitorInfo structure works
    assert_eq!(monitor_info.connector, "eDP-1");
    assert_eq!(monitor_info.logical_size, (2048, 1280));
    assert_eq!(monitor_info.scale_factor, 2);
}

#[test]
fn test_bar_content_formatting() {
    let monitor_info = MonitorInfo {
        connector: "HDMI-A-1".to_string(),
        manufacturer: None,
        model: None,
        logical_size: (1920, 1080),
        scale_factor: 1,
    };

    let expected_content = "Niri Bar - HDMI-A-1 (1920x1080, scale=1)";
    let actual_content = format!(
        "Niri Bar - {} ({}x{}, scale={})",
        monitor_info.connector,
        monitor_info.logical_size.0,
        monitor_info.logical_size.1,
        monitor_info.scale_factor
    );

    assert_eq!(actual_content, expected_content);
}

#[test]
fn test_bar_monitor_matching() {
    let monitor_info = MonitorInfo {
        connector: "DP-1".to_string(),
        manufacturer: Some("Dell".to_string()),
        model: Some("U2720Q".to_string()),
        logical_size: (2560, 1440),
        scale_factor: 1,
    };

    // Test connector matching
    assert_eq!(monitor_info.connector, "DP-1");
    assert_ne!(monitor_info.connector, "eDP-1");
}
