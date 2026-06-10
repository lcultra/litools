use tauri::{LogicalPosition, Position, Window};

use crate::state::{
    AppState, LauncherMonitorFingerprint, LauncherSavedPosition, LauncherWindowPosition,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MonitorBounds {
    logical_rect: Rect,
    physical_size: WindowSize,
    scale_millis: i32,
}

impl MonitorBounds {
    fn scale_factor(self) -> f64 {
        self.scale_millis as f64 / 1000.0
    }

    fn logical_size(self) -> WindowSize {
        WindowSize {
            width: self.logical_rect.width,
            height: self.logical_rect.height,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct CursorPosition {
    x: f64,
    y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WindowSize {
    width: i32,
    height: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

pub fn position_launcher_on_show(window: &Window, state: &AppState) {
    if position_launcher_on_target_monitor(window, state).is_err() {
        state.remember_any_programmatic_launcher_move();
        let _ = window.center();
    }
}

pub fn center_window_on_show(window: &Window) {
    match centered_target_position(window) {
        Ok(position) => {
            let _ = set_window_position(window, position);
        }
        Err(_) => {
            let _ = window.center();
        }
    }
}

pub fn center_main_window_on_show(window: &Window, state: &AppState) {
    match centered_target_position(window) {
        Ok(position) => {
            state.remember_programmatic_launcher_move(LauncherWindowPosition {
                x: position.x,
                y: position.y,
            });
            let _ = set_window_position(window, position);
        }
        Err(_) => {
            state.remember_any_programmatic_launcher_move();
            let _ = window.center();
        }
    }
}

pub fn save_launcher_position(window: &Window, state: &AppState, position: LauncherWindowPosition) {
    let Ok(monitor) = window
        .current_monitor()
        .map_err(|error| error.to_string())
        .and_then(|monitor| monitor.ok_or_else(|| "current monitor not found".to_string()))
        .map(|monitor| monitor_bounds(&monitor))
        .or_else(|_| monitor_for_physical_position(window, position))
    else {
        return;
    };
    let logical_position = logical_position_on_monitor(position, monitor);

    if state.should_ignore_launcher_moved(logical_position) {
        return;
    }

    state.replace_launcher_saved_position(LauncherSavedPosition {
        monitor: monitor_fingerprint(monitor),
        position: logical_position,
    });
}

fn position_launcher_on_target_monitor(window: &Window, state: &AppState) -> Result<(), String> {
    let target_monitor = target_monitor_bounds(window)?;
    let window_size = window_logical_size(window)?;
    let target_fingerprint = monitor_fingerprint(target_monitor);
    let target_position =
        launcher_show_position(target_monitor, window_size, state.launcher_saved_position());
    if target_position.should_forget_saved_position {
        state.clear_launcher_saved_position();
    }
    let saved_position = LauncherSavedPosition {
        monitor: target_fingerprint,
        position: LauncherWindowPosition {
            x: target_position.position.x,
            y: target_position.position.y,
        },
    };

    state.replace_launcher_saved_position(saved_position);
    state.remember_programmatic_launcher_move(saved_position.position);
    set_window_position(window, target_position.position)
}

fn centered_target_position(window: &Window) -> Result<Point, String> {
    let target_monitor = target_monitor_bounds(window)?;
    let window_size = window_logical_size(window)?;
    Ok(centered_window_position(target_monitor, window_size))
}

fn set_window_position(window: &Window, position: Point) -> Result<(), String> {
    window
        .set_position(Position::Logical(LogicalPosition {
            x: position.x as f64,
            y: position.y as f64,
        }))
        .map_err(|error| error.to_string())
}

fn window_logical_size(window: &Window) -> Result<WindowSize, String> {
    let size = window
        .outer_size()
        .or_else(|_| window.inner_size())
        .map_err(|error| error.to_string())?;
    let scale_factor = window.scale_factor().map_err(|error| error.to_string())?;
    let logical_size = size.to_logical::<f64>(scale_factor);

    Ok(WindowSize {
        width: logical_size.width.round() as i32,
        height: logical_size.height.round() as i32,
    })
}

fn target_monitor_bounds(window: &Window) -> Result<MonitorBounds, String> {
    if let Ok(cursor) = window.cursor_position() {
        if let Ok(monitors) = window.available_monitors() {
            let monitors = monitors.iter().map(monitor_bounds).collect::<Vec<_>>();
            if let Some(monitor) = select_monitor_for_cursor(
                &monitors,
                CursorPosition {
                    x: cursor.x,
                    y: cursor.y,
                },
            ) {
                return Ok(monitor);
            }
        }

        if let Ok(Some(monitor)) = window.monitor_from_point(cursor.x, cursor.y) {
            return Ok(monitor_bounds(&monitor));
        }
    }

    // Fallback: monitor the window is currently on.
    if let Ok(Some(monitor)) = window.current_monitor() {
        return Ok(monitor_bounds(&monitor));
    }

    // Last resort: primary monitor.
    if let Ok(Some(monitor)) = window.primary_monitor() {
        return Ok(monitor_bounds(&monitor));
    }

    Err("no monitor available for window positioning".to_string())
}

fn monitor_for_physical_position(
    window: &Window,
    position: LauncherWindowPosition,
) -> Result<MonitorBounds, String> {
    let monitors = window
        .available_monitors()
        .map_err(|error| error.to_string())?;
    monitors
        .iter()
        .map(monitor_bounds)
        .min_by_key(|monitor| {
            let physical_x = monitor.logical_rect.x * monitor.scale_millis / 1000;
            let physical_y = monitor.logical_rect.y * monitor.scale_millis / 1000;
            (position.x - physical_x).abs() + (position.y - physical_y).abs()
        })
        .ok_or_else(|| "no monitor available for window positioning".to_string())
}

fn logical_position_on_monitor(
    position: LauncherWindowPosition,
    monitor: MonitorBounds,
) -> LauncherWindowPosition {
    let scale_factor = monitor.scale_factor();
    LauncherWindowPosition {
        x: (position.x as f64 / scale_factor).round() as i32,
        y: (position.y as f64 / scale_factor).round() as i32,
    }
}

fn select_monitor_for_cursor(
    monitors: &[MonitorBounds],
    cursor: CursorPosition,
) -> Option<MonitorBounds> {
    // monitor_from_point 在 mixed-DPI 场景下可能返回 None；这里先统一到逻辑坐标再命中。
    let desktop_scale_factor = desktop_scale_factor(monitors);
    let cursor_logical_x = cursor.x / desktop_scale_factor;
    let cursor_logical_y = cursor.y / desktop_scale_factor;

    monitors.iter().copied().find(|monitor| {
        cursor_logical_x >= monitor.logical_rect.x as f64
            && cursor_logical_x < (monitor.logical_rect.x + monitor.logical_rect.width) as f64
            && cursor_logical_y >= monitor.logical_rect.y as f64
            && cursor_logical_y < (monitor.logical_rect.y + monitor.logical_rect.height) as f64
    })
}

fn desktop_scale_factor(monitors: &[MonitorBounds]) -> f64 {
    monitors
        .iter()
        .copied()
        .find(|monitor| monitor.logical_rect.x == 0 && monitor.logical_rect.y == 0)
        .or_else(|| {
            monitors
                .iter()
                .copied()
                .min_by_key(|monitor| monitor.logical_rect.x.abs() + monitor.logical_rect.y.abs())
        })
        .map(MonitorBounds::scale_factor)
        .unwrap_or(1.0)
}

fn monitor_bounds(monitor: &tauri::window::Monitor) -> MonitorBounds {
    // Tauri 在 macOS mixed-DPI 下返回的是逻辑桌面位置 + 物理分辨率。
    MonitorBounds {
        logical_rect: Rect {
            x: monitor.position().x,
            y: monitor.position().y,
            width: (monitor.size().width as f64 / monitor.scale_factor()).round() as i32,
            height: (monitor.size().height as f64 / monitor.scale_factor()).round() as i32,
        },
        physical_size: WindowSize {
            width: monitor.size().width as i32,
            height: monitor.size().height as i32,
        },
        scale_millis: (monitor.scale_factor() * 1000.0).round() as i32,
    }
}

fn monitor_fingerprint(monitor: MonitorBounds) -> LauncherMonitorFingerprint {
    LauncherMonitorFingerprint {
        x: monitor.logical_rect.x,
        y: monitor.logical_rect.y,
        width: monitor.physical_size.width,
        height: monitor.physical_size.height,
        scale_millis: monitor.scale_millis,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LauncherShowPosition {
    position: Point,
    should_forget_saved_position: bool,
}

fn launcher_show_position(
    monitor: MonitorBounds,
    window: WindowSize,
    saved_position: Option<LauncherSavedPosition>,
) -> LauncherShowPosition {
    if let Some(saved_position) = saved_position
        && saved_position.monitor == monitor_fingerprint(monitor)
    {
        let position = Point {
            x: saved_position.position.x,
            y: saved_position.position.y,
        };
        if is_mostly_visible(window_rect(saved_position.position, window), monitor) {
            return LauncherShowPosition {
                position,
                should_forget_saved_position: false,
            };
        }
    }

    LauncherShowPosition {
        position: centered_window_position(monitor, window),
        should_forget_saved_position: true,
    }
}

fn centered_window_position(monitor: MonitorBounds, window: WindowSize) -> Point {
    let monitor_size = monitor.logical_size();
    let max_x = monitor.logical_rect.x + (monitor_size.width - window.width).max(0);
    let max_y = monitor.logical_rect.y + (monitor_size.height - window.height).max(0);
    let centered_x = monitor.logical_rect.x + (monitor_size.width - window.width) / 2;
    let centered_y = monitor.logical_rect.y + (monitor_size.height - window.height) / 2;

    Point {
        x: centered_x.clamp(monitor.logical_rect.x, max_x),
        y: centered_y.clamp(monitor.logical_rect.y, max_y),
    }
}

fn window_rect(position: LauncherWindowPosition, size: WindowSize) -> Rect {
    Rect {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
    }
}

fn monitor_rect(monitor: MonitorBounds) -> Rect {
    monitor.logical_rect
}

fn is_mostly_visible(window: Rect, monitor: MonitorBounds) -> bool {
    let window_area = rect_area(window);
    if window_area == 0 {
        return false;
    }

    intersection_area(window, monitor_rect(monitor)) * 2 >= window_area
}

fn rect_area(rect: Rect) -> i64 {
    rect.width.max(0) as i64 * rect.height.max(0) as i64
}

fn intersection_area(a: Rect, b: Rect) -> i64 {
    let left = a.x.max(b.x);
    let top = a.y.max(b.y);
    let right = (a.x + a.width).min(b.x + b.width);
    let bottom = (a.y + a.height).min(b.y + b.height);

    if right <= left || bottom <= top {
        return 0;
    }

    (right - left) as i64 * (bottom - top) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn monitor(x: i32, y: i32, width: i32, height: i32) -> MonitorBounds {
        scaled_monitor(x, y, width, height, 1000)
    }

    fn scaled_monitor(x: i32, y: i32, width: i32, height: i32, scale_millis: i32) -> MonitorBounds {
        let scale_factor = scale_millis as f64 / 1000.0;
        MonitorBounds {
            logical_rect: Rect {
                x,
                y,
                width: (width as f64 / scale_factor).round() as i32,
                height: (height as f64 / scale_factor).round() as i32,
            },
            physical_size: WindowSize { width, height },
            scale_millis,
        }
    }

    fn saved(monitor: MonitorBounds, x: i32, y: i32) -> LauncherSavedPosition {
        LauncherSavedPosition {
            monitor: monitor_fingerprint(monitor),
            position: LauncherWindowPosition { x, y },
        }
    }

    #[test]
    fn centers_on_single_monitor() {
        assert_eq!(
            centered_window_position(
                monitor(0, 0, 1920, 1080),
                WindowSize {
                    width: 820,
                    height: 560
                },
            ),
            Point { x: 550, y: 260 }
        );
    }

    #[test]
    fn centers_on_right_side_monitor() {
        assert_eq!(
            centered_window_position(
                monitor(1920, 0, 1920, 1080),
                WindowSize {
                    width: 820,
                    height: 560
                },
            ),
            Point { x: 2470, y: 260 }
        );
    }

    #[test]
    fn centers_on_low_dpi_secondary_in_logical_coordinates() {
        let target_monitor = monitor(1680, 0, 1920, 1080);

        assert_eq!(
            centered_window_position(
                target_monitor,
                WindowSize {
                    width: 820,
                    height: 560
                },
            ),
            Point { x: 2230, y: 260 }
        );
    }

    #[test]
    fn centers_on_high_dpi_primary_in_logical_coordinates() {
        let target_monitor = scaled_monitor(0, 0, 3360, 2100, 2000);

        assert_eq!(
            centered_window_position(
                target_monitor,
                WindowSize {
                    width: 820,
                    height: 560
                },
            ),
            Point { x: 430, y: 245 }
        );
    }

    #[test]
    fn selects_monitor_by_logical_cursor_bounds_for_mixed_dpi() {
        let primary = scaled_monitor(0, 0, 3360, 2100, 2000);
        let secondary = monitor(1680, 0, 1920, 1080);
        let monitors = [primary, secondary];

        assert_eq!(
            select_monitor_for_cursor(
                &monitors,
                CursorPosition {
                    x: 3102.2,
                    y: 1158.3,
                },
            ),
            Some(primary)
        );
        assert_eq!(
            select_monitor_for_cursor(
                &monitors,
                CursorPosition {
                    x: 3572.4,
                    y: 578.6,
                },
            ),
            Some(secondary)
        );
        assert_eq!(
            select_monitor_for_cursor(
                &monitors,
                CursorPosition {
                    x: 3300.0,
                    y: 1900.0,
                },
            ),
            Some(primary)
        );
    }

    #[test]
    fn centers_on_left_side_monitor_with_negative_x() {
        assert_eq!(
            centered_window_position(
                monitor(-1920, 0, 1920, 1080),
                WindowSize {
                    width: 820,
                    height: 560
                },
            ),
            Point { x: -1370, y: 260 }
        );
    }

    #[test]
    fn centers_on_upper_monitor_with_negative_y() {
        assert_eq!(
            centered_window_position(
                monitor(0, -1080, 1920, 1080),
                WindowSize {
                    width: 820,
                    height: 560
                },
            ),
            Point { x: 550, y: -820 }
        );
    }

    #[test]
    fn clamps_when_window_is_larger_than_monitor() {
        assert_eq!(
            centered_window_position(
                monitor(100, 200, 640, 480),
                WindowSize {
                    width: 820,
                    height: 560
                },
            ),
            Point { x: 100, y: 200 }
        );
    }

    #[test]
    fn restores_saved_position_on_same_monitor_when_mostly_visible() {
        let target_monitor = monitor(0, 0, 1920, 1080);

        assert_eq!(
            launcher_show_position(
                target_monitor,
                WindowSize {
                    width: 820,
                    height: 560
                },
                Some(saved(target_monitor, 80, 120)),
            ),
            LauncherShowPosition {
                position: Point { x: 80, y: 120 },
                should_forget_saved_position: false,
            }
        );
    }

    #[test]
    fn centers_and_forgets_saved_position_on_different_monitor() {
        let saved_monitor = monitor(0, 0, 1920, 1080);
        let target_monitor = monitor(1920, 0, 1920, 1080);

        assert_eq!(
            launcher_show_position(
                target_monitor,
                WindowSize {
                    width: 820,
                    height: 560
                },
                Some(saved(saved_monitor, 80, 120)),
            ),
            LauncherShowPosition {
                position: Point { x: 2470, y: 260 },
                should_forget_saved_position: true,
            }
        );
    }

    #[test]
    fn centers_and_forgets_saved_position_when_monitor_size_changes() {
        let saved_monitor = monitor(0, 0, 1920, 1080);
        let target_monitor = monitor(0, 0, 2560, 1440);

        assert_eq!(
            launcher_show_position(
                target_monitor,
                WindowSize {
                    width: 820,
                    height: 560
                },
                Some(saved(saved_monitor, 80, 120)),
            ),
            LauncherShowPosition {
                position: Point { x: 870, y: 440 },
                should_forget_saved_position: true,
            }
        );
    }

    #[test]
    fn centers_and_forgets_saved_position_when_mostly_invisible() {
        let target_monitor = monitor(0, 0, 1920, 1080);

        assert_eq!(
            launcher_show_position(
                target_monitor,
                WindowSize {
                    width: 820,
                    height: 560
                },
                Some(saved(target_monitor, 1600, 900)),
            ),
            LauncherShowPosition {
                position: Point { x: 550, y: 260 },
                should_forget_saved_position: true,
            }
        );
    }
}
