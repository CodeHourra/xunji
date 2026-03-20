/**
 * Distiller 模块入口 —— 组装前处理器 + Provider（API 或 CLI）+ Prompt 模板，
 * 导出 JSON-RPC handler 供 index.ts 注册使用。
 *
 * Provider 选择策略（由 init 命令的 mode 参数决定）：
 *   mode = "api" → ApiProvider（HTTP OpenAI-compatible）
 *   mode = "cli" → CliProvider（本地 CLI 工具如 claude / gemini）
 */

import { clean, truncate } from './preprocessor'
import { PROMPT_B_LIGHT, PROMPT_B_FULL } from './prompts'
import { ApiProvider, type ApiProviderConfig, type DistillResult } from './api-provider'
import { CliProvider, type CliProviderConfig } from './cli-provider'

/** 统一的 Provider 接口（API 和 CLI 均实现） */
interface Provider {
  distill(systemPrompt: string, content: string): Promise<DistillResult>
}

let provider: Provider | null = null

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
 * 初始化 API Provider（mode = "api"）。
 */
export function initApiProvider(config: ApiProviderConfig): void {
  provider = new ApiProvider(config)
  console.error(`[distiller] API Provider initialized: ${config.provider}/${config.model}`)
}

/**
 * 初始化 CLI Provider（mode = "cli"）。
 */
export function initCliProvider(config: CliProviderConfig): void {
  provider = new CliProvider(config)
  console.error(`[distiller] CLI Provider initialized: command=${config.command}`)
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
 * 以及 LLM 常见的 JSON 格式问题。
 */
function extractJson(text: string): unknown {
  let cleaned = text.trim()

  // 移除可能的 markdown 代码块
  const jsonBlockMatch = cleaned.match(/```(?:json)?\s*\n?([\s\S]*?)\n?\s*```/)
  if (jsonBlockMatch) {
    cleaned = jsonBlockMatch[1].trim()
  }

  // 第一次尝试：直接解析
  try {
    return JSON.parse(cleaned)
  } catch {
    // 继续尝试修复
  }

  // 修复策略 1：提取最外层 { ... }（LLM 可能在 JSON 前后附加了文字）
  const braceMatch = cleaned.match(/\{[\s\S]*\}/)
  if (braceMatch) {
    try {
      return JSON.parse(braceMatch[0])
    } catch {
      // 继续
    }
  }

  // 修复策略 2：处理 note 字段中未转义的换行符和特殊字符
  // LLM 有时在 JSON 字符串值内直接写入换行（非 \n 转义），导致 JSON 不合法
  const fixedNewlines = (braceMatch?.[0] ?? cleaned).replace(
    /("(?:note|summary|reason|value_reason)":\s*")([\s\S]*?)("(?:\s*[,}]))/g,
    (_match, prefix: string, value: string, suffix: string) => {
      const escaped = value
        .replace(/\\/g, '\\\\')   // 反斜杠
        .replace(/\n/g, '\\n')    // 换行
        .replace(/\r/g, '\\r')    // 回车
        .replace(/\t/g, '\\t')    // 制表符
        .replace(/"/g, '\\"')     // 引号（先解决双重转义：还原已转义的）
      // 还原过度转义（如原本 LLM 就写了 \\n，上面会变成 \\\\n）
      const deduped = escaped
        .replace(/\\\\\\\\n/g, '\\\\n')
        .replace(/\\\\\\\\"/g, '\\\\"')
      return `${prefix}${deduped}${suffix}`
    },
  )

  try {
    return JSON.parse(fixedNewlines)
  } catch {
    // 继续
  }

  // 修复策略 3：最后尝试——逐行连接，去除控制字符
  const sanitized = (braceMatch?.[0] ?? cleaned)
    .replace(/[\x00-\x08\x0b\x0c\x0e-\x1f]/g, '') // 去除控制字符（保留 \n \r \t）
  try {
    return JSON.parse(sanitized)
  } catch (e) {
    // 所有策略都失败，抛出包含原始文本前 500 字符的错误
    const preview = cleaned.slice(0, 500)
    throw new Error(
      `JSON 解析失败: ${(e as Error).message}\n响应预览: ${preview}`,
    )
  }
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
 * init handler —— 初始化 Provider 配置。
 * 由 Rust 侧在 sidecar 启动后调用。
 *
 * API 模式输入：{ mode: "api", provider, base_url, api_key, model, timeout_secs? }
 * CLI 模式输入：{ mode: "cli", command, extra_args? }
 * 旧格式兼容：  { api_key, provider, base_url, model }（无 mode 字段时默认 api 模式）
 */
export async function handleInit(
  params: Record<string, unknown>,
): Promise<{ status: string }> {
  const mode = typeof params.mode === 'string' ? params.mode : 'api'

  if (mode === 'cli') {
    // CLI 模式
    if (typeof params.command !== 'string' || !params.command.trim()) {
      throw new Error('参数缺失或类型错误: CLI 模式需要 command 字段（如 "claude"）')
    }
    const cliConfig: CliProviderConfig = {
      command: params.command,
      extraArgs: Array.isArray(params.extra_args)
        ? (params.extra_args as string[])
        : [],
    }
    initCliProvider(cliConfig)
    return { status: 'ok' }
  }

  // API 模式（默认）
  if (typeof params.api_key !== 'string' || !params.api_key.trim()) {
    throw new Error('参数缺失或类型错误: API 模式需要 api_key 字段')
  }

  const apiConfig: ApiProviderConfig = {
    provider: typeof params.provider === 'string' ? params.provider : 'openai-compatible',
    baseUrl: typeof params.base_url === 'string' ? params.base_url : 'https://api.openai.com/v1',
    apiKey: params.api_key,
    model: typeof params.model === 'string' ? params.model : 'gpt-4o-mini',
    timeoutMs: typeof params.timeout_secs === 'number' ? params.timeout_secs * 1000 : undefined,
  }

  initApiProvider(apiConfig)
  return { status: 'ok' }
}
