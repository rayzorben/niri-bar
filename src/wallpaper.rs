use crate::config::WallpaperConfig;
use crate::niri::WorkspaceInfo;

/// Wallpaper switcher class that handles wallpaper switching logic
pub struct WallpaperSwitcher<E = DefaultWallpaperExecutor> {
    config: WallpaperConfig,
    executor: E,
}

/// Trait for wallpaper command execution to enable testing
pub trait WallpaperCommandExecutor {
    fn execute_command(&self, command: &str) -> Result<(), std::io::Error>;
    fn check_path_exists(&self, path: &str) -> bool;
}

/// Default implementation using system commands
pub struct DefaultWallpaperExecutor;

impl WallpaperCommandExecutor for DefaultWallpaperExecutor {
    fn execute_command(&self, command: &str) -> Result<(), std::io::Error> {
        // For testing, we don't actually execute commands
        // In production, this would run the command
        log::debug!("Would execute command: {}", command);
        Ok(())
    }

    fn check_path_exists(&self, path: &str) -> bool {
        std::path::Path::new(path).exists()
    }
}

impl<E> WallpaperSwitcher<E> {
    /// Create a new wallpaper switcher with the given configuration and executor
    ///
    /// # Examples
    ///
    /// ```
    /// use niri_bar::wallpaper::{WallpaperSwitcher, DefaultWallpaperExecutor};
    /// use niri_bar::config::WallpaperConfig;
    /// use std::collections::HashMap;
    ///
    /// let config = WallpaperConfig {
    ///     default: Some("/path/to/default.jpg".to_string()),
    ///     by_workspace: HashMap::new(),
    ///     special_cmd: None,
    ///     swww_options: None,
    /// };
    ///
    /// let switcher = WallpaperSwitcher::new(config, DefaultWallpaperExecutor);
    /// // Use switcher.switch_wallpaper(&workspace) to change wallpapers
    /// ```
    pub fn new(config: WallpaperConfig, executor: E) -> Self {
        Self { config, executor }
    }
}

impl WallpaperSwitcher {
    /// Create a new wallpaper switcher with the given configuration (convenience method)
    ///
    /// # Examples
    ///
    /// ```
    /// use niri_bar::wallpaper::WallpaperSwitcher;
    /// use niri_bar::config::WallpaperConfig;
    /// use std::collections::HashMap;
    ///
    /// let config = WallpaperConfig {
    ///     default: Some("/path/to/default.jpg".to_string()),
    ///     by_workspace: HashMap::new(),
    ///     special_cmd: None,
    ///     swww_options: None,
    /// };
    ///
    /// let switcher = WallpaperSwitcher::new_default(config);
    /// // Use switcher.switch_wallpaper(&workspace) to change wallpapers
    /// ```
    pub fn new_default(config: WallpaperConfig) -> Self {
        Self::new(config, DefaultWallpaperExecutor)
    }
}

impl<E: WallpaperCommandExecutor> WallpaperSwitcher<E> {
    /// Get access to the executor (for testing and debugging)
    pub fn get_executor(&self) -> &E {
        &self.executor
    }

    /// Create a new wallpaper switcher with default executor (convenience method)
    pub fn new_default_with_config(config: WallpaperConfig) -> WallpaperSwitcher<DefaultWallpaperExecutor> {
        WallpaperSwitcher::<DefaultWallpaperExecutor>::new(config, DefaultWallpaperExecutor)
    }

    /// Switch wallpaper for the given workspace
    ///
    /// # Examples
    ///
    /// ```
    /// use niri_bar::wallpaper::WallpaperSwitcher;
    /// use niri_bar::config::WallpaperConfig;
    /// use niri_bar::niri::WorkspaceInfo;
    /// use std::collections::HashMap;
    ///
    /// let config = WallpaperConfig {
    ///     default: Some("/tmp/default.jpg".to_string()),
    ///     by_workspace: HashMap::new(),
    ///     special_cmd: None,
    ///     swww_options: None,
    /// };
    ///
    /// let switcher = WallpaperSwitcher::new_default(config);
    ///
    /// // Create a mock workspace
    /// let workspace = WorkspaceInfo {
    ///     id: 1,
    ///     idx: 1,
    ///     name: Some("workspace1".to_string()),
    ///     is_focused: true,
    /// };
    ///
    /// // Switch wallpaper for the workspace
    /// switcher.switch_wallpaper(&workspace);
    /// ```
    pub fn switch_wallpaper(&self, workspace: &WorkspaceInfo) {
        log::info!("WallpaperSwitcher: 📸 switch requested for workspace: {} ({:?})", workspace.idx, workspace.name);
        if let Some(image_path) = self.resolve_wallpaper_path(workspace) {
            // Expand tilde to home directory
            let expanded_path = self.expand_tilde(&image_path);
            log::info!("WallpaperSwitcher: Resolved image path: {} -> {}", image_path, expanded_path);
            self.apply_wallpaper_command(&expanded_path);
        } else {
            log::info!("WallpaperSwitcher: No wallpaper path resolved for workspace: {} ({:?})", workspace.idx, workspace.name);
        }
    }

    /// Resolve the wallpaper path for a given workspace
    fn resolve_wallpaper_path(&self, workspace: &WorkspaceInfo) -> Option<String> {
        // Try by name first, then by index, then default
        let key_name = workspace.name.as_deref().unwrap_or("");
        let key_idx = workspace.idx.to_string();

        self.config.by_workspace.get(key_name)
            .or_else(|| self.config.by_workspace.get(&key_idx))
            .or(self.config.default.as_ref())
            .cloned()
    }

