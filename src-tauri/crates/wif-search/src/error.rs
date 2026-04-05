/// Errors that can occur during search operations.
#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("Search query error: {0}")]
    QueryError(String),
    #[error("Index error: {0}")]
    IndexError(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}
