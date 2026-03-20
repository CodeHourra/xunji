<script setup lang="ts">
import { NButton, NButtonGroup } from 'naive-ui'

defineProps<{
  mode: 'note' | 'chat'
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
          @click="emit('update:mode', 'note')"
        >
          <span class="inline-flex items-center gap-1">
            <span class="i-lucide-book-open w-3.5 h-3.5" />
            查看笔记
          </span>
        </n-button>
        <n-button
          :type="mode === 'chat' ? 'primary' : 'default'"
          @click="emit('update:mode', 'chat')"
        >
          <span class="inline-flex items-center gap-1">
            <span class="i-lucide-messages-square w-3.5 h-3.5" />
            查看对话
          </span>
        </n-button>
      </n-button-group>
      <n-button size="small" type="warning" secondary @click="emit('reanalyze')">
        <span class="inline-flex items-center gap-1">
          <span class="i-lucide-refresh-cw w-3.5 h-3.5" />
          重新分析
        </span>
      </n-button>
    </div>
    <n-button size="small" quaternary @click="emit('close')">
      关闭
    </n-button>
  </div>
</template>
