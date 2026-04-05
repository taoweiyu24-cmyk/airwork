use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// An auditable event attached to a [`WorkItem`](crate::entities::WorkItem).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkEvent {
    pub id: Ulid,
    pub work_item_id: Ulid,
    pub event_type: String,
    pub content: Option<String>,
    /// Unix timestamp (seconds).
    pub created_at: i64,
}
