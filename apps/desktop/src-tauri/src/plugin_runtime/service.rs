use litools_plugin::PluginCommandMode;
use tauri::Manager;

use crate::{
    plugin_runtime::{
        model::{
            PluginRuntimeContext, PluginRuntimeInfo, PluginRuntimeLifecycle, PluginRuntimePlacement,
        },
        preload,
        registry::PluginRuntimeRegistration,
    },
    protocol::plugin::plugin_entry_url,
    state::AppState,
    surface::service as surface_service,
    windowing::{labels, native, reparent},
};

#[derive(Clone, Debug)]
struct RuntimeLaunchDescriptor {
    plugin_id: String,
    command_id: String,
    plugin_name: String,
    title: String,
    entry_url: String,
    permissions: Vec<String>,
}

pub fn dock_plugin_runtime(
    app: &tauri::AppHandle,
    state: &AppState,
    plugin_id: &str,
    command_id: &str,
) -> Result<PluginRuntimeContext, String> {
    if let Some(context) = state.plugin_runtime_for_plugin_command(plugin_id, command_id) {
        if context.placement == PluginRuntimePlacement::Detached {
            focus_runtime_host(app, &context)?;
            return Ok(context);
        }
        return ensure_docked_runtime_webview(app, state, context);
    }

    let descriptor = runtime_launch_descriptor(state, plugin_id, command_id)?;
    let runtime_id = state.next_plugin_runtime_id()?;
    let webview_label = labels::plugin_webview_label(&runtime_id);
    let context = state.register_plugin_runtime(
        PluginRuntimeRegistration {
            plugin_id: descriptor.plugin_id,
            command_id: descriptor.command_id,
            plugin_name: descriptor.plugin_name,
            title: descriptor.title,
            entry_url: descriptor.entry_url,
            host_window_label: labels::MAIN_WINDOW_LABEL.to_string(),
            detached_window_label: None,
            placement: PluginRuntimePlacement::Docked,
            bounds: None,
            permissions: descriptor.permissions,
        },
        runtime_id,
        webview_label,
    )?;

    ensure_docked_runtime_webview(app, state, context)
}

pub fn hide_docked_plugin_runtime(
    app: &tauri::AppHandle,
    state: &AppState,
    plugin_id: &str,
    command_id: &str,
) -> Result<PluginRuntimeContext, String> {
    let context = state
        .plugin_runtime_for_plugin_command(plugin_id, command_id)
        .ok_or_else(|| format!("plugin runtime not found: {plugin_id}:{command_id}"))?;
    if context.placement != PluginRuntimePlacement::Docked {
        return Ok(context);
    }

    let _ = leave_runtime(app, state, &context.id);
    if let Some(webview) = app.get_webview(&context.webview_label) {
        let _ = native::hide_plugin_runtime_webview(&webview);
    }
    state
        .mark_plugin_runtime_bounds(&context.id, None)
        .ok_or_else(|| format!("plugin runtime not found: {}", context.id))
}

