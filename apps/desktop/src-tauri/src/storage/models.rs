use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub scan_paths: Option<String>,
    pub last_sync: Option<String>,
    pub config: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub source_id: String,
    pub session_id: String,
    pub source_host: String,
    pub project_path: Option<String>,
    pub project_name: Option<String>,
    pub message_count: i64,
    pub content_hash: Option<String>,
    pub raw_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub status: String,
    pub value: Option<String>,
    pub has_updates: bool,
    pub analyzed_at: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub source_id: String,
    pub session_id: String,
    pub source_host: String,
    pub project_path: Option<String>,
    pub project_name: Option<String>,
    pub message_count: i64,
    pub status: String,
    pub updated_at: String,
    pub has_updates: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub timestamp: Option<String>,
    pub tokens_in: i64,
    pub tokens_out: i64,
    pub seq_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub session_id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub card_type: Option<String>,
    pub value: Option<String>,
    pub summary: Option<String>,
    pub note: String,
    pub category_id: Option<String>,
    pub memory: Option<String>,
    pub skill: Option<String>,
    pub source_name: Option<String>,
    pub project_name: Option<String>,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub cost_yuan: f64,
    pub feedback: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    /// Populated from card_tags JOIN, not stored in cards table directly.
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub tag_type: Option<String>,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncLog {
    pub id: String,
    pub source_id: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub sessions_found: i64,
    pub sessions_new: i64,
    pub sessions_updated: i64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub id: String,
    pub card_id: Option<String>,
    pub provider: String,
    pub model: String,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub cost_yuan: f64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionFilters {
    pub source: Option<String>,
    pub project: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CardFilters {
    pub tags: Option<Vec<String>>,
    pub card_type: Option<String>,
    pub value: Option<String>,
    pub search: Option<String>,
}
