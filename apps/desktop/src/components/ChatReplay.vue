<script setup lang="ts">
import { computed } from 'vue'
import type { Message } from '../types'

const props = defineProps<{
  messages: Message[]
}>()

/** 将单条消息拆成「普通正文」与「可折叠 thinking 段」 */
const bubbles = computed(() => {
  return props.messages.map((m) => {
    const raw = m.content || ''
    const thinkingMatch = raw.match(/<thinking>([\s\S]*?)<\/thinking>/i)
    const thinking = thinkingMatch ? thinkingMatch[1].trim() : null
    const rest = thinkingMatch ? raw.replace(thinkingMatch[0], '').trim() : raw
    return { ...m, rest, thinking }
  })
})
</script>

<template>
  <div class="space-y-3 text-sm">
    <div
      v-for="m in bubbles"
      :key="m.id"
      class="flex"
      :class="m.role === 'assistant' ? 'justify-end' : 'justify-start'"
    >
      <div
        class="max-w-[85%] rounded-2xl px-3 py-2 shadow-sm border"
        :class="m.role === 'assistant'
          ? 'bg-emerald-900/10 border-emerald-800/30 text-neutral-900 dark:text-neutral-100'
          : 'bg-white dark:bg-neutral-900 border-neutral-200 dark:border-neutral-700'"
      >
        <div class="text-[10px] uppercase tracking-wide opacity-60 mb-1">
          {{ m.role }}
        </div>
        <details v-if="m.thinking" class="mb-2">
          <summary class="cursor-pointer text-xs text-neutral-500">
            thinking（点击展开）
          </summary>
          <pre class="mt-1 text-xs whitespace-pre-wrap opacity-80">{{ m.thinking }}</pre>
        </details>
        <div class="whitespace-pre-wrap break-words">
          {{ m.rest }}
        </div>
      </div>
    </div>
  </div>
</template>
