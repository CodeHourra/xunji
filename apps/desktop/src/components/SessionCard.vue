<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { getCardTypeLabel } from '@xunji/shared'
import { NTag, NButton, NCheckbox, NTooltip } from 'naive-ui'
import type { SessionSummary } from '../types'

const props = defineProps<{
  session: SessionSummary
  analyzing?: boolean
  selectable?: boolean
  selected?: boolean
}>()

const emit = defineEmits<{
  analyze: [id: string]
  'update:selected': [id: string, checked: boolean]
}>()

const router = useRouter()

// ── 价值 ───────────────────────────────────────────────────────────────────

const valueKey = computed(() => (props.session.value || 'none').toLowerCase())

/** 高/中价值色条颜色；低/无 不展示色条 */
const barColors: Record<string, string> = {
  high: '#10b981',
  medium: '#f59e0b',
}
const barColor = computed(() => barColors[valueKey.value] ?? null)

/** 是否低价值（low 或 none） */
const isLowValue = computed(() => valueKey.value === 'low' || valueKey.value === 'none')

/** 价值标签样式映射 */
const valueBadgeClass: Record<string, string> = {
  high:   'bg-emerald-50 dark:bg-emerald-950/40 text-emerald-700 dark:text-emerald-400 border-emerald-200 dark:border-emerald-800/60',
  medium: 'bg-amber-50  dark:bg-amber-950/40  text-amber-700  dark:text-amber-400  border-amber-200  dark:border-amber-800/60',
  low:    'bg-slate-50  dark:bg-slate-900/60  text-slate-500  dark:text-slate-400  border-slate-200  dark:border-slate-700/60',
  none:   'bg-slate-50  dark:bg-slate-900/60  text-slate-400  dark:text-slate-500  border-slate-200  dark:border-slate-700/60',
}
const valueBadgeText: Record<string, string> = {
  high: '高价值', medium: '中价值', low: '⚠ 低价值', none: '⚠ 无价值',
}

// ── 已分析判断 ─────────────────────────────────────────────────────────────
// 有卡片（medium/high）或状态已标记为 analyzed（low/none 无卡片但也已完成分析）
const isAnalyzed = computed(
  () => !!props.session.cardId || props.session.status === 'analyzed',
)

// ── 卡片元信息 ─────────────────────────────────────────────────────────────

/** 标签数组，最多 3 个 */
const tagList = computed(() => {
  const raw = props.session.cardTags
  if (!raw) return []
  return raw.split(',').filter(Boolean).slice(0, 3)
})

// ── 大小格式化 ─────────────────────────────────────────────────────────────

function formatSize(bytes: number): string {
  if (!bytes || bytes <= 0) return '—'
  if (bytes < 1024) return '<1 KB'
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
}

// ── 交互 ──────────────────────────────────────────────────────────────────

const isSelectDisabled = computed(
  () => props.selectable && (isAnalyzed.value || props.session.status === 'analyzing'),
)

function openSession() {
  if (props.selectable) {
    if (isSelectDisabled.value) return
    emit('update:selected', props.session.id, !props.selected)
    return
  }
  void router.push({
    name: 'session-detail',
    params: { sessionId: props.session.id },
    query: props.session.cardId ? { cardId: props.session.cardId } : {},
  })
}

function onAction(e: MouseEvent) {
  e.stopPropagation()
  if (isAnalyzed.value) {
    // 已分析：有卡片则带 cardId 打开笔记，低价值无卡片则只看对话
    void router.push({
      name: 'session-detail',
      params: { sessionId: props.session.id },
      query: props.session.cardId ? { cardId: props.session.cardId } : {},
    })
  } else {
    emit('analyze', props.session.id)
  }
}

const sourceIcon = computed(() => {
  if (props.session.sourceId === 'claude-code') return 'i-lucide-bot'
  if (props.session.sourceId === 'cursor') return 'i-lucide-terminal-square'
  return 'i-lucide-message-square'
})

const actionLoading = computed(
  () => props.analyzing || props.session.status === 'analyzing',
)

// ── 列表主标题：首条 user 预览优先（与计划 4.1 一致）──────────────────────

