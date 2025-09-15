use gtk4 as gtk;
use gtk4::prelude::*;
// use gdk_pixbuf::Pixbuf;
use anyhow::Result;
use glib;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc;

// Type aliases for complex types
type WindowColumnData = (i64, i64, f64, f64, String, bool); // (win_id, y_index, w_px, h_px, title, is_focused)
type WindowColumnMap = std::collections::BTreeMap<i64, Vec<WindowColumnData>>;
type OrderedColumns = Vec<(i64, Vec<WindowColumnData>)>;

use crate::config::ModuleConfig;
use crate::niri::niri_bus;

/// Represents a window with its layout information for viewport rendering
#[derive(Debug, Clone)]
pub struct WindowLayout {
    pub id: i64,
    pub title: String,
    pub workspace_id: i64,
    pub is_focused: bool,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Screen capture manager (simplified version without actual PipeWire integration)
pub struct ScreenCapture {
    _placeholder: bool,
}

impl Default for ScreenCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenCapture {
    pub fn new() -> Self {
        Self { _placeholder: true }
    }

    /// Start screen capture (placeholder implementation)
    pub async fn start_capture(&self) -> Result<()> {
        log::info!("Viewport: Screen capture would start here (placeholder)");
        Ok(())
    }

    pub fn stop_capture(&self) {
        log::info!("Viewport: Screen capture would stop here (placeholder)");
    }

    pub fn get_current_frame(&self) -> Option<()> {
        // Placeholder - would return actual frame data
        None
    }
}

/// Viewport module that displays a live view of the current workspace
pub struct ViewportModule;

impl ViewportModule {
    pub const IDENT: &'static str = "bar.module.viewport";

    pub fn create_widget(settings: &ModuleConfig) -> gtk::Widget {
        let _show_window_titles = settings.show_window_titles.unwrap_or(true);
        let highlight_focused = settings.highlight_focused.unwrap_or(true);
        // update_rate_ms removed; event-driven updates via NiriBus notifications

        // Create the main container
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        container.add_css_class("module-viewport");
        container.set_hexpand(false);
        container.set_vexpand(true);

        // Create the drawing area for the viewport
        let drawing_area = gtk::DrawingArea::new();
        drawing_area.add_css_class("viewport-canvas");
        drawing_area.set_hexpand(false);
        drawing_area.set_vexpand(true);

        // Set initial size
        if let Some(w) = settings.width {
            drawing_area.set_size_request(w.max(20), -1);
        } else {
            // Width will be calculated based on workspace aspect ratio
            // Height should match the bar height, width calculated to maintain workspace aspect ratio
            drawing_area.set_size_request(80, -1); // Minimum width, height will be set by bar
        }

        // Create screen capture manager
        let screen_capture = Rc::new(RefCell::new(ScreenCapture::new()));

        // Track current workspace and windows
        let current_workspace_id = Rc::new(RefCell::new(None::<i64>));
        let window_layouts = Rc::new(RefCell::new(HashMap::<i64, WindowLayout>::new()));
        let focused_window_id = Rc::new(RefCell::new(None::<i64>));

        // Set up drawing function
        {
            let window_layouts_ref = Rc::clone(&window_layouts);
            let focused_window_ref = Rc::clone(&focused_window_id);
            let screen_capture_ref = Rc::clone(&screen_capture);
            let _drawing_area_weak = drawing_area.downgrade();

            drawing_area.set_draw_func(move |_area, cr, width, height| {
                Self::draw_viewport(
                    cr,
                    width,
                    height,
                    &window_layouts_ref,
                    &focused_window_ref,
                    &screen_capture_ref,
                    highlight_focused,
                );
            });
        }

        // Subscribe to NiriBus updates; redraw immediately on events (no polling)
        {
            let drawing_area_weak = drawing_area.downgrade();
            let current_workspace_ref = Rc::clone(&current_workspace_id);
            let window_layouts_ref = Rc::clone(&window_layouts);
            let focused_window_ref = Rc::clone(&focused_window_id);
            let screen_capture_ref = Rc::clone(&screen_capture);
            let fixed_width_opt = settings.width;

            let (tx, rx) = mpsc::channel::<()>();
            // Only register with niri_bus if not in test environment
            if !cfg!(test) {
                crate::niri::niri_bus().register_ui_listener(tx);
            }

            // Only set up event loop if not in test environment
            if !cfg!(test) {
                glib::source::idle_add_local(move || {
                    while rx.try_recv().is_ok() {}
                    if let Some(area) = drawing_area_weak.upgrade() {
                        if let Some((workspace_width, workspace_height)) =
                            Self::update_viewport_state(
                                &current_workspace_ref,
                                &window_layouts_ref,
                                &focused_window_ref,
                                &screen_capture_ref,
                            )
                            && fixed_width_opt.is_none()
                        {
                                let current_height = area.allocated_height() as f64;
                                if current_height > 0.0 {
                                    let workspace_aspect_ratio = workspace_width / workspace_height;
                                    let target_width =
                                        (current_height * workspace_aspect_ratio).round() as i32;
                                    let target_width = target_width.max(40);
                                    if (area.allocated_width() - target_width).abs() > 2 {
                                        area.set_size_request(target_width, -1);
                                        log::debug!(
                                            "Viewport: Resized to {}x{} (aspect ratio: {:.2})",
                                            target_width,
                                            current_height as i32,
                                            workspace_aspect_ratio
                                        );
                                    }
                                }
                            }
                        area.queue_draw();
                    }
                    glib::ControlFlow::Continue
                });
            }
        }

        container.append(&drawing_area);
        container.upcast()
    }

