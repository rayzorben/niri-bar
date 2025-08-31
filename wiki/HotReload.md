# Hot Reload

Goals
- Immediate updates on YAML or theme file changes.
- Never block GTK; dispatch file events to GTK via GLib channel.

Implementation
- `notify` watcher → GLib main context channel → reload handlers.
- YAML reload: re-render bars (monitor/theme/layout/module merge).
- CSS reload: re-apply CssProvider for the active theme.

