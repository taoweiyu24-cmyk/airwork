use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::enums::{Priority, Source, WorkItemStatus};

/// A work item — the central entity of WorkItemFlow.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkItem {
    pub id: Ulid,
    pub title: String,
    pub content: Option<String>,
    pub status: WorkItemStatus,
    pub priority: Priority,
    pub source: Source,
    pub tags: Vec<String>,
    /// Unix timestamp (seconds).
    pub created_at: i64,
    /// Unix timestamp (seconds).
    pub updated_at: i64,
    pub parent_id: Option<Ulid>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub gis_feature_id: Option<Ulid>,
}
