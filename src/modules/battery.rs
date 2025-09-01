use gtk4 as gtk;
use gtk4::prelude::*;

use crate::config::ModuleConfig;
use gtk::gio;

#[derive(Clone)]
struct BatteryOpts {
    show_icon: bool,
    show_percentage: bool,
    warn: u8,
    crit: u8,
    pulse: bool,
}

pub struct BatteryModule;

impl BatteryModule {
    pub const IDENT: &'static str = "bar.module.battery";

    pub fn create_widget(settings: &ModuleConfig) -> gtk::Widget {
        // Read config with safe defaults
        let device = settings
            .additional
            .get("device")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "bat0".to_string());
        let show_icon = settings
            .additional
            .get("show_icon")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let show_percentage = settings
            .show_percentage
            .unwrap_or(true);
        let pulse = settings
            .additional
            .get("pulse")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let warn = settings.warn_threshold.unwrap_or(40);
        let crit = settings.critical_threshold.unwrap_or(10);

        let opts = BatteryOpts { show_icon, show_percentage, warn, crit, pulse };

        // Root container: box with label and optional menu button
        let root = gtk::Button::new();
        root.add_css_class("module-battery");
        root.set_has_frame(true);

        // Label inside (text and optional icon via Unicode)
        let label = gtk::Label::new(None);
        label.add_css_class("battery-label");
        label.set_hexpand(true);
        label.set_halign(gtk::Align::Fill);
        root.set_child(Some(&label));

        // Popover for power profiles if available
        let popover = gtk::Popover::new();
        popover.add_css_class("battery-popover");
        let list = gtk::ListBox::new();
        list.add_css_class("battery-popover-list");
        list.set_activate_on_single_click(true);
        popover.set_child(Some(&list));
        // Attach once; avoid reparent warnings
        popover.set_has_arrow(true);
        // Parent popover to the button
        popover.set_parent(&root);

        // Detect powerprofilesctl
        let ppd_path = find_powerprofilesctl();
        let has_ppd = ppd_path.is_some();
        if has_ppd {
            // Handle row activation for setting profiles
            let pop_for_cb = popover.clone();
            let list_for_cb = list.clone();
            let ppd_for_cb = ppd_path.as_ref().unwrap().clone();
            list.connect_row_activated(move |_, row| {
                // Get profile from row data
                let profile_nn = unsafe { row.data::<String>("profile") };
                if let Some(nn) = profile_nn {
                    let p = unsafe { nn.as_ref() }.clone();
                    // Spawn command asynchronously with logging
                    let child = std::process::Command::new(&ppd_for_cb).arg("set").arg(&p).spawn();
                    match child {
                        Ok(mut c) => {
                            let p_clone = p.clone();
                            std::thread::spawn(move || {
                                let status = c.wait().unwrap();
                                if status.success() {
                                    log::info!("Yo dude, power profile set to {} like a boss! 🔋⚡", p_clone);
                                } else {
                                    log::error!("Bummer, failed to set power profile to {} - check permissions or polkit! 😞", p_clone);
                                }
                            });
                        }
                        Err(e) => {
                            log::error!("Epic fail spawning powerprofilesctl: {} - path wrong? 🔍", e);
                        }
                    }
                    // Close menu immediately
                    pop_for_cb.popdown();
                    // Refresh list immediately after
                    let list_refresh = list_for_cb.clone();
                    let ppd_refresh = ppd_for_cb.clone();
                    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
                        rebuild_power_profile_list(&list_refresh, &ppd_refresh, None);
                        glib::ControlFlow::Break
                    });
                }
            });
            // Initial populate
            if let Some(ppd) = ppd_path.as_ref() { rebuild_power_profile_list(&list, ppd, Some(popover.clone())); }
            // Toggle popover on button click
            let pop = popover.clone();
            root.connect_clicked(move |_| {
                if pop.is_visible() { pop.popdown(); } else { pop.popup(); }
            });
            // Poll power profile every 2 seconds for real-time updates (event-driven alternative not available)
            let list_poll = list.clone();
            let ppd_poll = ppd_path.as_ref().unwrap().clone();
            glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
                rebuild_power_profile_list(&list_poll, &ppd_poll, None);
                glib::ControlFlow::Continue
            });
        } else {
            // no menu shown
        }

        // Determine sysfs paths (case-insensitive bat0/BAT0, and fallback scan)
        let resolved = resolve_battery_device(&device);
        log::info!("Battery device resolved to: {:?}", resolved);
        let cap_path = resolved.join("capacity");
        let stat_path = resolved.join("status");

        // Initial render
        update_battery_label(&label, &cap_path, &stat_path, &opts);

        // Event-driven updates via gio FileMonitor (no polling, stays on GTK main loop)
        let mut monitors: Vec<gio::FileMonitor> = Vec::new();
        for p in [&cap_path, &stat_path] {
            let file = gio::File::for_path(p);
            if let Ok(mon) = file.monitor_file(gio::FileMonitorFlags::NONE, None::<&gio::Cancellable>) {
                log::info!("Monitoring battery file: {:?}", p);
                let lbl = label.clone();
                let cap_p = cap_path.clone();
                let stat_p = stat_path.clone();
                let opts_c = opts.clone();
                mon.connect_changed(move |_, _file, _other, _event| {
                    log::info!("Battery file changed: {:?}", _event);
                    update_battery_label(&lbl, &cap_p, &stat_p, &opts_c);
                });
                monitors.push(mon);
            } else {
                log::warn!("Failed to monitor battery file: {:?}", p);
            }
        }
        // keep monitors alive by attaching to widget data (unsafe per GTK API contract)
        unsafe {
            root.set_data("battery_file_monitors", monitors);
        }

        // Fallback polling every 5 seconds (event-driven not reliable on sysfs)
        let label_weak = label.downgrade();
        let cap_path_clone = cap_path.clone();
        let stat_path_clone = stat_path.clone();
        let opts_clone = opts.clone();
        glib::timeout_add_local(std::time::Duration::from_secs(5), move || {
            if let Some(lbl) = label_weak.upgrade() {
                update_battery_label(&lbl, &cap_path_clone, &stat_path_clone, &opts_clone);
                glib::ControlFlow::Continue
            } else {
                glib::ControlFlow::Break
            }
        });

        root.upcast()
    }
}

