use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::input::InputContext;
use crate::query::SearchQuery;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: SearchQuery,
    pub context: InputContext,
    /// 预留扩展字段，Phase 4A 固定为空。
    pub metadata: HashMap<String, serde_json::Value>,
}
