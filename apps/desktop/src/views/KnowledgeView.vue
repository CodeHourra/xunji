<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { NSpin, NEmpty } from 'naive-ui'
import { api } from '../lib/tauri'
import type { CardSummary } from '../types'
import { useFiltersStore } from '../stores/filters'
import Pagination from '../components/Pagination.vue'

const router = useRouter()
const filters = useFiltersStore()
const items = ref<CardSummary[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(20)
const loading = ref(false)

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

onMounted(load)

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
</script>

<template>
  <div class="h-full overflow-y-auto p-6 max-w-4xl mx-auto">
    <h1 class="text-lg font-semibold mb-4">
      知识库
    </h1>

    <div v-if="loading" class="flex items-center justify-center py-20">
      <n-spin size="medium" />
    </div>

    <n-empty v-else-if="!items.length" description="暂无知识卡片" class="py-16" />

    <div v-else class="space-y-2">
      <div
        v-for="c in items"
        :key="c.id"
        class="rounded-lg border border-neutral-200 dark:border-neutral-800 p-3 cursor-pointer hover:bg-neutral-50 dark:hover:bg-neutral-800/40 transition-colors"
        @click="open(c)"
      >
        <div class="font-medium">
          {{ c.title }}
        </div>
        <div class="text-xs text-neutral-500 line-clamp-2 mt-1">
          {{ c.summary }}
        </div>
      </div>
      <Pagination
        v-if="items.length"
        :page="page"
        :page-size="pageSize"
        :total="total"
        @update:page="setPage"
        @update:page-size="setPageSize"
      />
    </div>
  </div>
</template>
