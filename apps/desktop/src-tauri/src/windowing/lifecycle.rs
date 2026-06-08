use tauri::{Manager, Window, WindowEvent};

use crate::{
    state::AppState,
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
