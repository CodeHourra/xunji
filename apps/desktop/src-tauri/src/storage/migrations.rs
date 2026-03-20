use rusqlite::Connection;

use super::db::DbResult;

pub fn run(conn: &Connection) -> DbResult<()> {
    let version: u32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

    if version < 1 {
        migrate_v1(conn)?;
    }

    Ok(())
}

fn migrate_v1(conn: &Connection) -> DbResult<()> {
    log::info!("Running database migration v1...");

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

        -- FTS5 full-text search index
        CREATE VIRTUAL TABLE IF NOT EXISTS cards_fts USING fts5(
            title, summary, note, tags,
            content='cards',
            content_rowid='rowid',
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

    log::info!("Database migration v1 complete.");
    Ok(())
}
