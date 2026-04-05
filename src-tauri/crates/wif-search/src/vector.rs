//! Vector search trait and a no-op stub implementation.
//!
//! The full DuckDB-backed implementation will be added once the Windows build
//! environment supports the DuckDB crate.

use crate::error::SearchError;

/// Interface for vector embedding storage and similarity search.
pub trait VectorStore: Send + Sync {
    /// Store an embedding for a document identified by `id`.
    fn store_embedding(&self, id: &str, embedding: &[f32]) -> Result<(), SearchError>;

    /// Return the `limit` most similar document IDs and their similarity scores.
    fn search_similar(
        &self,
        embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<(String, f64)>, SearchError>;

    /// Remove the embedding for the document identified by `id`.
    fn remove_embedding(&self, id: &str) -> Result<(), SearchError>;
}

/// A no-op [`VectorStore`] that always returns empty results.
///
/// Used as the default until a real vector store is configured.
pub struct NoOpVectorStore;

impl VectorStore for NoOpVectorStore {
    fn store_embedding(&self, _id: &str, _embedding: &[f32]) -> Result<(), SearchError> {
        Ok(())
    }

    fn search_similar(
        &self,
        _embedding: &[f32],
        _limit: usize,
    ) -> Result<Vec<(String, f64)>, SearchError> {
        Ok(vec![])
    }

    fn remove_embedding(&self, _id: &str) -> Result<(), SearchError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_store_embedding_succeeds() {
        let store = NoOpVectorStore;
        assert!(store.store_embedding("id1", &[0.1, 0.2, 0.3]).is_ok());
    }

    #[test]
    fn noop_search_similar_returns_empty() {
        let store = NoOpVectorStore;
        let results = store.search_similar(&[0.1, 0.2], 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn noop_remove_embedding_succeeds() {
        let store = NoOpVectorStore;
        assert!(store.remove_embedding("id1").is_ok());
    }
}
