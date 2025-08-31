# Architecture

This bar is event-driven, config-first, and CSS-themed. High-level components:

- Application: GTK app lifecycle, monitor discovery, hot-reload wiring.
- ConfigManager: Parses/validates `niri-bar.yaml`, broadcasts updates.
- FileWatcher: Event-driven file change notifications (YAML + themes/).
- Monitor: Represents a physical output; owns a `Bar`.
- Bar: GTK4 layer-shell window; creates columns and injects module widgets.
- Modules: Dynamic registry (`bar.module.*`) for independent widgets.
- Niri IPC + NiriBus: Persistent event stream reader + short-lived request sender; `NiriBus` caches state for windows/workspaces/focus.
- Logger: Structured logs in playful 1990s high-school slang.

Threads/async:
- IPC reading on a background thread updates `NiriBus`.
- GTK main thread polls `NiriBus` (timers) and handles UI only.
- File watching posts to GTK via GLib channel; no polling loops.

