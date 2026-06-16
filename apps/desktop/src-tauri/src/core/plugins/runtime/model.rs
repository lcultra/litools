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
    /// 关联的 Surface ID，运行时通过它单向引用窗口/webview 元数据。
    pub surface_id: String,
    pub permissions: Vec<String>,
    /// 是否为 trusted（bundled）插件的运行时。内部权限仅 trusted 插件可用。
    pub trusted: bool,
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
    pub surface_id: String,
    pub host_kind: Option<String>,
    pub lifecycle: PluginRuntimeLifecycle,
    pub permissions: Vec<String>,
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
    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
