mod commands;
mod collector;
mod config;
mod sidecar;
mod storage;

use storage::Database;

pub struct AppState {
    pub db: Database,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 默认 info 级别，可通过 RUST_LOG 环境变量覆盖（如 RUST_LOG=debug）
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let db = Database::open_default().expect("数据库初始化失败");

    let state = AppState { db };

    // Tauri 的 manage() 内部使用 Arc 包装，无需额外 Arc
    tauri::Builder::default()
        .manage(state)
        .run(tauri::generate_context!())
        .expect("Tauri 应用启动失败");
}
