use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::enums::AnalysisType;

/// An AI-generated proposal or analysis result for a work item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Proposal {
    pub id: Ulid,
    pub work_item_id: Ulid,
    pub analysis_type: AnalysisType,
    pub content: String,
    /// Unix timestamp (seconds).
    pub created_at: i64,
}
