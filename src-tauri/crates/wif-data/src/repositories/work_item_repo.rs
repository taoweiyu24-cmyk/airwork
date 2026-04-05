use ulid::Ulid;
use wif_domain::entities::WorkItem;
use wif_domain::enums::{Priority, Source, WorkItemStatus};
use wif_domain::repository::{DomainError, Result, WorkItemRepository};

use crate::connection::Database;
use crate::repositories::util::{anyhow_to_domain, to_json_array, parse_ulid_col};

pub struct SqliteWorkItemRepo<'a> {
    db: &'a Database,
}

impl<'a> SqliteWorkItemRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    fn row_to_work_item(row: &rusqlite::Row) -> rusqlite::Result<WorkItem> {
        let id_str: String = row.get(0)?;
        let tags_str: String = row.get(6)?;
        let parent_str: Option<String> = row.get(9)?;
        let gis_str: Option<String> = row.get(12)?;
        let status_str: String = row.get(3)?;
        let priority_str: String = row.get(4)?;
        let source_str: String = row.get(5)?;

        let id = parse_ulid_col(&id_str, 0)?;
        let tags = serde_json::from_str::<Vec<String>>(&tags_str).unwrap_or_default();
        let parent_id = parent_str.as_deref().map(|s| parse_ulid_col(s, 9)).transpose()?;
        let gis_feature_id = gis_str.as_deref().map(|s| parse_ulid_col(s, 12)).transpose()?;

        Ok(WorkItem {
            id,
            title: row.get(1)?,
            content: row.get(2)?,
            status: status_from_str(&status_str),
            priority: priority_from_str(&priority_str),
            source: source_from_str(&source_str),
            tags,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
            parent_id,
            latitude: row.get(10)?,
            longitude: row.get(11)?,
            gis_feature_id,
        })
    }
}

const SELECT_COLS: &str =
    "SELECT id,title,content,status,priority,source,tags,created_at,updated_at,parent_id,latitude,longitude,gis_feature_id FROM work_items";

impl<'a> WorkItemRepository for SqliteWorkItemRepo<'a> {
    fn find_all(&self) -> Result<Vec<WorkItem>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(&format!("{SELECT_COLS} ORDER BY created_at DESC"))?;
            let rows = stmt.query_map([], Self::row_to_work_item)?;
            rows.map(|r| r.map_err(|e| anyhow::anyhow!(e))).collect()
        }).map_err(anyhow_to_domain)
    }

    fn find_by_id(&self, id: Ulid) -> Result<Option<WorkItem>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(&format!("{SELECT_COLS} WHERE id=?1"))?;
            let mut rows = stmt.query_map([id.to_string()], Self::row_to_work_item)?;
            match rows.next() {
                None => Ok(None),
                Some(r) => r.map(Some).map_err(|e| anyhow::anyhow!(e)),
            }
        }).map_err(anyhow_to_domain)
    }

    fn find_by_status(&self, status: WorkItemStatus) -> Result<Vec<WorkItem>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(&format!("{SELECT_COLS} WHERE status=?1 ORDER BY created_at DESC"))?;
            let rows = stmt.query_map([status_to_str(status)], Self::row_to_work_item)?;
            rows.map(|r| r.map_err(|e| anyhow::anyhow!(e))).collect()
        }).map_err(anyhow_to_domain)
    }

    fn create(&self, item: &WorkItem) -> Result<WorkItem> {
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO work_items (id,title,content,status,priority,source,tags,created_at,updated_at,parent_id,latitude,longitude,gis_feature_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
                rusqlite::params![
                    item.id.to_string(),
                    item.title,
                    item.content,
                    status_to_str(item.status),
                    priority_to_str(item.priority),
                    source_to_str(item.source),
                    to_json_array(&item.tags),
                    item.created_at,
                    item.updated_at,
                    item.parent_id.map(|u| u.to_string()),
                    item.latitude,
                    item.longitude,
                    item.gis_feature_id.map(|u| u.to_string()),
                ],
            )?;
            Ok(item.clone())
        }).map_err(anyhow_to_domain)
    }

    fn update(&self, item: &WorkItem) -> Result<WorkItem> {
        self.db.with_conn(|conn| {
            let n = conn.execute(
                "UPDATE work_items SET title=?1,content=?2,status=?3,priority=?4,source=?5,tags=?6,updated_at=?7,parent_id=?8,latitude=?9,longitude=?10,gis_feature_id=?11 WHERE id=?12",
                rusqlite::params![
                    item.title,
                    item.content,
                    status_to_str(item.status),
                    priority_to_str(item.priority),
                    source_to_str(item.source),
                    to_json_array(&item.tags),
                    item.updated_at,
                    item.parent_id.map(|u| u.to_string()),
                    item.latitude,
                    item.longitude,
                    item.gis_feature_id.map(|u| u.to_string()),
                    item.id.to_string(),
                ],
            )?;
            if n == 0 {
                return Err(anyhow::anyhow!(DomainError::NotFound(item.id.to_string())));
            }
            Ok(item.clone())
        }).map_err(anyhow_to_domain)
    }

    fn delete(&self, id: Ulid) -> Result<()> {
        self.db.with_conn(|conn| {
            let n = conn.execute("DELETE FROM work_items WHERE id=?1", [id.to_string()])?;
            if n == 0 {
                return Err(anyhow::anyhow!(DomainError::NotFound(id.to_string())));
            }
            Ok(())
        }).map_err(anyhow_to_domain)
    }

    fn find_children(&self, parent_id: Ulid) -> Result<Vec<WorkItem>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(&format!("{SELECT_COLS} WHERE parent_id=?1 ORDER BY created_at DESC"))?;
            let rows = stmt.query_map([parent_id.to_string()], Self::row_to_work_item)?;
            rows.map(|r| r.map_err(|e| anyhow::anyhow!(e))).collect()
        }).map_err(anyhow_to_domain)
    }

    fn update_location(&self, id: Ulid, lat: Option<f64>, lon: Option<f64>) -> Result<()> {
        self.db.with_conn(|conn| {
            let n = conn.execute(
                "UPDATE work_items SET latitude=?1,longitude=?2 WHERE id=?3",
                rusqlite::params![lat, lon, id.to_string()],
            )?;
            if n == 0 {
                return Err(anyhow::anyhow!(DomainError::NotFound(id.to_string())));
            }
            Ok(())
        }).map_err(anyhow_to_domain)
    }
}

