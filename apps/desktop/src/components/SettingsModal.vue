<script setup lang="ts">
/**
 * 设置对话框 —— 提炼配置（API / CLI 模式切换）+ 数据源管理 + 关于寻迹（版本信息）
 *
 * 配置项：
 * 1. 提炼引擎
 *    - API 模式：provider / base_url / api_key / model / timeout
 *    - CLI 模式：command（短命令名或可执行文件绝对路径）/ extra_args
 * 2. 数据源
 *    - 启用/禁用各数据源
 * 3. 关于寻迹
 *    - 应用名、版本号、Tauri 运行时版本、Bundle Identifier（来自 tauri.conf / 打包元数据）
 *    - 各版本变更说明：`src/data/app-changelog.md`（与根目录 `CHANGELOG.md` 同步，Vite `?raw` 打入包）
 *    - 软件更新见顶栏「检查更新」，本页不再重复入口
 */
import { ref, computed, watch } from 'vue'
import {
  NModal, NCard, NTabs, NTabPane, NForm, NFormItem,
  NInput, NInputNumber, NSelect, NSwitch, NButton,
  NSpace, NAlert, NDivider, NSpin, NTag,
  NDescriptions, NDescriptionsItem,
} from 'naive-ui'
import { getIdentifier, getTauriVersion, getVersion } from '@tauri-apps/api/app'
import MarkdownRenderer from './MarkdownRenderer.vue'
import appChangelogMd from '../data/app-changelog.md?raw'
import { api } from '../lib/tauri'
import type { AppConfigDto, CliConfigDto, SourceConfigDto } from '../types'

const props = defineProps<{ show: boolean }>()
const emit = defineEmits<{
  'update:show': [v: boolean]
}>()

/** 用户可见品牌名（与窗口标题、macOS 显示名一致；GitHub Release 安装包前缀仍为 tauri.conf 的 productName「XunJi」） */
const APP_DISPLAY_NAME = '寻迹'

// ── 状态 ────────────────────────────────────────────────────────────────────

const loading = ref(false)
const saving = ref(false)
const errorMsg = ref('')
const successMsg = ref('')
/** 设置页：CLI「自动检测」进行中 */
const cliProbing = ref(false)
/** 自动检测后的提示（成功 / 未找到 / 失败） */
const cliProbeHint = ref('')

/**
 * 关于页：由 Tauri App API 拉取（与 tauri.conf.json / Cargo 打包版本一致）
 * model 含义：均为应用元数据字符串，供用户排障与对照发行说明
 */
const aboutMeta = ref<{
  /** 应用语义化版本 */
  version: string
  /** Tauri 框架版本 */
  tauriVersion: string
  /** Bundle ID，如 com.xunji.app */
  identifier: string
} | null>(null)
const aboutLoading = ref(false)
const aboutError = ref('')

// 工作副本：从后端加载后复制，保存时提交
const workingConfig = ref<AppConfigDto | null>(null)

// 提炼模式快捷访问
const distillerMode = computed({
  get: () => workingConfig.value?.distiller.mode ?? 'api',
  set: (v) => {
    if (workingConfig.value) workingConfig.value.distiller.mode = v
  },
})

const cliConfig = computed((): CliConfigDto => {
  if (workingConfig.value?.distiller.cli) return workingConfig.value.distiller.cli
  return { command: 'claude', extraArgs: [] }
})

const providerOptions = [
  { label: 'OpenAI', value: 'openai' },
  { label: 'DeepSeek', value: 'deepseek' },
  { label: 'Moonshot (Kimi)', value: 'moonshot' },
  { label: 'Zhipu (GLM)', value: 'zhipu' },
  { label: 'OpenAI-Compatible', value: 'openai-compatible' },
]

const defaultBaseUrls: Record<string, string> = {
  openai: 'https://api.openai.com/v1',
  deepseek: 'https://api.deepseek.com/v1',
  moonshot: 'https://api.moonshot.cn/v1',
  zhipu: 'https://open.bigmodel.cn/api/paas/v4',
  'openai-compatible': '',
}

