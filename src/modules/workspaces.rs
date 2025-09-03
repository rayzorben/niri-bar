use gtk4 as gtk;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
// no direct glib import; prefer gtk::glib to avoid version mismatches

use crate::config::ModuleConfig;
use crate::niri::{focus_workspace_index, niri_bus};
use std::collections::HashMap;
// no mpsc needed; thumbnails come from YAML mapping only

pub struct WorkspacesModule;

impl WorkspacesModule {
    pub const IDENT: &'static str = "bar.module.workspaces";

    pub fn create_widget(settings: &ModuleConfig) -> gtk::Widget {
        let show_numbers = settings.show_numbers.unwrap_or(true);
        let show_wallpaper = settings.show_wallpaper.unwrap_or(false);
        let default_wp = settings.default_wallpaper.clone();
        let map_wp = settings.wallpapers.clone().unwrap_or_default();
        let _special_cmd = settings.special_cmd.clone();
        let scroll_wrap = settings
            .additional
            .get("scroll_wraparound")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let scroll_throttle_ms = settings
            .additional
            .get("scroll_throttle_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(50);

        let container = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        container.add_css_class("module-workspaces");

        // Track last focused id to pulse on change
        let last_focused = std::rc::Rc::new(std::cell::Cell::new(None::<i64>));
        // Track last snapshot to avoid unnecessary rebuilds (prevents hover flicker)
        #[allow(clippy::type_complexity)]
        let last_snapshot: Rc<RefCell<Vec<(i64, i64, Option<String>, bool)>>> =
            Rc::new(RefCell::new(Vec::new()));
        // Thumbnails are resolved directly from YAML mapping; no runtime capture
        // Build initial buttons
        Self::rebuild_buttons(
            &container,
            show_numbers,
            show_wallpaper,
            &last_focused,
            &map_wp,
            &default_wp,
        );

        // Poll Niri bus for changes; animate focus changes via CSS class
        let container_weak = container.downgrade();
        let last_focused_clone = last_focused.clone();
        let last_snapshot_clone = last_snapshot.clone();
        // show_wallpaper is already defined above, no need to redefine
        let map_wp = map_wp.clone();
        let default_wp = default_wp.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
            if let Some(container) = container_weak.upgrade() {
                // Compare current snapshot to last
                let bus = niri_bus();
                let list = bus.workspaces_snapshot();
                let current: Vec<(i64, i64, Option<String>, bool)> = list
                    .iter()
                    .map(|w| (w.id, w.idx, w.name.clone(), w.is_focused))
                    .collect();
                let changed = {
                    let mut last = last_snapshot_clone.borrow_mut();
                    if *last != current {
                        *last = current;
                        true
                    } else {
                        false
                    }
                };
                if changed {
                    Self::rebuild_buttons(
                        &container,
                        show_numbers,
                        show_wallpaper,
                        &last_focused_clone,
                        &map_wp,
                        &default_wp,
                    );
                }
                glib::ControlFlow::Continue
            } else {
                glib::ControlFlow::Break
            }
        });

        // Mouse scroll to cycle workspaces (optimized throttling)
        container.add_controller({
            let busy = Rc::new(RefCell::new(false));
            let last_scroll_time = Rc::new(RefCell::new(std::time::Instant::now()));
            let scroll_throttle_ms_clone = scroll_throttle_ms;
            let scroll_wrap_clone = scroll_wrap;
            let gesture = gtk::EventControllerScroll::new(
                gtk::EventControllerScrollFlags::VERTICAL
                    | gtk::EventControllerScrollFlags::DISCRETE,
            );
            gesture.connect_scroll(move |_, _dx, dy| {
                let now = std::time::Instant::now();
                let mut last_time = last_scroll_time.borrow_mut();
                let time_since_last = now.duration_since(*last_time);

                // Use configurable throttle timeout
                let throttle_duration = std::time::Duration::from_millis(scroll_throttle_ms_clone);
                if *busy.borrow() || time_since_last < throttle_duration {
                    return gtk::glib::Propagation::Proceed;
                }

                *busy.borrow_mut() = true;
                *last_time = now;

                // Only process significant scroll movements
                if dy.abs() > 0.1 {
                    // Only log significant movements to reduce noise
                    if dy.abs() > 0.8 {
                        log::debug!("Workspaces: 🛞 scroll dy={:.3}", dy);
                    }

                    let direction_up = dy < 0.0;
                    if let Some(idx) =
                        niri_bus().next_prev_workspace_idx(direction_up, scroll_wrap_clone)
                    {
                        if dy.abs() > 0.8 {
                            // Only log significant movements
                            if direction_up {
                                log::info!("Workspaces: ➡️ focus idx {}", idx);
                            } else {
                                log::info!("Workspaces: ⬅️ focus idx {}", idx);
                            }
                        }
                        let _ = focus_workspace_index(idx);
                    }
                } else {
                    // Small movement, just release the busy flag quickly
                    *busy.borrow_mut() = false;
                    return gtk::glib::Propagation::Proceed;
                }

                // Configurable throttle release for better responsiveness
                let busy_reset = busy.clone();
                let release_duration = std::time::Duration::from_millis(scroll_throttle_ms_clone);
                glib::timeout_add_local(release_duration, move || {
                    *busy_reset.borrow_mut() = false;
                    glib::ControlFlow::Break
                });
                gtk::glib::Propagation::Proceed
            });
            gesture
        });

