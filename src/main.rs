use niri_bar::application::Application;
use niri_bar::config::LoggingConfig;
use niri_bar::logger::NiriBarLogger;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with default configuration
    let logging_config = LoggingConfig {
        level: "debug".to_string(),
        file: "~/.local/share/niri-bar/niri-bar.log".to_string(),
        console: true,
        format: "iso8601".to_string(),
        include_file: true,
        include_line: true,
        include_class: true,
    };

    NiriBarLogger::init(logging_config.clone())?;

    // Initialize GTK
    gtk4::init()?;

    // Create and run the application
    let mut app = Application::new(logging_config)?;
    app.run()?;

    Ok(())
}
