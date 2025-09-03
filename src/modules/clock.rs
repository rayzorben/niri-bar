use gtk4 as gtk;
use gtk4::prelude::*;
use chrono::Local;

use crate::config::ModuleConfig;
use super::BarModule;

pub struct ClockModule;

impl ClockModule {
    pub const IDENT: &'static str = "bar.module.clock";

    pub fn create_widget(settings: &ModuleConfig) -> gtk::Widget {
        let default_fmt = "%a %b %d, %Y @ %I:%M:%S %p".to_string();
        let fmt = settings.format.clone().unwrap_or(default_fmt);

        let label = gtk::Label::new(None);
        label.add_css_class("module-clock");
        // Fill column width; alignment set by column logic
        label.set_hexpand(true);
        label.set_halign(gtk::Align::Fill);

        let now_text = Local::now().format(&fmt).to_string();
        label.set_text(&now_text);

        // Event-driven clock updates with efficient timing
        let label_weak = label.downgrade();
        let fmt_clone = fmt.clone();
        
        // Use a more efficient update mechanism
        glib::timeout_add_local(std::time::Duration::from_millis(1000), move || {
            if let Some(label) = label_weak.upgrade() {
                let text = Local::now().format(&fmt_clone).to_string();
                label.set_text(&text);
                glib::ControlFlow::Continue
            } else {
                glib::ControlFlow::Break
            }
        });

        label.upcast()
    }
}

impl BarModule for ClockModule {
    fn id(&self) -> &'static str { Self::IDENT }
    fn create(&self, settings: &ModuleConfig) -> gtk::Widget { Self::create_widget(settings) }
}