pub fn detach_plugin_runtime(
    app: &tauri::AppHandle,
    state: &AppState,
    plugin_id: &str,
    command_id: &str,
) -> Result<PluginRuntimeContext, String> {
    let context = state
        .plugin_runtime_for_plugin_command(plugin_id, command_id)
        .ok_or_else(|| format!("plugin runtime not found: {plugin_id}:{command_id}"))?;
    if context.placement == PluginRuntimePlacement::Detached {
        focus_runtime_host(app, &context)?;
        return Ok(context);
    }

    let webview = app.get_webview(&context.webview_label).ok_or_else(|| {
        format!(
            "plugin runtime webview not found: {}",
            context.webview_label
        )
    })?;
    // Use a preloaded window if available, otherwise create one.
    let (detached_window, actual_label, was_preloaded) =
        if let Some(pooled_label) = state.take_pooled_detached() {
            let window = app
                .get_window(&pooled_label)
                .ok_or_else(|| format!("pooled detached window not found: {pooled_label}"))?;
            window
                .set_title(&context.title)
                .map_err(|e| e.to_string())?;
            spawn_pooled_detached(app, state);
            (window, pooled_label, true)
        } else {
            let label = context
                .detached_window_label
                .clone()
                .unwrap_or_else(|| labels::plugin_window_label(&context.id));
            let window =
                native::create_plugin_runtime_detached_host(app, label.clone(), &context.title)?;
            (window, label, false)
        };

    let plugin_route = litools_core::plugin_route(&context.plugin_id, &context.command_id);

    if was_preloaded {
        // Navigate the already-loaded SolidJS app to the plugin route.
        // The window is still hidden so the user never sees the pooled route.
        for wv in detached_window.webviews() {
            let _ = wv.eval(&format!("window.location.hash = '#{}';", plugin_route));
        }
    } else {
        native::add_surface_webview(
            &detached_window,
            &crate::surface::model::SurfaceMetadata {
                id: format!("detached_{}", context.id),
                webview_label: labels::surface_webview_label(&context.id),
                view_id: "plugin".to_string(),
                provider: crate::view::model::ViewProvider::Plugin {
                    plugin_id: context.plugin_id.clone(),
                },
                route: plugin_route.clone(),
                title: context.title.clone(),
                host_window_label: actual_label.clone(),
                host_kind: crate::view::model::WindowHostKind::Detached,
                lifecycle: crate::surface::model::SurfaceLifecycle::Active,
                focused: true,
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            },
            &plugin_route,
        )?;
    }

    // Reparent plugin content webview into detached window.
    reparent::reparent_webview_to_window(&webview, &detached_window)?;
    webview
        .set_auto_resize(false)
        .map_err(|error| error.to_string())?;
    let bounds = native::set_plugin_runtime_content_bounds(&detached_window, &webview)?;
    native::show_plugin_runtime_webview(&webview)?;

    state.mark_plugin_runtime_detached_window(&context.id, Some(actual_label.clone()));
    let context = state
        .move_plugin_runtime_to_host(
            &context.id,
            actual_label.clone(),
            PluginRuntimePlacement::Detached,
            Some(bounds),
        )
        .ok_or_else(|| format!("plugin runtime not found: {}", context.id))?;
    native::show_panel_host(&detached_window);
    let _ = crate::surface::service::open_view_route(app, state, "/");
    enter_runtime(app, state, &context.id)
}

/// Pre-create a hidden detached window so the first detach is instant.
pub fn warm_detached_pool(app: &tauri::AppHandle, state: &AppState) {
    spawn_pooled_detached(app, state);
}

