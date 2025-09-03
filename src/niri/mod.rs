use anyhow::{Result, anyhow};
use once_cell::sync::Lazy;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use std::thread;

// use glib::MainContext; // not used currently; keep imports minimal

/// Niri IPC: manages a read (event-stream) connection and a write connection
pub struct NiriIpc {
    socket_path: String,
}

impl NiriIpc {
    pub fn new() -> Result<Self> {
        let socket_path = env::var("NIRI_SOCKET").map_err(|_| anyhow!("NIRI_SOCKET not set"))?;
        Ok(Self { socket_path })
    }

    /// Start event-stream reader; feeds lines into the NiriBus for state + UI events.
    /// Non-blocking: spawns a background task; does not touch GTK main thread directly.
    pub fn start_event_stream(&self) -> Result<()> {
        let path = self.socket_path.clone();
        thread::spawn(move || {
            match UnixStream::connect(&path) {
                Ok(mut stream) => {
                    if let Err(e) = writeln!(stream, "\"EventStream\"") {
                        eprintln!("Niri IPC: write error: {}", e);
                        return;
                    }
                    if let Err(e) = stream.flush() {
                        eprintln!("Niri IPC: flush error: {}", e);
                        return;
                    }
                    let reader = BufReader::new(stream);
                    for line in reader.lines() {
                        match line {
                            Ok(s) => {
                                // Feed global bus (and also print for debugging)
                                niri_bus().handle_json_line(&s);
                                println!("{}", s);
                            }
                            Err(e) => {
                                eprintln!("Niri IPC: read error: {}", e);
                                break;
                            }
                        }
                    }
                }
                Err(e) => eprintln!("Niri IPC: connect error: {}", e),
            }
        });
        Ok(())
    }

    /// Send a one-shot request (JSON on one line), returns nothing for now
    pub fn send_request(&self, json_line: &str) -> Result<()> {
        let path = self.socket_path.clone();
        let payload = format!("{}\n", json_line);
        log::info!("Niri IPC: ➡️ sending request: {}", json_line);
        thread::spawn(move || {
            if let Ok(mut stream) = UnixStream::connect(&path) {
                let _ = stream.write_all(payload.as_bytes());
                let _ = stream.flush();
            } else {
                eprintln!("Niri IPC: connect error while sending");
            }
        });
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub id: i64,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    pub id: i64,
    pub idx: i64,
    pub name: Option<String>,
    pub is_focused: bool,
}

/// Central bus that caches state and broadcasts UI-friendly updates
pub struct NiriBus {
    windows_by_id: Mutex<HashMap<i64, WindowInfo>>, // id -> info
    focused_window_id: Mutex<Option<i64>>,
    workspaces: Mutex<Vec<WorkspaceInfo>>,     // ordered by idx
    keyboard_layout_names: Mutex<Vec<String>>, // from KeyboardLayoutsChanged
    current_keyboard_layout_index: Mutex<Option<usize>>, // from KeyboardLayoutsChanged
    overview_is_open: Mutex<bool>,             // from OverviewOpenedOrClosed
}

impl NiriBus {
    fn new() -> Self {
        Self {
            windows_by_id: Mutex::new(HashMap::new()),
            focused_window_id: Mutex::new(None),
            workspaces: Mutex::new(Vec::new()),
            keyboard_layout_names: Mutex::new(Vec::new()),
            current_keyboard_layout_index: Mutex::new(None),
            overview_is_open: Mutex::new(false),
        }
    }

    pub fn current_title(&self) -> String {
        let focused_id = self.focused_window_id.lock().ok().and_then(|g| *g);
        if let Some(fid) = focused_id
            && let Ok(map) = self.windows_by_id.lock()
            && let Some(win) = map.get(&fid)
        {
            return win.title.clone();
        }
        String::new()
    }

    // Modules poll for title; keep as minimal helpers to avoid dead code warnings
    fn queue_broadcast_title(&self) {}

