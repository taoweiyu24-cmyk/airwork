use ulid::Ulid;

use crate::entities::Contact;
use crate::repository::Result;

/// Repository interface for [`Contact`] persistence.
pub trait ContactRepository: Send + Sync {
    fn find_all(&self) -> Result<Vec<Contact>>;
    fn find_by_id(&self, id: Ulid) -> Result<Option<Contact>>;
    fn find_by_email(&self, email: &str) -> Result<Option<Contact>>;
    fn create(&self, contact: &Contact) -> Result<Contact>;
    fn update(&self, contact: &Contact) -> Result<Contact>;
    fn delete(&self, id: Ulid) -> Result<()>;
}
