# 踩坑：CLI 模式 LLM 返回 JSON 解析失败

## 问题描述

使用 CLI Provider（`claude-internal -p`）调用 LLM 提炼笔记时，`distill_full` 返回错误：

```
JSON Parse error: Unrecognized token '\'
```

API Provider（HTTP 调用）不受影响。

## 根因

### LLM 返回的 JSON 中 note 字段含大段 Markdown

Prompt 要求 LLM 返回如下 JSON：

```json
{
  "title": "...",
  "note": "完整的 Markdown 笔记...",
  ...
}
```

`note` 字段可能包含：
- 原始换行符（非 `\n` 转义）
- 反斜杠 `\`（如文件路径 `C:\Users\...`）
- 未转义的引号
- Markdown 代码块中的特殊字符

### CLI vs API 的差异

| | API Provider | CLI Provider |
|---|---|---|
| 传输方式 | HTTP JSON response | stdout 文本 |
| JSON 格式 | 模型内部生成，通常合法 | 模型输出文本，可能有格式问题 |
| 多角色支持 | system + user 分开 | 合并为一条 prompt |

CLI 模式下 LLM 的 JSON 输出更容易出现格式问题，因为：
1. 没有 API 层的 JSON mode 强制约束
2. system prompt 和 content 合并后上下文更长，模型更容易出错
3. stdout 可能有编码/转义差异

### 原始 extractJson 太脆弱

```typescript
function extractJson(text: string): unknown {
  // 只是简单的 JSON.parse，没有任何容错
  return JSON.parse(cleaned)
}
```

## 修复方案

`extractJson` 采用 4 级渐进修复策略：

```
1. 直接 JSON.parse（大多数情况）
      ↓ 失败
2. 提取最外层 { ... }（LLM 在 JSON 前后加了文字）
      ↓ 失败
3. 修复 note/summary 等字段中未转义的 \n、\、" 等字符
      ↓ 失败
4. 去除控制字符后重试
      ↓ 失败
5. 抛出包含响应预览的详细错误（方便排查）
```

## Prompt 设计建议

为减少 JSON 解析失败的概率，提示词中应强调：
- "只返回纯 JSON，不含其他内容"
- "note 字段中的换行用 `\\n` 表示"
- 考虑使用更短的 note 格式（如 summary only）做第一版，稳定后再加长文本

## 教训

1. **CLI 模式的 LLM 输出比 API 更不可控**：没有 JSON mode 约束，容错必须更强
2. **JSON 中嵌入长文本是高风险操作**：Markdown 内容充满特殊字符，LLM 很难完美转义
3. **渐进式解析优于一刀切**：先试严格解析，失败后逐步放宽修复，比一开始就做激进替换更安全
4. **错误信息要包含上下文**：抛出 JSON 解析错误时附带响应预览，极大加速排查

## 相关文件

- `packages/sidecar/src/distiller/index.ts` — extractJson() 函数
- `packages/sidecar/src/distiller/cli-provider.ts` — CLI 调用逻辑
- `packages/sidecar/src/distiller/prompts.ts` — Prompt 模板
