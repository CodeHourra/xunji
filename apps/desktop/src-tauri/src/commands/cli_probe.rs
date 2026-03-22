//! 在桌面应用环境中探测本机已安装的 AI CLI 可执行文件路径。
//!
//! ```text
//! GUI 启动时进程 PATH 往往不含 nvm / Homebrew，与终端不一致。
//! 此处与 sidecar 的 cli-provider 思路一致：在 Unix 上通过「登录 shell」执行
//! `command -v <name>`，拿到与终端一致的绝对路径，便于直接写入配置。
//! ```

use std::process::{Command, Stdio};

/// 前端展示用：每个候选命令及其解析结果（绝对路径或未发现）
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliProbeResultDto {
    /// 探测用的命令名（如 claude-internal）
    pub name: String,
    /// `command -v` 成功时的绝对路径；未发现则为 null
    pub resolved_path: Option<String>,
}

/// 与 UI 快捷标签对齐：先 *-internal 再正式发行名，固定顺序便于行为一致
const PROBE_NAMES: &[&str] = &[
    "claude-internal",
    "gemini-internal",
    "codex-internal",
    "claude",
    "gemini",
    "codex",
];

/// 在「交互 + 登录」shell 下解析单个命令名 → 绝对路径（失败返回 None）
fn resolve_via_login_shell(name: &str) -> Option<String> {
    #[cfg(unix)]
    {
        // 与 sidecar cli-provider 一致：-il 才会读 .zshrc/.bashrc（nvm 常写在此处，仅 -l 不够）
        let (shell, shell_arg) = if cfg!(target_os = "macos") {
            ("/bin/zsh", "-il")
        } else {
            ("/bin/bash", "-il")
        };
        let script = format!("command -v {}", name);
        let output = Command::new(shell)
            .arg(shell_arg)
            .arg("-c")
            .arg(&script)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }
    #[cfg(windows)]
    {
        // `where` 返回 PATH 中第一个匹配的可执行文件路径
        let output = Command::new("cmd")
            .args(["/C", "where", name])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        output
            .stdout
            .split(|&b| b == b'\r' || b == b'\n')
            .next()
            .and_then(|line| {
                let s = String::from_utf8_lossy(line).trim().to_string();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            })
    }
}

/// 依次探测常见 CLI，返回每张「名字 → 路径」表（路径可为空）
fn probe_all() -> Vec<CliProbeResultDto> {
    let v: Vec<CliProbeResultDto> = PROBE_NAMES
        .iter()
        .map(|&name| CliProbeResultDto {
            name: name.to_string(),
            resolved_path: resolve_via_login_shell(name),
        })
        .collect();
    let found = v.iter().filter(|x| x.resolved_path.is_some()).count();
    log::info!(
        "CLI 探测完成：{}/{} 个候选在登录 shell 中解析到路径",
        found,
        v.len()
    );
    v
}

/// 供设置页「自动检测」：在阻塞线程中执行 shell，避免卡住 async runtime
#[tauri::command(rename_all = "camelCase")]
pub async fn probe_cli_tools() -> Result<Vec<CliProbeResultDto>, String> {
    tokio::task::spawn_blocking(probe_all)
        .await
        .map_err(|e| format!("probe_cli_tools join 失败: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_all_returns_one_row_per_candidate() {
        let v = probe_all();
        assert_eq!(v.len(), PROBE_NAMES.len());
    }
}
