pub mod engine;
pub mod matcher;
pub mod provider;
pub mod query;
pub mod ranking;

pub use engine::SearchEngine;
pub use matcher::{
    FieldMatcher, FieldWeights, MatchKind, MatchRange, SearchResultMatches, TextMatch,
    VisibleField, match_text,
};
pub use provider::{SearchContext, SearchProvider, SearchResult, SearchResultAction};
pub use query::SearchQuery;
