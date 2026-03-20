/**
 * Distiller 模块入口 —— 组装前处理器 + API Provider + Prompt 模板，
 * 导出 JSON-RPC handler 供 index.ts 注册使用。
 */

import { clean, truncate } from './preprocessor'
import { PROMPT_B_LIGHT, PROMPT_B_FULL } from './prompts'
import { ApiProvider, type ApiProviderConfig } from './api-provider'

let provider: ApiProvider | null = null

/** LLM 提炼结果类型定义（完整版） */
export interface DistillFullResult {
  title: string
  type: string
  value: string
  value_reason?: string
  summary: string
  note: string
  tags: string[]
  tech_stack: string[]
  prompt_tokens: number
  completion_tokens: number
}

/** 价值判断结果类型定义（轻量版） */
export interface JudgeValueResult {
  value: string
  type: string
  reason: string
  prompt_tokens: number
  completion_tokens: number
}

/**
 * 初始化 distiller，配置 API Provider。
 * 由 index.ts 在收到 init 命令或首次调用时执行。
 */
export function initProvider(config: ApiProviderConfig): void {
  provider = new ApiProvider(config)
  console.error(`[distiller] Provider initialized: ${config.provider}/${config.model}`)
}

function getProvider(): ApiProvider {
  if (!provider) {
    throw new Error('Distiller 未初始化，请先调用 init 方法配置 API')
  }
  return provider
}

/**
 * 对对话内容做前处理（清理 + 截断）
 */
function preprocess(content: string): string {
  const cleaned = clean(content)
  return truncate(cleaned)
}

/**
 * 从 LLM 响应文本中提取 JSON。
 * 处理可能的 markdown 代码块包裹（```json ... ```）
 */
function extractJson(text: string): unknown {
  let cleaned = text.trim()

  // 移除可能的 markdown 代码块
  const jsonBlockMatch = cleaned.match(/```(?:json)?\s*\n?([\s\S]*?)\n?\s*```/)
  if (jsonBlockMatch) {
    cleaned = jsonBlockMatch[1].trim()
  }

  return JSON.parse(cleaned)
}

// ── JSON-RPC Handlers ──

/**
 * judge_value handler —— 轻量价值判断。
 * 输入：{ content: string }
 * 输出：{ value, type, reason, prompt_tokens, completion_tokens }
 */
export async function handleJudgeValue(
  params: Record<string, unknown>,
): Promise<JudgeValueResult> {
  if (typeof params.content !== 'string' || !params.content.trim()) {
    throw new Error('参数缺失或类型错误: content 必须为非空字符串')
  }
  const content = params.content

  const p = getProvider()
  const processed = preprocess(content)

  const result = await p.distill(PROMPT_B_LIGHT, processed)
  const parsed = extractJson(result.content) as {
    value: string
    type: string
    reason: string
  }

  return {
    ...parsed,
    prompt_tokens: result.promptTokens,
    completion_tokens: result.completionTokens,
  }
}

/**
 * distill_full handler —— 完整技术笔记提炼。
 * 输入：{ content: string }
 * 输出：{ title, type, value, summary, note, tags, tech_stack, prompt_tokens, completion_tokens }
 */
export async function handleDistillFull(
  params: Record<string, unknown>,
): Promise<DistillFullResult> {
  if (typeof params.content !== 'string' || !params.content.trim()) {
    throw new Error('参数缺失或类型错误: content 必须为非空字符串')
  }
  const content = params.content

  const p = getProvider()
  const processed = preprocess(content)

  const result = await p.distill(PROMPT_B_FULL, processed)
  const parsed = extractJson(result.content) as {
    title: string
    type: string
    value: string
    value_reason?: string
    summary: string
    note: string
    tags: string[]
    tech_stack: string[]
  }

  return {
    ...parsed,
    prompt_tokens: result.promptTokens,
    completion_tokens: result.completionTokens,
  }
}

/**
 * init handler —— 初始化 API Provider 配置。
 * 由 Rust 侧在 sidecar 启动后调用。
 * 输入：{ provider, base_url, api_key, model, timeout_secs? }
 */
export async function handleInit(
  params: Record<string, unknown>,
): Promise<{ status: string }> {
  if (typeof params.api_key !== 'string' || !params.api_key.trim()) {
    throw new Error('参数缺失或类型错误: api_key 必须为非空字符串')
  }

  const config: ApiProviderConfig = {
    provider: typeof params.provider === 'string' ? params.provider : 'openai-compatible',
    baseUrl: typeof params.base_url === 'string' ? params.base_url : 'https://api.openai.com/v1',
    apiKey: params.api_key,
    model: typeof params.model === 'string' ? params.model : 'gpt-4o-mini',
    timeoutMs: typeof params.timeout_secs === 'number' ? params.timeout_secs * 1000 : undefined,
  }

  initProvider(config)
  return { status: 'ok' }
}
