//! 应用配置管理 —— 解析 ~/.xunji/config.toml，提供默认值。
//!
//! 配置文件路径优先级：
//! 1. 环境变量 XUNJI_CONFIG 指定的路径
//! 2. ~/.xunji/config.toml（默认）
//!
//! 首次启动时自动创建默认配置文件，包含 Claude Code / Cursor / CodeBuddy（CodeBuddyExtension）的默认扫描路径。
//!
//! 加载已有配置时，若缺少新版默认数据源条目，会 **自动追加**（不覆盖用户已配置的同名 id）。

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 应用顶层配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 数据采集配置
    pub collector: CollectorConfig,
    /// LLM 提炼配置
    pub distiller: DistillerConfig,
    /// 同步行为配置
    #[serde(default)]
    pub sync: SyncConfig,
}

/// 数据采集配置 —— 管理各数据源的扫描路径
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorConfig {
    /// 数据源列表（Claude Code、Cursor 等）
    pub sources: Vec<SourceConfig>,
}

/// 单个数据源配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    /// 数据源唯一标识（如 "claude-code"、"cursor"）
    pub id: String,
    /// 数据源显示名称
    pub name: String,
    /// 是否启用该数据源
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 扫描目录列表（支持 ~ 展开为用户主目录）
    pub scan_dirs: Vec<String>,
}

/// LLM 提炼配置 —— 控制 Sidecar 调用 LLM 的方式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillerConfig {
    /// 提炼模式: "api"（HTTP API）| "cli"（本地 CLI 工具）
    /// 默认为 "api"
    #[serde(default = "default_distiller_mode")]
    pub mode: String,
    /// API 模式配置（mode = "api" 时必填）
    pub api: Option<ApiConfig>,
    /// CLI 模式配置（mode = "cli" 时必填）
    pub cli: Option<CliConfig>,
}

/// 本地 CLI 工具配置（如 claude、gemini 等 AI 编程助手 CLI）
///
/// CLI 工具通过 stdin 接收提示词，从 stdout 输出响应，
/// 命令行参数与各工具官方用法保持一致（-p 非交互模式）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    /// CLI 命令名称，可自定义（如 "claude"、"claude3"、"gemini"、"codex"）
    pub command: String,
    /// 附加参数列表，追加在 -p prompt 之前（如 ["--no-history"]）
    #[serde(default)]
    pub extra_args: Vec<String>,
}

/// LLM API 连接配置（兼容 OpenAI-compatible API）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// API 提供商标识（如 "openai"、"deepseek"、"openai-compatible"）
    pub provider: String,
    /// API 基础 URL（如 "https://api.openai.com/v1"）
    pub base_url: Option<String>,
    /// API 密钥
    pub api_key: String,
    /// 模型名称（如 "gpt-4o-mini"、"deepseek-chat"）
    pub model: String,
    /// 请求超时秒数
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

/// 同步行为配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// 同步模式："manual"（手动）| "ask"（询问）| "auto"（自动）
    /// v0.1 仅支持 manual
    #[serde(default = "default_sync_mode")]
    pub mode: String,
    /// 自动同步间隔（秒），仅 auto 模式生效
    #[serde(default = "default_sync_interval")]
    pub interval_secs: u64,
}

// ── 默认值辅助函数 ──

fn default_true() -> bool {
    true
}

fn default_timeout() -> u64 {
    120
}

fn default_distiller_mode() -> String {
    "api".to_string()
}

fn default_sync_mode() -> String {
    "manual".to_string()
}

fn default_sync_interval() -> u64 {
    300
}

/// CodeBuddy 扩展数据根目录（与问渠 `scanner/codebuddy.py` 的 LOCALHOST_CANDIDATES 一致）。
/// macOS / Linux 各一条，不存在目录时在采集阶段跳过。
fn default_codebuddy_scan_dirs() -> Vec<String> {
    vec![
        "~/Library/Application Support/CodeBuddyExtension/Data".to_string(),
        "~/.local/share/CodeBuddyExtension".to_string(),
    ]
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            mode: default_sync_mode(),
            interval_secs: default_sync_interval(),
        }
    }
}

