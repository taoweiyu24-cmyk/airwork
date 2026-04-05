use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// A person or organisation contact.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    pub id: Ulid,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub organization: Option<String>,
    /// Unix timestamp (seconds).
    pub created_at: i64,
}
