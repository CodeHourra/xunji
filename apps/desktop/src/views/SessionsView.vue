<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { NSpin, NEmpty, NButton } from 'naive-ui'
import SessionToolbar from '../components/SessionToolbar.vue'
import SessionCard from '../components/SessionCard.vue'
import Pagination from '../components/Pagination.vue'
import { useSessionsStore } from '../stores/sessions'
import { useSearchStore } from '../stores/search'
import { api } from '../lib/tauri'

const sessions = useSessionsStore()
const router = useRouter()
const search = useSearchStore()
const analyzing = ref<string | null>(null)
const toast = ref<string | null>(null)

onMounted(() => {
  void sessions.loadPage()
})

function showToast(msg: string) {
  toast.value = msg
  setTimeout(() => { toast.value = null }, 4000)
}

async function onAnalyze(sessionId: string) {
  analyzing.value = sessionId
  try {
    const card = await api.distillSession(sessionId)
    showToast(`笔记已生成：${card.title}`)
    await sessions.loadPage()
    void router.push({
      name: 'session-detail',
      params: { sessionId },
      query: { cardId: card.id },
    })
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    showToast(msg)
    await sessions.loadPage()
  } finally {
    analyzing.value = null
  }
}

function openSearchHit(cardId: string, sessionId: string) {
  void router.push({
    name: 'session-detail',
    params: { sessionId },
    query: { cardId },
  })
}
</script>

<template>
  <!-- 整体用 flex-col + h-full 撑满父容器，列表区域自动填充并内部滚动 -->
  <div class="flex flex-col h-full px-5 pt-5 max-w-5xl mx-auto w-full">
    <SessionToolbar />

    <!-- Toast -->
    <Transition
      enter-active-class="transition duration-200 ease-out"
      enter-from-class="-translate-y-3 opacity-0"
      enter-to-class="translate-y-0 opacity-100"
      leave-active-class="transition duration-150 ease-in"
      leave-from-class="translate-y-0 opacity-100"
      leave-to-class="-translate-y-3 opacity-0"
    >
      <div
        v-if="toast"
        class="fixed top-4 left-1/2 -translate-x-1/2 z-50 rounded-lg border border-brand-200 bg-brand-50 dark:bg-brand-950/80 dark:border-brand-800 px-3 py-2 shadow-lg flex items-center gap-2 text-sm text-brand-800 dark:text-brand-200"
      >
        <span class="i-lucide-check-circle w-4 h-4 text-brand-500" />
        {{ toast }}
      </div>
    </Transition>

    <!-- 搜索结果 -->
    <div v-if="search.query.trim()" class="flex-1 min-h-0 overflow-y-auto space-y-3 pb-4">
      <div class="text-xs text-neutral-500 font-medium">搜索结果（{{ search.results.length }}）</div>
      <n-empty v-if="!search.results.length" description="未找到匹配内容" class="py-12" />
      <div
        v-for="c in search.results"
        :key="c.id"
        class="rounded-lg border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900 p-3 cursor-pointer hover:border-brand-300 dark:hover:border-brand-800 transition-all group"
        @click="openSearchHit(c.id, c.sessionId)"
      >
        <div class="flex items-center justify-between gap-3">
          <div class="min-w-0">
            <div class="text-sm font-medium text-neutral-800 dark:text-neutral-200 group-hover:text-brand-600 dark:group-hover:text-brand-400 truncate">{{ c.title }}</div>
            <div class="text-xs text-neutral-500 line-clamp-1 mt-0.5">{{ c.summary }}</div>
          </div>
          <span class="i-lucide-arrow-right w-4 h-4 text-neutral-400 group-hover:text-brand-500 shrink-0" />
        </div>
      </div>
    </div>

    <!-- 会话列表 -->
    <template v-else>
      <div v-if="sessions.loading" class="flex-1 flex items-center justify-center">
        <n-spin size="medium" />
      </div>

      <div v-else-if="!sessions.items.length" class="flex-1 flex items-center justify-center">
        <n-empty description="暂无会话，请先同步">
          <template #extra>
            <n-button type="primary" @click="sessions.syncAll()">
              <span class="inline-flex items-center gap-1.5">
                <span class="i-lucide-refresh-cw w-3.5 h-3.5" />
                立即同步
              </span>
            </n-button>
          </template>
        </n-empty>
      </div>

      <template v-else>
        <!-- 可滚动列表区域 -->
        <div class="flex-1 min-h-0 overflow-y-auto space-y-2 pb-2">
          <SessionCard
            v-for="s in sessions.items"
            :key="s.id"
            :session="s"
            @analyze="onAnalyze"
          />
        </div>

        <!-- 分页固定在底部 -->
        <div class="shrink-0 py-3 border-t border-neutral-200 dark:border-neutral-800">
          <Pagination
            :page="sessions.page"
            :page-size="sessions.pageSize"
            :total="sessions.total"
            @update:page="sessions.setPage"
            @update:page-size="sessions.setPageSize"
          />
        </div>
      </template>
    </template>
  </div>
</template>
