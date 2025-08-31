use gtk4 as gtk;
use gtk4::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
// no direct glib import; prefer gtk::glib to avoid version mismatches

use crate::config::ModuleConfig;
use crate::niri::{niri_bus, focus_workspace_index};

pub struct WorkspacesModule;

impl WorkspacesModule {
    pub const IDENT: &'static str = "bar.module.workspaces";

    pub fn create_widget(settings: &ModuleConfig) -> gtk::Widget {
        let show_numbers = settings.show_numbers.unwrap_or(true);
        let scroll_wrap = settings.additional.get("scroll_wraparound").and_then(|v| v.as_bool()).unwrap_or(false);

        let container = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        container.add_css_class("module-workspaces");

        // Track last focused id to pulse on change
        let last_focused = std::rc::Rc::new(std::cell::Cell::new(None::<i64>));
        // Track last snapshot to avoid unnecessary rebuilds (prevents hover flicker)
        #[allow(clippy::type_complexity)]
        let last_snapshot: Rc<RefCell<Vec<(i64, i64, Option<String>, bool)>>> =
            Rc::new(RefCell::new(Vec::new()));
        // Build initial buttons
        Self::rebuild_buttons(&container, show_numbers, &last_focused);

        // Poll Niri bus for changes; animate focus changes via CSS class
        let container_weak = container.downgrade();
        let last_focused_clone = last_focused.clone();
        let last_snapshot_clone = last_snapshot.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
            if let Some(container) = container_weak.upgrade() {
                // Compare current snapshot to last
                let bus = niri_bus();
                let list = bus.workspaces_snapshot();
                let current: Vec<(i64,i64,Option<String>,bool)> = list.iter()
                    .map(|w| (w.id, w.idx, w.name.clone(), w.is_focused)).collect();
                let changed = {
                    let mut last = last_snapshot_clone.borrow_mut();
                    if *last != current { *last = current; true } else { false }
                };
                if changed { Self::rebuild_buttons(&container, show_numbers, &last_focused_clone); }
                glib::ControlFlow::Continue
            } else {
                glib::ControlFlow::Break
            }
        });

        // Mouse scroll to cycle workspaces (throttled)
        container.add_controller({
            let busy = Rc::new(RefCell::new(false));
            let gesture = gtk::EventControllerScroll::new(
                gtk::EventControllerScrollFlags::VERTICAL | gtk::EventControllerScrollFlags::DISCRETE,
            );
            gesture.connect_scroll(move |_, _dx, dy| {
                if *busy.borrow() { return gtk::glib::Propagation::Proceed; }
                *busy.borrow_mut() = true;
                log::info!("Workspaces: 🛞 scroll dy={:.3}", dy);
                if dy < 0.0
                    && let Some(idx) = niri_bus().next_prev_workspace_idx(true, scroll_wrap) {
                    log::info!("Workspaces: ➡️ focus idx {}", idx);
                    let _ = focus_workspace_index(idx);
                } else if dy > 0.0
                    && let Some(idx) = niri_bus().next_prev_workspace_idx(false, scroll_wrap) {
                    log::info!("Workspaces: ⬅️ focus idx {}", idx);
                    let _ = focus_workspace_index(idx);
                }
                // release throttle shortly after to avoid flooding IPC
                let busy_reset = busy.clone();
                glib::timeout_add_local(std::time::Duration::from_millis(120), move || {
                    *busy_reset.borrow_mut() = false;
                    glib::ControlFlow::Break
                });
                gtk::glib::Propagation::Proceed
            });
            gesture
        });

        container.upcast()
    }

    fn rebuild_buttons(container: &gtk::Box, show_numbers: bool, last_focused: &std::rc::Rc<std::cell::Cell<Option<i64>>>) {
        // Clear and rebuild (simple for now; can be optimized later)
        while let Some(child) = container.first_child() { container.remove(&child); }

        let bus = niri_bus();
        let list = bus.workspaces_snapshot();

        for ws in list.iter() {
            let label_text = if show_numbers {
                format!("{}", ws.idx)
            } else {
                ws.name.clone().unwrap_or_else(|| format!("{}", ws.idx))
            };
            let btn = gtk::Button::with_label(&label_text);
            btn.add_css_class("workspace-pill");
            if ws.is_focused {
                btn.add_css_class("active");
                if last_focused.get() != Some(ws.id) {
                    // Pulse on focus change
                    btn.add_css_class("pulse");
                    let btn_weak = btn.downgrade();
                    glib::timeout_add_local(std::time::Duration::from_millis(260), move || {
                        if let Some(btn) = btn_weak.upgrade() { btn.remove_css_class("pulse"); }
                        glib::ControlFlow::Break
                    });
                    last_focused.set(Some(ws.id));
                }
            }

            let target_idx = ws.idx;
            btn.connect_clicked(move |_| {
                let _ = focus_workspace_index(target_idx);
            });

            container.append(&btn);
        }
    }
}


