//! Domain enums for WorkItemFlow.

use serde::{Deserialize, Serialize};

/// Work item lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WorkItemStatus {
    New,
    InProgress,
    Reviewing,
    Done,
    Archived,
}

/// Work item priority level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Priority {
    Low,
    Normal,
    High,
    Urgent,
}

/// Origin source of a work item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Source {
    Manual,
    Email,
    File,
    Api,
}

/// Type of AI analysis performed on a work item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AnalysisType {
    Summary,
    ActionExtraction,
    Classification,
    PrioritySuggestion,
}

/// GIS layer data type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LayerType {
    Vector,
    Raster,
    Tile,
}

/// OGC geometry type used in GIS features.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GeometryType {
    Point,
    LineString,
    Polygon,
    MultiPoint,
    MultiLineString,
    MultiPolygon,
}
