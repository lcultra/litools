use litools_plugin::RuntimePolicy;
use litools_plugin::PluginCommandMode;
use tauri::Manager;

use crate::{
    core::plugins::runtime::{
        model::{
            PluginRuntimeContext, PluginRuntimeInfo, PluginRuntimeLifecycle,
        },
        permissions,
        preload,
        registry::PluginRuntimeRegistration,
    },
    protocol::plugin::resolve_entry_url,
    state::AppState,
    core::surface::{model::SurfaceBounds, service as surface_service},
    view::WindowHostKind,
    windowing::{labels, factory, webview},
};

/// 定义启动时的操作类型。
enum LaunchAction {
    /// 创建新运行时
    #[allow(dead_code)]
    Create,
    /// 确保已有停靠运行时可见
    EnsureVisible(PluginRuntimeContext),
    /// 聚焦已分离的运行时（仅将窗口提到前台，不 re-dock）
    FocusDetached(PluginRuntimeContext),
}

/// 根据 policy 和当前运行时状态决定启动行为。
fn resolve_launch_action(
    state: &AppState,
    plugin_id: &str,
    command_id: &str,
    policy: RuntimePolicy,
) -> Option<LaunchAction> {
    match policy {
        RuntimePolicy::Singleton => {
            let context = state
                .plugin_runtimes
                .lock()
                .ok()?
                .runtime_for_plugin_command(plugin_id, command_id)?;
            let host_kind = state
                .surfaces
                .lock()
                .ok()?
                .host_kind(&context.surface_id)?;
            match host_kind {
                WindowHostKind::Detached => Some(LaunchAction::FocusDetached(context)),
                WindowHostKind::Main => Some(LaunchAction::EnsureVisible(context)),
            }
        }
        RuntimePolicy::MultiInstance => None, // 总是创建新实例
    }
}

/// dispatch 表：
///
/// | policy        | 是否存在 | host_kind  | action         |
/// |---------------|----------|-----------|----------------|
/// | Singleton     | 否       | —         | Create         |
/// | Singleton     | 是       | Main      | EnsureVisible  |
/// | Singleton     | 是       | Detached  | FocusDetached  |
/// | MultiInstance | —        | —         | Create         |

#[derive(Clone, Debug)]
struct RuntimeLaunchDescriptor {
    plugin_id: String,
    command_id: String,
    plugin_name: String,
    title: String,
    entry_url: String,
    permissions: Vec<String>,
    trusted: bool,
    policy: RuntimePolicy,
}

pub fn dock_plugin_runtime(
    app: &tauri::AppHandle,
    state: &AppState,
    plugin_id: &str,
    command_id: &str,
) -> Result<PluginRuntimeContext, String> {
    let (_, _, _, policy) = find_enabled_plugin_command(state, plugin_id, command_id)?;

    if let Some(action) = resolve_launch_action(state, plugin_id, command_id, policy) {
        return match action {
            LaunchAction::EnsureVisible(context) => {
                ensure_docked_runtime_webview(app, state, context)
            }
            LaunchAction::FocusDetached(context) => {
                focus_runtime_host(app, state, &context)?;
                Ok(context)
            }
            LaunchAction::Create => unreachable!(), // Create 不在 resolve 中返回
        };
    }

    // 非单例 或 单例首次：创建新运行时
    let descriptor = runtime_launch_descriptor(state, plugin_id, command_id)?;
    let runtime_id = state
        .plugin_runtimes
        .lock()
        .map_err(|e| e.to_string())?
        .next_runtime_id();

    // 1. 先注册 surface
    let surface_view = crate::view::plugin_view_definition(plugin_id, command_id, &descriptor.title);
    let surface = state
        .surfaces
        .lock()
        .map_err(|e| e.to_string())?
        .register_surface(surface_view, labels::MAIN_WINDOW_LABEL.to_string(), WindowHostKind::Main);

    // 2. 注册 runtime，绑定 surface_id
    let context = state
        .plugin_runtimes
        .lock()
        .map_err(|e| e.to_string())?
        .register_runtime(
            PluginRuntimeRegistration {
                plugin_id: descriptor.plugin_id,
                command_id: descriptor.command_id,
                plugin_name: descriptor.plugin_name,
                title: descriptor.title,
                entry_url: descriptor.entry_url,
                surface_id: surface.id.clone(),
                permissions: descriptor.permissions,
                trusted: descriptor.trusted,
                policy: descriptor.policy,
            },
            runtime_id,
        )?;

    log::info!("打开插件视图: {plugin_id}:{command_id}");
    ensure_docked_runtime_webview(app, state, context)
}