impl Default for AppConfig {
    /// 生成默认配置，包含 Claude Code 和 Cursor 的常见扫描路径
    fn default() -> Self {
        Self {
            collector: CollectorConfig {
                sources: vec![
                    SourceConfig {
                        id: "claude-code".to_string(),
                        name: "Claude Code".to_string(),
                        enabled: true,
                        scan_dirs: vec![
                            "~/.claude".to_string(),
                            "~/.claude-internal".to_string(),
                        ],
                    },
                    SourceConfig {
                        id: "cursor".to_string(),
                        name: "Cursor".to_string(),
                        enabled: true,
                        scan_dirs: vec![
                            "~/Library/Application Support/Cursor".to_string(),
                        ],
                    },
                    SourceConfig {
                        id: "codebuddy-cli".to_string(),
                        name: "CodeBuddy".to_string(),
                        // 默认关闭：未安装 CodeBuddy 的用户无需扫盘；需要时在设置中开启
                        enabled: false,
                        scan_dirs: default_codebuddy_scan_dirs(),
                    },
                ],
            },
            distiller: DistillerConfig {
                mode: default_distiller_mode(),
                api: None,
                cli: None,
            },
            sync: SyncConfig::default(),
        }
    }
}

// ── 配置加载 / 保存 ──

/// 配置相关错误
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML 解析错误: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("TOML 序列化错误: {0}")]
    Serialize(#[from] toml::ser::Error),
}

pub type ConfigResult<T> = Result<T, ConfigError>;

impl AppConfig {
    /// 获取默认配置文件路径: ~/.xunji/config.toml
    pub fn default_path() -> PathBuf {
        let home = dirs::home_dir().expect("无法获取用户主目录");
        home.join(".xunji").join("config.toml")
    }

    /// 加载配置文件。文件不存在时自动创建默认配置。
    ///
    /// 优先级：
    /// 1. 传入的 path 参数
    /// 2. 环境变量 XUNJI_CONFIG
    /// 3. ~/.xunji/config.toml
    pub fn load(path: Option<&Path>) -> ConfigResult<Self> {
        let config_path = match path {
            Some(p) => p.to_path_buf(),
            None => std::env::var("XUNJI_CONFIG")
                .map(PathBuf::from)
                .unwrap_or_else(|_| Self::default_path()),
        };

        if !config_path.exists() {
            log::info!("配置文件不存在，创建默认配置: {}", config_path.display());
            let default_config = Self::default();
            default_config.save(&config_path)?;
            return Ok(default_config);
        }

        log::info!("加载配置文件: {}", config_path.display());
        let content = fs::read_to_string(&config_path)?;
        let mut config: AppConfig = toml::from_str(&content)?;
        config.ensure_default_collector_sources();
        if config.migrate_codebuddy_scan_dirs() {
            // 将迁移结果写回磁盘，避免用户误以为仍只扫 ~/.codebuddy
            if let Err(e) = config.save(&config_path) {
                log::warn!("CodeBuddy 扫描路径迁移后写回 config.toml 失败（内存已更正）: {}", e);
            } else {
                log::info!("已将 CodeBuddy 扫描路径迁移结果写回 {}", config_path.display());
            }
        }
        log::debug!("配置加载成功: {:?}", config);
        Ok(config)
    }

    /// 若配置中尚无 **CodeBuddy CLI** 数据源，则追加默认项（升级用户无需手改 config）。
    /// 不批量注入全部内置源，避免覆盖「仅自定义源」的极简配置。
    pub fn ensure_default_collector_sources(&mut self) {
        let has_codebuddy = self
            .collector
            .sources
            .iter()
            .any(|s| s.id == "codebuddy-cli");
        if has_codebuddy {
            return;
        }
        log::info!("配置中缺少数据源「codebuddy-cli」，已追加默认项（默认关闭采集）");
        self.collector.sources.push(SourceConfig {
            id: "codebuddy-cli".to_string(),
            name: "CodeBuddy".to_string(),
            enabled: false,
            scan_dirs: default_codebuddy_scan_dirs(),
        });
    }