    /// Update the viewport state based on current Niri IPC data
    fn update_viewport_state(
        current_workspace_id: &Rc<RefCell<Option<i64>>>,
        window_layouts: &Rc<RefCell<HashMap<i64, WindowLayout>>>,
        focused_window_id: &Rc<RefCell<Option<i64>>>,
        screen_capture: &Rc<RefCell<ScreenCapture>>,
    ) -> Option<(f64, f64)> {
        // Returns workspace dimensions for aspect ratio calculation
        let bus = niri_bus();
        let workspaces = bus.workspaces_snapshot();

        // Find the currently focused workspace
        let focused_workspace = workspaces.iter().find(|ws| ws.is_focused);

        if let Some(workspace) = focused_workspace {
            let mut needs_capture_restart = false;

            // Check if workspace changed
            if let Ok(mut current_ws) = current_workspace_id.try_borrow_mut()
                && current_ws.as_ref() != Some(&workspace.id)
            {
                *current_ws = Some(workspace.id);
                needs_capture_restart = true;
                log::info!("Viewport: Workspace changed to {}", workspace.id);
            }

            // Update focused window from bus snapshot to be authoritative
            let workspace_windows = bus.windows_for_workspace(workspace.id);
            let focused_id = bus
                .focused_window_id_snapshot()
                .filter(|fid| workspace_windows.iter().any(|w| w.id == *fid));
            if let Ok(mut current_focused) = focused_window_id.try_borrow_mut() {
                *current_focused = focused_id;
            }

            // Get windows for current workspace and update layouts
            let workspace_windows = bus.windows_for_workspace(workspace.id);

            if let Ok(mut layouts) = window_layouts.try_borrow_mut() {
                layouts.clear();

                // Find the workspace bounds to normalize window positions
                let mut min_x = f64::MAX;
                let mut min_y = f64::MAX;
                let mut max_x = f64::MIN;
                let mut max_y = f64::MIN;

                // First pass: find workspace bounds
                for window in &workspace_windows {
                    if let Some(layout) = &window.layout {
                        let x = layout.pos_in_scrolling_layout[0];
                        let y = layout.pos_in_scrolling_layout[1];
                        let w = layout.tile_size[0];
                        let h = layout.tile_size[1];

                        min_x = min_x.min(x);
                        min_y = min_y.min(y);
                        max_x = max_x.max(x + w);
                        max_y = max_y.max(y + h);
                    }
                }

                // Calculate workspace dimensions
                let workspace_width = if max_x > min_x { max_x - min_x } else { 1920.0 }; // Default to common resolution
                let workspace_height = if max_y > min_y { max_y - min_y } else { 1080.0 };

                // Second pass: create normalized layout info using per-column grouping
                use std::collections::BTreeMap;
                let mut cols: WindowColumnMap = BTreeMap::new();

                for window in &workspace_windows {
                    if let Some(l) = &window.layout {
                        // x_index orders columns; y_index orders items within a column (top to bottom)
                        let x_index = l.pos_in_scrolling_layout[0].round() as i64;
                        let y_index = l.pos_in_scrolling_layout[1].round() as i64;
                        let w_px = l.tile_size[0];
                        let h_px = l.tile_size[1];
                        cols.entry(x_index).or_default().push((
                            window.id,
                            y_index,
                            w_px,
                            h_px,
                            window.title.clone(),
                            window.is_focused,
                        ));
                    }
                }

                // Sort columns left-to-right by x index
                let mut ordered_cols: OrderedColumns = cols.into_iter().collect();
                ordered_cols.sort_by_key(|(x, _)| *x);

                // Column widths are the max width of windows in that column
                let mut col_widths: Vec<f64> = Vec::new();
                for (_, items) in ordered_cols.iter() {
                    let w = items.iter().fold(0.0f64, |acc, it| acc.max(it.2));
                    col_widths.push(w.max(1.0));
                }
                let total_width_px: f64 = col_widths.iter().sum::<f64>().max(1.0);

                // Build normalized layouts: each column spans its normalized width; within column, stack by heights over workspace_height
                let mut x_cursor_norm = 0.0f64;
                for ((_, mut items), col_w_px) in
                    ordered_cols.into_iter().zip(col_widths.into_iter())
                {
                    // Sort items in column by y_index ascending (top to bottom)
                    items.sort_by_key(|it| it.1);

                    let col_w_norm = (col_w_px / total_width_px).clamp(0.0, 1.0);
                    let mut y_cursor_norm = 0.0f64;
                    for (win_id, _y_index, _w_px, h_px, title, is_focused) in items.into_iter() {
                        let h_norm = if workspace_height > 0.0 {
                            (h_px / workspace_height).clamp(0.0, 1.0)
                        } else {
                            1.0
                        };

                        layouts.insert(
                            win_id,
                            WindowLayout {
                                id: win_id,
                                title,
                                workspace_id: workspace.id,
                                is_focused,
                                x: x_cursor_norm,
                                y: y_cursor_norm,
                                width: col_w_norm,
                                height: h_norm,
                            },
                        );

                        y_cursor_norm += h_norm;
                    }

                    x_cursor_norm += col_w_norm;
                }

                // Return workspace dimensions for aspect ratio calculation
                return Some((workspace_width, workspace_height));
            }

            // Restart screen capture if workspace changed
            if needs_capture_restart
                && let Ok(capture) = screen_capture.try_borrow()
            {
                capture.stop_capture();

                // Start new capture asynchronously
                let capture_clone = screen_capture.clone();
                glib::spawn_future_local(async move {
                    if let Ok(capture) = capture_clone.try_borrow()
                        && let Err(e) = capture.start_capture().await
                    {
                        log::error!("Viewport: Failed to start screen capture: {}", e);
                    }
                });
            }
        }
        None
    }

