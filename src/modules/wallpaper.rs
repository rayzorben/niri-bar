use gtk4 as gtk;
use gtk4::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

use crate::config::ModuleConfig;
use crate::niri::niri_bus;

/// Monitor-scoped wallpaper module: applies wallpaper per monitor on workspace focus
pub struct WallpaperModule;

impl WallpaperModule {
    pub const IDENT: &'static str = "bar.module.wallpaper";

    pub fn create_widget(settings: &ModuleConfig) -> gtk::Widget {
        // Invisible controller widget; reacts to focus changes and applies wallpaper
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        container.add_css_class("module-wallpaper-controller");

        let default_wp = settings.default_wallpaper.clone();
        let map_wp = settings.wallpapers.clone().unwrap_or_default();
        let special_cmd = settings.special_cmd.clone();
        let swww_opts = settings.swww_options.clone();

        // Discover monitor connector from ancestor CSS class monitor-<connector_with_underscores>
        let monitor_conn: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
        {
            let container_weak = container.downgrade();
            let monitor_conn = monitor_conn.clone();
            gtk::glib::idle_add_local(move || {
                if let Some(w) = container_weak.upgrade() {
                    let mut cur = Some(w.upcast::<gtk::Widget>());
                    let mut hops = 0;
                    while let Some(widget) = cur {
                        // GTK4 doesn't expose listing classes; sniff via widget_name fallback
                        let name = widget.widget_name();
                        if let Some(suffix) = name.strip_prefix("monitor-") {
                            let conn = suffix.replace('_', "-");
                            *monitor_conn.borrow_mut() = Some(conn);
                            return gtk::glib::ControlFlow::Break;
                        }
                        cur = widget.parent();
                        hops += 1;
                        if hops > 8 { break; }
                    }
                }
                gtk::glib::ControlFlow::Break
            });
        }

        // Track last applied (workspace id, image path) to avoid re-issuing commands
        let last_applied: Rc<RefCell<Option<(i64, String)>>> = Rc::new(RefCell::new(None));

        // Periodically check focus changes; apply only on change
        let container_weak = container.downgrade();
        let last_applied_clone = last_applied.clone();
        gtk::glib::timeout_add_local(std::time::Duration::from_millis(150), move || {
            if container_weak.upgrade().is_some() {
                if let Some(focused) = niri_bus().workspaces_snapshot().into_iter().find(|w| w.is_focused) {
                    let key_idx = focused.idx.to_string();
                    let key_name = focused.name.clone().unwrap_or_default();
                    let target = map_wp.get(&key_idx)
                        .or_else(|| if key_name.is_empty() { None } else { map_wp.get(&key_name) })
                        .cloned()
                        .or(default_wp.clone());
                    if let Some(img) = target {
                        // Only apply when (workspace id, image) differs
                        let should_apply = {
                            let last = last_applied_clone.borrow();
                            match &*last {
                                Some((wid, last_img)) => *wid != focused.id || last_img != &img,
                                None => true,
                            }
                        };
                        if should_apply {
                            let out = monitor_conn.borrow().clone();
                            log::info!("WallpaperModule: 🎯 Switching to workspace {} -> {}", focused.idx, img);
                            Self::apply_wallpaper_command(&special_cmd, swww_opts.as_ref(), &img, out.as_deref());
                            *last_applied_clone.borrow_mut() = Some((focused.id, img));
                        }
                    }
                }
                gtk::glib::ControlFlow::Continue
            } else {
                gtk::glib::ControlFlow::Break
            }
        });

        container.upcast()
    }

    fn apply_wallpaper_command(special_cmd: &Option<String>, swww_opts: Option<&crate::config::SwwwOptions>, image_path: &str, output: Option<&str>) {
        log::info!("WallpaperModule: 🚀 Applying wallpaper: {} (output: {:?})", image_path, output);
        fn expand_tilde(path: &str) -> String {
            if let Some(stripped) = path.strip_prefix("~/")
                && let Some(home) = std::env::var_os("HOME") {
                format!("{}/{}", home.to_string_lossy(), stripped)
            } else {
                path.to_string()
            }
        }

        let img_expanded = expand_tilde(image_path);
        if !std::path::Path::new(&img_expanded).exists() {
            log::warn!("WallpaperModule: path missing: {} (from {})", img_expanded, image_path);
            return;
        }

        if let Some(cmd) = special_cmd {
            let prepared = cmd.replace("${current_workspace_image}", &img_expanded);
            let _ = std::process::Command::new("sh").arg("-c").arg(prepared).spawn();
            return;
        }

        if let Some(swww) = find_in_path("swww") {
            // Ensure daemon is initialized; if query fails, attempt init (best-effort, non-blocking)
            let daemon_ok = std::process::Command::new(&swww)
                .arg("query")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if !daemon_ok {
                let _ = std::process::Command::new(&swww).arg("init").spawn();
                // Defer applying until next tick to avoid racing the daemon startup
                return;
            }

            // Build argument list instead of shell string
            let mut cmd = std::process::Command::new(&swww);
            cmd.arg("img");
            if let Some(out) = output {
                cmd.arg("--outputs").arg(out);
            }
            if let Some(opts) = swww_opts {
                cmd.arg("--transition-type").arg(&opts.transition_type);
                if !matches!(opts.transition_type.as_str(), "simple" | "none") {
                    cmd.arg("--transition-duration").arg(format!("{}", opts.transition_duration));
                }
                cmd.arg("--transition-step").arg(format!("{}", opts.transition_step));
                cmd.arg("--transition-fps").arg(format!("{}", opts.transition_fps));
                cmd.arg("--filter").arg(&opts.filter);
                cmd.arg("--resize").arg(&opts.resize);
                cmd.arg("--fill-color").arg(&opts.fill_color);
            }
            cmd.arg(&img_expanded);
            let _ = cmd.spawn();
            return;
        }

        if let Some(swaybg) = find_in_path("swaybg") {
            let _ = std::process::Command::new("pkill").arg("swaybg").spawn();
            let mut cmd = std::process::Command::new(swaybg);
            cmd.args(["-m", "fill", "-i", &img_expanded]);
            if let Some(out) = output {
                // swaybg uses -o <OUTPUT> for output selection
                cmd.args(["-o", out]);
            }
            let _ = cmd.spawn();
        }
    }
}

fn find_in_path(cmd: &str) -> Option<String> {
    use std::env;
    if let Some(paths) = env::var_os("PATH") {
        for p in env::split_paths(&paths) {
            let cand = p.join(cmd);
            if cand.exists() { return cand.to_str().map(|s| s.to_string()); }
        }
    }
    None
}


