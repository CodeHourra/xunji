# 踩坑：Monorepo 混用包管理器导致依赖链断裂

## 问题描述

项目根目录同时存在 `bun.lock`、`package-lock.json`、`pnpm-lock.yaml` 三种锁文件，
导致 `@iconify/utils` 等传递依赖未被正确安装，UnoCSS 图标全部丢失。

## 根因

### 三种包管理器并存

| 锁文件 | 来源 |
|--------|------|
| `bun.lock` | 根目录用 bun 初始化 |
| `package-lock.json` | 某次用 npm install |
| `pnpm-lock.yaml` | apps/desktop 中用 pnpm install |

### 依赖解析冲突

bun 的 workspace hoisting 策略与 npm/pnpm 不同：
- bun 将依赖放在 `node_modules/.bun/` 下通过符号链接引用
- npm 直接平铺在 `node_modules/`
- pnpm 使用 `node_modules/.pnpm/` 的嵌套结构

三者并存时，某些传递依赖（如 `@iconify/utils`）可能只存在于旧的 `.old_modules-*` 备份目录中，
无法被正确解析。

## 修复方案

```bash
# 1. 删除冲突的锁文件，只保留一种
rm package-lock.json
rm apps/desktop/pnpm-lock.yaml

# 2. 清理所有 node_modules
rm -rf node_modules apps/desktop/node_modules

# 3. 统一用 bun 重装
bun install
```

## 教训

1. **monorepo 必须统一包管理器**：在 `package.json` 中加 `"packageManager"` 字段锁定
2. **锁文件入 .gitignore 要谨慎**：保留一种，其余锁文件可以加入 `.gitignore`
3. **CI 中强制校验包管理器**：`only-allow bun` 之类的 preinstall 脚本

### 推荐配置

```json
// package.json
{
  "packageManager": "bun@1.3.6",
  "scripts": {
    "preinstall": "npx only-allow bun"
  }
}
```

## 相关文件

- `package.json` — 根 workspace 配置
- `bun.lock` — 唯一保留的锁文件
