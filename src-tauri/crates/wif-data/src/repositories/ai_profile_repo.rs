use ulid::Ulid;
use wif_domain::entities::AiProfile;
use wif_domain::repository::{DomainError, Result, AiProfileRepository};

use crate::connection::Database;
use crate::repositories::util::{anyhow_to_domain, parse_ulid_col};

pub struct SqliteAiProfileRepo<'a> {
    db: &'a Database,
}

impl<'a> SqliteAiProfileRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    fn row_to_profile(row: &rusqlite::Row) -> rusqlite::Result<AiProfile> {
        let id_str: String = row.get(0)?;
        let is_default: i64 = row.get(6)?;
        Ok(AiProfile {
            id: parse_ulid_col(&id_str, 0)?,
            name: row.get(1)?,
            provider: row.get(2)?,
            api_key: row.get(3)?,
            model: row.get(4)?,
            base_url: row.get(5)?,
            is_default: is_default != 0,
            max_tokens: row.get::<_, Option<i64>>(7)?.map(|v| v as u32),
            temperature: row.get(8)?,
        })
    }
}

const SELECT_COLS: &str =
    "SELECT id,name,provider,api_key,model,base_url,is_default,max_tokens,temperature FROM ai_profiles";

impl<'a> AiProfileRepository for SqliteAiProfileRepo<'a> {
    fn find_all(&self) -> Result<Vec<AiProfile>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(&format!("{SELECT_COLS} ORDER BY name"))?;
            let rows = stmt.query_map([], Self::row_to_profile)?;
            rows.map(|r| r.map_err(|e| anyhow::anyhow!(e))).collect()
        }).map_err(anyhow_to_domain)
    }

    fn find_default(&self) -> Result<Option<AiProfile>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(&format!("{SELECT_COLS} WHERE is_default=1 LIMIT 1"))?;
            let mut rows = stmt.query_map([], Self::row_to_profile)?;
            match rows.next() {
                None => Ok(None),
                Some(r) => r.map(Some).map_err(|e| anyhow::anyhow!(e)),
            }
        }).map_err(anyhow_to_domain)
    }

    fn create(&self, profile: &AiProfile) -> Result<AiProfile> {
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO ai_profiles (id,name,provider,api_key,model,base_url,is_default,max_tokens,temperature) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                rusqlite::params![
                    profile.id.to_string(),
                    profile.name, profile.provider, profile.api_key,
                    profile.model, profile.base_url,
                    profile.is_default as i64,
                    profile.max_tokens.map(|v| v as i64),
                    profile.temperature,
                ],
            )?;
            Ok(profile.clone())
        }).map_err(anyhow_to_domain)
    }

    fn update(&self, profile: &AiProfile) -> Result<AiProfile> {
        self.db.with_conn(|conn| {
            let n = conn.execute(
                "UPDATE ai_profiles SET name=?1,provider=?2,api_key=?3,model=?4,base_url=?5,is_default=?6,max_tokens=?7,temperature=?8 WHERE id=?9",
                rusqlite::params![
                    profile.name, profile.provider, profile.api_key,
                    profile.model, profile.base_url,
                    profile.is_default as i64,
                    profile.max_tokens.map(|v| v as i64),
                    profile.temperature,
                    profile.id.to_string(),
                ],
            )?;
            if n == 0 {
                return Err(anyhow::anyhow!(DomainError::NotFound(profile.id.to_string())));
            }
            Ok(profile.clone())
        }).map_err(anyhow_to_domain)
    }

    fn delete(&self, id: Ulid) -> Result<()> {
        self.db.with_conn(|conn| {
            let n = conn.execute("DELETE FROM ai_profiles WHERE id=?1", [id.to_string()])?;
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

    fn sample() -> AiProfile {
        AiProfile {
            id: Ulid::new(), name: "GPT-4".into(), provider: "openai".into(),
            api_key: "sk-xxx".into(), model: "gpt-4o".into(),
            base_url: "https://api.openai.com/v1".into(),
            is_default: true, max_tokens: Some(4096), temperature: Some(0.7),
        }
    }

    #[test]
    fn create_and_find_default() {
        let db = setup();
        let repo = SqliteAiProfileRepo::new(&db);
        let p = sample();
        repo.create(&p).unwrap();
        let def = repo.find_default().unwrap().unwrap();
        assert_eq!(def.model, "gpt-4o");
        assert!(def.is_default);
    }

    #[test]
    fn update_profile() {
        let db = setup();
        let repo = SqliteAiProfileRepo::new(&db);
        let mut p = sample();
        repo.create(&p).unwrap();
        p.model = "gpt-4o-mini".into();
        repo.update(&p).unwrap();
        let all = repo.find_all().unwrap();
        assert_eq!(all[0].model, "gpt-4o-mini");
    }

    #[test]
    fn delete_profile() {
        let db = setup();
        let repo = SqliteAiProfileRepo::new(&db);
        let p = sample();
        repo.create(&p).unwrap();
        repo.delete(p.id).unwrap();
        assert!(repo.find_all().unwrap().is_empty());
    }
}
