use ulid::Ulid;

use crate::entities::{MailAccount, MailMessage};
use crate::repository::Result;

/// Repository interface for [`MailMessage`] persistence.
pub trait MailMessageRepository: Send + Sync {
    fn find_by_account(&self, account_id: Ulid) -> Result<Vec<MailMessage>>;
    fn find_by_id(&self, id: Ulid) -> Result<Option<MailMessage>>;
    fn create(&self, msg: &MailMessage) -> Result<MailMessage>;
    fn mark_read(&self, id: Ulid) -> Result<()>;
    fn link_to_work_item(&self, id: Ulid, work_item_id: Ulid) -> Result<()>;
}

/// Repository interface for [`MailAccount`] persistence.
pub trait MailAccountRepository: Send + Sync {
    fn find_all(&self) -> Result<Vec<MailAccount>>;
    fn find_by_id(&self, id: Ulid) -> Result<Option<MailAccount>>;
    fn create(&self, account: &MailAccount) -> Result<MailAccount>;
    fn update(&self, account: &MailAccount) -> Result<MailAccount>;
    fn delete(&self, id: Ulid) -> Result<()>;
}
