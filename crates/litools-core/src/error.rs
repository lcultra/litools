pub type LitoolsResult<T> = Result<T, LitoolsError>;

#[derive(Debug, thiserror::Error)]
pub enum LitoolsError {
    #[error("index error: {0}")]
    Index(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("command not found: {0}")]
    CommandNotFound(String),
}
