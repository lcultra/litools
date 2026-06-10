use tauri::{Manager, Window, WindowEvent};

use crate::{
    plugin_runtime,
    state::{AppState, LauncherWindowPosition},
    surface::{events, model::SurfaceLifecycle},
    windowing::{labels, native},
};

pub fn handle_window_event(window: &Window, event: &WindowEvent) {
    let Some(state) = window.try_state::<AppState>() else {
        return;
    };

    match window.label() {
        labels::MAIN_WINDOW_LABEL => handle_main_window_event(window, event, &state),
        label if labels::is_detached_panel_window_label(label) => {
            handle_detached_panel_event(window, event, &state)
        }
        label if labels::is_plugin_window_label(label) => {
            handle_plugin_runtime_event(window, event, &state)
        }
        _ => {}
    }
}

fn handle_main_window_event(window: &Window, event: &WindowEvent, state: &AppState) {
    match event {
        WindowEvent::CloseRequested { api, .. } if !state.is_quitting() => {
            if state.close_to_tray() {
                api.prevent_close();
                native::hide_window(window);
            } else {
                state.request_quit();
            }
        }
        WindowEvent::Focused(false) if !state.is_quitting() && state.hide_on_blur() => {
            native::hide_window(window);
        }
        WindowEvent::Moved(position) => {
            // 程序化布局窗口内（show/resize/set_position）的所有 Moved 忽略，
            // 只有用户拖拽触发的才会写入位置记忆。
            if state.is_programmatic_layout() {
                return;
            }
            super::positioning::save_launcher_position(
                window,
                state,
                LauncherWindowPosition {
                    x: position.x,
                    y: position.y,
                },
            );
        }
        _ => {}
    }
}

fn handle_detached_panel_event(window: &Window, event: &WindowEvent, state: &AppState) {
    match event {
        WindowEvent::CloseRequested { api, .. } if !state.is_quitting() => {
            api.prevent_close();
            native::hide_window(window);
            if let Some(metadata) =
                state.mark_surface_lifecycle(window.label(), SurfaceLifecycle::Hidden)
            {
                events::emit_metadata_changed(window.app_handle(), &metadata);
            }
        }
        WindowEvent::Focused(focused) => {
            if let Some(metadata) = state.mark_surface_focused(window.label(), *focused) {
                events::emit_metadata_changed(window.app_handle(), &metadata);
            }
            if *focused
                && let Some(metadata) =
                    state.mark_surface_lifecycle(window.label(), SurfaceLifecycle::Active)
            {
                events::emit_metadata_changed(window.app_handle(), &metadata);
            }
        }
        WindowEvent::Destroyed => {
            state.remove_surface(window.label());
        }
        _ => {}
    }
}

fn handle_plugin_runtime_event(window: &Window, event: &WindowEvent, state: &AppState) {
    match event {
        WindowEvent::Focused(true) => {
            if let Some(context) = state.plugin_runtime_for_window_label(window.label()) {
                let _ =
                    plugin_runtime::service::enter_runtime(window.app_handle(), state, &context.id);
            }
        }
        WindowEvent::Focused(false) => {
            if let Some(context) = state.plugin_runtime_for_window_label(window.label()) {
                let _ =
                    plugin_runtime::service::leave_runtime(window.app_handle(), state, &context.id);
            }
        }
        WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
            let _ = plugin_runtime::service::layout_runtime_window(
                window.app_handle(),
                state,
                window.label(),
            );
        }
        WindowEvent::CloseRequested { .. } | WindowEvent::Destroyed => {
            let _ = plugin_runtime::service::cleanup_runtime_window(
                window.app_handle(),
                state,
                window.label(),
            );
        }
        _ => {}
    }
}
