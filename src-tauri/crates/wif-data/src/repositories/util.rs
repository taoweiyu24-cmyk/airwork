use ulid::Ulid;
use wif_domain::repository::DomainError;

/// Convert an anyhow error to DomainError (for use with `with_conn`).
pub fn anyhow_to_domain(e: anyhow::Error) -> DomainError {
    DomainError::DatabaseError(e.to_string())
}

/// Parse a ULID stored as TEXT, returning a rusqlite conversion error.
pub fn parse_ulid_col(s: &str, col: usize) -> rusqlite::Result<Ulid> {
    Ulid::from_string(s)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(col, rusqlite::types::Type::Text, Box::new(e)))
}

/// Serialise a Vec<String> to a JSON array string.
pub fn to_json_array(v: &[String]) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| "[]".to_string())
}
