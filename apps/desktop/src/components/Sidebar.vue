<script setup lang="ts">
import { computed, h, onMounted, ref, watch } from 'vue'
import { NTree, NTag, NEmpty, NSpin, NTooltip, NButton, useDialog, useMessage } from 'naive-ui'
import type { TreeOption } from 'naive-ui'
import { useUiStore } from '../stores/ui'
import { useFiltersStore } from '../stores/filters'
import { useSessionsStore } from '../stores/sessions'
import { useSidebarStore } from '../stores/sidebar'
import { getCardTypeLabel } from '@xunji/shared'
import { api } from '../lib/tauri'
import type { SessionFilterPayload } from '../types'

// 品牌图标（官方 logo，通过 Vite asset URL 导入）
import claudeCodeIcon from '../assets/brands/claude-code.png?url'
import cursorIcon from '../assets/brands/cursor.png?url'
import codebuddyIcon from '../assets/brands/codebuddy.svg?url'

const ui = useUiStore()
const filters = useFiltersStore()
const sessions = useSessionsStore()
const sidebar = useSidebarStore()
const dialog = useDialog()
const message = useMessage()

/** 侧栏批量移除会话时的按钮 loading */
const removeGroupBusy = ref(false)

/**
 * 会话整理模式：目录树显示多选框，勾选多个分组后可批量删除（筛选并集去重）。
 * 进入整理时列表重置为全部对话；单击节点筛选列表的逻辑仍保留。
 */
const sessionOrganizeMode = ref(false)
/** 整理模式下 NTree 勾选的分组 key（与 treeData 中 key 一致） */
const checkedKeys = ref<string[]>([])

// ────────────────── 数据源配置 ──────────────────

/**
 * 数据源元信息：每个 source_id 对应其品牌标签和图标
 * imgIcon 优先使用品牌 SVG；fallbackIcon 为 UnoCSS 类名兜底
 */
const SOURCE_META: Record<string, { label: string; imgIcon?: string; fallbackIcon: string }> = {
  'claude-code': { label: 'Claude Code', imgIcon: claudeCodeIcon, fallbackIcon: 'i-lucide-terminal' },
  'cursor': { label: 'Cursor', imgIcon: cursorIcon, fallbackIcon: 'i-lucide-mouse-pointer-click' },
  'codebuddy-cli': { label: 'CodeBuddy', imgIcon: codebuddyIcon, fallbackIcon: 'i-lucide-code' },
  'codebuddy-jetbrains': { label: 'CodeBuddy JB', imgIcon: codebuddyIcon, fallbackIcon: 'i-lucide-braces' },
}

function getSourceMeta(sourceId: string) {
  return SOURCE_META[sourceId] ?? { label: sourceId, fallbackIcon: 'i-lucide-folder' }
}

/** 根据 source meta 渲染品牌图标前缀 */
function renderSourceIcon(sourceId: string) {
  const meta = getSourceMeta(sourceId)
  if (meta.imgIcon) {
    return h('img', { src: meta.imgIcon, class: 'brand-icon', width: 16, height: 16 })
  }
  return h('span', { class: `${meta.fallbackIcon} w-4 h-4 opacity-70` })
}

/** host 显示名映射，"local" → "本地" */
const HOST_LABELS: Record<string, string> = {
  local: '本地',
  localhost: '本地',
}

function hostLabel(host: string) {
  return HOST_LABELS[host] ?? host
}

// ────────────────── NTree 数据构建 ──────────────────

/**
 * 将扁平的 SessionGroupCount[] 组装成 NTree 需要的 TreeOption[]
 *
 * 树结构：数据源 → 主机 → 项目
 * key 编码规则：
 *   "src:<sourceId>" / "host:<sourceId>/<host>" / "proj:<sourceId>/<host>/<project>"
 */
