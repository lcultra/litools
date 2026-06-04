use std::sync::Arc;

use litools_index::IndexDatabase;
use litools_search::{SearchEngine, SearchQuery, SearchResult};
use litools_telemetry::init_logging;

use crate::{command::BuiltinCommandProvider, context::AppContext, error::LitoolsResult};

pub struct LitoolsApp {
    context: AppContext,
}

impl LitoolsApp {
    pub fn bootstrap_in_memory() -> LitoolsResult<Self> {
        init_logging();

        let database = IndexDatabase::in_memory()?;
        let mut search = SearchEngine::new();
        search.register_provider(Arc::new(BuiltinCommandProvider));

        Ok(Self {
            context: AppContext::new(database, search),
        })
    }

    pub fn search(&self, text: impl Into<String>) -> Vec<SearchResult> {
        self.context.search.search(SearchQuery::new(text))
    }

    pub fn context(&self) -> &AppContext {
        &self.context
    }
}