// --- enum helpers ---

fn status_to_str(s: WorkItemStatus) -> &'static str {
    match s {
        WorkItemStatus::New => "new",
        WorkItemStatus::InProgress => "inProgress",
        WorkItemStatus::Reviewing => "reviewing",
        WorkItemStatus::Done => "done",
        WorkItemStatus::Archived => "archived",
    }
}

fn status_from_str(s: &str) -> WorkItemStatus {
    match s {
        "inProgress" => WorkItemStatus::InProgress,
        "reviewing" => WorkItemStatus::Reviewing,
        "done" => WorkItemStatus::Done,
        "archived" => WorkItemStatus::Archived,
        _ => WorkItemStatus::New,
    }
}

fn priority_to_str(p: Priority) -> &'static str {
    match p {
        Priority::Low => "low",
        Priority::Normal => "normal",
        Priority::High => "high",
        Priority::Urgent => "urgent",
    }
}

fn priority_from_str(s: &str) -> Priority {
    match s {
        "low" => Priority::Low,
        "high" => Priority::High,
        "urgent" => Priority::Urgent,
        _ => Priority::Normal,
    }
}

fn source_to_str(s: Source) -> &'static str {
    match s {
        Source::Manual => "manual",
        Source::Email => "email",
        Source::File => "file",
        Source::Api => "api",
    }
}

fn source_from_str(s: &str) -> Source {
    match s {
        "email" => Source::Email,
        "file" => Source::File,
        "api" => Source::Api,
        _ => Source::Manual,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::Database;

    fn setup() -> Database {
        let db = Database::open_in_memory().unwrap();
        db.run_migrations().unwrap();
        db
    }

    fn sample_item() -> WorkItem {
        WorkItem {
            id: Ulid::new(),
            title: "Test item".into(),
            content: Some("body".into()),
            status: WorkItemStatus::New,
            priority: Priority::Normal,
            source: Source::Manual,
            tags: vec!["a".into(), "b".into()],
            created_at: 1_000_000,
            updated_at: 1_000_000,
            parent_id: None,
            latitude: None,
            longitude: None,
            gis_feature_id: None,
        }
    }

    #[test]
    fn create_and_find_by_id() {
        let db = setup();
        let repo = SqliteWorkItemRepo::new(&db);
        let item = sample_item();
        repo.create(&item).unwrap();
        let found = repo.find_by_id(item.id).unwrap().unwrap();
        assert_eq!(found.title, "Test item");
        assert_eq!(found.tags, vec!["a", "b"]);
    }

    #[test]
    fn find_all_returns_items() {
        let db = setup();
        let repo = SqliteWorkItemRepo::new(&db);
        repo.create(&sample_item()).unwrap();
        assert_eq!(repo.find_all().unwrap().len(), 1);
    }

    #[test]
    fn update_item() {
        let db = setup();
        let repo = SqliteWorkItemRepo::new(&db);
        let mut item = sample_item();
        repo.create(&item).unwrap();
        item.title = "Updated".into();
        item.updated_at = 2_000_000;
        let updated = repo.update(&item).unwrap();
        assert_eq!(updated.title, "Updated");
    }

    #[test]
    fn delete_item() {
        let db = setup();
        let repo = SqliteWorkItemRepo::new(&db);
        let item = sample_item();
        repo.create(&item).unwrap();
        repo.delete(item.id).unwrap();
        assert!(repo.find_by_id(item.id).unwrap().is_none());
    }

    #[test]
    fn find_by_status() {
        let db = setup();
        let repo = SqliteWorkItemRepo::new(&db);
        repo.create(&sample_item()).unwrap();
        let results = repo.find_by_status(WorkItemStatus::New).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn update_location() {
        let db = setup();
        let repo = SqliteWorkItemRepo::new(&db);
        let item = sample_item();
        repo.create(&item).unwrap();
        repo.update_location(item.id, Some(39.9), Some(116.3)).unwrap();
        let found = repo.find_by_id(item.id).unwrap().unwrap();
        assert_eq!(found.latitude, Some(39.9));
    }
}
