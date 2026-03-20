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
  <div ref="el" class="md-body" v-html="html" />
</template>

<style>
/* Markdown 正文排版（不加 scoped，供 v-html 生成的子元素继承）*/
.md-body {
  font-size: 0.875rem;
  line-height: 1.7;
  color: inherit;
  word-break: break-word;
}
.md-body h1, .md-body h2, .md-body h3, .md-body h4 {
  font-weight: 600;
  margin-top: 1.5em;
  margin-bottom: 0.5em;
  line-height: 1.3;
}
.md-body h1 { font-size: 1.5em; }
.md-body h2 { font-size: 1.25em; border-bottom: 1px solid #e5e7eb; padding-bottom: 0.3em; }
.md-body h3 { font-size: 1.1em; }
.md-body p  { margin: 0.75em 0; }
.md-body ul, .md-body ol { padding-left: 1.5em; margin: 0.75em 0; }
.md-body li { margin: 0.25em 0; }
.md-body code:not(pre code) {
  font-size: 0.85em;
  font-family: ui-monospace, 'SFMono-Regular', Menlo, Monaco, Consolas, monospace;
  background: rgba(110, 118, 129, 0.1);
  border-radius: 4px;
  padding: 0.15em 0.4em;
}
.md-body pre {
  border-radius: 8px;
  padding: 1em;
  overflow-x: auto;
  margin: 1em 0;
}
.md-body pre code { background: none; padding: 0; font-size: 0.85em; }
.md-body blockquote {
  border-left: 3px solid #10b981;
  padding-left: 1em;
  margin: 1em 0;
  color: #6b7280;
  font-style: italic;
}
.md-body table {
  width: 100%;
  border-collapse: collapse;
  margin: 1em 0;
  font-size: 0.9em;
}
.md-body th, .md-body td {
  border: 1px solid #e5e7eb;
  padding: 0.5em 0.75em;
  text-align: left;
}
.md-body th { background: #f9fafb; font-weight: 600; }
.md-body a { color: #10b981; text-decoration: underline; }
.dark .md-body code:not(pre code) { background: rgba(255, 255, 255, 0.1); }
.dark .md-body h2 { border-bottom-color: #374151; }
.dark .md-body th { background: #1f2937; }
.dark .md-body th, .dark .md-body td { border-color: #374151; }
.dark .md-body blockquote { color: #9ca3af; }
</style>
