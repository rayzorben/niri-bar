use gtk::prelude::*;
use gtk4 as gtk;

use crate::config::ModuleConfig;

pub struct TrayModule;

impl TrayModule {
    pub const IDENT: &'static str = "bar.module.tray";

    pub fn create_widget(_settings: &ModuleConfig) -> gtk::Widget {
        let root = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        root.add_css_class("module-tray");
        root.set_hexpand(false);
        root.set_halign(gtk::Align::End);

        // Placeholder label until SNI host is wired
        let lbl = gtk::Label::new(Some("tray"));
        root.append(&lbl);

        root.upcast()
    }
}
