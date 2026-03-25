# 踩坑：桌面应用提炼 CLI 路径与环境变量

本文汇总在 **Tauri 桌面应用** 中使用「提炼引擎 → CLI 模式」时，与 **PATH、node、绝对路径** 相关的现象、根因与处理方式。涉及 `packages/sidecar` 的 `CliProvider` 与桌面端 `probe_cli_tools` 等实现。

---

## 一、报错里仍是短名（如 `claude-internal`），与设置里填的绝对路径不一致

### 现象

- 设置中填写了可执行文件的**绝对路径**，远程仍返回类似：`CLI 命令 "claude-internal" 未找到`。
- 或命令名拼写错误（如 `caude-internal`），报错中的名称与输入一致，属预期。

### 根因

- 错误文案里的字符串来自 sidecar 里 **`spawn` 使用的第一个参数**（即当前配置里的 `command`）。若仍是短名，说明**当时**传入提炼流程的配置未包含绝对路径（未保存、未热更新、或看错旧报错）。

### 处理与排查

1. 保存设置后查看 `~/.xunji/config.toml` 中 `[distiller.cli]` 的 `command` 是否为绝对路径。
2. 桌面端日志中搜索：`保存提炼配置 CLI: command=`、`sidecar init 将使用 CLI command=`，与界面是否一致。
3. 侧栏/sidecar 日志：`[distiller] CLI Provider initialized: command=...`。

---

## 二、从 .app / 桌面图标启动时：命令「在终端能用」，在 App 里报未找到（ENOENT）

### 现象

- 终端执行 `claude-internal` 正常，在寻迹里提炼报 **CLI 未找到** 或 **spawn ENOENT**。

### 根因

- 桌面应用继承的 **PATH 往往远短于登录/交互终端**（不含 Homebrew、nvm、`~/.local/bin` 等）。
- 子进程 `spawn(command, …)` 在 **当前进程的 PATH** 里解析可执行文件名；PATH 不全则找不到同名命令。

### 解决方法（已实现）

- Sidecar：`envForCliSpawn()` 中合并 **macOS 登录/交互 shell** 下的 PATH，并增加常见前缀目录（如 `/opt/homebrew/bin`、`~/.bun/bin` 等）。
- 设置页提供 **「自动检测」**：Rust 命令 `probe_cli_tools`，在 `zsh -il` / `bash -il` 下执行 `command -v`，按固定顺序取**第一个解析到的绝对路径**写入配置，减少手填路径。

### 探测顺序（可随版本调整）

当前顺序为：`claude-internal` → `gemini-internal` → `codex-internal` → `claude` → `gemini` → `codex`。

---

## 三、`env: node: No such file or directory`

### 现象

- 使用绝对路径指向 **npm 全局脚本**（shebang 常为 `#!/usr/bin/env node`）时，stderr 出现 **`env: node: No such file or directory`**。

### 根因

- 内核先执行 `/usr/bin/env`，由它在 **子进程 PATH** 中查找 `node`。
- **nvm** 等常写在 **`~/.zshrc`**，仅 **`zsh -l`（登录非交互）不会加载 .zshrc**，导致合并后的 PATH 仍无 `node`。

### 解决方法（已实现）

- 使用 **`zsh -il` / `bash -il`** 拉取与日常终端更接近的 PATH。
- 额外执行 **`command -v node`**，将 **node 所在目录** 前置到 PATH。
- 增加常见 **Homebrew node** 路径前缀（如 `/opt/homebrew/opt/node/bin`）。

---

## 四、`Node.js executable for command 'node' could not be found`（含 ServBay / NVM 提示）

### 现象

- 报错来自 **第三方 CLI 自带逻辑**（非寻迹仓库字符串），文案中可能包含 **ServBay、NVM_BIN、Homebrew** 等，说明其内部自行解析 Node 可执行文件。

### 根因

- 仅扩充 **PATH** 或只保证 `env` 能找到 `node` 仍不够：这类工具常读取 **`NVM_DIR`、`NVM_BIN`、ServBay 相关变量** 等；桌面进程默认 **不带** 交互终端里的这些变量。

### 解决方法（已实现）

- 在 **仅用于 `spawn(用户 CLI)` 的环境** 中，执行 **`command env`**（经 `zsh -il` / `bash -il`）解析**整份环境变量**，与当前 `process.env` 合并后再传入子进程。
- **失败时**回退为仅合并 PATH（与早期行为一致）。

---

## 五、合并「整份终端 env」会不会搞坏寻迹自己？

### 结论：**不会**

| 对象 | 说明 |
|------|------|
| **合并结果作用域** | 仅作为 `child_process.spawn({ env })` 传入 **外部 CLI 子进程**，**不** `Object.assign(process.env, …)`，**不** 改写 sidecar 进程自身的全局环境。 |
| **Tauri 主进程** | Rust，不消费这份 env。 |
| **Sidecar 本体** | 一般为 **Bun 编译产物**，运行不依赖用户机器上的 `node` 版本。 |
| **用户 node 版本** | 只影响被拉起的 **第三方 CLI**；与寻迹内置运行时无关，属于预期隔离。 |

代码注释位置：`packages/sidecar/src/distiller/cli-provider.ts` 中 `envForCliSpawn` 的说明块。

---

## 六、排查清单（建议顺序）

1. **设置是否已保存**，`config.toml` 中 `command` 是否为预期（短名或绝对路径）。
2. 点 **「自动检测」**，看是否写入与终端 `command -v` 一致的绝对路径。
3. 若仍报 **node / ServBay / NVM**：在**同一台机器**的终端里执行与寻迹相同的 CLI，确认 `node -v` 与 ServBay/NVM 项目配置正常。
4. 重新 **构建 sidecar**（修改 env 逻辑后需更新 `xunji-sidecar` 二进制），再跑桌面端提炼。

---

## 相关代码位置（便于对照）

| 模块 | 路径 |
|------|------|
| CLI 子进程 env、PATH、交互 shell `env` | `packages/sidecar/src/distiller/cli-provider.ts` |
| init 参数与日志 | `apps/desktop/src-tauri/src/config.rs`（`sidecar_init_params`）、`packages/sidecar/src/distiller/index.ts`（`handleInit`） |
| 自动探测命令 | `apps/desktop/src-tauri/src/commands/cli_probe.rs` |
| 设置 UI | `apps/desktop/src/components/SettingsModal.vue` |

---

## 版本说明

行为随实现迭代可能微调；若与当前代码不一致，以仓库内实现为准。
