<script setup lang="ts">
import type { Card } from '../types'
import NoteHeader from './NoteHeader.vue'
import MarkdownRenderer from './MarkdownRenderer.vue'
import SourceInfo from './SourceInfo.vue'
import ActionBar from './ActionBar.vue'

defineProps<{
  card: Card
  mode: 'note' | 'chat'
  /** 重新分析进行中 */
  analyzing?: boolean
}>()

const emit = defineEmits<{
  'update:mode': [m: 'note' | 'chat']
  close: []
  reanalyze: []
}>()
</script>

<template>
  <article class="rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900 p-6 shadow-sm">
    <NoteHeader
      :title="card.title"
      :summary="card.summary"
      :card-type="card.type"
      :tags="card.tags"
      :tech-stack="card.techStack"
    />
    <MarkdownRenderer v-if="mode === 'note'" :source="card.note" />
    <slot v-else name="chat" />
    <SourceInfo
      :source-name="card.sourceName"
      :project-name="card.projectName"
      :created-at="card.createdAt"
      :prompt-tokens="card.promptTokens"
      :completion-tokens="card.completionTokens"
      :cost-yuan="card.costYuan"
    />
    <ActionBar
      :mode="mode"
      :analyzing="analyzing"
      @update:mode="emit('update:mode', $event)"
      @close="emit('close')"
      @reanalyze="emit('reanalyze')"
    />
  </article>
</template>
