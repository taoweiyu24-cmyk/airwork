//! FTS5 full-text keyword search via the `work_items_fts` virtual table.

use wif_data::Database;

use crate::error::SearchError;
use crate::types::{SearchQuery, SearchResult, SearchSource};

/// Performs full-text keyword search using the SQLite FTS5 virtual table.
pub struct KeywordSearchService<'a> {
    db: &'a Database,
}

impl<'a> KeywordSearchService<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Search the FTS5 index and return ranked results.
    ///
    /// Uses the built-in `rank` column for ordering and `snippet()` for
    /// highlighted excerpts.
    pub fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
        if query.text.trim().is_empty() {
            return Err(SearchError::QueryError("Search text must not be empty".into()));
        }

        self.db
            .with_conn(|conn| {
                let sql = "\
                    SELECT wi.id,
                           wi.title,
                           snippet(work_items_fts, 1, '<b>', '</b>', '...', 32) AS snippet,
                           work_items_fts.rank
                    FROM   work_items_fts
                    JOIN   work_items wi ON wi.rowid = work_items_fts.rowid
                    WHERE  work_items_fts MATCH ?1
                    ORDER  BY rank
                    LIMIT  ?2 OFFSET ?3";

                let mut stmt = conn.prepare(sql)?;
                let rows = stmt.query_map(
                    rusqlite::params![&query.text, query.limit as i64, query.offset as i64],
                    |row| {
                        let id: String = row.get(0)?;
                        let title: String = row.get(1)?;
                        let snippet: String = row.get(2)?;
                        let rank: f64 = row.get(3)?;
                        Ok((id, title, snippet, rank))
                    },
                )?;

                let mut results = Vec::new();
                for row in rows {
                    let (id, title, snippet, rank) = row.map_err(|e| anyhow::anyhow!(e))?;
                    // FTS5 rank values are negative; convert to a positive score in [0, 1].
                    let score = rank_to_score(rank);
                    results.push(SearchResult {
                        id,
                        title,
                        snippet,
                        score,
                        source: SearchSource::Keyword,
                    });
                }
                Ok(results)
            })
            .map_err(|e| SearchError::DatabaseError(e.to_string()))
    }
}

/// Convert an FTS5 rank (a negative BM25 value) to a score in [0, 1].
///
/// FTS5 rank is negative; more negative means a better match.  We clamp the
/// raw value into a reasonable range and invert it so that 1.0 is best.
fn rank_to_score(rank: f64) -> f64 {
    // Ranks are typically in the range (-50, 0).  Clamp to [-50, 0] then
    // scale to [0, 1].
    let clamped = rank.max(-50.0).min(0.0);
    1.0 - (clamped / -50.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_to_score_zero_rank_gives_one() {
        assert!((rank_to_score(0.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn rank_to_score_negative_fifty_gives_zero() {
        assert!((rank_to_score(-50.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn rank_to_score_clamped_below_minus_fifty() {
        // Values more negative than -50 should clamp to 0.
        assert!((rank_to_score(-100.0)).abs() < f64::EPSILON);
    }
}
