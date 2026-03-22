<script setup lang="ts">
/**
 * 知识库 —— 笔记卡片列表 / 卡片网格双视图，内容区固定高度可滚动，分页贴底。
 *
 * 视图模式持久化：localStorage key `xunji:knowledgeViewMode`
 */
import { onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { NSpin, NEmpty, NTag, NRadioGroup, NRadioButton } from 'naive-ui'
import { getCardTypeLabel } from '@xunji/shared'
import { api } from '../lib/tauri'
import type { CardSummary } from '../types'
import { useFiltersStore } from '../stores/filters'
import Pagination from '../components/Pagination.vue'

const VIEW_MODE_KEY = 'xunji:knowledgeViewMode'
type ViewMode = 'list' | 'card'

const router = useRouter()
const filters = useFiltersStore()
const items = ref<CardSummary[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(20)
const loading = ref(false)

const viewMode = ref<ViewMode>('list')

watch(viewMode, (v) => {
  localStorage.setItem(VIEW_MODE_KEY, v)
})

async function load() {
  loading.value = true
  try {
    const r = await api.listCards({
      cardType: filters.cardType || undefined,
      tags: filters.selectedTags.length ? [...filters.selectedTags] : undefined,
      page: page.value,
      pageSize: pageSize.value,
    })
    items.value = r.items
    total.value = r.total
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  const raw = localStorage.getItem(VIEW_MODE_KEY)
  if (raw === 'list' || raw === 'card') viewMode.value = raw
  void load()
})

watch(
  [() => filters.cardType, () => filters.selectedTags.length],
  () => {
    page.value = 1
    void load()
  },
)

function open(c: CardSummary) {
  void router.push({
    name: 'session-detail',
    params: { sessionId: c.sessionId },
    query: { cardId: c.id },
  })
}

function setPage(p: number) {
  page.value = p
  void load()
}

function setPageSize(n: number) {
  pageSize.value = n
  page.value = 1
  void load()
}

function formatTime(iso: string) {
  return iso?.replace('T', ' ').slice(0, 16) ?? '—'
}
</script>

<template>
  <!--
    布局：整列 flex + min-h-0，保证主内容区占满剩余高度且内部滚动；
    分页 shrink-0 固定在可视区域底部（相对 main 视口）。
  -->
  <div class="flex flex-col h-full min-h-0 max-w-5xl mx-auto w-full px-5 pt-5">
    <!-- 顶栏：标题 + 视图切换 -->
    <header class="shrink-0 flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between mb-3">
      <div>
        <h1 class="text-base font-semibold text-neutral-900 dark:text-neutral-100 tracking-tight">
          知识库
        </h1>
        <p class="text-[11px] text-emerald-700/70 dark:text-emerald-400/80 mt-0.5">
          共 {{ total }} 条 · 列表 / 卡片切换
        </p>
      </div>
      <n-radio-group v-model:value="viewMode" size="small" class="shrink-0">
        <n-radio-button value="list">
          <span class="inline-flex items-center gap-1.5">
            <span class="i-lucide-list w-3.5 h-3.5" />
            列表
          </span>
        </n-radio-button>
        <n-radio-button value="card">
          <span class="inline-flex items-center gap-1.5">
            <span class="i-lucide-layout-grid w-3.5 h-3.5" />
            卡片
          </span>
        </n-radio-button>
      </n-radio-group>
    </header>

    <!-- 内容区：清新底色 + 内部条目统一悬浮阴影（无 hover 抬升） -->
    <div
      class="flex-1 min-h-0 flex flex-col overflow-hidden rounded-xl border border-emerald-200/40 dark:border-emerald-900/35 bg-gradient-to-br from-emerald-50/90 via-teal-50/50 to-cyan-50/40 dark:from-emerald-950/25 dark:via-neutral-950 dark:to-slate-950/90 shadow-[inset_0_1px_0_0_rgba(255,255,255,0.6)] dark:shadow-none"
    >
      <div class="flex-1 min-h-0 overflow-y-auto p-3 sm:p-4">
        <div v-if="loading" class="flex items-center justify-center py-24">
          <n-spin size="medium" />
        </div>

        <n-empty v-else-if="!items.length" description="暂无知识卡片" class="py-16" />

        <!-- 列表视图：紧凑行高 + 统一悬浮块（静态阴影，hover 仅微调描边） -->
        <div v-else-if="viewMode === 'list'" class="space-y-2">
          <button
            v-for="c in items"
            :key="c.id"
            type="button"
            class="group w-full text-left flex gap-0 rounded-lg border border-white/70 dark:border-emerald-900/50 bg-white/85 dark:bg-neutral-900/75 shadow-[0_2px_14px_-2px_rgba(16,185,129,0.18)] dark:shadow-[0_2px_16px_-4px_rgba(0,0,0,0.45)] backdrop-blur-sm hover:border-emerald-300/55 dark:hover:border-emerald-700/45 transition-[border-color] duration-150 overflow-hidden focus:outline-none focus-visible:ring-2 focus-visible:ring-emerald-400/40"
            @click="open(c)"
          >
            <div
              class="w-1 shrink-0 bg-gradient-to-b from-emerald-400 to-teal-500"
              aria-hidden="true"
            />
            <div class="flex-1 min-w-0 py-2 pl-2.5 pr-3">
              <div class="flex items-center justify-between gap-2">
                <h2 class="text-[13px] font-semibold text-neutral-800 dark:text-neutral-100 line-clamp-1 leading-tight">
                  {{ c.title }}
                </h2>
                <time
                  class="text-[10px] text-emerald-700/65 dark:text-emerald-500/80 font-mono tabular-nums shrink-0"
                  :datetime="c.updatedAt"
                >
                  {{ formatTime(c.updatedAt) }}
                </time>
              </div>
              <div class="flex flex-wrap items-center gap-x-2 gap-y-0.5 mt-1">
                <n-tag v-if="c.type" size="tiny" :bordered="false" type="success" class="!text-[10px] !px-1.5 !py-0">
                  {{ getCardTypeLabel(c.type) }}
                </n-tag>
                <n-tag
                  v-if="c.value"
                  size="tiny"
                  :bordered="false"
                  :type="c.value === 'high' ? 'success' : c.value === 'medium' ? 'warning' : 'default'"
                  class="!text-[10px] !px-1.5 !py-0"
                >
                  {{ c.value }}
                </n-tag>
                <span
                  v-if="c.projectName"
                  class="text-[10px] text-neutral-600 dark:text-neutral-400 truncate max-w-[10rem]"
                >
                  {{ c.projectName }}
                </span>
                <span v-if="c.sourceName" class="text-[10px] text-neutral-400 dark:text-neutral-500">
                  · {{ c.sourceName }}
                </span>
              </div>
              <p class="text-[11px] text-neutral-600/90 dark:text-neutral-400/95 line-clamp-2 mt-1.5 leading-snug">
                {{ c.summary || '暂无摘要' }}
              </p>
            </div>
          </button>
        </div>

        <!-- 卡片视图：更高密度网格，与列表同一套悬浮与配色 -->
        <div
          v-else
          class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2.5"
        >
          <button
            v-for="c in items"
            :key="c.id"
            type="button"
            class="group flex flex-col text-left rounded-xl border border-white/70 dark:border-emerald-900/50 bg-white/85 dark:bg-neutral-900/75 min-h-[132px] p-3 shadow-[0_2px_14px_-2px_rgba(16,185,129,0.18)] dark:shadow-[0_2px_16px_-4px_rgba(0,0,0,0.45)] backdrop-blur-sm hover:border-emerald-300/55 dark:hover:border-emerald-700/45 transition-[border-color] duration-150 focus:outline-none focus-visible:ring-2 focus-visible:ring-emerald-400/40"
            @click="open(c)"
          >
            <div class="flex items-center justify-between gap-2">
              <div class="flex flex-wrap gap-1 min-w-0">
                <n-tag v-if="c.type" size="tiny" :bordered="false" type="success" class="!text-[10px] !px-1.5 !py-0">
                  {{ getCardTypeLabel(c.type) }}
                </n-tag>
                <n-tag
                  v-if="c.value"
                  size="tiny"
                  :bordered="false"
                  :type="c.value === 'high' ? 'success' : c.value === 'medium' ? 'warning' : 'default'"
                  class="!text-[10px] !px-1.5 !py-0"
                >
                  {{ c.value }}
                </n-tag>
              </div>
              <time
                class="text-[10px] text-emerald-700/65 dark:text-emerald-500/80 font-mono tabular-nums shrink-0"
                :datetime="c.updatedAt"
              >
                {{ formatTime(c.updatedAt) }}
              </time>
            </div>
            <h2 class="text-[13px] font-semibold text-neutral-800 dark:text-neutral-100 line-clamp-2 mt-1.5 leading-snug">
              {{ c.title }}
            </h2>
            <p class="text-[11px] text-neutral-600/90 dark:text-neutral-400/95 line-clamp-2 mt-1 flex-1 leading-snug">
              {{ c.summary || '暂无摘要' }}
            </p>
            <div class="mt-1.5 pt-1.5 border-t border-emerald-100/80 dark:border-emerald-900/40 flex items-center justify-between gap-2 text-[10px] text-neutral-500 dark:text-neutral-400">
              <span class="truncate">{{ c.projectName || '—' }}</span>
              <span class="i-lucide-chevron-right w-3 h-3 text-emerald-400/80 shrink-0" />
            </div>
          </button>
        </div>
      </div>
    </div>

    <!-- 分页：固定在内容区下方、不随列表滚动 -->
    <footer
      v-if="total > 0"
      class="shrink-0 py-2 mt-1.5 border-t border-emerald-100/70 dark:border-emerald-900/40 bg-emerald-50/40 dark:bg-neutral-950/90 backdrop-blur-sm"
    >
      <Pagination
        :page="page"
        :page-size="pageSize"
        :total="total"
        @update:page="setPage"
        @update:page-size="setPageSize"
      />
    </footer>
  </div>
</template>
