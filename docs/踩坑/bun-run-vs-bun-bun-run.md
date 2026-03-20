# 踩坑：`bun run` 与 `bun --bun run` 的区别

## 问题描述

统一用 bun 管理依赖后，`bun run build`（执行 `vite build`）仍然无法生成 UnoCSS 图标 CSS。
但用 `bun --bun run build` 就正常了。

## 根因

### `node_modules/.bin/vite` 的 shebang

```bash
#!/usr/bin/env node
```

`bun run build` 的行为：
1. 读取 `package.json` 的 `scripts.build`：`vue-tsc --noEmit && vite build`
2. 执行 `vite build` → 找到 `node_modules/.bin/vite`
3. 看到 shebang `#!/usr/bin/env node` → **用系统 Node.js 执行**

所以 `bun run` 只是一个脚本运行器，实际执行 vite 的还是 Node.js v20.10.0。

### `bun --bun run` 的行为

`--bun` 标志让 bun 拦截所有子进程的 Node.js 调用，用 bun 运行时代替：
1. 读取 scripts.build
2. 执行 `vite build` → 看到 shebang `#!/usr/bin/env node`
3. **bun 拦截** → 用 bun 运行时执行（而非 Node.js）

bun v1.3.6 支持 `node:util.styleText`，所以 `@iconify/utils` 的 node-loader 正常加载。

## 验证

```bash
# 用 Node.js 测试
node -e "import { styleText } from 'node:util'"
# → 报错: SyntaxError

# 用 bun 测试
bun -e "import { styleText } from 'node:util'; console.log(typeof styleText)"
# → function
```

## Tauri 配置

```json
{
  "build": {
    "beforeDevCommand": "bun --bun run dev",
    "beforeBuildCommand": "bun --bun run build"
  }
}
```

## 教训

1. **`bun run` ≠ 用 bun 执行**：它只是替代 `npm run`，实际脚本仍由 shebang 决定运行时
2. **需要 bun 运行时兼容性时，必须加 `--bun`**
3. **注意 Tauri 的 beforeDevCommand / beforeBuildCommand**：这里配置的命令决定了前端构建的运行时环境

## 相关文件

- `apps/desktop/src-tauri/tauri.conf.json` — Tauri 构建命令配置
