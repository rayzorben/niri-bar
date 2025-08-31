use gtk4 as gtk;
use once_cell::sync::Lazy;
use std::collections::HashMap;

pub mod clock;
pub mod window_title;
pub mod workspaces;

/// Trait for all modules. Each module exposes an identifier and can create its widget.
pub trait BarModule: Send + Sync {
    /// Unique module identifier, e.g., "bar.module.clock"
    fn id(&self) -> &'static str;
    /// Create a GTK widget instance for this module given merged module settings
    fn create(&self, settings: &crate::config::ModuleConfig) -> gtk::Widget;
}

type FactoryFn = fn(&crate::config::ModuleConfig) -> gtk::Widget;

static REGISTRY: Lazy<HashMap<&'static str, FactoryFn>> = Lazy::new(|| {
    let mut m: HashMap<&'static str, FactoryFn> = HashMap::new();
    // Register built-in modules
    m.insert(clock::ClockModule::IDENT, clock::ClockModule::create_widget);
    m.insert(window_title::WindowTitleModule::IDENT, window_title::WindowTitleModule::create_widget);
    m.insert(workspaces::WorkspacesModule::IDENT, workspaces::WorkspacesModule::create_widget);
    m
});

/// Resolve a module name from YAML (e.g., "clock") to an identifier (e.g., "bar.module.clock").
fn resolve_identifier(name: &str) -> String {
    format!("bar.module.{}", name)
}

/// Create a module widget dynamically based on the YAML module name and merged settings.
pub fn create_module_widget(module_name: &str, settings: &crate::config::ModuleConfig) -> Option<gtk::Widget> {
    let ident = resolve_identifier(module_name);
    REGISTRY.get(ident.as_str()).map(|factory| factory(settings))
}