    /// Apply wallpaper using available providers
    fn apply_wallpaper_command(&self, image_path: &str) where E: WallpaperCommandExecutor {
        // Order: special_cmd -> swww (daemon) -> swaybg -> noop
        log::info!("WallpaperSwitcher: 📸 switch requested for image: {}", image_path);

        // 1) special_cmd
        if let Some(cmd) = &self.config.special_cmd {
            log::info!("WallpaperSwitcher: 🎯 taking special_cmd path: {}", cmd);
            let prepared = cmd.replace("${current_workspace_image}", image_path);
            // best-effort split; users can wrap their command to handle complex args
            let mut parts = prepared.split_whitespace();
            if let Some(_bin) = parts.next() {
                let _args: Vec<&str> = parts.collect();
                log::debug!("WallpaperSwitcher: 🚀 executing command: {}", prepared);
                match self.executor.execute_command(&prepared) {
                    Ok(_) => log::info!("WallpaperSwitcher: ✅ applied via special_cmd: {}", prepared),
                    Err(e) => log::error!("WallpaperSwitcher: 💥 failed to execute special_cmd '{}': {}", prepared, e),
                }
            } else {
                log::warn!("WallpaperSwitcher: 🤨 special_cmd looked sus; no binary parsed: '{}'", prepared);
            }
            return;
        }

        // 2) swww if present: set first, then query (for logging/diagnostics)
        let swww_path = self.find_in_path_via_executor("swww");
        log::debug!("WallpaperSwitcher: 🔎 swww in PATH: {}", swww_path.as_deref().unwrap_or("<none>"));
        if let Some(swww) = swww_path {
            log::info!("WallpaperSwitcher: 🧪 using swww → img {}", image_path);
            // Build swww command with options
            let mut cmd_string = String::from("swww img");

            // Add swww options if configured
            if let Some(swww_opts) = &self.config.swww_options {
                cmd_string.push_str(&format!(" --transition-type {}", swww_opts.transition_type));

                // Transition duration (only if not 'simple' or 'none')
                if !matches!(swww_opts.transition_type.as_str(), "simple" | "none") {
                    cmd_string.push_str(&format!(" --transition-duration {}", swww_opts.transition_duration));
                }

                cmd_string.push_str(&format!(" --transition-step {}", swww_opts.transition_step));
                cmd_string.push_str(&format!(" --transition-fps {}", swww_opts.transition_fps));
                cmd_string.push_str(&format!(" --filter {}", swww_opts.filter));
                cmd_string.push_str(&format!(" --resize {}", swww_opts.resize));
                cmd_string.push_str(&format!(" --fill-color {}", swww_opts.fill_color));
            }

            // Add the image path
            cmd_string.push_str(&format!(" {}", image_path));

            log::debug!("WallpaperSwitcher: 🚀 executing swww command: {}", cmd_string);
            match self.executor.execute_command(&cmd_string) {
                Ok(_) => log::info!("WallpaperSwitcher: ✅ applied via swww"),
                Err(e) => log::error!("WallpaperSwitcher: 💥 failed to execute swww: {}", e),
            }
            // Query after setting for diagnostics (non-fatal), in background to avoid blocking
            let swww_clone = swww.clone();
            std::thread::spawn(move || {
                let post_query_ok = std::process::Command::new(&swww_clone)
                    .arg("query")
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);
                log::debug!("WallpaperSwitcher: 🔎 swww post-set query ok: {}", post_query_ok);
            });

            return;
        }

        // 3) swaybg if present
        let swaybg_path = self.find_in_path_via_executor("swaybg");
        log::debug!("WallpaperSwitcher: 🔎 swaybg in PATH: {}", swaybg_path.as_deref().unwrap_or("<none>"));
        if let Some(swaybg) = swaybg_path {
            log::info!("WallpaperSwitcher: 🧪 using swaybg → fill {}", image_path);
            // kill existing (best-effort), then spawn
            let _ = self.executor.execute_command("pkill swaybg");
            let swaybg_cmd = format!("{} -m fill -i {}", swaybg, image_path);
            match self.executor.execute_command(&swaybg_cmd) {
                Ok(_) => log::info!("WallpaperSwitcher: ✅ applied via swaybg"),
                Err(e) => log::error!("WallpaperSwitcher: 💥 failed to execute swaybg: {}", e),
            }
            return;
        }

        // 4) noop
        log::warn!("WallpaperSwitcher: 😴 no providers available; not switching wallpaper");
    }

    /// Find executable in PATH using the executor's path checking
    fn find_in_path_via_executor(&self, cmd: &str) -> Option<String> {
        use std::env;
        if let Some(paths) = env::var_os("PATH") {
            for p in env::split_paths(&paths) {
                let cand = p.join(cmd);
                let cand_str = cand.to_str().unwrap_or("");
                if self.executor.check_path_exists(cand_str) {
                    return Some(cand_str.to_string());
                }
            }
        }
        None
    }

    /// Expand tilde (~) to user's home directory
    fn expand_tilde(&self, path: &str) -> String {
        if let Some(stripped) = path.strip_prefix("~/")
            && let Some(home) = std::env::var_os("HOME") {
            format!("{}/{}", home.to_string_lossy(), stripped)
        } else {
            path.to_string()
        }
    }
}
