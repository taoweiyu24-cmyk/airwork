use ulid::Ulid;
use wif_domain::entities::Proposal;
use wif_domain::enums::AnalysisType;
use wif_domain::repository::{DomainError, Result, ProposalRepository};

use crate::connection::Database;
use crate::repositories::util::{anyhow_to_domain, parse_ulid_col};

pub struct SqliteProposalRepo<'a> {
    db: &'a Database,
}

impl<'a> SqliteProposalRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    fn row_to_proposal(row: &rusqlite::Row) -> rusqlite::Result<Proposal> {
        let id_str: String = row.get(0)?;
        let wi_str: String = row.get(1)?;
        let at_str: String = row.get(2)?;
        Ok(Proposal {
            id: parse_ulid_col(&id_str, 0)?,
            work_item_id: parse_ulid_col(&wi_str, 1)?,
            analysis_type: analysis_type_from_str(&at_str),
            content: row.get(3)?,
            created_at: row.get(4)?,
        })
    }
}

impl<'a> ProposalRepository for SqliteProposalRepo<'a> {
    fn find_by_work_item(&self, work_item_id: Ulid) -> Result<Vec<Proposal>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id,work_item_id,analysis_type,content,created_at FROM proposals WHERE work_item_id=?1 ORDER BY created_at DESC",
            )?;
            let rows = stmt.query_map([work_item_id.to_string()], Self::row_to_proposal)?;
            rows.map(|r| r.map_err(|e| anyhow::anyhow!(e))).collect()
        }).map_err(anyhow_to_domain)
    }

    fn create(&self, proposal: &Proposal) -> Result<Proposal> {
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO proposals (id,work_item_id,analysis_type,content,created_at) VALUES (?1,?2,?3,?4,?5)",
                rusqlite::params![
                    proposal.id.to_string(),
                    proposal.work_item_id.to_string(),
                    analysis_type_to_str(proposal.analysis_type),
                    proposal.content,
                    proposal.created_at,
                ],
            )?;
            Ok(proposal.clone())
        }).map_err(anyhow_to_domain)
    }

    fn delete(&self, id: Ulid) -> Result<()> {
        self.db.with_conn(|conn| {
            let n = conn.execute("DELETE FROM proposals WHERE id=?1", [id.to_string()])?;
            if n == 0 {
                return Err(anyhow::anyhow!(DomainError::NotFound(id.to_string())));
            }
            Ok(())
        }).map_err(anyhow_to_domain)
    }
}

fn analysis_type_to_str(a: AnalysisType) -> &'static str {
    match a {
        AnalysisType::Summary => "summary",
        AnalysisType::ActionExtraction => "actionExtraction",
        AnalysisType::Classification => "classification",
        AnalysisType::PrioritySuggestion => "prioritySuggestion",
    }
}

fn analysis_type_from_str(s: &str) -> AnalysisType {
    match s {
        "actionExtraction" => AnalysisType::ActionExtraction,
        "classification" => AnalysisType::Classification,
        "prioritySuggestion" => AnalysisType::PrioritySuggestion,
        _ => AnalysisType::Summary,
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
            id: Ulid::new(), title: "parent".into(), content: None,
            status: WorkItemStatus::New, priority: Priority::Normal,
            source: Source::Manual, tags: vec![],
            created_at: 1_000_000, updated_at: 1_000_000,
            parent_id: None, latitude: None, longitude: None, gis_feature_id: None,
        };
        repo.create(&item).unwrap();
        item.id
    }

    #[test]
    fn create_and_find_proposal() {
        let db = setup();
        let wi_id = insert_work_item(&db);
        let repo = SqliteProposalRepo::new(&db);
        let proposal = Proposal {
            id: Ulid::new(), work_item_id: wi_id,
            analysis_type: AnalysisType::Summary,
            content: "summary text".into(), created_at: 1_000_000,
        };
        repo.create(&proposal).unwrap();
        let found = repo.find_by_work_item(wi_id).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].content, "summary text");
    }

    #[test]
    fn delete_proposal() {
        let db = setup();
        let wi_id = insert_work_item(&db);
        let repo = SqliteProposalRepo::new(&db);
        let proposal = Proposal {
            id: Ulid::new(), work_item_id: wi_id,
            analysis_type: AnalysisType::Classification,
            content: "cls".into(), created_at: 1_000_000,
        };
        repo.create(&proposal).unwrap();
        repo.delete(proposal.id).unwrap();
        assert!(repo.find_by_work_item(wi_id).unwrap().is_empty());
    }
}
