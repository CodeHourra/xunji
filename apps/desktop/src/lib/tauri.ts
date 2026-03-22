/**
 * Tauri `invoke` 封装 —— 统一错误与类型，避免业务组件直接依赖字符串命令名。
 *
 * Rust 命令使用 `#[tauri::command(rename_all = "camelCase")]`，
 * 所以这里把 params 直接平铺传递即可，无需 `{ args: ... }` 包装。
 */
import { invoke } from '@tauri-apps/api/core'
import type {
  AppConfigDto,
  Card,
  CardSummary,
  DistillSessionResult,
  ListCardsParams,
  Message,
  PaginatedResult,
  SearchCardsParams,
  Session,
  SessionGroupCount,
  SessionListParams,
  SessionSummary,
  SyncResult,
  TagCount,
  TypeCount,
} from '../types'

export const api = {
  /** 全量采集 */
  syncAll: () => invoke<SyncResult>('sync_all'),

  /** 分页查询会话列表 */
  listSessions: (params: SessionListParams) =>
    invoke<PaginatedResult<SessionSummary>>('list_sessions', {
      source: params.source ?? null,
      host: params.host ?? null,
      project: params.project ?? null,
      status: params.status ?? null,
      page: params.page ?? null,
      pageSize: params.pageSize ?? null,
    }),

  /** 获取单个会话 */
  getSession: (id: string) => invoke<Session>('get_session', { id }),

  /** 获取会话所有消息（对话回放用） */
  getSessionMessages: (sessionId: string) =>
    invoke<Message[]>('get_session_messages', { sessionId }),

  /** 提炼会话 → 价值判断 + 可选笔记生成 */
  distillSession: (sessionId: string) =>
    invoke<DistillSessionResult>('distill_session', { sessionId }),

  /** FTS5 全文搜索卡片 */
  searchCards: (params: SearchCardsParams) =>
    invoke<CardSummary[]>('search_cards', {
      query: params.query,
      tags: params.tags ?? null,
      cardType: params.cardType ?? null,
    }),

  /** 分页查询知识卡片列表 */
  listCards: (params: ListCardsParams) =>
    invoke<PaginatedResult<CardSummary>>('list_cards', {
      tags: params.tags ?? null,
      cardType: params.cardType ?? null,
      value: params.value ?? null,
      page: params.page ?? null,
      pageSize: params.pageSize ?? null,
    }),

  /** 获取单张卡片（含 tags） */
  getCard: (id: string) => invoke<Card>('get_card', { id }),

  /** 获取会话分组统计（侧栏目录树） */
  getSessionGroups: () => invoke<SessionGroupCount[]>('get_session_groups'),

  /** 获取所有标签及数量（知识库侧栏） */
  listTags: () => invoke<TagCount[]>('list_tags'),

  /** 获取技术栈聚合及数量（来自 cards.tech_stack 列） */
  listTechStackCounts: () => invoke<TagCount[]>('list_tech_stack_counts'),

  /** 获取知识类型统计（知识库侧栏） */
  listCardTypes: () => invoke<TypeCount[]>('list_card_types'),

  /** 读取当前应用配置 */
  getConfig: () => invoke<AppConfigDto>('get_config'),

  /** 保存应用配置（写磁盘 + 热更新内存） */
  saveConfig: (config: AppConfigDto) => invoke<void>('save_config', { config }),
}