const treeData = computed<TreeOption[]>(() => {
  const groups = sidebar.sessionGroups
  const sourceMap = new Map<string, Map<string, { project: string | null; count: number }[]>>()

  for (const g of groups) {
    if (!sourceMap.has(g.sourceId)) sourceMap.set(g.sourceId, new Map())
    const hostMap = sourceMap.get(g.sourceId)!
    if (!hostMap.has(g.sourceHost)) hostMap.set(g.sourceHost, [])
    hostMap.get(g.sourceHost)!.push({ project: g.projectName, count: g.count })
  }

  const result: TreeOption[] = []
  for (const [sourceId, hostMap] of sourceMap) {
    const hostChildren: TreeOption[] = []
    let sourceTotal = 0

    for (const [host, items] of hostMap) {
      const projChildren: TreeOption[] = []
      let hostTotal = 0
      for (const item of items) {
        hostTotal += item.count
        const projName = item.project ?? '(未关联项目)'
        projChildren.push({
          key: `proj:${sourceId}/${host}/${projName}`,
          label: projName,
          isLeaf: true,
          suffix: () => h('span', { class: 'sidebar-count' }, String(item.count)),
        })
      }
      sourceTotal += hostTotal
      hostChildren.push({
        key: `host:${sourceId}/${host}`,
        label: hostLabel(host),
        children: projChildren,
        prefix: () => h('span', { class: 'i-lucide-server w-3.5 h-3.5 opacity-50' }),
        suffix: () => h('span', { class: 'sidebar-count' }, String(hostTotal)),
      })
    }

    result.push({
      key: `src:${sourceId}`,
      label: getSourceMeta(sourceId).label,
      children: hostChildren,
      prefix: () => renderSourceIcon(sourceId),
      suffix: () => h('span', { class: 'sidebar-count font-medium' }, String(sourceTotal)),
    })
  }
  return result
})

const totalSessions = computed(() =>
  sidebar.sessionGroups.reduce((sum, g) => sum + g.count, 0),
)

/** 自定义树节点 label 渲染：超长截断 + tooltip */
function renderLabel({ option }: { option: TreeOption }) {
  const label = String(option.label ?? '')
  return h(NTooltip, {
    trigger: 'hover',
    delay: 500,
    placement: 'right',
  }, {
    trigger: () => h('span', {
      class: 'sidebar-label',
      style: 'display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;',
    }, label),
    default: () => label,
  })
}

// ────────────────── 树节点选中 ──────────────────

/**
 * 用 ref 维护选中 key（而非 computed），避免 NTree 内部状态与外部冲突。
 * 当 filter 变更时手动同步。
 */
const selectedKeys = ref<string[]>([])

/** 默认展开所有数据源节点 */
const expandedKeys = ref<string[]>([])

watch(treeData, (data) => {
  if (expandedKeys.value.length === 0 && data.length > 0) {
    expandedKeys.value = data.map((n) => n.key as string)
  }
}, { immediate: true })

/** 将树节点 key 转为后端 `SessionFilters`（并合并当前 `statusFilter`） */
function treeKeyToFilterPayload(key: string): SessionFilterPayload {
  const status = filters.statusFilter || null
  if (key.startsWith('src:')) {
    return { source: key.slice(4), host: null, project: null, status }
  }
  if (key.startsWith('host:')) {
    const parts = key.slice(5).split('/')
    return {
      source: parts[0],
      host: parts.slice(1).join('/') || null,
      project: null,
      status,
    }
  }
  if (key.startsWith('proj:')) {
    const rest = key.slice(5)
    const firstSlash = rest.indexOf('/')
    const sourceId = rest.slice(0, firstSlash)
    const afterSource = rest.slice(firstSlash + 1)
    const secondSlash = afterSource.indexOf('/')
    const host = afterSource.slice(0, secondSlash)
    const project = afterSource.slice(secondSlash + 1)
    return { source: sourceId, host, project, status }
  }
  throw new Error(`无效的树节点 key: ${key}`)
}

function exitSessionOrganizeMode() {
  sessionOrganizeMode.value = false
  checkedKeys.value = []
}

/** 进入整理：列表回到全部对话，便于专注勾选分组 */
function enterSessionOrganizeMode() {
  checkedKeys.value = []
  filters.resetSessions()
  selectedKeys.value = []
  sessions.page = 1
  void sessions.loadPage()
  sessionOrganizeMode.value = true
}

function toggleSessionOrganizeMode() {
  if (sessionOrganizeMode.value) {
    exitSessionOrganizeMode()
  } else {
    enterSessionOrganizeMode()
  }
}

