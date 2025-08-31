use crate::config::{ConfigManager, LoggingConfig};
use crate::monitor::Monitor;
use gtk4::prelude::*;
use gtk4::{Application as GtkApplication};
use gdk4::{Display, Monitor as GdkMonitor};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use std::path::PathBuf;
use notify::{Config as NotifyConfig, Event, RecommendedWatcher, RecursiveMode, Watcher};

use std::time::Duration;
use glib::ControlFlow;

/// Main application class that manages the entire niri-bar program
pub struct Application {
    gtk_app: GtkApplication,
    pub monitors: Arc<Mutex<HashMap<String, Monitor>>>,
    config_manager: ConfigManager,
    logging_config: LoggingConfig,
    runtime: Runtime,
}

impl Application {
    /// Create a new application instance
    pub fn new(logging_config: LoggingConfig) -> Result<Self, Box<dyn std::error::Error>> {
        log::info!("Application: Initializing Niri Bar Application...");
        
        // Create the GTK application
        let gtk_app = GtkApplication::builder()
            .application_id("com.niri.bar")
            .build();

        // Create async runtime for config management
        let runtime = Runtime::new()?;
        
        // Create config manager
        let config_manager = ConfigManager::new();

        Ok(Self {
            gtk_app,
            monitors: Arc::new(Mutex::new(HashMap::new())),
            config_manager,
            logging_config,
            runtime,
        })
    }

    /// Start the application
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Application: Starting Niri Bar Application...");
        
        // Set up the application activation handler
        self.gtk_app.connect_activate({
            let app = self.gtk_app.clone();
            let monitors = self.monitors.clone();
            let config_manager = self.config_manager.clone();
            move |gtk_app| {
                log::info!("Application: 🎯 GTK Application activated!");
                Self::on_application_activate(gtk_app, &app, &monitors, &config_manager);
                // Start Niri IPC event stream if NIRI_SOCKET is set
                if std::env::var("NIRI_SOCKET").is_ok() {
                    match crate::niri::NiriIpc::new() {
                        Ok(niri) => {
                            if let Err(e) = niri.start_event_stream() {
                                log::warn!("Application: Niri IPC event stream failed to start: {}", e);
                            } else {
                                log::info!("Application: 🛰️ Niri IPC event stream attached (dumping to stdout)");
                            }
                        }
                        Err(e) => log::warn!("Application: Niri IPC init failed: {}", e),
                    }
                } else {
                    log::info!("Application: NIRI_SOCKET not set; skipping Niri IPC");
                }
            }
        });

        // Set up a timer to check for config changes periodically
        self.setup_config_checking();

        log::info!("Application: 🚀 Starting GTK main loop...");
        
        // Start the GTK main loop (this will trigger activation)
        self.gtk_app.run();
        
