use serde::{Deserialize, Serialize};

/// Metadata extracted from a document file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentMetadata {
    pub file_name: String,
    pub file_size: u64,
    pub content_type: String,
    pub page_count: Option<u32>,
    pub author: Option<String>,
    pub created_date: Option<String>,
    pub modified_date: Option<String>,
    pub title: Option<String>,
}
