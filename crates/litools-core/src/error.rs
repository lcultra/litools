pub type LitoolsResult<T> = Result<T, LitoolsError>;

#[derive(Debug, thiserror::Error)]
pub enum LitoolsError {
    #[error("index error: {0}")]
    Index(#[from] rusqlite::Error),
    #[error("command not found: {0}")]
    CommandNotFound(String),
}
