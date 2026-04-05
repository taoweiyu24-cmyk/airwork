use ulid::Ulid;

use crate::entities::{GisFeature, GisLayer};
use crate::repository::Result;

/// Repository interface for [`GisLayer`] persistence.
pub trait GisLayerRepository: Send + Sync {
    fn find_all(&self) -> Result<Vec<GisLayer>>;
    fn find_by_id(&self, id: Ulid) -> Result<Option<GisLayer>>;
    fn create(&self, layer: &GisLayer) -> Result<GisLayer>;
    fn update(&self, layer: &GisLayer) -> Result<GisLayer>;
    fn delete(&self, id: Ulid) -> Result<()>;
}

/// Repository interface for [`GisFeature`] persistence.
pub trait GisFeatureRepository: Send + Sync {
    fn find_by_layer(&self, layer_id: Ulid) -> Result<Vec<GisFeature>>;
    fn find_by_id(&self, id: Ulid) -> Result<Option<GisFeature>>;
    fn find_by_work_item(&self, work_item_id: Ulid) -> Result<Vec<GisFeature>>;
    fn create(&self, feature: &GisFeature) -> Result<GisFeature>;
    fn update(&self, feature: &GisFeature) -> Result<GisFeature>;
    fn delete(&self, id: Ulid) -> Result<()>;
}
