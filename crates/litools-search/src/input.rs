use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── feature_kinds ──

pub mod feature_kinds {
    pub const JSON: &str = "json";
    pub const URL: &str = "url";
    pub const BASE64: &str = "base64";
    pub const IMAGE: &str = "image";
    pub const FILE: &str = "file";
    pub const CURL: &str = "curl";
    pub const JWT: &str = "jwt";
    pub const UUID: &str = "uuid";
    pub const COLOR: &str = "color";
    pub const MARKDOWN: &str = "markdown";
}

// ── SearchFeature ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFeature {
    pub kind: String,
    pub source: String,
    pub confidence: f32,
    pub metadata: HashMap<String, serde_json::Value>,
}

// ── SearchAttachment ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchAttachment {
    pub kind: AttachmentKind,
    pub data: Vec<u8>,
    pub mime_type: Option<String>,
    pub filename: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttachmentKind {
    Clipboard,
    DragDrop,
    FilePath,
}

// ── InputContext ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputContext {
    pub version: u32,
    pub raw: String,
    pub normalized: String,
    pub features: Vec<SearchFeature>,
    pub attachments: Vec<SearchAttachment>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl InputContext {
    pub fn empty() -> Self {
        Self {
            version: 1,
            raw: String::new(),
            normalized: String::new(),
            features: vec![],
            attachments: vec![],
            metadata: HashMap::new(),
        }
    }

    pub fn has_feature(&self, kind: &str) -> bool {
        self.features.iter().any(|f| f.kind == kind)
    }

    pub fn first_feature(&self, kind: &str) -> Option<&SearchFeature> {
        self.features.iter().find(|f| f.kind == kind)
    }

    pub fn features_of_kind(&self, kind: &str) -> Vec<&SearchFeature> {
        self.features.iter().filter(|f| f.kind == kind).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_feature(kind: &str, source: &str) -> SearchFeature {
        SearchFeature {
            kind: kind.to_string(),
            source: source.to_string(),
            confidence: 1.0,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn empty_context_has_no_features() {
        let ctx = InputContext::empty();
        assert!(!ctx.has_feature("json"));
        assert!(ctx.first_feature("json").is_none());
    }

    #[test]
    fn has_feature_finds_match() {
        let ctx = InputContext {
            features: vec![
                make_feature("json", "builtin.json"),
                make_feature("url", "builtin.url"),
            ],
            ..InputContext::empty()
        };
        assert!(ctx.has_feature("json"));
        assert!(!ctx.has_feature("base64"));
    }

    #[test]
    fn first_feature_returns_first_match() {
        let ctx = InputContext {
            features: vec![
                make_feature("url", "builtin.url"),
                make_feature("url", "plugin.dev.test"),
            ],
            ..InputContext::empty()
        };
        let f = ctx.first_feature("url").unwrap();
        assert_eq!(f.source, "builtin.url");
    }

    #[test]
    fn features_of_kind_collects_all() {
        let ctx = InputContext {
            features: vec![
                make_feature("url", "builtin.url"),
                make_feature("url", "plugin.dev.test"),
            ],
            ..InputContext::empty()
        };
        assert_eq!(ctx.features_of_kind("url").len(), 2);
    }
}
