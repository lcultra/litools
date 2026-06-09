use tauri::{Manager, Webview, Window};

use crate::{
    state::AppState,
    surface::{
        events,
        model::{SurfaceLifecycle, SurfaceMetadata},
    },
    view::{
        model::{ViewKind, WindowHostKind},
        registry,
    },
    windowing::{labels::MAIN_WINDOW_LABEL, native, reparent},
};

pub fn bootstrap_main_surface(app: &tauri::AppHandle, state: &AppState) -> Result<Window, String> {
    let window = native::create_main_host(app)?;
    if window.webviews().is_empty() {
        let metadata = state.register_main_launcher_surface()?;
        native::add_surface_webview(&window, &metadata, "/")?;
    }
    Ok(window)
}

pub fn ensure_main_launcher_surface(
    app: &tauri::AppHandle,
    state: &AppState,
) -> Result<Window, String> {
    let window = native::main_window(app).ok_or_else(|| "main window not found".to_string())?;
    let has_main_surface = window.webviews().iter().any(|webview| {
        state
            .surface_metadata_for_webview_label(webview.label())
            .is_some_and(|metadata| {
                metadata.host_window_label == MAIN_WINDOW_LABEL
                    && metadata.host_kind == WindowHostKind::Main
                    && metadata.lifecycle != SurfaceLifecycle::Destroyed
            })
    });

    if !has_main_surface {
        let metadata = state.register_main_launcher_surface()?;
        native::add_surface_webview(&window, &metadata, "/")?;
    }
    Ok(window)
}

pub fn open_view_route(
    app: &tauri::AppHandle,
    state: &AppState,
    route: &str,
    center_on_show: bool,
) -> Result<(), String> {
    let view = resolve_view_route(state, route)?;
    registry::validate_host_allowed(&view, &WindowHostKind::Main)?;
    let window = ensure_main_launcher_surface(app, state)?;
    native::show_host_for_view(&window, &view.kind, center_on_show);

    for webview in window.webviews() {
        let Some(metadata) = state.surface_metadata_for_webview_label(webview.label()) else {
            continue;
        };

        if metadata.host_window_label == window.label() {
            events::emit_navigate(&webview, &view.route);
            if view.kind == ViewKind::Launcher {
                events::emit_focus_search(&webview);
            }
        }
    }

    Ok(())
}

pub fn detach_current_surface(
    app: &tauri::AppHandle,
    state: &AppState,
    webview: &Webview,
    route: &str,
    center_on_show: bool,
) -> Result<SurfaceMetadata, String> {
    let view = resolve_view_route(state, route)?;
    registry::validate_detachable(&view)?;

    let current_metadata = state
        .surface_metadata_for_webview_label(webview.label())
        .ok_or_else(|| format!("surface metadata not found: {}", webview.label()))?;
    if current_metadata.lifecycle == SurfaceLifecycle::Destroyed {
        return Err(format!("surface has been destroyed: {}", webview.label()));
    }
    if current_metadata.host_kind == WindowHostKind::Detached
        || current_metadata.host_window_label != MAIN_WINDOW_LABEL
    {
        return Err(format!("surface is already detached: {}", webview.label()));
    }

    let source_window_label = current_metadata.host_window_label.clone();
    let detached_label = state.next_detached_host_label()?;
    let detached_window = native::create_detached_panel_host(app, detached_label)?;
    let detached_window_label = detached_window.label().to_string();

    reparent::reparent_surface_webview(webview, &detached_window)?;

    let metadata = state
        .move_surface_to_host(
            webview.label(),
            detached_window_label,
            WindowHostKind::Detached,
        )
        .ok_or_else(|| {
            format!(
                "surface metadata not found after reparent: {}",
                webview.label()
            )
        })?;
    let metadata = state
        .mark_surface_route(webview.label(), view)
        .unwrap_or(metadata);

    native::show_panel_host(&detached_window, center_on_show);
    events::emit_metadata_changed(app, &metadata);

    if source_window_label == MAIN_WINDOW_LABEL {
        match create_replacement_main_surface(app, state) {
            Ok((main_window, main_webview)) => {
                native::show_launcher_host(&main_window, center_on_show);
                events::emit_focus_search(&main_webview);
            }
            Err(error) => {
                let _ = ensure_main_launcher_surface(app, state);
                return Err(format!(
                    "detached surface moved, but main surface recovery failed: {error}"
                ));
            }
        }
    }

    Ok(metadata)
}

