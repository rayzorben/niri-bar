use crate::config::{ConfigManager, NiriBarConfig};
use anyhow::Result;
use gtk4 as gtk;
use gtk4::prelude::*;
use gtk4_layer_shell::{Layer, LayerShell, Edge};
use std::collections::HashMap;

/// Simple UI bar for a monitor
pub struct MonitorBar {
    window: gtk::Window,
    box_container: gtk::Box,
    monitor_name: String,
}

impl MonitorBar {
    /// Create a new bar for a specific monitor
    pub fn new(monitor_name: &str) -> Self {
        // Create the main window
        let window = gtk::Window::new();
        
        // Set up layer shell for Wayland
        window.init_layer_shell();
        window.set_layer(Layer::Top);
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);
        window.set_margin(Edge::Top, 0);
        window.set_margin(Edge::Left, 0);
        window.set_margin(Edge::Right, 0);
        
        // Note: set_monitor requires a Monitor object, not a string
        // For now, we'll skip monitor-specific placement
        
        // Create the main container
        let box_container = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        box_container.set_margin_start(8);
        box_container.set_margin_end(8);
        box_container.set_margin_top(4);
        box_container.set_margin_bottom(4);
        
        // Add red border styling
        let css_provider = gtk::CssProvider::new();
        css_provider.load_from_data(
            r#"
            window {
                background-color: rgba(0, 0, 0, 0.8);
                border: 2px solid red;
                border-radius: 0px;
            }
            "#,
        );
        
        gtk::style_context_add_provider_for_display(
            &gtk::gdk::Display::default().expect("Could not connect to a display."),
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        
        // Add the container to the window
        window.set_child(Some(&box_container));
        
        Self {
            window,
            box_container,
            monitor_name: monitor_name.to_string(),
        }
    }
    
    /// Add a simple label to the bar
    pub fn add_label(&self, text: &str, position: &str) {
        let label = gtk::Label::new(Some(text));
        
        match position {
            "left" => {
                self.box_container.prepend(&label);
            }
            "center" => {
                // For center, we need to add it after left items
                // This is a simplified approach
                self.box_container.append(&label);
            }
            "right" => {
                self.box_container.append(&label);
            }
            _ => {
                self.box_container.append(&label);
            }
        }
    }
    
    /// Show the bar
    pub fn show(&self) {
        self.window.show();
    }
    
    /// Hide the bar
    pub fn hide(&self) {
        self.window.hide();
    }
    
    /// Get the monitor name
    pub fn monitor_name(&self) -> &str {
        &self.monitor_name
    }
}

/// UI manager that handles all monitor bars
pub struct UIManager {
    bars: HashMap<String, MonitorBar>,
    config_manager: ConfigManager,
}

impl UIManager {
    /// Create a new UI manager
    pub fn new(config_manager: ConfigManager) -> Self {
        Self {
            bars: HashMap::new(),
            config_manager,
        }
    }
    
    /// Initialize the UI with bars for enabled monitors
    pub fn initialize(&mut self) -> Result<()> {
        // Get the current configuration
        if let Some(config) = self.config_manager.get_config() {
            self.create_bars_for_config(&config)?;
        }
        
        Ok(())
    }
    
    /// Create bars based on configuration
    pub fn create_bars_for_config(&mut self, config: &NiriBarConfig) -> Result<()> {
        // Clear existing bars
        self.bars.clear();
        
        // Check if we should show bars on all monitors or specific ones
        // For now, we'll create bars for all enabled monitors
        // In a real implementation, you'd enumerate actual monitors and match them
        
        // Create bars for each monitor using the new application config
        for monitor_match in &config.application.monitors {
            if !monitor_match.enabled {
                continue;
            }
            
            // For now, we'll create bars for all enabled monitors
            // In a real implementation, you'd enumerate actual monitors and match them
            let monitor_name = "eDP-1"; // Placeholder - should match against actual monitors
            
            // Create the bar
            let bar = MonitorBar::new(monitor_name);
            
            // Add content based on layout configuration
            if let Some(layout_config) = &monitor_match.layout {
                for (column_name, spec) in &layout_config.columns {
                    for module_name in &spec.modules {
                        match module_name.as_str() {
                            "workspaces" => {
                                bar.add_label("1 2 3 4 5", column_name);
                            }
                            "window_title" => {
                                bar.add_label("Terminal", column_name);
                            }
                            "clock" => {
                                bar.add_label("12:34:56", column_name);
                            }
                            "battery" => {
                                bar.add_label("🔋 85%", column_name);
                            }
                            "system" => {
                                bar.add_label("CPU: 25% | RAM: 45%", column_name);
                            }
                            _ => {
                                bar.add_label(&format!("[{}]", module_name), column_name);
                            }
                        }
                    }
                }
            }
            
            // Show the bar
            bar.show();
            
            // Store the bar
            self.bars.insert(monitor_name.to_string(), bar);
        }
        
        println!("Created {} bars for monitors: {:?}", 
            self.bars.len(), 
            self.bars.keys().collect::<Vec<_>>()
        );
        
        Ok(())
    }
    
    /// Update the UI when configuration changes
    pub fn update_from_config(&mut self, config: &NiriBarConfig) -> Result<()> {
        self.create_bars_for_config(config)
    }
    
    /// Get the number of active bars
    pub fn bar_count(&self) -> usize {
        self.bars.len()
    }
    
    /// Get all monitor names with bars
    pub fn monitor_names(&self) -> Vec<String> {
        self.bars.keys().cloned().collect()
    }
}


