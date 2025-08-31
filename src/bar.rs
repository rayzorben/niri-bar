// Re-export MonitorInfo for use in tests
pub use crate::monitor::MonitorInfo;
use gtk4::prelude::*;
use gtk4::{Application as GtkApplication, ApplicationWindow, CssProvider};
use gtk4_layer_shell::{LayerShell, Layer, Edge};
use gdk4::{Display, Monitor as GdkMonitor};
use gtk4 as gtk;
use gtk4::{MenuButton, Popover, ListBox};
use crate::config::{ColumnSpec, ColumnOverflowPolicy};
use crate::modules::create_module_widget;
use std::collections::HashMap;

/// Bar class that manages a single status bar for a monitor
pub struct Bar {
    window: ApplicationWindow,
    container: gtk::Box,
    monitor_info: MonitorInfo,
    is_visible: bool,
    css_provider: CssProvider,
}

impl Bar {
    /// Create a new bar for a monitor
    pub fn new(
        monitor_info: &MonitorInfo,
        gdk_monitor: &GdkMonitor,
        app: &GtkApplication,
        theme: &str,
    ) -> Self {
        log::info!("Bar: 🎯 Creating bar for monitor: {}", monitor_info.connector);
        
        // Create the window
        let window = ApplicationWindow::new(app);
        
        // Set monitor-specific CSS class
        window.add_css_class(&format!("monitor-{}", monitor_info.connector.replace("-", "_")));
        
        // Initialize layer shell
        window.init_layer_shell();
        window.set_layer(Layer::Top);
        window.auto_exclusive_zone_enable();
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);
        
        // Pin to specific monitor
        window.set_monitor(Some(gdk_monitor));
        
        // Set bar height
        window.set_default_height(40);
        
        // Load and apply CSS theme
        let css_provider = CssProvider::new();
        let css_content = Self::load_theme_css(theme);
        css_provider.load_from_data(&css_content);
        
        // Apply CSS to the display
        if let Some(display) = Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &css_provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
        
        // Create main container for columns
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        container.set_homogeneous(true);
        container.set_hexpand(true);
        container.set_halign(gtk::Align::Fill);
        container.add_css_class("bar-columns");
        container.add_css_class(&format!("monitor-{}", monitor_info.connector.replace("-", "_")));
        window.set_child(Some(&container));
        
        log::info!("Bar: ✅ Bar created and pinned to monitor: {} ({}x{}, scale={})", 
            monitor_info.connector, 
            monitor_info.logical_size.0, 
            monitor_info.logical_size.1,
            monitor_info.scale_factor
        );
        
