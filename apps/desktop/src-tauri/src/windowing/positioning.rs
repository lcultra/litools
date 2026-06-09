use tauri::{
    PhysicalPosition, Position, Window,
};

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

pub fn maybe_position_on_show(window: &Window, center_on_show: bool) {
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
                MonitorBounds { x: 0, y: 0, width: 1920, height: 1080 },
                WindowSize { width: 820, height: 560 },
            ),
            Point { x: 550, y: 260 }
        );
    }

    #[test]
    fn centers_on_right_side_monitor() {
        assert_eq!(
            centered_window_position(
                MonitorBounds { x: 1920, y: 0, width: 1920, height: 1080 },
                WindowSize { width: 820, height: 560 },
            ),
            Point { x: 2470, y: 260 }
        );
    }

    #[test]
    fn centers_on_left_side_monitor_with_negative_x() {
        assert_eq!(
            centered_window_position(
                MonitorBounds { x: -1920, y: 0, width: 1920, height: 1080 },
                WindowSize { width: 820, height: 560 },
            ),
            Point { x: -1370, y: 260 }
        );
    }

    #[test]
    fn centers_on_upper_monitor_with_negative_y() {
        assert_eq!(
            centered_window_position(
                MonitorBounds { x: 0, y: -1080, width: 1920, height: 1080 },
                WindowSize { width: 820, height: 560 },
            ),
            Point { x: 550, y: -820 }
        );
    }

    #[test]
    fn clamps_when_window_is_larger_than_monitor() {
        assert_eq!(
            centered_window_position(
                MonitorBounds { x: 100, y: 200, width: 640, height: 480 },
                WindowSize { width: 820, height: 560 },
            ),
            Point { x: 100, y: 200 }
        );
    }

    #[test]
    fn detects_monitor_containing_point() {
        let monitors = [
            MonitorBounds { x: 0, y: 0, width: 1920, height: 1080 },
            MonitorBounds { x: 1920, y: 0, width: 1920, height: 1080 },
        ];

        assert_eq!(
            select_monitor_for_point(&monitors, Point { x: 2200, y: 100 }),
            Some(monitors[1])
        );
    }
}
