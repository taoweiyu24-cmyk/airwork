use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::enums::LayerType;

/// A GIS map layer definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GisLayer {
    pub id: Ulid,
    pub name: String,
    pub layer_type: LayerType,
    pub source_path: Option<String>,
    pub visible: bool,
    /// Opacity in the range `[0.0, 1.0]`.
    pub opacity: f32,
    /// Serialised style definition (provider-specific JSON).
    pub style_json: Option<String>,
    /// Unix timestamp (seconds).
    pub created_at: i64,
}
