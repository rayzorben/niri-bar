use crate::file_watcher::FileWatcher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use indexmap::IndexMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use anyhow::Result;

/// Module configuration with YAML anchor support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub tooltip: Option<bool>,
    #[serde(default)]
    pub highlight_active: Option<bool>,
    #[serde(default)]
    pub show_numbers: Option<bool>,
    #[serde(default)]
    pub max_length: Option<usize>,
    #[serde(default)]
    pub ellipsize: Option<String>,
    #[serde(default)]
    pub show_percentage: Option<bool>,
    #[serde(default)]
    pub warn_threshold: Option<u8>,
    #[serde(default)]
    pub critical_threshold: Option<u8>,
    #[serde(default)]
    pub cpu: Option<bool>,
    #[serde(default)]
    pub mem: Option<bool>,
    #[serde(default)]
    pub net: Option<bool>,
    #[serde(default)]
    pub enabled: Option<bool>,
    // Allow additional fields
    #[serde(flatten)]
    pub additional: HashMap<String, serde_yaml::Value>,
}

/// Column overflow behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColumnOverflowPolicy {
    #[serde(rename = "hide", alias = "crop")] // accept legacy name "crop"
    Hide,
    #[serde(rename = "kebab")]
    Kebab,
}

impl Default for ColumnOverflowPolicy {
    fn default() -> Self { Self::Hide }
}

/// Per-column spec: modules + overflow policy
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ColumnSpec {
    #[serde(default)]
    pub modules: Vec<String>,
    #[serde(default)]
    pub overflow: ColumnOverflowPolicy,
}

/// Layout configuration with column mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    #[serde(default)]
    pub columns: IndexMap<String, ColumnSpec>,
}

/// Monitor configuration with layout and module overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    #[serde(rename = "match")]
    pub match_pattern: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub layout: Option<LayoutConfig>,
    #[serde(default)]
    pub modules: Option<HashMap<String, ModuleConfig>>,
}

/// Application-level configuration with YAML anchors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    /// CSS theme to use for styling the bar
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Global module defaults (YAML anchors)
    pub modules: HashMap<String, ModuleConfig>,
    /// Reusable layout profiles (YAML anchors)
    pub layouts: HashMap<String, LayoutConfig>,
    /// Monitor configurations with pattern matching
    pub monitors: Vec<MonitorConfig>,
}

/// Default theme function
fn default_theme() -> String {
    "wombat".to_string()
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file: String,
    pub console: bool,
    #[serde(default = "default_log_format")]
    pub format: String,
    #[serde(default = "default_true")]
    pub include_file: bool,
    #[serde(default = "default_true")]
    pub include_line: bool,
    #[serde(default = "default_true")]
    pub include_class: bool,
}

/// Complete configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NiriBarConfig {
    pub application: ApplicationConfig,
    #[serde(default = "default_logging_config")]
    pub logging: LoggingConfig,
}

fn default_enabled() -> bool {
    true
}

fn default_logging_config() -> LoggingConfig {
    LoggingConfig {
        level: "info".to_string(),
        file: "~/.local/share/niri-bar/niri-bar.log".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    }
}

fn default_log_format() -> String {
    "iso8601".to_string()
}

fn default_true() -> bool {
    true
}

/// Configuration events that can be emitted
#[derive(Debug, Clone)]
pub enum ConfigEvent {
    /// Configuration was loaded successfully
    Loaded(NiriBarConfig),
    /// Configuration was updated successfully
    Updated(NiriBarConfig),
    /// An error occurred while loading/parsing configuration
    Error(String),
}

/// Configuration manager that monitors and parses the YAML file
pub struct ConfigManager {
    pub config: Arc<Mutex<Option<NiriBarConfig>>>,
    event_tx: broadcast::Sender<ConfigEvent>,
    watcher: Option<FileWatcher>,
}