pub fn update_surface_route(
    app: &tauri::AppHandle,
    state: &AppState,
    webview_label: &str,
    route: &str,
) -> Result<SurfaceMetadata, String> {
    let view = resolve_view_route(state, route)?;
    let current_metadata = state
        .surface_metadata_for_webview_label(webview_label)
        .ok_or_else(|| format!("surface metadata not found: {webview_label}"))?;
    registry::validate_host_allowed(&view, &current_metadata.host_kind)?;
    if current_metadata.route == view.route {
        return Ok(current_metadata);
    }

    let metadata = state
        .mark_surface_route(webview_label, view)
        .ok_or_else(|| format!("surface metadata not found: {webview_label}"))?;
    events::emit_metadata_changed(app, &metadata);
    Ok(metadata)
}

fn resolve_view_route(
    state: &AppState,
    route: &str,
) -> Result<crate::view::model::ViewDefinition, String> {
    if let Some(view) = registry::view_for_route(route) {
        return Ok(view);
    }

    if registry::plugin_route_parts(route).is_some() {
        return crate::ipc::plugins::validate_plugin_view_route(state, route);
    }

    registry::validate_route(route)
}

pub fn list_surfaces(state: &AppState) -> Vec<SurfaceMetadata> {
    state.list_surfaces()
}

pub fn current_surface_metadata(state: &AppState, webview_label: &str) -> Option<SurfaceMetadata> {
    state.surface_metadata_for_webview_label(webview_label)
}

pub fn hide_surface(app: &tauri::AppHandle, state: &AppState, target: &str) -> Result<(), String> {
    let metadata = state
        .surface_metadata(target)
        .ok_or_else(|| format!("surface metadata not found: {target}"))?;
    let window = app
        .get_window(&metadata.host_window_label)
        .ok_or_else(|| format!("window not found: {}", metadata.host_window_label))?;
    native::hide_window(&window);
    if let Some(metadata) =
        state.mark_surface_lifecycle(&metadata.webview_label, SurfaceLifecycle::Hidden)
    {
        events::emit_metadata_changed(app, &metadata);
    }
    Ok(())
}

pub fn focus_surface_or_host(
    app: &tauri::AppHandle,
    state: &AppState,
    target: Option<&str>,
    fallback_window: Window,
) -> Result<(), String> {
    let target_window = match target {
        Some("main") => native::main_window(app),
        Some(target) => host_by_surface_id_or_label(app, state, target),
        None => Some(fallback_window),
    };

    if let Some(target_window) = target_window {
        native::focus_window(&target_window)?;
        if let Some(metadata) =
            state.mark_surface_lifecycle(target_window.label(), SurfaceLifecycle::Active)
        {
            events::emit_metadata_changed(app, &metadata);
        }
        if let Some(metadata) = state.mark_surface_focused(target_window.label(), true) {
            events::emit_metadata_changed(app, &metadata);
        }
    }

    Ok(())
}

pub fn destroy_surface(
    app: &tauri::AppHandle,
    state: &AppState,
    target: &str,
) -> Result<(), String> {
    let metadata = state
        .surface_metadata(target)
        .ok_or_else(|| format!("surface metadata not found: {target}"))?;

    if metadata.host_window_label == MAIN_WINDOW_LABEL {
        return Err("main surface cannot be destroyed through managed surface API".to_string());
    }

    if let Some(webview) = app.get_webview(&metadata.webview_label) {
        if let Some(metadata) =
            state.mark_surface_lifecycle(&metadata.webview_label, SurfaceLifecycle::Destroyed)
        {
            events::emit_metadata_changed(app, &metadata);
        }
        webview.close().map_err(|error| error.to_string())?;
    }
    if let Some(window) = app.get_window(&metadata.host_window_label) {
        window.destroy().map_err(|error| error.to_string())?;
    }
    state.remove_surface(&metadata.webview_label);
    Ok(())
}

pub fn toggle_main_launcher(app: &tauri::AppHandle, state: &AppState) -> Result<(), String> {
    let window = ensure_main_launcher_surface(app, state)?;
    if window.is_visible().unwrap_or(false) {
        native::hide_window(&window);
    } else {
        open_view_route(app, state, "/", state.center_on_show())?;
    }
    Ok(())
}

pub fn host_by_surface_id_or_label(
    app: &tauri::AppHandle,
    state: &AppState,
    target: &str,
) -> Option<Window> {
    if target == MAIN_WINDOW_LABEL {
        return native::main_window(app);
    }

    state
        .surface_metadata(target)
        .and_then(|metadata| app.get_window(&metadata.host_window_label))
        .or_else(|| app.get_window(target))
}

fn create_replacement_main_surface(
    app: &tauri::AppHandle,
    state: &AppState,
) -> Result<(Window, Webview), String> {
    let window = native::main_window(app).ok_or_else(|| "main window not found".to_string())?;
    let metadata = state.register_main_launcher_surface()?;
    let webview = native::add_surface_webview(&window, &metadata, "/")?;
    Ok((window, webview))
}
