<script setup lang="ts">
defineProps<{
  sourceName: string | null
  projectName: string | null
  createdAt: string
  promptTokens: number
  completionTokens: number
  costYuan: number
}>()
</script>

<template>
  <div class="text-xs text-neutral-500 space-y-1 border-t border-neutral-200 dark:border-neutral-800 pt-3 mt-3">
    <div>
      💬 来自 {{ sourceName ?? '—' }} / {{ projectName ?? '—' }} · 📅 {{ createdAt?.slice(0, 16) }}
    </div>
    <div>
      提炼消耗：{{ promptTokens + completionTokens }} tokens
      · ¥{{ costYuan.toFixed(4) }}
    </div>
  </div>
</template>
