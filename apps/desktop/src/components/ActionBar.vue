<script setup lang="ts">
import { NButton, NButtonGroup } from 'naive-ui'

defineProps<{
  mode: 'note' | 'chat'
  /** 重新分析进行中 */
  analyzing?: boolean
}>()

const emit = defineEmits<{
  'update:mode': [m: 'note' | 'chat']
  close: []
  reanalyze: []
}>()
</script>

<template>
  <div class="flex flex-wrap items-center justify-between gap-2 pt-3 border-t border-neutral-200 dark:border-neutral-800">
    <div class="flex flex-wrap gap-2">
      <n-button-group size="small">
        <n-button
          :type="mode === 'note' ? 'primary' : 'default'"
          :disabled="analyzing"
          @click="emit('update:mode', 'note')"
        >
          <span class="inline-flex items-center gap-1">
            <span class="i-lucide-book-open w-3.5 h-3.5" />
            查看笔记
          </span>
        </n-button>
        <n-button
          :type="mode === 'chat' ? 'primary' : 'default'"
          :disabled="analyzing"
          @click="emit('update:mode', 'chat')"
        >
          <span class="inline-flex items-center gap-1">
            <span class="i-lucide-messages-square w-3.5 h-3.5" />
            查看对话
          </span>
        </n-button>
      </n-button-group>
      <!-- 重新分析：分析中时显示 loading 状态并禁用 -->
      <n-button
        size="small"
        type="warning"
        secondary
        :loading="analyzing"
        :disabled="analyzing"
        @click="emit('reanalyze')"
      >
        <span v-if="!analyzing" class="inline-flex items-center gap-1">
          <span class="i-lucide-refresh-cw w-3.5 h-3.5" />
          重新分析
        </span>
        <span v-else>分析中…</span>
      </n-button>
    </div>
    <n-button size="small" quaternary :disabled="analyzing" @click="emit('close')">
      关闭
    </n-button>
  </div>
</template>
