# Niri IPC

Connections
- Read: one persistent socket for the event stream (background thread).
- Write: one short-lived socket per request (never batch different actions).

Events handled (examples):
- `WorkspacesChanged`, `WorkspaceActivated`
- `WindowsChanged`, `WindowOpenedOrChanged`, `WindowClosed`
- `WindowFocusChanged`, `WorkspaceActiveWindowChanged`

State bus (`NiriBus`)
- Caches windows, workspaces, focused window/workspace.
- Modules poll from GTK thread to remain thread-safe.

