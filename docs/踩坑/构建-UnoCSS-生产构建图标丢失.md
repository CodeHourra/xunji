# 问题记录：Tauri 构建后 UnoCSS 图标全部丢失

## 问题描述

Tauri 桌面应用中所有 `i-lucide-*` 图标无法显示，图标位置呈现空白。
UnoCSS 的工具类（如 `flex`、`text-sm`）正常生效，唯独图标类没有生成对应的 CSS。

## 环境信息

| 项目 | 值 |
|------|----|
| Node.js | v20.10.0 |
| bun | v1.3.6 |
| @unocss/preset-icons | 66.6.7 |
| @iconify/utils | 3.1.0 |
| @iconify-json/lucide | 1.2.98 |
| Vite | 6.x |
| Tauri | 2.0 |

## 根因分析

### 调查过程

1. **确认现象**：构建产物 CSS 中有 0 条 `i-lucide-*` 规则，而源码中有 30+ 处引用
2. **验证依赖链**：
   - `@iconify-json/lucide`（图标数据）- 已安装，1744 个图标
   - `@unocss/preset-icons`（图标预设）- 已安装
   - `@iconify/utils`（图标加载核心库）- 初始缺失（混合包管理器导致），后安装仍不工作
3. **深入定位**：直接调用 preset-icons 的 handler，确认返回 `undefined`
4. **追溯到 node-loader**：preset-icons 通过 `@iconify/utils/lib/loader/node-loader` 加载图标
5. **最终根因**：`node-loader` 模块依赖 `node:util.styleText` API，该 API 在 **Node.js v20.12+** 才引入

### 根因总结

```
@unocss/preset-icons@66.6.7
  └─ @iconify/utils@3.1.0
       └─ lib/loader/node-loader.js
            └─ import { styleText } from 'node:util'  ← Node.js v20.10 不支持！
```

`@iconify/utils@3.1.0` 使用了 `node:util.styleText`（Node.js v20.12+ 新增 API），
当 Node.js 版本低于 v20.12 时，icon loader 模块无法加载，preset-icons 静默失败，
所有图标 CSS 规则不会生成。

### 额外发现

项目根目录同时存在 `bun.lock`、`package-lock.json`、`pnpm-lock.yaml` 三种锁文件，
导致依赖安装混乱——`@iconify/utils` 最初根本没有被安装到正确位置。

## 修复方案

### 已执行的修复

1. **清理冲突的锁文件**：删除 `package-lock.json` 和 `pnpm-lock.yaml`，统一使用 bun
2. **重装依赖**：`bun install`
3. **修改 Tauri 构建命令**：使用 `bun --bun` 强制 bun 运行时

```json
// src-tauri/tauri.conf.json
{
  "build": {
    "beforeDevCommand": "bun --bun run dev",
    "beforeBuildCommand": "bun --bun run build"
  }
}
```

**关键点**：`bun --bun` 会让 bun 代替 Node.js 作为 JavaScript 运行时执行 Vite，
而 bun v1.3.6 已支持 `node:util.styleText`，因此图标加载正常。

### 替代修复方案

如果不想依赖 bun 运行时，也可以升级 Node.js：

```bash
# 升级到 Node.js v20.12+ 或 v22 LTS
# 使用 nvm/fnm/volta 等版本管理器
nvm install 22
nvm use 22
```

## 验证方法

```bash
# 构建后检查 CSS 中是否包含图标数据
grep -c 'un-icon' dist/assets/*.css
# 应输出大于 0 的数字

# 检查图标 SVG 数据
grep -o 'i-lucide-[a-z-]*' dist/assets/*.css | sort -u | wc -l
# 应输出与源码中引用数量一致的数字
```

## 教训

1. **统一包管理器**：monorepo 中混用 npm/bun/pnpm 会导致依赖链断裂
2. **注意 Node.js 版本兼容性**：新版第三方库可能使用新的 Node.js API
3. **UnoCSS 图标失败是静默的**：preset-icons 加载失败时不会报错，只是不生成 CSS，排查困难
4. **`bun run` vs `bun --bun run`**：前者仍然通过 Node.js 执行带 `#!/usr/bin/env node` 的脚本，后者才真正使用 bun 运行时

## 相关文件

- `apps/desktop/uno.config.ts` - UnoCSS 配置
- `apps/desktop/vite.config.ts` - Vite 构建配置
- `apps/desktop/src-tauri/tauri.conf.json` - Tauri 构建命令配置
- `apps/desktop/package.json` - 前端依赖声明
