use std::path::PathBuf;

use tauri::{
    LogicalPosition, LogicalSize, Manager, PhysicalPosition, Position, Size, Webview, WebviewUrl,
    Window, webview::WebviewBuilder, window::WindowBuilder,
};

use crate::{
    plugin_runtime::model::PluginRuntimeBounds,
    surface::{events, model::SurfaceMetadata},
    view::model::ViewKind,
    windowing::labels::MAIN_WINDOW_LABEL,
};

const MANAGEMENT_WINDOW_WIDTH: f64 = 820.0;
const MANAGEMENT_WINDOW_HEIGHT: f64 = 560.0;
pub const PLUGIN_RUNTIME_HEADER_HEIGHT: f64 = 68.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MonitorBounds {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WindowSize {
    width: i32,
    height: i32,
}

pub fn main_window(app: &tauri::AppHandle) -> Option<Window> {
    app.get_window(MAIN_WINDOW_LABEL)
}

pub fn create_main_host(app: &tauri::AppHandle) -> Result<Window, String> {
    if let Some(window) = main_window(app) {
        configure_space_behavior(&window);
        return Ok(window);
    }

    let window = WindowBuilder::new(app, MAIN_WINDOW_LABEL)
        .title("litools")
        .inner_size(MANAGEMENT_WINDOW_WIDTH, MANAGEMENT_WINDOW_HEIGHT)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .visible_on_all_workspaces(true)
        .build()
        .map_err(|error| error.to_string())?;
    configure_space_behavior(&window);
    Ok(window)
}

pub fn create_detached_panel_host(app: &tauri::AppHandle, label: String) -> Result<Window, String> {
    if let Some(window) = app.get_window(&label) {
        configure_space_behavior(&window);
        return Ok(window);
    }

    let window = WindowBuilder::new(app, label)
        .title("litools")
        .inner_size(MANAGEMENT_WINDOW_WIDTH, MANAGEMENT_WINDOW_HEIGHT)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .visible(false)
        .visible_on_all_workspaces(true)
        .build()
        .map_err(|error| error.to_string())?;
    configure_space_behavior(&window);
    Ok(window)
}

pub fn create_plugin_runtime_detached_host(
    app: &tauri::AppHandle,
    window_label: String,
    title: &str,
    center_on_show: bool,
) -> Result<Window, String> {
    if let Some(window) = app.get_window(&window_label) {
        configure_space_behavior(&window);
        maybe_position_on_show(&window, center_on_show);
        window.show().map_err(|error| error.to_string())?;
        window.set_focus().map_err(|error| error.to_string())?;
        return Ok(window);
    }

    let window = WindowBuilder::new(app, window_label)
        .title(title)
        .inner_size(MANAGEMENT_WINDOW_WIDTH, MANAGEMENT_WINDOW_HEIGHT)
        .resizable(true)
        .decorations(false)
        .transparent(false)
        .visible(false)
        .visible_on_all_workspaces(true)
        .build()
        .map_err(|error| error.to_string())?;
    configure_space_behavior(&window);
    maybe_position_on_show(&window, center_on_show);
    Ok(window)
}

pub fn add_plugin_runtime_webview(
    window: &Window,
    webview_label: String,
    entry_url: &str,
    initialization_script: String,
) -> Result<(Webview, PluginRuntimeBounds), String> {
    let url = tauri::Url::parse(entry_url).map_err(|error| error.to_string())?;
    let bounds = plugin_runtime_content_bounds(window)?;
    let webview = window
        .add_child(
            WebviewBuilder::new(webview_label, WebviewUrl::CustomProtocol(url))
                .initialization_script(initialization_script),
            LogicalPosition::new(bounds.x, bounds.y),
            Size::Logical(LogicalSize {
                width: bounds.width,
                height: bounds.height,
            }),
        )
        .map_err(|error| error.to_string())?;
    webview
        .set_auto_resize(false)
        .map_err(|error| error.to_string())?;
    Ok((webview, bounds))
}

pub fn add_plugin_runtime_header_webview(
    window: &Window,
    webview_label: String,
    route: &str,
) -> Result<Webview, String> {
    let width = window_inner_logical_size(window)?.width;
    let url = format!("index.html#{route}");
    let webview = window
        .add_child(
            WebviewBuilder::new(webview_label, WebviewUrl::App(PathBuf::from(url)))
                .transparent(true),
            LogicalPosition::new(0.0, 0.0),
            Size::Logical(LogicalSize {
                width,
                height: PLUGIN_RUNTIME_HEADER_HEIGHT,
            }),
        )
        .map_err(|error| error.to_string())?;
    webview
        .set_auto_resize(false)
        .map_err(|error| error.to_string())?;
    Ok(webview)
}

pub fn set_plugin_runtime_content_bounds(
    window: &Window,
    webview: &Webview,
) -> Result<PluginRuntimeBounds, String> {
    let bounds = plugin_runtime_content_bounds(window)?;
    webview
        .set_position(Position::Logical(LogicalPosition::new(bounds.x, bounds.y)))
        .map_err(|error| error.to_string())?;
    webview
        .set_size(Size::Logical(LogicalSize {
            width: bounds.width,
            height: bounds.height,
        }))
        .map_err(|error| error.to_string())?;
    Ok(bounds)
}

pub fn set_plugin_runtime_header_bounds(window: &Window, webview: &Webview) -> Result<(), String> {
    let width = window_inner_logical_size(window)?.width;
    webview
        .set_position(Position::Logical(LogicalPosition::new(0.0, 0.0)))
        .map_err(|error| error.to_string())?;
    webview
        .set_size(Size::Logical(LogicalSize {
            width,
            height: PLUGIN_RUNTIME_HEADER_HEIGHT,
        }))
        .map_err(|error| error.to_string())
}

pub fn hide_plugin_runtime_webview(webview: &Webview) -> Result<(), String> {
    webview.hide().map_err(|error| error.to_string())
}

pub fn show_plugin_runtime_webview(webview: &Webview) -> Result<(), String> {
    webview.show().map_err(|error| error.to_string())
}

pub fn plugin_runtime_content_bounds(window: &Window) -> Result<PluginRuntimeBounds, String> {
    let size = window_inner_logical_size(window)?;
    Ok(PluginRuntimeBounds {
        x: 0.0,
        y: PLUGIN_RUNTIME_HEADER_HEIGHT,
        width: size.width,
        height: (size.height - PLUGIN_RUNTIME_HEADER_HEIGHT).max(0.0),
    })
}

fn window_inner_logical_size(window: &Window) -> Result<LogicalSize<f64>, String> {
    let size = window.inner_size().map_err(|error| error.to_string())?;
    let scale_factor = window.scale_factor().map_err(|error| error.to_string())?;
    Ok(size.to_logical::<f64>(scale_factor))
}

pub fn add_surface_webview(
    window: &Window,
    metadata: &SurfaceMetadata,
    route: &str,
) -> Result<Webview, String> {
    let url = format!("index.html#{route}");
    let size = window.inner_size().map_err(|error| error.to_string())?;
    let webview = window
        .add_child(
            WebviewBuilder::new(
                metadata.webview_label.clone(),
                WebviewUrl::App(PathBuf::from(url)),
            )
            .transparent(true)
            .auto_resize(),
            LogicalPosition::new(0, 0),
            size,
        )
        .map_err(|error| error.to_string())?;
    webview
        .set_auto_resize(true)
        .map_err(|error| error.to_string())?;
    Ok(webview)
}

pub fn show_launcher_host(window: &Window, center_on_show: bool) {
    configure_space_behavior(window);
    maybe_position_on_show(window, center_on_show);
    let _ = window.show();
    let _ = window.set_focus();
}

pub fn show_panel_host(window: &Window, center_on_show: bool) {
    configure_space_behavior(window);
    let _ = set_panel_size(window);
    maybe_position_on_show(window, center_on_show);
    let _ = window.show();
    let _ = window.set_focus();
}

pub fn show_host_for_view(window: &Window, view_kind: &ViewKind, center_on_show: bool) {
    match view_kind {
        ViewKind::Launcher => show_launcher_host(window, center_on_show),
        ViewKind::Panel | ViewKind::Runtime => show_panel_host(window, center_on_show),
    }
}

pub fn hide_window(window: &Window) {
    let _ = window.hide();
}

pub fn focus_window(window: &Window) -> Result<(), String> {
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())
}

