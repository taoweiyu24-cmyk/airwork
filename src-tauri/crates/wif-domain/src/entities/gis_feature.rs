use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// A geographic feature belonging to a [`GisLayer`](crate::entities::GisLayer).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GisFeature {
    pub id: Ulid,
    pub layer_id: Ulid,
    /// Geometry encoded as Well-Known Text (WKT).
    pub geometry_wkt: String,
    /// Arbitrary feature properties serialised as JSON.
    pub properties_json: Option<String>,
    pub work_item_id: Option<Ulid>,
    /// Unix timestamp (seconds).
    pub created_at: i64,
}
