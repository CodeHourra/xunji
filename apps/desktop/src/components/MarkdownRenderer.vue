<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { marked } from 'marked'
import hljs from 'highlight.js/lib/core'
import javascript from 'highlight.js/lib/languages/javascript'
import typescript from 'highlight.js/lib/languages/typescript'
import json from 'highlight.js/lib/languages/json'
import bash from 'highlight.js/lib/languages/bash'
import 'highlight.js/styles/github-dark.css'

hljs.registerLanguage('javascript', javascript)
hljs.registerLanguage('typescript', typescript)
hljs.registerLanguage('json', json)
hljs.registerLanguage('bash', bash)

const props = defineProps<{
  source: string
}>()

const el = ref<HTMLElement | null>(null)
const html = ref('')

marked.setOptions({
  gfm: true,
  breaks: true,
})

function highlightBlocks() {
  if (!el.value) {
    return
  }
  el.value.querySelectorAll('pre code').forEach((block) => {
    hljs.highlightElement(block as HTMLElement)
  })
}

watch(
  () => props.source,
  async (s) => {
    html.value = await marked.parse(s || '')
    requestAnimationFrame(highlightBlocks)
  },
  { immediate: true },
)

onMounted(highlightBlocks)
</script>

<template>
  <div
    ref="el"
    class="prose prose-neutral dark:prose-invert max-w-none text-sm
      [&_pre]:rounded-lg [&_pre]:p-3 [&_pre]:overflow-x-auto
      [&_code]:text-[0.9em]"
    v-html="html"
  />
</template>