pub fn set_panel_size(window: &Window) -> Result<(), String> {
    window
        .set_size(management_size())
        .map_err(|error| error.to_string())
}

pub fn resize_main_window_height(app: &tauri::AppHandle, height: f64) -> Result<(), String> {
    if let Some(window) = main_window(app) {
        let width = window
            .outer_size()
            .map_err(|error| error.to_string())?
            .to_logical::<f64>(window.scale_factor().map_err(|error| error.to_string())?)
            .width;
        window
            .set_size(Size::Logical(LogicalSize { width, height }))
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

pub fn emit_focus_to_owned_launcher_surfaces(window: &Window) {
    for webview in window.webviews() {
        let Some(metadata) = window
            .try_state::<crate::state::AppState>()
            .and_then(|state| state.surface_metadata_for_webview_label(webview.label()))
        else {
            continue;
        };

        if metadata.host_window_label == window.label() {
            events::emit_focus_search(&webview);
        }
    }
}

fn management_size() -> Size {
    Size::Logical(LogicalSize {
        width: MANAGEMENT_WINDOW_WIDTH,
        height: MANAGEMENT_WINDOW_HEIGHT,
    })
}

fn configure_space_behavior(window: &Window) {
    let _ = window.set_visible_on_all_workspaces(true);
}

fn maybe_position_on_show(window: &Window, center_on_show: bool) {
    if !center_on_show {
        return;
    }

    if position_window_on_target_monitor(window).is_err() {
        let _ = window.center();
    }
}

fn position_window_on_target_monitor(window: &Window) -> Result<(), String> {
    let window_size = window_size(window)?;
    let target_monitor = target_monitor_bounds(window)?;
    let position = centered_window_position(target_monitor, window_size);

    window
        .set_position(Position::Physical(PhysicalPosition {
            x: position.x,
            y: position.y,
        }))
        .map_err(|error| error.to_string())
}

fn window_size(window: &Window) -> Result<WindowSize, String> {
    let size = window
        .outer_size()
        .or_else(|_| window.inner_size())
        .map_err(|error| error.to_string())?;

    Ok(WindowSize {
        width: size.width as i32,
        height: size.height as i32,
    })
}

fn target_monitor_bounds(window: &Window) -> Result<MonitorBounds, String> {
    let monitors = window
        .available_monitors()
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|monitor| MonitorBounds {
            x: monitor.position().x,
            y: monitor.position().y,
            width: monitor.size().width as i32,
            height: monitor.size().height as i32,
        })
        .collect::<Vec<_>>();

    if let Ok(cursor_position) = window.cursor_position() {
        let cursor = Point {
            x: cursor_position.x as i32,
            y: cursor_position.y as i32,
        };
        if let Some(monitor) = select_monitor_for_point(&monitors, cursor) {
            return Ok(monitor);
        }
    }

    if let Some(monitor) = window
        .current_monitor()
        .map_err(|error| error.to_string())?
    {
        return Ok(MonitorBounds {
            x: monitor.position().x,
            y: monitor.position().y,
            width: monitor.size().width as i32,
            height: monitor.size().height as i32,
        });
    }

    if let Some(monitor) = window
        .primary_monitor()
        .map_err(|error| error.to_string())?
    {
        return Ok(MonitorBounds {
            x: monitor.position().x,
            y: monitor.position().y,
            width: monitor.size().width as i32,
            height: monitor.size().height as i32,
        });
    }

    Err("no monitor available for window positioning".to_string())
}

fn select_monitor_for_point(monitors: &[MonitorBounds], point: Point) -> Option<MonitorBounds> {
    monitors
        .iter()
        .copied()
        .find(|monitor| monitor_contains_point(*monitor, point))
}

fn monitor_contains_point(monitor: MonitorBounds, point: Point) -> bool {
    point.x >= monitor.x
        && point.x < monitor.x + monitor.width
        && point.y >= monitor.y
        && point.y < monitor.y + monitor.height
}

fn centered_window_position(monitor: MonitorBounds, window: WindowSize) -> Point {
    let max_x = monitor.x + (monitor.width - window.width).max(0);
    let max_y = monitor.y + (monitor.height - window.height).max(0);
    let centered_x = monitor.x + (monitor.width - window.width) / 2;
    let centered_y = monitor.y + (monitor.height - window.height) / 2;

    Point {
        x: centered_x.clamp(monitor.x, max_x),
        y: centered_y.clamp(monitor.y, max_y),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centers_on_single_monitor() {
        assert_eq!(
            centered_window_position(
                MonitorBounds {
                    x: 0,
                    y: 0,
                    width: 1920,
                    height: 1080,
                },
                WindowSize {
                    width: 820,
                    height: 560,
                },
            ),
            Point { x: 550, y: 260 }
        );
    }

    #[test]
    fn centers_on_right_side_monitor() {
        assert_eq!(
            centered_window_position(
                MonitorBounds {
                    x: 1920,
                    y: 0,
                    width: 1920,
                    height: 1080,
                },
                WindowSize {
                    width: 820,
                    height: 560,
                },
            ),
            Point { x: 2470, y: 260 }
        );
    }

    #[test]
    fn centers_on_left_side_monitor_with_negative_x() {
        assert_eq!(
            centered_window_position(
                MonitorBounds {
                    x: -1920,
                    y: 0,
                    width: 1920,
                    height: 1080,
                },
                WindowSize {
                    width: 820,
                    height: 560,
                },
            ),
            Point { x: -1370, y: 260 }
        );
    }

    #[test]
    fn centers_on_upper_monitor_with_negative_y() {
        assert_eq!(
            centered_window_position(
                MonitorBounds {
                    x: 0,
                    y: -1080,
                    width: 1920,
                    height: 1080,
                },
                WindowSize {
                    width: 820,
                    height: 560,
                },
            ),
            Point { x: 550, y: -820 }
        );
    }

    #[test]
    fn clamps_when_window_is_larger_than_monitor() {
        assert_eq!(
            centered_window_position(
                MonitorBounds {
                    x: 100,
                    y: 200,
                    width: 640,
                    height: 480,
                },
                WindowSize {
                    width: 820,
                    height: 560,
                },
            ),
            Point { x: 100, y: 200 }
        );
    }

    #[test]
    fn detects_monitor_containing_point() {
        let monitors = [
            MonitorBounds {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
            },
            MonitorBounds {
                x: 1920,
                y: 0,
                width: 1920,
                height: 1080,
            },
        ];

        assert_eq!(
            select_monitor_for_point(&monitors, Point { x: 2200, y: 100 }),
            Some(monitors[1])
        );
    }
}
