mod commands;
mod collector;
mod config;
mod sidecar;
mod storage;

use std::sync::Arc;
use storage::Database;

pub struct AppState {
    pub db: Arc<Database>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    let db = Database::open_default().expect("failed to initialize database");

    let state = AppState { db: Arc::new(db) };

    tauri::Builder::default()
        .manage(state)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
