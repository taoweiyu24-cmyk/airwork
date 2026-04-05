use ulid::Ulid;

use crate::entities::Attachment;
use crate::repository::Result;

/// Repository interface for [`Attachment`] persistence.
pub trait AttachmentRepository: Send + Sync {
    fn find_by_work_item(&self, work_item_id: Ulid) -> Result<Vec<Attachment>>;
    fn create(&self, attachment: &Attachment) -> Result<Attachment>;
    fn delete(&self, id: Ulid) -> Result<()>;
}