pub fn hide_docked_plugin_runtime(
    app: &tauri::AppHandle,
    state: &AppState,
    plugin_id: &str,
    command_id: &str,
) -> Result<PluginRuntimeContext, String> {
    let context = state
        .plugin_runtimes
        .lock()
        .ok()
        .and_then(|r| r.runtime_for_plugin_command(plugin_id, command_id))
        .ok_or_else(|| format!("plugin runtime not found: {plugin_id}:{command_id}"))?;

    let host_kind = state
        .surfaces
        .lock()
        .map_err(|e| e.to_string())?
        .host_kind(&context.surface_id)
        .ok_or_else(|| format!("surface not found: {}", context.surface_id))?;
    if host_kind != WindowHostKind::Main {
        return Ok(context);
    }

    let _ = leave_runtime(app, state, &context.id);
    let webview_label = state
        .surfaces
        .lock()
        .map_err(|e| e.to_string())?
        .webview_label(&context.surface_id)
        .ok_or_else(|| format!("surface not found: {}", context.surface_id))?
        .to_string();
    if let Some(webview) = app.get_webview(&webview_label) {
        let _ = webview::hide_plugin_runtime_webview(&webview);
    }
    state
        .surfaces
        .lock()
        .map_err(|e| e.to_string())?
        .mark_bounds(&context.surface_id, None)
        .ok_or_else(|| format!("surface not found: {}", context.surface_id))?;
    Ok(context)
}

