pub mod detector;
pub mod engine;
pub mod input;
pub mod matcher;
pub mod provider;
pub mod query;
pub mod ranking;
pub mod request;

pub use detector::{Detection, InputDetector};
pub use engine::SearchEngine;
pub use input::{
    AttachmentKind, InputContext, SearchAttachment, SearchFeature, feature_kinds,
};
pub use matcher::{
    FieldMatcher, FieldWeights, MatchKind, MatchRange, SearchResultMatches, TextMatch,
    VisibleField, match_text,
};
pub use provider::{SearchProvider, SearchResult, SearchResultAction};
pub use query::SearchQuery;
pub use request::SearchRequest;
