//! CodeBuddy CLI JSONL 数据源解析器。
//!
//! 目录结构（与 PRD 一致）：
//! ```text
//! ~/.codebuddy/projects/
//! └── {项目目录名}/
//!     └── {session-uuid}.jsonl
//! ```
//!
//! 每行 JSON 关键字段（参见 PRD）：
//! - `type`: `message` | `function_call` | `function_call_result` —— 仅采集 `message` 中的 user/assistant 文本
//! - `role`: `user` | `assistant`
//! - `content`: `[{ "type": "input_text|output_text", "text": "..." }]` 或兼容字符串
//! - `providerData.usage`: `{ "inputTokens", "outputTokens" }`（camelCase）

use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

use super::normalizer::{NormalizedMessage, NormalizedSession};

/// CodeBuddy CLI 采集器（`source_id` 固定为 `codebuddy-cli`，与 config.toml 一致）
pub struct CodeBuddyCliCollector {
    scan_dirs: Vec<PathBuf>,
}

impl CodeBuddyCliCollector {
    pub fn new(scan_dirs: Vec<PathBuf>) -> Self {
        Self { scan_dirs }
    }

    /// 扫描 `scan_dirs` 下 `projects/*/xxx.jsonl`，产出标准化会话列表
    pub fn collect(&self) -> Vec<NormalizedSession> {
        let mut sessions = Vec::new();

        for root in &self.scan_dirs {
            let projects_dir = root.join("projects");
            if !projects_dir.is_dir() {
                log::debug!(
                    "CodeBuddy projects 目录不存在，跳过: {}",
                    projects_dir.display()
                );
                continue;
            }

            match self.scan_projects_dir(&projects_dir) {
                Ok(mut found) => {
                    log::info!(
                        "从 {} 扫描到 {} 个 CodeBuddy CLI 会话",
                        projects_dir.display(),
                        found.len()
                    );
                    sessions.append(&mut found);
                }
                Err(e) => {
                    log::error!(
                        "扫描 CodeBuddy 目录失败: {} - {}",
                        projects_dir.display(),
                        e
                    );
                }
            }
        }

        log::info!("CodeBuddy CLI 采集完成，共 {} 个会话", sessions.len());
        sessions
    }

    fn scan_projects_dir(&self, projects_dir: &Path) -> Result<Vec<NormalizedSession>, std::io::Error> {
        let mut sessions = Vec::new();

        for entry in fs::read_dir(projects_dir)? {
            let entry = entry?;
            let project_dir = entry.path();
            if !project_dir.is_dir() {
                continue;
            }

            let project_folder = entry.file_name().to_string_lossy().to_string();
            let project_name = Some(project_folder.clone());

            for file_entry in fs::read_dir(&project_dir)? {
                let file_entry = file_entry?;
                let file_path = file_entry.path();
                if file_path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                    continue;
                }

                let session_id = file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                if session_id.is_empty() {
                    continue;
                }

                match parse_jsonl_session(&file_path) {
                    Ok(parse_result) => {
                        if parse_result.messages.is_empty() {
                            log::debug!("会话无有效消息，跳过: {}", file_path.display());
                            continue;
                        }

                        let created_at = parse_result
                            .first_timestamp
                            .clone()
                            .unwrap_or_else(|| parse_result.fallback_time.clone());
                        let updated_at = parse_result
                            .last_timestamp
                            .clone()
                            .unwrap_or_else(|| parse_result.fallback_time.clone());

                        sessions.push(NormalizedSession {
                            source_id: "codebuddy-cli".to_string(),
                            session_id,
                            project_path: None,
                            project_name: project_name.clone(),
                            messages: parse_result.messages,
                            raw_path: file_path.to_string_lossy().to_string(),
                            created_at,
                            updated_at,
                        });
                    }
                    Err(e) => {
                        log::warn!("解析 CodeBuddy JSONL 失败: {} - {}", file_path.display(), e);
                    }
                }
            }
        }

        Ok(sessions)
    }
}

struct ParseResult {
    messages: Vec<NormalizedMessage>,
    first_timestamp: Option<String>,
    last_timestamp: Option<String>,
    /// 当 JSON 内无时间戳时，用文件修改时间作为会话时间兜底
    fallback_time: String,
}

fn parse_jsonl_session(path: &Path) -> Result<ParseResult, Box<dyn std::error::Error>> {
    let meta = fs::metadata(path)?;
    let fallback_time = file_mtime_rfc3339(&meta)?;

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut messages = Vec::new();
    let mut first_timestamp: Option<String> = None;
    let mut last_timestamp: Option<String> = None;

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let v: serde_json::Value = match serde_json::from_str(&line) {
            Ok(x) => x,
            Err(e) => {
                log::trace!("跳过无法解析的行: {}", e);
                continue;
            }
        };

        let msg_type = v["type"].as_str().unwrap_or("");
        if msg_type != "message" {
            continue;
        }

        let role = v["role"].as_str().unwrap_or("");
        if role != "user" && role != "assistant" {
            continue;
        }

        let text = extract_codebuddy_content(&v["content"]);
        if text.trim().is_empty() {
            continue;
        }

        let usage = &v["providerData"]["usage"];
        let tokens_in = usage["inputTokens"]
            .as_u64()
            .or_else(|| usage["input_tokens"].as_u64())
            .unwrap_or(0) as u32;
        let tokens_out = usage["outputTokens"]
            .as_u64()
            .or_else(|| usage["output_tokens"].as_u64())
            .unwrap_or(0) as u32;

        let timestamp = v["timestamp"]
            .as_str()
            .map(String::from)
            .or_else(|| v["createdAt"].as_str().map(String::from))
            .or_else(|| v["time"].as_str().map(String::from));

        if let Some(ref ts) = timestamp {
            if first_timestamp.is_none() {
                first_timestamp = Some(ts.clone());
            }
            last_timestamp = Some(ts.clone());
        }

        messages.push(NormalizedMessage {
            role: role.to_string(),
            content: text,
            timestamp,
            tokens_in,
            tokens_out,
        });
    }

    Ok(ParseResult {
        messages,
        first_timestamp,
        last_timestamp,
        fallback_time,
    })
}

/// 从 `content` 字段提取纯文本（数组或字符串）
fn extract_codebuddy_content(content: &serde_json::Value) -> String {
    match content {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => {
            let mut parts = Vec::new();
            for item in arr {
                let t = item["text"].as_str().unwrap_or("");
                let typ = item["type"].as_str().unwrap_or("");
                if typ == "input_text" || typ == "output_text" || typ.is_empty() {
                    if !t.is_empty() {
                        parts.push(t);
                    }
                }
            }
            parts.join("\n")
        }
        _ => String::new(),
    }
}

fn file_mtime_rfc3339(meta: &std::fs::Metadata) -> Result<String, Box<dyn std::error::Error>> {
    let t = meta.modified().or_else(|_| meta.created())?;
    let secs = t.duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;
    let dt = DateTime::<Utc>::from_timestamp(secs, 0).unwrap_or_else(Utc::now);
    Ok(dt.to_rfc3339())
}
