use serde::{Deserialize, Serialize};

// ─────────────────────────────── 数据源 ───────────────────────────────

/// 数据源配置（如 Claude Code、Cursor 等 AI IDE）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // v0.2 设置页面使用
pub struct Source {
    pub id: String,
    /// 数据源显示名称，如 "Claude Code"
    pub name: String,
    /// 是否启用该数据源的采集
    pub enabled: bool,
    /// 扫描目录列表，JSON 数组格式存储
    pub scan_paths: Option<String>,
    /// 最近一次同步时间 (RFC 3339)
    pub last_sync: Option<String>,
    /// 数据源专属配置，JSON 格式
    pub config: Option<String>,
}

// ─────────────────────────────── 会话 ─────────────────────────────────
//
//  一条 Session 对应一次 AI 对话（如 Claude Code 的一个 JSONL 文件）。
//  去重键: (session_id, source_host)

/// AI 对话会话完整信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    /// 数据库主键 (UUID v4)
    pub id: String,
    /// 来源数据源 ID，关联 sources.id
    pub source_id: String,
    /// 原始会话 ID（数据源自身的标识符）
    pub session_id: String,
    /// 来源主机标识，用于本地/远端去重，默认 "local"
    pub source_host: String,
    /// 关联项目路径
    pub project_path: Option<String>,
    /// 关联项目名称（从路径推导或手动设置）
    pub project_name: Option<String>,
    /// 采集或分析阶段的可读标题（如 CodeBuddy 工作区 index 中的会话 name）
    #[serde(default)]
    pub analysis_title: Option<String>,
    /// 消息总条数
    pub message_count: i64,
    /// 消息内容的 MD5 摘要，用于变更检测
    pub content_hash: Option<String>,
    /// 原始文件路径（如 JSONL 文件位置）
    pub raw_path: Option<String>,
    /// 会话创建时间 (RFC 3339)
    pub created_at: String,
    /// 会话最后更新时间 (RFC 3339)
    pub updated_at: String,
    /// 分析状态: pending | analyzing | analyzed | error
    pub status: String,
    /// LLM 判定的价值等级: high | medium | low | none
    pub value: Option<String>,
    /// 增量同步时检测到新消息，标记为 true
    pub has_updates: bool,
    /// 完成分析的时间 (RFC 3339)
    pub analyzed_at: Option<String>,
    /// 分析失败时的错误信息
    pub error_message: Option<String>,
}

/// 会话列表轻量版（列表所需路径 / 首条 user 预览等；不含 content_hash）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub id: String,
    pub source_id: String,
    /// 数据源侧会话标识（如 JSONL 元数据中的 sessionId）
    pub session_id: String,
    pub source_host: String,
    pub project_path: Option<String>,
    pub project_name: Option<String>,
    pub message_count: i64,
    pub status: String,
    /// 价值评估结果（列表展示色条 / Badge 用，未分析时为 null）
    #[serde(default)]
    pub value: Option<String>,
    pub updated_at: String,
    pub has_updates: bool,
    pub created_at: String,
    /// 该会话下最新一张知识卡片的 ID（无卡片时为 null，供列表「查看笔记」跳转）
    #[serde(default)]
    pub card_id: Option<String>,
    /// 所有消息内容字节总量（SUM(LENGTH(content))），供列表显示 xx KB
    #[serde(default)]
    pub raw_size_bytes: i64,
    /// 最新知识卡片标题（已分析时展示，代替"待分析"占位文字）
    #[serde(default)]
    pub card_title: Option<String>,
    /// 最新知识卡片一句话摘要
    #[serde(default)]
    pub card_summary: Option<String>,
    /// 最新知识卡片类型（debug / implementation / research / …）
    #[serde(default)]
    pub card_type: Option<String>,
    /// 最新知识卡片标签（逗号分隔字符串，如 "Rust,SQLite,FTS5"）
    #[serde(default)]
    pub card_tags: Option<String>,
    /// 原始会话文件路径（如 JSONL），列表与 tooltip 展示
    #[serde(default)]
    pub raw_path: Option<String>,
    /// 分析失败原因（status = error 时由 update_session_error 写入）
    #[serde(default)]
    pub error_message: Option<String>,
    /// 首条 user 消息正文（SQL 中 SUBSTR 限制长度），供列表主标题与延展 tooltip
    #[serde(default)]
    pub first_user_preview: Option<String>,
}