        container.upcast()
    }

    fn rebuild_buttons(
        container: &gtk::Box,
        show_numbers: bool,
        show_wallpaper: bool,
        last_focused: &std::rc::Rc<std::cell::Cell<Option<i64>>>,
        map_wp: &HashMap<String, String>,
        default_wp: &Option<String>,
    ) {
        // Clear and rebuild (simple for now; can be optimized later)
        while let Some(child) = container.first_child() {
            container.remove(&child);
        }

        let bus = niri_bus();
        let list = bus.workspaces_snapshot();

        for ws in list.iter() {
            let label_text = if show_numbers {
                format!("{}", ws.idx)
            } else {
                // If numbers are hidden, prefer the workspace name, fallback to number only if missing
                ws.name.clone().unwrap_or_else(|| format!("{}", ws.idx))
            };
            let btn = gtk::Button::new();
            btn.add_css_class("workspace-pill");
            // Apply wallpaper directly to the button so it fills entire pill including padding
            btn.add_css_class("workspace-thumb");
            btn.set_widget_name(&format!("workspace-btn-{}", ws.id));

            // Ensure button can receive events
            btn.set_can_focus(true);
            btn.set_focusable(true);
            btn.set_sensitive(true);
            // Overlay: background wallpaper + centered label
            let overlay = gtk::Overlay::new();
            overlay.set_hexpand(true);
            overlay.set_vexpand(true);
            overlay.set_widget_name(&format!("workspace-overlay-{}", ws.id));
            overlay.add_css_class("workspace-overlay");
            if show_wallpaper {
                // Determine and apply wallpaper from YAML mapping only (no runtime capture)
                let path_opt = resolve_workspace_wallpaper(ws, map_wp, default_wp);
                if let Some(path) = path_opt {
                    set_background_image(&btn.clone().upcast::<gtk::Widget>(), &path);
                }
                // Ensure overlay stretches fully
                let filler = gtk::Box::new(gtk::Orientation::Vertical, 0);
                filler.set_hexpand(true);
                filler.set_vexpand(true);
                filler.set_halign(gtk::Align::Fill);
                filler.set_valign(gtk::Align::Fill);
                overlay.set_child(Some(&filler));
            }
            let lbl = gtk::Label::new(Some(&label_text));
            lbl.add_css_class("workspace-label");
            lbl.set_halign(gtk::Align::Fill);
            lbl.set_valign(gtk::Align::Center);
            lbl.set_xalign(0.5);
            overlay.add_overlay(&lbl);
            btn.set_child(Some(&overlay));
            if ws.is_focused {
                btn.add_css_class("active");
                if last_focused.get() != Some(ws.id) {
                    // Pulse on focus change
                    btn.add_css_class("pulse");
                    let btn_weak = btn.downgrade();
                    glib::timeout_add_local(std::time::Duration::from_millis(260), move || {
                        if let Some(btn) = btn_weak.upgrade() {
                            btn.remove_css_class("pulse");
                        }
                        glib::ControlFlow::Break
                    });
                    last_focused.set(Some(ws.id));
                }
            }

            let target_idx = ws.idx;
            let ws_id = ws.id;
            btn.connect_clicked(move |_| {
                log::info!(
                    "Workspaces: 🖱️ clicked workspace {} (idx {})",
                    ws_id,
                    target_idx
                );
                match focus_workspace_index(target_idx) {
                    Ok(_) => log::debug!(
                        "Workspaces: ✅ successfully focused workspace {}",
                        target_idx
                    ),
                    Err(e) => log::error!(
                        "Workspaces: ❌ failed to focus workspace {}: {}",
                        target_idx,
                        e
                    ),
                }
            });

            // Add a gesture click controller for better click handling
            let click_gesture = gtk::GestureClick::new();
            click_gesture.set_button(gtk::gdk::BUTTON_PRIMARY); // Left click only
            let target_idx_click = target_idx;
            let ws_id_click = ws_id;
            click_gesture.connect_pressed(move |gesture, _n_press, _x, _y| {
                log::info!(
                    "Workspaces: 🖱️ clicked workspace {} (idx {})",
                    ws_id_click,
                    target_idx_click
                );
                match focus_workspace_index(target_idx_click) {
                    Ok(_) => log::debug!(
                        "Workspaces: ✅ successfully focused workspace {}",
                        target_idx_click
                    ),
                    Err(e) => log::error!(
                        "Workspaces: ❌ failed to focus workspace {}: {}",
                        target_idx_click,
                        e
                    ),
                }
                gesture.set_state(gtk::EventSequenceState::Claimed);
            });
            btn.add_controller(click_gesture);

            container.append(&btn);
        }
    }
}

