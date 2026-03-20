export interface NormalizedSession {
  sourceId: string
  sessionId: string
  projectPath?: string
  projectName?: string
  messages: NormalizedMessage[]
  rawPath: string
  createdAt: string
  updatedAt: string
}

export interface NormalizedMessage {
  role: 'user' | 'assistant'
  content: string
  timestamp?: string
  tokensIn: number
  tokensOut: number
}

export interface KnowledgeCard {
  id: string
  sessionId: string
  title: string
  type: CardType
  value: ValueLevel
  summary: string
  note: string
  tags: string[]
  techStack: string[]
  memory?: string
  skill?: string
  sourceName: string
  projectName: string
  promptTokens: number
  completionTokens: number
  costYuan: number
  feedback?: 'positive' | 'negative'
  createdAt: string
  updatedAt: string
}

export type ValueLevel = 'high' | 'medium' | 'low' | 'none'

export type CardType =
  | 'debug'
  | 'architecture'
  | 'performance'
  | 'best-practice'
  | 'concept'
  | 'tool-usage'
  | 'refactor'
  | 'other'

export interface DistillResult {
  title: string
  type: CardType
  value: ValueLevel
  summary: string
  note: string
  tags: string[]
  techStack: string[]
  memory?: string
  skill?: string
}

export interface JudgeValueResult {
  value: ValueLevel
  type: CardType
  reason: string
}

export interface SyncResult {
  found: number
  new: number
  updated: number
  skipped: number
}
