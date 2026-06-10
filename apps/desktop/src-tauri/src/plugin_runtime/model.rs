use litools_plugin::RuntimePolicy;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PluginRuntimeLifecycle {
    Created,
    Ready,
    Active,
    Closed,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PluginRuntimePlacement {
    Docked,
    Detached,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginRuntimeBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginRuntimeContext {
    pub id: String,
    pub plugin_id: String,
    pub command_id: String,
    pub plugin_name: String,
    pub title: String,
    pub entry_url: String,
    pub host_window_label: String,
    pub detached_window_label: Option<String>,
    pub webview_label: String,
    pub placement: PluginRuntimePlacement,
    pub bounds: Option<PluginRuntimeBounds>,
    pub permissions: Vec<String>,
    pub policy: RuntimePolicy,
    pub lifecycle: PluginRuntimeLifecycle,
    pub pending_enter: bool,
    pub entered: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginRuntimeInfo {
    pub runtime_id: String,
    pub plugin_id: String,
    pub command_id: String,
    pub plugin_name: String,
    pub title: String,
    pub host_window_label: String,
    pub detached_window_label: Option<String>,
    pub webview_label: String,
    pub placement: PluginRuntimePlacement,
    pub bounds: Option<PluginRuntimeBounds>,
    pub lifecycle: PluginRuntimeLifecycle,
    pub permissions: Vec<String>,
}

impl From<&PluginRuntimeContext> for PluginRuntimeInfo {
    fn from(context: &PluginRuntimeContext) -> Self {
        Self {
            runtime_id: context.id.clone(),
            plugin_id: context.plugin_id.clone(),
            command_id: context.command_id.clone(),
            plugin_name: context.plugin_name.clone(),
            title: context.title.clone(),
            host_window_label: context.host_window_label.clone(),
            detached_window_label: context.detached_window_label.clone(),
            webview_label: context.webview_label.clone(),
            placement: context.placement.clone(),
            bounds: context.bounds,
            lifecycle: context.lifecycle.clone(),
            permissions: context.permissions.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionQueryResult {
    pub permission: String,
    pub state: PermissionQueryState,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionQueryState {
    Granted,
    Denied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PluginRuntimeErrorCode {
    PermissionDenied,
    InvalidParams,
    Unsupported,
    Internal,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginRuntimeError {
    pub code: PluginRuntimeErrorCode,
    pub message: String,
}

impl PluginRuntimeError {
    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self {
            code: PluginRuntimeErrorCode::PermissionDenied,
            message: message.into(),
        }
    }

    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self {
            code: PluginRuntimeErrorCode::InvalidParams,
            message: message.into(),
        }
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self {
            code: PluginRuntimeErrorCode::Unsupported,
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            code: PluginRuntimeErrorCode::Internal,
            message: message.into(),
        }
    }
}
