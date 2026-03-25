# 桌面应用：FTS5 `MATCH` 必须用虚拟表名

### 现象

- 知识库全文搜索在输入部分关键词（如 `SQL`）时报错：`SQLite error: no such column: fts`，SQL 片段中含 `WHERE fts MATCH ?`。

### 根因

- SQLite FTS5 要求 **`MATCH` 左侧为 FTS5 虚拟表本身的名字**（此处为 `cards_fts`）。
- 若写成 `JOIN cards_fts fts ... WHERE fts MATCH ?`，解析器会把 **`fts` 当成普通列名**，从而报错 `no such column: fts`（与具体搜索词无关，一旦执行到该语句即失败）。

### 解决方法

- `WHERE` 子句使用 **`cards_fts MATCH ?`**；`JOIN` 仍可保留别名 `fts` 仅用于 `ON c.rowid = fts.rowid` 等。
- 实现位置：`apps/desktop/src-tauri/src/storage/search.rs` 中 `search_cards`。

### 相关位置

- `apps/desktop/src-tauri/src/storage/search.rs`
