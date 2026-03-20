import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { useDebounceFn } from '@vueuse/core'
import { api } from '../lib/tauri'
import type { CardSummary } from '../types'

/**
 * 全文搜索状态（工具栏与列表页共享）
 */
export const useSearchStore = defineStore('search', () => {
  const query = ref('')
  const results = ref<CardSummary[]>([])
  const searching = ref(false)
  const searchError = ref<string | null>(null)

  const run = useDebounceFn(async (q: string) => {
    const t = q.trim()
    if (!t) {
      results.value = []
      return
    }
    searching.value = true
    searchError.value = null
    try {
      results.value = await api.searchCards({ query: t })
    } catch (e) {
      searchError.value = e instanceof Error ? e.message : String(e)
      results.value = []
    } finally {
      searching.value = false
    }
  }, 300)

  watch(query, (q) => {
    void run(q)
  })

  function clear() {
    query.value = ''
    results.value = []
    searchError.value = null
  }

  return { query, results, searching, searchError, clear }
})
