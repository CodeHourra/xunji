//! 知识卡片导出为 Markdown（YAML frontmatter + 正文），由前端选择路径后调用。
//!
//! ```text
//! 前端 dialog 选路径 → invoke → 本模块读库拼内容 → std::fs 落盘
//! （桌面端 Rust 进程可直接写用户目录，无需 tauri-plugin-fs）
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::State;

use crate::storage::models::Card;
use crate::AppState;

/// 与 `docs/specs/导出-Markdown-约定.md` 一致的 frontmatter 字段
#[derive(Serialize)]
struct ExportFrontmatter {
    title: String,
    source: Option<String>,
    project: Option<String>,
    created_at: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    card_type: Option<String>,
    /// 数据源侧会话标识（sessions.session_id）
    session_id: Option<String>,
    card_id: String,
    /// 应用内 sessions 表主键（UUID），便于与 UI/DB 对照
    session_internal_id: String,
}

/// 文件名非法字符替换，并限制长度，避免跨平台问题
fn sanitize_filename_base(title: &str) -> String {
    let mut out = String::new();
    for c in title.chars().take(100) {
        match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '#' | '\0'..='\x1f' => {
                out.push('-')
            }
            c => out.push(c),
        }
    }
    let s = out.trim().trim_matches('.').trim_matches('-').to_string();
    if s.is_empty() {
        "note".into()
    } else {
        s
    }
}

/// 默认导出文件名：`{标题摘要}-{cardId 前 8 位}.md`，UUID 前缀保证唯一
fn default_export_filename(card: &Card) -> String {
    let base = sanitize_filename_base(&card.title);
    let short = card.id.chars().take(8).collect::<String>();
    format!("{}-{}.md", base, short)
}

fn build_markdown_document(card: &Card) -> Result<String, String> {
    let fm = ExportFrontmatter {
        title: card.title.clone(),
        source: card.source_name.clone(),
        project: card.project_name.clone(),
        created_at: card.created_at.clone(),
        tags: card.tags.clone(),
        card_type: card.card_type.clone(),
        session_id: card.source_session_external_id.clone(),
        card_id: card.id.clone(),
        session_internal_id: card.session_id.clone(),
    };
    let yaml = serde_yaml::to_string(&fm).map_err(|e| format!("YAML 序列化失败: {}", e))?;
    Ok(format!("---\n{yaml}---\n\n{}", card.note))
}

fn write_file(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    fs::write(path, content).map_err(|e| format!("写入文件失败: {}", e))?;
    Ok(())
}

/// 单条导出：用户通过「另存为」得到完整文件路径
#[tauri::command(rename_all = "camelCase")]
pub async fn export_card_markdown(
    state: State<'_, AppState>,
    card_id: String,
    file_path: String,
) -> Result<(), String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let card = db.get_card(&card_id).map_err(|e| e.to_string())?;
        let md = build_markdown_document(&card)?;
        let path = PathBuf::from(file_path);
        write_file(&path, &md)?;
        log::info!("导出单条笔记: card_id={} -> {:?}", card_id, path);
        Ok(())
    })
    .await
    .map_err(|e| format!("export_card_markdown 任务失败: {}", e))?
}

/// 批量导出：将所选 id 各写为一文件，文件名由标题+id 派生
#[tauri::command(rename_all = "camelCase")]
pub async fn export_cards_markdown_dir(
    state: State<'_, AppState>,
    card_ids: Vec<String>,
    dir_path: String,
) -> Result<u32, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let dir = Path::new(&dir_path);
        fs::create_dir_all(dir).map_err(|e| format!("创建目录失败: {}", e))?;
        let mut n: u32 = 0;
        for cid in card_ids {
            let card = db.get_card(&cid).map_err(|e| e.to_string())?;
            let md = build_markdown_document(&card)?;
            let name = default_export_filename(&card);
            let path = dir.join(name);
            write_file(&path, &md)?;
            n += 1;
            log::debug!("批量导出: {} -> {:?}", cid, path);
        }
        log::info!("批量导出完成: {} 条 -> {}", n, dir_path);
        Ok(n)
    })
    .await
    .map_err(|e| format!("export_cards_markdown_dir 任务失败: {}", e))?
}

/// 全库导出（**无视筛选**）：遍历 `cards` 全表
#[tauri::command(rename_all = "camelCase")]
pub async fn export_all_cards_markdown_dir(
    state: State<'_, AppState>,
    dir_path: String,
) -> Result<u32, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let ids = db.list_all_card_ids().map_err(|e| e.to_string())?;
        let dir = Path::new(&dir_path);
        fs::create_dir_all(dir).map_err(|e| format!("创建目录失败: {}", e))?;
        let mut n: u32 = 0;
        for cid in ids {
            let card = db.get_card(&cid).map_err(|e| e.to_string())?;
            let md = build_markdown_document(&card)?;
            let path = dir.join(default_export_filename(&card));
            write_file(&path, &md)?;
            n += 1;
        }
        log::info!("全库导出完成: {} 条 -> {}", n, dir_path);
        Ok(n)
    })
    .await
    .map_err(|e| format!("export_all_cards_markdown_dir 任务失败: {}", e))?
}

/// 全库卡片数量（与当前知识库筛选无关）
#[tauri::command(rename_all = "camelCase")]
pub async fn count_all_cards(state: State<'_, AppState>) -> Result<u64, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || db.count_all_cards().map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("count_all_cards 任务失败: {}", e))?
}
