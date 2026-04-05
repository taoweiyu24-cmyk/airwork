use ulid::Ulid;

use crate::entities::{Proposal, WorkItem};
use crate::enums::WorkItemStatus;
use crate::repository::Result;

/// Repository interface for [`WorkItem`] persistence.
pub trait WorkItemRepository: Send + Sync {
    fn find_all(&self) -> Result<Vec<WorkItem>>;
    fn find_by_id(&self, id: Ulid) -> Result<Option<WorkItem>>;
    fn find_by_status(&self, status: WorkItemStatus) -> Result<Vec<WorkItem>>;
    fn create(&self, item: &WorkItem) -> Result<WorkItem>;
    fn update(&self, item: &WorkItem) -> Result<WorkItem>;
    fn delete(&self, id: Ulid) -> Result<()>;
    fn find_children(&self, parent_id: Ulid) -> Result<Vec<WorkItem>>;
    fn update_location(&self, id: Ulid, lat: Option<f64>, lon: Option<f64>) -> Result<()>;
}

/// Repository interface for [`Proposal`] persistence.
pub trait ProposalRepository: Send + Sync {
    fn find_by_work_item(&self, work_item_id: Ulid) -> Result<Vec<Proposal>>;
    fn create(&self, proposal: &Proposal) -> Result<Proposal>;
    fn delete(&self, id: Ulid) -> Result<()>;
}
