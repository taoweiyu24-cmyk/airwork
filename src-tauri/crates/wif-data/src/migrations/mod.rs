pub mod v001_initial;

/// Run all migrations in order. Each migration is idempotent.
pub fn run_all(conn: &rusqlite::Connection) -> anyhow::Result<()> {
    v001_initial::apply(conn)?;
    Ok(())
}
