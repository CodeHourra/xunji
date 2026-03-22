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
/*
 * Markdown 正文排版（不加 scoped，供 v-html 子节点使用）
 * 目标：层次更清晰，深浅色与品牌色协调，避免「一片灰字」
 */
.md-body {
  font-size: 0.875rem;
  line-height: 1.75;
  color: inherit;
  word-break: break-word;
  letter-spacing: 0.01em;
}
.md-body > :first-child { margin-top: 0; }
.md-body > :last-child { margin-bottom: 0; }

.md-body h1, .md-body h2, .md-body h3, .md-body h4 {
  font-weight: 600;
  margin-top: 1.35em;
  margin-bottom: 0.45em;
  line-height: 1.28;
  letter-spacing: -0.02em;
}
.md-body h1 { font-size: 1.45em; }
.md-body h2 {
  font-size: 1.22em;
  border-bottom: 1px solid #e5e7eb;
  padding-bottom: 0.35em;
}
.md-body h3 { font-size: 1.08em; color: #374151; }
.md-body h4 { font-size: 1em; color: #4b5563; }

.md-body p { margin: 0.7em 0; }
.md-body strong { font-weight: 600; color: #111827; }

.md-body ul, .md-body ol {
  padding-left: 1.35em;
  margin: 0.65em 0;
}
.md-body li { margin: 0.35em 0; padding-left: 0.2em; }
.md-body li::marker { color: #10b981; }
.md-body ol li::marker { color: #6b7280; font-weight: 500; }

.md-body hr {
  border: none;
  height: 1px;
  margin: 1.35em 0;
  background: linear-gradient(90deg, transparent, #d1d5db 12%, #d1d5db 88%, transparent);
}

.md-body code:not(pre code) {
  font-size: 0.84em;
  font-family: ui-monospace, 'SFMono-Regular', Menlo, Monaco, Consolas, monospace;
  background: rgba(16, 185, 129, 0.09);
  border: 1px solid rgba(16, 185, 129, 0.18);
  border-radius: 4px;
  padding: 0.12em 0.4em;
}

/* 代码块容器：浅色底 + 与 highlight.js 深色主题衔接（正文区多为白底卡片） */
.md-body pre {
  border-radius: 10px;
  padding: 1rem 1.05rem;
  overflow-x: auto;
  margin: 1rem 0;
  border: 1px solid #e5e7eb;
  background: #f4f6f8 !important;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.6);
}
.md-body pre code {
  background: none !important;
  padding: 0;
  font-size: 0.82em;
  line-height: 1.55;
}

.md-body blockquote {
  border-left: 4px solid #10b981;
  padding: 0.5em 0 0.5em 1em;
  margin: 1em 0;
  color: #4b5563;
  font-style: italic;
  background: rgba(16, 185, 129, 0.04);
  border-radius: 0 8px 8px 0;
}

.md-body table {
  width: 100%;
  border-collapse: collapse;
  margin: 1em 0;
  font-size: 0.9em;
  border-radius: 8px;
  overflow: hidden;
  border: 1px solid #e5e7eb;
}
.md-body th, .md-body td {
  border: 1px solid #e5e7eb;
  padding: 0.55em 0.8em;
  text-align: left;
}
.md-body tr:nth-child(even) td { background: #f9fafb; }
.md-body th { background: #f3f4f6; font-weight: 600; }

.md-body a {
  color: #059669;
  text-decoration: underline;
  text-underline-offset: 2px;
  transition: color 0.15s ease;
}
.md-body a:hover { color: #047857; }

.md-body img {
  max-width: 100%;
  height: auto;
  border-radius: 8px;
  margin: 0.75em 0;
}

/* ─── 深色模式 ─── */
.dark .md-body h2 { border-bottom-color: #374151; }
.dark .md-body h3 { color: #d1d5db; }
.dark .md-body h4 { color: #9ca3af; }
.dark .md-body strong { color: #f3f4f6; }
.dark .md-body code:not(pre code) {
  background: rgba(16, 185, 129, 0.12);
  border-color: rgba(16, 185, 129, 0.25);
}
.dark .md-body pre {
  border-color: #374151;
  background: #111827 !important;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
}
.dark .md-body hr {
  background: linear-gradient(90deg, transparent, #4b5563 15%, #4b5563 85%, transparent);
}
.dark .md-body blockquote {
  color: #9ca3af;
  background: rgba(16, 185, 129, 0.06);
}
.dark .md-body th, .dark .md-body td { border-color: #374151; }
.dark .md-body th { background: #1f2937; }
.dark .md-body tr:nth-child(even) td { background: rgba(31, 41, 55, 0.5); }
.dark .md-body table { border-color: #374151; }
.dark .md-body a { color: #34d399; }
.dark .md-body a:hover { color: #6ee7b7; }
</style>
