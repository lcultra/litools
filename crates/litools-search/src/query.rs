use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SearchQuery {
    pub text: String,
    pub limit: Option<usize>,
}

impl SearchQuery {
    pub fn new(text: impl Into<String>) -> Self {
        Self::with_limit(text, 20)
    }

    pub fn with_limit(text: impl Into<String>, limit: usize) -> Self {
        Self {
            text: text.into(),
            limit: Some(limit),
        }
    }

    pub fn without_limit(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            limit: None,
        }
    }
}
