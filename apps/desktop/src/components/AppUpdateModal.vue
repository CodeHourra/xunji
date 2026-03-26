<script setup lang="ts">
/**
 * 独立「软件更新」弹窗：检查更新 + 下载进度条（与设置页解耦，入口在顶栏）。
 */
import { watch } from 'vue'
import {
  NModal,
  NCard,
  NButton,
  NSpace,
  NAlert,
  NProgress,
  NSpin,
  useMessage,
} from 'naive-ui'
import { useAppUpdater } from '../composables/useAppUpdater'

const props = defineProps<{ show: boolean }>()
const emit = defineEmits<{ 'update:show': [v: boolean] }>()

const message = useMessage()
const {
  phase,
  currentVersion,
  busy,
  downloadPercent,
  downloadIndeterminate,
  statusText,
  errorText,
  pendingUpdate,
  reset,
  loadCurrentVersion,
  runCheck,
  runInstall,
} = useAppUpdater()

watch(
  () => props.show,
  async (v) => {
    if (v) {
      reset()
      await loadCurrentVersion()
    }
  },
)

function onClose() {
  emit('update:show', false)
}

async function onCheckClick() {
  await runCheck()
  if (phase.value === 'uptodate') {
    message.success('当前已是最新版本')
  }
}

</script>

<template>
  <n-modal
    :show="props.show"
    :mask-closable="!busy"
    transform-origin="center"
    @update:show="emit('update:show', $event)"
  >
    <n-card
      style="width: 420px; max-width: 92vw"
      title="软件更新"
      :bordered="false"
      size="huge"
      role="dialog"
      aria-modal="true"
    >
      <template #header-extra>
        <n-button quaternary size="small" class="rounded-lg" :disabled="busy" @click="onClose">
          <span class="i-lucide-x w-4 h-4" />
        </n-button>
      </template>

      <div class="space-y-4">
        <div class="text-sm text-neutral-600 dark:text-neutral-400">
          当前版本
          <span class="font-mono text-brand-600 dark:text-brand-400 ml-2">{{ currentVersion }}</span>
        </div>

        <n-alert v-if="errorText" type="error" :bordered="false" class="!text-xs">
          {{ errorText }}
        </n-alert>

        <!-- 发现新版本 -->
        <template v-if="phase === 'available' && pendingUpdate">
          <n-alert type="info" :bordered="false" class="!text-xs">
            <div class="space-y-1">
              <p class="m-0 font-medium">发现新版本：{{ pendingUpdate.version }}</p>
              <div
                v-if="pendingUpdate.body?.trim()"
                class="max-h-32 overflow-y-auto text-neutral-600 dark:text-neutral-400 whitespace-pre-wrap"
              >
                {{ pendingUpdate.body.trim() }}
              </div>
            </div>
          </n-alert>
        </template>

        <!-- 检查中 -->
        <div v-if="phase === 'checking'" class="flex items-center gap-2 py-2">
          <n-spin size="small" />
          <span class="text-sm text-neutral-500">正在检查更新…</span>
        </div>

        <!-- 下载中：有总大小时用百分比进度条；否则仅转圈 + 已接收字节 -->
        <div v-if="phase === 'downloading'" class="space-y-2">
          <n-progress
            v-if="!downloadIndeterminate"
            type="line"
            :percentage="downloadPercent"
            :processing="downloadPercent > 0 && downloadPercent < 100"
            indicator-placement="inside"
            :height="22"
            border-radius="8px"
          />
          <div v-else class="flex items-center gap-2 py-1">
            <n-spin size="small" />
            <span class="text-sm text-neutral-500">{{ statusText }}</span>
          </div>
          <p
            v-if="statusText && !downloadIndeterminate"
            class="text-xs text-neutral-500 dark:text-neutral-400 m-0"
          >
            {{ statusText }}
          </p>
        </div>

        <div v-if="phase === 'installing'" class="flex items-center gap-2 py-1">
          <n-spin size="small" />
          <span class="text-sm text-neutral-500">正在重启应用…</span>
        </div>

        <n-space vertical :size="10">
          <n-button
            v-if="phase === 'idle' || phase === 'uptodate' || phase === 'error'"
            type="primary"
            secondary
            block
            class="rounded-lg"
            :loading="busy"
            @click="onCheckClick"
          >
            <span class="inline-flex items-center justify-center gap-2">
              <span class="i-lucide-refresh-cw w-4 h-4" />
              检查更新
            </span>
          </n-button>

          <n-button
            v-if="phase === 'available'"
            type="primary"
            block
            class="rounded-lg"
            :loading="busy"
            @click="runInstall"
          >
            <span class="inline-flex items-center justify-center gap-2">
              <span class="i-lucide-download-cloud w-4 h-4" />
              下载并安装
            </span>
          </n-button>
        </n-space>
      </div>

      <template #footer>
        <div class="flex justify-end">
          <n-button size="small" class="rounded-lg" :disabled="busy" @click="onClose">关闭</n-button>
        </div>
      </template>
    </n-card>
  </n-modal>
</template>
