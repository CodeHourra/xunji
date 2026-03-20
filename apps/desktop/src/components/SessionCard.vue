<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { NTag, NButton, NCheckbox } from 'naive-ui'
import type { SessionSummary } from '../types'

const props = defineProps<{
  session: SessionSummary
  /** 外部正在分析该卡片（SessionsView 传入） */
  analyzing?: boolean
  /** 批量选择模式：展示 checkbox */
  selectable?: boolean
  /** 当前是否被选中 */
  selected?: boolean
}>()

const emit = defineEmits<{
  analyze: [id: string]
  'update:selected': [id: string, checked: boolean]
}>()

const router = useRouter()

const valueKey = computed(() => (props.session.value || 'none').toLowerCase())

const valueColors: Record<string, string> = {
  high: '#10b981',
  medium: '#f59e0b',
  low: '#94a3b8',
  none: 'transparent',
}

const barColor = computed(() => valueColors[valueKey.value] || valueColors.none)

/**
 * 格式化字节数为可读大小（KB / MB）
 * < 1 KB 显示 "<1 KB"
 */
function formatSize(bytes: number): string {
  if (!bytes || bytes <= 0) return '—'
  if (bytes < 1024) return '<1 KB'
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
}

/** 点击卡片整行 → 批量模式切换选中，否则跳转详情 */
function openSession() {
  if (props.selectable) {
    emit('update:selected', props.session.id, !props.selected)
    return
  }
  void router.push({
    name: 'session-detail',
    params: { sessionId: props.session.id },
    query: props.session.cardId ? { cardId: props.session.cardId } : {},
  })
}

/** 右侧按钮：有卡片→直接查看笔记，无卡片→触发分析 */
function onAction(e: MouseEvent) {
  e.stopPropagation()
  if (props.session.cardId) {
    void router.push({
      name: 'session-detail',
      params: { sessionId: props.session.id },
      query: { cardId: props.session.cardId },
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

/** 状态对应的 NTag type */
const statusTagType = computed<'default' | 'info' | 'success' | 'error'>(() => {
  switch (props.session.status) {
    case 'analyzed': return 'success'
    case 'analyzing': return 'info'
    case 'error': return 'error'
    default: return 'default'
  }
})

const statusLabel = computed(() => {
  switch (props.session.status) {
    case 'pending': return '待分析'
    case 'analyzing': return '分析中'
    case 'analyzed': return '已分析'
    case 'error': return '失败'
    default: return props.session.status
  }
})

/** 按钮 loading 态：外部正在分析 OR 会话状态为 analyzing */
const actionLoading = computed(
  () => props.analyzing || props.session.status === 'analyzing',
)
</script>

<template>
  <div
    class="group relative rounded-lg border bg-white dark:bg-neutral-900
           transition-all duration-150"
    :class="[
      selectable ? 'cursor-pointer' : 'cursor-pointer hover:shadow-sm',
      selected
        ? 'border-brand-400 dark:border-brand-600 bg-brand-50/40 dark:bg-brand-950/30'
        : session.cardId
          ? 'border-neutral-200 dark:border-neutral-800 hover:border-neutral-300 dark:hover:border-neutral-700'
          : 'border-neutral-200/70 dark:border-neutral-800/70 border-dashed hover:border-neutral-300 dark:hover:border-neutral-700',
    ]"
    @click="openSession"
  >
    <!-- 左侧价值色条 -->
    <div
      v-if="valueKey !== 'none'"
      class="absolute left-0 top-2 bottom-2 w-[3px] rounded-r-full"
      :style="{ backgroundColor: barColor }"
    />

    <div class="flex items-center gap-3 px-4 py-3">
      <!-- 批量模式：checkbox -->
      <div v-if="selectable" class="shrink-0" @click.stop>
        <n-checkbox
          :checked="selected"
          @update:checked="emit('update:selected', session.id, $event)"
        />
      </div>

      <!-- 左区：时间 + 项目名 -->
      <div class="w-44 shrink-0 min-w-0">
        <div class="text-[11px] text-neutral-400 font-mono tabular-nums">
          {{ session.updatedAt?.replace('T', ' ').slice(0, 16) ?? '—' }}
        </div>
        <div class="flex items-center gap-1.5 mt-1">
          <span :class="sourceIcon" class="w-3.5 h-3.5 text-neutral-400 shrink-0" />
          <span
            class="text-sm font-medium text-neutral-800 dark:text-neutral-200 truncate
                   group-hover:text-brand-700 dark:group-hover:text-brand-400 transition-colors"
            :title="session.projectName || session.projectPath || '未命名项目'"
          >
            {{ session.projectName || session.projectPath || '未命名项目' }}
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

      <!-- 中区：徽章 + 描述 -->
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-1.5 flex-wrap">
          <n-tag size="tiny" :bordered="false">{{ session.sourceId }}</n-tag>
          <n-tag size="tiny" :bordered="false" :type="statusTagType">{{ statusLabel }}</n-tag>
          <n-tag
            v-if="session.value"
            size="tiny"
            :bordered="false"
            :color="{ color: barColor, textColor: '#fff' }"
            class="uppercase font-bold"
          >
            {{ session.value }}
          </n-tag>
        </div>
        <p class="text-xs text-neutral-500 dark:text-neutral-400 line-clamp-1 mt-1.5 leading-relaxed">
          <template v-if="session.cardId">知识已提炼，点击查看笔记与对话回放。</template>
          <template v-else-if="session.status === 'error'">分析失败，点击查看详情或重试。</template>
          <template v-else>原始对话，点击查看内容或进行知识提炼。</template>
        </p>
      </div>

      <!-- 右区：操作按钮（阻止冒泡，不触发整行跳转） -->
      <div v-if="!selectable" class="shrink-0" @click.stop>
        <n-button
          :type="session.cardId ? 'default' : 'primary'"
          size="small"
          :loading="actionLoading"
          :disabled="actionLoading"
          @click="onAction"
        >
          <span class="inline-flex items-center gap-1.5">
            <span
              v-if="!actionLoading"
              :class="session.cardId ? 'i-lucide-book-open' : (session.status === 'analyzed' ? 'i-lucide-refresh-cw' : 'i-lucide-sparkles')"
              class="w-3.5 h-3.5"
            />
            {{ actionLoading ? '分析中…' : (session.cardId ? '查看笔记' : (session.status === 'analyzed' ? '重新分析' : '分析')) }}
          </span>
        </n-button>
      </div>
    </div>
  </div>
</template>