// ─────────────────────────────── 消息 ─────────────────────────────────

/// 单条对话消息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    /// 所属会话的数据库主键
    pub session_id: String,
    /// 角色: user | assistant | tool | model 等（送提炼 LLM 时仅保留 user/assistant/model）
    pub role: String,
    /// 消息文本内容
    pub content: String,
    /// 消息时间戳 (RFC 3339)
    pub timestamp: Option<String>,
    /// 该消息消耗的输入 token 数
    pub tokens_in: i64,
    /// 该消息消耗的输出 token 数
    pub tokens_out: i64,
    /// 消息在会话中的顺序（0-based）
    pub seq_order: i64,
}

// ─────────────────────────────── 知识卡片 ──────────────────────────────
//
//  一条 Card 由 LLM 从 Session 中提炼生成。
//
//  Session 1:1 Card （v0.1 单次分析产生一张卡片）
//  Card  N:M  Tag  （通过 card_tags 关联表）

/// LLM 提炼的知识卡片完整信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub id: String,
    /// 来源会话的数据库主键
    pub session_id: String,
    /// 卡片标题（LLM 生成）
    pub title: String,
    /// 知识类型: debug | architecture | performance | best-practice | concept | tool-usage | refactor | other
    #[serde(rename = "type")]
    pub card_type: Option<String>,
    /// 价值等级: high | medium | low | none
    pub value: Option<String>,
    /// 一句话摘要（LLM 生成）
    pub summary: Option<String>,
    /// Markdown 格式的完整技术笔记
    pub note: String,
    /// 所属分类 ID（v0.1 暂未使用）
    pub category_id: Option<String>,
    /// LLM 生成的 Memory 规则（v0.2 实现）
    pub memory: Option<String>,
    /// LLM 生成的 Skill 定义（v0.2 实现）
    pub skill: Option<String>,
    /// 来源数据源名称（冗余存储，方便展示）
    pub source_name: Option<String>,
    /// 来源项目名称（冗余存储，方便展示）
    pub project_name: Option<String>,
    /// 提炼消耗的 prompt tokens
    pub prompt_tokens: i64,
    /// 提炼消耗的 completion tokens
    pub completion_tokens: i64,
    /// 提炼消耗的费用（元）
    pub cost_yuan: f64,
    /// 用户反馈: positive | negative
    pub feedback: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    /// 通过 card_tags + tags 表 JOIN 查询填充，不直接存储在 cards 表
    #[serde(default)]
    pub tags: Vec<String>,
    /// LLM 提炼时识别到的技术栈（如 Rust、SQLite、Tauri 等），逗号分隔存储
    #[serde(default)]
    pub tech_stack: Vec<String>,
    /// 来源会话在数据源侧的标识（`sessions.session_id`，非 DB 主键 `sessions.id`）
    #[serde(default)]
    pub source_session_external_id: Option<String>,
    /// 来源会话路径：优先 `raw_path`，否则 `project_path`（与列表会话路径语义一致）
    #[serde(default)]
    pub source_session_path: Option<String>,
}

/// 卡片列表轻量版（不含 note、memory、skill 等大字段）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardSummary {
    pub id: String,
    pub session_id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub card_type: Option<String>,
    pub value: Option<String>,
    pub summary: Option<String>,
    pub category_id: Option<String>,
    pub source_name: Option<String>,
    pub project_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// ─────────────────────────────── 标签 ─────────────────────────────────

/// 知识标签（LLM 自动生成或用户手动创建）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // v0.2 标签管理页使用
pub struct Tag {
    pub id: String,
    /// 标签名称（唯一约束）
    pub name: String,
    /// 标签来源: auto（LLM 生成）| manual（用户创建）
    #[serde(rename = "type")]
    pub tag_type: Option<String>,
    /// 使用该标签的卡片数量（非实时计算）
    pub count: i64,
}

// ─────────────────────────────── 同步日志 ──────────────────────────────

/// 数据源同步操作日志
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // v0.2 同步历史页使用
pub struct SyncLog {
    pub id: String,
    /// 触发同步的数据源 ID
    pub source_id: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    /// 本次扫描发现的会话总数
    pub sessions_found: i64,
    /// 本次新导入的会话数
    pub sessions_new: i64,
    /// 本次检测到更新的会话数
    pub sessions_updated: i64,
    /// 同步状态: running | completed | failed
    pub status: String,
}

