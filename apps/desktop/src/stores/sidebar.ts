import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api } from '../lib/tauri'
import type { SessionGroupCount, TagCount, TypeCount } from '../types'

/**
 * 侧栏数据：会话目录树分组 + 知识库标签/类型统计
 *
 * 数据在切换 Tab 或同步后刷新，避免每次渲染都查询。
 */
export const useSidebarStore = defineStore('sidebar', () => {
  // ── 会话目录树 ──
  const sessionGroups = ref<SessionGroupCount[]>([])
  const groupsLoading = ref(false)

  // ── 知识库标签、技术栈 & 类型 ──
  const tags = ref<TagCount[]>([])
  const techStacks = ref<TagCount[]>([])
  const cardTypes = ref<TypeCount[]>([])
  const tagsLoading = ref(false)

  /** 加载会话分组统计（切换到对话记录 Tab 或同步完成后调用） */
  async function loadSessionGroups() {
    groupsLoading.value = true
    try {
      sessionGroups.value = await api.getSessionGroups()
    } catch (e) {
      console.error('[sidebar] loadSessionGroups', e)
    } finally {
      groupsLoading.value = false
    }
  }

  /** 加载标签和类型统计（切换到知识库 Tab 时调用） */
  async function loadLibraryMeta() {
    tagsLoading.value = true
    try {
      const [t, ts, c] = await Promise.all([
        api.listTags(),
        api.listTechStackCounts(),
        api.listCardTypes(),
      ])
      tags.value = t
      techStacks.value = ts
      cardTypes.value = c
    } catch (e) {
      console.error('[sidebar] loadLibraryMeta', e)
    } finally {
      tagsLoading.value = false
    }
  }

  return {
    sessionGroups,
    groupsLoading,
    tags,
    techStacks,
    cardTypes,
    tagsLoading,
    loadSessionGroups,
    loadLibraryMeta,
  }
})
