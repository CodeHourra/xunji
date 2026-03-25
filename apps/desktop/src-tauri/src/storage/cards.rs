use std::collections::HashMap;

use chrono::Utc;
use rusqlite::{params, OptionalExtension, Row};
use uuid::Uuid;

use super::db::{Database, DbError, DbResult};
use super::models::*;

// ─────────────────────────── 共享列定义与行映射 ────────────────────────
//
// cards.rs 和 search.rs 共用这些函数，避免列顺序不一致导致的映射错误。

pub(super) const CARD_SUMMARY_COLUMNS: &str =
    r#"c.id, c.session_id, c.title, c."type", c.value, c.summary, c.category_id, c.source_name, c.project_name, c.created_at, c.updated_at"#;

/// 从查询行映射 CardSummary（列顺序需与 CARD_SUMMARY_COLUMNS 一致）
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
    // tech_stack 以逗号分隔字符串存储，读出后拆分还原为 Vec
    let tech_stack_str: Option<String> = row.get(18)?;
    let tech_stack = tech_stack_str
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

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
        tech_stack,
        // 19–20：与 `get_card` 中 JOIN sessions 列顺序一致
        source_session_external_id: row.get(19)?,
        source_session_path: row.get(20)?,
    })
}

// ─────────────────────────── 动态筛选条件构建 ─────────────────────────
//
// 被 list_cards 和 search_cards 共用。
//
// 标签、技术栈筛选均为 AND 语义（须同时满足）：
// - 标签：card_tags 子查询 HAVING COUNT(DISTINCT name) = N
// - 技术栈：cards.tech_stack 为逗号串，对每项用 instr 边界匹配（大小写不敏感）

/// 根据 CardFilters 构建 WHERE 子句和参数列表
pub(super) fn build_card_where(filters: &CardFilters) -> (String, Vec<String>) {
    let mut conds: Vec<String> = Vec::new();
    let mut params: Vec<String> = Vec::new();

    if let Some(ref t) = filters.card_type {
        conds.push(r#"c."type" = ?"#.to_string());
        params.push(t.clone());
    }
    if let Some(ref v) = filters.value {
        conds.push("c.value = ?".to_string());
        params.push(v.clone());
    }
    if let Some(ref tags) = filters.tags {
        if !tags.is_empty() {
            let unique: Vec<&String> = {
                let mut seen = std::collections::HashSet::new();
                tags.iter().filter(|t| seen.insert(*t)).collect()
            };
            let n = unique.len();
            let placeholders = vec!["?"; n].join(",");
            conds.push(format!(
                "c.id IN (\
                    SELECT ct.card_id FROM card_tags ct \
                    INNER JOIN tags t ON ct.tag_id = t.id \
                    WHERE t.name IN ({placeholders}) \
                    GROUP BY ct.card_id HAVING COUNT(DISTINCT t.name) = {n}\
                )"
            ));
            params.extend(unique.into_iter().cloned());
        }
    }

    // 技术栈存为逗号分隔字符串；用边界匹配避免子串误命中（如 go / golang）
    if let Some(ref stacks) = filters.tech_stack {
        let unique: Vec<&String> = {
            let mut seen = std::collections::HashSet::new();
            stacks
                .iter()
                .filter(|s| !s.trim().is_empty())
                .filter(|s| seen.insert(*s))
                .collect()
        };
        for s in unique {
            conds.push(
                "instr(',' || lower(ifnull(c.tech_stack, '')) || ',', ',' || lower(?) || ',') > 0"
                    .to_string(),
            );
            params.push(s.clone());
        }
    }

    let where_sql = if conds.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conds.join(" AND "))
    };
    (where_sql, params)
}

// ─────────────────────────── Database 方法 ────────────────────────────

impl Database {
    /// 创建知识卡片。在单个事务内完成：
    ///   1. 写入 cards 表
    ///   2. 创建/关联标签（tags + card_tags）
    ///   3. 同步 FTS5 全文索引
    pub fn insert_card(&self, card: &NewCard<'_>) -> DbResult<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let tags_joined = card.tags.join(",");
        // tech_stack 同样逗号拼接存储
        let tech_stack_joined = card.tech_stack.join(",");

        let mut conn = self.conn();
        let tx = conn.transaction()?;

        //  19 列 = 15 个 ? 占位符 + 4 个 NULL（category_id, memory, skill, feedback）
        //
        //  id  session_id  title  type  value  summary  note
        //  ?   ?           ?      ?     ?      ?        ?
        //
        //  category_id  memory  skill  source_name  project_name
        //  NULL         NULL    NULL   ?            ?
        //
        //  prompt_tokens  completion_tokens  cost_yuan  feedback  tech_stack  created_at  updated_at
        //  ?              ?                  ?          NULL      ?           ?           ?
        tx.execute(
            "INSERT INTO cards (
                id, session_id, title, type, value, summary, note,
                category_id, memory, skill, source_name, project_name,
                prompt_tokens, completion_tokens, cost_yuan, feedback,
                tech_stack, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, NULL, NULL, NULL, ?, ?, ?, ?, ?, NULL, ?, ?, ?)",
            params![
                &id, card.session_id, card.title, card.card_type, card.value,
                card.summary, card.note, card.source_name, card.project_name,
                card.prompt_tokens, card.completion_tokens, card.cost_yuan,
                &tech_stack_joined, &now, &now,
            ],
        )?;

