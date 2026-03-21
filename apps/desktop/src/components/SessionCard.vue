<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { NTag, NButton, NCheckbox } from 'naive-ui'
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

const cardTypeLabel: Record<string, string> = {
  debug: 'Debug',
  implementation: '实现',
  research: '调研',
  optimization: '优化',
  learning: '学习',
  other: '其他',
}

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
</script>

<template>
  <div
    class="group relative rounded-lg border bg-white dark:bg-neutral-900 transition-all duration-150"
    :class="[
      isSelectDisabled ? 'cursor-default opacity-60' : 'cursor-pointer hover:shadow-sm',
      selected
        ? 'border-brand-400 dark:border-brand-600 bg-brand-50/40 dark:bg-brand-950/30'
        : isAnalyzed
          ? 'border-neutral-200 dark:border-neutral-800 hover:border-neutral-300 dark:hover:border-neutral-700'
          : 'border-neutral-200/70 dark:border-neutral-800/70 border-dashed hover:border-neutral-300 dark:hover:border-neutral-700',
    ]"
    @click="openSession"
  >
    <!-- 左侧高/中价值色条 -->
    <div
      v-if="barColor"
      class="absolute left-0 top-2 bottom-2 w-[3px] rounded-r-full"
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
           左列：时间 / 标题 or 项目名 / 消息+大小
           ════════════════════════════════════ -->
      <div class="w-44 shrink-0 min-w-0">

        <!-- 时间 -->
        <div class="text-[11px] text-neutral-400 font-mono tabular-nums">
          {{ session.updatedAt?.replace('T', ' ').slice(0, 16) ?? '—' }}
        </div>

        <!-- 已分析：卡片标题（主标题）；未分析：项目名 -->
        <div class="flex items-center gap-1.5 mt-1 min-w-0">
          <span :class="sourceIcon" class="w-3.5 h-3.5 text-neutral-400 shrink-0" />
          <span
            class="text-sm font-medium truncate transition-colors
                   group-hover:text-brand-700 dark:group-hover:text-brand-400"
            :class="isAnalyzed
              ? 'text-neutral-900 dark:text-neutral-100'
              : 'text-neutral-700 dark:text-neutral-300'"
            :title="isAnalyzed
              ? (session.cardTitle ?? session.projectName ?? '未命名')
              : (session.projectName ?? session.projectPath ?? '未命名项目')"
          >
            <!-- 已分析显示卡片标题，未分析显示项目名 -->
            <template v-if="isAnalyzed && session.cardTitle">
              {{ session.cardTitle }}
            </template>
            <template v-else>
              {{ session.projectName || session.projectPath || '未命名项目' }}
            </template>
          </span>
        </div>

        <!-- 消息数 + 大小 -->
        <div class="text-[11px] text-neutral-400 mt-0.5 flex items-center gap-2">
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
              {{ cardTypeLabel[session.cardType] ?? session.cardType }}
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
          <p class="text-xs text-neutral-500 dark:text-neutral-400 line-clamp-1 mt-1.5 leading-relaxed">
            <template v-if="session.status === 'error'">分析失败，点击查看详情或重试。</template>
            <template v-else>原始对话，点击查看内容或进行知识提炼。</template>
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
