---
name: 提交代码
description: "按功能维度渐进式分批提交代码"
argument-hint: "[--dry-run | --auto | --interactive]"
---

# 渐进式分批提交命令

按功能维度将代码变更分组，逐批生成符合规范的 commit message 并提交。

## 使用方式

- `--dry-run`: 仅分析变更并展示分批计划，不执行提交
- `--auto`: 自动按分析结果分批提交（需用户确认每批）
- `--interactive`: 交互式模式，每批提交前可修改 commit message
- 无参数: 默认进入 `--interactive` 模式

## 执行流程

### 1. 分析变更

首先执行 `git status` 和 `git diff --stat` 获取所有变更文件，然后对每个变更文件执行 `git diff <file>` 分析具体改动内容。

### 2. 功能分组

根据变更内容按以下维度进行功能分组：

- **同一功能模块**: 前后端协同的改动归为一组（如 API 字段变更涉及的 model/service/handler/前端 api/store/view）
- **文档更新**: 文档更新和功能变化一组
- **独立功能点**: 单一功能的完整实现归为一组
- **UI/UX 优化**: 界面样式、交互优化归为一组
- **Bug 修复**: 单个 bug 的修复归为独立一组
- **重构/优化**: 代码重构、性能优化归为一组
- **配置变更**: 配置文件修改单独一组（通常不提交本地开发配置）


### 3. 生成 Commit Message

每批提交使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Type 类型**:
- `feat`: 新功能
- `fix`: Bug 修复
- `refactor`: 代码重构（不改变功能）
- `style`: 样式调整（不影响代码逻辑）
- `perf`: 性能优化
- `docs`: 文档更新
- `chore`: 构建/工具/配置变更
- `test`: 测试相关

**Scope 范围** (可选): `server`, `web`, `api`, `ws`, `auth`, `friend`, `message`, `group`, `ui` 等

**Subject 规则**:
- 使用中文描述
- 不超过 50 个字符
- 以动词开头（添加、修复、优化、重构等）
- 不以句号结尾

**Body 规则**:
- 详细描述改动内容
- 解释为什么做这个改动
- 列出影响的文件/模块

### 4. 执行提交

对每批变更执行：

```bash
# 添加该批次的文件
git add <files...>

# 提交
git commit -m "<commit message>"
```

### 5. 跳过规则

以下变更默认跳过，不纳入提交：
- 本地开发配置文件（如 `config-local.yaml`、`.env.local`）
- IDE 配置文件（如 `.idea/`、`.vscode/`）
- 构建产物（如 `dist/`、`build/`、`node_modules/`）
- 临时/调试代码

## 示例输出

```
📊 变更分析完成，共检测到 12 个文件变更

📦 分批计划 (4 批):

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🔹 批次 1: 好友请求消息字段重构
   类型: refactor(friend)
   文件:
   - server/internal/model/friendship.go
   - server/internal/service/friend.go
   - web/src/api/index.js
   - web/src/stores/friend.js
   - web/src/views/contacts/SearchUser.vue

   Commit Message:
   refactor(friend): 将好友请求的 remark 字段重命名为 message

   - 后端 Friendship model 新增 message 字段用于好友请求验证消息
   - remark 字段保留用于好友备注功能
   - 更新前后端 API 参数命名保持一致

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🔹 批次 2: 会话未读数逻辑修复
   类型: fix(message)
   文件:
   - web/src/stores/conversation.js
   - web/src/stores/ws.js

   Commit Message:
   fix(message): 修复新会话未读数计算逻辑

   - 区分自己发送和收到消息的未读数处理
   - 自己发送的消息不增加未读数
   - 收到的消息在新会话时正确增加未读数

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🔹 批次 3: 移动端适配优化
   类型: style(ui)
   文件:
   - web/index.html
   - web/src/views/chat/components/ChatWindow.vue
   - web/src/views/chat/components/MessageInput.vue

   Commit Message:
   style(ui): 优化移动端聊天界面适配

   - 添加聊天窗口返回按钮支持
   - 禁用页面缩放提升移动端体验
   - 修复输入框被底部导航栏遮挡问题

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🔹 批次 4: 用户资料页增强
   类型: feat(profile)
   文件:
   - web/src/views/profile/Index.vue

   Commit Message:
   feat(profile): 添加昵称和用户名一键复制功能

   - 支持点击昵称/用户名复制到剪贴板
   - 添加复制图标和悬停动画效果

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

⚠️ 跳过的文件:
   - server/internal/config/config.go (本地开发配置)

是否开始提交? [Y/n]
```

## 注意事项

1. **原子性**: 每批提交应该是一个完整的功能单元，可独立回滚
2. **关联性**: 前后端配合的改动应放在同一批次
3. **可读性**: Commit message 应清晰描述改动目的
4. **可追溯**: Body 中列出影响范围便于后续排查