    /// 将旧版误配的 `~/.codebuddy` 扫描路径迁移为问渠一致的 CodeBuddyExtension 根目录。
    /// 返回 `true` 表示已修改内存中的配置（调用方可选择写回磁盘）。
    pub fn migrate_codebuddy_scan_dirs(&mut self) -> bool {
        let mut changed = false;
        for s in &mut self.collector.sources {
            if s.id != "codebuddy-cli" {
                continue;
            }
            if s.scan_dirs.len() == 1 && s.scan_dirs[0] == "~/.codebuddy" {
                log::info!(
                    "codebuddy-cli：扫描路径已从 ~/.codebuddy 更新为 CodeBuddyExtension 根目录（与问渠 scanner 一致）"
                );
                s.scan_dirs = default_codebuddy_scan_dirs();
                changed = true;
            }
        }
        changed
    }

    /// 将配置写入指定路径（自动创建父目录）
    pub fn save(&self, path: &Path) -> ConfigResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(path, &content)?;
        log::info!("配置已保存: {}", path.display());
        Ok(())
    }

    /// 获取所有已启用的数据源
    pub fn enabled_sources(&self) -> Vec<&SourceConfig> {
        self.collector
            .sources
            .iter()
            .filter(|s| s.enabled)
            .collect()
    }

    /// 根据数据源 ID 取展示名称（写入卡片冗余字段用）
    pub fn source_display_name(&self, source_id: &str) -> Option<String> {
        self.collector
            .sources
            .iter()
            .find(|s| s.id == source_id)
            .map(|s| s.name.clone())
    }

    /// 构造 Sidecar `init` 方法的 JSON-RPC params（与 TS `handleInit` 字段一致）
    ///
    /// 根据 `distiller.mode` 自动选择 API 或 CLI 参数：
    /// - "api" → 包含 provider / base_url / api_key / model
    /// - "cli" → 包含 command / extra_args
    pub fn sidecar_init_params(&self) -> Result<Value, String> {
        match self.distiller.mode.as_str() {
            "cli" => {
                let cli = self.distiller.cli.as_ref().ok_or_else(|| {
                    "未配置 distiller.cli：请在设置中填写 CLI 命令名称（如 claude）".to_string()
                })?;
                // 与 sidecar 日志对照：若此处已是短名而用户填了绝对路径，说明配置未写入或未热更新
                log::info!("sidecar init 将使用 CLI command={}", cli.command);
                Ok(serde_json::json!({
                    "mode": "cli",
                    "command": cli.command,
                    "extra_args": cli.extra_args,
                }))
            }
            _ => {
                // 默认 API 模式
                let api = self.distiller.api.as_ref().ok_or_else(|| {
                    "未配置 distiller.api：请在设置中填写 LLM API（api_key、model 等）".to_string()
                })?;
                let base_url = api
                    .base_url
                    .clone()
                    .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
                Ok(serde_json::json!({
                    "mode": "api",
                    "provider": api.provider,
                    "base_url": base_url,
                    "api_key": api.api_key,
                    "model": api.model,
                    "timeout_secs": api.timeout_secs,
                }))
            }
        }
    }
}

