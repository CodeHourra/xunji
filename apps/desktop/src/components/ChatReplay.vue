<script setup lang="ts">
import { computed, ref } from 'vue'
import MarkdownRenderer from './MarkdownRenderer.vue'
import type { Message } from '../types'

const props = defineProps<{
  messages: Message[]
}>()

/**
 * 将单条消息内容分解为：
 *   - thinking  <thinking>...</thinking> 内的思考块（可折叠）
 *   - rest       剩余正文
 */
function parseMessage(m: Message) {
  const raw = m.content || ''
  const thinkingMatch = raw.match(/<thinking>([\s\S]*?)<\/thinking>/i)
  const thinking = thinkingMatch ? thinkingMatch[1].trim() : null
  const rest = thinkingMatch ? raw.replace(thinkingMatch[0], '').trim() : raw
  return { ...m, rest, thinking }
}

const bubbles = computed(() => props.messages.map(parseMessage))

/** 超长消息折叠阈值（字符数） */
const COLLAPSE_THRESHOLD = 800

/** 跟踪每条消息是否展开 */
const expanded = ref<Record<string, boolean>>({})

function isLong(content: string) {
  return content.length > COLLAPSE_THRESHOLD
}

function toggle(id: string) {
  expanded.value[id] = !expanded.value[id]
}

function displayContent(id: string, content: string) {
  if (!isLong(content) || expanded.value[id]) return content
  return content.slice(0, COLLAPSE_THRESHOLD) + '…'
}

/**
 * 判断消息的展示类型。
 *
 * 返回:
 *   'tool-result'  — 工具执行结果（折叠显示）
 *   'tool-use'     — 工具调用摘要（单行标记）
 *   'bubble'       — 正常对话气泡
 */
function messageType(role: string, content: string): 'tool-result' | 'tool-use' | 'bubble' {
  const trimmed = content.trim()

  // ① 采集器标记的 tool 角色
  if (role === 'tool') return 'tool-result'

  // ② 旧数据兼容：role=user 但内容以 [Tool Result: 开头 → 工具返回值
  if (role === 'user' && trimmed.startsWith('[Tool Result:')) return 'tool-result'

  // ③ assistant 消息中仅有 [Tool: xxx] 标记（无实质文字）→ 工具调用摘要
  if (role === 'assistant') {
    // 去掉所有 [Tool: xxx] 后，如果剩余内容为空，则为纯工具调用
    const withoutToolTags = trimmed.replace(/\[Tool:\s*\w+\]/g, '').trim()
    if (trimmed.includes('[Tool:') && withoutToolTags === '') return 'tool-use'
  }

  return 'bubble'
}

/** 从 [Tool: Bash] 格式中提取工具名列表 */
function extractToolNames(content: string): string {
  const matches = content.match(/\[Tool:\s*(\w+)\]/g) || []
  return matches.map(m => m.replace(/\[Tool:\s*|\]/g, '')).join(', ') || '工具'
}

/**
 * 预处理气泡内容，将 [Tool: xxx] 标记转换为友好的内联 Markdown 格式。
 */
function preprocessContent(content: string): string {
  return content
    .replace(/\[Tool:\s*(\w+)\]/g, '`🔧 $1`')
}
</script>