        log::info!("Application: Application shutdown complete");
        Ok(())
    }

    /// Set up file watching for configuration and CSS changes
    fn setup_config_checking(&mut self) {
        // Channel of changed file paths -> GTK thread
        let (tx, rx) = tokio::sync::mpsc::channel::<String>(100);

        // Spawn file watchers in background
        self.runtime.spawn(async move {
            let mut watcher = RecommendedWatcher::new(
                move |res| {
                    match res {
                        Ok(Event { paths, .. }) => {
                            for p in paths {
                                let _ = tx.blocking_send(p.display().to_string());
                            }
                        }
                        Err(err) => {
                            log::warn!("Application: 🤷 file watcher hiccup: {}", err);
                        }
                    }
                },
                NotifyConfig::default().with_poll_interval(Duration::from_secs(2)),
            ).unwrap();

            // Watch YAML configuration file
            if let Err(e) = watcher.watch(PathBuf::from("niri-bar.yaml").as_path(), RecursiveMode::NonRecursive) {
                log::error!("Application: Failed to watch niri-bar.yaml: {}", e);
            }

            // Watch CSS theme files
            let css_files = ["themes/wombat.css", "themes/solarized.css", "themes/dracula.css"];
            for css_file in &css_files {
                if let Err(e) = watcher.watch(PathBuf::from(css_file).as_path(), RecursiveMode::NonRecursive) {
                    log::error!("Application: Failed to watch {}: {}", css_file, e);
                }
            }

            log::info!("Application: 🔍 File watchers started");

            // Keep the watcher alive
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });

        // Handle file change events in GTK main thread (recurring)
        let gtk_app = self.gtk_app.clone();
        let monitors = self.monitors.clone();
        let config_manager = self.config_manager.clone();
        // Receiver must be mutable across calls; wrap in RefCell
        let rx = std::cell::RefCell::new(rx);

        glib::timeout_add_local(Duration::from_millis(250), move || {
            // Drain pending file-change events
            let mut changed_paths: Vec<String> = Vec::new();
            while let Ok(p) = rx.borrow_mut().try_recv() {
                changed_paths.push(p);
            }

            if !changed_paths.is_empty() {
                log::info!(
                    "Application: 🔔 File change vibes detected, homie: {}",
                    changed_paths.join(", ")
                );
                log::info!("Application: 🔄 Reloading config because files went glow-up...");
                if let Err(e) = Self::reload_configuration_and_update_bars(&gtk_app, &monitors, &config_manager) {
                    log::error!("Application: Failed to reload configuration: {}", e);
                }
            }

            ControlFlow::Continue
        });
    }

    /// Reload configuration and update all bars
    fn reload_configuration_and_update_bars(
        gtk_app: &GtkApplication,
        monitors: &Arc<Mutex<HashMap<String, Monitor>>>,
        config_manager: &ConfigManager,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Application: 🔄 Reloading configuration...");
        
        // Load new configuration
        let config_content = std::fs::read("niri-bar.yaml")?;
        let config = ConfigManager::parse_config(&config_content)?;
        log::info!("Application: 📋 Loaded configuration with theme: '{}'", config.application.theme);
        
        // Update config manager
        {
            let mut config_guard = config_manager.config.lock().unwrap();
            *config_guard = Some(config.clone());
        }

        // Get current monitors from GTK
        if let Some(display) = Display::default() {
            let gdk_monitors = display.monitors();
            let n_monitors = gdk_monitors.n_items();
            
            for i in 0..n_monitors {
                if let Some(monitor_obj) = gdk_monitors.item(i)
                    && let Ok(gdk_monitor) = monitor_obj.downcast::<GdkMonitor>() {
                        let connector = gdk_monitor.connector()
                            .unwrap_or_else(|| "Unknown".into())
                            .to_string();
                        
                        let logical_size = {
                            let geometry = gdk_monitor.geometry();
                            (geometry.width(), geometry.height())
                        };
                        
                        let scale_factor = gdk_monitor.scale_factor();
                        
                        // Check if monitor should be enabled
                        let should_enable = config_manager.is_monitor_enabled(&connector);
                        let new_theme = &config.application.theme;
                        
                        // Update or create monitor
                        let mut monitors_guard = monitors.lock().unwrap();
                        
                        if should_enable {
                            if monitors_guard.contains_key(&connector) {
                                // Update existing monitor with new theme
                                log::info!("Application: 🔄 Updating existing monitor '{}' with theme '{}'", connector, new_theme);
                                if let Some(existing_monitor) = monitors_guard.get_mut(&connector) {
                                    existing_monitor.update_theme(new_theme);
                                    // Update columns from layout
                                    let column_specs: Vec<(String, crate::config::ColumnSpec)> = config_manager
                                        .get_monitor_layout(&connector)
                                        .map(|layout| layout.columns.into_iter().collect())
                                        .unwrap_or_default();
                                    // Build module format map (single merged format; date_format deprecated)
                                    let module_formats = Self::collect_module_formats(config_manager, &connector);
                                    existing_monitor.update_columns_with_specs(&column_specs, &module_formats);
                                }
                            } else {
                                // Create new monitor
                                log::info!("Application: ➕ Creating new monitor '{}' with theme '{}'", connector, new_theme);
                                let mut new_monitor = Monitor::new(
                                    connector.clone(),
                                    logical_size,
                                    scale_factor,
                                    gdk_monitor,
                                    gtk_app,
                                    new_theme,
                                );
                                // Initialize columns from layout
                                let column_specs: Vec<(String, crate::config::ColumnSpec)> = config_manager
                                    .get_monitor_layout(&connector)
                                    .map(|layout| layout.columns.into_iter().collect())
                                    .unwrap_or_default();
                                let module_formats = Self::collect_module_formats(config_manager, &connector);
                                new_monitor.update_columns_with_specs(&column_specs, &module_formats);
                                new_monitor.show_bar();
                                monitors_guard.insert(connector.clone(), new_monitor);
                            }
                        } else {
                            // Remove monitor if it exists and should be disabled
                            if let Some(mut removed_monitor) = monitors_guard.remove(&connector) {
                                log::info!("Application: ➖ Removing disabled monitor '{}'", connector);
                                removed_monitor.hide_bar();
                            }
                        }
                    }
                }
        }
        
        log::info!("Application: ✅ Configuration reload complete");
        Ok(())
    }

    /// Collect per-module merged formats for a given monitor.
    /// We accept either `format` on the module. `date_format` is ignored (deprecated).
    fn collect_module_formats(config_manager: &ConfigManager, connector: &str) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();
        // read current config
        let config_guard = config_manager.config.lock().unwrap();
        if let Some(cfg) = &*config_guard {
            // Start with global module defaults
            for (name, mc) in &cfg.application.modules {
                if let Some(fmt) = mc.format.clone() {
                    map.insert(name.clone(), fmt);
                }
            }
            // Overlay per-monitor overrides
            // Find best matching monitor config
            let mut best: Option<&crate::config::MonitorConfig> = None;
            let mut best_spec = 0;
            for m in &cfg.application.monitors {
                if ConfigManager::matches_pattern(connector, &m.match_pattern) {
                    let s = ConfigManager::pattern_specificity(&m.match_pattern);
                    if s > best_spec { best = Some(m); best_spec = s; }
                }
            }
            if let Some(m) = best {
                if let Some(mods) = &m.modules {
                    for (name, mc) in mods {
                        if let Some(fmt) = mc.format.clone() {
                            map.insert(name.clone(), fmt);
                        }
                    }
                }
            }
        }
        map
    }

    /// Handle application activation (when GTK app starts)
    fn on_application_activate(
        gtk_app: &GtkApplication, 
        _app: &GtkApplication,
        monitors: &Arc<Mutex<HashMap<String, Monitor>>>,
        config_manager: &ConfigManager,
    ) {
        log::info!("Application: Application activated, initializing monitors...");
        
        // Initial configuration load and monitor setup
        if let Err(e) = Self::reload_configuration_and_update_bars(gtk_app, monitors, config_manager) {
            log::error!("Application: Failed to load initial configuration: {}", e);
        }
        
        log::info!("Application: 🎊 Initial monitor setup complete!");
        log::info!("Application: Configuration-driven bar display active!");
        log::info!("Application: 🔄 Hot-reload enabled for YAML and CSS changes!");
    }

    /// Get the number of monitors
    pub fn monitor_count(&self) -> usize {
        self.monitors.lock().unwrap().len()
    }

    /// Get the logging configuration
    pub fn get_logging_config(&self) -> &LoggingConfig {
        &self.logging_config
    }

    /// Get the config manager
    pub fn get_config_manager(&self) -> &ConfigManager {
        &self.config_manager
    }
}