use ulid::Ulid;
use wif_domain::entities::Attachment;
use wif_domain::repository::{DomainError, Result, AttachmentRepository};

use crate::connection::Database;
use crate::repositories::util::{anyhow_to_domain, parse_ulid_col};

pub struct SqliteAttachmentRepo<'a> {
    db: &'a Database,
}

impl<'a> SqliteAttachmentRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    fn row_to_attachment(row: &rusqlite::Row) -> rusqlite::Result<Attachment> {
        let id_str: String = row.get(0)?;
        let wi_str: String = row.get(1)?;
        Ok(Attachment {
            id: parse_ulid_col(&id_str, 0)?,
            work_item_id: parse_ulid_col(&wi_str, 1)?,
            file_name: row.get(2)?,
            file_path: row.get(3)?,
            content_type: row.get(4)?,
            size: row.get(5)?,
            created_at: row.get(6)?,
        })
    }
}

impl<'a> AttachmentRepository for SqliteAttachmentRepo<'a> {
    fn find_by_work_item(&self, work_item_id: Ulid) -> Result<Vec<Attachment>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id,work_item_id,file_name,file_path,content_type,size,created_at FROM attachments WHERE work_item_id=?1 ORDER BY created_at DESC",
            )?;
            let rows = stmt.query_map([work_item_id.to_string()], Self::row_to_attachment)?;
            rows.map(|r| r.map_err(|e| anyhow::anyhow!(e))).collect()
        }).map_err(anyhow_to_domain)
    }

    fn create(&self, attachment: &Attachment) -> Result<Attachment> {
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO attachments (id,work_item_id,file_name,file_path,content_type,size,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                rusqlite::params![
                    attachment.id.to_string(),
                    attachment.work_item_id.to_string(),
                    attachment.file_name, attachment.file_path,
                    attachment.content_type, attachment.size, attachment.created_at,
                ],
            )?;
            Ok(attachment.clone())
        }).map_err(anyhow_to_domain)
    }

    fn delete(&self, id: Ulid) -> Result<()> {
        self.db.with_conn(|conn| {
            let n = conn.execute("DELETE FROM attachments WHERE id=?1", [id.to_string()])?;
            if n == 0 {
                return Err(anyhow::anyhow!(DomainError::NotFound(id.to_string())));
            }
            Ok(())
        }).map_err(anyhow_to_domain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::Database;
    use crate::repositories::work_item_repo::SqliteWorkItemRepo;
    use wif_domain::entities::WorkItem;
    use wif_domain::enums::{Priority, Source, WorkItemStatus};
    use wif_domain::repository::WorkItemRepository;

    fn setup() -> Database {
        let db = Database::open_in_memory().unwrap();
        db.run_migrations().unwrap();
        db
    }

    fn insert_work_item(db: &Database) -> Ulid {
        let repo = SqliteWorkItemRepo::new(db);
        let item = WorkItem {
            id: Ulid::new(), title: "wi".into(), content: None,
            status: WorkItemStatus::New, priority: Priority::Normal,
            source: Source::Manual, tags: vec![],
            created_at: 1_000_000, updated_at: 1_000_000,
            parent_id: None, latitude: None, longitude: None, gis_feature_id: None,
        };
        repo.create(&item).unwrap();
        item.id
    }

    #[test]
    fn create_and_find_attachment() {
        let db = setup();
        let wi_id = insert_work_item(&db);
        let repo = SqliteAttachmentRepo::new(&db);
        let att = Attachment {
            id: Ulid::new(), work_item_id: wi_id,
            file_name: "report.pdf".into(), file_path: "/tmp/report.pdf".into(),
            content_type: "application/pdf".into(), size: 12345, created_at: 1_000_000,
        };
        repo.create(&att).unwrap();
        let found = repo.find_by_work_item(wi_id).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].file_name, "report.pdf");
    }

    #[test]
    fn delete_attachment() {
        let db = setup();
        let wi_id = insert_work_item(&db);
        let repo = SqliteAttachmentRepo::new(&db);
        let att = Attachment {
            id: Ulid::new(), work_item_id: wi_id,
            file_name: "x.txt".into(), file_path: "/tmp/x.txt".into(),
            content_type: "text/plain".into(), size: 10, created_at: 1_000_000,
        };
        repo.create(&att).unwrap();
        repo.delete(att.id).unwrap();
        assert!(repo.find_by_work_item(wi_id).unwrap().is_empty());
    }
}