        Self {
            window,
            container,
            monitor_info: monitor_info.clone(),
            is_visible: false,
            css_provider,
        }
    }

    /// Load CSS theme from file
    fn load_theme_css(theme: &str) -> String {
        let theme_path = format!("themes/{}.css", theme);
        match std::fs::read_to_string(&theme_path) {
            Ok(css) => {
                log::info!("Bar: Loaded CSS theme: {}", theme);
                css
            }
            Err(e) => {
                log::warn!("Bar: Failed to load theme '{}': {}, using default", theme, e);
                // Fallback to default CSS
                r#"
                window {
                    background-color: rgba(0, 0, 0, 0.9);
                    border: 2px solid #ff6b6b;
                    border-radius: 0px;
                    color: white;
                    font-family: 'Sans', sans-serif;
                    font-size: 14px;
                    font-weight: bold;
                }
                label {
                    color: white;
                    padding: 8px 16px;
                    background: transparent;
                }
                "#.to_string()
            }
        }
    }

    /// Show the bar
    pub fn show(&mut self) {
        if !self.is_visible {
            log::info!("Bar: Showing bar for monitor: {}", self.monitor_info.connector);
            self.window.present();
            self.is_visible = true;
        }
    }

    /// Hide the bar
    pub fn hide(&mut self) {
        if self.is_visible {
            log::info!("Bar: Hiding bar for monitor: {}", self.monitor_info.connector);
            self.window.close();
            self.is_visible = false;
        }
    }

    /// Update the theme for this bar
    pub fn update_theme(&mut self, theme: &str) {
        log::info!("Bar: Updating theme for monitor {} to '{}'", self.monitor_info.connector, theme);
        
        // Remove old CSS provider from display
        if let Some(display) = Display::default() {
            gtk::style_context_remove_provider_for_display(&display, &self.css_provider);
        }
        
        // Load new CSS theme
        let new_css_provider = CssProvider::new();
        let css_content = Self::load_theme_css(theme);
        new_css_provider.load_from_data(&css_content);
        
        // Apply new CSS to the display
        if let Some(display) = Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &new_css_provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
        
        // Update the stored CSS provider
        self.css_provider = new_css_provider;
        
        // Force a style update on the widgets
        self.window.queue_draw();
        self.container.queue_draw();
        
        log::info!("Bar: ✅ Theme updated to '{}' for monitor {}", theme, self.monitor_info.connector);
    }

    /// Update columns by names only (legacy)
    pub fn update_layout_columns_by_names(&mut self, column_names: &[String]) {
        // Clear existing content
        while let Some(child) = self.container.first_child() {
            self.container.remove(&child);
        }

        for name in column_names {
            let safe = name.replace(' ', "_").replace('-', "_");
            let column_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            column_box.set_hexpand(true);
            column_box.set_halign(gtk::Align::Fill);
            column_box.add_css_class("column");
            column_box.add_css_class(&format!("column-{}", safe));

            let label = gtk::Label::new(Some(name));
            label.set_hexpand(true);
            label.set_halign(gtk::Align::Center);
            label.add_css_class("column-label");
            label.add_css_class(&format!("column-label-{}", safe));

            column_box.append(&label);

            // Placeholder kebab button and popover list; visibility and items decided by overflow policy later
            let kebab = MenuButton::builder().has_frame(false).icon_name("open-menu-symbolic").build();
            kebab.add_css_class("column-kebab");
            kebab.add_css_class(&format!("column-kebab-{}", safe));
            let popover = Popover::new();
            let list = ListBox::new();
            list.add_css_class("column-kebab-list");
            popover.set_child(Some(&list));
            kebab.set_popover(Some(&popover));
            column_box.append(&kebab);

            self.container.append(&column_box);
        }

        self.container.queue_draw();
    }

    /// Update columns from (name, ColumnSpec) pairs; applies overflow policy
    pub fn update_layout_columns(&mut self, columns: &[(String, ColumnSpec)], module_formats: &HashMap<String, String>) {
        // Clear existing content
        while let Some(child) = self.container.first_child() {
            self.container.remove(&child);
        }

        for (name, spec) in columns {
            let safe = name.replace(' ', "_").replace('-', "_");
            let column_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            column_box.set_hexpand(true);
            column_box.set_halign(gtk::Align::Fill);
            column_box.add_css_class("column");
            column_box.add_css_class(&format!("column-{}", safe));
            match spec.overflow {
                ColumnOverflowPolicy::Hide => column_box.add_css_class("overflow-hide"),
                ColumnOverflowPolicy::Kebab => column_box.add_css_class("overflow-kebab"),
            }

            // Build module widgets dynamically via registry
            for module in &spec.modules {
                // Merge module settings from provided format map into a minimal settings struct
                let settings = crate::config::ModuleConfig {
                    format: module_formats.get(module).cloned(),
                    tooltip: None,
                    highlight_active: None,
                    show_numbers: None,
                    max_length: None,
                    ellipsize: None,
                    show_percentage: None,
                    warn_threshold: None,
                    critical_threshold: None,
                    cpu: None,
                    mem: None,
                    net: None,
                    enabled: None,
                    additional: std::collections::HashMap::new(),
                };

                if let Some(widget) = create_module_widget(module, &settings) {
                    column_box.append(&widget);
                } else {
                    // Unknown module: skip rendering silently
                    log::warn!("Bar: unknown module '{}', skipping", module);
                }
            }

            // Kebab menu, only visible if overflow policy is kebab (CSS can hide by default)
            let kebab = MenuButton::builder().has_frame(false).icon_name("open-menu-symbolic").build();
            kebab.add_css_class("column-kebab");
            kebab.add_css_class(&format!("column-kebab-{}", safe));
            if let ColumnOverflowPolicy::Kebab = spec.overflow {
                kebab.set_visible(true);
            } else {
                kebab.set_visible(false);
            }
            let popover = Popover::new();
            let list = ListBox::new();
            list.add_css_class("column-kebab-list");
            popover.set_child(Some(&list));
            kebab.set_popover(Some(&popover));
            column_box.append(&kebab);

            // CSS border for columns to visualize sections
            column_box.add_css_class("column-outline");

            self.container.append(&column_box);
        }

        self.container.queue_draw();
    }

    /// Destroy the bar
    pub fn destroy(&mut self) {
        log::info!("Bar: Destroying bar for monitor: {}", self.monitor_info.connector);
        self.window.close();
        self.is_visible = false;
    }

    /// Get the monitor connector this bar belongs to
    pub fn get_monitor_connector(&self) -> &str {
        &self.monitor_info.connector
    }

    /// Get the monitor information
    pub fn get_monitor_info(&self) -> &MonitorInfo {
        &self.monitor_info
    }

    /// Check if the bar is currently visible
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    /// Set the bar height
    pub fn set_height(&self, height: i32) {
        log::debug!("Bar: Setting height for monitor {}: {}", 
            self.monitor_info.connector, height);
        self.window.set_default_height(height);
    }

    /// Set the bar width
    pub fn set_width(&self, width: i32) {
        log::debug!("Bar: Setting width for monitor {}: {}", 
            self.monitor_info.connector, width);
        self.window.set_default_width(width);
    }

    /// Set the bar size
    pub fn set_size(&self, width: i32, height: i32) {
        log::debug!("Bar: Setting size for monitor {}: {}x{}", 
            self.monitor_info.connector, width, height);
        self.window.set_default_size(width, height);
    }

    /// Update the bar with monitor information
    pub fn update_with_monitor_info(&mut self, monitor_info: &MonitorInfo) {
        log::debug!("Bar: Updating bar with new monitor info for: {}", monitor_info.connector);
        self.monitor_info = monitor_info.clone();
        self.container.queue_draw();
    }

    /// Get the GTK window (for advanced operations)
    pub fn get_window(&self) -> &ApplicationWindow {
        &self.window
    }

    /// Get the GTK container for advanced operations
    pub fn get_container(&self) -> &gtk::Box {
        &self.container
    }

    /// Check if the bar matches a given monitor connector
    pub fn matches_monitor(&self, connector: &str) -> bool {
        self.monitor_info.connector == connector
    }

    /// Get the bar's unique identifier
    pub fn get_id(&self) -> &str {
        &self.monitor_info.connector
    }
}