    pub fn handle_json_line(&self, line: &str) {
        // Parse JSON and update caches
        match serde_json::from_str::<JsonValue>(line) {
            Ok(json) => self.handle_json(json),
            Err(e) => eprintln!("NiriBus: JSON parse error: {} -> {}", e, line),
        }
    }

    fn handle_json(&self, json: JsonValue) {
        if let Some(obj) = json.as_object() {
            if obj.contains_key("WindowsChanged") {
                if let Some(wv) = obj
                    .get("WindowsChanged")
                    .and_then(|v| v.get("windows"))
                    .and_then(|v| v.as_array())
                {
                    self.ingest_windows_array(wv);
                }
            } else if obj.contains_key("WindowOpenedOrChanged") {
                if let Some(win) = obj
                    .get("WindowOpenedOrChanged")
                    .and_then(|v| v.get("window"))
                    .and_then(|v| v.as_object())
                {
                    self.ingest_window_object(win);
                }
            } else if obj.contains_key("WindowClosed") {
                if let Some(id) = obj
                    .get("WindowClosed")
                    .and_then(|v| v.get("id"))
                    .and_then(|v| v.as_i64())
                {
                    if let Ok(mut map) = self.windows_by_id.lock() {
                        map.remove(&id);
                    }
                    // If the closed window was focused, clear focus and broadcast
                    if let Ok(mut f) = self.focused_window_id.lock()
                        && f.map(|x| x == id).unwrap_or(false)
                    {
                        *f = None;
                    }
                    self.queue_broadcast_title();
                }
            } else if obj.contains_key("WindowFocusChanged") {
                // {"WindowFocusChanged":{"id":<id|null>}}
                let new_id_opt = obj
                    .get("WindowFocusChanged")
                    .and_then(|v| v.get("id"))
                    .and_then(|v| {
                        if v.is_null() {
                            Some(None)
                        } else {
                            v.as_i64().map(Some)
                        }
                    })
                    .flatten();
                if let Ok(mut f) = self.focused_window_id.lock() {
                    *f = new_id_opt;
                }
                self.queue_broadcast_title();
            } else if obj.contains_key("WorkspaceActiveWindowChanged") {
                // {"WorkspaceActiveWindowChanged":{"workspace_id":X,"active_window_id":Y|null}}
                let new_id_opt = obj
                    .get("WorkspaceActiveWindowChanged")
                    .and_then(|v| v.get("active_window_id"))
                    .and_then(|v| {
                        if v.is_null() {
                            Some(None)
                        } else {
                            v.as_i64().map(Some)
                        }
                    })
                    .flatten();
                if let Ok(mut f) = self.focused_window_id.lock() {
                    *f = new_id_opt;
                }
                self.queue_broadcast_title();
            } else if obj.contains_key("WorkspaceActivated") {
                // {"WorkspaceActivated":{"id":<workspace_id>,"focused":true}}
                if let Some(ws_id) = obj
                    .get("WorkspaceActivated")
                    .and_then(|v| v.get("id"))
                    .and_then(|v| v.as_i64())
                    && let Ok(mut list) = self.workspaces.lock()
                {
                    for w in list.iter_mut() {
                        w.is_focused = w.id == ws_id;
                    }
                    // Title will be driven by subsequent WindowFocusChanged; nothing to do here
                }
            } else if obj.contains_key("WorkspacesChanged") {
                // Update cached workspaces and seed focus
                if let Some(wv) = obj
                    .get("WorkspacesChanged")
                    .and_then(|v| v.get("workspaces"))
                    .and_then(|v| v.as_array())
                {
                    let mut list: Vec<WorkspaceInfo> = Vec::new();
                    let mut focused_active: Option<Option<i64>> = None;
                    for ws in wv.iter() {
                        if let Some(o) = ws.as_object() {
                            let id = o.get("id").and_then(|v| v.as_i64());
                            let idx = o.get("idx").and_then(|v| v.as_i64());
                            let name = o
                                .get("name")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            let is_focused = o
                                .get("is_focused")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);
                            if let (Some(id), Some(idx)) = (id, idx) {
                                list.push(WorkspaceInfo {
                                    id,
                                    idx,
                                    name,
                                    is_focused,
                                });
                            }
                            if is_focused {
                                let aw = o
                                    .get("active_window_id")
                                    .and_then(|v| {
                                        if v.is_null() {
                                            Some(None)
                                        } else {
                                            v.as_i64().map(Some)
                                        }
                                    })
                                    .flatten();
                                focused_active = Some(aw);
                            }
                        }
                    }
                    list.sort_by_key(|w| w.idx);
                    if let Ok(mut slot) = self.workspaces.lock() {
                        *slot = list;
                    }
                    if let Some(new_id_opt) = focused_active {
                        if let Ok(mut f) = self.focused_window_id.lock() {
                            *f = new_id_opt;
                        }
                        self.queue_broadcast_title();
                    }
                }
            } else if obj.contains_key("KeyboardLayoutsChanged") {
                // {"KeyboardLayoutsChanged":{"keyboard_layouts":{"names":[...],"current_idx":0}}}
                if let Some(kb) = obj
                    .get("KeyboardLayoutsChanged")
                    .and_then(|v| v.get("keyboard_layouts"))
                    .and_then(|v| v.as_object())
                {
                    let names: Vec<String> = kb
                        .get("names")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default();
                    let idx_opt = kb
                        .get("current_idx")
                        .and_then(|v| v.as_u64())
                        .and_then(|u| usize::try_from(u).ok());
                    if let Ok(mut slot) = self.keyboard_layout_names.lock() {
                        *slot = names;
                    }
                    if let Ok(mut cur) = self.current_keyboard_layout_index.lock() {
                        *cur = idx_opt;
                    }
                }
            } else if obj.contains_key("OverviewOpenedOrClosed") {
                // {"OverviewOpenedOrClosed":{"is_open":true}}
                if let Some(is_open) = obj
                    .get("OverviewOpenedOrClosed")
                    .and_then(|v| v.get("is_open"))
                    .and_then(|v| v.as_bool())
                    && let Ok(mut slot) = self.overview_is_open.lock()
                {
                    *slot = is_open;
                }
            }
        }
    }

