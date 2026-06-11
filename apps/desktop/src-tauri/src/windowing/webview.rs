use std::path::PathBuf;

use tauri::{
    LogicalPosition, LogicalSize, Position, Size, Webview, WebviewUrl, Window,
    webview::WebviewBuilder,
};

use crate::{
    core::plugins::runtime::model::PluginRuntimeBounds, core::surface::model::SurfaceMetadata,
};

pub use litools_config::window::{CHROME_INSET, TITLEBAR_HEIGHT};

// === 原 native.rs webview 相关函数 ===

pub fn add_plugin_runtime_webview(
    window: &Window,
    webview_label: String,
    entry_url: &str,
    initialization_script: String,
) -> Result<(Webview, PluginRuntimeBounds), String> {
    let url = tauri::Url::parse(entry_url).map_err(|error| error.to_string())?;
    let webview_url = match url.scheme() {
        "http" | "https" => WebviewUrl::External(url),
        _ => WebviewUrl::CustomProtocol(url),
    };
    let bounds = plugin_runtime_content_bounds(window)?;
    let webview = window
        .add_child(
            WebviewBuilder::new(webview_label, webview_url)
                .initialization_script(initialization_script)
                .transparent(true),
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

pub fn hide_plugin_runtime_webview(webview: &Webview) -> Result<(), String> {
    webview.hide().map_err(|error| error.to_string())
}

pub fn show_plugin_runtime_webview(webview: &Webview) -> Result<(), String> {
    webview.show().map_err(|error| error.to_string())
}

pub fn plugin_runtime_content_bounds(window: &Window) -> Result<PluginRuntimeBounds, String> {
    let size = window_inner_logical_size(window)?;
    Ok(PluginRuntimeBounds {
        x: CHROME_INSET,
        y: TITLEBAR_HEIGHT + CHROME_INSET,
        width: (size.width - CHROME_INSET * 2.0).max(0.0),
        height: (size.height - TITLEBAR_HEIGHT - CHROME_INSET * 2.0).max(0.0),
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

// === 原 reparent.rs ===

pub fn reparent_surface_webview(webview: &Webview, target_window: &Window) -> Result<(), String> {
    reparent_webview_to_window(webview, target_window)?;
    webview
        .set_position(LogicalPosition::new(0, 0))
        .map_err(|error| error.to_string())?;
    webview
        .set_size(
            target_window
                .inner_size()
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
    webview
        .set_auto_resize(true)
        .map_err(|error| error.to_string())
}

pub fn reparent_webview_to_window(webview: &Webview, target_window: &Window) -> Result<(), String> {
    webview
        .reparent(target_window)
        .map_err(|error| error.to_string())
}