pub fn detach_plugin_runtime(
    app: &tauri::AppHandle,
    state: &AppState,
    plugin_id: &str,
    command_id: &str,
) -> Result<PluginRuntimeContext, String> {
    log::info!("分离插件视图: {plugin_id}:{command_id}");
    let context = state
        .plugin_runtimes
        .lock()
        .ok()
        .and_then(|r| r.runtime_for_plugin_command(plugin_id, command_id))
        .ok_or_else(|| format!("plugin runtime not found: {plugin_id}:{command_id}"))?;

    let host_kind = state
        .surfaces
        .lock()
        .map_err(|e| e.to_string())?
        .host_kind(&context.surface_id)
        .ok_or_else(|| format!("surface not found: {}", context.surface_id))?;
    if host_kind == WindowHostKind::Detached {
        focus_runtime_host(app, state, &context)?;
        return Ok(context);
    }

    let webview_label = state
        .surfaces
        .lock()
        .map_err(|e| e.to_string())?
        .webview_label(&context.surface_id)
        .ok_or_else(|| format!("surface not found: {}", context.surface_id))?
        .to_string();
    let webview = app.get_webview(&webview_label).ok_or_else(|| {
        format!("plugin runtime webview not found: {}", webview_label)
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
            // 唯一标签已消除碰撞，同步补充预热窗口
            spawn_pooled_detached(app, state);
            (window, pooled_label, true)
        } else {
            let label = labels::plugin_window_label(&context.id);
            let window =
                factory::create_plugin_runtime_detached_host(app, label.clone(), &context.title)?;
            (window, label, false)
        };

    let plugin_route = litools_core::plugin_route(&context.plugin_id, &context.command_id);

    if was_preloaded {
        // Navigate the already-loaded SolidJS app to the plugin route.
        for wv in detached_window.webviews() {
            let _ = wv.eval(&format!("window.location.hash = '#{}';", plugin_route));
        }
    } else {
        webview::add_surface_webview(
            &detached_window,
            &crate::core::surface::model::SurfaceMetadata {
                id: format!("detached_{}", context.id),
                webview_label: labels::surface_webview_label(&context.id),
                view_id: "plugin".to_string(),
                provider: crate::view::ViewProvider::Plugin {
                    plugin_id: context.plugin_id.clone(),
                },
                route: plugin_route.clone(),
                title: context.title.clone(),
                host_window_label: actual_label.clone(),
                host_kind: WindowHostKind::Detached,
                bounds: None,
                lifecycle: crate::core::surface::model::SurfaceLifecycle::Active,
                focused: true,
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            },
            &plugin_route,
        )?;
    }

    // Reparent plugin content webview into detached window.
    webview::reparent_webview_to_window(&webview, &detached_window)?;
    webview
        .set_auto_resize(false)
        .map_err(|error| error.to_string())?;
    let bounds = webview::set_plugin_runtime_content_bounds(&detached_window, &webview)?;
    webview::show_plugin_runtime_webview(&webview)?;

    // 更新 SurfaceRegistry：设置新的 host_window_label、host_kind、bounds
    state
        .surfaces
        .lock()
        .map_err(|e| e.to_string())?
        .move_to_host(&webview_label, actual_label.clone(), WindowHostKind::Detached)
        .ok_or_else(|| format!("surface not found by webview: {}", webview_label))?;
    state
        .surfaces
        .lock()
        .map_err(|e| e.to_string())?
        .mark_bounds(&context.surface_id, Some(SurfaceBounds {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: bounds.height,
        }))
        .ok_or_else(|| format!("surface not found: {}", context.surface_id))?;

    // 整个 detach 流程内的 Moved 由 begin_programmatic_layout 统一过滤
    state.begin_programmatic_layout();
    factory::show_panel_host(&detached_window);
    let _ = crate::core::surface::service::reset_launcher_surface(app, state, false);
    enter_runtime(app, state, &context.id)
}

/// Pre-create a hidden detached window so the first detach is instant.
pub fn warm_detached_pool(app: &tauri::AppHandle, state: &AppState) {
    spawn_pooled_detached(app, state);
}

fn spawn_pooled_detached(app: &tauri::AppHandle, state: &AppState) {
    // 每次生成唯一标签，避免复用已被实际分离窗口占用的标签
    let Ok(pooled_label) = state
        .surfaces
        .lock()
        .map(|mut r| r.next_detached_host_label())
    else {
        return;
    };

    let Ok(window) =
        factory::create_plugin_runtime_detached_host(app, pooled_label.clone(), "litools")
    else {
        return;
    };
    let _ = window.hide();

    // Pre-warm: load SolidJS app at /pooled (no route matches → empty page).
    let metadata = crate::core::surface::model::SurfaceMetadata {
        id: format!(
            "detached_pool_{}",
            pooled_label
                .strip_prefix(labels::DETACHED_PANEL_WINDOW_PREFIX)
                .unwrap_or("unknown")
        ),
        webview_label: labels::surface_webview_label(&format!(
            "pool_{}",
            pooled_label
                .strip_prefix(labels::DETACHED_PANEL_WINDOW_PREFIX)
                .unwrap_or("unknown")
        )),
        view_id: "core.launcher".to_string(),
        provider: crate::view::ViewProvider::Core,
        route: "/pooled".to_string(),
        title: "litools".to_string(),
        host_window_label: pooled_label.clone(),
        host_kind: WindowHostKind::Detached,
        bounds: None,
        lifecycle: crate::core::surface::model::SurfaceLifecycle::Active,
        focused: false,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    if webview::add_surface_webview(&window, &metadata, "/pooled").is_ok() {
        state.return_pooled_detached(pooled_label);
    }
}

pub fn close_plugin_runtime_by_plugin_command(
    app: &tauri::AppHandle,
    state: &AppState,
    plugin_id: &str,
    command_id: &str,
) -> Result<(), String> {
    log::info!("关闭插件视图: {plugin_id}:{command_id}");
    let context = state
        .plugin_runtimes
        .lock()
        .ok()
        .and_then(|r| r.runtime_for_plugin_command(plugin_id, command_id))
        .ok_or_else(|| format!("plugin runtime not found: {plugin_id}:{command_id}"))?;
    close_runtime(app, state, &context.id)
}

pub fn mark_runtime_ready(
    app: &tauri::AppHandle,
    state: &AppState,
    runtime_id: &str,
) -> Result<PluginRuntimeContext, String> {
    let before = state
        .plugin_runtimes
        .lock()
        .map_err(|e| e.to_string())?
        .runtime(runtime_id)
        .ok_or_else(|| format!("plugin runtime not found: {runtime_id}"))?;
    let context = state
        .plugin_runtimes
        .lock()
        .map_err(|e| e.to_string())?
        .mark_ready(runtime_id)
        .ok_or_else(|| format!("plugin runtime not found: {runtime_id}"))?;
    if context.entered && !before.entered {
        emit_lifecycle_event(app, state, &context, preload::LIFECYCLE_ENTER_EVENT)?;
    }
    Ok(context)
}

pub fn enter_runtime(
    app: &tauri::AppHandle,
    state: &AppState,
    runtime_id: &str,
) -> Result<PluginRuntimeContext, String> {
    let before = state
        .plugin_runtimes
        .lock()
        .map_err(|e| e.to_string())?
        .runtime(runtime_id)
        .ok_or_else(|| format!("plugin runtime not found: {runtime_id}"))?;
    let context = state
        .plugin_runtimes
        .lock()
        .map_err(|e| e.to_string())?
        .mark_focus_enter(runtime_id)
        .ok_or_else(|| format!("plugin runtime not found: {runtime_id}"))?;

    if context.entered && !before.entered {
        emit_lifecycle_event(app, state, &context, preload::LIFECYCLE_ENTER_EVENT)?;
    }
    Ok(context)
}

pub fn leave_runtime(
    app: &tauri::AppHandle,
    state: &AppState,
    runtime_id: &str,
) -> Result<PluginRuntimeContext, String> {
    let before = state
        .plugin_runtimes
        .lock()
        .map_err(|e| e.to_string())?
        .runtime(runtime_id)
        .ok_or_else(|| format!("plugin runtime not found: {runtime_id}"))?;
    if before.entered {
        emit_lifecycle_event(app, state, &before, preload::LIFECYCLE_LEAVE_EVENT)?;
    }
    state
        .plugin_runtimes
        .lock()
        .map_err(|e| e.to_string())?
        .mark_leave(runtime_id)
        .ok_or_else(|| format!("plugin runtime not found: {runtime_id}"))
}

pub fn mark_runtime_title(
    app: &tauri::AppHandle,
    state: &AppState,
    runtime_id: &str,
    title: String,
) -> Result<PluginRuntimeContext, String> {
    let context = state
        .plugin_runtimes
        .lock()
        .map_err(|e| e.to_string())?
        .mark_title(runtime_id, title.clone())
        .ok_or_else(|| format!("plugin runtime not found: {runtime_id}"))?;

    let host_kind = state
        .surfaces
        .lock()
        .map_err(|e| e.to_string())?
        .host_kind(&context.surface_id)
        .ok_or_else(|| format!("surface not found: {}", context.surface_id))?;
    if host_kind == WindowHostKind::Detached {
        let host_window_label = state
            .surfaces
            .lock()
            .map_err(|e| e.to_string())?
            .host_window_label(&context.surface_id)
            .ok_or_else(|| format!("surface not found: {}", context.surface_id))?
            .to_string();
        if let Some(window) = app.get_window(&host_window_label) {
            window
                .set_title(&title)
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(context)
}

pub fn close_runtime(
    app: &tauri::AppHandle,
    state: &AppState,
    runtime_id: &str,
) -> Result<(), String> {
    let context = state
        .plugin_runtimes
        .lock()
        .map_err(|e| e.to_string())?
        .runtime(runtime_id)
        .ok_or_else(|| format!("plugin runtime not found: {runtime_id}"))?;
    let _ = leave_runtime(app, state, runtime_id);

    let surface_id = context.surface_id.clone();
    let webview_label = {
        let surfaces = state.surfaces.lock().map_err(|e| e.to_string())?;
        surfaces.webview_label(&surface_id).ok_or_else(|| format!("surface not found: {surface_id}"))?.to_string()
    };
    let host_kind = state
        .surfaces
        .lock()
        .map_err(|e| e.to_string())?
        .host_kind(&surface_id)
        .ok_or_else(|| format!("surface not found: {surface_id}"))?;
    let host_window_label = state
        .surfaces
        .lock()
        .map_err(|e| e.to_string())?
        .host_window_label(&surface_id)
        .ok_or_else(|| format!("surface not found: {surface_id}"))?
        .to_string();

    if let Some(webview) = app.get_webview(&webview_label) {
        let _ = webview.close();
    }
    if host_kind == WindowHostKind::Detached {
        if let Some(window) = app.get_window(&host_window_label) {
            let _ = window.destroy();
        }
    }
    if host_kind == WindowHostKind::Main {
        let _ = surface_service::open_view_route(app, state, "/");
    }

    state
        .plugin_runtimes
        .lock()
        .map_err(|e| e.to_string())?
        .mark_lifecycle(runtime_id, PluginRuntimeLifecycle::Closed);
    state.plugin_runtimes.lock().map_err(|e| e.to_string())?.remove(runtime_id);
    state.surfaces.lock().map_err(|e| e.to_string())?.remove(&surface_id);
    Ok(())
}

pub fn cleanup_runtime_window(
    app: &tauri::AppHandle,
    state: &AppState,
    window_label: &str,
) -> Result<(), String> {
    let context = find_runtime_by_window_label(state, window_label);
    let Some(context) = context else {
        return Ok(());
    };

    let host_kind = state
        .surfaces
        .lock()
        .map_err(|e| e.to_string())?
        .host_kind(&context.surface_id)
        .ok_or_else(|| format!("surface not found: {}", context.surface_id))?;
    if host_kind != WindowHostKind::Detached {
        return Ok(());
    }
    let _ = leave_runtime(app, state, &context.id);
    state
        .plugin_runtimes
        .lock()
        .map_err(|e| e.to_string())?
        .mark_lifecycle(&context.id, PluginRuntimeLifecycle::Closed);
    state.plugin_runtimes.lock().map_err(|e| e.to_string())?.remove(&context.id);
    state.surfaces.lock().map_err(|e| e.to_string())?.remove(&context.surface_id);
    Ok(())
}

pub fn layout_runtime_window(
    app: &tauri::AppHandle,
    state: &AppState,
    window_label: &str,
) -> Result<Option<PluginRuntimeContext>, String> {
    let context = find_runtime_by_window_label(state, window_label);
    let Some(context) = context else {
        return Ok(None);
    };

    let host_kind = state
        .surfaces
        .lock()
        .map_err(|e| e.to_string())?
        .host_kind(&context.surface_id)
        .ok_or_else(|| format!("surface not found: {}", context.surface_id))?;
    if host_kind != WindowHostKind::Detached {
        return Ok(Some(context));
    }
    let Some(window) = app.get_window(window_label) else {
        return Ok(Some(context));
    };
    let webview_label = state
        .surfaces
        .lock()
        .map_err(|e| e.to_string())?
        .webview_label(&context.surface_id)
        .ok_or_else(|| format!("surface not found: {}", context.surface_id))?
        .to_string();
    let Some(webview) = app.get_webview(&webview_label) else {
        return Ok(Some(context));
    };
    let bounds = webview::set_plugin_runtime_content_bounds(&window, &webview)?;
    state
        .surfaces
        .lock()
        .map_err(|e| e.to_string())?
        .mark_bounds(&context.surface_id, Some(SurfaceBounds {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: bounds.height,
        }))
        .ok_or_else(|| format!("surface not found: {}", context.surface_id))?;
    Ok(Some(context))
}

pub fn runtime_info(state: &AppState, runtime_id: &str) -> Result<PluginRuntimeInfo, String> {
    let context = state
        .plugin_runtimes
        .lock()
        .map_err(|e| e.to_string())?
        .runtime(runtime_id)
        .ok_or_else(|| format!("plugin runtime not found: {runtime_id}"))?;
    Ok(build_runtime_info(state, &context))
}

fn ensure_docked_runtime_webview(
    app: &tauri::AppHandle,
    state: &AppState,
    context: PluginRuntimeContext,
) -> Result<PluginRuntimeContext, String> {
    let window = surface_service::ensure_main_launcher_surface(app, state)?;
    factory::show_main_panel_host(&window, state);

    let surface_id = context.surface_id.clone();
    let webview_label = state
        .surfaces
        .lock()
        .map_err(|e| e.to_string())?
        .webview_label(&surface_id)
        .ok_or_else(|| format!("surface not found: {surface_id}"))?
        .to_string();

    let bounds = if let Some(webview) = app.get_webview(&webview_label) {
        if webview.window().label() != labels::MAIN_WINDOW_LABEL {
            webview::reparent_webview_to_window(&webview, &window)?;
        }
        webview
            .set_auto_resize(false)
            .map_err(|error| error.to_string())?;
        let bounds = webview::set_plugin_runtime_content_bounds(&window, &webview)?;
        webview::show_plugin_runtime_webview(&webview)?;
        bounds
    } else {
        let (webview, bounds) = webview::add_plugin_runtime_webview(
            &window,
            webview_label.clone(),
            &context.entry_url,
            preload::initialization_script(),
        )?;
        webview::show_plugin_runtime_webview(&webview)?;
        bounds
    };

    // 更新 SurfaceRegistry 的 host_window_label 和 bounds
    state
        .surfaces
        .lock()
        .map_err(|e| e.to_string())?
        .move_to_host(&webview_label, labels::MAIN_WINDOW_LABEL.to_string(), WindowHostKind::Main)
        .ok_or_else(|| format!("surface not found by webview: {}", webview_label))?;
    state
        .surfaces
        .lock()
        .map_err(|e| e.to_string())?
        .mark_bounds(&surface_id, Some(SurfaceBounds {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: bounds.height,
        }))
        .ok_or_else(|| format!("surface not found: {surface_id}"))?;

    // 为新创建的 webview 动态授予 Tauri 插件能力
    let plugin_perms: Vec<String> = context
        .permissions
        .iter()
        .filter(|p| p.contains(':'))
        .cloned()
        .collect();
    if !plugin_perms.is_empty() {
        permissions::setup_plugin_capability(
            app,
            &webview_label,
            &plugin_perms,
            context.trusted,
        )
        .map_err(|e| log::warn!("插件权限配置失败: {e}"))
        .ok();
    }

    enter_runtime(app, state, &context.id)
}

fn focus_runtime_host(
    app: &tauri::AppHandle,
    state: &AppState,
    context: &PluginRuntimeContext,
) -> Result<(), String> {
    let host_window_label = state
        .surfaces
        .lock()
        .map_err(|e| e.to_string())?
        .host_window_label(&context.surface_id)
        .ok_or_else(|| format!("surface not found: {}", context.surface_id))?
        .to_string();
    let window = app.get_window(&host_window_label).ok_or_else(|| {
        format!(
            "plugin runtime host not found: {}",
            host_window_label
        )
    })?;
    factory::focus_window(&window)
}

fn emit_lifecycle_event(
    app: &tauri::AppHandle,
    state: &AppState,
    context: &PluginRuntimeContext,
    event_name: &str,
) -> Result<(), String> {
    let webview_label = state
        .surfaces
        .lock()
        .map_err(|e| e.to_string())?
        .webview_label(&context.surface_id)
        .ok_or_else(|| format!("surface not found: {}", context.surface_id))?
        .to_string();
    if let Some(webview) = app.get_webview(&webview_label) {
        webview
            .eval(preload::lifecycle_eval_script(event_name))
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

/// 通过 window_label 查找 PluginRuntimeContext。
fn find_runtime_by_window_label(state: &AppState, window_label: &str) -> Option<PluginRuntimeContext> {
    let surfaces = state.surfaces.lock().ok()?;
    let surface_id = surfaces.surface_id_by_window_label.get(window_label)?;
    state.plugin_runtimes.lock().ok()?.runtime_for_surface_id(surface_id)
}

/// 从 PluginRuntimeContext 构建 PluginRuntimeInfo。
pub fn build_runtime_info(state: &AppState, context: &PluginRuntimeContext) -> PluginRuntimeInfo {
    let host_kind = state
        .surfaces
        .lock()
        .ok()
        .and_then(|r| r.host_kind(&context.surface_id))
        .map(|k| match k {
            WindowHostKind::Main => "main".to_string(),
            WindowHostKind::Detached => "detached".to_string(),
        });
    PluginRuntimeInfo {
        runtime_id: context.id.clone(),
        plugin_id: context.plugin_id.clone(),
        command_id: context.command_id.clone(),
        plugin_name: context.plugin_name.clone(),
        title: context.title.clone(),
        surface_id: context.surface_id.clone(),
        host_kind,
        lifecycle: context.lifecycle.clone(),
        permissions: context.permissions.clone(),
    }
}

/// Shared helper: look up a plugin and one of its commands, returning the
/// plugin name, command title, declared permissions, and runtime policy.
pub fn find_enabled_plugin_command(
    state: &AppState,
    plugin_id: &str,
    command_id: &str,
) -> Result<(String, String, Vec<String>, RuntimePolicy), String> {
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
        plugin.manifest.runtime_policy(),
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
        entry_url: resolve_entry_url(&plugin.manifest.id, &plugin.manifest)?,
        permissions: plugin.manifest.permissions.clone(),
        trusted: plugin.trusted,
        policy: plugin.manifest.runtime_policy(),
    })
}
