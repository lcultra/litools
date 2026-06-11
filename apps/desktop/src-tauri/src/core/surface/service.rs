use tauri::{Manager, Webview, Window};

use crate::{
    state::AppState,
    core::surface::{
        events,
        model::{SurfaceLifecycle, SurfaceMetadata},
    },
    view::{self, ViewKind, WindowHostKind},
    windowing::{labels::MAIN_WINDOW_LABEL, factory, webview},
};

pub fn bootstrap_main_surface(app: &tauri::AppHandle, state: &AppState) -> Result<Window, String> {
    let window = factory::create_main_host(app)?;
    if window.webviews().is_empty() {
        let metadata = state
            .surfaces
            .lock()
            .map_err(|e| e.to_string())?
            .register_surface(
                crate::view::validate_route("/")?,
                MAIN_WINDOW_LABEL.to_string(),
                WindowHostKind::Main,
            );
        webview::add_surface_webview(&window, &metadata, "/")?;
    }
    Ok(window)
}

pub fn ensure_main_launcher_surface(
    app: &tauri::AppHandle,
    state: &AppState,
) -> Result<Window, String> {
    let window = factory::main_window(app).ok_or_else(|| "main window not found".to_string())?;
    let has_main_surface = window.webviews().iter().any(|webview| {
        state
            .surfaces
            .lock()
            .ok()
            .and_then(|r| r.metadata_for_webview_label(webview.label()))
            .is_some_and(|metadata| {
                metadata.host_window_label == MAIN_WINDOW_LABEL
                    && metadata.host_kind == WindowHostKind::Main
                    && metadata.lifecycle != SurfaceLifecycle::Destroyed
            })
    });

    if !has_main_surface {
        let metadata = state
            .surfaces
            .lock()
            .map_err(|e| e.to_string())?
            .register_surface(
                crate::view::validate_route("/")?,
                MAIN_WINDOW_LABEL.to_string(),
                WindowHostKind::Main,
            );
        webview::add_surface_webview(&window, &metadata, "/")?;
    }
    Ok(window)
}

pub fn open_view_route(
    app: &tauri::AppHandle,
    state: &AppState,
    route: &str,
) -> Result<(), String> {
    let view = resolve_view_route(state, route)?;
    view::validate_host_allowed(&view, &WindowHostKind::Main)?;
    let window = ensure_main_launcher_surface(app, state)?;
    factory::show_host_for_view(&window, state, &view.kind);

    if view.kind == ViewKind::Launcher {
        factory::emit_focus_to_owned_launcher_surfaces(&window);
    }

    Ok(())
}

