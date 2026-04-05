use ulid::Ulid;
use wif_domain::entities::MailMessage;
use wif_domain::repository::{DomainError, Result, MailMessageRepository};

use crate::connection::Database;
use crate::repositories::util::{anyhow_to_domain, parse_ulid_col, to_json_array};

pub struct SqliteMailMessageRepo<'a> {
    db: &'a Database,
}

impl<'a> SqliteMailMessageRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    fn row_to_message(row: &rusqlite::Row) -> rusqlite::Result<MailMessage> {
        let id_str: String = row.get(0)?;
        let acc_str: String = row.get(1)?;
        let to_str: String = row.get(5)?;
        let is_read: i64 = row.get(9)?;
        let wi_str: Option<String> = row.get(10)?;

        let work_item_id = wi_str.as_deref().map(|s| parse_ulid_col(s, 10)).transpose()?;
        let to_addresses = serde_json::from_str::<Vec<String>>(&to_str).unwrap_or_default();

        Ok(MailMessage {
            id: parse_ulid_col(&id_str, 0)?,
            account_id: parse_ulid_col(&acc_str, 1)?,
            message_id: row.get(2)?,
            subject: row.get(3)?,
            from_address: row.get(4)?,
            to_addresses,
            body_text: row.get(6)?,
            body_html: row.get(7)?,
            received_at: row.get(8)?,
            is_read: is_read != 0,
            work_item_id,
        })
    }
}

const SELECT_COLS: &str =
    "SELECT id,account_id,message_id,subject,from_address,to_addresses,body_text,body_html,received_at,is_read,work_item_id FROM mail_messages";

impl<'a> MailMessageRepository for SqliteMailMessageRepo<'a> {
    fn find_by_account(&self, account_id: Ulid) -> Result<Vec<MailMessage>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(&format!("{SELECT_COLS} WHERE account_id=?1 ORDER BY received_at DESC"))?;
            let rows = stmt.query_map([account_id.to_string()], Self::row_to_message)?;
            rows.map(|r| r.map_err(|e| anyhow::anyhow!(e))).collect()
        }).map_err(anyhow_to_domain)
    }

    fn find_by_id(&self, id: Ulid) -> Result<Option<MailMessage>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(&format!("{SELECT_COLS} WHERE id=?1"))?;
            let mut rows = stmt.query_map([id.to_string()], Self::row_to_message)?;
            match rows.next() {
                None => Ok(None),
                Some(r) => r.map(Some).map_err(|e| anyhow::anyhow!(e)),
            }
        }).map_err(anyhow_to_domain)
    }

    fn create(&self, msg: &MailMessage) -> Result<MailMessage> {
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO mail_messages (id,account_id,message_id,subject,from_address,to_addresses,body_text,body_html,received_at,is_read,work_item_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                rusqlite::params![
                    msg.id.to_string(),
                    msg.account_id.to_string(),
                    msg.message_id, msg.subject, msg.from_address,
                    to_json_array(&msg.to_addresses),
                    msg.body_text, msg.body_html, msg.received_at,
                    msg.is_read as i64,
                    msg.work_item_id.map(|u| u.to_string()),
                ],
            )?;
            Ok(msg.clone())
        }).map_err(anyhow_to_domain)
    }

    fn mark_read(&self, id: Ulid) -> Result<()> {
        self.db.with_conn(|conn| {
            let n = conn.execute("UPDATE mail_messages SET is_read=1 WHERE id=?1", [id.to_string()])?;
            if n == 0 {
                return Err(anyhow::anyhow!(DomainError::NotFound(id.to_string())));
            }
            Ok(())
        }).map_err(anyhow_to_domain)
    }

    fn link_to_work_item(&self, id: Ulid, work_item_id: Ulid) -> Result<()> {
        self.db.with_conn(|conn| {
            let n = conn.execute(
                "UPDATE mail_messages SET work_item_id=?1 WHERE id=?2",
                rusqlite::params![work_item_id.to_string(), id.to_string()],
            )?;
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
    use crate::repositories::mail_account_repo::SqliteMailAccountRepo;
    use wif_domain::entities::MailAccount;
    use wif_domain::repository::MailAccountRepository;

    fn setup() -> Database {
        let db = Database::open_in_memory().unwrap();
        db.run_migrations().unwrap();
        db
    }

    fn insert_account(db: &Database) -> Ulid {
        let repo = SqliteMailAccountRepo::new(db);
        let acct = MailAccount {
            id: Ulid::new(), name: "acc".into(), email: "a@b.com".into(), provider: "x".into(),
            imap_host: "imap".into(), imap_port: 993,
            smtp_host: "smtp".into(), smtp_port: 465,
            use_oauth: false, access_token: None, refresh_token: None, is_active: true,
        };
        repo.create(&acct).unwrap();
        acct.id
    }

    fn sample_msg(account_id: Ulid) -> MailMessage {
        MailMessage {
            id: Ulid::new(), account_id,
            message_id: "<abc@mail>".into(), subject: "Hello".into(),
            from_address: "sender@x.com".into(), to_addresses: vec!["me@x.com".into()],
            body_text: Some("body".into()), body_html: None,
            received_at: 1_000_000, is_read: false, work_item_id: None,
        }
    }

    #[test]
    fn create_and_find_message() {
        let db = setup();
        let acc_id = insert_account(&db);
        let repo = SqliteMailMessageRepo::new(&db);
        let msg = sample_msg(acc_id);
        repo.create(&msg).unwrap();
        let found = repo.find_by_id(msg.id).unwrap().unwrap();
        assert_eq!(found.subject, "Hello");
        assert!(!found.is_read);
    }

    #[test]
    fn mark_read_works() {
        let db = setup();
        let acc_id = insert_account(&db);
        let repo = SqliteMailMessageRepo::new(&db);
        let msg = sample_msg(acc_id);
        repo.create(&msg).unwrap();
        repo.mark_read(msg.id).unwrap();
        let found = repo.find_by_id(msg.id).unwrap().unwrap();
        assert!(found.is_read);
    }
}
