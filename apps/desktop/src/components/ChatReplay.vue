<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Message } from '../types'

const props = defineProps<{
  messages: Message[]
}>()

/**
 * 将单条消息内容分解为：
 *   - thinking  <thinking>...</thinking> 内的思考块（可折叠）
 *   - rest       剩余正文
 */
function parseMessage(m: Message) {
  const raw = m.content || ''
  const thinkingMatch = raw.match(/<thinking>([\s\S]*?)<\/thinking>/i)
  const thinking = thinkingMatch ? thinkingMatch[1].trim() : null
  const rest = thinkingMatch ? raw.replace(thinkingMatch[0], '').trim() : raw
  return { ...m, rest, thinking }
}

const bubbles = computed(() => props.messages.map(parseMessage))

/** 超长消息折叠阈值（字符数） */
const COLLAPSE_THRESHOLD = 800

/** 跟踪每条消息是否展开 */
const expanded = ref<Record<string, boolean>>({})

function isLong(content: string) {
  return content.length > COLLAPSE_THRESHOLD
}

function toggle(id: string) {
  expanded.value[id] = !expanded.value[id]
}

function displayContent(id: string, content: string) {
  if (!isLong(content) || expanded.value[id]) return content
  return content.slice(0, COLLAPSE_THRESHOLD) + '…'
}

/** 判断是否为工具消息（tool_use / tool_result） */
function isToolMessage(role: string) {
  return role === 'tool'
}
</script>

<template>
  <div class="space-y-4 text-sm">
    <div
      v-for="m in bubbles"
      :key="m.id"
    >
      <!-- ─── 工具消息：紧凑折叠样式 ─── -->
      <div v-if="isToolMessage(m.role)" class="flex justify-center">
        <details class="w-full max-w-[85%]">
          <summary
            class="cursor-pointer text-[11px] text-neutral-400 dark:text-neutral-500
                   select-none flex items-center gap-1.5 py-1.5 px-3
                   hover:text-neutral-600 dark:hover:text-neutral-300 transition-colors"
          >
            <span class="i-lucide-terminal-square w-3 h-3 opacity-60" />
            <span>工具执行结果</span>
            <span class="i-lucide-chevron-right w-3 h-3 opacity-40 transition-transform details-open:rotate-90" />
          </summary>
          <pre
            class="mt-1 text-[11px] whitespace-pre-wrap break-words leading-relaxed
                   bg-neutral-50 dark:bg-neutral-900/50 border border-neutral-200/60 dark:border-neutral-700/50
                   rounded-lg px-3 py-2 text-neutral-500 dark:text-neutral-400 font-mono
                   max-h-48 overflow-y-auto"
          >{{ m.rest }}</pre>
        </details>
      </div>

      <!-- ─── 用户 / AI 消息：对话气泡样式 ─── -->
      <div
        v-else
        class="flex gap-3"
        :class="m.role === 'assistant' ? 'flex-row-reverse' : 'flex-row'"
      >
        <!-- 角色头像 -->
        <div
          class="w-7 h-7 rounded-full flex items-center justify-center shrink-0 mt-0.5 text-[11px] font-bold"
          :class="m.role === 'assistant'
            ? 'bg-brand-100 dark:bg-brand-900/40 text-brand-700 dark:text-brand-400'
            : 'bg-neutral-100 dark:bg-neutral-800 text-neutral-500'"
        >
          <span v-if="m.role === 'assistant'" class="i-lucide-bot w-3.5 h-3.5" />
          <span v-else class="i-lucide-user w-3.5 h-3.5" />
        </div>

        <!-- 气泡 -->
        <div
          class="max-w-[78%] flex flex-col gap-1.5"
          :class="m.role === 'assistant' ? 'items-end' : 'items-start'"
        >
          <!-- Thinking 折叠块 -->
          <details v-if="m.thinking" class="w-full">
            <summary
              class="cursor-pointer text-[11px] text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-300
                     select-none list-none flex items-center gap-1"
              :class="m.role === 'assistant' ? 'justify-end' : ''"
            >
              <span class="i-lucide-brain w-3 h-3" />
              思考过程
              <span class="i-lucide-chevron-down w-3 h-3" />
            </summary>
            <pre
              class="mt-1.5 text-[11px] whitespace-pre-wrap break-words leading-relaxed
                     bg-neutral-50 dark:bg-neutral-900/60 border border-neutral-200 dark:border-neutral-700
                     rounded-lg px-3 py-2 text-neutral-500 dark:text-neutral-400 font-mono"
            >{{ m.thinking }}</pre>
          </details>

          <!-- 正文气泡 -->
          <div
            class="rounded-2xl px-3.5 py-2.5 shadow-sm border leading-relaxed break-words"
            :class="m.role === 'assistant'
              ? 'bg-brand-50 dark:bg-brand-950/20 border-brand-100 dark:border-brand-900/40 text-neutral-800 dark:text-neutral-200'
              : 'bg-white dark:bg-neutral-800 border-neutral-200 dark:border-neutral-700 text-neutral-800 dark:text-neutral-200'"
          >
            <pre class="whitespace-pre-wrap font-sans text-sm leading-relaxed">{{ displayContent(m.id, m.rest) }}</pre>
            <!-- 长文展开/收起按钮 -->
            <button
              v-if="isLong(m.rest)"
              class="mt-1.5 text-[11px] text-brand-500 hover:text-brand-700 dark:text-brand-400 dark:hover:text-brand-300 font-medium"
              @click="toggle(m.id)"
            >
              {{ expanded[m.id] ? '收起 ↑' : `展开全文（${m.rest.length} 字）↓` }}
            </button>
          </div>

          <!-- 时间戳 -->
          <span
            v-if="m.timestamp"
            class="text-[10px] text-neutral-400 tabular-nums px-1"
          >
            {{ m.timestamp.replace('T', ' ').slice(0, 16) }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* details open 时旋转箭头 */
details[open] > summary .details-open\:rotate-90 {
  transform: rotate(90deg);
}
</style>