        // 标签处理: INSERT OR IGNORE 保证幂等 → 查回 id → 关联
        for tag_name in card.tags {
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

        // 同步 FTS5 全文索引（独立表，手动写入）
        // 取出刚插入行的 rowid，使 FTS 行与 cards 行对应
        let rowid: i64 = tx.query_row(
            "SELECT rowid FROM cards WHERE id = ?",
            params![&id],
            |row| row.get(0),
        )?;
        tx.execute(
            "INSERT INTO cards_fts(rowid, title, summary, note, tags) VALUES (?, ?, ?, ?, ?)",
            params![
                rowid,
                card.title,
                card.summary.unwrap_or_default(),
                card.note,
                &tags_joined,
            ],
        )?;

        tx.commit()?;
        log::info!(
            "创建卡片: id={}, title={:?}, value={:?}, tags=[{}], tech_stack列='{}' ({}段)",
            id,
            card.title,
            card.value,
            tags_joined,
            tech_stack_joined,
            card.tech_stack.len()
        );
        Ok(id)
    }

    /// 获取卡片完整信息（含关联标签）
    pub fn get_card(&self, id: &str) -> DbResult<Card> {
        let conn = self.conn();
        let mut card = conn
            .query_row(
                "SELECT c.id, c.session_id, c.title, c.type, c.value, c.summary, c.note, \
                 c.category_id, c.memory, c.skill, c.source_name, c.project_name, \
                 c.prompt_tokens, c.completion_tokens, c.cost_yuan, c.feedback, \
                 c.created_at, c.updated_at, c.tech_stack, \
                 sess.session_id, COALESCE(sess.raw_path, sess.project_path) \
                 FROM cards c \
                 INNER JOIN sessions sess ON c.session_id = sess.id \
                 WHERE c.id = ?",
                params![id],
                |r| card_from_row(r),
            )
            .optional()?
            .ok_or_else(|| DbError::NotFound(format!("card {}", id)))?;

        // 关联查询标签名
        let mut stmt = conn.prepare(
            "SELECT t.name FROM tags t \
             INNER JOIN card_tags ct ON t.id = ct.tag_id \
             WHERE ct.card_id = ? ORDER BY t.name",
        )?;
        card.tags = stmt
            .query_map(params![id], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(card)
    }

    /// 分页查询知识卡片列表，支持按类型/价值/标签筛选
    pub fn list_cards(
        &self,
        filters: &CardFilters,
        page: u32,
        page_size: u32,
    ) -> DbResult<PaginatedResult<CardSummary>> {
        let (where_sql, filter_params) = build_card_where(filters);
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
            "SELECT {} FROM cards c{} ORDER BY c.created_at DESC LIMIT {} OFFSET {}",
            CARD_SUMMARY_COLUMNS, where_sql, limit, offset
        );
        let mut stmt = conn.prepare(&select_sql)?;
        let items = stmt
            .query_map(rusqlite::params_from_iter(filter_params.iter()), |r| {
                card_summary_from_row(r)
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PaginatedResult {
            items,
            total: total as u64,
            page,
            page_size,
        })
    }

    /// 库内卡片总数（不受筛选；供「导出全部」前提示条数）
    pub fn count_all_cards(&self) -> DbResult<u64> {
        let n: i64 = self
            .conn()
            .query_row("SELECT COUNT(*) FROM cards", [], |r| r.get(0))?;
        Ok(n.max(0) as u64)
    }

    /// 全部卡片 id（按创建时间升序，批量导出顺序稳定）
    pub fn list_all_card_ids(&self) -> DbResult<Vec<String>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT id FROM cards ORDER BY created_at ASC")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
    }