    fn ingest_windows_array(&self, arr: &[JsonValue]) {
        let mut focused_id: Option<i64> = None;

        if let Ok(mut map) = self.windows_by_id.lock() {
            for w in arr.iter() {
                if let Some(o) = w.as_object()
                    && let (Some(id), Some(title)) = (
                        o.get("id").and_then(|v| v.as_i64()),
                        o.get("title").and_then(|v| v.as_str()),
                    )
                {
                    map.insert(
                        id,
                        WindowInfo {
                            id,
                            title: title.to_string(),
                        },
                    );

                    // Check if this window is focused
                    if let Some(is_focused) = o.get("is_focused").and_then(|v| v.as_bool())
                        && is_focused
                    {
                        focused_id = Some(id);
                    }
                }
            }
        }

        // Update focused window ID if we found a focused window
        if let Some(fid) = focused_id
            && let Ok(mut f) = self.focused_window_id.lock()
        {
            *f = Some(fid);
        }

        self.queue_broadcast_title();
    }

    fn ingest_window_object(&self, o: &serde_json::Map<String, JsonValue>) {
        if let (Some(id), Some(title)) = (
            o.get("id").and_then(|v| v.as_i64()),
            o.get("title").and_then(|v| v.as_str()),
        ) {
            if let Ok(mut map) = self.windows_by_id.lock() {
                map.insert(
                    id,
                    WindowInfo {
                        id,
                        title: title.to_string(),
                    },
                );
            }
            self.queue_broadcast_title();
        }
    }
}

