//! 同步相关 Tauri 命令 —— 触发采集调度器，阻塞 I/O 放在 `spawn_blocking` 中执行。

use tauri::State;

use crate::collector::scheduler::CollectorScheduler;
use crate::collector::scheduler::SyncResult;
use crate::AppState;

/// 手动全量同步：扫描已启用数据源并写入数据库。
#[tauri::command]
pub async fn sync_all(state: State<'_, AppState>) -> Result<SyncResult, String> {
    let db = state.db.clone();
    let config = state.config.clone();

    tokio::task::spawn_blocking(move || {
        let scheduler = CollectorScheduler::new(config.as_ref(), db.as_ref());
        scheduler.collect_all()
    })
    .await
    .map_err(|e| format!("同步任务 join 失败: {}", e))
}
