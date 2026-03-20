<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { NTag, NButton } from 'naive-ui'
import type { SessionSummary } from '../types'

const props = defineProps<{
  session: SessionSummary
}>()

const emit = defineEmits<{
  analyze: [id: string]
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

function openNote() {
  if (props.session.cardId) {
    void router.push({
      name: 'session-detail',
      params: { sessionId: props.session.id },
      query: { cardId: props.session.cardId },
    })
  }
}

function onPrimary() {
  if (props.session.cardId) {
    openNote()
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
</script>

<template>
  <div
    class="group relative rounded-lg border bg-white dark:bg-neutral-900 transition-all duration-150 hover:shadow-sm cursor-default"
    :class="session.cardId
      ? 'border-neutral-200 dark:border-neutral-800'
      : 'border-neutral-200/70 dark:border-neutral-800/70 border-dashed'"
  >
    <!-- 左侧价值色条 -->
    <div
      v-if="valueKey !== 'none'"
      class="absolute left-0 top-2 bottom-2 w-[3px] rounded-r-full"
      :style="{ backgroundColor: barColor }"
    />

    <div class="flex items-center gap-4 px-4 py-3">
      <!-- 左区：时间 + 项目名 -->
      <div class="w-44 shrink-0 min-w-0">
        <div class="text-[11px] text-neutral-400 font-mono tabular-nums">
          {{ session.updatedAt?.replace('T', ' ').slice(0, 16) ?? '—' }}
        </div>
        <div class="flex items-center gap-1.5 mt-1">
          <span :class="sourceIcon" class="w-3.5 h-3.5 text-neutral-400 shrink-0" />
          <span class="text-sm font-medium text-neutral-800 dark:text-neutral-200 truncate" :title="session.projectName || session.projectPath || '未命名项目'">
            {{ session.projectName || session.projectPath || '未命名项目' }}
          </span>
        </div>
        <div class="text-[11px] text-neutral-400 mt-0.5 flex items-center gap-2">
          <span class="flex items-center gap-0.5">
            <span class="i-lucide-message-circle w-3 h-3" />
            {{ session.messageCount }} 条消息
          </span>
          <span class="truncate max-w-20" :title="session.sessionId">{{ session.sessionId.slice(0, 8) }}…</span>
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
          <template v-if="session.cardId">知识已提炼，可点击查看笔记与对话回放。</template>
          <template v-else>原始对话，点击「分析」提炼编程知识。</template>
        </p>
      </div>

      <!-- 右区：操作 -->
      <div class="shrink-0">
        <n-button
          :type="session.cardId ? 'default' : 'primary'"
          size="small"
          :loading="session.status === 'analyzing'"
          :disabled="session.status === 'analyzing'"
          @click="onPrimary"
        >
          <span class="inline-flex items-center gap-1.5">
            <span v-if="session.status !== 'analyzing'" :class="session.cardId ? 'i-lucide-book-open' : 'i-lucide-sparkles'" class="w-3.5 h-3.5" />
            {{ session.status === 'analyzing' ? '分析中…' : (session.cardId ? '查看笔记' : '分析') }}
          </span>
        </n-button>
      </div>
    </div>
  </div>
</template>
