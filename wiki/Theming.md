# Theming

Themes live in `themes/` and are applied via a `CssProvider` per bar.

Built-in themes:
- `wombat.css`
- `solarized.css`
- `dracula.css`

Selectors:
- Per-monitor: `.monitor-<name>`
- Per-column: `.column`, `.column-<safe_name>`, `.column-outline`
- Per-module: `.module-<name>` (e.g., `.module-clock`, `.module-workspaces`)

Hot-reload:
- Editing `niri-bar.yaml` or any theme file updates the bar immediately.
- Use CSS transitions/animations only; never hardcode animations in Rust.

