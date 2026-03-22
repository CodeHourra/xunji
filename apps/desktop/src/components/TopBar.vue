<script setup lang="ts">
import { ref } from 'vue'
import {
  NTabs,
  NTabPane,
  NTooltip,
  NButton,
  NDivider,
  NDropdown,
  useMessage,
  useDialog,
} from 'naive-ui'
import type { DropdownOption } from 'naive-ui'
import { useRouter } from 'vue-router'
import { useUiStore } from '../stores/ui'
import { useSessionsStore } from '../stores/sessions'
import { useSidebarStore } from '../stores/sidebar'
import { api } from '../lib/tauri'
import { exportAllCardsToDir } from '../lib/cardExport'
import SettingsModal from './SettingsModal.vue'

const ui = useUiStore()
const sessions = useSessionsStore()
const sidebar = useSidebarStore()
const router = useRouter()
const message = useMessage()
const dialog = useDialog()

const showSettings = ref(false)

/** 顶栏「导出」下拉：与知识库页能力对齐 */
const exportDropdownOptions: DropdownOption[] = [
  { key: 'library', label: '前往知识库（多选 / 所选导出）' },
  { type: 'divider' },
  { key: 'all', label: '导出全部笔记…' },
]

function onExportDropdownSelect(key: string | number) {
  if (key === 'library') {
    void router.push({ name: 'library' })
    return
  }
  if (key !== 'all') return
  void (async () => {
    const total = await api.countAllCards()
    if (total === 0) {
      message.info('知识库暂无笔记')
      return
    }
    dialog.warning({
      title: '导出全部笔记',
      content: `将导出库内全部 ${total} 条笔记到所选文件夹，不受当前列表筛选影响。`,
      positiveText: '选择文件夹',
      negativeText: '取消',
      onPositiveClick: async () => {
        try {
          const r = await exportAllCardsToDir()
          if (r.ok && r.count != null) message.success(`已导出 ${r.count} 条笔记`)
        } catch (e) {
          message.error(e instanceof Error ? e.message : String(e))
        }
      },
    })
  })()
}

function onTabChange(tab: string) {
  ui.activeTab = tab as 'sessions' | 'library'
  if (tab === 'sessions') {
    void router.push({ name: 'sessions' })
  } else {
    void router.push({ name: 'library' })
  }
}

async function onSync() {
  try {
    const r = await sessions.syncAll()
    void sidebar.loadSessionGroups()
    message.success(
      `同步完成：发现 ${r.found}，新增 ${r.new}，更新 ${r.updated}，跳过 ${r.skipped}`,
      { duration: 5000 },
    )
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    message.error(msg, { duration: 12000, closable: true })
  }
}
</script>

<template>
  <header class="h-13 flex items-center justify-between px-4 border-b border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-950 shrink-0 z-50">
    <!-- 左侧：Logo + Tab 导航 -->
    <div class="flex items-center gap-4">
      <div class="flex items-center gap-2">
        <div class="w-6 h-6 rounded-md bg-brand-500/15 flex items-center justify-center">
          <span class="i-lucide-compass w-3.5 h-3.5 text-brand-600 dark:text-brand-400" />
        </div>
        <span class="font-semibold text-sm text-neutral-900 dark:text-neutral-100 tracking-tight">寻迹</span>
      </div>

      <n-tabs type="segment" :value="ui.activeTab" size="small" :pane-style="{ display: 'none' }" @update:value="onTabChange">
        <n-tab-pane name="sessions">
          <template #tab>
            <span class="inline-flex items-center gap-1.5">
              <span class="i-lucide-messages-square w-3.5 h-3.5" />
              对话记录
            </span>
          </template>
        </n-tab-pane>
        <n-tab-pane name="library">
          <template #tab>
            <span class="inline-flex items-center gap-1.5">
              <span class="i-lucide-library w-3.5 h-3.5" />
              知识库
            </span>
          </template>
        </n-tab-pane>
      </n-tabs>
    </div>

    <!-- 右侧：同步 + 工具按钮 -->
    <div class="flex items-center gap-1">
      <n-button
        size="small"
        :loading="sessions.syncing"
        :disabled="sessions.syncing"
        @click="onSync"
      >
        <span class="inline-flex items-center gap-1.5">
          <span v-if="!sessions.syncing" class="i-lucide-refresh-cw w-3.5 h-3.5" />
          {{ sessions.syncing ? '同步中…' : '同步' }}
        </span>
      </n-button>

      <n-divider vertical class="!mx-1.5" />

      <n-dropdown
        trigger="click"
        :options="exportDropdownOptions"
        @select="onExportDropdownSelect"
      >
        <n-tooltip trigger="hover" :delay="400">
          <template #trigger>
            <n-button quaternary circle size="small" aria-label="导出">
              <span class="i-lucide-download w-4 h-4" />
            </n-button>
          </template>
          导出
        </n-tooltip>
      </n-dropdown>

      <n-tooltip trigger="hover" :delay="400">
        <template #trigger>
          <n-button quaternary circle size="small" @click="showSettings = true">
            <span class="i-lucide-settings w-4 h-4" />
          </n-button>
        </template>
        设置
      </n-tooltip>

      <n-tooltip trigger="hover" :delay="400">
        <template #trigger>
          <n-button quaternary circle size="small" @click="ui.toggleTheme()">
            <span :class="ui.darkMode ? 'i-lucide-sun' : 'i-lucide-moon'" class="w-4 h-4" />
          </n-button>
        </template>
        {{ ui.darkMode ? '浅色模式' : '深色模式' }}
      </n-tooltip>
    </div>
  </header>

  <settings-modal v-model:show="showSettings" />
</template>