const defaultModels: Record<string, string> = {
  openai: 'gpt-4o-mini',
  deepseek: 'deepseek-chat',
  moonshot: 'moonshot-v1-8k',
  zhipu: 'glm-4-flash',
  'openai-compatible': '',
}

// ── 生命周期 ─────────────────────────────────────────────────────────────────

watch(
  () => props.show,
  async (v) => {
    if (v) {
      await loadConfig()
      await loadAboutMeta()
    }
  },
  { immediate: true },
)

/** 打开设置时拉取关于信息（每次打开刷新，便于对照升级后的版本号） */
async function loadAboutMeta() {
  aboutLoading.value = true
  aboutError.value = ''
  aboutMeta.value = null
  try {
    const [version, tauriVersion, identifier] = await Promise.all([
      getVersion(),
      getTauriVersion(),
      getIdentifier(),
    ])
    aboutMeta.value = { version, tauriVersion, identifier }
  } catch (e) {
    aboutError.value =
      e instanceof Error
        ? e.message
        : '无法读取应用版本（请在寻迹桌面客户端内打开设置；纯浏览器预览不支持 Tauri App API）'
  } finally {
    aboutLoading.value = false
  }
}

async function loadConfig() {
  loading.value = true
  errorMsg.value = ''
  try {
    const cfg = await api.getConfig()
    // 确保子对象不为 null，避免模板直接写 null 导致问题
    if (!cfg.distiller.api) {
      cfg.distiller.api = { provider: 'openai-compatible', baseUrl: null, apiKey: '', model: '', timeoutSecs: 120 }
    }
    if (!cfg.distiller.cli) {
      cfg.distiller.cli = { command: 'claude', extraArgs: [] }
    }
    workingConfig.value = cfg
    cliProbeHint.value = ''
  } catch (e) {
    errorMsg.value = `配置加载失败: ${e}`
  } finally {
    loading.value = false
  }
}

/**
 * 调用 Rust：在登录 shell 下执行与终端一致的 `command -v`，按优先级填入第一个解析到的绝对路径。
 * 若均失败，提示用户用手动路径（与 PATH 受限的桌面环境有关）。
 */
async function onProbeCli() {
  if (!workingConfig.value?.distiller.cli) return
  cliProbeHint.value = ''
  cliProbing.value = true
  try {
    const rows = await api.probeCliTools()
    const hit = rows.find((r) => r.resolvedPath)
    if (hit?.resolvedPath) {
      workingConfig.value.distiller.cli.command = hit.resolvedPath
      cliProbeHint.value = `已填入 ${hit.name}：${hit.resolvedPath}`
    } else {
      cliProbeHint.value =
        '未在登录 shell 的 PATH 中找到常见 CLI。若终端可用，请手动填写「command -v 命令名」输出的绝对路径并保存。'
    }
  } catch (e) {
    cliProbeHint.value = `检测失败：${e}`
  } finally {
    cliProbing.value = false
  }
}

// ── 操作 ────────────────────────────────────────────────────────────────────

function onProviderChange(val: string) {
  if (!workingConfig.value?.distiller.api) return
  workingConfig.value.distiller.api.provider = val
  // 自动填充默认 base_url 和 model（仅在为空时）
  if (!workingConfig.value.distiller.api.baseUrl) {
    workingConfig.value.distiller.api.baseUrl = defaultBaseUrls[val] || null
  }
  if (!workingConfig.value.distiller.api.model) {
    workingConfig.value.distiller.api.model = defaultModels[val] || ''
  }
}

async function onSave() {
  if (!workingConfig.value) return
  saving.value = true
  errorMsg.value = ''
  successMsg.value = ''

  // 根据模式清理无效配置
  const config = { ...workingConfig.value }
  if (config.distiller.mode === 'api') {
    config.distiller.cli = null
  } else {
    config.distiller.api = null
  }

  try {
    await api.saveConfig(config)
    successMsg.value = '配置已保存'
    setTimeout(() => { successMsg.value = '' }, 3000)
  } catch (e) {
    errorMsg.value = `保存失败: ${e}`
  } finally {
    saving.value = false
  }
}

