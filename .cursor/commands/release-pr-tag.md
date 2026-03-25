---
name: /release-pr-tag
description: 从 release 分支发起合并 master 的 PR、打 v 标签并依赖现有 GitHub Actions（CI / Release）。
---

本仓库固定流程：**release 分支 → PR → `master`**，**打 `v*.*.*` tag** 触发 **Release**；**PR / master push** 触发 **CI**。

请在本机仓库中**按顺序执行**（将 `X.Y.Z` 换成实际版本，如 `0.1.2`；分支名一般为 `release/vX.Y.Z`）：

## 1. 前置检查

- `git fetch origin && git status`，当前应在目标 **release 分支**（如 `release/v0.1.2`），工作区干净。
- 确认版本与文档一致：`apps/desktop/package.json`、`apps/desktop/src-tauri/Cargo.toml` 中 `version`，以及根目录 `CHANGELOG.md` / `docs/CHANGELOG.md`和`apps/desktop/src-tauri/tauri.conf.json`。
- 校验关于页数据源与根目录 CHANGELOG 一致：在仓库根执行 `bun run sync:changelog`，再 `git diff apps/desktop/src/data/app-changelog.md` 应为空；若有差异则提交后再继续。

## 2. 确认合并关系（可选但推荐）

- `git log origin/master..release/vX.Y.Z --oneline`：应能看到尚未进 `master` 的提交；若为空说明已合并或无需发版。
- `git branch --merged origin/master` 是否包含 `release/vX.Y.Z`：不包含则仍需 PR。

## 3. 推送 release 分支

- `git push origin release/vX.Y.Z`，确保远端与本地一致。

## 4. 发起 PR

- 使用 GitHub CLI（需已 `gh auth login`）：
  - `gh pr create --base master --head release/vX.Y.Z --title "release: 合并 vX.Y.Z（<一句话说明>）" --body "<摘要、CI 说明、合并后是否打 tag 等>"`
- 若 PR 已存在：`gh pr list --head release/vX.Y.Z`。

## 5. 打标签并推送（触发 Release 工作流）

- 在**要打标签的提交**上（通常为当前 release 分支 HEAD，或合并到 `master` 后的合并提交）：
  - `git tag -a vX.Y.Z -m "vX.Y.Z: <简短说明>"`
  - `git push origin vX.Y.Z`
- 推送 `v*.*.*` 会触发 `.github/workflows/release.yml`，在 GitHub 上**自动创建 Release**（含 `generate_release_notes`）。
- **注意**：若希望 tag **只指向 `master` 上的合并提交**，应在 **合并 PR 之后**再在 `master` 上打 tag；若先在 release 分支打 tag，则 tag 指向 release 分支提交，与 `master` 合并提交 SHA 不同（可按团队规范二选一）。

## 6. 关注 Actions

- **CI**（`.github/workflows/ci.yml`）：对指向 `master` 的 PR 与 `master` 的 push 运行（install、changelog 校验、sidecar/desktop 构建、`cargo check`）。
- **Release**（`.github/workflows/release.yml`）：对推送的 `v*.*.*` tag 运行。

在浏览器打开 `https://github.com/<owner>/<repo>/actions` 确认工作流成功；失败则根据日志修复后重新推送分支或 tag。

## 7. 若尚无 CI / Release 工作流

- 在仓库中应有 `.github/workflows/ci.yml` 与 `.github/workflows/release.yml`（本仓库已配置）。若在新项目首次添加，需将这两个文件**随 release 分支一并提交并推送**，再发 PR，再打 tag。

---

**完成后**向用户简短汇报：PR 链接、tag 名、Actions 与 Release 页面是否成功。
