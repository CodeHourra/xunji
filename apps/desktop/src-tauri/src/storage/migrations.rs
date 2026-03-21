use rusqlite::Connection;

use super::db::DbResult;

/// 数据库 Schema 迁移入口。
///
/// 通过 PRAGMA user_version 追踪当前版本号，
/// 按版本递增依次执行对应的迁移函数。
///
///   user_version=0 → 执行 migrate_v1 → user_version=1
///   user_version=1 → 执行 migrate_v2 → user_version=2 (未来)
///
pub fn run(conn: &Connection) -> DbResult<()> {
    let version: u32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    log::debug!("当前数据库版本: v{}", version);

    if version < 1 {
        migrate_v1(conn)?;
    }
    if version < 2 {
        migrate_v2(conn)?;
    }
    if version < 3 {
        migrate_v3(conn)?;
    }
    if version < 4 {
        migrate_v4(conn)?;
    }
    if version < 5 {
        migrate_v5(conn)?;
    }

    Ok(())
}

fn migrate_v1(conn: &Connection) -> DbResult<()> {
    log::info!("执行数据库迁移 v1...");

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS sources (
            id          TEXT PRIMARY KEY,
            name        TEXT NOT NULL,
            enabled     INTEGER DEFAULT 1,
            scan_paths  TEXT,
            last_sync   TEXT,
            config      TEXT
        );

        CREATE TABLE IF NOT EXISTS sessions (
            id              TEXT PRIMARY KEY,
            source_id       TEXT NOT NULL,
            session_id      TEXT NOT NULL,
            source_host     TEXT DEFAULT 'local',
            project_path    TEXT,
            project_name    TEXT,
            message_count   INTEGER DEFAULT 0,
            content_hash    TEXT,
            raw_path        TEXT,
            created_at      TEXT NOT NULL,
            updated_at      TEXT NOT NULL,
            status          TEXT DEFAULT 'pending',
            value           TEXT,
            has_updates     INTEGER DEFAULT 0,
            analyzed_at     TEXT,
            error_message   TEXT,
            UNIQUE(session_id, source_host)
        );

        CREATE TABLE IF NOT EXISTS messages (
            id          TEXT PRIMARY KEY,
            session_id  TEXT NOT NULL,
            role        TEXT NOT NULL,
            content     TEXT NOT NULL,
            timestamp   TEXT,
            tokens_in   INTEGER DEFAULT 0,
            tokens_out  INTEGER DEFAULT 0,
            seq_order   INTEGER NOT NULL,
            FOREIGN KEY (session_id) REFERENCES sessions(id)
        );

        CREATE TABLE IF NOT EXISTS cards (
            id                TEXT PRIMARY KEY,
            session_id        TEXT NOT NULL,
            title             TEXT NOT NULL,
            type              TEXT,
            value             TEXT,
            summary           TEXT,
            note              TEXT NOT NULL,
            category_id       TEXT,
            memory            TEXT,
            skill             TEXT,
            source_name       TEXT,
            project_name      TEXT,
            prompt_tokens     INTEGER DEFAULT 0,
            completion_tokens INTEGER DEFAULT 0,
            cost_yuan         REAL DEFAULT 0,
            feedback          TEXT,
            created_at        TEXT NOT NULL,
            updated_at        TEXT NOT NULL,
            FOREIGN KEY (session_id) REFERENCES sessions(id)
        );

        CREATE TABLE IF NOT EXISTS tags (
            id      TEXT PRIMARY KEY,
            name    TEXT NOT NULL UNIQUE,
            type    TEXT DEFAULT 'auto',
            count   INTEGER DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS card_tags (
            card_id TEXT NOT NULL,
            tag_id  TEXT NOT NULL,
            PRIMARY KEY (card_id, tag_id),
            FOREIGN KEY (card_id) REFERENCES cards(id),
            FOREIGN KEY (tag_id)  REFERENCES tags(id)
        );

        CREATE TABLE IF NOT EXISTS categories (
            id          TEXT PRIMARY KEY,
            name        TEXT NOT NULL,
            parent_id   TEXT,
            sort_order  INTEGER DEFAULT 0,
            FOREIGN KEY (parent_id) REFERENCES categories(id)
        );

        CREATE TABLE IF NOT EXISTS sync_log (
            id               TEXT PRIMARY KEY,
            source_id        TEXT NOT NULL,
            started_at       TEXT NOT NULL,
            finished_at      TEXT,
            sessions_found   INTEGER DEFAULT 0,
            sessions_new     INTEGER DEFAULT 0,
            sessions_updated INTEGER DEFAULT 0,
            status           TEXT DEFAULT 'running'
        );

        CREATE TABLE IF NOT EXISTS token_usage (
            id                TEXT PRIMARY KEY,
            card_id           TEXT,
            provider          TEXT NOT NULL,
            model             TEXT NOT NULL,
            prompt_tokens     INTEGER NOT NULL,
            completion_tokens INTEGER NOT NULL,
            cost_yuan         REAL NOT NULL,
            created_at        TEXT NOT NULL,
            FOREIGN KEY (card_id) REFERENCES cards(id)
        );

        -- FTS5 全文搜索索引（独立表，手动管理写入/删除）
        -- 注意：不使用 content='cards'，因为 cards 表没有 tags 列
        CREATE VIRTUAL TABLE IF NOT EXISTS cards_fts USING fts5(
            title, summary, note, tags,
            tokenize='unicode61'
        );

        -- Indexes
        CREATE INDEX IF NOT EXISTS idx_sessions_source   ON sessions(source_id);
        CREATE INDEX IF NOT EXISTS idx_sessions_project  ON sessions(project_name);
        CREATE INDEX IF NOT EXISTS idx_sessions_status   ON sessions(status);
        CREATE INDEX IF NOT EXISTS idx_sessions_created  ON sessions(created_at);
        CREATE INDEX IF NOT EXISTS idx_cards_session     ON cards(session_id);
        CREATE INDEX IF NOT EXISTS idx_cards_type        ON cards(type);
        CREATE INDEX IF NOT EXISTS idx_cards_value       ON cards(value);
        CREATE INDEX IF NOT EXISTS idx_cards_created     ON cards(created_at);
        CREATE INDEX IF NOT EXISTS idx_card_tags_tag     ON card_tags(tag_id);
        CREATE INDEX IF NOT EXISTS idx_categories_parent ON categories(parent_id);

        PRAGMA user_version = 1;
        ",
    )?;

    log::info!("数据库迁移 v1 完成");
    Ok(())
}

