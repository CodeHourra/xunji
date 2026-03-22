<script setup lang="ts">
/**
 * 全局悬浮分析队列面板 —— 任意页面可见，展示当前任务、进度、最近完成、停止/关闭。
 * 支持收起为右下角细条，不挡主界面操作。
 */
import { storeToRefs } from 'pinia'
import { watch, ref } from 'vue'
import { NProgress, NButton, NSpin } from 'naive-ui'
import { useAnalysisQueueStore } from '../stores/analysisQueue'

const queue = useAnalysisQueueStore()
const {
  tasks,
  currentTask,
  pendingCount,
  totalCount,
  doneCount,
  hasAny,
  isIdle,
  progressPercent,
} = storeToRefs(queue)

/** 收起态：仅显示窄条，点击可展开 */
const collapsed = ref(false)

watch(hasAny, (v) => {
  if (!v) collapsed.value = false
})

function truncate(s: string, max = 28) {
  if (s.length <= max) return s
  return `${s.slice(0, max)}…`
}
</script>

<template>
  <!-- 收起：窄条，点击展开 -->
  <Transition
    enter-active-class="transition duration-200 ease-out"
    enter-from-class="opacity-0 translate-y-2"
    enter-to-class="opacity-100 translate-y-0"
    leave-active-class="transition duration-150 ease-in"
    leave-from-class="opacity-100 translate-y-0"
    leave-to-class="opacity-0 translate-y-2"
  >
    <button
      v-if="hasAny && collapsed"
      type="button"
      class="fixed bottom-5 right-5 z-50 flex items-center gap-2 pl-3 pr-2 py-2 rounded-full border border-neutral-200 dark:border-neutral-700 bg-white dark:bg-neutral-900 shadow-lg text-left max-w-[min(100vw-2rem,18rem)] hover:border-brand-300 dark:hover:border-brand-700 transition-colors"
      aria-label="展开分析队列"
      @click="collapsed = false"
    >
      <span class="i-lucide-zap w-4 h-4 text-brand-500 shrink-0" aria-hidden="true" />
      <span class="text-xs text-neutral-700 dark:text-neutral-200 tabular-nums shrink-0">
        {{ doneCount }}/{{ totalCount }}
      </span>
      <span
        v-if="currentTask"
        class="text-[11px] text-brand-600 dark:text-brand-400 truncate min-w-0 flex-1"
      >
        {{ truncate(currentTask.displayTitle, 16) }}
      </span>
      <span
        v-else-if="pendingCount > 0"
        class="text-[11px] text-neutral-500 shrink-0"
      >
        等待 {{ pendingCount }} 条
      </span>
      <span class="i-lucide-chevron-up w-4 h-4 text-neutral-400 shrink-0" aria-hidden="true" />
    </button>
  </Transition>

  <Transition
    enter-active-class="transition duration-200 ease-out"
    enter-from-class="opacity-0 translate-y-2"
    enter-to-class="opacity-100 translate-y-0"
    leave-active-class="transition duration-150 ease-in"
    leave-from-class="opacity-100 translate-y-0"
    leave-to-class="opacity-0 translate-y-2"
  >
    <div
      v-if="hasAny && !collapsed"
      class="fixed bottom-5 right-5 z-50 w-[min(100vw-2rem,20rem)] rounded-xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900 shadow-xl"
      role="status"
      aria-live="polite"
    >
      <!-- 标题栏 -->
      <div class="flex items-center justify-between gap-2 px-3 py-2 border-b border-neutral-100 dark:border-neutral-800">
        <div class="flex items-center gap-1.5 min-w-0">
          <span class="i-lucide-zap w-4 h-4 text-brand-500 shrink-0" aria-hidden="true" />
          <span class="text-sm font-medium text-neutral-800 dark:text-neutral-100 truncate">分析队列</span>
        </div>
        <div class="flex items-center gap-1 shrink-0">
          <span class="text-xs text-neutral-500 tabular-nums mr-1">
            已完成 {{ doneCount }}/{{ totalCount }}
          </span>
          <n-button
            quaternary
            size="tiny"
            class="!px-1"
            aria-label="收起面板"
            @click="collapsed = true"
          >
            <span class="i-lucide-chevron-down w-4 h-4" />
          </n-button>
          <n-button
            v-if="isIdle"
            quaternary
            size="tiny"
            class="!px-1"
            aria-label="关闭队列面板"
            @click="queue.clear()"
          >
            <span class="i-lucide-x w-4 h-4" />
          </n-button>
        </div>
      </div>

      <div class="px-3 py-2 space-y-2">
        <!-- 队列明细：每条状态、标题、失败原因（可滚动） -->
        <div
          v-if="tasks.length"
          class="max-h-56 overflow-y-auto space-y-1.5 pr-0.5 text-left border-b border-neutral-100 dark:border-neutral-800 pb-2 -mx-0.5"
        >
          <p class="text-[10px] uppercase tracking-wide text-neutral-400 px-0.5 mb-1">队列明细</p>
          <div
            v-for="t in tasks"
            :key="t.sessionId + t.status + (t.errorMessage ?? '')"
            class="rounded-lg border border-neutral-100 dark:border-neutral-800/80 bg-neutral-50/80 dark:bg-neutral-900/50 px-2 py-1.5"
          >
            <div class="flex items-start gap-1.5 min-w-0">
              <n-spin
                v-if="t.status === 'running'"
                size="small"
                class="mt-0.5 shrink-0"
              />
              <span
                v-else-if="t.status === 'pending'"
                class="i-lucide-clock w-3.5 h-3.5 text-amber-500 shrink-0 mt-0.5"
              />
              <span
                v-else-if="t.status === 'done'"
                class="i-lucide-check-circle w-3.5 h-3.5 text-emerald-500 shrink-0 mt-0.5"
              />
              <span
                v-else
                class="i-lucide-x-circle w-3.5 h-3.5 text-red-400 shrink-0 mt-0.5"
              />
              <div class="min-w-0 flex-1">
                <p class="text-[11px] text-neutral-800 dark:text-neutral-200 leading-snug font-medium truncate">
                  {{ truncate(t.displayTitle, 40) }}
                </p>
                <p
                  v-if="t.status === 'running'"
                  class="text-[10px] text-neutral-400 mt-0.5 tabular-nums"
                >
                  执行中 · {{ t.elapsedSec }}s
                  <span
                    v-if="t.elapsedSec > 150"
                    class="text-amber-600 dark:text-amber-400 ml-1"
                  >（较慢）</span>
                </p>
                <p
                  v-else-if="t.status === 'pending'"
                  class="text-[10px] text-neutral-400 mt-0.5"
                >
                  排队中
                </p>
                <p
                  v-else-if="t.status === 'done'"
                  class="text-[10px] text-emerald-600/90 dark:text-emerald-400/90 mt-0.5"
                >
                  {{ t.outcome === 'low_value' ? '已完成（低/无价值）' : '已完成' }}
                </p>
                <p
                  v-else-if="t.status === 'error' && t.errorMessage"
                  class="text-[10px] text-red-600 dark:text-red-400 mt-0.5 line-clamp-4 break-words"
                >
                  {{ t.errorMessage }}
                </p>
              </div>
            </div>
          </div>
        </div>

        <div v-if="currentTask">
          <n-progress
            type="line"
            :percentage="progressPercent"
            :show-indicator="true"
            :height="6"
            class="mt-1"
            :color="'var(--brand-500, #10b981)'"
          />
        </div>

        <!-- 无任务在跑但仍有排队（短暂空窗） -->
        <div
          v-else-if="pendingCount > 0 && !isIdle"
          class="text-xs text-neutral-500 flex items-center gap-1.5"
        >
          <span class="i-lucide-loader-2 w-3.5 h-3.5 animate-spin" />
          准备下一条…
        </div>

        <!-- 底部：等待数 + 停止 -->
        <div
          v-if="pendingCount > 0"
          class="flex items-center justify-between gap-2 pt-1 border-t border-neutral-100 dark:border-neutral-800"
        >
          <span class="text-[11px] text-neutral-400">
            等待中 {{ pendingCount }} 条
          </span>
          <n-button size="tiny" secondary type="warning" @click="queue.cancel()">
            <span class="inline-flex items-center gap-1">
              <span class="i-lucide-square w-3 h-3" />
              停止排队
            </span>
          </n-button>
        </div>
      </div>
    </div>
  </Transition>
</template>