/** Tooltip 内文过长时截断，避免 DOM 过大 */
const TOOLTIP_MAX = 500

function clipForTooltip(s: string, max = TOOLTIP_MAX): string {
  if (s.length <= max) return s
  return `${s.slice(0, max)}…`
}

/**
 * 路径展示：过长时只显示最后若干段，全文放 Tooltip
 *   foo/bar/baz.json → …/bar/baz.json（段数少时保持原样）
 */
function pathTail(path: string | null | undefined, segments = 2): string {
  if (!path) return ''
  const norm = path.replace(/\\/g, '/')
  const parts = norm.split('/').filter(Boolean)
  if (parts.length <= segments) return norm
  return `…/${parts.slice(-segments).join('/')}`
}

const sessionPathDisplay = computed(
  () => props.session.rawPath ?? props.session.projectPath ?? null,
)

/** 列表主标题：首条 user → 项目名/路径 → sessionId 前缀 */
const listPrimaryTitle = computed(() => {
  const preview = props.session.firstUserPreview?.trim()
  if (preview) return preview
  return (
    props.session.projectName
    || props.session.projectPath
    || props.session.sessionId.slice(0, 14) + (props.session.sessionId.length > 14 ? '…' : '')
  )
})

const listPrimaryTooltip = computed(() =>
  clipForTooltip(
    props.session.firstUserPreview?.trim()
    || props.session.projectName
    || props.session.projectPath
    || props.session.sessionId,
  ),
)

const timeTooltip = computed(() => {
  const u = props.session.updatedAt
  if (!u) return '会话时间：—'
  const readable = u.replace('T', ' ').slice(0, 19)
  return `会话时间：${readable}（updatedAt）`
})

/**
 * 会话卡片视觉分层（与 KnowledgeView 条目对齐）：
 * - 默认：半透底 + 固定轻阴影；浅/深用同一套「阴影表层次」逻辑（暗色同样保留 shadow，不靠纯扁平）
 * - hover：仅描边变化，不抬升阴影（与知识库一致）
 * - 批量选中：ring + 加粗边框 + 渐变底 + 更强阴影，避免仅靠淡绿底不够醒目
 */
const cardSurfaceClass = computed(() => {
  const baseDisabled = isSelectDisabled.value
    ? 'cursor-default opacity-60'
    : 'cursor-pointer'

  // 批量选中：强对比（与列表区渐变底区分仍清晰）
  if (props.selectable && props.selected) {
    return [
      baseDisabled,
      'border-2 border-brand-500 dark:border-brand-400',
      'bg-gradient-to-br from-brand-50/95 to-emerald-50/90 dark:from-brand-950/55 dark:to-emerald-950/40',
      'shadow-[0_4px_24px_-4px_rgba(16,185,129,0.42)] dark:shadow-[0_4px_28px_-6px_rgba(16,185,129,0.36)]',
      'ring-2 ring-brand-500/80 dark:ring-brand-400/70',
    ]
  }

  // 默认卡片面：与 KnowledgeView 列表项同源 class
  const knowledgeLike = [
    'border border-white/70 dark:border-emerald-900/50',
    'bg-white/85 dark:bg-neutral-900/75',
    'shadow-[0_2px_14px_-2px_rgba(16,185,129,0.18)] dark:shadow-[0_2px_16px_-4px_rgba(0,0,0,0.45)]',
    'backdrop-blur-sm',
    'transition-[border-color] duration-150',
    baseDisabled,
  ]

  if (isAnalyzed.value) {
    return [
      ...knowledgeLike,
      'hover:border-emerald-300/55 dark:hover:border-emerald-700/45',
    ]
  }

  // 未分析：虚线边框，仍保持同一套阴影逻辑
  return [
    ...knowledgeLike,
    'border-emerald-200/65 dark:border-emerald-800/50 border-dashed',
    'hover:border-emerald-400/75 dark:hover:border-emerald-600/55',
  ]
})
</script>