    /// Draw the viewport with windows and highlights
    fn draw_viewport(
        cr: &cairo::Context,
        width: i32,
        height: i32,
        window_layouts: &Rc<RefCell<HashMap<i64, WindowLayout>>>,
        focused_window_id: &Rc<RefCell<Option<i64>>>,
        _screen_capture: &Rc<RefCell<ScreenCapture>>,
        highlight_focused: bool,
    ) {
        let width_f = width as f64;
        let height_f = height as f64;

        // Set up the drawing context
        cr.set_antialias(cairo::Antialias::Best);

        // Clear the background with a dark workspace color
        cr.set_source_rgba(0.15, 0.15, 0.15, 1.0);
        cr.paint().unwrap();

        // Draw a subtle workspace background pattern
        let pattern = cairo::LinearGradient::new(0.0, 0.0, width_f, height_f);
        pattern.add_color_stop_rgba(0.0, 0.18, 0.20, 0.25, 1.0);
        pattern.add_color_stop_rgba(1.0, 0.12, 0.14, 0.18, 1.0);
        cr.set_source(&pattern).unwrap();
        cr.paint().unwrap();

        // Draw window overlays and highlights
        if let Ok(layouts) = window_layouts.try_borrow() {
            let focused_id = focused_window_id.try_borrow().ok().and_then(|f| *f);

            // Sort windows by focus state (focused window on top)
            let mut sorted_layouts: Vec<_> = layouts.values().collect();
            sorted_layouts.sort_by(|a, b| {
                let a_focused = Some(a.id) == focused_id;
                let b_focused = Some(b.id) == focused_id;
                b_focused.cmp(&a_focused) // Focused windows last (drawn on top)
            });

            for layout in sorted_layouts {
                let x = (layout.x * width_f).round();
                let y = (layout.y * height_f).round();
                let w = (layout.width * width_f).round().max(1.0); // Ensure minimum 1px width
                let h = (layout.height * height_f).round().max(1.0); // Ensure minimum 1px height

                let is_focused = highlight_focused && Some(layout.id) == focused_id;

                // Draw window background
                if is_focused {
                    // Focused window background - slightly brighter
                    cr.set_source_rgba(0.25, 0.35, 0.45, 0.9);
                } else {
                    // Normal window background
                    cr.set_source_rgba(0.20, 0.25, 0.30, 0.8);
                }
                cr.rectangle(x, y, w, h);
                cr.fill().unwrap();

                // Draw window border
                cr.set_line_width(if is_focused { 1.5 } else { 1.0 });
                if is_focused {
                    // Focused window border - bright blue
                    cr.set_source_rgba(0.4, 0.8, 1.0, 1.0);
                } else {
                    // Normal window border - subtle gray
                    cr.set_source_rgba(0.6, 0.6, 0.6, 0.7);
                }

                cr.rectangle(x, y, w, h);
                cr.stroke().unwrap();

                // Draw window title if there's enough space (at least 20x8 pixels)
                if h >= 8.0 && w >= 20.0 && !layout.title.is_empty() {
                    // Calculate appropriate font size based on available space
                    let font_size = (h / 3.0).clamp(4.0, 8.0);

                    cr.select_font_face(
                        "Sans",
                        cairo::FontSlant::Normal,
                        cairo::FontWeight::Normal,
                    );
                    cr.set_font_size(font_size);

                    let text_extents = cr.text_extents(&layout.title).unwrap();

                    // Only draw text if it fits
                    if text_extents.width() <= w - 4.0 && text_extents.height() <= h - 2.0 {
                        let text_x = x + (w - text_extents.width()) / 2.0; // Center horizontally
                        let text_y = y + (h + text_extents.height()) / 2.0; // Center vertically

                        // Draw text with slight outline for visibility
                        cr.set_source_rgba(0.0, 0.0, 0.0, 0.8); // Dark outline
                        for dx in [-0.5, 0.0, 0.5] {
                            for dy in [-0.5, 0.0, 0.5] {
                                if dx != 0.0 || dy != 0.0 {
                                    cr.move_to(text_x + dx, text_y + dy);
                                    cr.show_text(&layout.title).unwrap();
                                }
                            }
                        }

                        // Draw main text
                        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
                        cr.move_to(text_x, text_y);
                        cr.show_text(&layout.title).unwrap();
                    }
                }
            }
        }
    }
}
