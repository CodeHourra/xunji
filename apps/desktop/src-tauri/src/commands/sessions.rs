//! 会话列表 / 详情 / 提炼（Sidecar + DB）命令。
//!
//! ```text
//! distill_session (spawn_blocking)
//!   ├── 读 Session + Messages → 拼接 content
//!   ├── RPC init → judge_value
//!   ├── value ∈ {medium, high} → distill_full → insert_card
//!   └── value ∈ {low, none} → 仅更新会话价值，返回业务错误（无 Card）
//! ```
//!
//! # 参数命名约定
//! 使用 `#[tauri::command(rename_all = "camelCase")]` 让前端 camelCase
//! 自动映射到 Rust snake_case 参数，无需 `args` 包装结构体。

use tauri::State;

use crate::config::AppConfig;
use crate::sidecar::SidecarManager;
use crate::storage::models::{Card, Message, NewCard, PaginatedResult, Session, SessionFilters, SessionSummary};
use crate::storage::Database;
use crate::AppState;

/// Sidecar `judge_value` 返回结构（字段名与 TS 一致）
#[derive(Debug, serde::Deserialize)]
struct JudgeValueResult {
    value: String,
    #[serde(rename = "type")]
    card_type: String,
    /// 低/无价值时的原因（也用于构造标题和摘要）
    reason: String,
    #[allow(dead_code)]
    prompt_tokens: i64,
    #[allow(dead_code)]
    completion_tokens: i64,
}

/// Sidecar `distill_full` 返回结构
#[derive(Debug, serde::Deserialize)]
struct DistillFullResult {
    title: String,
    #[serde(rename = "type")]
    card_type: String,
    value: String,
    summary: String,
    note: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    tech_stack: Vec<String>,
    prompt_tokens: i64,
    completion_tokens: i64,
}

