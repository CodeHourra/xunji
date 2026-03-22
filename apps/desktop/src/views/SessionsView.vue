<script setup lang="ts">
import { computed, onActivated, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { NSpin, NEmpty, NButton, NCheckbox } from 'naive-ui'
import SessionToolbar from '../components/SessionToolbar.vue'
import SessionCard from '../components/SessionCard.vue'
import Pagination from '../components/Pagination.vue'
import { useSessionsStore } from '../stores/sessions'
import { useSearchStore } from '../stores/search'
import { useAnalysisQueueStore } from '../stores/analysisQueue'
import { api } from '../lib/tauri'
import { appendDistillHint } from '../lib/distillHints'

const sessions = useSessionsStore()
const router = useRouter()
const search = useSearchStore()
const queue = useAnalysisQueueStore()

// ── 批量分析完成统计（仅批量入口使用 callbacks 计数） ───────────────────────

const batchExpected = ref(0)
const batchFinished = ref(0)
const batchSuccess = ref(0)
const batchFail = ref(0)

/** 知识库卡片总数：用于区分「无笔记可搜」与「关键词无匹配」 */
const libraryCardTotal = ref<number | null>(null)

function resetBatchStats() {
  batchExpected.value = 0
  batchFinished.value = 0
  batchSuccess.value = 0
  batchFail.value = 0
}

function tryFinishBatchToast() {
  if (batchExpected.value === 0) return
  if (batchFinished.value < batchExpected.value) return
  if (batchFail.value === 0) {
    showToast(`批量分析完成，共处理 ${batchSuccess.value} 条`)
  } else {
    showToast(
      `分析完成：成功 ${batchSuccess.value}，失败 ${batchFail.value}`,
      batchFail.value > 0 ? 'error' : 'success',
    )
  }
  resetBatchStats()
}

// ── 批量选择 ─────────────────────────────────────────────────────────────────

/** 是否处于批量选择模式 */
const batchMode = ref(false)
/** 已选中的会话 ID Set */
const selectedIds = ref<Set<string>>(new Set())

// ── Toast ────────────────────────────────────────────────────────────────────

const toast = ref<{ msg: string; type: 'success' | 'error' | 'warning' } | null>(null)

function showToast(msg: string, type: 'success' | 'error' | 'warning' = 'success') {
  const text = type === 'error' ? appendDistillHint(msg) : msg
  toast.value = { msg: text, type }
  setTimeout(() => {
    toast.value = null
  }, type === 'error' ? 9000 : 4000)
}

// ── 生命周期 ─────────────────────────────────────────────────────────────────

onMounted(() => {
  void sessions.loadPage()
  void api
    .countAllCards()
    .then((n) => {
      libraryCardTotal.value = n
    })
    .catch(() => {
      libraryCardTotal.value = null
    })
})

onActivated(() => {
  void sessions.loadPage()
})

// ── 计算属性 ─────────────────────────────────────────────────────────────────

/**
 * 判断会话是否「待分析」（可参与批量分析）。
 * 已分析（含低价值 status=analyzed）和分析中的均不可选。
 */
function isPending(s: { cardId?: string | null; status: string }) {
  return !s.cardId && s.status !== 'analyzing' && s.status !== 'analyzed'
}

const unanalyzedCount = computed(() => sessions.items.filter(isPending).length)

const selectableItems = computed(() => sessions.items.filter(isPending))

const allSelected = computed(
  () =>
    selectableItems.value.length > 0 &&
    selectableItems.value.every((s) => selectedIds.value.has(s.id)),
)

const indeterminate = computed(
  () => selectedIds.value.size > 0 && !allSelected.value,
)

// 批量分析进行中：队列里仍有来自本次批量预期的任务未完成
const batchRunning = computed(
  () => batchExpected.value > 0 && batchFinished.value < batchExpected.value,
)

// ── 方法 ─────────────────────────────────────────────────────────────────────

/** 单条分析（从 SessionCard 触发） */
function onAnalyze(sessionId: string) {
  const s = sessions.items.find((x) => x.id === sessionId)
  const title = s?.cardTitle || s?.projectName || sessionId
  queue.enqueue(sessionId, title, {
    onLowValue: (result) => {
      showToast(
        `已判断为${result.value === 'none' ? '无价值' : '低价值'}：${result.reason ?? ''}`,
        'warning',
      )
    },
    onSuccess: (result) => {
      const card = result.card
      if (card) {
        showToast(`笔记已生成：${card.title}`)
        void router.push({
          name: 'session-detail',
          params: { sessionId },
          query: { cardId: card.id },
        })
      }
    },
    onError: (msg) => {
      showToast(msg, 'error')
    },
  })
}

function toggleBatchMode() {
  batchMode.value = !batchMode.value
  if (!batchMode.value) {
    selectedIds.value = new Set()
  }
}

function toggleSelectAll(checked: boolean) {
  if (checked) {
    selectedIds.value = new Set(sessions.items.filter(isPending).map((s) => s.id))
  } else {
    selectedIds.value = new Set()
  }
}

function onSelectionChange(id: string, checked: boolean) {
  const next = new Set(selectedIds.value)
  if (checked) {
    next.add(id)
  } else {
    next.delete(id)
  }
  selectedIds.value = next
}

/**
 * 批量分析：全部入队，由全局队列串行执行；通过 callbacks 统计完成后 Toast
 */
function startBatchAnalyze() {
  const ids = [...selectedIds.value]
  if (!ids.length) return

  resetBatchStats()

  ids.forEach((id) => {
    const s = sessions.items.find((x) => x.id === id)
    const title = s?.cardTitle || s?.projectName || id
    const enqueued = queue.enqueue(id, title, {
      onLowValue: () => {
        batchSuccess.value++
        batchFinished.value++
        tryFinishBatchToast()
      },
      onSuccess: () => {
        batchSuccess.value++
        batchFinished.value++
        tryFinishBatchToast()
      },
      onError: () => {
        batchFail.value++
        batchFinished.value++
        tryFinishBatchToast()
      },
    })
    if (enqueued) batchExpected.value++
  })

  batchMode.value = false
  selectedIds.value = new Set()
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
        class="fixed top-4 left-1/2 -translate-x-1/2 z-50 rounded-lg border px-3 py-2 shadow-lg flex items-center gap-2 text-sm"
        :class="{
          'border-red-200 bg-red-50 dark:bg-red-950/80 dark:border-red-800 text-red-800 dark:text-red-200': toast.type === 'error',
          'border-amber-200 bg-amber-50 dark:bg-amber-950/80 dark:border-amber-800 text-amber-800 dark:text-amber-200': toast.type === 'warning',
          'border-brand-200 bg-brand-50 dark:bg-brand-950/80 dark:border-brand-800 text-brand-800 dark:text-brand-200': toast.type === 'success',
        }"
      >
        <span
          :class="{
            'i-lucide-x-circle text-red-500': toast.type === 'error',
            'i-lucide-alert-triangle text-amber-500': toast.type === 'warning',
            'i-lucide-check-circle text-brand-500': toast.type === 'success',
          }"
          class="w-4 h-4"
        />
        {{ toast.msg }}
      </div>
    </Transition>

    <!-- 搜索结果（FTS 知识卡片） -->
    <div v-if="search.query.trim()" class="flex-1 min-h-0 overflow-y-auto space-y-3 pb-4">
      <div class="text-xs text-neutral-500 font-medium">搜索结果（{{ search.results.length }}）</div>
      <div v-if="search.searching && !search.results.length" class="flex justify-center py-16">
        <n-spin size="medium" />
      </div>
      <n-empty
        v-else-if="!search.searching && !search.results.length"
        class="py-12"
      >
        <template #default>
          <div class="text-center space-y-1 px-4">
            <template v-if="libraryCardTotal === 0">
              <p class="text-sm text-neutral-600 dark:text-neutral-300">知识库中暂无笔记</p>
              <p class="text-xs text-neutral-400">请先同步对话并完成提炼，再使用全文搜索。</p>
            </template>
            <template v-else-if="libraryCardTotal != null && libraryCardTotal > 0">
              <p class="text-sm text-neutral-600 dark:text-neutral-300">未找到包含该关键词的笔记</p>
              <p class="text-xs text-neutral-400">试试更短的关键词、同义词，或检查拼写。</p>
            </template>
            <template v-else>
              <p class="text-sm text-neutral-600 dark:text-neutral-300">未找到匹配内容</p>
            </template>
          </div>
        </template>
      </n-empty>
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
        <!-- 批量操作工具栏 -->
        <div class="flex items-center justify-between mb-2 min-h-8">
          <div class="flex items-center gap-3">
            <template v-if="batchMode">
              <n-checkbox
                :checked="allSelected"
                :indeterminate="indeterminate"
                @update:checked="toggleSelectAll"
              >
                <span class="text-xs text-neutral-600 dark:text-neutral-400">全选</span>
              </n-checkbox>
              <span v-if="selectedIds.size > 0" class="text-xs text-brand-600 dark:text-brand-400 font-medium">
                已选 {{ selectedIds.size }} 条
              </span>
            </template>
          </div>

          <div class="flex items-center gap-2">
            <template v-if="!batchMode">
              <n-button
                v-if="unanalyzedCount > 0"
                size="tiny"
                :disabled="batchRunning"
                @click="toggleBatchMode"
              >
                <span class="inline-flex items-center gap-1">
                  <span class="i-lucide-layers w-3 h-3" />
                  批量分析（{{ unanalyzedCount }}）
                </span>
              </n-button>
            </template>

            <template v-else>
              <n-button
                size="tiny"
                type="primary"
                :loading="batchRunning"
                :disabled="selectedIds.size === 0 || batchRunning"
                @click="startBatchAnalyze"
              >
                <span class="inline-flex items-center gap-1">
                  <span v-if="!batchRunning" class="i-lucide-sparkles w-3 h-3" />
                  {{ batchRunning ? `队列处理中 ${batchFinished}/${batchExpected}…` : `开始分析（${selectedIds.size}）` }}
                </span>
              </n-button>
              <n-button size="tiny" :disabled="batchRunning" @click="toggleBatchMode">
                取消
              </n-button>
            </template>
          </div>
        </div>

        <p v-if="batchRunning" class="text-xs text-neutral-500 mb-2">
          已加入全局分析队列，进度见右下角面板；请勿关闭应用。
        </p>

        <!-- 与会话列表区：整体悬浮块 + 渐变底，与 KnowledgeView 内容区同一套层次逻辑；暗色同样保留外阴影表深度 -->
        <div
          class="flex-1 min-h-0 flex flex-col overflow-hidden rounded-xl border border-emerald-200/40 dark:border-emerald-900/35 bg-gradient-to-br from-emerald-50/90 via-teal-50/50 to-cyan-50/40 dark:from-emerald-950/25 dark:via-neutral-950 dark:to-slate-950/90 shadow-[0_2px_14px_-2px_rgba(16,185,129,0.12)] dark:shadow-[0_2px_16px_-4px_rgba(0,0,0,0.4)]"
        >
          <div class="flex-1 min-h-0 overflow-y-auto p-3 sm:p-4 space-y-2.5 pb-3">
            <SessionCard
              v-for="s in sessions.items"
              :key="s.id"
              :session="s"
              :selectable="batchMode"
              :selected="selectedIds.has(s.id)"
              @analyze="onAnalyze"
              @update:selected="onSelectionChange"
            />
          </div>
        </div>

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