impl SourceConfig {
    /// 将 scan_dirs 中的 ~ 展开为用户主目录，返回绝对路径列表
    pub fn resolved_scan_dirs(&self) -> Vec<PathBuf> {
        let home = dirs::home_dir().expect("无法获取用户主目录");
        self.scan_dirs
            .iter()
            .map(|dir| {
                if dir.starts_with("~/") {
                    home.join(&dir[2..])
                } else if dir == "~" {
                    home.clone()
                } else {
                    PathBuf::from(dir)
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config_has_three_sources() {
        let config = AppConfig::default();
        assert_eq!(config.collector.sources.len(), 3);
        assert_eq!(config.collector.sources[0].id, "claude-code");
        assert_eq!(config.collector.sources[1].id, "cursor");
        assert_eq!(config.collector.sources[2].id, "codebuddy-cli");
        assert!(!config.collector.sources[2].enabled);
    }

    #[test]
    fn test_default_config_roundtrip() {
        let config = AppConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: AppConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.collector.sources.len(), config.collector.sources.len());
        assert_eq!(parsed.sync.mode, "manual");
    }

    #[test]
    fn test_load_creates_default_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        assert!(!path.exists());

        let config = AppConfig::load(Some(&path)).unwrap();
        assert!(path.exists());
        assert_eq!(config.collector.sources.len(), 3);
    }

    #[test]
    fn test_migrate_codebuddy_scan_dirs_from_old_default() {
        let mut file = NamedTempFile::new().unwrap();
        write!(
            file,
            r#"
[collector]
[[collector.sources]]
id = "codebuddy-cli"
name = "CodeBuddy"
enabled = false
scan_dirs = ["~/.codebuddy"]

[distiller]

[sync]
mode = "manual"
"#
        )
        .unwrap();

        let config = AppConfig::load(Some(file.path())).unwrap();
        let cb = config
            .collector
            .sources
            .iter()
            .find(|s| s.id == "codebuddy-cli")
            .unwrap();
        assert_eq!(cb.scan_dirs.len(), 2);
        assert!(cb.scan_dirs[0].contains("CodeBuddyExtension"));
        assert!(cb.scan_dirs[1].contains("CodeBuddyExtension"));
    }

    #[test]
    fn test_load_existing_config() {
        let mut file = NamedTempFile::new().unwrap();
        write!(
            file,
            r#"
[collector]
[[collector.sources]]
id = "test-source"
name = "Test"
enabled = true
scan_dirs = ["~/test"]

[distiller]

[sync]
mode = "manual"
interval_secs = 600
"#
        )
        .unwrap();

        let config = AppConfig::load(Some(file.path())).unwrap();
        // 升级合并：自动追加 codebuddy-cli
        assert_eq!(config.collector.sources.len(), 2);
        assert_eq!(config.collector.sources[0].id, "test-source");
        assert!(config.collector.sources.iter().any(|s| s.id == "codebuddy-cli"));
        assert_eq!(config.sync.interval_secs, 600);
    }

    #[test]
    fn test_enabled_sources_filter() {
        let mut config = AppConfig::default();
        config.collector.sources[1].enabled = false;
        let enabled = config.enabled_sources();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].id, "claude-code");
    }

    #[test]
    fn test_resolve_scan_dirs_expands_tilde() {
        let source = SourceConfig {
            id: "test".to_string(),
            name: "Test".to_string(),
            enabled: true,
            scan_dirs: vec!["~/.claude".to_string(), "/absolute/path".to_string()],
        };
        let dirs = source.resolved_scan_dirs();
        let home = dirs::home_dir().unwrap();
        assert_eq!(dirs[0], home.join(".claude"));
        assert_eq!(dirs[1], PathBuf::from("/absolute/path"));
    }

    #[test]
    fn test_config_with_api() {
        let toml_str = r#"
[collector]
[[collector.sources]]
id = "claude-code"
name = "Claude Code"
enabled = true
scan_dirs = ["~/.claude"]

[distiller]
[distiller.api]
provider = "openai-compatible"
base_url = "https://api.deepseek.com/v1"
api_key = "sk-test-key"
model = "deepseek-chat"
timeout_secs = 60

[sync]
mode = "manual"
"#;
        let config: AppConfig = toml::from_str(toml_str).unwrap();
        let api = config.distiller.api.unwrap();
        assert_eq!(api.provider, "openai-compatible");
        assert_eq!(api.model, "deepseek-chat");
        assert_eq!(api.timeout_secs, 60);
    }
}
