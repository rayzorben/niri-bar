use crate::bar::Bar;
use crate::config::{ColumnSpec, ModuleConfig};
use gdk4::Monitor as GdkMonitor;
use gtk4::Application as GtkApplication;
use gtk4::prelude::*;
use std::collections::HashMap;

/// Monitor information and management
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub connector: String,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub logical_size: (i32, i32),
    pub scale_factor: i32,
}

/// Monitor class that manages a single monitor and its associated bar
pub struct Monitor {
    info: MonitorInfo,
    gdk_monitor: GdkMonitor,
    bar: Option<Bar>,
}

impl Monitor {
    /// Create a new monitor instance
    pub fn new(
        connector: String,
        logical_size: (i32, i32),
        scale_factor: i32,
        gdk_monitor: GdkMonitor,
        app: &GtkApplication,
        theme: &str,
    ) -> Self {
        log::info!(
            "Monitor: Creating monitor: {} ({}x{}, scale={})",
            connector,
            logical_size.0,
            logical_size.1,
            scale_factor
        );

        // Create monitor info
        let info = MonitorInfo {
            connector: connector.clone(),
            manufacturer: gdk_monitor.manufacturer().map(|s| s.to_string()),
            model: gdk_monitor.model().map(|s| s.to_string()),
            logical_size,
            scale_factor,
        };

        // Create the bar for this monitor
        let bar = Bar::new(&info, &gdk_monitor, app, theme);

        log::info!("Monitor: ✅ Monitor '{}' created with bar", connector);

        Self {
            info,
            gdk_monitor,
            bar: Some(bar),
        }
    }

    /// Get the monitor connector name
    pub fn get_connector(&self) -> &str {
        &self.info.connector
    }

    /// Get the monitor information
    pub fn get_info(&self) -> &MonitorInfo {
        &self.info
    }

    /// Get the logical size of the monitor
    pub fn get_logical_size(&self) -> (i32, i32) {
        self.info.logical_size
    }

    /// Get the scale factor of the monitor
    pub fn get_scale_factor(&self) -> i32 {
        self.info.scale_factor
    }

    /// Get the manufacturer of the monitor
    pub fn get_manufacturer(&self) -> Option<&str> {
        self.info.manufacturer.as_deref()
    }

    /// Get the model of the monitor
    pub fn get_model(&self) -> Option<&str> {
        self.info.model.as_deref()
    }

    /// Get a reference to the GDK monitor
    pub fn get_gdk_monitor(&self) -> &GdkMonitor {
        &self.gdk_monitor
    }

    /// Get a reference to the bar
    pub fn get_bar(&self) -> Option<&Bar> {
        self.bar.as_ref()
    }

    /// Get a mutable reference to the bar
    pub fn get_bar_mut(&mut self) -> Option<&mut Bar> {
        self.bar.as_mut()
    }

    /// Check if the monitor has a bar
    pub fn has_bar(&self) -> bool {
        self.bar.is_some()
    }

    /// Show the bar for this monitor
    pub fn show_bar(&mut self) {
        if let Some(bar) = &mut self.bar {
            log::info!("Monitor: Showing bar for monitor: {}", self.info.connector);
            bar.show();
        }
    }

    /// Hide the bar for this monitor
    pub fn hide_bar(&mut self) {
        if let Some(bar) = &mut self.bar {
            log::info!("Monitor: Hiding bar for monitor: {}", self.info.connector);
            bar.hide();
        }
    }

    /// Update the theme for this monitor's bar
    pub fn update_theme(&mut self, theme: &str) {
        if let Some(bar) = &mut self.bar {
            log::info!(
                "Monitor: Updating theme for monitor {} to '{}'",
                self.info.connector,
                theme
            );
            bar.update_theme(theme);
        }
    }

    /// Update layout columns by names (ordered)
    pub fn update_columns(&mut self, column_names: &[String]) {
        if let Some(bar) = &mut self.bar {
            log::debug!(
                "Monitor: Updating columns for {}: {:?}",
                self.info.connector,
                column_names
            );
            bar.update_layout_columns_by_names(column_names);
        }
    }

    /// Update layout columns with full specs (name + modules + overflow)
    pub fn update_columns_with_specs(
        &mut self,
        columns: &[(String, ColumnSpec)],
        module_formats: &HashMap<String, String>,
        module_configs: &HashMap<String, ModuleConfig>,
    ) {
        if let Some(bar) = &mut self.bar {
            log::debug!(
                "Monitor: Updating column specs for {}: {} columns",
                self.info.connector,
                columns.len()
            );
            bar.update_layout_columns(columns, module_formats, module_configs);
        }
    }

    /// Update the bar content
    pub fn update_bar_content(&mut self, content: &str) {
        let _ = content; // deprecated path; content is handled via columns now
    }

    /// Destroy the bar for this monitor
    pub fn destroy_bar(&mut self) {
        if let Some(bar) = &mut self.bar {
            log::info!(
                "Monitor: Destroying bar for monitor: {}",
                self.info.connector
            );
            bar.destroy();
            self.bar = None;
        }
    }

    /// Print monitor information for debugging
    pub fn print_info(&self) {
        log::debug!("Monitor: Monitor: {}", self.info.connector);
        log::debug!("Monitor:   Manufacturer: {:?}", self.info.manufacturer);
        log::debug!("Monitor:   Model: {:?}", self.info.model);
        log::debug!(
            "Monitor:   Logical size: {}x{}",
            self.info.logical_size.0,
            self.info.logical_size.1
        );
        log::debug!("Monitor:   Scale factor: {}", self.info.scale_factor);
        log::debug!("Monitor:   Has bar: {}", self.has_bar());
    }

    /// Check if this monitor matches a given connector name
    pub fn matches_connector(&self, connector: &str) -> bool {
        self.info.connector == connector
    }

    /// Get the monitor's unique identifier
    pub fn get_id(&self) -> &str {
        &self.info.connector
    }
}

impl PartialEq for Monitor {
    fn eq(&self, other: &Self) -> bool {
        self.info.connector == other.info.connector
    }
}

impl Eq for Monitor {}
