import { defineStore } from 'pinia'
import { ref } from 'vue'

/**
 * 侧栏筛选条件
 *
 * 对话记录模式：通过目录树选择 source / host / project
 * 知识库模式：通过类型、标签筛选
 */
export const useFiltersStore = defineStore('filters', () => {
  // ── 对话记录筛选 ──
  /** 数据源 ID（如 "claude-code"） */
  const sourceId = ref('')
  /** 来源主机（如 "localhost"） */
  const sourceHost = ref('')
  /** 项目名称 */
  const projectQuery = ref('')

  // ── 知识库筛选 ──
  /** 知识类型（如 "debug"、"architecture"） */
  const cardType = ref('')
  /** 选中的标签列表 */
  const selectedTags = ref<string[]>([])

  /** 重置对话记录筛选（点击"全部对话"时） */
  function resetSessions() {
    sourceId.value = ''
    sourceHost.value = ''
    projectQuery.value = ''
  }

  /** 重置知识库筛选 */
  function resetLibrary() {
    cardType.value = ''
    selectedTags.value = []
  }

  /** 全部重置 */
  function reset() {
    resetSessions()
    resetLibrary()
  }

  return {
    sourceId,
    sourceHost,
    projectQuery,
    cardType,
    selectedTags,
    resetSessions,
    resetLibrary,
    reset,
  }
})
