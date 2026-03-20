use chrono::Utc;
use rusqlite::{params, OptionalExtension, Row};
use uuid::Uuid;

use super::db::{Database, DbError, DbResult};
use super::models::*;

pub(super) fn card_filter_where(filters: &CardFilters) -> (String, Vec<String>) {
    let mut conds: Vec<String> = Vec::new();
    let mut p: Vec<String> = Vec::new();

    if let Some(ref t) = filters.card_type {
        conds.push(r#"c."type" = ?"#.to_string());
        p.push(t.clone());
    }
    if let Some(ref v) = filters.value {
        conds.push("c.value = ?".to_string());
        p.push(v.clone());
    }
    if let Some(ref tags) = filters.tags {
        if !tags.is_empty() {
            let mut seen = std::collections::HashSet::new();
            let unique: Vec<String> = tags
                .iter()
                .filter(|t| seen.insert((*t).clone()))
                .cloned()
                .collect();
            let n = unique.len();
            let ph = (0..n).map(|_| "?").collect::<Vec<_>>().join(",");
            conds.push(format!(
                "c.id IN (SELECT ct.card_id FROM card_tags ct \
                 INNER JOIN tags t ON ct.tag_id = t.id \
                 WHERE t.name IN ({}) \
                 GROUP BY ct.card_id HAVING COUNT(DISTINCT t.name) = {})",
                ph, n
            ));
            p.extend(unique);
        }
    }

    let where_sql = if conds.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conds.join(" AND "))
    };
    (where_sql, p)
}

