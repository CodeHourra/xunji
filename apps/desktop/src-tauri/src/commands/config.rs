//! 配置读取与保存命令 —— 支持从前端设置页热更新 config.toml。
//!
//! ```text
//! get_config  → 返回当前 AppConfig（JSON，snake_case）
//! save_config → 写入磁盘 + 更新内存中的 Arc<RwLock<AppConfig>>
//! ```

use tauri::State;

use crate::config::{ApiConfig, AppConfig, CliConfig, CollectorConfig, DistillerConfig, SourceConfig, SyncConfig};
use crate::AppState;

// ─── IPC DTO ────────────────────────────────────────────────────────────────
//
// 与前端 TypeScript 接口对应，字段使用 camelCase（通过 rename_all）。
// 独立于 AppConfig（AppConfig 使用 snake_case 以兼容 TOML 格式）。

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfigDto {
    pub distiller: DistillerConfigDto,
    pub collector: CollectorConfigDto,
    pub sync: SyncConfigDto,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DistillerConfigDto {
    /// 提炼模式: "api" | "cli"
    pub mode: String,
    pub api: Option<ApiConfigDto>,
    pub cli: Option<CliConfigDto>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiConfigDto {
    /// 提供商标识（如 "openai"、"deepseek"、"openai-compatible"）
    pub provider: String,
    /// API Base URL（可选，为空时使用默认值）
    pub base_url: Option<String>,
    /// API 密钥
    pub api_key: String,
    /// 模型名称（如 "gpt-4o-mini"、"deepseek-chat"）
    pub model: String,
    /// 请求超时（秒）
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CliConfigDto {
    /// CLI 命令名（如 "claude"、"gemini"、"codex"）
    pub command: String,
    /// 附加参数
    pub extra_args: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectorConfigDto {
    pub sources: Vec<SourceConfigDto>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceConfigDto {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub scan_dirs: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncConfigDto {
    pub mode: String,
    pub interval_secs: u64,
}

// ─── DTO 转换 ────────────────────────────────────────────────────────────────

impl From<&AppConfig> for AppConfigDto {
    fn from(c: &AppConfig) -> Self {
        AppConfigDto {
            distiller: DistillerConfigDto {
                mode: c.distiller.mode.clone(),
                api: c.distiller.api.as_ref().map(|a| ApiConfigDto {
                    provider: a.provider.clone(),
                    base_url: a.base_url.clone(),
                    api_key: a.api_key.clone(),
                    model: a.model.clone(),
                    timeout_secs: a.timeout_secs,
                }),
                cli: c.distiller.cli.as_ref().map(|cl| CliConfigDto {
                    command: cl.command.clone(),
                    extra_args: cl.extra_args.clone(),
                }),
            },
            collector: CollectorConfigDto {
                sources: c
                    .collector
                    .sources
                    .iter()
                    .map(|s| SourceConfigDto {
                        id: s.id.clone(),
                        name: s.name.clone(),
                        enabled: s.enabled,
                        scan_dirs: s.scan_dirs.clone(),
                    })
                    .collect(),
            },
            sync: SyncConfigDto {
                mode: c.sync.mode.clone(),
                interval_secs: c.sync.interval_secs,
            },
        }
    }
}

impl From<AppConfigDto> for AppConfig {
    fn from(dto: AppConfigDto) -> Self {
        AppConfig {
            distiller: DistillerConfig {
                mode: dto.distiller.mode,
                api: dto.distiller.api.map(|a| ApiConfig {
                    provider: a.provider,
                    base_url: a.base_url,
                    api_key: a.api_key,
                    model: a.model,
                    timeout_secs: a.timeout_secs,
                }),
                cli: dto.distiller.cli.map(|cl| CliConfig {
                    command: cl.command,
                    extra_args: cl.extra_args,
                }),
            },
            collector: CollectorConfig {
                sources: dto
                    .collector
                    .sources
                    .into_iter()
                    .map(|s| SourceConfig {
                        id: s.id,
                        name: s.name,
                        enabled: s.enabled,
                        scan_dirs: s.scan_dirs,
                    })
                    .collect(),
            },
            sync: SyncConfig {
                mode: dto.sync.mode,
                interval_secs: dto.sync.interval_secs,
            },
        }
    }
}

// ─── Tauri 命令 ──────────────────────────────────────────────────────────────

/// 读取当前配置并返回给前端。
#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfigDto, String> {
    let config = state.config_snapshot();
    Ok(AppConfigDto::from(&config))
}

/// 接收前端配置、写入磁盘并更新内存中的 AppConfig（热更新，无需重启）。
#[tauri::command]
pub async fn save_config(
    state: State<'_, AppState>,
    config: AppConfigDto,
) -> Result<(), String> {
    let new_config = AppConfig::from(config);

    // 先写磁盘，成功后再更新内存（保证两者一致）
    let path = AppConfig::default_path();
    new_config.save(&path).map_err(|e| format!("配置保存失败: {}", e))?;

    // 写内存（写锁）
    let mut guard = state
        .config
        .write()
        .map_err(|_| "配置锁异常，请重启应用".to_string())?;
    *guard = new_config;
    drop(guard);

    log::info!("配置已热更新");
    Ok(())
}