impl Clone for ConfigManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            event_tx: self.event_tx.clone(),
            watcher: None, // Don't clone the watcher
        }
    }
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(100);
        
        Self {
            config: Arc::new(Mutex::new(None)),
            event_tx,
            watcher: None,
        }
    }

    /// Start monitoring the configuration file
    pub async fn start(&mut self) -> Result<()> {
        log::info!("ConfigManager: Starting configuration file monitoring...");
        
        let config = self.config.clone();
        let event_tx = self.event_tx.clone();
        
        // Create file watcher for the configuration file
        let mut watcher = FileWatcher::new("niri-bar.yaml")
            .on_load({
                let config = config.clone();
                let event_tx = event_tx.clone();
                move |path, content| {
                    Self::handle_config_load(&config, &event_tx, path, content);
                }
            })
            .on_change({
                let config = config.clone();
                let event_tx = event_tx.clone();
                move |path, content| {
                    Self::handle_config_change(&config, &event_tx, path, content);
                }
            })
            .on_error({
                let event_tx = event_tx.clone();
                move |path, error| {
                    Self::handle_config_error(&event_tx, path, error);
                }
            });

        // Start watching the file
        watcher.start().await?;
        
        self.watcher = Some(watcher);
        Ok(())
    }

    /// Get the current configuration
    pub fn get_config(&self) -> Option<NiriBarConfig> {
        self.config.lock().unwrap().clone()
    }

    /// Get a reference to the current configuration
    pub fn get_config_ref(&self) -> Option<std::sync::MutexGuard<'_, Option<NiriBarConfig>>> {
        Some(self.config.lock().unwrap())
    }

    /// Subscribe to configuration events
    pub fn subscribe(&self) -> broadcast::Receiver<ConfigEvent> {
        self.event_tx.subscribe()
    }

    /// Handle initial configuration load
    fn handle_config_load(
        config: &Arc<Mutex<Option<NiriBarConfig>>>,
        event_tx: &broadcast::Sender<ConfigEvent>,
        path: std::path::PathBuf,
        content: Vec<u8>,
    ) {
        log::info!("ConfigManager: Loading configuration from {:?}", path);
        
        match Self::parse_config(&content) {
            Ok(new_config) => {
                log::info!("ConfigManager: Configuration loaded successfully");
                
                // Update the configuration
                {
                    let mut config_guard = config.lock().unwrap();
                    *config_guard = Some(new_config.clone());
                }
                
                // Emit loaded event
                let _ = event_tx.send(ConfigEvent::Loaded(new_config));
            }
            Err(e) => {
                log::error!("ConfigManager: Failed to parse configuration from {:?}: {}", path, e);
                
                // Emit error event
                let _ = event_tx.send(ConfigEvent::Error(format!(
                    "Failed to parse configuration from {:?}: {}",
                    path, e
                )));
            }
        }
    }

    /// Handle configuration change
    fn handle_config_change(
        config: &Arc<Mutex<Option<NiriBarConfig>>>,
        event_tx: &broadcast::Sender<ConfigEvent>,
        path: std::path::PathBuf,
        content: Vec<u8>,
    ) {
        log::info!("ConfigManager: Configuration file changed, reloading...");
        
        match Self::parse_config(&content) {
            Ok(new_config) => {
                log::info!("ConfigManager: Configuration updated successfully");
                
                // Update the configuration
                {
                    let mut config_guard = config.lock().unwrap();
                    *config_guard = Some(new_config.clone());
                }
                
                // Emit updated event
                let _ = event_tx.send(ConfigEvent::Updated(new_config));
            }
            Err(e) => {
                log::error!("ConfigManager: Failed to parse updated configuration from {:?}: {}", path, e);
                
                // Emit error event (don't update current config)
                let _ = event_tx.send(ConfigEvent::Error(format!(
                    "Failed to parse updated configuration from {:?}: {}",
                    path, e
                )));
            }
        }
    }

    /// Handle configuration error
    fn handle_config_error(
        event_tx: &broadcast::Sender<ConfigEvent>,
        path: std::path::PathBuf,
        error: String,
    ) {
        log::error!("ConfigManager: Configuration error for {:?}: {}", path, error);
        
        let _ = event_tx.send(ConfigEvent::Error(format!(
            "Configuration error for {:?}: {}",
            path, error
        )));
    }

    /// Parse YAML content into configuration structure
    pub fn parse_config(content: &[u8]) -> Result<NiriBarConfig> {
        let content_str = String::from_utf8(content.to_vec())?;
        
        // Parse YAML
        let config: NiriBarConfig = serde_yaml::from_str(&content_str)?;
        
        // Validate against schema
        Self::validate_config(&config)?;
        
        Ok(config)
    }

    /// Validate configuration against JSON schema
    fn validate_config(config: &NiriBarConfig) -> Result<()> {
        // Load schema
        let _schema_content = include_str!("niri-bar-yaml.schema.json");
        let _schema: serde_json::Value = serde_json::from_str(_schema_content)?;
        
        // Convert config to JSON for validation
        let _config_json = serde_json::to_value(config)?;
        
        // Validate (using jsonschema crate if available, otherwise skip)
        // For now, we'll do basic validation manually
        Self::basic_validation(config)?;
        
        Ok(())
    }

    /// Basic configuration validation
    fn basic_validation(config: &NiriBarConfig) -> Result<()> {
        // Validate logging level
        let valid_levels = ["debug", "info", "warn", "error"];
        if !valid_levels.contains(&config.logging.level.as_str()) {
            return Err(anyhow::anyhow!("Invalid logging level: {}", config.logging.level));
        }
        
        // Validate logging format
        let valid_formats = ["iso8601", "simple"];
        if !valid_formats.contains(&config.logging.format.as_str()) {
            return Err(anyhow::anyhow!("Invalid logging format: {}", config.logging.format));
        }
        
        // Validate monitor patterns
        for monitor_config in &config.application.monitors {
            if monitor_config.match_pattern.is_empty() {
                return Err(anyhow::anyhow!("Monitor match pattern cannot be empty"));
            }
        }
        
        Ok(())
    }

    /// Check if a monitor matches any pattern in the application config
    pub fn is_monitor_enabled(&self, monitor_name: &str) -> bool {
        let config_guard = self.config.lock().unwrap();
        let config = match config_guard.as_ref() {
            Some(config) => config,
            None => return false,
        };

        // Check application-level monitor matching
        // Find the best (most specific) match rather than just the first match
        let mut best_match: Option<&MonitorConfig> = None;
        let mut best_specificity = 0;

        for monitor_config in &config.application.monitors {
            if Self::matches_pattern(monitor_name, &monitor_config.match_pattern) {
                let specificity = Self::pattern_specificity(&monitor_config.match_pattern);
                if specificity > best_specificity {
                    best_match = Some(monitor_config);
                    best_specificity = specificity;
                }
            }
        }

        best_match.map(|config| config.enabled).unwrap_or(false)
    }

    /// Calculate pattern specificity (higher = more specific)
    pub fn pattern_specificity(pattern: &str) -> u32 {
        if pattern == ".*" {
            return 1; // Least specific
        }
        if pattern.starts_with("^") && pattern.ends_with("$") {
            if pattern.contains(".*") {
                return 10; // Specific pattern with wildcard
            } else {
                return 100; // Exact match
            }
        }
        if pattern.contains(".*") {
            return 5; // Pattern with wildcard
        }
        50 // Other patterns
    }

    /// Get layout configuration for a specific monitor
    pub fn get_monitor_layout(&self, monitor_name: &str) -> Option<LayoutConfig> {
        let config_guard = self.config.lock().unwrap();
        let config = match config_guard.as_ref() {
            Some(config) => config,
            None => return None,
        };

        // Collect all matching monitor configs with specificity
        let mut matches: Vec<(&MonitorConfig, u32)> = config
            .application
            .monitors
            .iter()
            .filter(|m| Self::matches_pattern(monitor_name, &m.match_pattern))
            .map(|m| (m, Self::pattern_specificity(&m.match_pattern)))
            .collect();
        // Sort by specificity descending
        matches.sort_by(|a, b| b.1.cmp(&a.1));

        // Prefer the first with a non-empty layout.columns
        for (mc, _spec) in &matches {
            if let Some(layout) = mc.layout.clone() {
                if !layout.columns.is_empty() {
                    return Some(layout);
                }
            }
        }

        // Fallback: prefer a named default if present, then any
        if let Some(l) = config.application.layouts.get("three_column") {
            return Some(l.clone());
        }
        config.application.layouts.values().cloned().next()
    }

    /// Get module configuration for a specific monitor
    pub fn get_monitor_modules(&self, monitor_name: &str) -> Option<HashMap<String, ModuleConfig>> {
        let config_guard = self.config.lock().unwrap();
        let config = match config_guard.as_ref() {
            Some(config) => config,
            None => return None,
        };

        // Start from global module defaults
        let mut merged: HashMap<String, ModuleConfig> = config.application.modules.clone();

        // Overlay the most specific matching monitor's modules if present
        let mut best_match: Option<&MonitorConfig> = None;
        let mut best_specificity = 0;
        for monitor_config in &config.application.monitors {
            if Self::matches_pattern(monitor_name, &monitor_config.match_pattern) {
                let specificity = Self::pattern_specificity(&monitor_config.match_pattern);
                if specificity > best_specificity {
                    best_match = Some(monitor_config);
                    best_specificity = specificity;
                }
            }
        }
        if let Some(mc) = best_match {
            if let Some(overrides) = &mc.modules {
                for (k, v) in overrides {
                    merged.insert(k.clone(), v.clone());
                }
            }
        }

        Some(merged)
    }

    /// Get global module defaults
    pub fn get_global_modules(&self) -> Option<HashMap<String, ModuleConfig>> {
        let config_guard = self.config.lock().unwrap();
        let config = match config_guard.as_ref() {
            Some(config) => config,
            None => return None,
        };

        Some(config.application.modules.clone())
    }

    /// Get layout profiles
    pub fn get_layouts(&self) -> Option<HashMap<String, LayoutConfig>> {
        let config_guard = self.config.lock().unwrap();
        let config = match config_guard.as_ref() {
            Some(config) => config,
            None => return None,
        };

        Some(config.application.layouts.clone())
    }

    /// Simple regex pattern matching (supports basic patterns like ".*", "DP-.*", etc.)
    pub fn matches_pattern(monitor_name: &str, pattern: &str) -> bool {
        // Handle exact match patterns with ^ and $
        if pattern.starts_with("^") && pattern.ends_with("$") {
            let inner_pattern = &pattern[1..pattern.len()-1];
            // Very small subset: prefix.* pattern => starts_with(prefix)
            if let Some(pos) = inner_pattern.find(".*") {
                let prefix = &inner_pattern[..pos];
                return monitor_name.starts_with(prefix);
            }
            return monitor_name == inner_pattern;
        }
        
        // Convert simple patterns to regex
        let regex_pattern = pattern
            .replace(".", "\\.")
            .replace("*", ".*");
        
        // Simple matching for now - in production you'd want a proper regex engine
        if regex_pattern == ".*" {
            return true;
        }
        
        if pattern.ends_with(".*") {
            let prefix = &pattern[..pattern.len() - 2];
            return monitor_name.starts_with(prefix);
        }
        
        monitor_name == pattern
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}
