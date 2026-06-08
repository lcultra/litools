pub mod engine;
pub mod matcher;
pub mod provider;
pub mod query;
pub mod ranking;

pub use engine::SearchEngine;
pub use matcher::{MatchKind, MatchRange, SearchResultMatches, TextMatch, match_text};
pub use provider::{SearchProvider, SearchResult, SearchResultAction};
pub use query::SearchQuery;
