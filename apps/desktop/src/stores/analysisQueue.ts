/**
 * 全局分析队列 —— 列表单条、批量、详情页「提炼」共享同一串行队列。
 *
 * ```text
 * enqueue → tasks[] → _processNext（串行 api.distillSession）
 *              ↓
 *       sessions.patchItem（列表状态）
 *              ↓
 *       callbacks（详情页本地状态 / 单条成功后跳转）
 * ```
 */
import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { api } from '../lib/tauri'
import type { DistillSessionResult } from '../types'
import { useSessionsStore } from './sessions'

/** 队列任务状态 */
export type QueueTaskStatus = 'pending' | 'running' | 'done' | 'error'

/** 单条队列任务（model 字段见 types） */
export interface QueueTask {
  sessionId: string
  /** 面板展示的标题（项目名或卡片标题） */
  displayTitle: string
  status: QueueTaskStatus
  /** 成功产出笔记 / 低价值无卡片 */
  outcome?: 'success' | 'low_value'
  /** distill 失败时与 invoke / DB error_message 对齐的可读原因 */
  errorMessage?: string
  /** 当前条已耗时（秒），仅 running 时递增 */
  elapsedSec: number
  callbacks?: {
    onSuccess?: (r: DistillSessionResult) => void
    onLowValue?: (r: DistillSessionResult) => void
    onError?: (msg: string) => void
  }
}

export interface EnqueueOptions {
  onSuccess?: (r: DistillSessionResult) => void
  onLowValue?: (r: DistillSessionResult) => void
  onError?: (msg: string) => void
}

