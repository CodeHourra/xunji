<script setup lang="ts">
import { computed, onActivated, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { NSpin, NEmpty, NButton, NCheckbox, NProgress } from 'naive-ui'
import SessionToolbar from '../components/SessionToolbar.vue'
import SessionCard from '../components/SessionCard.vue'
import Pagination from '../components/Pagination.vue'
import { useSessionsStore } from '../stores/sessions'
import { useSearchStore } from '../stores/search'
import { api } from '../lib/tauri'

const sessions = useSessionsStore()
const router = useRouter()
const search = useSearchStore()

// ── 单条分析状态 ─────────────────────────────────────────────────────────────

/** 当前单条正在分析的会话 ID（单条模式） */
const singleAnalyzing = ref<string | null>(null)

// ── 批量分析状态 ─────────────────────────────────────────────────────────────

/** 是否处于批量选择模式 */
const batchMode = ref(false)
/** 已选中的会话 ID Set */
const selectedIds = ref<Set<string>>(new Set())
/** 批量分析总数 */
const batchTotal = ref(0)
/** 批量分析已完成数（含失败） */
const batchDone = ref(0)
/** 批量分析是否进行中 */
const batchRunning = ref(false)

// ── Toast ────────────────────────────────────────────────────────────────────

const toast = ref<{ msg: string; type: 'success' | 'error' | 'warning' } | null>(null)

function showToast(msg: string, type: 'success' | 'error' | 'warning' = 'success') {
  toast.value = { msg, type }
  setTimeout(() => { toast.value = null }, 4000)
}

// ── 生命周期 ─────────────────────────────────────────────────────────────────

onMounted(() => {
  void sessions.loadPage()
})

// 从详情页返回列表时（keep-alive 场景），自动刷新以获取后端最新状态
onActivated(() => {
  void sessions.loadPage()
})

// ── 计算属性 ─────────────────────────────────────────────────────────────────

/**
 * 判断会话是否"待分析"（可参与批量分析）。
 * 已分析（含低价值 status=analyzed）和分析中的均不可选。
 */
function isPending(s: { cardId?: string | null; status: string }) {
  return !s.cardId && s.status !== 'analyzing' && s.status !== 'analyzed'
}

/** 当前页中未分析（待分析）的会话数量 */
const unanalyzedCount = computed(
  () => sessions.items.filter(isPending).length,
)

/** 当前页可选（待分析）的会话列表 */
const selectableItems = computed(() =>
  sessions.items.filter(isPending),
)

/** 是否全选（基于可选列表） */
const allSelected = computed(
  () =>
    selectableItems.value.length > 0 &&
    selectableItems.value.every((s) => selectedIds.value.has(s.id)),
)

/** 是否半选（基于可选列表） */
const indeterminate = computed(
  () => selectedIds.value.size > 0 && !allSelected.value,
)

const batchProgress = computed(() =>
  batchTotal.value > 0 ? Math.round((batchDone.value / batchTotal.value) * 100) : 0,
)

// ── 方法 ─────────────────────────────────────────────────────────────────────

/** 单条分析（从 SessionCard 触发）*/
async function onAnalyze(sessionId: string) {
  singleAnalyzing.value = sessionId
  // 立即更新状态为 analyzing（乐观更新，提升体感）
  sessions.patchItem(sessionId, { status: 'analyzing' })
  try {
    const result = await api.distillSession(sessionId)

    if (result.isLowValue) {
      // 低/无价值：更新为已分析状态（无卡片），显示提示而非错误
      sessions.patchItem(sessionId, {
        status: 'analyzed',
        value: result.value,
        cardId: null,
        cardTitle: null,
        cardSummary: result.reason ?? null,
        cardType: null,
        cardTags: null,
      })
      showToast(
        `已判断为${result.value === 'none' ? '无价值' : '低价值'}：${result.reason ?? ''}`,
        'warning',
      )
    } else {
      // 中/高价值：原地更新会话列表行（状态 + 卡片摘要信息）
      const card = result.card!
      sessions.patchItem(sessionId, {
        status: 'analyzed',
        value: card.value ?? null,
        cardId: card.id,
        cardTitle: card.title,
        cardSummary: card.summary ?? null,
        cardType: card.type ?? null,
        cardTags: card.tags?.join(',') ?? null,
      })
      showToast(`笔记已生成：${card.title}`)
      void router.push({
        name: 'session-detail',
        params: { sessionId },
        query: { cardId: card.id },
      })
    }
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    sessions.patchItem(sessionId, { status: 'error' })
    showToast(msg, 'error')
  } finally {
    singleAnalyzing.value = null
  }
}

/** 切换批量选择模式 */
function toggleBatchMode() {
  batchMode.value = !batchMode.value
  if (!batchMode.value) {
    selectedIds.value = new Set()
  }
}

/** 全选 / 取消全选（仅选未分析的会话） */
function toggleSelectAll(checked: boolean) {
  if (checked) {
    // 只选当前页未分析的会话（无卡片且非 analyzing 状态）
        selectedIds.value = new Set(
          sessions.items.filter(isPending).map((s) => s.id),
        )
  } else {
    selectedIds.value = new Set()
  }
}

/**
 * 单条选中状态变更（SessionCard emit）。
 * 每次创建新 Set 实例，确保 Vue 3 响应式正确触发。
 */
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
 * 批量分析已选会话（顺序执行，避免 API 过载）
 * 每条完成后原地更新对应卡片状态
 */
async function startBatchAnalyze() {
  const ids = [...selectedIds.value]
  if (!ids.length) return

  batchRunning.value = true
  batchTotal.value = ids.length
  batchDone.value = 0
  let successCount = 0
  let failCount = 0

  // 标记所有选中为 analyzing（乐观 UI）
  ids.forEach((id) => sessions.patchItem(id, { status: 'analyzing' }))

  for (const id of ids) {
    try {
      const result = await api.distillSession(id)

      if (result.isLowValue) {
        // 低/无价值：标记为已分析，不计入失败
        sessions.patchItem(id, {
          status: 'analyzed',
          value: result.value,
          cardId: null,
          cardTitle: null,
          cardSummary: result.reason ?? null,
          cardType: null,
          cardTags: null,
        })
        successCount++
      } else {
        const card = result.card!
        sessions.patchItem(id, {
          status: 'analyzed',
          value: card.value ?? null,
          cardId: card.id,
          cardTitle: card.title,
          cardSummary: card.summary ?? null,
          cardType: card.type ?? null,
          cardTags: card.tags?.join(',') ?? null,
        })
        successCount++
      }
    } catch {
      sessions.patchItem(id, { status: 'error' })
      failCount++
    } finally {
      batchDone.value++
    }
  }

  batchRunning.value = false
  selectedIds.value = new Set()
  batchMode.value = false

  if (failCount === 0) {
    showToast(`批量分析完成，共生成 ${successCount} 篇笔记`)
  } else {
    showToast(`分析完成：成功 ${successCount}，失败 ${failCount}`, failCount > 0 ? 'error' : 'success')
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
        <!-- 批量操作工具栏 -->
        <div class="flex items-center justify-between mb-2 min-h-8">
          <!-- 左侧：全选 + 已选提示 -->
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

          <!-- 右侧：批量分析控制按钮 -->
          <div class="flex items-center gap-2">
            <template v-if="!batchMode">
              <!-- 非批量模式：显示「批量分析」入口（仅当有未分析会话时） -->
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
              <!-- 批量模式：开始分析 + 取消 -->
              <n-button
                size="tiny"
                type="primary"
                :loading="batchRunning"
                :disabled="selectedIds.size === 0 || batchRunning"
                @click="startBatchAnalyze"
              >
                <span class="inline-flex items-center gap-1">
                  <span v-if="!batchRunning" class="i-lucide-sparkles w-3 h-3" />
                  {{ batchRunning ? `分析中 ${batchDone}/${batchTotal}` : `开始分析（${selectedIds.size}）` }}
                </span>
              </n-button>
              <n-button size="tiny" :disabled="batchRunning" @click="toggleBatchMode">
                取消
              </n-button>
            </template>
          </div>
        </div>

        <!-- 批量进度条 -->
        <Transition
          enter-active-class="transition duration-200 ease-out"
          enter-from-class="opacity-0 -translate-y-1"
          enter-to-class="opacity-100 translate-y-0"
          leave-active-class="transition duration-150 ease-in"
          leave-from-class="opacity-100 translate-y-0"
          leave-to-class="opacity-0 -translate-y-1"
        >
          <div v-if="batchRunning" class="mb-2">
            <n-progress
              type="line"
              :percentage="batchProgress"
              :show-indicator="false"
              :height="4"
              border-radius="2px"
              :color="'var(--brand-500, #6366f1)'"
            />
            <p class="text-xs text-neutral-400 mt-1">
              正在分析第 {{ batchDone + 1 }} / {{ batchTotal }} 条…
            </p>
          </div>
        </Transition>

        <!-- 可滚动列表区域 -->
        <div class="flex-1 min-h-0 overflow-y-auto space-y-2 pb-2">
          <SessionCard
            v-for="s in sessions.items"
            :key="s.id"
            :session="s"
            :analyzing="singleAnalyzing === s.id"
            :selectable="batchMode"
            :selected="selectedIds.has(s.id)"
            @analyze="onAnalyze"
            @update:selected="onSelectionChange"
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
