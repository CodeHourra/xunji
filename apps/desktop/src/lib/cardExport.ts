/**
 * 笔记导出：系统对话框选路径 + invoke Rust 写盘。
 * 与 `commands/export.rs` 及 `docs/specs/导出-Markdown-约定.md` 一致。
 */
import { open, save } from '@tauri-apps/plugin-dialog'
import { api } from './tauri'

/** 另存为单文件，返回路径或取消 */
export async function pickSaveMarkdownFile(defaultName: string): Promise<string | null> {
  const path = await save({
    defaultPath: defaultName.endsWith('.md') ? defaultName : `${defaultName}.md`,
    filters: [{ name: 'Markdown', extensions: ['md'] }],
  })
  return path ?? null
}

/** 选择目录（批量 / 全库导出） */
export async function pickExportDirectory(title?: string): Promise<string | null> {
  const r = await open({
    directory: true,
    multiple: false,
    title: title ?? '选择导出目录',
  })
  if (r == null) return null
  return Array.isArray(r) ? (r[0] ?? null) : r
}

/** 由标题生成默认文件名（不含路径） */
export function defaultMarkdownFileName(title: string, cardId: string): string {
  const safe = title
    .replace(/[/\\:*?"<>|#\x00-\x1f]/g, '-')
    .trim()
    .slice(0, 80) || 'note'
  const short = cardId.slice(0, 8)
  return `${safe}-${short}.md`
}

export async function exportOneCardToFile(cardId: string, title: string): Promise<boolean> {
  const path = await pickSaveMarkdownFile(defaultMarkdownFileName(title, cardId))
  if (!path) return false
  await api.exportCardMarkdown(cardId, path)
  return true
}

export async function exportSelectedCards(cardIds: string[]): Promise<{ ok: boolean; count?: number }> {
  if (!cardIds.length) return { ok: false }
  const dir = await pickExportDirectory('导出所选笔记')
  if (!dir) return { ok: false }
  const n = await api.exportCardsMarkdownDir(cardIds, dir)
  return { ok: true, count: n }
}

/** 选目录后导出库内全部卡片（调用方已确认条数与语义） */
export async function exportAllCardsToDir(): Promise<{ ok: boolean; count?: number }> {
  const dir = await pickExportDirectory('导出全部笔记（库内全部）')
  if (!dir) return { ok: false }
  const n = await api.exportAllCardsMarkdownDir(dir)
  return { ok: true, count: n }
}

