use std::path::PathBuf;
use tokio::sync::watch;
use notify::{Watcher, RecursiveMode, Event};
use anyhow::Result;


/// Callback function type for file events
pub type FileEventCallback = Box<dyn Fn(PathBuf, Vec<u8>) + Send + Sync>;

/// Events that can be triggered by the file watcher
#[derive(Debug, Clone)]
pub enum FileEvent {
    /// File was loaded initially
    Loaded(PathBuf, Vec<u8>),
    /// File was modified
    Changed(PathBuf, Vec<u8>),
    /// File was deleted
    Deleted(PathBuf),
    /// An error occurred
    Error(PathBuf, String),
}

/// A generic file watcher that monitors a file for changes and notifies via callbacks
pub struct FileWatcher {
    filename: String,
    search_paths: Vec<PathBuf>,
    actual_path: Option<PathBuf>,
    on_load: Option<FileEventCallback>,
    on_change: Option<FileEventCallback>,
    on_delete: Option<Box<dyn Fn(PathBuf) + Send + Sync>>,
    on_error: Option<Box<dyn Fn(PathBuf, String) + Send + Sync>>,
}

impl FileWatcher {
    /// Create a new file watcher with default search paths
    pub fn new<P: Into<String>>(filename: P) -> Self {
        let filename = filename.into();
        
        // Default search paths: ~/.config/niri-bar and current directory
        let mut search_paths = Vec::new();
        
        // Add ~/.config/niri-bar
        if let Some(home) = dirs::home_dir() {
            search_paths.push(home.join(".config").join("niri-bar"));
        }
        
        // Add current directory
        search_paths.push(PathBuf::from("."));
        
        Self {
            filename,
            search_paths,
            actual_path: None,
            on_load: None,
            on_change: None,
            on_delete: None,
            on_error: None,
        }
    }

    /// Create a new file watcher with custom search paths
    pub fn with_search_paths<P: Into<String>, Q: Into<PathBuf>>(filename: P, search_paths: Vec<Q>) -> Self {
        let filename = filename.into();
        let search_paths: Vec<PathBuf> = search_paths.into_iter().map(|p| p.into()).collect();
        
        Self {
            filename,
            search_paths,
            actual_path: None,
            on_load: None,
            on_change: None,
            on_delete: None,
            on_error: None,
        }
    }

    /// Find the actual file path by searching through the search paths
    async fn find_file(&mut self) -> Result<Option<PathBuf>> {
        for search_path in &self.search_paths {
            let file_path = search_path.join(&self.filename);
            if tokio::fs::try_exists(&file_path).await? {
                return Ok(Some(file_path));
            }
        }
        Ok(None)
    }

    /// Set callback for when file is initially loaded
    pub fn on_load<F>(mut self, callback: F) -> Self 
    where 
        F: Fn(PathBuf, Vec<u8>) + Send + Sync + 'static 
    {
        self.on_load = Some(Box::new(callback));
        self
    }

    /// Set callback for when file is modified
    pub fn on_change<F>(mut self, callback: F) -> Self 
    where 
        F: Fn(PathBuf, Vec<u8>) + Send + Sync + 'static 
    {
        self.on_change = Some(Box::new(callback));
        self
    }

    /// Set callback for when file is deleted
    pub fn on_delete<F>(mut self, callback: F) -> Self 
    where 
        F: Fn(PathBuf) + Send + Sync + 'static 
    {
        self.on_delete = Some(Box::new(callback));
        self
    }

    /// Set callback for when an error occurs
    pub fn on_error<F>(mut self, callback: F) -> Self 
    where 
        F: Fn(PathBuf, String) + Send + Sync + 'static 
    {
        self.on_error = Some(Box::new(callback));
        self
    }

    /// Start watching the file (runs indefinitely)
    pub async fn start(&mut self) -> Result<()> {
        // Find the actual file path
        if let Some(file_path) = self.find_file().await? {
            self.actual_path = Some(file_path.clone());
            self.start_watching_file(file_path).await?;
        } else {
            // File not found in any search path
            if let Some(callback) = &self.on_error {
                callback(PathBuf::from(&self.filename), format!("File '{}' not found in any search path", self.filename));
            }
        }
        Ok(())
    }

