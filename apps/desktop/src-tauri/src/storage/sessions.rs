use chrono::Utc;
use rusqlite::params;
use uuid::Uuid;

use super::db::{Database, DbError, DbResult};
use super::models::*;

/// 从查询行映射 SessionSummary（列顺序需与 SESSION_SUMMARY_COLUMNS 一致）
fn session_summary_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SessionSummary> {
    Ok(SessionSummary {
        id: row.get(0)?,
        source_id: row.get(1)?,
        session_id: row.get(2)?,
        source_host: row.get(3)?,
        project_path: row.get(4)?,
        project_name: row.get(5)?,
        message_count: row.get(6)?,
        status: row.get(7)?,
        updated_at: row.get(8)?,
        // SQLite 存储 INTEGER 0/1，需手动转 bool
        has_updates: row.get::<_, i64>(9)? != 0,
        created_at: row.get(10)?,
    })
}

const SESSION_SUMMARY_COLUMNS: &str =
    "id, source_id, session_id, source_host, project_path, project_name, \
     message_count, status, updated_at, has_updates, created_at";

impl Database {
    /// 导入会话。使用 INSERT OR IGNORE 实现去重（唯一键: session_id + source_host）。
    ///
    /// 返回数据库主键 ID（新插入时为新 UUID，冲突时返回已有记录的 ID）。
    pub fn insert_session(
        &self,
        source_id: &str,
        session_id: &str,
        source_host: &str,
        project_path: Option<&str>,
        project_name: Option<&str>,
        message_count: i32,
        content_hash: Option<&str>,
        raw_path: &str,
        created_at: &str,
        updated_at: &str,
    ) -> DbResult<String> {
        let id = Uuid::new_v4().to_string();
        let conn = self.conn();
        let rows = conn.execute(
            "INSERT OR IGNORE INTO sessions (
                id, source_id, session_id, source_host, project_path, project_name,
                message_count, content_hash, raw_path, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                &id, source_id, session_id, source_host, project_path, project_name,
                message_count, content_hash, raw_path, created_at, updated_at,
            ],
        )?;