static NIRI_BUS: Lazy<Arc<NiriBus>> = Lazy::new(|| Arc::new(NiriBus::new()));

pub fn niri_bus() -> Arc<NiriBus> {
    NIRI_BUS.clone()
}

impl NiriBus {
    pub fn workspaces_snapshot(&self) -> Vec<WorkspaceInfo> {
        self.workspaces
            .lock()
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    pub fn focused_workspace_index(&self) -> Option<usize> {
        let list = self.workspaces.lock().ok()?;
        for (i, ws) in list.iter().enumerate() {
            if ws.is_focused {
                return Some(i);
            }
        }
        None
    }

    pub fn next_prev_workspace_id(&self, forward: bool, wrap: bool) -> Option<i64> {
        let list = self.workspaces.lock().ok()?;
        if list.is_empty() {
            return None;
        }
        let mut cur = 0usize;
        for (i, ws) in list.iter().enumerate() {
            if ws.is_focused {
                cur = i;
                break;
            }
        }
        if forward {
            if cur + 1 < list.len() {
                Some(list[cur + 1].id)
            } else if wrap {
                Some(list[0].id)
            } else {
                None
            }
        } else if cur > 0 {
            Some(list[cur - 1].id)
        } else if wrap {
            Some(list[list.len() - 1].id)
        } else {
            None
        }
    }

    pub fn next_prev_workspace_idx(&self, forward: bool, wrap: bool) -> Option<i64> {
        let list = self.workspaces.lock().ok()?;
        if list.is_empty() {
            return None;
        }
        let mut cur = 0usize;
        for (i, ws) in list.iter().enumerate() {
            if ws.is_focused {
                cur = i;
                break;
            }
        }
        if forward {
            if cur + 1 < list.len() {
                Some(list[cur + 1].idx)
            } else if wrap {
                Some(list[0].idx)
            } else {
                None
            }
        } else if cur > 0 {
            Some(list[cur - 1].idx)
        } else if wrap {
            Some(list[list.len() - 1].idx)
        } else {
            None
        }
    }

    /// Snapshot of keyboard layouts state: list of names and current index (if any)
    pub fn keyboard_layouts_snapshot(&self) -> (Vec<String>, Option<usize>) {
        let names = self
            .keyboard_layout_names
            .lock()
            .map(|v| v.clone())
            .unwrap_or_default();
        let idx = self
            .current_keyboard_layout_index
            .lock()
            .ok()
            .and_then(|g| *g);
        (names, idx)
    }

    /// Whether the overview is currently open
    pub fn is_overview_open(&self) -> bool {
        self.overview_is_open.lock().map(|v| *v).unwrap_or(false)
    }

    /// Reset all internal state for testing isolation
    pub fn reset(&self) {
        if let Ok(mut windows) = self.windows_by_id.lock() {
            windows.clear();
        }
        if let Ok(mut focused) = self.focused_window_id.lock() {
            *focused = None;
        }
        if let Ok(mut workspaces) = self.workspaces.lock() {
            workspaces.clear();
        }
        if let Ok(mut names) = self.keyboard_layout_names.lock() {
            names.clear();
        }
        if let Ok(mut cur) = self.current_keyboard_layout_index.lock() {
            *cur = None;
        }
        if let Ok(mut ov) = self.overview_is_open.lock() {
            *ov = false;
        }
    }
}

/// Convenience helper to send a raw JSON request in one shot
pub fn send_json_request(line: &str) -> Result<()> {
    let ipc = NiriIpc::new()?;
    ipc.send_request(line)
}

/// Focus workspace by index via Niri IPC Action
pub fn focus_workspace_index(idx: i64) -> Result<()> {
    let payload = format!(
        "{{\"Action\":{{\"FocusWorkspace\":{{\"reference\":{{\"Index\":{}}}}}}}}}",
        idx
    );
    send_json_request(&payload)
}
