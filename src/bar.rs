// Re-export MonitorInfo for use in tests
use crate::config::{ColumnOverflowPolicy, ColumnSpec, DisplayMode, ModuleConfig, TextAlign};
use crate::modules::create_module_widget;
pub use crate::monitor::MonitorInfo;
use gdk4::{Display, Monitor as GdkMonitor};
use gtk4 as gtk;
use gtk4::prelude::*;
use gtk4::{Application as GtkApplication, ApplicationWindow, CssProvider};
use gtk4::{ListBox, MenuButton, Popover};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
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
        log::info!(
            "Bar: 🎯 Creating bar for monitor: {}",
            monitor_info.connector
        );

        // Create the window
        let window = ApplicationWindow::new(app);

        // Set monitor-specific CSS class
        window.add_css_class(&format!(
            "monitor-{}",
            monitor_info.connector.replace("-", "_")
        ));

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
        // Equal-width columns across the bar
        container.set_homogeneous(true);
        container.set_hexpand(true);
        container.set_halign(gtk::Align::Fill);
        container.add_css_class("bar-columns");
        container.add_css_class(&format!(
            "monitor-{}",
            monitor_info.connector.replace("-", "_")
        ));
        window.set_child(Some(&container));

        log::info!(
            "Bar: ✅ Bar created and pinned to monitor: {} ({}x{}, scale={})",
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
        let mut css_content = String::new();

        // First, load base.css
        let base_path = "themes/base.css";
        match std::fs::read_to_string(base_path) {
            Ok(base_css) => {
                css_content.push_str(&base_css);
                css_content.push('\n');
                log::info!("Bar: Loaded base CSS");
            }
            Err(e) => {
                log::warn!("Bar: Failed to load base.css: {}, using fallback", e);
                css_content.push_str(
                    r#"
                window {
                    background-color: transparent;
                    border: none;
                    border-radius: 0px;
                    color: #f6f3e8;
                    font-family: 'Sans', sans-serif;
                    font-size: 12px;
                    font-weight: bold;
                }
                label {
                    color: #f6f3e8;
                    padding: 0 6px;
                    background: transparent;
                }
                "#,
                );
            }
        }

        // Then load theme-specific CSS
        let theme_path = format!("themes/{}.css", theme);
        match std::fs::read_to_string(&theme_path) {
            Ok(theme_css) => {
                css_content.push_str(&theme_css);
                log::info!("Bar: Loaded CSS theme: {}", theme);
            }
            Err(e) => {
                log::warn!(
                    "Bar: Failed to load theme '{}': {}, using defaults",
                    theme,
                    e
                );
                // Add default theme variables
                css_content.push_str(
                    r#"
                :root {
                    --bg-transparent: transparent;
                    --text-primary: #f6f3e8;
                    --border-color: #8f8f8f;
                    --column-bg: #242424;
                    --column-left-text: #cae682;
                    --column-center-text: #f6f3e8;
                    --column-right-text: #e5786d;
                    --column-left-of-center-text: #b4d273;
                    --column-right-of-center-text: #e5786d;
                    --module-workspaces: #cae682;
                    --module-window-title: #f6f3e8;
                    --module-clock: #e5786d;
                    --module-battery: #8ac6f2;
                    --module-system: #f4bf75;
                    --hover-bg: #3a3a3a;
                    --active-bg: #cae682;
                    --active-text: #242424;
                    --warning-color: #f4bf75;
                    --critical-color: #e5786d;
                }
                "#,
                );
            }
        }

        css_content
    }

    /// Show the bar
    pub fn show(&mut self) {
        if !self.is_visible {
            log::info!(
                "Bar: Showing bar for monitor: {}",
                self.monitor_info.connector
            );
            self.window.present();
            self.is_visible = true;
        }
    }

    /// Hide the bar
    pub fn hide(&mut self) {
        if self.is_visible {
            log::info!(
                "Bar: Hiding bar for monitor: {}",
                self.monitor_info.connector
            );
            self.window.close();
            self.is_visible = false;
        }
    }

    /// Update the theme for this bar
    pub fn update_theme(&mut self, theme: &str) {
        log::info!(
            "Bar: Updating theme for monitor {} to '{}'",
            self.monitor_info.connector,
            theme
        );

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

        log::info!(
            "Bar: ✅ Theme updated to '{}' for monitor {}",
            theme,
            self.monitor_info.connector
        );
    }

    /// Update columns by names only (legacy)
    pub fn update_layout_columns_by_names(&mut self, column_names: &[String]) {
        // Clear existing content
        while let Some(child) = self.container.first_child() {
            self.container.remove(&child);
        }

        let columns_count = column_names.len().max(1) as i32;
        for name in column_names {
            let safe = name.replace([' ', '-'], "_");
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

            // Placeholder kebab button and popover list; visibility toggled on actual overflow
            let kebab = MenuButton::builder()
                .has_frame(false)
                .icon_name("view-more-symbolic")
                .build();
            kebab.add_css_class("column-kebab");
            kebab.add_css_class(&format!("column-kebab-{}", safe));
            let popover = Popover::new();
            let list = ListBox::new();
            list.add_css_class("column-kebab-list");
            popover.set_child(Some(&list));
            kebab.set_popover(Some(&popover));
            kebab.set_visible(false);
            column_box.append(&kebab);

            // Approximate overflow check using monitor logical width divided by number of columns
            let available_w = (self.monitor_info.logical_size.0 / columns_count).max(1);
            let (_min_w, nat_w, _min_h, _nat_h) =
                column_box.measure(gtk::Orientation::Horizontal, -1);
            let is_overflowing = nat_w > available_w;
            kebab.set_visible(is_overflowing);

            self.container.append(&column_box);
        }

        self.container.queue_draw();
    }

    /// Update columns from (name, ColumnSpec) pairs; applies overflow policy
    pub fn update_layout_columns(
        &mut self,
        columns: &[(String, ColumnSpec)],
        module_formats: &HashMap<String, String>,
        module_configs: &HashMap<String, ModuleConfig>,
    ) {
        // Clear existing content
        while let Some(child) = self.container.first_child() {
            self.container.remove(&child);
        }

        let columns_count = columns.len().max(1) as i32;
        for (name, spec) in columns {
            let safe = name.replace([' ', '-'], "_");
            let column_box = gtk::Box::new(gtk::Orientation::Horizontal, spec.gap.unwrap_or(0));

            // Column sizing: equal width by default (container homogeneous=true)
            // Allow optional fixed pixel width per column
            if let Some(width) = spec.width {
                column_box.set_hexpand(false);
                column_box.set_size_request(width, -1);
            } else {
                column_box.set_hexpand(true);
            }

            // Columns themselves always fill their equal-width allocation
            column_box.set_halign(gtk::Align::Fill);
            column_box.set_hexpand(true);

            // Determine effective text alignment for this column (column-level only)
            let effective_align: TextAlign = spec.align.clone().unwrap_or(match name.as_str() {
                "center" => TextAlign::Center,
                "right" => TextAlign::Right,
                _ => TextAlign::Left,
            });
            column_box.add_css_class("column");
            column_box.add_css_class(&format!("column-{}", safe));
            match spec.overflow {
                ColumnOverflowPolicy::Hide => column_box.add_css_class("overflow-hide"),
                ColumnOverflowPolicy::Kebab => column_box.add_css_class("overflow-kebab"),
            }

            // Build module widgets dynamically via registry (collect first, decide overflow later)
            let mut module_widgets: Vec<gtk::Widget> = Vec::new();
            for module in &spec.modules {
                // Get module configuration
                let module_config = module_configs.get(module);

                // Check display property - skip hidden modules
                if let Some(config) = module_config
                    && matches!(
                        config.display.as_ref().unwrap_or(&DisplayMode::Show),
                        DisplayMode::Hide
                    )
                {
                    continue;
                }

                // Merge module settings from provided format map into a minimal settings struct
                let settings = crate::config::ModuleConfig {
                    format: module_formats.get(module).cloned(),
                    tooltip: module_config.and_then(|c| c.tooltip),
                    highlight_active: module_config.and_then(|c| c.highlight_active),
                    show_numbers: module_config.and_then(|c| c.show_numbers),
                    show_wallpaper: module_config.and_then(|c| c.show_wallpaper),
                    // Pass through wallpaper mapping and defaults so workspaces can prepopulate
                    default_wallpaper: module_config.and_then(|c| c.default_wallpaper.clone()),
                    wallpapers: module_config.and_then(|c| c.wallpapers.clone()),
                    special_cmd: module_config.and_then(|c| c.special_cmd.clone()),
                    swww_options: module_config.and_then(|c| c.swww_options.clone()),
                    max_length: module_config.and_then(|c| c.max_length),
                    ellipsize: module_config.and_then(|c| c.ellipsize.clone()),
                    show_percentage: module_config.and_then(|c| c.show_percentage),
                    warn_threshold: module_config.and_then(|c| c.warn_threshold),
                    critical_threshold: module_config.and_then(|c| c.critical_threshold),
                    cpu: module_config.and_then(|c| c.cpu),
                    mem: module_config.and_then(|c| c.mem),
                    net: module_config.and_then(|c| c.net),
                    enabled: module_config.and_then(|c| c.enabled),
                    display: module_config.and_then(|c| c.display.clone()),
                    width: module_config.and_then(|c| c.width),
                    show_window_titles: module_config.and_then(|c| c.show_window_titles),
                    highlight_focused: module_config.and_then(|c| c.highlight_focused),
                    additional: module_config
                        .map(|c| c.additional.clone())
                        .unwrap_or_default(),
                };

                if let Some(widget) = create_module_widget(module, &settings) {
                    module_widgets.push(widget);
                } else {
                    // Unknown module: skip rendering silently
                    log::warn!("Bar: unknown module '{}' , skipping", module);
                }
            }

            // Kebab menu (three vertical dots). Only show when actual overflow for kebab policy
            let kebab = MenuButton::builder()
                .has_frame(false)
                .icon_name("view-more-symbolic")
                .build();
            kebab.add_css_class("column-kebab");
            kebab.add_css_class(&format!("column-kebab-{}", safe));
            kebab.set_visible(false);
            let popover = Popover::new();
            let list = ListBox::new();
            list.add_css_class("column-kebab-list");
            popover.set_child(Some(&list));
            kebab.set_popover(Some(&popover));
            column_box.append(&kebab);

            // CSS border for columns to visualize sections
            column_box.add_css_class("column-outline");

            // Place widgets; overflow extras into kebab popover list
            let available_w = (self.monitor_info.logical_size.0 / columns_count).max(1);
            let (_k_min_w, kebab_nat_w, _k_min_h, _k_nat_h) =
                kebab.measure(gtk::Orientation::Horizontal, -1);
            let mut used_w = 0;
            let mut overflowed: Vec<gtk::Widget> = Vec::new();

            for w in &module_widgets {
                let (_m_min_w, m_nat_w, _m_min_h, _m_nat_h) =
                    w.measure(gtk::Orientation::Horizontal, -1);
                let budget = if matches!(spec.overflow, ColumnOverflowPolicy::Kebab) {
                    available_w - kebab_nat_w
                } else {
                    available_w
                };
                if used_w + m_nat_w <= budget {
                    used_w += m_nat_w;
                } else {
                    overflowed.push(w.clone());
                }
            }

            // Optionally insert spacers to realize column-level grouping alignment
            if matches!(effective_align, TextAlign::Right) {
                let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                spacer.set_hexpand(true);
                column_box.append(&spacer);
            }
            if matches!(effective_align, TextAlign::Center) {
                let spacer_left = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                spacer_left.set_hexpand(true);
                column_box.append(&spacer_left);
            }

            // Append widgets that fit, setting alignment based on column alignment
            for w in module_widgets.iter() {
                if !overflowed.iter().any(|ow| ow == w) {
                    // GTK4 CSS doesn't support text-align, so set alignment programmatically
                    if let Some(label) = w.downcast_ref::<gtk::Label>() {
                        // Use column-level alignment only
                        let module_align = &effective_align;

                        match module_align {
                            TextAlign::Center => label.set_xalign(0.5),
                            TextAlign::Left => label.set_xalign(0.0),
                            TextAlign::Right => label.set_xalign(1.0),
                        }
                        // Also set justification to be safe across label modes
                        label.set_justify(match module_align {
                            TextAlign::Center => gtk::Justification::Center,
                            TextAlign::Left => gtk::Justification::Left,
                            TextAlign::Right => gtk::Justification::Right,
                        });
                    }
                    // For grouped alignment, don't force children to expand; let spacers handle layout
                    w.set_hexpand(false);
                    w.set_halign(gtk::Align::Fill);
                    column_box.append(w);
                }
            }

            if matches!(effective_align, TextAlign::Center) {
                let spacer_right = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                spacer_right.set_hexpand(true);
                column_box.append(&spacer_right);
            }

            // Move overflowed widgets into popover as rows
            for w in overflowed {
                if matches!(spec.overflow, ColumnOverflowPolicy::Kebab) {
                    // Set alignment for overflowed labels too - use column alignment for overflow
                    if let Some(label) = w.downcast_ref::<gtk::Label>() {
                        match effective_align {
                            TextAlign::Center => label.set_xalign(0.5),
                            TextAlign::Left => label.set_xalign(0.0),
                            TextAlign::Right => label.set_xalign(1.0),
                        }
                    }
                    let row = gtk::ListBoxRow::new();
                    row.add_css_class("column-overflow-row");
                    row.set_child(Some(&w));
                    list.append(&row);
                }
            }

            // Show kebab only if there is at least one overflowed item
            kebab.set_visible(list.first_child().is_some());

            self.container.append(&column_box);
        }

        self.container.queue_draw();
    }

    /// Destroy the bar
    pub fn destroy(&mut self) {
        log::info!(
            "Bar: Destroying bar for monitor: {}",
            self.monitor_info.connector
        );
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
        log::debug!(
            "Bar: Setting height for monitor {}: {}",
            self.monitor_info.connector,
            height
        );
        self.window.set_default_height(height);
    }

    /// Set the bar width
    pub fn set_width(&self, width: i32) {
        log::debug!(
            "Bar: Setting width for monitor {}: {}",
            self.monitor_info.connector,
            width
        );
        self.window.set_default_width(width);
    }

    /// Set the bar size
    pub fn set_size(&self, width: i32, height: i32) {
        log::debug!(
            "Bar: Setting size for monitor {}: {}x{}",
            self.monitor_info.connector,
            width,
            height
        );
        self.window.set_default_size(width, height);
    }

    /// Update the bar with monitor information
    pub fn update_with_monitor_info(&mut self, monitor_info: &MonitorInfo) {
        log::debug!(
            "Bar: Updating bar with new monitor info for: {}",
            monitor_info.connector
        );
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