/** 解析 key 并设置对应的筛选条件 */
function onTreeSelect(keys: string[]) {
  if (sessionOrganizeMode.value) {
    selectedKeys.value = keys.length ? [keys[0] as string] : []
    return
  }

  const key = keys[0]
  if (!key) return

  if (key.startsWith('src:')) {
    const sourceId = key.slice(4)
    filters.sourceId = sourceId
    filters.sourceHost = ''
    filters.projectQuery = ''
  } else if (key.startsWith('host:')) {
    const parts = key.slice(5).split('/')
    filters.sourceId = parts[0]
    filters.sourceHost = parts.slice(1).join('/')
    filters.projectQuery = ''
  } else if (key.startsWith('proj:')) {
    const rest = key.slice(5)
    const firstSlash = rest.indexOf('/')
    const sourceId = rest.slice(0, firstSlash)
    const afterSource = rest.slice(firstSlash + 1)
    const secondSlash = afterSource.indexOf('/')
    const host = afterSource.slice(0, secondSlash)
    const project = afterSource.slice(secondSlash + 1)
    filters.sourceId = sourceId
    filters.sourceHost = host
    filters.projectQuery = project
  }
  selectedKeys.value = keys
  sessions.page = 1
  void sessions.loadPage()
}

/** "全部对话"按钮：清空筛选 + 清空树选中 + 重新加载 */
function selectAll() {
  exitSessionOrganizeMode()
  filters.resetSessions()
  selectedKeys.value = []
  sessions.page = 1
  void sessions.loadPage()
}

/** 当前是否处于"全部对话"模式 */
const isAllMode = computed(() => selectedKeys.value.length === 0)

/** 仅在「会话整理」且已勾选分组时展示移除按钮 */
const showRemoveSessionsButton = computed(
  () =>
    !sidebar.groupsLoading
    && treeData.value.length > 0
    && sessionOrganizeMode.value
    && checkedKeys.value.length > 0,
)

/**
 * summaryText 用于确认框文案。
 */
async function confirmAndRemoveSessionGroups(
  groups: SessionFilterPayload[],
  summaryText: string,
) {
  if (groups.length === 0 || removeGroupBusy.value)
    return

  removeGroupBusy.value = true
  let n = 0
  try {
    n = await api.countSessionsByFilterGroups(groups)
  }
  catch (e) {
    message.error(e instanceof Error ? e.message : String(e))
    removeGroupBusy.value = false
    return
  }
  removeGroupBusy.value = false

  if (n === 0) {
    message.info('没有可删除的会话')
    return
  }

  dialog.warning({
    title: '移除会话',
    content: `将永久删除 ${summaryText} 所覆盖范围内当前匹配的 ${n} 条会话及其消息、关联笔记。多选时同一会话只计一次。仅删除应用内数据，不会删除本地目录中的原始文件。此操作不可恢复。`,
    positiveText: '移除',
    negativeText: '取消',
    onPositiveClick: async () => {
      removeGroupBusy.value = true
      try {
        const deleted = await api.deleteSessionsByFilterGroups(groups)
        message.success(`已移除 ${deleted} 条会话`)
        exitSessionOrganizeMode()
        selectAll()
        await sidebar.loadSessionGroups()
        await sessions.loadPage()
      }
      catch (e) {
        message.error(e instanceof Error ? e.message : String(e))
        throw e
      }
      finally {
        removeGroupBusy.value = false
      }
    },
  })
}

/** 整理模式：移除勾选的多组并集 */
function removeCheckedSessionGroups() {
  if (!sessionOrganizeMode.value || checkedKeys.value.length === 0)
    return
  const groups = checkedKeys.value.map(k => treeKeyToFilterPayload(k))
  const summary = `已选的 ${groups.length} 个分组`
  void confirmAndRemoveSessionGroups(groups, summary)
}

function onUpdateCheckedKeys(keys: Array<string | number>) {
  checkedKeys.value = keys.map(String)
}

const totalCards = computed(() =>
  sidebar.cardTypes.reduce((sum, t) => sum + t.count, 0),
)

function selectCardType(typeName: string) {
  filters.cardType = filters.cardType === typeName ? '' : typeName
}

function toggleTag(tagName: string) {
  const idx = filters.selectedTags.indexOf(tagName)
  if (idx >= 0) {
    filters.selectedTags.splice(idx, 1)
  } else {
    filters.selectedTags.push(tagName)
  }
}

// ────────────────── 初始化与 Tab 切换时刷新 ──────────────────

onMounted(() => {
  if (ui.activeTab === 'sessions') {
    void sidebar.loadSessionGroups()
  } else {
    void sidebar.loadLibraryMeta()
  }
})

