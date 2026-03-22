---
name: pitfall-documentation
description: 将已解决的问题整理为仓库内「踩坑」文档（问题、现象、根因、解决方法、后续预防），写入 docs/踩坑 并约定与 Cursor hooks 的配合方式。在用户完成排查/修复后、提到踩坑/复盘/经验教训、或需要更新故障文档时使用。
---

# 踩坑文档（pitfall-documentation）

## 何时写

在同时满足以下条件时 **应主动** 提出或执行文档更新（若用户明确拒绝则停止）：

- 已定位 **根因** 并完成 **可验证的修复**（或明确结论：不可修、需规避）；
- 该问题对未来自己或他人有 **重复踩坑风险**（环境、框架陷阱、隐式约定等）。

不必写：纯格式化、重命名、无教训的例行修改。

## 文档应包含什么

每条记录至少覆盖：

| 区块 | 内容 |
|------|------|
| 问题/背景 | 当时在做什么、为何卡住（一句话） |
| 现象 | 报错、日志片段、UI 表现、复现条件 |
| 根因 | 技术原因；可画 ASCII 说明调用链或数据流 |
| 解决方法 | 实际改动或操作步骤；关键路径用 \`代码路径\` 标明 |
| 后续如何避免 | 可选：规范、测试、lint、文档化、监控；确实没有则注明「暂无」或省略 |

完整 Markdown 骨架见 [reference-template.md](reference-template.md)。

## 放在哪、怎么命名

- **目录**：优先 `docs/踩坑/`（与仓库现有文档一致）。
- **新文件 vs 合并**：
  - 与已有某篇 **同一主题线**（如同一模块、同一子系统）→ **打开该文件**，在文末或合适位置 **新增小节**，并必要时更新文件顶部摘要句。
  - 新领域或主题正交 → **新建** `领域-简短主题.md`。
- **标题风格**：可参考现有踩坑文：先 `# 踩坑：…`，再分「现象 / 根因 / 解决方法」。

## Agent 执行要点

1. 写完后 **自查**：现象与根因是否对应；解决方法是否可复现；是否重复已有段落。
2. 若涉及 **model / API 字段 / 配置项**，在文档中为这些名词加简短说明（与仓库 develop-rules 一致）。
3. 适度 **打日志** 的说明可写在「解决方法」或「后续如何避免」里，便于以后排查。

## 与 Cursor Hooks 的配合（重要）

Hooks 里是 **脚本**，不会在 hook 里自动跑 AI；能做的是 **注入提醒** 或 **在 stop 时追加一条用户侧 follow-up**，由 **下一轮 Agent** 写文档。

### 推荐：sessionStart（轻量）

- **作用**：每个新 Composer 会话开始时，把「记得在合适时更新 docs/踩坑」写进 **additional context**，不额外消耗一轮对话。
- **本项目**：若已配置 `.cursor/hooks.json` 中的 `sessionStart` → `.cursor/hooks/pitfall-session-reminder.sh`，则无需重复配置。

### 慎用：stop + followup_message

- **作用**：Agent 每次结束时可由脚本返回 `followup_message`，Cursor 会 **自动再发一条用户消息** 触发新一轮 Agent，用于追问「是否要写踩坑」。
- **代价**：**每次** Agent 正常结束都可能多一轮；且 `stop` 的 stdin 仅有 `status` / `loop_count`，**脚本无法判断**「这次是否踩过坑」，只能统一追问。
- **建议**：仅在团队明确希望「强提醒」时启用；并设置合理 `loop_limit`，避免与自动 follow-up 形成预期外的循环。可参考官方文档：[Hooks](https://cursor.com/docs/agent/hooks) 中 **stop** 一节。

若启用 stop 方案，脚本需从 stdin 读 JSON，向 stdout 输出例如：`{"followup_message":"若本会话涉及根因与修复，请按 pitfall-documentation 更新 docs/踩坑。"}`（注意 JSON 转义）。

### 不推荐依赖：sessionEnd

- `sessionEnd` 为 **fire-and-forget**，响应不会被 Agent 消费；只适合落日志/审计，**不能**替代 AI 写文档。

## 示例（对话收尾）

用户：「这个问题已经修好了。」

Agent：简要复述根因与修复 → 「是否需要把本条记入 `docs/踩坑/`？」→ 若需要，则按 [reference-template.md](reference-template.md) 写入或合并。
