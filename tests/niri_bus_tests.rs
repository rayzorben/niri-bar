use niri_bar::niri::niri_bus;

#[test]
fn test_niri_bus_initial_focus_and_title() {
    let bus = niri_bus();

    // Seed workspaces with one focused and an active window id
    bus.handle_json_line(
        "{\"WorkspacesChanged\":{\"workspaces\":[{\"id\":1,\"idx\":1,\"name\":\"one\",\"output\":\"eDP-1\",\"is_urgent\":false,\"is_active\":true,\"is_focused\":true,\"active_window_id\":42}]}}",
    );

    // Provide windows list with the active window title
    bus.handle_json_line(
        "{\"WindowsChanged\":{\"windows\":[{\"id\":42,\"title\":\"Hello World\",\"app_id\":\"app\",\"pid\":1,\"workspace_id\":1,\"is_focused\":true,\"is_floating\":false,\"is_urgent\":false}]}}",
    );

    // Title should be immediately available
    assert_eq!(bus.current_title(), "Hello World");
}

#[test]
fn test_niri_bus_workspace_activation_and_scroll_like_changes() {
    let bus = niri_bus();

    // Seed three workspaces, focus idx 1
    bus.handle_json_line(
        "{\"WorkspacesChanged\":{\"workspaces\":[{\"id\":1,\"idx\":1,\"name\":\"one\",\"output\":\"eDP-1\",\"is_urgent\":false,\"is_active\":true,\"is_focused\":true,\"active_window_id\":null},{\"id\":2,\"idx\":2,\"name\":\"two\",\"output\":\"eDP-1\",\"is_urgent\":false,\"is_active\":false,\"is_focused\":false,\"active_window_id\":null},{\"id\":3,\"idx\":3,\"name\":\"three\",\"output\":\"eDP-1\",\"is_urgent\":false,\"is_active\":false,\"is_focused\":false,\"active_window_id\":null}]}}",
    );

    // Activate workspace id=2
    bus.handle_json_line("{\"WorkspaceActivated\":{\"id\":2,\"focused\":true}} ");
    let list = bus.workspaces_snapshot();
    assert!(list.iter().any(|w| w.id == 2 && w.is_focused));
    assert!(list.iter().any(|w| w.id == 1 && !w.is_focused));
}

