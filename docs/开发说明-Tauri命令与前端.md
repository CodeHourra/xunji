# Tauri 命令与桌面前端（v0.1）

## Rust：`invoke` 命令

已在 `apps/desktop/src-tauri/src/lib.rs` 注册：

| 命令 | 模块 | 说明 |
|------|------|------|
| `sync_all` | `commands/sync` | 全量采集，`spawn_blocking` 内执行 |
| `list_sessions` | `commands/sessions` | 分页列表，参数 camelCase |
| `get_session` | `commands/sessions` | `{ id }` |
| `get_session_messages` | `commands/sessions` | `{ sessionId }` |
| `distill_session` | `commands/sessions` | `{ sessionId }`，阻塞线程内跑 Sidecar + DB |
| `search_cards` | `commands/cards` | FTS 搜索 |
| `list_cards` | `commands/cards` | 分页卡片 |
| `get_card` | `commands/cards` | `{ id }` |

`generate_handler!` 必须使用 **`commands::子模块::函数` 全路径**，否则找不到 `__cmd__*` 宏展开符号。

## IPC 序列化

`storage/models.rs` 中与前端交互的结构体已加 `#[serde(rename_all = "camelCase")]`，与 `apps/desktop/src/types/index.ts` 对齐。

## `AppState`

- `Arc<Database>`、`Arc<AppConfig>`：供异步 command 中 `spawn_blocking` 克隆使用。
- `Option<Arc<SidecarManager>>`：找不到 `xunji-sidecar` 时为 `None`，提炼命令会返回明确错误。

## Capabilities

`src-tauri/capabilities/default.json` 当前仅声明 `core:default`。若运行时报「未授权调用某 command」，需在构建产物生成的 ACL 中查找对应 `allow-*` 标识符并追加到该文件的 `permissions` 数组（具体名称以本地 `cargo tauri build` / 文档为准）。

## 前端入口

- 路由：`src/router/index.ts`
- API 封装：`src/lib/tauri.ts`
- 布局：`AppLayout` + `TopBar` + `Sidebar`
- 搜索状态：`stores/search.ts`（Pinia，300ms debounce）

依赖安装请在仓库根目录执行 **`bun install`**（workspace 使用 `workspace:` 协议，部分 npm 版本不兼容）。

## 全局分析队列（Pinia）

- **Store**：`stores/analysisQueue.ts` — `enqueue` / `cancel` / `clear`，内部**串行**调用 `distill_session`（与 Sidecar 单通道一致）。
- **入口统一**：会话列表单条「分析」、批量「开始分析」、详情页「提炼笔记 / 重新分析」均通过 `enqueue` 入队。
- **UI**：`components/AnalysisQueuePanel.vue` 挂在 `AppLayout`，任意页面可见进度、耗时、「停止排队」（未执行任务取消并恢复为 `pending`）。
- **批量完成 Toast**：`SessionsView` 对批量入队的任务在 callbacks 中计数，全部结束后提示成功/失败条数。
