# 踩坑：Claude API 协议中 tool_result 消息的 role 识别

## 问题描述

会话详情的聊天回放中，工具执行结果（如 `[Tool Result: AGENTS.md blueprints ...]`）被渲染为用户输入的蓝色气泡，与真正的用户消息混在一起，导致对话流难以阅读。

## 根因

Claude API 协议中，工具执行结果的消息结构为：

```json
{
  "type": "user",
  "message": {
    "role": "user",
    "content": [
      { "type": "tool_result", "content": "AGENTS.md\nblueprints\n创作进度\n正文" }
    ]
  }
}
```

**`type` 和 `role` 都是 `"user"`**，但它不是用户输入——而是 Claude 调用工具后，系统自动拼装的返回值回送给模型的消息。

采集器（`claude_code.rs`）直接使用 `message.role` 作为消息角色，没有区分 tool_result 内容。

## 修复方案

**后端（采集器）**：在 `extract_content` 中检测 content 数组是否全部由 `tool_result` 组成（无 `text` 块），是则将 role 标记为 `"tool"`。

```rust
let is_tool_result = !has_text && has_tool_result;
// → role 改为 "tool"
```

**前端（ChatReplay.vue）**：通过 `messageType()` 函数统一判断消息类型，兼容新旧数据：

```typescript
function messageType(role, content): 'tool-result' | 'tool-use' | 'bubble' {
  // ① role='tool' → 新数据，采集器已标记
  // ② role='user' 且内容以 [Tool Result: 开头 → 旧数据兼容
  // ③ role='assistant' 且去掉 [Tool: xxx] 后为空 → 工具调用摘要
  // ④ 其他 → 正常气泡
}
```

## 教训

1. **不能信任 API 协议的 role 字段做 UI 展示**：tool_result 的 role 是 "user"，但语义上不是用户输入
2. **旧数据兼容很重要**：改了采集器只影响新同步的数据，已入库的旧数据需要前端兜底识别
3. **内容模式匹配比 role 字段更可靠**：`startsWith('[Tool Result:')` 可以同时覆盖新旧数据

## 相关文件

- `apps/desktop/src-tauri/src/collector/claude_code.rs` — 采集器 role 标记
- `apps/desktop/src/components/ChatReplay.vue` — 前端消息类型判断与渲染