<template>
  <div
    class="group relative rounded-xl"
    :class="cardSurfaceClass"
    @click="openSession"
  >
    <!-- 左侧高/中价值色条（置于内容之上，避免与圆角/选中 ring 打架） -->
    <div
      v-if="barColor"
      class="absolute left-0 top-2 bottom-2 w-[3px] rounded-r-full z-[1] pointer-events-none"
      :style="{ backgroundColor: barColor }"
    />

    <div class="flex items-center gap-3 px-4 py-3">

      <!-- 批量模式 checkbox -->
      <div v-if="selectable" class="shrink-0" @click.stop>
        <n-checkbox
          :checked="selected"
          :disabled="isSelectDisabled"
          @update:checked="!isSelectDisabled && emit('update:selected', session.id, $event)"
        />
      </div>

      <!-- ════════════════════════════════════
           左列：项目名+时间同行 / 路径与会话 id / 消息+大小
           ════════════════════════════════════ -->
      <div class="w-48 shrink-0 min-w-0">
        <!-- 项目名与时间同一行；时间 Tooltip 为「会话时间」 -->
        <div class="flex items-center justify-between gap-2 min-w-0 text-[11px]">
          <span class="truncate text-neutral-600 dark:text-neutral-300 font-medium">
            {{ session.projectName || session.projectPath || '未命名项目' }}
          </span>
          <n-tooltip trigger="hover" :delay="300">
            <template #trigger>
              <span class="shrink-0 font-mono tabular-nums text-neutral-400 cursor-default">
                {{ session.updatedAt?.replace('T', ' ').slice(0, 16) ?? '—' }}
              </span>
            </template>
            {{ timeTooltip }}
          </n-tooltip>
        </div>

        <!-- 路径（省略末段 + 全文 Tooltip）与会话 ID -->
        <div class="mt-1 space-y-0.5 text-[10px] text-neutral-400 min-w-0">
          <n-tooltip
            v-if="sessionPathDisplay"
            trigger="hover"
            placement="top-start"
            :delay="200"
          >
            <template #trigger>
              <div class="flex items-center gap-1 min-w-0">
                <span class="i-lucide-folder-open w-3 h-3 shrink-0 opacity-70" />
                <span class="truncate font-mono">{{ pathTail(sessionPathDisplay) }}</span>
              </div>
            </template>
            {{ sessionPathDisplay }}
          </n-tooltip>
          <n-tooltip trigger="hover" :delay="200">
            <template #trigger>
              <div class="flex items-center gap-1 min-w-0 font-mono opacity-90">
                <span class="i-lucide-fingerprint w-3 h-3 shrink-0" />
                <span class="truncate">{{ session.sessionId }}</span>
              </div>
            </template>
            会话 ID：{{ session.sessionId }}
          </n-tooltip>
        </div>

        <div class="text-[11px] text-neutral-400 mt-1 flex items-center gap-2">
          <span :class="sourceIcon" class="w-3 h-3 opacity-70 shrink-0" />
          <span class="flex items-center gap-0.5">
            <span class="i-lucide-message-circle w-3 h-3" />
            {{ session.messageCount }} 条
          </span>
          <span class="flex items-center gap-0.5">
            <span class="i-lucide-hard-drive w-3 h-3" />
            {{ formatSize(session.rawSizeBytes) }}
          </span>
        </div>
      </div>

      <!-- ════════════════════════════════════
           右内容区：已分析 / 未分析 两态
           ════════════════════════════════════ -->
      <div class="flex-1 min-w-0">
        <!-- 主标题：首条 user 预览 + Tooltip；已分析时可附「笔记」副标题 -->
        <div class="mb-1.5 min-w-0">
          <n-tooltip
            trigger="hover"
            placement="top-start"
            :delay="250"
          >
            <template #trigger>
              <p
                class="text-sm font-semibold leading-snug text-neutral-900 dark:text-neutral-100 line-clamp-2"
              >
                {{ listPrimaryTitle }}
              </p>
            </template>
            <div class="max-w-md whitespace-pre-wrap break-words text-xs">
              {{ listPrimaryTooltip }}
            </div>
          </n-tooltip>
          <p
            v-if="isAnalyzed && session.cardTitle"
            class="text-[11px] text-neutral-500 dark:text-neutral-400 mt-0.5 line-clamp-1"
          >
            笔记：{{ session.cardTitle }}
          </p>
        </div>

        <!-- ── 已分析：三行布局 ── -->
        <template v-if="isAnalyzed">

          <!-- 第一行：类型 Badge + 价值 Badge -->
          <div class="flex items-center gap-1.5 flex-wrap">
            <n-tag
              v-if="session.cardType"
              size="tiny"
              :bordered="false"
              type="info"
            >
              {{ getCardTypeLabel(session.cardType) }}
            </n-tag>
            <!-- 价值标签（始终展示，颜色随价值变化） -->
            <span
              v-if="session.value"
              class="inline-flex items-center text-[10px] font-medium px-1.5 py-px rounded border"
              :class="valueBadgeClass[valueKey] ?? valueBadgeClass.none"
            >
              {{ valueBadgeText[valueKey] ?? session.value }}
            </span>
          </div>

          <!-- 第二行：摘要文字 -->
          <p
            v-if="session.cardSummary"
            class="text-xs mt-1 line-clamp-1 leading-relaxed"
            :class="isLowValue
              ? 'text-neutral-400 dark:text-neutral-500 italic'
              : 'text-neutral-600 dark:text-neutral-400'"
          >
            {{ session.cardSummary }}
          </p>
          <p v-else class="text-xs text-neutral-400 dark:text-neutral-600 mt-1 italic">暂无摘要</p>

          <!-- 第三行：标签 chips（最多 3 个） -->
          <div v-if="tagList.length" class="flex items-center gap-1 mt-1 flex-wrap">
            <span
              v-for="tag in tagList"
              :key="tag"
              class="inline-block text-[10px] px-1.5 py-px rounded-full
                     bg-neutral-100 dark:bg-neutral-800
                     text-neutral-500 dark:text-neutral-400
                     border border-neutral-200 dark:border-neutral-700"
            >{{ tag }}</span>
          </div>
        </template>

        <!-- ── 未分析 ── -->
        <template v-else>
          <div class="flex items-center gap-1.5 flex-wrap">
            <n-tag size="tiny" :bordered="false">{{ session.sourceId }}</n-tag>
            <n-tag
              size="tiny"
              :bordered="false"
              :type="session.status === 'analyzing' ? 'info'
                   : session.status === 'error' ? 'error'
                   : 'default'"
            >
              {{ session.status === 'pending'   ? '待分析'
               : session.status === 'analyzing' ? '分析中'
               : session.status === 'error'     ? '失败'
               : session.status }}
            </n-tag>
          </div>
          <p
            v-if="session.status === 'error'"
            class="text-xs text-red-600 dark:text-red-400 line-clamp-3 mt-1.5 leading-relaxed"
          >
            {{ session.errorMessage || '分析失败，点击查看详情或重试。' }}
          </p>
          <p
            v-else
            class="text-xs text-neutral-500 dark:text-neutral-400 line-clamp-1 mt-1.5 leading-relaxed"
          >
            原始对话，点击查看内容或进行知识提炼。
          </p>
        </template>
      </div>

      <!-- ── 操作按钮 ──
           无价值（none）：不显示任何按钮
           低价值（low）且已分析：查看对话
           有卡片（medium/high）：查看笔记
           未分析：分析
      -->
      <!-- 已分析且无价值（none）时不显示任何按钮；其他情况（待分析、低价值、中/高价值）均显示 -->
      <div
        v-if="!selectable && !(isAnalyzed && valueKey === 'none')"
        class="shrink-0"
        @click.stop
      >
        <n-button
          :type="isAnalyzed ? 'default' : 'primary'"
          size="small"
          :loading="actionLoading"
          :disabled="actionLoading"
          @click="onAction"
        >
          <span class="inline-flex items-center gap-1.5">
            <template v-if="!actionLoading">
              <span
                :class="session.cardId
                  ? 'i-lucide-book-open'
                  : isAnalyzed
                    ? 'i-lucide-messages-square'
                    : 'i-lucide-sparkles'"
                class="w-3.5 h-3.5"
              />
            </template>
            {{
              actionLoading ? '分析中…'
              : session.cardId ? '查看笔记'
              : isAnalyzed   ? '查看对话'
              : '分析'
            }}
          </span>
        </n-button>
      </div>
    </div>
  </div>
</template>