// ─────────────────────────────── Token 用量 ───────────────────────────

/// LLM 调用的 Token 用量与费用记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // v0.2 费用统计页使用
pub struct TokenUsage {
    pub id: String,
    /// 关联的知识卡片 ID
    pub card_id: Option<String>,
    /// LLM 提供商名称（如 openai、deepseek）
    pub provider: String,
    /// 使用的模型名称
    pub model: String,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    /// 费用（元）
    pub cost_yuan: f64,
    pub created_at: String,
}

// ─────────────────────────────── 写入用结构体 ─────────────────────────
//
// 将多参数方法的入参封装为结构体，提升可读性和可维护性。

/// 创建知识卡片的入参
pub struct NewCard<'a> {
    pub session_id: &'a str,
    pub title: &'a str,
    /// 知识类型: debug | architecture | performance | best-practice | ...
    pub card_type: Option<&'a str>,
    /// 价值等级: high | medium | low | none
    pub value: Option<&'a str>,
    pub summary: Option<&'a str>,
    /// Markdown 格式技术笔记
    pub note: &'a str,
    pub source_name: Option<&'a str>,
    pub project_name: Option<&'a str>,
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    /// 费用（元），浮点精度在 v0.1 可接受（不用于财务计算）
    pub cost_yuan: f64,
    /// LLM 生成的标签列表
    pub tags: &'a [String],
    /// LLM 识别的技术栈列表（如 ["Rust", "SQLite", "Tauri"]）
    pub tech_stack: &'a [String],
}

/// 批量写入消息的单条入参
pub struct NewMessage {
    /// 角色: user | assistant | tool
    pub role: String,
    /// 消息文本内容
    pub content: String,
    /// 消息时间戳 (RFC 3339)
    pub timestamp: Option<String>,
    /// 该消息消耗的输入 token 数
    pub tokens_in: i32,
    /// 该消息消耗的输出 token 数
    pub tokens_out: i32,
}

// ─────────────────────────────── 分页与筛选 ───────────────────────────

/// 分页查询结果的通用封装
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    /// 满足筛选条件的总记录数（不受分页限制）
    pub total: u64,
    /// 当前页码（1-based）
    pub page: u32,
    pub page_size: u32,
}

/// 会话列表筛选条件
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionFilters {
    /// 按数据源 ID 筛选
    pub source: Option<String>,
    /// 按来源主机筛选（目录树第二层）
    pub host: Option<String>,
    /// 按项目名称或路径筛选（目录树第三层）
    pub project: Option<String>,
    /// 按分析状态筛选: pending | analyzing | analyzed | error
    pub status: Option<String>,
    /// 全文搜索关键词（v0.2 实现）
    pub search: Option<String>,
}

/// 会话按 source → host → project 分组统计，用于侧栏目录树
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionGroupCount {
    /// 数据源 ID（如 "claude-code"）
    pub source_id: String,
    /// 来源主机（如 "localhost"）
    pub source_host: String,
    /// 项目名称（无项目时为 null）
    pub project_name: Option<String>,
    /// 该分组下的会话数量
    pub count: i64,
}

/// 标签及其关联卡片数量，用于侧栏标签筛选
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagCount {
    /// 标签名称
    pub name: String,
    /// 使用该标签的卡片数量
    pub count: i64,
}

/// 知识卡片类型及数量统计，用于侧栏类型筛选
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeCount {
    /// 类型名称（如 debug、architecture 等）
    pub name: String,
    /// 该类型的卡片数量
    pub count: i64,
}

/// 知识卡片筛选条件
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CardFilters {
    /// 按标签名称筛选（AND 语义，需同时匹配所有标签）
    pub tags: Option<Vec<String>>,
    /// 按知识类型筛选
    pub card_type: Option<String>,
    /// 按价值等级筛选
    pub value: Option<String>,
    /// FTS5 全文搜索关键词
    pub search: Option<String>,
    /// 按技术栈名称筛选（AND 语义；与 `cards.tech_stack` 逗号分隔字段逐 token 匹配，大小写不敏感）
    pub tech_stack: Option<Vec<String>>,
}
