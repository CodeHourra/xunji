<script setup lang="ts">
/**
 * 全局悬浮分析队列面板 —— 任意页面可见，展示当前任务、进度、最近完成、停止/关闭。
 */
import { storeToRefs } from 'pinia'
import { NProgress, NButton, NSpin } from 'naive-ui'
import { useAnalysisQueueStore } from '../stores/analysisQueue'

const queue = useAnalysisQueueStore()
const {
  currentTask,
  pendingCount,
  totalCount,
  doneCount,
  hasAny,
  isIdle,
  progressPercent,
  recentCompleted,
} = storeToRefs(queue)

function truncate(s: string, max = 28) {
  if (s.length <= max) return s
  return `${s.slice(0, max)}…`
}
</script>

<template>
  <Transition
    enter-active-class="transition duration-200 ease-out"
    enter-from-class="opacity-0 translate-y-2"
    enter-to-class="opacity-100 translate-y-0"
    leave-active-class="transition duration-150 ease-in"
    leave-from-class="opacity-100 translate-y-0"
    leave-to-class="opacity-0 translate-y-2"
  >
    <div
      v-if="hasAny"
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
        <div class="flex items-center gap-2 shrink-0">
          <span class="text-xs text-neutral-500 tabular-nums">
            已完成 {{ doneCount }}/{{ totalCount }}
          </span>
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
        <!-- 当前运行中 -->
        <div v-if="currentTask">
          <div class="flex items-start gap-2">
            <n-spin size="small" class="mt-0.5 shrink-0" />
            <div class="min-w-0 flex-1">
              <p class="text-xs text-neutral-700 dark:text-neutral-300 leading-snug">
                {{ truncate(currentTask.displayTitle) }}
              </p>
              <p class="text-[11px] text-neutral-400 mt-0.5 tabular-nums">
                已耗时 {{ currentTask.elapsedSec }}s
                <span
                  v-if="currentTask.elapsedSec > 150"
                  class="text-amber-600 dark:text-amber-400 ml-1"
                >
                  （耗时较长，请耐心等待）
                </span>
              </p>
            </div>
          </div>
          <n-progress
            type="line"
            :percentage="progressPercent"
            :show-indicator="true"
            :height="6"
            class="mt-2"
            :color="'var(--brand-500, #10b981)'"
          />
        </div>

        <!-- 无当前任务但仍有排队（理论上短暂） -->
        <div v-else-if="pendingCount > 0 && !isIdle" class="text-xs text-neutral-500 flex items-center gap-1.5">
          <span class="i-lucide-loader-2 w-3.5 h-3.5 animate-spin" />
          准备下一条…
        </div>

        <!-- 最近完成 -->
        <ul v-if="recentCompleted.length" class="space-y-1 border-t border-neutral-100 dark:border-neutral-800 pt-2">
          <li
            v-for="t in recentCompleted"
            :key="t.sessionId + t.status + (t.outcome ?? '')"
            class="flex items-center gap-1.5 text-[11px] text-neutral-500 dark:text-neutral-400"
          >
            <span
              v-if="t.status === 'done'"
              class="i-lucide-check-circle w-3.5 h-3.5 text-emerald-500 shrink-0"
            />
            <span
              v-else
              class="i-lucide-x-circle w-3.5 h-3.5 text-red-400 shrink-0"
            />
            <span class="truncate">{{ truncate(t.displayTitle, 32) }}</span>
          </li>
        </ul>

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
