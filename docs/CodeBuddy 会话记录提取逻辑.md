# CodeBuddy 会话记录提取逻辑

> 本文档详细说明 CodeBuddy IDE 的本地会话记录文件结构，以及如何从中提取**项目名（project_name）**、**工作区路径（workspace_folder）** 等关键信息。  
> 适用于需要在非 Python 技术栈中复用此逻辑的场景。

---

## 1. 存储位置

CodeBuddy 的会话数据存储在以下路径：

| 平台 | 路径 |
|------|------|
| **macOS** | `~/Library/Application Support/CodeBuddyExtension/Data/{profile_id}/CodeBuddyIDE/{profile_id}/history/` |
| **Windows** | `%APPDATA%/CodeBuddyExtension/Data/{profile_id}/CodeBuddyIDE/{profile_id}/history/` |
| **Linux** | `~/.config/CodeBuddyExtension/Data/{profile_id}/CodeBuddyIDE/{profile_id}/history/` |

其中 `{profile_id}` 是一个 UUID 格式的字符串，如 `363d8e66-2fd1-4b66-9de2-072ff116f842`。

---

## 2. 目录结构

```
{history_dir}/
  └── {workspace_id}/                    # 32 位 hex hash 字符串
        ├── index.json                   # workspace 级索引（含会话列表）
        └── {conversation_id}/           # 32 位 hex 字符串
              ├── index.json             # 会话级索引（含消息列表）
              └── messages/
                    └── {msg_id}.json    # 单条消息文件
```

### 各级说明

| 层级 | 说明 | 示例 |
|------|------|------|
| `{workspace_id}/` | 一个工作区（项目），hash 标识 | `389bc7591f797f51a238cf478d48407c` |
| `{conversation_id}/` | 一次对话会话 | `3f161540dfa14ccf948c7f46b53bdc2c` |
| `messages/{msg_id}.json` | 对话中的单条消息 | `d50d0df94d114d9cbd6eab11f32853f2.json` |

---

## 3. 索引文件格式

### 3.1 Workspace 级 `index.json`

