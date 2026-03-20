/**
 * 与 Rust Tauri 返回值对应的 TypeScript 类型（字段名使用 camelCase，与 serde 配置一致）
 */

export interface SyncResult {
  found: number
  new: number
  updated: number
  skipped: number
}

export interface PaginatedResult<T> {
  items: T[]
  total: number
  page: number
  pageSize: number
}

/** 列表行：会话摘要 */
export interface SessionSummary {
  id: string
  sourceId: string
  sessionId: string
  sourceHost: string
  projectPath: string | null
  projectName: string | null
  messageCount: number
  status: string
  value?: string | null
  updatedAt: string
  hasUpdates: boolean
  createdAt: string
  cardId?: string | null
  /** 所有消息内容字节总量（SUM(LENGTH(content))），用于展示 xx KB */
  rawSizeBytes: number
}

export interface Session {
  id: string
  sourceId: string
  sessionId: string
  sourceHost: string
  projectPath: string | null
  projectName: string | null
  messageCount: number
  contentHash: string | null
  rawPath: string | null
  createdAt: string
  updatedAt: string
  status: string
  value: string | null
  hasUpdates: boolean
  analyzedAt: string | null
  errorMessage: string | null
}

export interface Message {
  id: string
  sessionId: string
  role: string
  content: string
  timestamp: string | null
  tokensIn: number
  tokensOut: number
  seqOrder: number
}

export interface Card {
  id: string
  sessionId: string
  title: string
  type: string | null
  value: string | null
  summary: string | null
  note: string
  categoryId: string | null
  memory: string | null
  skill: string | null
  sourceName: string | null
  projectName: string | null
  promptTokens: number
  completionTokens: number
  costYuan: number
  feedback: string | null
  createdAt: string
  updatedAt: string
  tags: string[]
}

export interface CardSummary {
  id: string
  sessionId: string
  title: string
  type: string | null
  value: string | null
  summary: string | null
  categoryId: string | null
  sourceName: string | null
  projectName: string | null
  createdAt: string
  updatedAt: string
}

export interface SessionListParams {
  source?: string | null
  host?: string | null
  project?: string | null
  status?: string | null
  page?: number
  pageSize?: number
}

/** 会话按 source → host → project 分组统计 */
export interface SessionGroupCount {
  sourceId: string
  sourceHost: string
  projectName: string | null
  count: number
}

/** 标签及关联卡片数量 */
export interface TagCount {
  name: string
  count: number
}

/** 知识类型及卡片数量 */
export interface TypeCount {
  name: string
  count: number
}

export interface SearchCardsParams {
  query: string
  tags?: string[] | null
  cardType?: string | null
}

export interface ListCardsParams {
  tags?: string[] | null
  cardType?: string | null
  value?: string | null
  page?: number
  pageSize?: number
}

// ─── 配置相关类型（与 Rust AppConfigDto 对应） ───────────────────────────────

export interface AppConfigDto {
  distiller: DistillerConfigDto
  collector: CollectorConfigDto
  sync: SyncConfigDto
}

export interface DistillerConfigDto {
  /** 提炼模式: "api" | "cli" */
  mode: string
  api: ApiConfigDto | null
  cli: CliConfigDto | null
}

export interface ApiConfigDto {
  /** 提供商标识（如 "openai"、"deepseek"、"openai-compatible"） */
  provider: string
  /** API Base URL（可选） */
  baseUrl: string | null
  /** API 密钥 */
  apiKey: string
  /** 模型名称 */
  model: string
  /** 请求超时（秒） */
  timeoutSecs: number
}

export interface CliConfigDto {
  /** CLI 命令名（如 "claude"、"gemini"） */
  command: string
  /** 附加参数 */
  extraArgs: string[]
}

export interface CollectorConfigDto {
  sources: SourceConfigDto[]
}

export interface SourceConfigDto {
  id: string
  name: string
  enabled: boolean
  scanDirs: string[]
}

export interface SyncConfigDto {
  mode: string
  intervalSecs: number
}
