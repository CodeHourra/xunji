//! 采集统一格式 —— 所有数据源解析后转换为此格式，再写入 SQLite。
//!
//! ```text
//! 数据源文件 → Source Parser → NormalizedSession → Dedup → SQLite
//! ```

/// 统一的会话格式，各数据源解析器产出此结构后交给调度器去重写入。
#[derive(Debug, Clone)]
pub struct NormalizedSession {
    /// 数据源 ID（如 "claude-code"、"cursor"），对应 config.toml 中的 source.id
    pub source_id: String,
    /// 原始会话 ID（数据源内部标识，如 UUID）
    pub session_id: String,
    /// 关联项目的绝对路径
    pub project_path: Option<String>,
    /// 项目名称（从路径末段推导）
    pub project_name: Option<String>,
    /// 该会话包含的消息列表（按时间顺序）
    pub messages: Vec<NormalizedMessage>,
    /// 原始数据文件路径（如 JSONL 文件位置）
    pub raw_path: String,
    /// 会话创建时间 (RFC 3339)
    pub created_at: String,
    /// 会话最后更新时间 (RFC 3339)
    pub updated_at: String,
}

/// 统一的单条消息格式
#[derive(Debug, Clone)]
pub struct NormalizedMessage {
    /// 角色: "user" | "assistant"
    pub role: String,
    /// 消息文本内容（已提取纯文本，tool_use 等保留简要摘要）
    pub content: String,
    /// 消息时间戳 (RFC 3339)
    pub timestamp: Option<String>,
    /// 该消息消耗的输入 token 数
    pub tokens_in: u32,
    /// 该消息消耗的输出 token 数
    pub tokens_out: u32,
}
