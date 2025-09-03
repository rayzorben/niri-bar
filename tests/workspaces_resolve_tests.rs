use niri_bar::niri::WorkspaceInfo;
use std::collections::HashMap;

#[test]
fn test_expand_tilde_home() {
    let path = "~/Pictures/example.png";
    let expanded = {
        // reuse same logic as in module via a local helper copy
        fn expand_tilde_local(path: &str) -> String {
            if let Some(stripped) = path.strip_prefix("~/")
                && let Some(home) = std::env::var_os("HOME") {
                format!("{}/{}", home.to_string_lossy(), stripped)
            } else { path.to_string() }
        }
        expand_tilde_local(path)
    };
    assert!(expanded.starts_with(&format!("{}/", std::env::var("HOME").unwrap())));
}

#[test]
fn test_resolve_workspace_wallpaper_by_idx() {
    let ws = WorkspaceInfo { id: 10, idx: 2, name: Some("dev".into()), is_focused: false };
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("2".into(), "/tmp/idx.png".into());
    let def = Some("/tmp/default.png".into());
    let out = niri_bar::modules::workspaces::resolve_workspace_wallpaper(&ws, &map, &def);
    assert_eq!(out.as_deref(), Some("/tmp/idx.png"));
}

#[test]
fn test_resolve_workspace_wallpaper_by_name() {
    let ws = WorkspaceInfo { id: 11, idx: 9, name: Some("video".into()), is_focused: false };
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("video".into(), "/tmp/name.png".into());
    let def = Some("/tmp/default.png".into());
    let out = niri_bar::modules::workspaces::resolve_workspace_wallpaper(&ws, &map, &def);
    assert_eq!(out.as_deref(), Some("/tmp/name.png"));
}

#[test]
fn test_resolve_workspace_wallpaper_default() {
    let ws = WorkspaceInfo { id: 12, idx: 7, name: None, is_focused: false };
    let map: HashMap<String, String> = HashMap::new();
    let def = Some("/tmp/default.png".into());
    let out = niri_bar::modules::workspaces::resolve_workspace_wallpaper(&ws, &map, &def);
    assert_eq!(out.as_deref(), Some("/tmp/default.png"));
}


