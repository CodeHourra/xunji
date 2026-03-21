<script setup lang="ts">
import { NInput, NButton } from 'naive-ui'
import { useSessionsStore } from '../stores/sessions'
import { useSearchStore } from '../stores/search'
import { useFiltersStore } from '../stores/filters'

const sessions = useSessionsStore()
const search = useSearchStore()
const filters = useFiltersStore()

async function refresh() {
  search.clear()
  await sessions.loadPage()
}

/**
 * 切换已分析/全部过滤。
 * 点击已选中的按钮 → 恢复全部；点击未选中的按钮 → 激活该过滤
 */
function toggleStatus(val: 'analyzed' | 'pending') {
  filters.statusFilter = filters.statusFilter === val ? '' : val
  sessions.page = 1
  void sessions.loadPage()
}
</script>

<template>
  <div class="flex flex-col gap-3 mb-4 shrink-0">
    <!-- 搜索行 -->
    <div class="flex items-center gap-2">
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

      <!-- 刷新 -->
      <n-button size="medium" @click="refresh">
        <span class="inline-flex items-center gap-1.5">
          <span class="i-lucide-rotate-ccw w-3.5 h-3.5" />
          刷新
        </span>
      </n-button>

      <!-- 已分析过滤按钮 -->
      <n-button
        size="medium"
        :type="filters.statusFilter === 'analyzed' ? 'primary' : 'default'"
        @click="toggleStatus('analyzed')"
      >
        <span class="inline-flex items-center gap-1.5">
          <span
            :class="filters.statusFilter === 'analyzed'
              ? 'i-lucide-check-circle w-3.5 h-3.5'
              : 'i-lucide-circle w-3.5 h-3.5 opacity-50'"
          />
          已分析
        </span>
      </n-button>

      <!-- 待分析过滤按钮 -->
      <n-button
        size="medium"
        :type="filters.statusFilter === 'pending' ? 'warning' : 'default'"
        @click="toggleStatus('pending')"
      >
        <span class="inline-flex items-center gap-1.5">
          <span
            :class="filters.statusFilter === 'pending'
              ? 'i-lucide-clock w-3.5 h-3.5'
              : 'i-lucide-clock w-3.5 h-3.5 opacity-50'"
          />
          待分析
        </span>
      </n-button>
    </div>

    <!-- 状态行 -->
    <div class="flex items-center justify-between text-xs text-neutral-500 dark:text-neutral-400">
      <div class="flex items-center gap-2">
        <span class="flex items-center gap-1.5">
          <span class="i-lucide-list w-3.5 h-3.5" />
          共 <strong class="text-neutral-700 dark:text-neutral-200 mx-0.5">{{ sessions.total }}</strong> 条
          <template v-if="filters.statusFilter === 'analyzed'">
            <span class="text-emerald-500 font-medium">已分析</span>
          </template>
          <template v-else-if="filters.statusFilter === 'pending'">
            <span class="text-amber-500 font-medium">待分析</span>
          </template>
          <template v-else>
            会话
          </template>
        </span>

        <!-- 当前过滤激活时显示清除链接 -->
        <button
          v-if="filters.statusFilter"
          class="text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-200 flex items-center gap-0.5 transition-colors"
          @click="filters.statusFilter = ''; void sessions.loadPage()"
        >
          <span class="i-lucide-x w-3 h-3" />
          清除过滤
        </button>
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