pub(super) fn card_summary_from_row(row: &Row<'_>) -> rusqlite::Result<CardSummary> {
    Ok(CardSummary {
        id: row.get(0)?,
        session_id: row.get(1)?,
        title: row.get(2)?,
        card_type: row.get(3)?,
        value: row.get(4)?,
        summary: row.get(5)?,
        category_id: row.get(6)?,
        source_name: row.get(7)?,
        project_name: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

fn card_from_row(row: &Row<'_>) -> rusqlite::Result<Card> {
    Ok(Card {
        id: row.get(0)?,
        session_id: row.get(1)?,
        title: row.get(2)?,
        card_type: row.get(3)?,
        value: row.get(4)?,
        summary: row.get(5)?,
        note: row.get(6)?,
        category_id: row.get(7)?,
        memory: row.get(8)?,
        skill: row.get(9)?,
        source_name: row.get(10)?,
        project_name: row.get(11)?,
        prompt_tokens: row.get(12)?,
        completion_tokens: row.get(13)?,
        cost_yuan: row.get(14)?,
        feedback: row.get(15)?,
        created_at: row.get(16)?,
        updated_at: row.get(17)?,
        tags: Vec::new(),
    })
}

impl Database {
    pub fn insert_card(
        &self,
        session_id: &str,
        title: &str,
        card_type: Option<&str>,
        value: Option<&str>,
        summary: Option<&str>,
        note: &str,
        source_name: Option<&str>,
        project_name: Option<&str>,
        prompt_tokens: i32,
        completion_tokens: i32,
        cost_yuan: f64,
        tags: &[String],
    ) -> DbResult<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let tags_joined = tags.join(",");

        let mut conn = self.conn();
        let tx = conn.transaction()?;

        tx.execute(
            "INSERT INTO cards (
                id, session_id, title, type, value, summary, note,
                category_id, memory, skill, source_name, project_name,
                prompt_tokens, completion_tokens, cost_yuan, feedback,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, NULL, NULL, NULL, ?, ?, ?, ?, NULL, ?, ?)",
            params![
                &id,
                session_id,
                title,
                card_type,
                value,
                summary,
                note,
                source_name,
                project_name,
                prompt_tokens,
                completion_tokens,
                cost_yuan,
                &now,
                &now,
            ],
        )?;

        for tag_name in tags {
            let tag_id = Uuid::new_v4().to_string();
            tx.execute(
                "INSERT OR IGNORE INTO tags (id, name, type) VALUES (?, ?, 'auto')",
                params![&tag_id, tag_name.as_str()],
            )?;
            let resolved_id: String = tx.query_row(
                "SELECT id FROM tags WHERE name = ?",
                params![tag_name.as_str()],
                |row| row.get(0),
            )?;
            tx.execute(
                "INSERT OR IGNORE INTO card_tags (card_id, tag_id) VALUES (?, ?)",
                params![&id, &resolved_id],
            )?;
        }

        tx.execute(
            "INSERT INTO cards_fts(rowid, title, summary, note, tags) \
             SELECT rowid, title, summary, note, ? FROM cards WHERE id = ?",
            params![&tags_joined, &id],
        )?;

        tx.commit()?;
        log::info!("Created card: id={}, title={:?}, tags={}", id, title, tags_joined);
        Ok(id)
    }

    pub fn get_card(&self, id: &str) -> DbResult<Card> {
        let conn = self.conn();
        let mut card = conn
            .query_row(
                "SELECT id, session_id, title, type, value, summary, note, \
                 category_id, memory, skill, source_name, project_name, \
                 prompt_tokens, completion_tokens, cost_yuan, feedback, \
                 created_at, updated_at \
                 FROM cards WHERE id = ?",
                params![id],
                |r| card_from_row(r),
            )
            .optional()?
            .ok_or_else(|| DbError::NotFound(format!("card {}", id)))?;

        let mut stmt = conn.prepare(
            "SELECT t.name FROM tags t \
             INNER JOIN card_tags ct ON t.id = ct.tag_id \
             WHERE ct.card_id = ? ORDER BY t.name",
        )?;
        let tag_iter = stmt.query_map(params![id], |row| row.get::<_, String>(0))?;
        card.tags = tag_iter.filter_map(|r| r.ok()).collect();
        Ok(card)
    }

    pub fn list_cards(
        &self,
        filters: &CardFilters,
        page: u32,
        page_size: u32,
    ) -> DbResult<PaginatedResult<CardSummary>> {
        let (where_sql, filter_params) = card_filter_where(filters);
        let page = page.max(1);
        let limit = page_size as i64;
        let offset = (page - 1) as i64 * limit;

        let conn = self.conn();

        let count_sql = format!("SELECT COUNT(*) FROM cards c{}", where_sql);
        let total: i64 = conn.query_row(
            &count_sql,
            rusqlite::params_from_iter(filter_params.iter()),
            |r| r.get(0),
        )?;

        let select_sql = format!(
            "SELECT c.id, c.session_id, c.title, c.\"type\", c.value, c.summary, \
             c.category_id, c.source_name, c.project_name, c.created_at, c.updated_at \
             FROM cards c{} \
             ORDER BY c.created_at DESC LIMIT {} OFFSET {}",
            where_sql, limit, offset
        );

        let mut stmt = conn.prepare(&select_sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(filter_params.iter()), |r| {
            card_summary_from_row(r)
        })?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }

        Ok(PaginatedResult {
            items,
            total: total as u64,
            page,
            page_size,
        })
    }

    pub fn delete_card(&self, id: &str) -> DbResult<()> {
        let mut conn = self.conn();
        let tx = conn.transaction()?;

        tx.execute(
            "DELETE FROM card_tags WHERE card_id = ?",
            params![id],
        )?;
        tx.execute(
            "DELETE FROM cards_fts WHERE rowid = (SELECT rowid FROM cards WHERE id = ?)",
            params![id],
        )?;
        tx.execute("DELETE FROM cards WHERE id = ?", params![id])?;
        tx.commit()?;
        log::info!("Deleted card: id={}", id);
        Ok(())
    }

    pub fn update_card_feedback(&self, id: &str, feedback: &str) -> DbResult<()> {
        let conn = self.conn();
        let n = conn.execute(
            "UPDATE cards SET feedback = ? WHERE id = ?",
            params![feedback, id],
        )?;
        if n == 0 {
            return Err(DbError::NotFound(format!("card {}", id)));
        }
        Ok(())
    }
}
