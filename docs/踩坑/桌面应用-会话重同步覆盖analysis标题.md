# 踩坑：增量重同步把 `sessions.analysis_title` 写成 NULL

## 问题/背景

会话经「低/无价值」分析后，由 `update_session_analysis_meta` 写入 `analysis_title` 等字段；随后若该会话在 Claude Code / Cursor 侧有新消息并触发采集重同步，列表里标题会消失。

## 现象

- 分析完成后列表有展示标题，同步一次后标题变空或恢复为默认逻辑。
- 数据库中该行的 `analysis_title` 变为 `NULL`。

## 根因

```text
dedup_and_write → message_count 增加
  → update_session_resync_metadata(..., analysis_title = session.analysis_title)
       NormalizedSession.analysis_title 对 claude-code / cursor 恒为 None
  → SQL: UPDATE ... SET analysis_title = ?4  绑定 NULL
       → 覆盖掉此前 analysis 步骤写入的标题
```

原先仅调用 `update_session_message_count` 时不会碰 `analysis_title`，引入「重同步元数据」后未区分「采集未提供」与「显式清空」。

## 解决方法

在 `update_session_resync_metadata` 的 `UPDATE` 中使用：

`analysis_title = COALESCE(?4, analysis_title)`

- 采集端传入非 NULL（如 CodeBuddy 工作区 index 中的名称）时仍更新；
- 传入 NULL 时保留库内已有 `analysis_title`。

实现位置：`apps/desktop/src-tauri/src/storage/sessions.rs` 中 `update_session_resync_metadata`。

## 后续如何避免

- 凡「可选字段」若语义为「未提供则不改」，在 SQL 层用 `COALESCE(新值, 列)` 或拆成两条 `UPDATE`，避免把 `NULL` 绑进无条件 `SET col = ?`。
- 回归：`cargo test resync_metadata`（`storage::sessions` 内单元测试）；或手工先跑分析写入标题，再触发该会话增量同步，确认标题仍在。
