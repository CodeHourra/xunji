import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api } from '../lib/tauri'
import type { SessionSummary } from '../types'
import { useFiltersStore } from './filters'

/**
 * 会话列表分页与加载状态
 */
export const useSessionsStore = defineStore('sessions', () => {
  const items = ref<SessionSummary[]>([])
  const total = ref(0)
  const page = ref(1)
  const pageSize = ref(20)
  const loading = ref(false)
  const syncing = ref(false)
  const error = ref<string | null>(null)

  async function loadPage() {
    loading.value = true
    error.value = null
    const filters = useFiltersStore()
    try {
      const res = await api.listSessions({
        source: filters.sourceId || undefined,
        host: filters.sourceHost || undefined,
        project: filters.projectQuery || undefined,
        page: page.value,
        pageSize: pageSize.value,
      })
      items.value = res.items
      total.value = res.total
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      console.error('[sessions] loadPage', e)
    } finally {
      loading.value = false
    }
  }

  async function syncAll() {
    syncing.value = true
    error.value = null
    try {
      const r = await api.syncAll()
      console.info('[sessions] sync', r)
      await loadPage()
      return r
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      syncing.value = false
    }
  }

  function setPage(p: number) {
    page.value = Math.max(1, p)
    void loadPage()
  }

  function setPageSize(n: number) {
    pageSize.value = Math.min(200, Math.max(1, n))
    page.value = 1
    void loadPage()
  }

  return {
    items,
    total,
    page,
    pageSize,
    loading,
    syncing,
    error,
    loadPage,
    syncAll,
    setPage,
    setPageSize,
  }
})
