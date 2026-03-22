# 寻迹（XunJi）SQLite 数据模型与表关系

本文档描述桌面端（Tauri）本地库当前 Schema，依据 `apps/desktop/src-tauri/src/storage/migrations.rs` 中的迁移逻辑整理，并用 Mermaid 辅助说明实体关系。

## 1. 概述

| 项目 | 说明 |
|------|------|
| 引擎 | SQLite，连接时开启 `PRAGMA foreign_keys=ON`、`journal_mode=WAL` |
| 默认路径 | `~/.xunji/db/xunji.db`（见应用配置） |
| 版本 | `PRAGMA user_version`，当前迁移最高为 **v6** |
| 权威来源 | `storage/migrations.rs`（DDL）+ `storage/models.rs`（Rust 结构体与业务语义） |

**外键策略**：仅在 `messages`、`cards`、`card_tags`、`categories`（自关联）、`token_usage` 上声明 SQLite `FOREIGN KEY`。`sessions.source_id`、`sync_log.source_id`、`cards.category_id` 等在库中为**逻辑关联**，应用层负责一致性，便于数据源与分类的灵活演进。

---

## 2. 实体关系图（ER）

下列图中标注「逻辑」的连线在数据库中**没有** `FOREIGN KEY` 约束。

**图例**：字段类型在图中使用 Mermaid 支持的写法（`string` / `int` / `float`），与 SQLite 的 `TEXT` / `INTEGER` / `REAL` 一一对应；字段行尾双引号内为「中文说明 + 英文字段名」。  
**说明**：部分预览器对 `erDiagram` 的 `实体["别名"] { ... }` 或 SQL 风格类型名解析不兼容，会导致实体框只显示 “(no attributes)”。下图使用**纯表名 + 标准类型**，以保证属性能渲染；中文表名见下表。

| 表名 `identifier` | 中文含义 |
|---------------------|----------|
| `sources` | 数据源 |
| `sessions` | 会话 |
| `messages` | 对话消息 |
| `cards` | 知识卡片 |
| `tags` | 标签 |
| `card_tags` | 卡片与标签关联（多对多） |
| `categories` | 分类 |
| `sync_log` | 同步日志 |
| `token_usage` | Token 用量 / 计费流水 |

```mermaid
erDiagram
    sources {
        string id PK "主键 id"
        string name "显示名称 name"
        int enabled "是否启用 enabled"
        string scan_paths "扫描目录列表 JSON scan_paths"
        string last_sync "上次同步时间 last_sync"
        string config "扩展配置 JSON config"
    }

    sessions {
        string id PK "主键 id(应用内 UUID)"
        string source_id "数据源 id source_id, 逻辑关联 sources.id"
        string session_id "数据源侧会话标识 session_id"
        string source_host "来源主机 source_host, 去重维度之一"
        string project_path "项目路径 project_path"
        string project_name "项目名称 project_name"
        int message_count "消息条数 message_count"
        string content_hash "内容摘要哈希 content_hash"
        string raw_path "原始文件路径 raw_path"
        string created_at "创建时间 created_at"
        string updated_at "更新时间 updated_at"
        string status "分析状态 status"
        string value "价值等级 value"
        int has_updates "是否有新消息 has_updates"
        string analyzed_at "分析完成时间 analyzed_at"
        string error_message "分析错误信息 error_message"
        string analysis_note "低价值说明 analysis_note (v3)"
        string analysis_title "列表展示标题 analysis_title (v4)"
        string analysis_type "分析类型徽章 analysis_type (v4)"
    }

    messages {
        string id PK "主键 id"
        string session_id "所属会话 id session_id, FK sessions.id"
        string role "角色 role"
        string content "正文 content"
        string timestamp "时间戳 timestamp"
        int tokens_in "输入 token tokens_in"
        int tokens_out "输出 token tokens_out"
        int seq_order "会话内顺序 seq_order"
    }

    cards {
        string id PK "主键 id"
        string session_id "来源会话 id session_id, FK sessions.id"
        string title "标题 title"
        string type "知识类型 type"
        string value "价值等级 value"
        string summary "一句话摘要 summary"
        string note "笔记正文 Markdown note"
        string category_id "分类 id category_id, 逻辑 categories.id"
        string memory "Memory 规则 memory"
        string skill "Skill 定义 skill"
        string source_name "数据源名称冗余 source_name"
        string project_name "项目名称冗余 project_name"
        int prompt_tokens "提炼提示 token prompt_tokens"
        int completion_tokens "提炼补全 token completion_tokens"
        float cost_yuan "费用(元) cost_yuan"
        string feedback "用户反馈 feedback"
        string created_at "创建时间 created_at"
        string updated_at "更新时间 updated_at"
        string tech_stack "技术栈逗号串 tech_stack (v5)"
    }

    tags {
        string id PK "主键 id"
        string name "标签名 name, 唯一"
        string type "标签来源 type"
        int count "引用计数 count"
    }

    card_tags {
        string card_id "卡片 id card_id, FK cards.id"
        string tag_id "标签 id tag_id, FK tags.id"
    }

    categories {
        string id PK "主键 id"
        string name "分类名称 name"
        string parent_id "父分类 id parent_id, FK categories.id"
        int sort_order "排序 sort_order"
    }

    sync_log {
        string id PK "主键 id"
        string source_id "数据源 id source_id, 逻辑 sources.id"
        string started_at "开始时间 started_at"
        string finished_at "结束时间 finished_at"
        int sessions_found "扫描到会话数 sessions_found"
        int sessions_new "新导入会话数 sessions_new"
        int sessions_updated "检测到更新数 sessions_updated"
        string status "任务状态 status"
    }

    token_usage {
        string id PK "主键 id"
        string card_id "关联卡片 id card_id, 可空, FK cards.id"
        string provider "模型提供商 provider"
        string model "模型名 model"
        int prompt_tokens "提示 token prompt_tokens"
        int completion_tokens "补全 token completion_tokens"
        float cost_yuan "费用(元) cost_yuan"
        string created_at "记录时间 created_at"
    }

    sources ||--o{ sessions : "按数据源归类(逻辑关联 source_id)"
    sources ||--o{ sync_log : "同步任务归属(逻辑关联 source_id)"
    sessions ||--o{ messages : "会话包含多条消息(一对多)"
    sessions ||--o{ cards : "会话可有多张卡片(一对多)"
    cards ||--o{ card_tags : "卡片到关联表"
    card_tags }o--|| tags : "关联表到标签"
    categories ||--o| categories : "父子分类(自关联 parent_id)"
    cards }o--o| categories : "卡片归属分类(逻辑 category_id)"
    cards ||--o{ token_usage : "提炼计费流水(card_id 可空)"
```

