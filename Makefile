# 寻迹 —— 常用构建入口（macOS 为主，需已安装 Bun / Rust / Xcode CLT）
# 使用：在仓库根目录执行 `make macos` 或 `make help`

REPO := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))

.PHONY: help deps sidecar desktop-frontend macos

help:
	@echo "目标说明："
	@echo "  make deps     - bun install（workspace）"
	@echo "  make sidecar  - 仅编译 packages/sidecar → dist/xunji-sidecar"
	@echo "  make macos    - 依赖 + 打 macOS 安装包（.app + .dmg，内含 Sidecar）"
	@echo "产物：apps/desktop/src-tauri/target/release/bundle/dmg/寻迹_<版本>_<arch>.dmg"

deps:
	cd "$(REPO)" && bun install

sidecar:
	cd "$(REPO)/packages/sidecar" && bun run build

# 一键：安装依赖并执行 Tauri 发布构建（beforeBuild 会编译 sidecar + 前端，bundle.resources 打入 sidecar）
macos: deps
	cd "$(REPO)/apps/desktop" && env -u CI bun run tauri:build

# 一键：安装依赖并执行 Tauri 开发构建（beforeDev 会编译 sidecar + 前端）
dev:
	cd "$(REPO)/apps/desktop" && bun run tauri dev