/// 分页查询会话列表。
/// 参数全部 camelCase，与前端 `SessionListParams` 字段一致。
#[tauri::command(rename_all = "camelCase")]
pub async fn list_sessions(
    state: State<'_, AppState>,
    source: Option<String>,
    host: Option<String>,
    project: Option<String>,
    status: Option<String>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<PaginatedResult<SessionSummary>, String> {
    let db = state.db.clone();
    let filters = SessionFilters {
        source,
        host,
        project,
        status,
        search: None,
    };
    let page = page.unwrap_or(1).max(1);
    let page_size = page_size.unwrap_or(20).max(1).min(200);

    tokio::task::spawn_blocking(move || {
        db.list_sessions(&filters, page, page_size)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("list_sessions join 失败: {}", e))?
}

/// 获取会话完整信息。
#[tauri::command]
pub async fn get_session(state: State<'_, AppState>, id: String) -> Result<Session, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || db.get_session(&id).map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("get_session join 失败: {}", e))?
}

/// 拉取会话下全部消息（按 seq_order 排序），供详情页对话回放。
#[tauri::command(rename_all = "camelCase")]
pub async fn get_session_messages(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<Message>, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        db.get_session_messages(&session_id).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("get_session_messages join 失败: {}", e))?
}

/// `distill_session` 返回结果
///
/// ```text
/// is_low_value = true  → low / none，DB 已记录价值，无 Card 产出
/// is_low_value = false → medium / high，Card 已写库
/// ```
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DistillSessionResult {
    /// 分析后的价值等级（high / medium / low / none）
    pub value: String,
    /// true = 低/无价值，未生成笔记；false = 已生成笔记
    pub is_low_value: bool,
    /// 低/无价值时：由 reason 构造的简短标题
    pub card_title: Option<String>,
    /// 低/无价值时：judge_value 返回的对话类型（debug / learning / …）
    pub card_type: Option<String>,
    /// 低/无价值时的原因说明（作为摘要展示）
    pub reason: Option<String>,
    /// 生成的卡片（仅 is_low_value = false 时有值）
    pub card: Option<Card>,
}

/// 对指定会话执行价值判断 +（可选）完整提炼，并写入 `cards` 表。
#[tauri::command(rename_all = "camelCase")]
pub async fn distill_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<DistillSessionResult, String> {
    let sidecar = state
        .sidecar
        .clone()
        .ok_or_else(|| "未找到 xunji-sidecar 可执行文件，请先构建 packages/sidecar 或安装到 ~/.xunji/bin/".to_string())?;

    let db = state.db.clone();
    // 取当前配置快照（RwLock → Clone），传入 spawn_blocking
    let config = state.config_snapshot();

    tokio::task::spawn_blocking(move || {
        run_distill_pipeline(&db, &config, sidecar.as_ref(), &session_id)
    })
    .await
    .map_err(|e| format!("distill_session join 失败: {}", e))?
}


/// 在阻塞线程内跑完 DB + Sidecar 全流程（避免阻塞 tokio worker）。
///
/// 流程：
/// ```text
/// analyzing 状态 → init → judge_value (PROMPT_B_LIGHT)
///   ├── low / none → 更新 DB 为 analyzed + value → Ok(isLowValue=true)
///   └── medium / high → distill_full (PROMPT_B_FULL) → insert_card → Ok(isLowValue=false)
/// ```
fn run_distill_pipeline(
    db: &Database,
    config: &AppConfig,
    sidecar: &SidecarManager,
    session_db_id: &str,
) -> Result<DistillSessionResult, String> {
    let session = db.get_session(session_db_id).map_err(|e| e.to_string())?;
    let messages = db
        .get_session_messages(session_db_id)
        .map_err(|e| e.to_string())?;

    if messages.is_empty() {
        return Err("该会话没有消息，无法提炼".to_string());
    }

    let content = build_transcript(&messages);
    if content.trim().is_empty() {
        return Err("会话正文为空，无法提炼".to_string());
    }

    // 设置 analyzing 状态
    db.update_session_status(session_db_id, "analyzing", None)
        .map_err(|e| e.to_string())?;

    // 用闭包包裹后续逻辑，确保任何真正失败都能将状态回退为 error
    let result = (|| -> Result<DistillSessionResult, String> {
        let init_params = config.sidecar_init_params()?;

        sidecar
            .call_with_timeout::<serde_json::Value>(
                "init",
                init_params,
                std::time::Duration::from_secs(30),
            )
            .map_err(|e| e.to_string())?;

        // ── 第一步：轻量价值判断（PROMPT_B_LIGHT） ──
        let judge: JudgeValueResult = sidecar
            .call_with_timeout(
                "judge_value",
                serde_json::json!({ "content": content }),
                std::time::Duration::from_secs(120),
            )
            .map_err(|e| e.to_string())?;

        let v_norm = judge.value.to_lowercase();
        log::info!("价值判断结果: {} (session={})", v_norm, session_db_id);

        // ── low / none：更新价值 + 持久化标题/类型/原因 → 正常返回（非错误） ──
        if v_norm == "low" || v_norm == "none" {
            if let Err(e) =
                db.update_session_status(session_db_id, "analyzed", Some(&judge.value))
            {
                log::error!("低/无价值会话状态回写失败: {}", e);
            }

            // 由 reason 截取前 30 字符作为显示标题（不截断词，保留语义）
            let title = build_analysis_title(&judge.reason);

            // 将标题、类型、原因一并持久化，刷新后列表仍可正确展示
            if let Err(e) = db.update_session_analysis_meta(
                session_db_id,
                &title,
                &judge.card_type,
                &judge.reason,
            ) {
                log::error!("写入 analysis_meta 失败: {}", e);
            }

            return Ok(DistillSessionResult {
                value: judge.value,
                is_low_value: true,
                card_title: Some(title),
                card_type: Some(judge.card_type),
                reason: Some(judge.reason),
                card: None,
            });
        }

        if v_norm != "medium" && v_norm != "high" {
            return Err(format!("未知的价值等级: {}", judge.value));
        }

        // ── 第二步：完整笔记提炼（PROMPT_B_FULL，仅 medium / high） ──
        let full: DistillFullResult = sidecar
            .call_with_timeout(
                "distill_full",
                serde_json::json!({ "content": content }),
                std::time::Duration::from_secs(300),
            )
            .map_err(|e| e.to_string())?;

        db.delete_cards_for_session(session_db_id)
            .map_err(|e| e.to_string())?;

        let source_name = config.source_display_name(&session.source_id);
        let project_name = session.project_name.clone();

        let new_card = NewCard {
            session_id: session_db_id,
            title: full.title.as_str(),
            card_type: Some(full.card_type.as_str()),
            value: Some(full.value.as_str()),
            summary: Some(full.summary.as_str()),
            note: full.note.as_str(),
            source_name: source_name.as_deref(),
            project_name: project_name.as_deref(),
            prompt_tokens: full.prompt_tokens.clamp(0, i32::MAX as i64) as i32,
            completion_tokens: full.completion_tokens.clamp(0, i32::MAX as i64) as i32,
            cost_yuan: 0.0,
            tags: &full.tags,
            tech_stack: &full.tech_stack,
        };

        let card_id = db.insert_card(&new_card).map_err(|e| e.to_string())?;

        db.update_session_status(session_db_id, "analyzed", Some(&full.value))
            .map_err(|e| e.to_string())?;

        let card = db.get_card(&card_id).map_err(|e| e.to_string())?;
        Ok(DistillSessionResult {
            value: full.value,
            is_low_value: false,
            card_title: None,
            card_type: None,
            reason: None,
            card: Some(card),
        })
    })();

    // 真正失败时（非低价值场景），状态回退为 error
    if let Err(ref e) = result {
        if let Ok(s) = db.get_session(session_db_id) {
            if s.status == "analyzing" {
                let _ = db.update_session_error(session_db_id, e);
            }
        }
    }

    result
}

/// 从 LLM 判断原因截取简短标题（最多 30 个 Unicode 字符，超出时附加省略号）。
///
/// 示例：
///   "问题过于简单，答案是常识" → "问题过于简单，答案是常识"（未超长）
///   "内容高度重复，没有新知识点，用户反复询问相同内容，建议归档。" → "内容高度重复，没有新知识点，用户反…"
fn build_analysis_title(reason: &str) -> String {
    const MAX_CHARS: usize = 30;
    let chars: Vec<char> = reason.chars().collect();
    if chars.len() <= MAX_CHARS {
        reason.to_string()
    } else {
        chars[..MAX_CHARS].iter().collect::<String>() + "…"
    }
}

/// 将消息列表拼成单一字符串，供 LLM 前处理（角色标签 + 换行分隔）。
fn build_transcript(messages: &[Message]) -> String {
    let mut out = String::new();
    for m in messages {
        if !out.is_empty() {
            out.push_str("\n\n");
        }
        out.push('[');
        out.push_str(&m.role);
        out.push_str("]\n");
        out.push_str(&m.content);
    }
    out
}
