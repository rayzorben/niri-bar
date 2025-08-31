# Niri Bar

Pixel-perfect, CSS-themed, configuration-driven bar for the Niri Wayland compositor.

## Requirements
- Linux + Wayland only (Niri WM). No X11/macOS/Windows.
- GTK4 + `gtk4-layer-shell`.
- Rust toolchain (stable).

## Build and Run
```bash
cargo build --release
./target/release/niri-bar-new
```

## Configure (YAML is the source of truth)
Edit `niri-bar.yaml`. Key sections:
- `application.theme`: one of `wombat`, `solarized`, `dracula` (default: `wombat`).
- `application.modules`: global module defaults (use YAML anchors/aliases).
- `application.layouts`: reusable layout profiles (columns → modules + overflow policy).
- `application.monitors`: list of regex-matched monitors with `enabled`, `layout`, `modules` overrides.

Example (excerpt):
```yaml
application:
  theme: wombat
  modules:
    clock: &clock_default { format: "%a %b %d, %Y @ %H:%M:%S", tooltip: true }
  layouts:
    three_column: &layout_three
      columns:
        left:   { modules: ["workspaces"], overflow: hide }
        center: { modules: ["window_title"], overflow: hide }
        right:  { modules: ["clock", "battery", "system"], overflow: kebab }
  monitors:
    - match: ".*"
      enabled: true
      layout: { <<: *layout_three }
```

## Theming (CSS)
- Themes live in `themes/` (`wombat.css`, `solarized.css`, `dracula.css`).
- Every monitor/column/module gets stable CSS classes/ids for granular styling.
- Hot-reload: editing `niri-bar.yaml` or any CSS in `themes/` updates the bar immediately.

## Columns
- GTK homogeneous layout ensures equal spacing and perfect centering (odd: exact center, even: symmetric).
- Column names are for CSS only; renderer uses order/count.
- Per-column overflow: `hide` (crop) or `kebab` (dropdown popover for overflowed items).

## Modules (dynamic)
Modules are loaded via a registry by name (e.g., `clock` → `bar.module.clock`).
- `clock`: single `format` string, updates every 1s independently.
- `window_title`: shows focused window title via Niri IPC state.
- `workspaces`: buttons per workspace; click to focus; wheel to next/prev; `scroll_wraparound` supported.

## Logging
- Playful 1990s high-school slang style. Configurable in `logging` section.
- Default file: `~/.local/share/niri-bar/niri-bar.log`.

## Testing & CI
- All tests live under `tests/`; none inline in `src/*` unless critical.
- Validate the real `niri-bar.yaml` against `src/niri-bar-yaml.schema.json`.
```bash
cargo test -- --test-threads=1
# Style & lint
cargo fmt -- --check
cargo clippy -- -D warnings
```

## Niri IPC
- One persistent read socket (event stream), one short-lived write socket per request (no batching).
- State cached in a central bus consumed by UI modules on the GTK thread.

## Troubleshooting
- Ensure Niri is running (Wayland) and `$NIRI_SOCKET` is set by the compositor.
- If nothing appears: check logs, validate YAML (`cargo test -p niri-bar-new -- tests/config_tests.rs`).

## Documentation
See the `wiki/` folder for detailed architecture, configuration, theming, IPC, hot-reload, testing, and logging docs.