    /// Start watching the file for a limited time (useful for testing)
    pub async fn start_with_timeout(&mut self, timeout: std::time::Duration) -> Result<()> {
        // Find the actual file path
        if let Some(file_path) = self.find_file().await? {
            self.actual_path = Some(file_path.clone());
            self.start_watching_file_with_timeout(file_path, timeout).await?;
        } else {
            // File not found in any search path
            if let Some(callback) = &self.on_error {
                callback(PathBuf::from(&self.filename), format!("File '{}' not found in any search path", self.filename));
            }
        }
        Ok(())
    }

    /// Start watching a specific file path
    async fn start_watching_file(&mut self, file_path: PathBuf) -> Result<()> {
        // Load the file initially
        self.load_file(&file_path).await?;
        
        // Set up file watcher with proper configuration
        let path = file_path.clone();
        let on_change = self.on_change.take();
        let on_error = self.on_error.take();
        
        tokio::spawn(async move {
            let (tx, mut rx) = watch::channel(());
            
            let mut watcher = notify::recommended_watcher(
                move |res| {
                    if let Ok(Event { .. }) = res {
                        let _ = tx.send(());
                    }
                },
            ).unwrap();

            // Watch the file's parent directory for changes
            if let Some(parent) = path.parent() {
                watcher.watch(parent, RecursiveMode::NonRecursive).unwrap();
            }
            
            // Process events
            while rx.changed().await.is_ok() {
                if let Ok(content) = tokio::fs::read(&path).await {
                    if let Some(callback) = &on_change {
                        callback(path.clone(), content);
                    }
                } else if let Some(callback) = &on_error {
                    callback(path.clone(), "Failed to read file after change".to_string());
                }
            }
        });
        
        // Return immediately - the spawned task will continue running
        Ok(())
    }

    /// Start watching a specific file path with timeout
    async fn start_watching_file_with_timeout(&mut self, file_path: PathBuf, timeout: std::time::Duration) -> Result<()> {
        // Load the file initially
        self.load_file(&file_path).await?;
        
        // Set up file watcher with proper configuration
        let path = file_path.clone();
        let on_change = self.on_change.take();
        let on_error = self.on_error.take();
        
        let handle = tokio::spawn(async move {
            let (tx, mut rx) = watch::channel(());
            
            let mut watcher = notify::recommended_watcher(
                move |res| {
                    if let Ok(Event { .. }) = res {
                        let _ = tx.send(());
                    }
                },
            ).unwrap();

            // Watch the file's parent directory for changes
            if let Some(parent) = path.parent() {
                watcher.watch(parent, RecursiveMode::NonRecursive).unwrap();
            }
            
            // Process events with timeout
            tokio::select! {
                _ = async {
                    while rx.changed().await.is_ok() {
                        if let Ok(content) = tokio::fs::read(&path).await {
                            if let Some(callback) = &on_change {
                                callback(path.clone(), content);
                            }
                        } else if let Some(callback) = &on_error {
                            callback(path.clone(), "Failed to read file after change".to_string());
                        }
                    }
                } => {},
                _ = tokio::time::sleep(timeout) => {
                    // Timeout reached
                }
            }
        });
        
        // Wait for the watcher task to complete
        let _ = handle.await;
        Ok(())
    }

    /// Load the file content
    async fn load_file(&self, file_path: &PathBuf) -> Result<()> {
        match tokio::fs::read(file_path).await {
            Ok(content) => {
                if let Some(callback) = &self.on_load {
                    callback(file_path.clone(), content);
                }
            }
            Err(e) => {
                if let Some(callback) = &self.on_error {
                    callback(file_path.clone(), e.to_string());
                }
            }
        }
        Ok(())
    }

    /// Get the filename being watched
    pub fn filename(&self) -> &str {
        &self.filename
    }

    /// Get the actual file path if found
    pub fn actual_path(&self) -> Option<&PathBuf> {
        self.actual_path.as_ref()
    }

    /// Get the search paths
    pub fn search_paths(&self) -> &[PathBuf] {
        &self.search_paths
    }
}