watch(() => ui.activeTab, (tab) => {
  if (tab === 'sessions') {
    void sidebar.loadSessionGroups()
  } else {
    exitSessionOrganizeMode()
    void sidebar.loadLibraryMeta()
  }
})
</script>

<template>
  <aside class="shrink-0 flex flex-col w-56 bg-neutral-50 dark:bg-neutral-950 border-r border-neutral-200 dark:border-neutral-800 select-none">
    <div class="flex-1 overflow-y-auto">

      <!-- ============== 对话记录模式 ============== -->
      <template v-if="ui.activeTab === 'sessions'">
        <!-- 全部对话按钮 -->
        <div
          class="flex items-center gap-2 mx-2 mt-2 mb-1 px-2 h-8 rounded-md transition-colors cursor-pointer"
          :class="isAllMode
            ? 'bg-brand-50 dark:bg-brand-950/30 text-brand-700 dark:text-brand-300'
            : 'text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-900'"
          @click="selectAll"
        >
          <span class="i-lucide-messages-square w-4 h-4 shrink-0" />
          <span class="text-sm font-medium">全部对话</span>
          <span class="ml-auto text-xs tabular-nums opacity-60">{{ totalSessions }}</span>
        </div>

        <div class="mx-3 my-1 border-t border-neutral-200/70 dark:border-neutral-800" />

        <!-- 会话整理：多选分组后批量删除（仅应用内数据） -->
        <div
          v-if="!sidebar.groupsLoading && treeData.length"
          class="px-2 pb-2 space-y-1.5"
        >
          <n-button
            size="small"
            block
            secondary
            :type="sessionOrganizeMode ? 'primary' : 'default'"
            @click="toggleSessionOrganizeMode"
          >
            <template #icon>
              <span class="i-lucide-list-checks w-4 h-4" />
            </template>
            {{ sessionOrganizeMode ? '退出整理' : '会话整理' }}
          </n-button>
          <p
            v-if="sessionOrganizeMode"
            class="text-[11px] text-neutral-500 dark:text-neutral-400 leading-snug px-0.5"
          >
            勾选目录中的多个分组，再点「移除已选分组会话」。与顶部列表状态筛选组合时，每组都会带上当前状态条件。
          </p>
          <p
            v-else-if="isAllMode"
            class="text-[11px] text-neutral-500 dark:text-neutral-400 leading-snug px-0.5"
          >
            点「会话整理」后勾选目录中的分组，即可批量移除（仅删除应用内记录）。
          </p>
          <n-button
            v-if="showRemoveSessionsButton"
            secondary
            type="error"
            size="small"
            block
            :loading="removeGroupBusy"
            :disabled="sessions.loading"
            @click="removeCheckedSessionGroups"
          >
            <template #icon>
              <span class="i-lucide-trash-2 w-4 h-4" />
            </template>
            移除已选分组会话
          </n-button>
        </div>

        <!-- 加载中 -->
        <div v-if="sidebar.groupsLoading" class="flex items-center justify-center py-8">
          <n-spin size="small" />
        </div>

        <!-- NTree 目录树 -->
        <n-tree
          v-else-if="treeData.length"
          :data="treeData"
          :selected-keys="selectedKeys"
          :expanded-keys="expandedKeys"
          :checked-keys="checkedKeys"
          :checkable="sessionOrganizeMode"
          :cascade="true"
          :render-label="renderLabel"
          block-line
          selectable
          expand-on-click
          class="px-1 py-1 sidebar-tree"
          @update:selected-keys="onTreeSelect"
          @update:checked-keys="onUpdateCheckedKeys"
          @update:expanded-keys="(keys: string[]) => expandedKeys = keys"
        />

        <!-- 空状态 -->
        <div v-else class="py-8 px-4">
          <n-empty size="small" description="暂无对话数据，请先同步" />
        </div>
      </template>

      <!-- ============== 知识库模式 ============== -->
      <template v-else>
        <!-- 类型筛选 -->
        <div class="px-3 pt-3.5 pb-2">
          <div class="flex items-center gap-1.5 text-neutral-400 dark:text-neutral-500 mb-2.5">
            <span class="i-lucide-layout-grid w-3.5 h-3.5" />
            <span class="text-[11px] font-semibold uppercase tracking-widest">类型</span>
          </div>
          <div class="flex flex-wrap gap-1.5">
            <n-tag
              :bordered="!!filters.cardType"
              :type="!filters.cardType ? 'primary' : 'default'"
              size="small"
              round
              :class="{ 'cursor-pointer': true }"
              @click="filters.cardType = ''"
            >
              全部 {{ totalCards }}
            </n-tag>
            <n-tag
              v-for="t in sidebar.cardTypes"
              :key="t.name"
              :bordered="filters.cardType !== t.name"
              :type="filters.cardType === t.name ? 'primary' : 'default'"
              size="small"
              round
              class="cursor-pointer"
              @click="selectCardType(t.name)"
            >
              {{ getCardTypeLabel(t.name) }} {{ t.count }}
            </n-tag>
          </div>
        </div>

        <div class="mx-3 my-1.5 border-t border-neutral-200/70 dark:border-neutral-800" />

        <!-- 标签筛选 -->
        <div class="px-3 py-2">
          <div class="flex items-center gap-1.5 text-neutral-400 dark:text-neutral-500 mb-2.5">
            <span class="i-lucide-tags w-3.5 h-3.5" />
            <span class="text-[11px] font-semibold uppercase tracking-widest">标签</span>
          </div>
          <div v-if="sidebar.tagsLoading" class="flex items-center justify-center py-4">
            <n-spin size="small" />
          </div>
          <div
            v-else-if="sidebar.tags.length"
            class="flex flex-wrap gap-1.5 max-h-64 overflow-y-auto pr-0.5"
          >
            <n-tag
              v-for="tag in sidebar.tags"
              :key="tag.name"
              :bordered="!filters.selectedTags.includes(tag.name)"
              :type="filters.selectedTags.includes(tag.name) ? 'primary' : 'default'"
              size="small"
              round
              class="cursor-pointer"
              @click="toggleTag(tag.name)"
            >
              {{ tag.name }}
              <template #avatar>
                <span class="text-[10px] opacity-60 ml-0.5">{{ tag.count }}</span>
              </template>
            </n-tag>
          </div>
          <n-empty v-else size="small" description="暂无标签" class="py-2" />
        </div>

        <div class="mx-3 my-1.5 border-t border-neutral-200/70 dark:border-neutral-800" />

        <!-- 技术栈：由 cards.tech_stack 列聚合；单卡详情仍在笔记标题下 -->
        <div class="px-3 py-2">
          <div class="flex items-center gap-1.5 text-neutral-400 dark:text-neutral-500 mb-2.5">
            <span class="i-lucide-layers w-3.5 h-3.5" />
            <span class="text-[11px] font-semibold uppercase tracking-widest">技术栈</span>
          </div>
          <div v-if="sidebar.tagsLoading" class="flex items-center justify-center py-3">
            <n-spin size="small" />
          </div>
          <div
            v-else-if="sidebar.techStacks.length"
            class="flex flex-wrap gap-1.5 max-h-52 overflow-y-auto pr-0.5"
          >
            <n-tag
              v-for="row in sidebar.techStacks"
              :key="row.name"
              type="info"
              size="small"
              round
              :bordered="false"
            >
              {{ row.name }}
              <template #avatar>
                <span class="text-[10px] opacity-60 ml-0.5">{{ row.count }}</span>
              </template>
            </n-tag>
          </div>
          <p v-else class="text-[11px] text-neutral-400 dark:text-neutral-500 py-2 leading-relaxed">
            暂无。需会话经提炼写入 tech_stack；排查终端
            <code class="text-[10px] opacity-90">normalize: tech_stack</code>
            与
            <code class="text-[10px] opacity-90">创建卡片: … tech_stack列</code>
          </p>
        </div>
      </template>

    </div>
  </aside>
</template>

<style scoped>
.sidebar-tree :deep(.n-tree-node-content__text) {
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  /* 限定文本区域宽度，让截断生效 */
  min-width: 0;
}
.sidebar-tree :deep(.n-tree-node-content__prefix) {
  margin-right: 6px;
  flex-shrink: 0;
}
.sidebar-tree :deep(.n-tree-node-content__suffix) {
  margin-left: auto;
  flex-shrink: 0;
}
.sidebar-tree :deep(.n-tree-node-content) {
  overflow: hidden;
}
.sidebar-count {
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  margin-right: 4px;
  opacity: 0.5;
}
/* 品牌图标：圆角使外观更精致 */
.brand-icon {
  width: 16px;
  height: 16px;
  border-radius: 3px;
  display: inline-block;
  flex-shrink: 0;
}
</style>
