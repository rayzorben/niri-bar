use niri_bar::config::{ConfigManager, ConfigEvent};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Config Manager Example");
    println!("=====================");
    
    // Create a new configuration manager
    let mut config_manager = ConfigManager::new();
    
    // Subscribe to configuration events
    let mut event_rx = config_manager.subscribe();
    
    // Start monitoring the configuration file
    println!("Starting configuration monitoring...");
    config_manager.start().await?;
    
    // Handle configuration events
    tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            match event {
                ConfigEvent::Loaded(config) => {
                    println!("✅ Configuration loaded successfully!");
                    println!("   Application modules: {:?}", config.application.modules.keys().collect::<Vec<_>>());
                    println!("   Application layouts: {:?}", config.application.layouts.keys().collect::<Vec<_>>());
                    println!("   Monitor patterns: {:?}", config.application.monitors.iter().map(|m| &m.match_pattern).collect::<Vec<_>>());
                }
                ConfigEvent::Updated(config) => {
                    println!("🔄 Configuration updated!");
                    println!("   Application modules: {:?}", config.application.modules.keys().collect::<Vec<_>>());
                    println!("   Application layouts: {:?}", config.application.layouts.keys().collect::<Vec<_>>());
                }
                ConfigEvent::Error(error) => {
                    println!("❌ Configuration error: {}", error);
                }
            }
        }
    });
    
    // Example: Access configuration data
    tokio::spawn(async move {
        // Wait a bit for initial load
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Get current configuration
        if let Some(config) = config_manager.get_config() {
            println!("\n📋 Current Configuration:");
            println!("   Logging level: {}", config.logging.level);
            
            // Check specific monitors
            for monitor_name in ["eDP-1", "DP-1", "DP-2", "DP-3"] {
                let enabled = config_manager.is_monitor_enabled(monitor_name);
                let modules = config_manager.get_monitor_modules(monitor_name);
                let layout = config_manager.get_monitor_layout(monitor_name);
                
                println!("   Monitor {}: enabled={}, has_modules={}, has_layout={}", 
                    monitor_name, enabled, modules.is_some(), layout.is_some());
                
                if let Some(modules) = modules {
                    for (module_name, _module_config) in modules {
                        println!("     - {}", module_name);
                    }
                }
            }
            
            // Get global modules and layouts
            if let Some(global_modules) = config_manager.get_global_modules() {
                println!("   Global modules: {:?}", global_modules.keys().collect::<Vec<_>>());
            }
            
            if let Some(layouts) = config_manager.get_layouts() {
                println!("   Available layouts: {:?}", layouts.keys().collect::<Vec<_>>());
            }
        } else {
            println!("❌ No configuration loaded yet");
        }
    });
    
    // Keep the main task alive for 30 seconds
    println!("Monitoring configuration for 30 seconds...");
    println!("Modify niri-bar.yaml to see configuration updates!");
    tokio::time::sleep(Duration::from_secs(30)).await;
    
    println!("Configuration monitoring finished.");
    Ok(())
}