<template>
  <div class="space-y-4 text-sm">
    <div
      v-for="m in bubbles"
      :key="m.id"
    >
      <!-- ─── 工具结果消息：紧凑折叠样式 ─── -->
      <div v-if="messageType(m.role, m.rest) === 'tool-result'" class="flex justify-center">
        <details class="w-full max-w-[85%]">
          <summary
            class="cursor-pointer text-[11px] text-neutral-400 dark:text-neutral-500
                   select-none flex items-center gap-1.5 py-1 px-3
                   hover:text-neutral-600 dark:hover:text-neutral-300 transition-colors"
          >
            <span class="i-lucide-terminal-square w-3 h-3 opacity-60" />
            <span>工具执行结果</span>
            <span class="i-lucide-chevron-right w-3 h-3 opacity-40 transition-transform details-open:rotate-90" />
          </summary>
          <pre
            class="mt-1 text-[11px] whitespace-pre-wrap break-words leading-relaxed
                   bg-neutral-50 dark:bg-neutral-900/50 border border-neutral-200/60 dark:border-neutral-700/50
                   rounded-lg px-3 py-2 text-neutral-500 dark:text-neutral-400 font-mono
                   max-h-48 overflow-y-auto"
          >{{ m.rest }}</pre>
        </details>
      </div>

      <!-- ─── 工具调用摘要：单行紧凑样式 ─── -->
      <div v-else-if="messageType(m.role, m.rest) === 'tool-use'" class="flex justify-center">
        <div class="flex items-center gap-1.5 py-0.5 px-3 text-[11px] text-neutral-400 dark:text-neutral-500">
          <span class="i-lucide-play w-3 h-3 opacity-50" />
          <span>调用 <span class="font-mono font-medium text-neutral-500 dark:text-neutral-400">{{ extractToolNames(m.rest) }}</span></span>
        </div>
      </div>

      <!-- ─── 用户 / AI 消息：对话气泡样式 ─── -->
      <div
        v-else
        class="flex gap-3"
        :class="m.role === 'assistant' ? 'flex-row-reverse' : 'flex-row'"
      >
        <!-- 角色头像 -->
        <div
          class="w-7 h-7 rounded-full flex items-center justify-center shrink-0 mt-0.5 text-[11px] font-bold"
          :class="m.role === 'assistant'
            ? 'bg-brand-100 dark:bg-brand-900/40 text-brand-700 dark:text-brand-400'
            : 'bg-neutral-100 dark:bg-neutral-800 text-neutral-500'"
        >
          <span v-if="m.role === 'assistant'" class="i-lucide-bot w-3.5 h-3.5" />
          <span v-else class="i-lucide-user w-3.5 h-3.5" />
        </div>

        <!-- 气泡 -->
        <div
          class="max-w-[78%] flex flex-col gap-1.5"
          :class="m.role === 'assistant' ? 'items-end' : 'items-start'"
        >
          <!-- Thinking 折叠块 -->
          <details v-if="m.thinking" class="w-full">
            <summary
              class="cursor-pointer text-[11px] text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-300
                     select-none list-none flex items-center gap-1"
              :class="m.role === 'assistant' ? 'justify-end' : ''"
            >
              <span class="i-lucide-brain w-3 h-3" />
              思考过程
              <span class="i-lucide-chevron-down w-3 h-3" />
            </summary>
            <pre
              class="mt-1.5 text-[11px] whitespace-pre-wrap break-words leading-relaxed
                     bg-neutral-50 dark:bg-neutral-900/60 border border-neutral-200 dark:border-neutral-700
                     rounded-lg px-3 py-2 text-neutral-500 dark:text-neutral-400 font-mono"
            >{{ m.thinking }}</pre>
          </details>

          <!-- 正文气泡（Markdown 渲染） -->
          <div
            class="chat-bubble rounded-2xl px-3.5 py-2.5 shadow-sm border leading-relaxed break-words overflow-hidden"
            :class="m.role === 'assistant'
              ? 'bg-brand-50 dark:bg-brand-950/20 border-brand-100 dark:border-brand-900/40 text-neutral-800 dark:text-neutral-200'
              : 'bg-white dark:bg-neutral-800 border-neutral-200 dark:border-neutral-700 text-neutral-800 dark:text-neutral-200'"
          >
            <MarkdownRenderer :source="preprocessContent(displayContent(m.id, m.rest))" />
            <!-- 长文展开/收起按钮 -->
            <button
              v-if="isLong(m.rest)"
              class="mt-1.5 text-[11px] text-brand-500 hover:text-brand-700 dark:text-brand-400 dark:hover:text-brand-300 font-medium cursor-pointer"
              @click="toggle(m.id)"
            >
              {{ expanded[m.id] ? '收起 ↑' : `展开全文（${m.rest.length} 字）↓` }}
            </button>
          </div>

          <!-- 时间戳 -->
          <span
            v-if="m.timestamp"
            class="text-[10px] text-neutral-400 tabular-nums px-1"
          >
            {{ m.timestamp.replace('T', ' ').slice(0, 16) }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<style>
/* 气泡内 Markdown 排版微调：缩小间距适配气泡紧凑空间 */
.chat-bubble .md-body {
  font-size: 0.875rem;
  line-height: 1.65;
}
.chat-bubble .md-body > :first-child { margin-top: 0; }
.chat-bubble .md-body > :last-child  { margin-bottom: 0; }
.chat-bubble .md-body p  { margin: 0.4em 0; }
.chat-bubble .md-body h1,
.chat-bubble .md-body h2,
.chat-bubble .md-body h3,
.chat-bubble .md-body h4 {
  margin-top: 0.8em;
  margin-bottom: 0.3em;
}
.chat-bubble .md-body h1 { font-size: 1.2em; }
.chat-bubble .md-body h2 { font-size: 1.1em; border-bottom: none; padding-bottom: 0; }
.chat-bubble .md-body h3 { font-size: 1em; }
.chat-bubble .md-body ul,
.chat-bubble .md-body ol { margin: 0.4em 0; padding-left: 1.25em; }
.chat-bubble .md-body li { margin: 0.15em 0; }
.chat-bubble .md-body pre {
  border-radius: 6px;
  padding: 0.75em;
  margin: 0.5em 0;
  font-size: 0.8em;
}
.chat-bubble .md-body blockquote {
  margin: 0.5em 0;
  padding-left: 0.75em;
}
.chat-bubble .md-body table { font-size: 0.85em; }
.chat-bubble .md-body img { max-width: 100%; border-radius: 6px; }

/* details open 时旋转箭头 */
details[open] > summary .details-open\:rotate-90 {
  transform: rotate(90deg);
}
</style>
