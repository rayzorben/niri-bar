use niri_bar_new::file_watcher::FileWatcher;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("File Watcher Example");
    println!("===================");
    
    // Create a file watcher for a configuration file with default search paths
    let mut watcher = FileWatcher::new("config.yaml")
        .on_load(|path, content| {
            println!("✅ File loaded: {:?} ({} bytes)", path, content.len());
            if let Ok(content_str) = String::from_utf8(content) {
                println!("📄 Content: {}", content_str);
            }
        })
        .on_change(|path, content| {
            println!("🔄 File changed: {:?} ({} bytes)", path, content.len());
            if let Ok(content_str) = String::from_utf8(content) {
                println!("📄 New content: {}", content_str);
            }
        })
        .on_delete(|path| {
            println!("🗑️  File deleted: {:?}", path);
        })
        .on_error(|path, error| {
            eprintln!("❌ Error with file {:?}: {}", path, error);
        });

    println!("Starting to watch file: {}", watcher.filename());
    println!("Search paths: {:?}", watcher.search_paths());
    println!();
    println!("Instructions:");
    println!("1. Create a file named 'config.yaml' in ~/.config/niri-bar/ or current directory");
    println!("2. Modify the file to see change events");
    println!("3. Delete the file to see delete events");
    println!("4. The watcher will run for 30 seconds then exit");
    println!();
    
    // Start watching with a 30-second timeout for demonstration
    watcher.start_with_timeout(Duration::from_secs(30)).await?;
    
    println!("File watcher finished.");
    Ok(())
}
