<script setup lang="ts">
/**
 * 设置对话框 —— 提炼配置（API / CLI 模式切换）+ 数据源管理
 *
 * 配置项：
 * 1. 提炼引擎
 *    - API 模式：provider / base_url / api_key / model / timeout
 *    - CLI 模式：command（claude / gemini / codex 等）/ extra_args
 * 2. 数据源
 *    - 启用/禁用各数据源
 */
import { ref, computed, watch } from 'vue'
import {
  NModal, NCard, NTabs, NTabPane, NForm, NFormItem,
  NInput, NInputNumber, NSelect, NSwitch, NButton,
  NSpace, NAlert, NDivider, NSpin, NTag,
} from 'naive-ui'
import { api } from '../lib/tauri'
import type { AppConfigDto, CliConfigDto, SourceConfigDto } from '../types'

const props = defineProps<{ show: boolean }>()
const emit = defineEmits<{
  'update:show': [v: boolean]
}>()

// ── 状态 ────────────────────────────────────────────────────────────────────

const loading = ref(false)
const saving = ref(false)
const errorMsg = ref('')
const successMsg = ref('')

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
    if (v) await loadConfig()
  },
  { immediate: true },
)

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
  } catch (e) {
    errorMsg.value = `配置加载失败: ${e}`
  } finally {
    loading.value = false
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

function toggleSource(source: SourceConfigDto) {
  source.enabled = !source.enabled
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
        <n-button quaternary circle size="small" @click="onClose">
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
        <n-button size="small" @click="loadConfig">重新加载</n-button>
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
                  调用本地已安装的 AI 编程 CLI 工具（如 <code>claude</code>、<code>gemini</code>），
                  使用 <code>-p</code> 非交互参数，输出结果从 stdout 读取。
                </n-alert>

                <n-form size="small" label-placement="left" label-width="80" class="mt-3">
                  <n-form-item label="命令名">
                    <n-space vertical :size="4" style="width: 100%">
                      <n-input
                        v-model:value="workingConfig.distiller.cli.command"
                        placeholder="claude"
                      />
                      <div class="flex flex-wrap gap-1.5 mt-1">
                        <n-tag
                          v-for="cmd in ['claude', 'gemini', 'codex']"
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
              <p class="text-xs text-neutral-500 dark:text-neutral-400 mb-3">
                选择启用的 AI 编程工具对话记录来源，同步时只扫描已启用的数据源。
              </p>

              <div
                v-for="source in workingConfig.collector.sources"
                :key="source.id"
                class="flex items-start justify-between p-3 rounded-lg border border-neutral-200 dark:border-neutral-800 hover:bg-neutral-50 dark:hover:bg-neutral-900 transition-colors"
              >
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 mb-1">
                    <span class="text-sm font-medium text-neutral-800 dark:text-neutral-200">{{ source.name }}</span>
                    <n-tag size="tiny" :type="source.enabled ? 'success' : 'default'">
                      {{ source.enabled ? '已启用' : '已禁用' }}
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
                  @update:value="toggleSource(source)"
                />
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
          <n-button size="small" @click="onClose">取消</n-button>
          <n-button
            type="primary"
            size="small"
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
