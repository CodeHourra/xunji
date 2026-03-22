#!/usr/bin/env node
/**
 * 将仓库根目录 CHANGELOG.md 同步到桌面端关于页数据源。
 *
 * 用法（仓库根目录）：
 *   npm run sync:changelog
 *
 * 输出与 CHANGELOG.md 正文一致，不包含维护说明，便于关于页仅展示用户可见的更新内容。
 * 说明：apps/desktop/src/data/app-changelog.md 由本脚本覆盖写入，请勿手改。
 */
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const root = path.resolve(__dirname, '..')
const changelogPath = path.join(root, 'CHANGELOG.md')
const destPath = path.join(root, 'apps/desktop/src/data/app-changelog.md')

if (!fs.existsSync(changelogPath)) {
  console.error('sync-changelog: 未找到', changelogPath)
  process.exit(1)
}

const body = fs.readFileSync(changelogPath, 'utf8')
fs.mkdirSync(path.dirname(destPath), { recursive: true })
fs.writeFileSync(destPath, body, 'utf8')
console.log('sync-changelog: OK →', path.relative(root, destPath))
