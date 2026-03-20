use super::cards::{card_filter_where, card_summary_from_row};
use super::db::{Database, DbResult};
use super::models::*;

impl Database {
    pub fn search_cards(&self, query: &str, filters: &CardFilters) -> DbResult<Vec<CardSummary>> {
        let q = query.trim();
        if q.is_empty() {
            return Ok(Vec::new());
        }

        let (where_sql, filter_params) = card_filter_where(filters);
        let where_clause = if where_sql.is_empty() {
            " WHERE fts MATCH ? ".to_string()
        } else {
            let rest = where_sql
                .strip_prefix(" WHERE ")
                .unwrap_or(where_sql.as_str());
            format!(" WHERE fts MATCH ? AND ({})", rest)
        };

        let sql = format!(
            "SELECT c.id, c.session_id, c.title, c.\"type\", c.value, c.summary, \
             c.category_id, c.source_name, c.project_name, c.created_at, c.updated_at \
             FROM cards c \
             JOIN cards_fts fts ON c.rowid = fts.rowid \
             {} \
             ORDER BY bm25(cards_fts) \
             LIMIT 50",
            where_clause
        );

        let mut params_vec: Vec<String> = Vec::with_capacity(1 + filter_params.len());
        params_vec.push(q.to_string());
        params_vec.extend(filter_params);

        let conn = self.conn();
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params_vec.iter()), |r| {
            card_summary_from_row(r)
        })?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        log::debug!("Search query={:?} returned {} results", q, out.len());
        Ok(out)
    }
}