fn spawn_pooled_detached(app: &tauri::AppHandle, state: &AppState) {
    use labels::DETACHED_PANEL_WINDOW_PREFIX;

    let pooled_label = format!("{DETACHED_PANEL_WINDOW_PREFIX}pool");
    if app.get_window(&pooled_label).is_some() {
        return; // already exists
    }

    let Ok(window) =
        native::create_plugin_runtime_detached_host(app, pooled_label.clone(), "litools")
    else {
        return;
    };
    let _ = window.hide();

    // Pre-warm: load SolidJS app at /pooled (no route matches → empty page).
    // SolidJS boots but renders nothing visible, so when the window is later
    // shown after navigating to the plugin route, the user never sees the launcher.
    let metadata = crate::surface::model::SurfaceMetadata {
        id: "detached_pool".to_string(),
        webview_label: labels::surface_webview_label("pool"),
        view_id: "core.launcher".to_string(),
        provider: crate::view::model::ViewProvider::Core,
        route: "/pooled".to_string(),
        title: "litools".to_string(),
        host_window_label: pooled_label.clone(),
        host_kind: crate::view::model::WindowHostKind::Detached,
        lifecycle: crate::surface::model::SurfaceLifecycle::Active,
        focused: false,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    if native::add_surface_webview(&window, &metadata, "/pooled").is_ok() {
        state.return_pooled_detached(pooled_label);
    }
}

pub fn close_plugin_runtime_by_plugin_command(
    app: &tauri::AppHandle,
    state: &AppState,
    plugin_id: &str,
    command_id: &str,
) -> Result<(), String> {
    let context = state
        .plugin_runtime_for_plugin_command(plugin_id, command_id)
        .ok_or_else(|| format!("plugin runtime not found: {plugin_id}:{command_id}"))?;
    close_runtime(app, state, &context.id)
}

pub fn mark_runtime_ready(
    app: &tauri::AppHandle,
    state: &AppState,
    runtime_id: &str,
) -> Result<PluginRuntimeContext, String> {
    let before = state
        .plugin_runtime(runtime_id)
        .ok_or_else(|| format!("plugin runtime not found: {runtime_id}"))?;
    let context = state
        .mark_plugin_runtime_ready(runtime_id)
        .ok_or_else(|| format!("plugin runtime not found: {runtime_id}"))?;
    if context.entered && !before.entered {
        emit_lifecycle_event(app, &context.webview_label, preload::LIFECYCLE_ENTER_EVENT)?;
    }
    Ok(context)
}

pub fn enter_runtime(
    app: &tauri::AppHandle,
    state: &AppState,
    runtime_id: &str,
) -> Result<PluginRuntimeContext, String> {
    let before = state
        .plugin_runtime(runtime_id)
        .ok_or_else(|| format!("plugin runtime not found: {runtime_id}"))?;
    let context = state
        .mark_plugin_runtime_focus_enter(runtime_id)
        .ok_or_else(|| format!("plugin runtime not found: {runtime_id}"))?;

    if context.entered && !before.entered {
        emit_lifecycle_event(app, &context.webview_label, preload::LIFECYCLE_ENTER_EVENT)?;
    }
    Ok(context)
}

pub fn leave_runtime(
    app: &tauri::AppHandle,
    state: &AppState,
    runtime_id: &str,
) -> Result<PluginRuntimeContext, String> {
    let before = state
        .plugin_runtime(runtime_id)
        .ok_or_else(|| format!("plugin runtime not found: {runtime_id}"))?;
    if before.entered {
        emit_lifecycle_event(app, &before.webview_label, preload::LIFECYCLE_LEAVE_EVENT)?;
    }
    state
        .mark_plugin_runtime_leave(runtime_id)
        .ok_or_else(|| format!("plugin runtime not found: {runtime_id}"))
}

pub fn mark_runtime_title(
    app: &tauri::AppHandle,
    state: &AppState,
    runtime_id: &str,
    title: String,
) -> Result<PluginRuntimeContext, String> {
    let context = state
        .mark_plugin_runtime_title(runtime_id, title.clone())
        .ok_or_else(|| format!("plugin runtime not found: {runtime_id}"))?;
    if context.placement == PluginRuntimePlacement::Detached
        && let Some(window) = app.get_window(&context.host_window_label)
    {
        window
            .set_title(&title)
            .map_err(|error| error.to_string())?;
    }
    Ok(context)
}

pub fn close_runtime(
    app: &tauri::AppHandle,
    state: &AppState,
    runtime_id: &str,
) -> Result<(), String> {
    let context = state
        .plugin_runtime(runtime_id)
        .ok_or_else(|| format!("plugin runtime not found: {runtime_id}"))?;
    let _ = leave_runtime(app, state, runtime_id);

    if let Some(webview) = app.get_webview(&context.webview_label) {
        let _ = webview.close();
    }
    if context.placement == PluginRuntimePlacement::Detached
        && let Some(window) = app.get_window(&context.host_window_label)
    {
        let _ = window.destroy();
    }
    if context.placement == PluginRuntimePlacement::Docked {
        let _ = surface_service::open_view_route(app, state, "/");
    }

    state.mark_plugin_runtime_lifecycle(runtime_id, PluginRuntimeLifecycle::Closed);
    state.remove_plugin_runtime(runtime_id);
    Ok(())
}

pub fn cleanup_runtime_window(
    app: &tauri::AppHandle,
    state: &AppState,
    window_label: &str,
) -> Result<(), String> {
    let Some(context) = state.plugin_runtime_for_window_label(window_label) else {
        return Ok(());
    };
    if context.placement != PluginRuntimePlacement::Detached {
        return Ok(());
    }
    let _ = leave_runtime(app, state, &context.id);
    state.mark_plugin_runtime_lifecycle(&context.id, PluginRuntimeLifecycle::Closed);
    state.remove_plugin_runtime(&context.id);
    Ok(())
}

pub fn layout_runtime_window(
    app: &tauri::AppHandle,
    state: &AppState,
    window_label: &str,
) -> Result<Option<PluginRuntimeContext>, String> {
    let Some(context) = state.plugin_runtime_for_window_label(window_label) else {
        return Ok(None);
    };
    if context.placement != PluginRuntimePlacement::Detached {
        return Ok(Some(context));
    }
    let Some(window) = app.get_window(window_label) else {
        return Ok(Some(context));
    };
    let Some(webview) = app.get_webview(&context.webview_label) else {
        return Ok(Some(context));
    };
    let bounds = native::set_plugin_runtime_content_bounds(&window, &webview)?;
    Ok(state.mark_plugin_runtime_bounds(&context.id, Some(bounds)))
}

pub fn runtime_info(state: &AppState, runtime_id: &str) -> Result<PluginRuntimeInfo, String> {
    state
        .plugin_runtime(runtime_id)
        .map(|context| PluginRuntimeInfo::from(&context))
        .ok_or_else(|| format!("plugin runtime not found: {runtime_id}"))
}

fn ensure_docked_runtime_webview(
    app: &tauri::AppHandle,
    state: &AppState,
    context: PluginRuntimeContext,
) -> Result<PluginRuntimeContext, String> {
    let window = surface_service::ensure_main_launcher_surface(app, state)?;
    native::show_main_panel_host(&window, state);

    let bounds = if let Some(webview) = app.get_webview(&context.webview_label) {
        if webview.window().label() != labels::MAIN_WINDOW_LABEL {
            reparent::reparent_webview_to_window(&webview, &window)?;
        }
        webview
            .set_auto_resize(false)
            .map_err(|error| error.to_string())?;
        let bounds = native::set_plugin_runtime_content_bounds(&window, &webview)?;
        native::show_plugin_runtime_webview(&webview)?;
        bounds
    } else {
        let (webview, bounds) = native::add_plugin_runtime_webview(
            &window,
            context.webview_label.clone(),
            &context.entry_url,
            preload::initialization_script(),
        )?;
        native::show_plugin_runtime_webview(&webview)?;
        bounds
    };

    let context = state
        .move_plugin_runtime_to_host(
            &context.id,
            labels::MAIN_WINDOW_LABEL.to_string(),
            PluginRuntimePlacement::Docked,
            Some(bounds),
        )
        .ok_or_else(|| format!("plugin runtime not found: {}", context.id))?;
    enter_runtime(app, state, &context.id)
}

fn focus_runtime_host(
    app: &tauri::AppHandle,
    context: &PluginRuntimeContext,
) -> Result<(), String> {
    let window = app.get_window(&context.host_window_label).ok_or_else(|| {
        format!(
            "plugin runtime host not found: {}",
            context.host_window_label
        )
    })?;
    native::focus_window(&window)
}

fn emit_lifecycle_event(
    app: &tauri::AppHandle,
    webview_label: &str,
    event_name: &str,
) -> Result<(), String> {
    if let Some(webview) = app.get_webview(webview_label) {
        webview
            .eval(preload::lifecycle_eval_script(event_name))
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

/// Shared helper: look up a plugin and one of its commands, returning the
/// plugin name, command title, and declared permissions.
pub fn find_enabled_plugin_command(
    state: &AppState,
    plugin_id: &str,
    command_id: &str,
) -> Result<(String, String, Vec<String>), String> {
    let app = state.app().lock().map_err(|error| error.to_string())?;
    let plugin = app
        .context()
        .plugins
        .find_plugin(plugin_id)
        .ok_or_else(|| format!("plugin not found: {plugin_id}"))?;
    if !plugin.enabled {
        return Err(format!("plugin is disabled: {plugin_id}"));
    }
    let command = plugin
        .manifest
        .commands
        .iter()
        .find(|c| c.id == command_id)
        .ok_or_else(|| format!("plugin command not found: {plugin_id}:{command_id}"))?;

    Ok((
        plugin.manifest.name.clone(),
        command.title.clone(),
        plugin.manifest.permissions.clone(),
    ))
}

fn runtime_launch_descriptor(
    state: &AppState,
    plugin_id: &str,
    command_id: &str,
) -> Result<RuntimeLaunchDescriptor, String> {
    let app = state.app().lock().map_err(|error| error.to_string())?;
    let plugin = app
        .context()
        .plugins
        .find_plugin(plugin_id)
        .ok_or_else(|| format!("plugin not found: {plugin_id}"))?;
    if !plugin.enabled {
        return Err(format!("plugin is disabled: {plugin_id}"));
    }
    let command = plugin
        .manifest
        .commands
        .iter()
        .find(|command| command.id == command_id)
        .ok_or_else(|| format!("plugin command not found: {plugin_id}:{command_id}"))?;
    if command.mode != PluginCommandMode::View {
        return Err(format!(
            "plugin command is not a view command: {plugin_id}:{command_id}"
        ));
    }

    Ok(RuntimeLaunchDescriptor {
        plugin_id: plugin.manifest.id.clone(),
        command_id: command.id.clone(),
        plugin_name: plugin.manifest.name.clone(),
        title: command.title.clone(),
        entry_url: plugin_entry_url(&plugin.manifest.id, &plugin.manifest.entry)?,
        permissions: plugin.manifest.permissions.clone(),
    })
}
