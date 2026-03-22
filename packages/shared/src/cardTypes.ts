/**
 * 知识卡片 `type` 字段：存储值为英文枚举，UI 统一通过本模块转为中文。
 *
 * - **当前提炼约定**：与 `packages/sidecar` 中 `PROMPT_B_FULL` / `PROMPT_B_LIGHT` 的 type 枚举一致。
 * - **兼容项**：历史文档或其它管线曾出现的类型键，库中若仍有数据则一并映射，避免侧栏/列表裸显英文。
 */

/** 英文 type → 中文展示名（单一数据源） */
export const CARD_TYPE_LABELS = {
  // ── 当前 sidecar 约定（judge / distill）──
  debug: '调试',
  research: '调研',
  implementation: '实现',
  optimization: '优化',
  learning: '学习',
  other: '其他',
  // ── 历史或扩展（数据库中可能存在）──
  architecture: '架构',
  performance: '性能',
  'best-practice': '最佳实践',
  concept: '概念',
  'tool-usage': '工具使用',
  refactor: '重构',
} as const

/** 与 `CARD_TYPE_LABELS` 键集合一致，供类型标注使用 */
export type CardType = keyof typeof CARD_TYPE_LABELS

/**
 * 将存储用的 type 转为界面展示文案；未知键原样返回，避免空白。
 */
export function getCardTypeLabel(code: string | null | undefined): string {
  if (code == null || code === '') return ''
  const map = CARD_TYPE_LABELS as Record<string, string>
  return map[code] ?? code
}
