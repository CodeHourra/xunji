mod commands;
mod collector;
pub mod config;
mod sidecar;
mod storage;

use std::sync::{Arc, RwLock};

use config::AppConfig;
use sidecar::SidecarManager;
use storage::Database;

/// 应用全局状态，由 Tauri manage() 注入，各 command 通过 State<AppState> 访问
///
/// ```text
/// Arc<Database>              —— 供 spawn_blocking 中克隆使用
/// Arc<RwLock<AppConfig>>     —— 支持运行时热更新（设置页保存配置后无需重启）
/// Option<Arc<SidecarManager>> —— 未找到 sidecar 二进制时为 None
/// ```
pub struct AppState {
    pub db: Arc<Database>,
    pub config: Arc<RwLock<AppConfig>>,
    pub sidecar: Option<Arc<SidecarManager>>,
}

impl AppState {
    /// 获取当前配置快照（读锁，Clone 出一份用于 spawn_blocking）
    pub fn config_snapshot(&self) -> AppConfig {
        self.config.read().expect("config RwLock poisoned").clone()
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 默认 info 级别，可通过 RUST_LOG 环境变量覆盖（如 RUST_LOG=debug）
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = AppConfig::load(None).expect("配置加载失败");
    log::info!(
        "已加载 {} 个数据源，{} 个已启用，提炼模式: {}",
        config.collector.sources.len(),
        config.enabled_sources().len(),
        config.distiller.mode,
    );

    let db = Database::open_default().expect("数据库初始化失败");

    let sidecar = SidecarManager::find_binary().map(|path| {
        Arc::new(SidecarManager::new(path))
    });

    if sidecar.is_none() {
        log::warn!("Sidecar 未就绪：提炼功能将不可用，直至构建或安装 xunji-sidecar");
    }

    let state = AppState {
        db: Arc::new(db),
        config: Arc::new(RwLock::new(config)),
        sidecar,
    };

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::sync::sync_all,
            commands::sessions::list_sessions,
            commands::sessions::get_session,
            commands::sessions::get_session_messages,
            commands::sessions::distill_session,
            commands::cards::search_cards,
            commands::cards::list_cards,
            commands::cards::get_card,
            commands::sidebar::get_session_groups,
            commands::sidebar::list_tags,
            commands::sidebar::list_card_types,
            commands::config::get_config,
            commands::config::save_config,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 应用启动失败");
}
