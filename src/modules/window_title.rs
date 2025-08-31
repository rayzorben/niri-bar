use gtk4 as gtk;
use gtk4::prelude::*;

use crate::config::ModuleConfig;
use crate::niri::niri_bus;

pub struct WindowTitleModule;

impl WindowTitleModule {
    pub const IDENT: &'static str = "bar.module.window_title";

    pub fn create_widget(_settings: &ModuleConfig) -> gtk::Widget {
        let label = gtk::Label::new(None);
        label.add_css_class("module-window-title");
        label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        label.set_xalign(0.0);
        // Allow the label to shrink but keep a minimum width controlled via CSS
        label.set_hexpand(false);

        // Set initial title from bus state
        let bus = niri_bus();
        let initial = bus.current_title();
        if !initial.is_empty() {
            label.set_text(&initial);
        }

        // Poll bus state on GTK thread every 200ms (non-blocking)
        let label_weak = label.downgrade();
        glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
            let title = niri_bus().current_title();
            if let Some(label) = label_weak.upgrade() {
                label.set_text(&title);
                glib::ControlFlow::Continue
            } else {
                glib::ControlFlow::Break
            }
        });

        label.upcast()
    }
}


