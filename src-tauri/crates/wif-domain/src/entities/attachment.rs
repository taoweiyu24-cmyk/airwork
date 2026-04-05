use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// A file attachment associated with a work item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: Ulid,
    pub work_item_id: Ulid,
    pub file_name: String,
    pub file_path: String,
    pub content_type: String,
    pub size: i64,
    /// Unix timestamp (seconds).
    pub created_at: i64,
}
