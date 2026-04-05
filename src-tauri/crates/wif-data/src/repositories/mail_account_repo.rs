use ulid::Ulid;
use wif_domain::entities::MailAccount;
use wif_domain::repository::{DomainError, Result, MailAccountRepository};

use crate::connection::Database;
use crate::repositories::util::{anyhow_to_domain, parse_ulid_col};

pub struct SqliteMailAccountRepo<'a> {
    db: &'a Database,
}

impl<'a> SqliteMailAccountRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    fn row_to_account(row: &rusqlite::Row) -> rusqlite::Result<MailAccount> {
        let id_str: String = row.get(0)?;
        let use_oauth: i64 = row.get(8)?;
        let is_active: i64 = row.get(11)?;
        Ok(MailAccount {
            id: parse_ulid_col(&id_str, 0)?,
            name: row.get(1)?,
            email: row.get(2)?,
            provider: row.get(3)?,
            imap_host: row.get(4)?,
            imap_port: row.get::<_, i64>(5)? as u16,
            smtp_host: row.get(6)?,
            smtp_port: row.get::<_, i64>(7)? as u16,
            use_oauth: use_oauth != 0,
            access_token: row.get(9)?,
            refresh_token: row.get(10)?,
            is_active: is_active != 0,
        })
    }
}

const SELECT_COLS: &str =
    "SELECT id,name,email,provider,imap_host,imap_port,smtp_host,smtp_port,use_oauth,access_token,refresh_token,is_active FROM mail_accounts";

impl<'a> MailAccountRepository for SqliteMailAccountRepo<'a> {
    fn find_all(&self) -> Result<Vec<MailAccount>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(&format!("{SELECT_COLS} ORDER BY name"))?;
            let rows = stmt.query_map([], Self::row_to_account)?;
            rows.map(|r| r.map_err(|e| anyhow::anyhow!(e))).collect()
        }).map_err(anyhow_to_domain)
    }

    fn find_by_id(&self, id: Ulid) -> Result<Option<MailAccount>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(&format!("{SELECT_COLS} WHERE id=?1"))?;
            let mut rows = stmt.query_map([id.to_string()], Self::row_to_account)?;
            match rows.next() {
                None => Ok(None),
                Some(r) => r.map(Some).map_err(|e| anyhow::anyhow!(e)),
            }
        }).map_err(anyhow_to_domain)
    }

    fn create(&self, account: &MailAccount) -> Result<MailAccount> {
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO mail_accounts (id,name,email,provider,imap_host,imap_port,smtp_host,smtp_port,use_oauth,access_token,refresh_token,is_active) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
                rusqlite::params![
                    account.id.to_string(),
                    account.name, account.email, account.provider,
                    account.imap_host, account.imap_port as i64,
                    account.smtp_host, account.smtp_port as i64,
                    account.use_oauth as i64,
                    account.access_token, account.refresh_token,
                    account.is_active as i64,
                ],
            )?;
            Ok(account.clone())
        }).map_err(anyhow_to_domain)
    }

    fn update(&self, account: &MailAccount) -> Result<MailAccount> {
        self.db.with_conn(|conn| {
            let n = conn.execute(
                "UPDATE mail_accounts SET name=?1,email=?2,provider=?3,imap_host=?4,imap_port=?5,smtp_host=?6,smtp_port=?7,use_oauth=?8,access_token=?9,refresh_token=?10,is_active=?11 WHERE id=?12",
                rusqlite::params![
                    account.name, account.email, account.provider,
                    account.imap_host, account.imap_port as i64,
                    account.smtp_host, account.smtp_port as i64,
                    account.use_oauth as i64,
                    account.access_token, account.refresh_token,
                    account.is_active as i64,
                    account.id.to_string(),
                ],
            )?;
            if n == 0 {
                return Err(anyhow::anyhow!(DomainError::NotFound(account.id.to_string())));
            }
            Ok(account.clone())
        }).map_err(anyhow_to_domain)
    }

    fn delete(&self, id: Ulid) -> Result<()> {
        self.db.with_conn(|conn| {
            let n = conn.execute("DELETE FROM mail_accounts WHERE id=?1", [id.to_string()])?;
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

    fn sample() -> MailAccount {
        MailAccount {
            id: Ulid::new(), name: "Work".into(), email: "me@example.com".into(),
            provider: "gmail".into(), imap_host: "imap.gmail.com".into(), imap_port: 993,
            smtp_host: "smtp.gmail.com".into(), smtp_port: 465,
            use_oauth: true, access_token: Some("tok".into()), refresh_token: None, is_active: true,
        }
    }

    #[test]
    fn create_and_find_account() {
        let db = setup();
        let repo = SqliteMailAccountRepo::new(&db);
        let acct = sample();
        repo.create(&acct).unwrap();
        let found = repo.find_by_id(acct.id).unwrap().unwrap();
        assert_eq!(found.email, "me@example.com");
        assert!(found.use_oauth);
    }

    #[test]
    fn update_account() {
        let db = setup();
        let repo = SqliteMailAccountRepo::new(&db);
        let mut acct = sample();
        repo.create(&acct).unwrap();
        acct.is_active = false;
        repo.update(&acct).unwrap();
        let found = repo.find_by_id(acct.id).unwrap().unwrap();
        assert!(!found.is_active);
    }

    #[test]
    fn delete_account() {
        let db = setup();
        let repo = SqliteMailAccountRepo::new(&db);
        let acct = sample();
        repo.create(&acct).unwrap();
        repo.delete(acct.id).unwrap();
        assert!(repo.find_by_id(acct.id).unwrap().is_none());
    }
}