        if rows == 0 {
            // 唯一约束冲突 → 查询已有记录的 ID 返回
            let existing: String = conn.query_row(
                "SELECT id FROM sessions WHERE session_id = ?1 AND source_host = ?2",
                params![session_id, source_host],
                |row| row.get(0),
            )?;
            log::debug!("会话已存在: session_id={}, db_id={}", session_id, existing);
            Ok(existing)
        } else {
            log::info!(
                "导入会话: source={}, project={:?}, messages={}",
                source_id, project_name, message_count
            );
            Ok(id)
        }
    }

    /// 批量写入消息（事务内执行，保证原子性）
    pub fn insert_messages(
        &self,
        session_db_id: &str,
        messages: &[NewMessage],
    ) -> DbResult<()> {
        let conn = self.conn();
        let tx = conn.unchecked_transaction()?;
        for (seq_order, msg) in messages.iter().enumerate() {
            let id = Uuid::new_v4().to_string();
            tx.execute(
                "INSERT INTO messages (
                    id, session_id, role, content, timestamp, tokens_in, tokens_out, seq_order
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    id, session_db_id, msg.role, msg.content,
                    msg.timestamp, msg.tokens_in, msg.tokens_out, seq_order as i32
                ],
            )?;
        }
        tx.commit()?;
        log::debug!("写入 {} 条消息 → session {}", messages.len(), session_db_id);
        Ok(())
    }

    pub fn get_session(&self, id: &str) -> DbResult<Session> {
        let conn = self.conn();
        conn.query_row(
            "SELECT id, source_id, session_id, source_host, project_path, project_name,
                    message_count, content_hash, raw_path, created_at, updated_at,
                    status, value, has_updates, analyzed_at, error_message
             FROM sessions WHERE id = ?1",
            params![id],
            |row| {
                Ok(Session {
                    id: row.get(0)?,
                    source_id: row.get(1)?,
                    session_id: row.get(2)?,
                    source_host: row.get(3)?,
                    project_path: row.get(4)?,
                    project_name: row.get(5)?,
                    message_count: row.get(6)?,
                    content_hash: row.get(7)?,
                    raw_path: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                    status: row.get(11)?,
                    value: row.get(12)?,
                    has_updates: row.get::<_, i64>(13)? != 0,
                    analyzed_at: row.get(14)?,
                    error_message: row.get(15)?,
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => DbError::NotFound(id.to_string()),
            _ => e.into(),
        })
    }

    pub fn get_session_messages(&self, session_id: &str) -> DbResult<Vec<Message>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, timestamp, tokens_in, tokens_out, seq_order
             FROM messages WHERE session_id = ?1 ORDER BY seq_order ASC",
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(Message {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                timestamp: row.get(4)?,
                tokens_in: row.get(5)?,
                tokens_out: row.get(6)?,
                seq_order: row.get(7)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
    }

    /// 分页查询会话列表，支持按数据源/项目/状态动态筛选。
    ///
    /// 分页参数: page 从 1 开始，page=0 等同于 page=1。
    pub fn list_sessions(
        &self,
        filters: &SessionFilters,
        page: u32,
        page_size: u32,
    ) -> DbResult<PaginatedResult<SessionSummary>> {
        // 动态拼装 WHERE 子句
        let mut conditions = Vec::new();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref source) = filters.source {
            conditions.push("source_id = ?");
            param_values.push(Box::new(source.clone()));
        }
        if let Some(ref project) = filters.project {
            // 同时匹配项目名称和路径，兼容不同填充方式
            conditions.push("(project_name = ? OR project_path = ?)");
            param_values.push(Box::new(project.clone()));
            param_values.push(Box::new(project.clone()));
        }
        if let Some(ref status) = filters.status {
            conditions.push("status = ?");
            param_values.push(Box::new(status.clone()));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };

        let conn = self.conn();

        // 先查总数
        let count_sql = format!("SELECT COUNT(*) FROM sessions{}", where_clause);
        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|b| b.as_ref()).collect();
        let total: i64 = conn
            .query_row(&count_sql, param_refs.as_slice(), |row| row.get(0))?;

        // 再查当页数据
        let offset = page.saturating_sub(1) as i64 * page_size as i64;
        let list_sql = format!(
            "SELECT {} FROM sessions{} ORDER BY created_at DESC LIMIT ? OFFSET ?",
            SESSION_SUMMARY_COLUMNS, where_clause
        );

        let mut data_params = param_values;
        data_params.push(Box::new(page_size as i64));
        data_params.push(Box::new(offset));
        let data_refs: Vec<&dyn rusqlite::types::ToSql> =
            data_params.iter().map(|b| b.as_ref()).collect();

        let mut stmt = conn.prepare(&list_sql)?;
        let items = stmt
            .query_map(data_refs.as_slice(), |row| session_summary_from_row(row))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PaginatedResult {
            items,
            total: total.max(0) as u64,
            page,
            page_size,
        })
    }

    /// 检查是否已存在相同的会话（去重键: session_id + source_host）
    pub fn check_duplicate(
        &self,
        session_id: &str,
        source_host: &str,
    ) -> DbResult<Option<String>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id FROM sessions WHERE session_id = ?1 AND source_host = ?2 LIMIT 1",
        )?;
        let mut rows = stmt.query(params![session_id, source_host])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    /// 更新会话分析状态。status 为 "analyzed" 时自动设置 analyzed_at 为当前时间。
    pub fn update_session_status(
        &self,
        id: &str,
        status: &str,
        value: Option<&str>,
    ) -> DbResult<()> {
        let now = Utc::now().to_rfc3339();
        let conn = self.conn();
        let n = conn.execute(
            "UPDATE sessions SET status = ?1, value = ?2,
                analyzed_at = CASE WHEN ?3 = 'analyzed' THEN ?4 ELSE analyzed_at END
             WHERE id = ?5",
            params![status, value, status, now, id],
        )?;
        if n == 0 {
            return Err(DbError::NotFound(id.to_string()));
        }
        log::info!("会话状态变更: id={}, status={}, value={:?}", id, status, value);
        Ok(())
    }

    /// 标记会话有新消息（增量同步检测到 message_count 变化时调用）
    pub fn mark_has_updates(&self, id: &str) -> DbResult<()> {
        let n = self.conn().execute(
            "UPDATE sessions SET has_updates = 1 WHERE id = ?1",
            params![id],
        )?;
        if n == 0 {
            return Err(DbError::NotFound(id.to_string()));
        }
        Ok(())
    }

    pub fn update_session_message_count(&self, id: &str, count: i32) -> DbResult<()> {
        let n = self.conn().execute(
            "UPDATE sessions SET message_count = ?1 WHERE id = ?2",
            params![count, id],
        )?;
        if n == 0 {
            return Err(DbError::NotFound(id.to_string()));
        }
        Ok(())
    }
}