fn resolve_battery_device(preferred: &str) -> std::path::PathBuf {
    let base = std::path::Path::new("/sys/class/power_supply");
    let mut candidates = vec![preferred.to_string(), preferred.to_uppercase(), preferred.to_lowercase()];
    // Common default
    candidates.push("BAT0".to_string());
    candidates.push("bat0".to_string());
    for cand in candidates {
        let p = base.join(&cand);
        if p.exists() { return p; }
    }
    // Fallback: first BAT*
    if let Ok(read_dir) = std::fs::read_dir(base) {
        for e in read_dir.flatten() {
            if let Some(name) = e.file_name().to_str()
                && name.to_ascii_uppercase().starts_with("BAT") {
                return e.path();
            }
        }
    }
    base.join(preferred)
}

fn find_powerprofilesctl() -> Option<std::path::PathBuf> {
    use std::{env, path::PathBuf};
    for p in ["/usr/bin/powerprofilesctl", "/bin/powerprofilesctl", "/usr/local/bin/powerprofilesctl"] {
        let path = std::path::Path::new(p);
        if path.exists() { return Some(path.to_path_buf()); }
    }
    if let Some(paths) = env::var_os("PATH") {
        for part in env::split_paths(&paths) {
            let cand: PathBuf = part.join("powerprofilesctl");
            if cand.exists() { return Some(cand); }
        }
    }
    None
}

fn update_battery_label(
    label: &gtk::Label,
    capacity_path: &std::path::Path,
    status_path: &std::path::Path,
    opts: &BatteryOpts,
) {
    let mut pct: Option<u8> = None;
    let mut stat: Option<String> = None;
    if let Ok(s) = std::fs::read_to_string(capacity_path) {
        pct = s.trim().parse::<u8>().ok();
    }
    if let Ok(s) = std::fs::read_to_string(status_path) {
        stat = Some(s.trim().to_string());
    }

    let p = pct.unwrap_or(0);
    let charging = stat.as_deref() == Some("Charging");

    // Choose icon
    let icon = if opts.show_icon {
        let base = if p >= 95 { "🔋" } else if p >= 80 { "🟩" } else if p >= 50 { "🟨" } else if p >= 30 { "🟧" } else { "🟥" };
        if charging { format!("{}⚡", base) } else { base.to_string() }
    } else { String::new() };

    let txt = if opts.show_percentage { format!("{} {}%", icon, p) } else { icon };
    log::debug!("Battery update: {}%, status: {:?}, text: {}", p, stat, txt);
    label.set_text(&txt);

    // Set classes for colorization
    label.remove_css_class("battery-ok");
    label.remove_css_class("battery-warn");
    label.remove_css_class("battery-crit");
    if p <= opts.crit { label.add_css_class("battery-crit"); }
    else if p <= opts.warn { label.add_css_class("battery-warn"); }
    else { label.add_css_class("battery-ok"); }

    // Pulse behaviour
    if opts.pulse {
        let should_pulse = (p <= opts.crit) || (p <= opts.warn && p % 5 == 0);
        if should_pulse {
            label.add_css_class("pulse");
            let label_weak = label.downgrade();
            glib::timeout_add_local(std::time::Duration::from_millis(1000), move || {
                if let Some(lbl) = label_weak.upgrade() { lbl.remove_css_class("pulse"); }
                glib::ControlFlow::Break
            });
        }
    }
}

fn rebuild_power_profile_list(list: &gtk::ListBox, ppd_path: &std::path::Path, _popover: Option<gtk::Popover>) {
    while let Some(child) = list.first_child() { list.remove(&child); }
    let current = std::process::Command::new(ppd_path).arg("get").output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok()).map(|s| s.trim().to_string());
    for (name, arg) in [("Performance", "performance"), ("Balanced", "balanced"), ("Power Saver", "power-saver")] {
        let row = gtk::ListBoxRow::new();
        let arg_string = arg.to_string();
        let r_label = gtk::Label::new(Some(name));
        r_label.set_xalign(0.0);
        if current.as_deref() == Some(arg) { row.add_css_class("active"); }
        row.set_child(Some(&r_label));
        // Store profile in row data for activation callback
        unsafe { row.set_data("profile", arg_string); }
        list.append(&row);
    }
}


