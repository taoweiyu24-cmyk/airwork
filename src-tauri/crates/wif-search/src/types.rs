use serde::{Deserialize, Serialize};

/// A single result returned by any search strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    /// ULID of the matching work item, as a string.
    pub id: String,
    /// Title of the work item.
    pub title: String,
    /// Highlighted snippet from the content.
    pub snippet: String,
    /// Relevance score in the range 0–1.
    pub score: f64,
    /// Which search strategy produced this result.
    pub source: SearchSource,
}

/// Indicates which search strategy produced a [`SearchResult`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SearchSource {
    Keyword,
    Vector,
    Hybrid,
}

/// Input parameters for a search request.
#[derive(Debug, Clone)]
pub struct SearchQuery {
    /// The user-supplied search text.
    pub text: String,
    /// Maximum number of results to return.
    pub limit: usize,
    /// Number of results to skip (for pagination).
    pub offset: usize,
}
