<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { NButton, NSpin, NResult } from 'naive-ui'
import NoteCard from '../components/NoteCard.vue'
import ChatReplay from '../components/ChatReplay.vue'
import { api } from '../lib/tauri'
import type { Card, Message, Session } from '../types'

const props = defineProps<{
  sessionId: string
}>()

const route = useRoute()
const router = useRouter()

const session = ref<Session | null>(null)
const messages = ref<Message[]>([])
const card = ref<Card | null>(null)
const loading = ref(true)
const err = ref<string | null>(null)
const mode = ref<'note' | 'chat'>('note')

async function load() {
  loading.value = true
  err.value = null
  try {
    session.value = await api.getSession(props.sessionId)
    messages.value = await api.getSessionMessages(props.sessionId)
    const cardId = typeof route.query.cardId === 'string' ? route.query.cardId : null
    if (cardId) {
      card.value = await api.getCard(cardId)
      mode.value = 'note'
    } else {
      card.value = null
    }
  } catch (e) {
    err.value = e instanceof Error ? e.message : String(e)
  } finally {
    loading.value = false
  }
}

onMounted(load)
watch(
  () => [props.sessionId, route.query.cardId],
  () => {
    void load()
  },
)

function close() {
  void router.push({ name: 'sessions' })
}

async function reanalyze() {
  try {
    const c = await api.distillSession(props.sessionId)
    card.value = c
    mode.value = 'note'
    void router.replace({
      name: 'session-detail',
      params: { sessionId: props.sessionId },
      query: { cardId: c.id },
    })
  } catch (e) {
    err.value = e instanceof Error ? e.message : String(e)
  }
}
</script>

<template>
  <div class="h-full overflow-y-auto p-6 max-w-4xl mx-auto">
    <n-button text class="mb-4" @click="close">
      <span class="inline-flex items-center gap-1">
        <span class="i-lucide-arrow-left w-4 h-4" />
        返回列表
      </span>
    </n-button>

    <div v-if="loading" class="flex items-center justify-center py-20">
      <n-spin size="medium" />
    </div>

    <n-result v-else-if="err" status="error" :title="err" class="py-12">
      <template #footer>
        <n-button @click="close">返回列表</n-button>
      </template>
    </n-result>

    <template v-else-if="card">
      <NoteCard
        :card="card"
        :mode="mode"
        @update:mode="mode = $event"
        @close="close"
        @reanalyze="reanalyze"
      >
        <template #chat>
          <div class="max-h-[480px] overflow-auto pr-1">
            <ChatReplay :messages="messages" />
          </div>
        </template>
      </NoteCard>
    </template>

    <template v-else>
      <div class="rounded-xl border border-neutral-200 dark:border-neutral-800 p-6 bg-white dark:bg-neutral-900">
        <p class="text-sm text-neutral-600 dark:text-neutral-400 mb-4">
          该会话尚无笔记。可点击下方进行首次分析。
        </p>
        <ChatReplay :messages="messages" />
        <n-button type="primary" class="mt-4" @click="reanalyze">
          <span class="inline-flex items-center gap-1.5">
            <span class="i-lucide-sparkles w-4 h-4" />
            分析并生成笔记
          </span>
        </n-button>
      </div>
    </template>
  </div>
</template>