    /// 删除卡片（级联清理关联表和 FTS 索引，不存在时返回 NotFound）
    /// 删除某会话关联的全部卡片（含 card_tags 与 FTS 行）。
    ///
    /// 用于重新分析前清理旧笔记，避免同一 session 下多张卡片。
    /// 注意：在单事务内完成，避免与 `delete_card` 嵌套加锁导致死锁。
    pub fn delete_cards_for_session(&self, session_db_id: &str) -> DbResult<u64> {
        let mut conn = self.conn();
        let tx = conn.transaction()?;
        let mut stmt = tx.prepare("SELECT id FROM cards WHERE session_id = ?")?;
        let ids: Vec<String> = stmt
            .query_map(params![session_db_id], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        drop(stmt);

        for id in &ids {
            tx.execute("DELETE FROM card_tags WHERE card_id = ?", params![id])?;
            tx.execute(
                "DELETE FROM cards_fts WHERE rowid = (SELECT rowid FROM cards WHERE id = ?)",
                params![id],
            )?;
            tx.execute("DELETE FROM cards WHERE id = ?", params![id])?;
        }
        tx.commit()?;
        if !ids.is_empty() {
            log::info!(
                "已删除会话 {} 下的 {} 张旧卡片",
                session_db_id,
                ids.len()
            );
        }
        Ok(ids.len() as u64)
    }

    pub fn delete_card(&self, id: &str) -> DbResult<()> {
        let mut conn = self.conn();
        let tx = conn.transaction()?;

        tx.execute("DELETE FROM card_tags WHERE card_id = ?", params![id])?;
        tx.execute(
            "DELETE FROM cards_fts WHERE rowid = (SELECT rowid FROM cards WHERE id = ?)",
            params![id],
        )?;
        let n = tx.execute("DELETE FROM cards WHERE id = ?", params![id])?;
        if n == 0 {
            // 事务会在 drop 时自动回滚
            return Err(DbError::NotFound(format!("card {}", id)));
        }
        tx.commit()?;
        log::info!("删除卡片: id={}", id);
        Ok(())
    }

    pub fn update_card_feedback(&self, id: &str, feedback: &str) -> DbResult<()> {
        let n = self.conn().execute(
            "UPDATE cards SET feedback = ? WHERE id = ?",
            params![feedback, id],
        )?;
        if n == 0 {
            return Err(DbError::NotFound(format!("card {}", id)));
        }
        log::debug!("卡片反馈更新: id={}, feedback={}", id, feedback);
        Ok(())
    }

    /// 查询所有标签及其关联的卡片数量（按数量降序），用于知识库侧栏标签筛选。
    pub fn list_all_tags(&self) -> DbResult<Vec<TagCount>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT t.name, COUNT(ct.card_id) as cnt
             FROM tags t
             LEFT JOIN card_tags ct ON t.id = ct.tag_id
             GROUP BY t.id, t.name
             HAVING cnt > 0
             ORDER BY cnt DESC, t.name ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(TagCount {
                name: row.get(0)?,
                count: row.get(1)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
    }

    /// 从 `cards.tech_stack` 逗号分隔列聚合技术栈及卡片命中次数（知识库侧栏「技术栈」区块）。
    /// 展示名取**首次出现**的写法；统计时按小写合并（Rust/rust 视为同一项）。
    pub fn list_all_tech_stack_counts(&self) -> DbResult<Vec<TagCount>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT tech_stack FROM cards WHERE tech_stack IS NOT NULL AND TRIM(tech_stack) != ''",
        )?;
        let mut counts: HashMap<String, (String, i64)> = HashMap::new();
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        for row in rows {
            let s = row?;
            for part in s.split(',') {
                let t = part.trim();
                if t.is_empty() {
                    continue;
                }
                let key = t.to_lowercase();
                let e = counts
                    .entry(key)
                    .or_insert_with(|| (t.to_string(), 0));
                e.1 += 1;
            }
        }
        let mut v: Vec<TagCount> = counts
            .into_values()
            .map(|(name, count)| TagCount { name, count })
            .collect();
        v.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));
        log::debug!(
            "list_all_tech_stack_counts: {} 个不同技术栈条目",
            v.len()
        );
        Ok(v)
    }

    /// 按知识类型统计卡片数量（按数量降序），用于知识库侧栏类型筛选。
    pub fn list_card_type_counts(&self) -> DbResult<Vec<TypeCount>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            r#"SELECT "type", COUNT(*) as cnt
               FROM cards
               WHERE "type" IS NOT NULL AND "type" != ''
               GROUP BY "type"
               ORDER BY cnt DESC, "type" ASC"#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(TypeCount {
                name: row.get(0)?,
                count: row.get(1)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
    }
}