pub fn detach_current_surface(
    app: &tauri::AppHandle,
    state: &AppState,
    webview: &Webview,
    route: &str,
) -> Result<SurfaceMetadata, String> {
    let view = resolve_view_route(state, route)?;
    view::validate_detachable(&view)?;

    let current_metadata = state
        .surfaces
        .lock()
        .ok()
        .and_then(|r| r.metadata_for_webview_label(webview.label()))
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
    let detached_label = state
        .surfaces
        .lock()
        .map_err(|e| e.to_string())?
        .next_detached_host_label();
    let detached_window = factory::create_detached_panel_host(app, detached_label)?;
    let detached_window_label = detached_window.label().to_string();

    webview::reparent_surface_webview(webview, &detached_window)?;

    let metadata = state
        .surfaces
        .lock()
        .map_err(|e| e.to_string())?
        .move_to_host(
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
        .surfaces
        .lock()
        .map_err(|e| e.to_string())?
        .mark_route(webview.label(), view)
        .unwrap_or(metadata);

    factory::show_panel_host(&detached_window);
    events::emit_metadata_changed(app, &metadata);

    if source_window_label == MAIN_WINDOW_LABEL {
        match create_replacement_main_surface(app, state) {
            Ok((main_window, main_webview)) => {
                factory::show_launcher_host(&main_window, state);
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
        .surfaces
        .lock()
        .ok()
        .and_then(|r| r.metadata_for_webview_label(webview_label))
        .ok_or_else(|| format!("surface metadata not found: {webview_label}"))?;
    view::validate_host_allowed(&view, &current_metadata.host_kind)?;
    if current_metadata.route == view.route {
        return Ok(current_metadata);
    }

    let metadata = state
        .surfaces
        .lock()
        .map_err(|e| e.to_string())?
        .mark_route(webview_label, view)
        .ok_or_else(|| format!("surface metadata not found: {webview_label}"))?;
    events::emit_metadata_changed(app, &metadata);
    Ok(metadata)
}

fn resolve_view_route(
    state: &AppState,
    route: &str,
) -> Result<crate::view::ViewDefinition, String> {
    if let Some(view) = view::view_for_route(route) {
        return Ok(view);
    }

    if let Some((plugin_id, command_id)) = view::plugin_route_parts(route) {
        let (_, title, _, _) =
            crate::core::plugins::runtime::service::find_enabled_plugin_command(
                state, plugin_id, command_id,
            )
            .map_err(|_| format!("unknown plugin route: {route}"))?;
        return Ok(crate::view::plugin_view_definition(plugin_id, command_id, title));
    }

    view::validate_route(route)
}

pub fn list_surfaces(state: &AppState) -> Vec<SurfaceMetadata> {
    state
        .surfaces
        .lock()
        .map(|registry| registry.list())
        .unwrap_or_default()
}

pub fn current_surface_metadata(
    state: &AppState,
    webview_label: &str,
) -> Option<SurfaceMetadata> {
    state
        .surfaces
        .lock()
        .ok()?
        .metadata_for_webview_label(webview_label)
}

pub fn hide_surface(app: &tauri::AppHandle, state: &AppState, target: &str) -> Result<(), String> {
    let metadata = state
        .surfaces
        .lock()
        .ok()
        .and_then(|r| r.metadata(target))
        .ok_or_else(|| format!("surface metadata not found: {target}"))?;
    let window = app
        .get_window(&metadata.host_window_label)
        .ok_or_else(|| format!("window not found: {}", metadata.host_window_label))?;
    factory::hide_window(&window);
    if let Some(metadata) = state
        .surfaces
        .lock()
        .map_err(|e| e.to_string())?
        .mark_lifecycle(&metadata.webview_label, SurfaceLifecycle::Hidden)
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
        Some("main") => factory::main_window(app),
        Some(target) => host_by_surface_id_or_label(app, state, target),
        None => Some(fallback_window),
    };

    if let Some(target_window) = target_window {
        factory::focus_window(&target_window)?;
        if let Some(metadata) = state
            .surfaces
            .lock()
            .map_err(|e| e.to_string())?
            .mark_lifecycle(target_window.label(), SurfaceLifecycle::Active)
        {
            events::emit_metadata_changed(app, &metadata);
        }
        if let Some(metadata) = state
            .surfaces
            .lock()
            .map_err(|e| e.to_string())?
            .mark_focused(target_window.label(), true)
        {
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
        .surfaces
        .lock()
        .ok()
        .and_then(|r| r.metadata(target))
        .ok_or_else(|| format!("surface metadata not found: {target}"))?;

    if metadata.host_window_label == MAIN_WINDOW_LABEL {
        return Err("main surface cannot be destroyed through managed surface API".to_string());
    }

    if let Some(webview) = app.get_webview(&metadata.webview_label) {
        if let Some(metadata) = state
            .surfaces
            .lock()
            .map_err(|e| e.to_string())?
            .mark_lifecycle(&metadata.webview_label, SurfaceLifecycle::Destroyed)
        {
            events::emit_metadata_changed(app, &metadata);
        }
        webview.close().map_err(|error| error.to_string())?;
    }
    if let Some(window) = app.get_window(&metadata.host_window_label) {
        window.destroy().map_err(|error| error.to_string())?;
    }
    state.surfaces.lock().map_err(|e| e.to_string())?.remove(&metadata.webview_label);
    Ok(())
}

pub fn toggle_main_launcher(app: &tauri::AppHandle, state: &AppState) -> Result<(), String> {
    let window = ensure_main_launcher_surface(app, state)?;
    if window.is_visible().unwrap_or(false) {
        factory::hide_window(&window);
    } else {
        // Just show the window — don't force navigate to /.
        // The surface retains whatever route the user was on (launcher or plugin view).
        let current_route = window
            .webviews()
            .iter()
            .find_map(|webview| {
                state
                    .surfaces
                    .lock()
                    .ok()
                    .and_then(|r| r.metadata_for_webview_label(webview.label()))
            })
            .map(|metadata| metadata.route)
            .unwrap_or_else(|| "/".to_string());
        let view = resolve_view_route(state, &current_route)?;
        factory::show_host_for_view(&window, state, &view.kind);

        // 仅在从隐藏→显示 且 当前为启动器页面时聚焦搜索框，避免抢插件焦点
        if view.kind == ViewKind::Launcher {
            factory::emit_focus_to_owned_launcher_surfaces(&window);
        }
    }
    Ok(())
}

/// 重置启动器 surface 到首页，可选择显示或隐藏主窗口。
///
/// 通过 `webview.eval` 直接设置 hash 来驱动 HashRouter 导航，
/// 同时同步 metadata route 保持前后端路由一致。
pub fn reset_launcher_surface(
    app: &tauri::AppHandle,
    state: &AppState,
    show: bool,
) -> Result<(), String> {
    let window = ensure_main_launcher_surface(app, state)?;
    for webview in window.webviews() {
        if let Some(metadata) = state
            .surfaces
            .lock()
            .ok()
            .and_then(|r| r.metadata_for_webview_label(webview.label()))
        {
            if metadata.host_kind == WindowHostKind::Main {
                // 同步 metadata route，让 toggle_main_launcher 能读到正确的当前路由
                let _ = state
                    .surfaces
                    .lock()
                    .map_err(|e| e.to_string())?
                    .mark_route(webview.label(), view::validate_route("/")?);
                // 实际驱动前端 HashRouter 导航
                let _ = webview.eval("window.location.hash = '#/'");
                break;
            }
        }
    }
    if show {
        let view = view::validate_route("/")?;
        factory::show_host_for_view(&window, state, &view.kind);
    } else {
        factory::hide_window(&window);
    }
    Ok(())
}

pub fn host_by_surface_id_or_label(
    app: &tauri::AppHandle,
    state: &AppState,
    target: &str,
) -> Option<Window> {
    if target == MAIN_WINDOW_LABEL {
        return factory::main_window(app);
    }

    state
        .surfaces
        .lock()
        .ok()
        .and_then(|r| r.metadata(target))
        .and_then(|metadata| app.get_window(&metadata.host_window_label))
        .or_else(|| app.get_window(target))
}

fn create_replacement_main_surface(
    app: &tauri::AppHandle,
    state: &AppState,
) -> Result<(Window, Webview), String> {
    let window = factory::main_window(app).ok_or_else(|| "main window not found".to_string())?;
    let metadata = state
        .surfaces
        .lock()
        .map_err(|e| e.to_string())?
        .register_surface(
            crate::view::validate_route("/")?,
            MAIN_WINDOW_LABEL.to_string(),
            WindowHostKind::Main,
        );
    let webview = webview::add_surface_webview(&window, &metadata, "/")?;
    Ok((window, webview))
}
