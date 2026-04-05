//! High-level [`HybridSearchService`] that combines keyword and vector search.

use wif_data::Database;

use crate::error::SearchError;
use crate::keyword::KeywordSearchService;
use crate::types::{SearchQuery, SearchResult};
use crate::vector::{NoOpVectorStore, VectorStore};

/// Orchestrates keyword search (FTS5) and vector search, fusing results with RRF.
pub struct HybridSearchService<'a> {
    keyword: KeywordSearchService<'a>,
    vector: Box<dyn VectorStore>,
    /// RRF smoothing constant (typically 60).  Reserved for the full hybrid
    /// implementation once a vector store is wired up.
    #[allow(dead_code)]
    k: u32,
}

impl<'a> HybridSearchService<'a> {
    /// Create a new service backed by `db`, using a no-op vector store.
    pub fn new(db: &'a Database) -> Self {
        Self {
            keyword: KeywordSearchService::new(db),
            vector: Box::new(NoOpVectorStore),
            k: 60,
        }
    }

    /// Replace the default no-op vector store with a real implementation.
    pub fn with_vector_store(mut self, store: Box<dyn VectorStore>) -> Self {
        self.vector = store;
        self
    }

    /// Perform a hybrid search: keyword results are returned directly.
    ///
    /// When a vector store and embedding service are available, the query
    /// would be embedded and vector results fused via RRF.  Until then,
    /// this delegates to [`keyword_search`](Self::keyword_search).
    pub fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
        // Full hybrid flow (future):
        //   1. embed query text via AI service
        //   2. run self.vector.search_similar(...)
        //   3. run self.keyword.search(...)
        //   4. hybrid::reciprocal_rank_fusion(keyword, vector, self.k)
        //
        // For now, return keyword results only.
        self.keyword.search(query)
    }

    /// Run only the FTS5 keyword search.
    pub fn keyword_search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
        self.keyword.search(query)
    }
}
