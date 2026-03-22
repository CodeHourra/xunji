# macOS 打包说明（寻迹 / Tauri）

本文说明如何在 **Apple Silicon（aarch64）** 或 **Intel（x86_64）** 的 macOS 上从源码打出可安装的 `.app` / `.dmg`。

## 环境要求

| 工具 | 说明 |
|------|------|
| **Xcode Command Line Tools** | `xcode-select --install` |
| **Rust** | 建议 [rustup](https://rustup.rs/)，`rustc` 能正常编译即可 |
| **Bun** | 用于安装 workspace 依赖、执行 `tauri.conf.json` 里的 `beforeBuildCommand`、编译 sidecar |
| **Node**（可选） | 若不用 Bun，需自行保证能安装 monorepo 依赖 |

版本号不锁死，与仓库当前 `package.json` / `Cargo.toml` 兼容即可。

## 一键流程（在项目根目录 `xunji/`）

```bash
# 1. 安装前端与 workspace 依赖
bun install

# 2. 编译 LLM Sidecar（提炼等功能依赖该二进制）
cd packages/sidecar && bun run build && cd ../..

# 3. 打 macOS 安装包
cd apps/desktop && bun run tauri:build
```

第 3 步等价于在 **清除 `CI` 环境变量** 的前提下执行 `tauri build`（部分环境 `CI=1` 会导致 Tauri CLI 报错，已用脚本规避）。

## 产物位置

构建成功后，在：

`apps/desktop/src-tauri/target/release/bundle/`

- **macOS 应用包**：`macos/寻迹.app`
- **磁盘映像（便于分发安装）**：`dmg/寻迹_<版本>_aarch64.dmg`（或带 `x86_64` 后缀，取决于本机架构）

双击 `.dmg`，把「寻迹」拖进「应用程序」即可试用。

## Sidecar 与「提炼」功能

发布包运行时，会在下面两处查找 `xunji-sidecar`：

1. 本机构建目录下的 `packages/sidecar/dist/xunji-sidecar`（仅当路径仍存在时）
2. 用户目录 **`~/.xunji/bin/xunji-sidecar`**

若你只在另一台 Mac 上拷贝 `.app` / `.dmg`，请把 sidecar 一并部署到全局路径，否则界面可用，但**提炼相关能力会提示未找到 sidecar**：

```bash
mkdir -p ~/.xunji/bin
cp packages/sidecar/dist/xunji-sidecar ~/.xunji/bin/
chmod +x ~/.xunji/bin/xunji-sidecar
```

## 仅构建命令行（不打包安装包）

```bash
cd apps/desktop/src-tauri
cargo build --release
# 二进制：target/release/xunji-desktop
```

## 常见问题

- **Tauri 提示 `invalid value '1' for '--ci'`**  
  使用本仓库提供的 `bun run tauri:build`（或手动 `env -u CI tauri build`），不要在外层 CI 把 `CI=1` 传给 CLI。

- **Bundle identifier 以 `.app` 结尾的警告**  
  来自 `identifier: com.xunji.app`，若以后要上架或严格签名，可改为不含 `.app` 后缀的 ID（需同步改 `tauri.conf.json`）。

- **代码签名 / 公证**  
  本地试用可直接打开；若需分发给他人且减少「无法验证开发者」提示，需在 Apple 开发者账号下配置证书并对 `.app` 签名/notarize，此处不展开。
