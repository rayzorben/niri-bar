use niri_bar::file_watcher::FileWatcher;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::fs;
use tokio::time::Duration;

#[tokio::test]
async fn test_file_watcher_creation() {
    let watcher = FileWatcher::new("test.yaml");
    assert_eq!(watcher.filename(), "test.yaml");
    assert!(!watcher.search_paths().is_empty());
}

#[tokio::test]
async fn test_file_watcher_with_custom_paths() {
    let search_paths = vec![PathBuf::from("."), PathBuf::from("/tmp")];
    let watcher = FileWatcher::with_search_paths("test.yaml", search_paths.clone());
    assert_eq!(watcher.filename(), "test.yaml");
    assert_eq!(watcher.search_paths(), search_paths.as_slice());
}

#[tokio::test]
async fn test_file_watcher_callbacks() {
    let load_called = Arc::new(Mutex::new(false));
    let change_called = Arc::new(Mutex::new(false));
    let error_called = Arc::new(Mutex::new(false));

    let load_called_clone = load_called.clone();
    let change_called_clone = change_called.clone();
    let error_called_clone = error_called.clone();

    let mut watcher = FileWatcher::with_search_paths("nonexistent.yaml", vec![PathBuf::from(".")])
        .on_load(move |_, _| {
            *load_called_clone.lock().unwrap() = true;
        })
        .on_change(move |_, _| {
            *change_called_clone.lock().unwrap() = true;
        })
        .on_error(move |_, _| {
            *error_called_clone.lock().unwrap() = true;
        });

    // Try to start watching a nonexistent file with timeout - should trigger error callback
    let _ = watcher.start_with_timeout(Duration::from_millis(100)).await;

    // The error callback should have been called since the file doesn't exist
    assert!(*error_called.lock().unwrap());
}

#[tokio::test]
async fn test_file_watcher_with_real_file() {
    let test_file = "test_config.yaml";
    let test_content = "test: value\nbar: foo";

    // Create a test file
    fs::write(test_file, test_content).await.unwrap();

    let load_called = Arc::new(Mutex::new(false));
    let load_called_clone = load_called.clone();

    let mut watcher = FileWatcher::with_search_paths(test_file, vec![PathBuf::from(".")]).on_load(
        move |_, content| {
            *load_called_clone.lock().unwrap() = true;
            let content_str = String::from_utf8(content).unwrap();
            assert_eq!(content_str, test_content);
        },
    );

    // Start watching with timeout
    let _ = watcher.start_with_timeout(Duration::from_millis(100)).await;

    // The load callback should have been called
    assert!(*load_called.lock().unwrap());

    // Clean up
    let _ = fs::remove_file(test_file).await;
}

#[tokio::test]
async fn test_file_watcher_finds_file_in_search_path() {
    let test_file = "search_test.yaml";
    let test_content = "found: true";

    // Create a test file
    fs::write(test_file, test_content).await.unwrap();

    let load_called = Arc::new(Mutex::new(false));
    let load_called_clone = load_called.clone();

    let mut watcher = FileWatcher::with_search_paths(test_file, vec![PathBuf::from(".")]).on_load(
        move |path, content| {
            *load_called_clone.lock().unwrap() = true;
            let content_str = String::from_utf8(content).unwrap();
            assert_eq!(content_str, test_content);
            assert!(path.ends_with(test_file));
        },
    );

    // Start watching with timeout
    let _ = watcher.start_with_timeout(Duration::from_millis(100)).await;

    // The load callback should have been called
    assert!(*load_called.lock().unwrap());

    // Check that the actual path was found
    assert!(watcher.actual_path().is_some());

    // Clean up
    let _ = fs::remove_file(test_file).await;
}