export const useAnalysisQueueStore = defineStore('analysisQueue', () => {
  const tasks = ref<QueueTask[]>([])

  /** 是否正在执行 _processNext 内的一次 distill（防止重入） */
  let processing = false

  let elapsedTimer: ReturnType<typeof setInterval> | null = null

  function clearElapsedTimer() {
    if (elapsedTimer) {
      clearInterval(elapsedTimer)
      elapsedTimer = null
    }
  }

  /** 当前正在跑的任务（至多一条） */
  const currentTask = computed(() => tasks.value.find((t) => t.status === 'running') ?? null)

  const pendingCount = computed(() => tasks.value.filter((t) => t.status === 'pending').length)

  const totalCount = computed(() => tasks.value.length)

  const doneCount = computed(
    () => tasks.value.filter((t) => t.status === 'done' || t.status === 'error').length,
  )

  /** 队列中是否还有任务（含已结束待用户点关闭） */
  const hasAny = computed(() => tasks.value.length > 0)

  /**
   * 是否全部结束（无 pending / running），可显示「关闭」清空面板
   */
  const isIdle = computed(
    () =>
      tasks.value.length > 0 &&
      !tasks.value.some((t) => t.status === 'pending' || t.status === 'running'),
  )

  /** 进度百分比：已完成（含失败/取消） / 总数 */
  const progressPercent = computed(() => {
    if (totalCount.value === 0) return 0
    return Math.round((doneCount.value / totalCount.value) * 100)
  })

  /** 最近完成的最多 3 条（从新到旧） */
  const recentCompleted = computed(() => {
    const out: QueueTask[] = []
    for (let i = tasks.value.length - 1; i >= 0 && out.length < 3; i--) {
      const t = tasks.value[i]
      if (t.status === 'done' || t.status === 'error') out.push(t)
    }
    return out
  })

  /**
   * 入队：若同 session 已在 pending/running 则忽略（防重复点击）
   * @returns 是否实际入队（用于批量统计）
   */
  function enqueue(sessionId: string, displayTitle: string, options: EnqueueOptions = {}): boolean {
    const dup = tasks.value.some(
      (t) =>
        t.sessionId === sessionId && (t.status === 'pending' || t.status === 'running'),
    )
    if (dup) {
      console.warn('[analysisQueue] 会话已在队列中，忽略重复入队:', sessionId)
      return false
    }

    const sessions = useSessionsStore()
    sessions.patchItem(sessionId, { status: 'analyzing' })

    tasks.value.push({
      sessionId,
      displayTitle: displayTitle || sessionId,
      status: 'pending',
      elapsedSec: 0,
      callbacks: { ...options },
    })

    void processNext()
    return true
  }

  /**
   * 停止：未开始的 pending 全部标记为 error，会话状态恢复为 pending
   * 当前 running 的一条会继续跑完（无法中断后端 distill）
   */
  function cancel() {
    const sessions = useSessionsStore()
    for (const t of tasks.value) {
      if (t.status === 'pending') {
        t.status = 'error'
        t.errorMessage = '已取消排队'
        sessions.patchItem(t.sessionId, { status: 'pending' })
        // 让调用方回调（如批量统计）能收到「本条已结束」
        t.callbacks?.onError?.('已取消排队')
      }
    }
  }

  /** 清空已结束任务，收起面板 */
  function clear() {
    tasks.value = tasks.value.filter(
      (t) => t.status !== 'done' && t.status !== 'error',
    )
  }

  function applyResultToSessions(sessionId: string, result: DistillSessionResult) {
    const sessions = useSessionsStore()
    if (result.isLowValue) {
      sessions.patchItem(sessionId, {
        status: 'analyzed',
        value: result.value,
        cardId: null,
        cardTitle: result.cardTitle ?? null,
        cardSummary: result.reason ?? null,
        cardType: result.cardType ?? null,
        cardTags: null,
      })
    } else {
      const card = result.card!
      sessions.patchItem(sessionId, {
        status: 'analyzed',
        value: card.value ?? null,
        cardId: card.id,
        cardTitle: card.title,
        cardSummary: card.summary ?? null,
        cardType: card.type ?? null,
        cardTags: card.tags?.join(',') ?? null,
      })
    }
  }

  async function processNext() {
    // 防止 await 间隙被重复触发导致双开 distill
    if (processing) return
    if (tasks.value.some((t) => t.status === 'running')) return

    const next = tasks.value.find((t) => t.status === 'pending')
    if (!next) return

    processing = true
    next.status = 'running'
    next.elapsedSec = 0

    clearElapsedTimer()
    elapsedTimer = setInterval(() => {
      const running = tasks.value.find((t) => t.sessionId === next.sessionId && t.status === 'running')
      if (running) running.elapsedSec++
    }, 1000)

    try {
      const result = await api.distillSession(next.sessionId)
      clearElapsedTimer()

      if (import.meta.env.DEV) {
        console.info(
          `[distill] traceId=${result.traceId} sessionId=${next.sessionId}（与终端 Rust/sidecar 日志 grep 同一 id）`,
        )
      }

      next.status = 'done'
      if (result.isLowValue) {
        next.outcome = 'low_value'
        applyResultToSessions(next.sessionId, result)
        next.callbacks?.onLowValue?.(result)
      } else {
        next.outcome = 'success'
        applyResultToSessions(next.sessionId, result)
        next.callbacks?.onSuccess?.(result)
      }
    } catch (e) {
      clearElapsedTimer()
      const msg = e instanceof Error ? e.message : String(e)
      next.status = 'error'
      next.errorMessage = msg
      const sessions = useSessionsStore()
      // 与后端 update_session_error 一致，列表 SessionCard 可立即展示原因而无需整页刷新
      sessions.patchItem(next.sessionId, { status: 'error', errorMessage: msg })
      next.callbacks?.onError?.(msg)
    } finally {
      processing = false
      setTimeout(() => {
        void processNext()
      }, 500)
    }
  }

  return {
    tasks,
    currentTask,
    pendingCount,
    totalCount,
    doneCount,
    hasAny,
    isIdle,
    progressPercent,
    recentCompleted,
    enqueue,
    cancel,
    clear,
  }
})
