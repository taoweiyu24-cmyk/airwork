//! `wif-search` — hybrid search engine for WorkItemFlow.
//!
//! Provides FTS5 full-text keyword search (via `rusqlite`) and a trait-based
//! vector search interface for future DuckDB integration.  Results are fused
//! with Reciprocal Rank Fusion (RRF, k=60).

pub mod error;
pub mod hybrid;
pub mod keyword;
pub mod service;
pub mod types;
pub mod vector;

pub use error::SearchError;
pub use hybrid::reciprocal_rank_fusion;
pub use keyword::KeywordSearchService;
pub use service::HybridSearchService;
pub use types::{SearchQuery, SearchResult, SearchSource};
pub use vector::{NoOpVectorStore, VectorStore};