```json
{
  "version": 1,
  "conversations": [
    {
      "id": "c44ebb41a3f24265a5484b3116b9a96d",
      "name": "activity-service-php 项目分析",
      "createdAt": 1731036781123,
      "lastMessageAt": 1731036890456
    }
  ]
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `conversations[].id` | string | 会话 ID，对应子目录名 |
| `conversations[].name` | string | 会话标题（由 AI 自动生成） |
| `conversations[].createdAt` | number | 创建时间戳（毫秒） |
| `conversations[].lastMessageAt` | number | 最后消息时间戳（毫秒） |

### 3.2 Conversation 级 `index.json`

```json
{
  "version": 1,
  "messages": [
    {
      "id": "d50d0df94d114d9cbd6eab11f32853f2",
      "role": "user",
      "type": "say",
      "isComplete": true
    },
    {
      "id": "a1b2c3d4e5f6...",
      "role": "assistant",
      "type": "say",
      "isComplete": true
    }
  ]
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `messages[].id` | string | 消息 ID，对应 `messages/` 下的文件名（不含 `.json`） |
| `messages[].role` | string | `"user"` 或 `"assistant"` |
| `messages[].type` | string | 消息类型，通常是 `"say"` |
| `messages[].isComplete` | boolean | 消息是否完整 |

---

## 4. 消息文件格式（两次 JSON 解析）

消息文件 `messages/{msg_id}.json` 需要**两次 JSON 解析**：

### 第一层：文件本身

```json
{
  "role": "user",
  "message": "<JSON 字符串>",
  "id": "d50d0df94d114d9cbd6eab11f32853f2",
  "references": [],
  "extra": {}
}
```

> ⚠️ `message` 字段是一个 **JSON 字符串**（string），不是对象！需要二次 `JSON.parse()`。

### 第二层：`message` 字段解析后

```json
{
  "role": "user",
  "content": [...]
}
```

> ⚠️ `content` 在新旧版本中格式不同！详见下一节。

---

## 5. Content 字段的两种格式（新版 vs 旧版）

### 5.1 新版格式（约 2025-12 之后）

```json
{
  "content": [
    {
      "type": "text",
      "text": "<user_info>\nOS Version: darwin\nShell: zsh\nWorkspace Folder: /Users/steve/Codes/workspace/ssv/ssv-ai/wenqu\nCurrent date: Sunday, Mar 22, 2026\n</user_info>\n\n<user_query>\n你好\n</user_query>"
    }
  ]
}
```

- `content` 是**对象数组**，每个元素有 `type` 和 `text` 字段
- 工作区字段名为 `Workspace Folder`

### 5.2 旧版格式（约 2025-11 及之前）

```json
{
  "content": ["<", "u", "s", "e", "r", "_", "i", "n", "f", "o", ">", "\n", ...]
}
```

- `content` 是**单字符字符串数组**，每个元素是一个字符
- 需要 `join("")` 拼接成完整文本
- 工作区字段名为 `Workspace Path`

### 5.3 如何判断版本

```
if content[0] 是 string 且 length == 1:
    → 旧版（单字符数组）
else if content[0] 是 object 且有 type 字段:
    → 新版（block 对象数组）
```

---

## 6. 项目名提取逻辑

### 6.1 核心原理

项目名**不是**存在某个配置或元数据文件里的，而是从**每条会话的第一条 user 消息内容**中，通过正则匹配提取出来的。

CodeBuddy 在每次用户发消息时，会自动在消息中注入一段 `<user_info>` 标签，其中包含 `Workspace Folder`（或旧版的 `Workspace Path`）。

### 6.2 提取伪代码

```
function extractWorkspaceFolder(msgFilePath):
    // ═══ Step 1: 两次 JSON 解析 ═══
    raw = JSON.parse(readFile(msgFilePath))
    msgBody = JSON.parse(raw.message || "{}")

    if msgBody.role != "user":
        return null

    content = msgBody.content

    // ═══ Step 2: 从 content 提取文本（兼容两种格式）═══
    text = ""

    if content is array AND content.length > 0:
        firstElement = content[0]

        if typeof firstElement == "string" AND firstElement.length <= 1:
            // 旧版：单字符数组 → 拼接
            text = content.join("")

        else if typeof firstElement == "object" AND firstElement.type == "text":
            // 新版：block 对象数组 → 遍历取 text
            for block in content:
                if block.type == "text":
                    text += block.text

    // ═══ Step 3: 正则提取 Workspace 路径（兼容两个字段名）═══
    match = regex(text, /Workspace Folder:\s*(.+)/)
    if !match:
        match = regex(text, /Workspace Path:\s*(.+)/)

    if match:
        return match.group(1).trim()

    return null


function extractProjectName(workspaceFolder):
    if workspaceFolder:
        return basename(workspaceFolder)   // 取路径最后一级目录名
    return "CodeBuddy"                     // 默认值
```

### 6.3 遍历整个 history 的伪代码

```
function extractAllSessions(historyDir):
    results = []

    for workspaceDir in listDirs(historyDir):
        // 读取 workspace 级 index.json（可选，含会话名称等元数据）
        wsIndex = JSON.parse(readFile(workspaceDir + "/index.json"))
        convMeta = {}
        for c in wsIndex.conversations:
            convMeta[c.id] = c    // { id, name, createdAt, lastMessageAt }

        for convDir in listDirs(workspaceDir):
            // 跳过 index.json 文件
            if convDir is not directory:
                continue

            // 读取 conversation 级 index.json
            convIndex = JSON.parse(readFile(convDir + "/index.json"))
            msgList = convIndex.messages

            // 找到第一条 role="user" 的消息，提取 workspace folder
            workspaceFolder = null
            for msgMeta in msgList:
                if msgMeta.role != "user":
                    continue
                msgFile = convDir + "/messages/" + msgMeta.id + ".json"
                workspaceFolder = extractWorkspaceFolder(msgFile)
                if workspaceFolder:
                    break

            projectName = extractProjectName(workspaceFolder)
            convName = convMeta[convDir.name]?.name || ""

            results.push({
                workspaceId:     workspaceDir.name,
                conversationId:  convDir.name,
                workspaceFolder: workspaceFolder,
                projectName:     projectName,
                convName:        convName,
                createdAt:       convMeta[convDir.name]?.createdAt,
                lastMessageAt:   convMeta[convDir.name]?.lastMessageAt
            })

    return results
```

---

## 7. 新旧版本差异速查表

| 项目 | 旧版（~2025-11 及之前） | 新版（~2025-12 之后） |
|------|------------------------|---------------------|
| `content` 格式 | `["<","u","s","e","r",...]` 单字符数组 | `[{"type":"text","text":"..."}]` 对象数组 |
| `content` 处理方式 | `content.join("")` 拼接 | 遍历取 `block.text` |
| 工作区字段名 | `Workspace Path:` | `Workspace Folder:` |
| 日期格式 | `2025-11-07` | `Sunday, Mar 22, 2026` |

---

## 8. 实际数据示例

以本机实际数据为例，提取多个 workspace 的项目名：

| workspace_id（前12位） | 消息中的 Workspace 路径 | 提取出的项目名 |
|------------------------|------------------------|---------------|
| `1694a20c0947` | `/Users/steve/资料/学习资料/KM 文章/知识库` | **知识库** |
| `1f2b3bada274` | `/Users/steve/资料/学习资料/AI/AI编程资料包` | **AI编程资料包** |
| `337f7f20cfdc` | `/Users/steve/CodeBuddy/AutoPublishStory` | **AutoPublishStory** |
| `389bc7591f79` | `/Users/steve/Codes/workspace/ssv/ssv-ai/wenqu` | **wenqu** |
| `4879786f00cd` | `/Users/steve/杂七杂八/tech4good_docs` | **tech4good_docs** |
| `538ffc8b3206` | `/Users/steve/Codes/workspace/ssv/ssv-ai/统计/ssv_docs` | **ssv_docs** |

---

## 9. 注意事项

1. **两次 JSON 解析**：消息文件的 `message` 字段本身是 JSON 字符串，必须解析两次。
2. **只取第一条 user 消息**：`Workspace Folder` 通常在第一条 user 消息中注入，找到后即停止搜索。
3. **部分会话可能没有 Workspace Folder**：例如没有打开项目目录时的会话，此时 fallback 为 `"CodeBuddy"`。
4. **字符编码**：路径可能包含中文等 Unicode 字符，注意编码处理。
5. **跨平台路径**：macOS/Linux 用 `/`，Windows 用 `\`，提取 basename 时需适配。

---

## 10. 数据流总结

```
消息文件 messages/{id}.json
  │
  │  第一次 JSON.parse → raw 对象
  │
  ▼
raw.message（JSON 字符串）
  │
  │  第二次 JSON.parse → msgBody 对象
  │
  ▼
msgBody.content（数组）
  │
  │  判断 content[0] 的类型
  │  ├── string(len=1) → 旧版，join("") 拼接
  │  └── object         → 新版，取 block.text
  │
  ▼
完整文本
  │
  │  正则匹配：Workspace Folder: (.+)
  │         或：Workspace Path: (.+)
  │
  ▼
workspaceFolder = "/Users/steve/.../wenqu"
  │
  │  basename()
  │
  ▼
projectName = "wenqu"
```
