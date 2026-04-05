use ulid::Ulid;

use crate::entities::AiProfile;
use crate::repository::Result;

/// Repository interface for [`AiProfile`] persistence.
pub trait AiProfileRepository: Send + Sync {
    fn find_all(&self) -> Result<Vec<AiProfile>>;
    fn find_default(&self) -> Result<Option<AiProfile>>;
    fn create(&self, profile: &AiProfile) -> Result<AiProfile>;
    fn update(&self, profile: &AiProfile) -> Result<AiProfile>;
    fn delete(&self, id: Ulid) -> Result<()>;
}
