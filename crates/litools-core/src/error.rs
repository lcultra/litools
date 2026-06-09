pub type LitoolsResult<T> = Result<T, LitoolsError>;

#[derive(Debug, thiserror::Error)]
pub enum LitoolsError {
    #[error("索引错误：{0}")]
    Index(#[from] rusqlite::Error),
    #[error("文件读写错误：{0}")]
    Io(#[from] std::io::Error),
    #[error("JSON 解析错误：{0}")]
    Json(#[from] serde_json::Error),
    #[error("未找到命令：{0}")]
    CommandNotFound(String),
    #[error("未找到应用：{0}")]
    AppNotFound(String),
    #[error("未找到插件：{0}")]
    PluginNotFound(String),
    #[error("插件已禁用：{0}")]
    PluginDisabled(String),
    #[error("无效操作：{0}")]
    InvalidAction(String),
    #[error("系统操作失败：{0}")]
    System(String),
}

impl LitoolsError {
    /// Formats the error with its error code prefix, suitable for IPC responses.
    ///
    /// Format: `[ERROR_CODE] message`
    pub fn to_error_string(&self) -> String {
        format!("[{}] {}", self.error_code(), self)
    }

    /// Returns a stable error code for frontend error handling.
    pub fn error_code(&self) -> &'static str {
        match self {
            LitoolsError::Index(_) => "INDEX_ERROR",
            LitoolsError::Io(_) => "IO_ERROR",
            LitoolsError::Json(_) => "JSON_ERROR",
            LitoolsError::CommandNotFound(_) => "COMMAND_NOT_FOUND",
            LitoolsError::AppNotFound(_) => "APP_NOT_FOUND",
            LitoolsError::PluginNotFound(_) => "PLUGIN_NOT_FOUND",
            LitoolsError::PluginDisabled(_) => "PLUGIN_DISABLED",
            LitoolsError::InvalidAction(_) => "INVALID_ACTION",
            LitoolsError::System(_) => "SYSTEM_ERROR",
        }
    }
}
