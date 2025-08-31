# Modules

Modules are registered in a static registry and addressed by id `bar.module.<name>`.

Common rules:
- Independent execution; one module failing should not affect others.
- Each module has YAML config (merged from global + monitor) and CSS hooks.

Clock
- Config: `format` (single strftime format). Updates every 1s.

Window Title
- Reads focused window title from `NiriBus`.
- Immediate title on initial `WorkspacesChanged` using `is_focused`.

Workspaces
- Buttons per workspace (idx or name). Click to focus. Scroll to next/prev.
- `scroll_wraparound` (bool) option.
- CSS classes: `.workspace-pill`, `.active`, `.pulse`.

