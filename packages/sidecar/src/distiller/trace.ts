/**
 * 提炼流水线 traceId：由 Rust 传入，贯穿 judge_value → distill_full → API 日志，便于按一次分析串联 stderr。
 */
import { randomUUID } from 'node:crypto'

/**
 * 从 JSON-RPC params 解析 traceId；无则生成本地兜底 id（兼容旧调用方）。
 */
export function resolveTraceId(params: Record<string, unknown>): string {
  const v = params.traceId
  if (typeof v === 'string' && v.trim()) {
    return v.trim()
  }
  return `local-${randomUUID()}`
}

/** 统一 stderr 前缀，便于 grep traceId= */
export function distillLog(traceId: string, message: string): string {
  return `[distiller][traceId=${traceId}] ${message}`
}
