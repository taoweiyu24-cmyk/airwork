use ulid::Ulid;
use wif_domain::entities::Contact;
use wif_domain::repository::{DomainError, Result, ContactRepository};

use crate::connection::Database;
use crate::repositories::util::{anyhow_to_domain, parse_ulid_col};

pub struct SqliteContactRepo<'a> {
    db: &'a Database,
}

impl<'a> SqliteContactRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    fn row_to_contact(row: &rusqlite::Row) -> rusqlite::Result<Contact> {
        let id_str: String = row.get(0)?;
        Ok(Contact {
            id: parse_ulid_col(&id_str, 0)?,
            name: row.get(1)?,
            email: row.get(2)?,
            phone: row.get(3)?,
            organization: row.get(4)?,
            created_at: row.get(5)?,
        })
    }
}

const SELECT_COLS: &str =
    "SELECT id,name,email,phone,organization,created_at FROM contacts";

impl<'a> ContactRepository for SqliteContactRepo<'a> {
    fn find_all(&self) -> Result<Vec<Contact>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(&format!("{SELECT_COLS} ORDER BY name"))?;
            let rows = stmt.query_map([], Self::row_to_contact)?;
            rows.map(|r| r.map_err(|e| anyhow::anyhow!(e))).collect()
        }).map_err(anyhow_to_domain)
    }

    fn find_by_id(&self, id: Ulid) -> Result<Option<Contact>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(&format!("{SELECT_COLS} WHERE id=?1"))?;
            let mut rows = stmt.query_map([id.to_string()], Self::row_to_contact)?;
            match rows.next() {
                None => Ok(None),
                Some(r) => r.map(Some).map_err(|e| anyhow::anyhow!(e)),
            }
        }).map_err(anyhow_to_domain)
    }

    fn find_by_email(&self, email: &str) -> Result<Option<Contact>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(&format!("{SELECT_COLS} WHERE email=?1"))?;
            let mut rows = stmt.query_map([email], Self::row_to_contact)?;
            match rows.next() {
                None => Ok(None),
                Some(r) => r.map(Some).map_err(|e| anyhow::anyhow!(e)),
            }
        }).map_err(anyhow_to_domain)
    }

    fn create(&self, contact: &Contact) -> Result<Contact> {
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO contacts (id,name,email,phone,organization,created_at) VALUES (?1,?2,?3,?4,?5,?6)",
                rusqlite::params![
                    contact.id.to_string(), contact.name, contact.email,
                    contact.phone, contact.organization, contact.created_at,
                ],
            )?;
            Ok(contact.clone())
        }).map_err(anyhow_to_domain)
    }

    fn update(&self, contact: &Contact) -> Result<Contact> {
        self.db.with_conn(|conn| {
            let n = conn.execute(
                "UPDATE contacts SET name=?1,email=?2,phone=?3,organization=?4 WHERE id=?5",
                rusqlite::params![
                    contact.name, contact.email, contact.phone,
                    contact.organization, contact.id.to_string(),
                ],
            )?;
            if n == 0 {
                return Err(anyhow::anyhow!(DomainError::NotFound(contact.id.to_string())));
            }
            Ok(contact.clone())
        }).map_err(anyhow_to_domain)
    }

    fn delete(&self, id: Ulid) -> Result<()> {
        self.db.with_conn(|conn| {
            let n = conn.execute("DELETE FROM contacts WHERE id=?1", [id.to_string()])?;
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

    fn setup() -> Database {
        let db = Database::open_in_memory().unwrap();
        db.run_migrations().unwrap();
        db
    }

    fn sample() -> Contact {
        Contact {
            id: Ulid::new(), name: "Alice".into(),
            email: Some("alice@example.com".into()),
            phone: None, organization: Some("ACME".into()), created_at: 1_000_000,
        }
    }

    #[test]
    fn create_and_find_contact() {
        let db = setup();
        let repo = SqliteContactRepo::new(&db);
        let c = sample();
        repo.create(&c).unwrap();
        let found = repo.find_by_id(c.id).unwrap().unwrap();
        assert_eq!(found.name, "Alice");
    }

    #[test]
    fn find_by_email() {
        let db = setup();
        let repo = SqliteContactRepo::new(&db);
        let c = sample();
        repo.create(&c).unwrap();
        let found = repo.find_by_email("alice@example.com").unwrap().unwrap();
        assert_eq!(found.id, c.id);
    }

    #[test]
    fn update_contact() {
        let db = setup();
        let repo = SqliteContactRepo::new(&db);
        let mut c = sample();
        repo.create(&c).unwrap();
        c.name = "Bob".into();
        repo.update(&c).unwrap();
        let found = repo.find_by_id(c.id).unwrap().unwrap();
        assert_eq!(found.name, "Bob");
    }
}