**字段名说明**：`tags` 表中业务列在 SQLite 里名为 `type`；Mermaid 属性名若使用 `type` 易与语法关键字冲突，图中写作 `tag_type`，并在注释中标明「列名 type」。

---

## 3. 关系说明（业务视角）

```text
sources（数据源）
    │ source_id（逻辑）
    ├── sessions（会话，去重键：session_id + source_host UNIQUE）
    │       ├── messages（对话消息）
    │       └── cards（知识卡片）
    │               ├── card_tags ── tags（标签，多对多）
    │               └── token_usage（提炼费用，可关联卡片）
    └── sync_log（同步任务日志）

categories（分类树，自引用 parent_id）

cards_fts（FTS5 虚拟表，见下节，非业务实体表）
```

- **会话去重**：同一数据源侧 `session_id` 在不同 `source_host` 下可共存；组合 `(session_id, source_host)` 唯一。
- **卡片与标签**：标签名在 `tags.name` 上唯一；`card_tags` 为联合主键 `(card_id, tag_id)`。
- **全文搜索**：`cards_fts` 为 FTS5 独立索引表，字段 `title, summary, note, tags`；`tags` 列内容由 `card_tags` + `tags` 拼接后写入，与 `cards` 表无 `content=` 回链（见迁移 v2 说明）。

---

## 4. 各表字段概要

### 4.1 `sources`

AI IDE 数据源配置（名称、启用、扫描路径 JSON、上次同步时间、扩展配置 JSON）。

### 4.2 `sessions`

一条记录对应一次采集到的 AI 对话（如一个 JSONL 文件）。含分析流水线字段：`status`、`value`、`analyzed_at`、`error_message`；v3/v4 起增加低价值会话展示用 `analysis_note`、`analysis_title`、`analysis_type`。

### 4.3 `messages`

会话内消息，`seq_order` 为会话内顺序；`role` 如 user / assistant / tool 等。

### 4.4 `cards`

由 LLM 从会话提炼的笔记；`type` / `value` 等为业务枚举字符串；`tech_stack`（v5）为逗号分隔技术栈字符串；展示用标签来自关联查询，非 `cards` 表列。

### 4.5 `tags` / `card_tags`

标签字典与卡片—标签多对多桥表。

### 4.6 `categories`

分类树，`parent_id` 指向同表（可为空表示根）。

### 4.7 `sync_log`

某次同步任务的统计与状态，按 `source_id` 与数据源对应（逻辑关联）。

### 4.8 `token_usage`

LLM 调用计费流水；`card_id` 可空，声明外键指向 `cards(id)`。

### 4.9 `cards_fts`（虚拟表）

FTS5，用于卡片侧栏/列表等全文检索；应用层在卡片写入、更新、删除及标签变更时维护与 `cards` 一致。

---

## 5. 迁移版本与 Schema 增量

| 版本 | 内容摘要 |
|------|----------|
| v1 | 创建全部业务表、`cards_fts`、索引 |
| v2 | 重建 `cards_fts` 为独立 FTS5，并从现有 `cards` + 标签关联回填 |
| v3 | `sessions.analysis_note` |
| v4 | `sessions.analysis_title`、`sessions.analysis_type` |
| v5 | `cards.tech_stack` |
| v6 | 数据修复：解码 `sessions` 中仍含 URL 编码的 `project_path` / `project_name` |

执行顺序见 `run()`：按 `user_version` 递增直至当前最高版本。

---

## 6. 代码索引

| 模块 | 路径 |
|------|------|
| 迁移 DDL | `apps/desktop/src-tauri/src/storage/migrations.rs` |
| 模型与注释 | `apps/desktop/src-tauri/src/storage/models.rs` |
| 连接与错误类型 | `apps/desktop/src-tauri/src/storage/db.rs` |

文档随迁移变更时请同步更新本文与 `CHANGELOG`（若项目约定记录 Schema 变更）。
