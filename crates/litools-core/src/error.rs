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
}
