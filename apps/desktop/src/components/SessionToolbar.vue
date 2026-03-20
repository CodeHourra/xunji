<script setup lang="ts">
import { NInput, NButton } from 'naive-ui'
import { useSessionsStore } from '../stores/sessions'
import { useSearchStore } from '../stores/search'

const sessions = useSessionsStore()
const search = useSearchStore()

async function refresh() {
  search.clear()
  await sessions.loadPage()
}
</script>

<template>
  <div class="flex flex-col gap-3 mb-4 shrink-0">
    <!-- 搜索行 -->
    <div class="flex items-center gap-3">
      <n-input
        v-model:value="search.query"
        placeholder="搜索对话…"
        clearable
        size="medium"
        class="flex-1"
        @clear="refresh"
      >
        <template #prefix>
          <span class="i-lucide-search w-4 h-4 opacity-40" />
        </template>
        <template #suffix>
          <span v-if="search.searching" class="i-lucide-loader-2 w-3.5 h-3.5 animate-spin opacity-40" />
        </template>
      </n-input>
      <n-button @click="refresh">
        <span class="inline-flex items-center gap-1.5">
          <span class="i-lucide-rotate-ccw w-3.5 h-3.5" />
          刷新
        </span>
      </n-button>
    </div>

    <!-- 状态行 -->
    <div class="flex items-center justify-between text-xs text-neutral-500 dark:text-neutral-400">
      <div class="flex items-center gap-1.5">
        <span class="i-lucide-list w-3.5 h-3.5" />
        <span>共 <strong class="text-neutral-700 dark:text-neutral-200">{{ sessions.total }}</strong> 条会话</span>
      </div>
      <span v-if="sessions.error" class="text-red-500 flex items-center gap-1">
        <span class="i-lucide-alert-circle w-3 h-3" />
        {{ sessions.error }}
      </span>
      <span v-if="search.searchError" class="text-red-500 flex items-center gap-1">
        <span class="i-lucide-alert-circle w-3 h-3" />
        {{ search.searchError }}
      </span>
    </div>
  </div>
</template>