/// v2 迁移：修复 cards_fts FTS5 表。
///
/// v1 的 `cards_fts` 使用了 `content='cards'`，导致 SQLite 在读取时
/// 反查 cards 表并报 "no such column: T.tags"（cards 表没有 tags 列）。
///
/// 修复方案：
/// 1. 删除旧的 content-backed FTS5 表
/// 2. 重建为独立 FTS5 表（不绑定 content 表）
/// 3. 从现有卡片 + card_tags 重建索引
fn migrate_v2(conn: &Connection) -> DbResult<()> {
    log::info!("执行数据库迁移 v2：修复 cards_fts 表...");

    conn.execute_batch(
        "
        -- 删除旧的 content-backed FTS5 表（及其影子表）
        DROP TABLE IF EXISTS cards_fts;

        -- 重建为独立 FTS5 表，不使用 content='cards'
        CREATE VIRTUAL TABLE cards_fts USING fts5(
            title, summary, note, tags,
            tokenize='unicode61'
        );

        -- 从现有卡片数据重建全文索引
        -- tags 字段通过 card_tags + tags 关联表拼接为逗号分隔字符串
        INSERT INTO cards_fts(rowid, title, summary, note, tags)
        SELECT
            c.rowid,
            c.title,
            COALESCE(c.summary, ''),
            c.note,
            COALESCE(
                (SELECT GROUP_CONCAT(t.name, ',')
                 FROM card_tags ct
                 JOIN tags t ON ct.tag_id = t.id
                 WHERE ct.card_id = c.id),
                ''
            )
        FROM cards c;

        PRAGMA user_version = 2;
        ",
    )?;

    log::info!("数据库迁移 v2 完成");
    Ok(())
}

/// v3 迁移：为 sessions 表新增 `analysis_note` 列。
///
/// 用于持久化低/无价值会话的判断原因，使页面刷新后仍可在会话列表展示摘要，
/// 而不依赖前端内存中的临时 patchItem。
fn migrate_v3(conn: &Connection) -> DbResult<()> {
    log::info!("执行数据库迁移 v3：为 sessions 表新增 analysis_note 列...");

    conn.execute_batch(
        "
        ALTER TABLE sessions ADD COLUMN analysis_note TEXT;
        PRAGMA user_version = 3;
        ",
    )?;

    log::info!("数据库迁移 v3 完成");
    Ok(())
}

/// v5 迁移：为 cards 表新增 `tech_stack` 列。
///
/// LLM 在 distill_full 时会返回涉及的技术栈列表（如 ["Rust", "SQLite", "Tauri"]），
/// 此前该字段被丢弃（#[allow(dead_code)]），v5 开始持久化为逗号分隔字符串。
fn migrate_v5(conn: &Connection) -> DbResult<()> {
    log::info!("执行数据库迁移 v5：为 cards 表新增 tech_stack 列...");

    conn.execute_batch(
        "
        ALTER TABLE cards ADD COLUMN tech_stack TEXT;
        PRAGMA user_version = 5;
        ",
    )?;

    log::info!("数据库迁移 v5 完成");
    Ok(())
}
///
/// 低/无价值会话无 Card 产出，但 judge_value 仍返回对话类型（type）和原因（reason）。
/// 这两列持久化该信息，使列表在刷新后仍能展示类型徽章和由原因构造的标题，
/// 而无需依赖前端内存中的临时 patchItem。
fn migrate_v4(conn: &Connection) -> DbResult<()> {
    log::info!("执行数据库迁移 v4：为 sessions 表新增 analysis_title / analysis_type 列...");

    conn.execute_batch(
        "
        ALTER TABLE sessions ADD COLUMN analysis_title TEXT;
        ALTER TABLE sessions ADD COLUMN analysis_type  TEXT;
        PRAGMA user_version = 4;
        ",
    )?;

    log::info!("数据库迁移 v4 完成");
    Ok(())
}