/// Helper to set a CSS background-image on a widget using a file path
fn set_background_image(widget: &gtk::Widget, path: &str) {
    // Convert to file:// URL and escape quotes for CSS
    let abs = expand_tilde(path);
    let mut uri = if abs.starts_with("file://") {
        abs
    } else {
        format!("file://{}", abs)
    };
    // Very light URL escaping for spaces; avoids CSS parser issues
    uri = uri.replace(" ", "%20");
    let css = format!(
        "#{} {{ background-image: url(\"{}\"); background-size: cover; background-position: center; background-repeat: no-repeat; }}",
        widget.widget_name(),
        uri.replace("\\", "\\\\").replace("\"", "\\\"")
    );
    let provider = gtk::CssProvider::new();
    provider.load_from_data(&css);
    let ctx = widget.style_context();
    ctx.add_provider(&provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
}

fn expand_tilde(path: &str) -> String {
    if let Some(stripped) = path.strip_prefix("~/")
        && let Some(home) = std::env::var_os("HOME")
    {
        format!("{}/{}", home.to_string_lossy(), stripped)
    } else {
        path.to_string()
    }
}

#[allow(dead_code)]
fn query_current_wallpaper_path() -> Option<String> {
    // Try swww first: `swww query`
    if let Some(swww) = find_in_path("swww")
        && let Ok(out) = std::process::Command::new(swww).arg("query").output()
        && out.status.success()
        && let Ok(txt) = String::from_utf8(out.stdout)
    {
        for line in txt.lines() {
            if let Some(idx) = line.find("image:") {
                let path = line[idx + 6..].trim();
                if !path.is_empty() {
                    return Some(path.to_string());
                }
            }
        }
    }
    // Fallback swaybg: `swaybg -p` has no query; many setups write the path to env or config.
    // Attempt common env var (non-standard):
    if let Ok(path) = std::env::var("SWAYBG_IMAGE")
        && !path.is_empty()
    {
        return Some(path);
    }
    // Attempt reading ~/.cache/wallpaper or similar common files (best-effort only)
    if let Some(home) = std::env::var_os("HOME") {
        let p = std::path::Path::new(&home).join(".cache/wallpaper");
        if p.exists()
            && let Ok(s) = std::fs::read_to_string(p)
        {
            let path = s.trim();
            if !path.is_empty() {
                return Some(path.to_string());
            }
        }
    }
    None
}

#[allow(dead_code)]
fn find_in_path(cmd: &str) -> Option<String> {
    use std::env;
    if let Some(paths) = env::var_os("PATH") {
        for p in env::split_paths(&paths) {
            let cand = p.join(cmd);
            if cand.exists() {
                return cand.to_str().map(|s| s.to_string());
            }
        }
    }
    None
}

#[allow(dead_code)]
fn apply_wallpaper_command(special_cmd: &Option<String>, image_path: &str) {
    // Order: special_cmd -> swww (daemon) -> swaybg -> noop
    // Expand tilde and validate path before issuing commands
    let img_expanded = expand_tilde(image_path);
    if !std::path::Path::new(&img_expanded).exists() {
        log::error!(
            "Workspaces: 💥 wallpaper path does not exist: {} (expanded from: {})",
            img_expanded,
            image_path
        );
        return;
    }

    if let Some(cmd) = special_cmd {
        let prepared = cmd.replace("${current_workspace_image}", &img_expanded);
        // best-effort split; users can wrap their command to handle complex args
        let mut parts = prepared.split_whitespace();
        if let Some(bin) = parts.next() {
            let args: Vec<&str> = parts.collect();
            let _ = std::process::Command::new(bin).args(args).spawn();
            return;
        }
    }
    if let Some(swww) = find_in_path("swww") {
        // Check if swww-daemon is running
        if std::process::Command::new(&swww)
            .arg("query")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            let _ = std::process::Command::new(&swww)
                .args(["img", &img_expanded])
                .spawn();
            return;
        }
    }
    if let Some(swaybg) = find_in_path("swaybg") {
        // kill existing (best-effort), then spawn
        let _ = std::process::Command::new("pkill").arg("swaybg").spawn();
        let _ = std::process::Command::new(swaybg)
            .args(["-m", "fill", "-i", &img_expanded])
            .spawn();
    }
}

pub fn resolve_workspace_wallpaper(
    workspace: &crate::niri::WorkspaceInfo,
    wallpapers: &HashMap<String, String>,
    default_wp: &Option<String>,
) -> Option<String> {
    let key_idx = workspace.idx.to_string();
    let key_name = workspace.name.clone().unwrap_or_default();

    // Try to find a specific wallpaper for this workspace
    if let Some(path) = wallpapers.get(&key_idx) {
        return Some(path.clone());
    }
    if let Some(path) = wallpapers.get(&key_name) {
        return Some(path.clone());
    }

    // Fallback to default if no specific wallpaper found
    default_wp.clone()
}