function onClose() {
  emit('update:show', false)
}

/** 与 n-switch 的受控值一致，避免仅「翻转」与 Naive 传入的新值不同步 */
function setSourceEnabled(source: SourceConfigDto, enabled: boolean) {
  source.enabled = enabled
}

/** 与 `collector/scheduler.rs` 中 `match source.id` 已实现的采集器一致 */
const IMPLEMENTED_SOURCE_IDS = new Set(['claude-code', 'cursor', 'codebuddy-cli'])

function isSourceCollectorImplemented(id: string): boolean {
  return IMPLEMENTED_SOURCE_IDS.has(id)
}

// extra_args 字符串 ↔ 数组转换辅助
const extraArgsStr = computed({
  get: () => cliConfig.value.extraArgs.join(' '),
  set: (v: string) => {
    if (workingConfig.value?.distiller.cli) {
      workingConfig.value.distiller.cli.extraArgs = v.trim() ? v.trim().split(/\s+/) : []
    }
  },
})
</script>

<template>
  <n-modal
    :show="props.show"
    :mask-closable="false"
    transform-origin="center"
    @update:show="emit('update:show', $event)"
  >
    <!--
      布局：NCard 固定宽度 + max-height 限高；
      tab pane 内层 div 各自负责 overflow-y: auto 滚动，
      避免依赖多层 flex 链导致内容区塌缩。
    -->
    <n-card
      style="width: 580px; max-height: 80vh; overflow: hidden;"
      :content-style="{ padding: 0, minHeight: '240px', overflow: 'hidden' }"
      title="设置"
      :bordered="false"
      size="huge"
      role="dialog"
      aria-modal="true"
    >
      <template #header-extra>
        <!-- 圆角矩形，与页内其它操作按钮一致 -->
        <n-button quaternary size="small" class="settings-btn-rounded" @click="onClose">
          <span class="i-lucide-x w-4 h-4" />
        </n-button>
      </template>

      <!-- 加载中 -->
      <div v-if="loading" class="flex items-center justify-center py-16">
        <n-spin size="large" />
      </div>

      <!-- 加载失败 -->
      <div v-else-if="errorMsg && !workingConfig" class="flex flex-col items-center gap-3 py-10 px-6">
        <span class="i-lucide-alert-circle w-8 h-8 text-red-400" />
        <p class="text-sm text-red-500 dark:text-red-400 text-center">{{ errorMsg }}</p>
        <n-button size="small" class="settings-btn-rounded" @click="loadConfig">重新加载</n-button>
      </div>

      <!--
        workingConfig 就绪后渲染 tab 内容。
        每个 tab pane 内部的 div 独立负责高度限制和滚动。
      -->
      <div v-else-if="workingConfig">
        <n-tabs type="line" animated style="padding: 0 24px">
          <!-- ── 提炼配置 Tab ── -->
          <n-tab-pane name="distiller" tab="提炼引擎">
            <div style="max-height: calc(70vh - 220px); overflow-y: auto; padding: 12px 0 16px;" class="space-y-4">

              <!-- 模式选择 -->
              <div class="flex items-center gap-3">
                <span class="text-sm text-neutral-600 dark:text-neutral-400 shrink-0">提炼方式</span>
                <div class="flex gap-2">
                  <n-button
                    size="small"
                    class="settings-btn-rounded"
                    :type="distillerMode === 'api' ? 'primary' : 'default'"
                    @click="distillerMode = 'api'"
                  >
                    <span class="inline-flex items-center gap-1.5">
                      <span class="i-lucide-cloud w-3.5 h-3.5" />
                      API 模式
                    </span>
                  </n-button>
                  <n-button
                    size="small"
                    class="settings-btn-rounded"
                    :type="distillerMode === 'cli' ? 'primary' : 'default'"
                    @click="distillerMode = 'cli'"
                  >
                    <span class="inline-flex items-center gap-1.5">
                      <span class="i-lucide-terminal w-3.5 h-3.5" />
                      CLI 模式
                    </span>
                  </n-button>
                </div>
              </div>

              <n-divider class="!my-3" />

              <!-- API 模式配置 -->
              <template v-if="distillerMode === 'api' && workingConfig.distiller.api">
                <n-alert type="info" :bordered="false" class="!text-xs">
                  通过 OpenAI-compatible HTTP API 调用大模型，支持 OpenAI、DeepSeek、Moonshot 等。
                </n-alert>

                <n-form size="small" label-placement="left" label-width="80" class="mt-3">
                  <n-form-item label="服务商">
                    <n-select
                      :value="workingConfig.distiller.api.provider"
                      :options="providerOptions"
                      @update:value="onProviderChange"
                    />
                  </n-form-item>

                  <n-form-item label="Base URL">
                    <n-input
                      v-model:value="workingConfig.distiller.api.baseUrl"
                      placeholder="https://api.openai.com/v1"
                      clearable
                    />
                  </n-form-item>

                  <n-form-item label="API Key">
                    <n-input
                      v-model:value="workingConfig.distiller.api.apiKey"
                      type="password"
                      placeholder="sk-..."
                      show-password-on="click"
                    />
                  </n-form-item>

                  <n-form-item label="模型">
                    <n-input
                      v-model:value="workingConfig.distiller.api.model"
                      placeholder="gpt-4o-mini"
                    />
                  </n-form-item>

                  <n-form-item label="超时（秒）">
                    <n-input-number
                      v-model:value="workingConfig.distiller.api.timeoutSecs"
                      :min="10"
                      :max="600"
                      style="width: 120px;"
                    />
                  </n-form-item>
                </n-form>
              </template>

              <!-- CLI 模式配置 -->
              <template v-else-if="distillerMode === 'cli' && workingConfig.distiller.cli">
                <n-alert type="info" :bordered="false" class="!text-xs">
                  <div class="space-y-1.5">
                    <p>
                      调用本机已安装的 AI 编程 CLI（如 <code>claude</code>、<code>gemini</code>），使用
                      <code>-p</code> 非交互参数，从 stdout 读取输出。
                    </p>
                    <p class="text-neutral-500 dark:text-neutral-400">
                      从桌面图标启动时，应用继承的 <strong>PATH 往往比终端少</strong>（例如未加载 nvm）。
                      请先点「自动检测」；若仍失败，在终端执行
                      <code class="px-0.5">command -v &lt;命令名&gt;</code>，
                      将打印出的<strong>绝对路径</strong>粘贴到下方，并务必点击「保存」。
                    </p>
                  </div>
                </n-alert>

                <n-form size="small" label-placement="left" label-width="80" class="mt-3">
                  <n-form-item label="命令或路径">
                    <n-space vertical :size="4" style="width: 100%">
                      <n-space :size="8" align="center" style="width: 100%; flex-wrap: wrap">
                        <n-input
                          v-model:value="workingConfig.distiller.cli.command"
                          placeholder="claude 或 /path/to/cli"
                          style="min-width: 200px; flex: 1"
                        />
                        <n-button
                          size="small"
                          secondary
                          class="settings-btn-rounded"
                          :loading="cliProbing"
                          @click="onProbeCli"
                        >
                          自动检测
                        </n-button>
                      </n-space>
                      <p v-if="cliProbeHint" class="text-xs text-neutral-500 dark:text-neutral-400 m-0">
                        {{ cliProbeHint }}
                      </p>
                      <div class="flex flex-wrap gap-1.5 mt-1">
                        <n-tag
                          v-for="cmd in [
                            'claude-internal',
                            'gemini-internal',
                            'codex-internal',
                            'claude',
                            'gemini',
                            'codex',
                          ]"
                          :key="cmd"
                          size="small"
                          :type="workingConfig.distiller.cli.command === cmd ? 'primary' : 'default'"
                          style="cursor: pointer;"
                          @click="workingConfig!.distiller.cli!.command = cmd"
                        >{{ cmd }}</n-tag>
                      </div>
                    </n-space>
                  </n-form-item>

                  <n-form-item label="额外参数">
                    <n-space vertical :size="4" style="width: 100%">
                      <n-input
                        :value="extraArgsStr"
                        placeholder="如：--model claude-opus-4-5（空格分隔）"
                        @update:value="extraArgsStr = $event"
                      />
                      <span class="text-xs text-neutral-400">
                        这些参数会追加在 <code>-p prompt</code> 之前
                      </span>
                    </n-space>
                  </n-form-item>
                </n-form>

                <n-alert type="warning" :bordered="false" class="!text-xs mt-2">
                  CLI 模式下 token 统计将显示为 0，费用记录不可用。
                </n-alert>
              </template>
            </div>
          </n-tab-pane>

          <!-- ── 数据源 Tab ── -->
          <n-tab-pane name="sources" tab="数据源">
            <div style="max-height: calc(70vh - 220px); overflow-y: auto; padding: 12px 0 16px;" class="space-y-2">
              <p class="text-xs text-neutral-500 dark:text-neutral-400 mb-2">
                选择启用的 AI 编程工具对话记录来源，同步时只扫描已启用的数据源。
              </p>
              <n-alert type="info" :bordered="false" class="!text-xs mb-3">
                <span class="leading-relaxed">
                  当前版本已接入采集的数据源：<strong>Claude Code</strong>（<code class="text-[11px]">claude-code</code>，JSONL）、
                  <strong>Cursor</strong>（<code class="text-[11px]">cursor</code>，本地库）、
                  <strong>CodeBuddy</strong>（<code class="text-[11px]">codebuddy-cli</code>，CodeBuddyExtension 下会话目录，与问渠路径一致；默认关闭，需在设置中启用）。
                  启用后请先点击<strong>保存</strong>，再点击顶部<strong>同步</strong>；侧栏若选了某一数据源筛选，请点「全部对话」或对应 CodeBuddy 节点以免被过滤。
                  若配置中存在其他 id，侧栏仍可显示品牌名，但<strong>同步时会跳过</strong>，直至后续版本接入采集器。
                </span>
              </n-alert>

              <div
                v-for="source in workingConfig.collector.sources"
                :key="source.id"
                class="flex items-start justify-between p-3 rounded-lg border border-neutral-200 dark:border-neutral-800 hover:bg-neutral-50 dark:hover:bg-neutral-900 transition-colors"
              >
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 mb-1 flex-wrap">
                    <span class="text-sm font-medium text-neutral-800 dark:text-neutral-200">{{ source.name }}</span>
                    <n-tag size="tiny" :type="source.enabled ? 'success' : 'default'">
                      {{ source.enabled ? '已启用' : '已禁用' }}
                    </n-tag>
                    <n-tag
                      v-if="!isSourceCollectorImplemented(source.id)"
                      size="tiny"
                      type="warning"
                    >
                      采集未接入
                    </n-tag>
                  </div>
                  <div class="space-y-0.5">
                    <p
                      v-for="dir in source.scanDirs"
                      :key="dir"
                      class="text-xs text-neutral-400 dark:text-neutral-500 font-mono truncate"
                    >{{ dir }}</p>
                  </div>
                </div>
                <n-switch
                  :value="source.enabled"
                  size="small"
                  class="ml-3 mt-0.5"
                  @update:value="(v: boolean) => setSourceEnabled(source, v)"
                />
              </div>
            </div>
          </n-tab-pane>

          <!-- ── 关于寻迹 Tab（版本与标识，无表单保存） ── -->
          <n-tab-pane name="about" tab="关于寻迹">
            <div style="max-height: calc(70vh - 220px); overflow-y: auto; padding: 12px 0 24px;" class="space-y-4">
              <div class="flex flex-col items-center gap-2 text-center pb-2">
                <div class="w-12 h-12 rounded-xl bg-brand-500/15 flex items-center justify-center">
                  <span class="i-lucide-compass w-7 h-7 text-brand-600 dark:text-brand-400" />
                </div>
                <p class="text-base font-semibold text-neutral-800 dark:text-neutral-100">寻迹 XunJi</p>
                <p class="text-xs text-neutral-500 dark:text-neutral-400 max-w-sm leading-relaxed">
                  AI 编程知识管理平台 —— 从对话中提炼可复用的技术笔记。
                </p>
              </div>

              <div v-if="aboutLoading" class="flex justify-center py-8">
                <n-spin size="medium" />
              </div>
              <n-alert v-else-if="aboutError" type="warning" :bordered="false" class="!text-xs">
                {{ aboutError }}
              </n-alert>
              <n-descriptions
                v-else-if="aboutMeta"
                label-placement="left"
                :column="1"
                size="small"
                bordered
                class="rounded-lg overflow-hidden"
              >
                <n-descriptions-item label="应用名称">
                  <span class="text-xs">{{ APP_DISPLAY_NAME }}</span>
                </n-descriptions-item>
                <n-descriptions-item label="应用版本">
                  <span class="font-mono text-xs text-brand-600 dark:text-brand-400">{{ aboutMeta.version }}</span>
                </n-descriptions-item>
                <n-descriptions-item label="Tauri 版本">
                  <span class="font-mono text-xs">{{ aboutMeta.tauriVersion }}</span>
                </n-descriptions-item>
                <n-descriptions-item label="应用标识">
                  <span class="font-mono text-xs break-all">{{ aboutMeta.identifier }}</span>
                </n-descriptions-item>
              </n-descriptions>

              <p class="text-xs text-neutral-500 dark:text-neutral-400 m-0">
                软件更新请使用顶栏 <span class="font-medium text-neutral-600 dark:text-neutral-300">检查更新</span>。
              </p>

              <!-- 更新日志：Markdown 静态内容 -->
              <n-divider class="!my-3" />
              <p class="text-xs font-medium text-neutral-600 dark:text-neutral-300">更新日志</p>
              <div class="changelog-panel rounded-lg border border-neutral-200 dark:border-neutral-800 bg-neutral-50/80 dark:bg-neutral-900/40 px-3 py-2">
                <MarkdownRenderer :source="appChangelogMd" />
              </div>
            </div>
          </n-tab-pane>
        </n-tabs>
      </div>

      <!-- 底部操作 区域不参与滚动，始终固定在底部 -->
      <template #footer>
        <!-- 保存结果提示（仅在 workingConfig 已加载后才有意义） -->
        <n-alert v-if="errorMsg && workingConfig" type="error" class="mb-3 !text-xs" :bordered="false">
          {{ errorMsg }}
        </n-alert>
        <n-alert v-if="successMsg" type="success" class="mb-3 !text-xs" :bordered="false">
          {{ successMsg }}
        </n-alert>
        <div class="flex justify-end gap-2">
          <n-button size="small" class="settings-btn-rounded" @click="onClose">取消</n-button>
          <n-button
            type="primary"
            size="small"
            class="settings-btn-rounded"
            :loading="saving"
            :disabled="!workingConfig || saving"
            @click="onSave"
          >
            保存配置
          </n-button>
        </div>
      </template>
    </n-card>
  </n-modal>
</template>

<style scoped>
/* Naive 默认按钮偏 pill；设置页统一为明显圆角矩形（覆盖组件内部圆角变量） */
.settings-btn-rounded :deep(.n-button__border),
.settings-btn-rounded :deep(.n-button__state-border) {
  border-radius: 10px;
}
.settings-btn-rounded {
  border-radius: 10px;
}

/* 关于页内嵌 Markdown：压缩标题层级，避免与上方元信息抢视觉 */
.changelog-panel :deep(.md-body) {
  font-size: 0.8125rem;
}
.changelog-panel :deep(.md-body h2) {
  font-size: 1.05rem;
  margin-top: 1rem;
  border-bottom: 1px solid rgba(229, 231, 235, 0.9);
  padding-bottom: 0.25rem;
}
.dark .changelog-panel :deep(.md-body h2) {
  border-bottom-color: rgba(55, 65, 81, 0.9);
}
.changelog-panel :deep(.md-body h3) {
  font-size: 0.95rem;
  margin-top: 0.75rem;
}
.changelog-panel :deep(.md-body blockquote) {
  margin: 0.5rem 0;
  font-size: 0.75rem;
}
</style>
