<script setup lang="ts">
defineProps<{
  sourceName: string | null
  projectName: string | null
  createdAt: string
  promptTokens: number
  completionTokens: number
  costYuan: number
  /** 来源会话路径（优先 raw_path，后端 COALESCE） */
  sourceSessionPath?: string | null
  /** 数据源侧会话 ID（非本应用 DB 主键） */
  sourceSessionExternalId?: string | null
}>()
</script>

<template>
  <div class="text-xs text-neutral-500 space-y-1 border-t border-neutral-200 dark:border-neutral-800 pt-3 mt-3">
    <div>
      💬 来自 {{ sourceName ?? '—' }} / {{ projectName ?? '—' }} · 📅 {{ createdAt?.slice(0, 16) }}
    </div>
    <div
      v-if="sourceSessionPath || sourceSessionExternalId"
      class="text-[11px] text-neutral-400 dark:text-neutral-500 space-y-0.5 pl-0.5"
    >
      <div v-if="sourceSessionPath" class="break-all">
        <span class="text-neutral-400">来源会话路径</span>
        {{ sourceSessionPath }}
      </div>
      <div v-if="sourceSessionExternalId" class="font-mono break-all">
        <span class="text-neutral-400 font-sans">来源会话 ID</span>
        {{ sourceSessionExternalId }}
      </div>
    </div>
    <div>
      提炼消耗：{{ promptTokens + completionTokens }} tokens
      · ¥{{ costYuan.toFixed(4) }}
    </div>
  </div>
</template>
