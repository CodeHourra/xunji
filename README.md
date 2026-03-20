# 寻迹（XunJi）

> AI 编程知识管理平台 —— 从 AI 对话中提炼可复用的技术知识

## 简介

寻迹自动采集你与 AI 编程助手（Claude Code、Cursor 等）的对话记录，通过 LLM 提炼出有价值的技术笔记、最佳实践和编程技巧，构建个人知识库。

## 技术栈

- **桌面框架**: Tauri 2.0
- **前端**: Vue 3 + TypeScript + UnoCSS + Radix Vue + Pinia
- **后端**: Rust (rusqlite, serde, tokio)
- **LLM Sidecar**: TypeScript + Bun (OpenAI-compatible API)
- **MCP Server**: TypeScript + Bun (stdio transport)

## 项目结构

```
xunji/
├── apps/desktop/          # Tauri 桌面应用
│   ├── src-tauri/         # Rust 后端
│   └── src/               # Vue 3 前端
├── packages/
│   ├── sidecar/           # TS Sidecar（LLM 调用 + 知识提炼）
│   ├── mcp-server/        # MCP Server（AI IDE 集成）
│   └── shared/            # 共享类型和工具
└── docs/                  # 文档
```

## 开发

```bash
# 安装依赖
npm install

# 启动开发模式
cd apps/desktop
npm run tauri dev

# 构建 sidecar
cd packages/sidecar
bun build src/index.ts --compile --outfile dist/xunji-sidecar
```

## 许可证

MIT
