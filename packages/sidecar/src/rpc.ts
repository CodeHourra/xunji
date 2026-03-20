/**
 * JSON-RPC 2.0 协议处理模块。
 *
 * 职责：stdin 读取 → 解析 → 路由到 handler → 返回 stdout
 * 错误码遵循 JSON-RPC 2.0 规范：
 *   -32700 Parse error
 *   -32601 Method not found
 *   -32603 Internal error
 */

import { createInterface } from 'readline'

export interface JsonRpcRequest {
  jsonrpc: '2.0'
  method: string
  params?: Record<string, unknown>
  id: number | string
}

export interface JsonRpcResponse {
  jsonrpc: '2.0'
  result?: unknown
  error?: { code: number; message: string; data?: unknown }
  id: number | string
}

export type Handler = (params: Record<string, unknown>) => Promise<unknown>

function makeResponse(id: number | string, result: unknown): JsonRpcResponse {
  return { jsonrpc: '2.0', result, id }
}

function makeError(id: number | string, code: number, message: string): JsonRpcResponse {
  return { jsonrpc: '2.0', error: { code, message }, id }
}

/**
 * 启动 JSON-RPC 服务器，从 stdin 逐行读取请求，路由到注册的 handler。
 * 日志输出到 stderr（避免污染 stdout 的 JSON-RPC 通道）。
 */
export function startRpcServer(handlers: Record<string, Handler>): void {
  const rl = createInterface({ input: process.stdin })

  rl.on('line', async (line) => {
    const trimmed = line.trim()
    if (!trimmed) return

    const response = await handleRequest(trimmed, handlers)
    process.stdout.write(JSON.stringify(response) + '\n')
  })

  rl.on('close', () => {
    process.exit(0)
  })

  console.error('[sidecar] JSON-RPC server started, waiting for requests on stdin...')
}

async function handleRequest(
  raw: string,
  handlers: Record<string, Handler>,
): Promise<JsonRpcResponse> {
  let req: JsonRpcRequest
  try {
    req = JSON.parse(raw)
  } catch {
    return makeError(0, -32700, 'Parse error')
  }

  const handler = handlers[req.method]
  if (!handler) {
    console.error(`[sidecar] Unknown method: ${req.method}`)
    return makeError(req.id, -32601, `Method not found: ${req.method}`)
  }

  try {
    console.error(`[sidecar] Handling: ${req.method}`)
    const result = await handler(req.params ?? {})
    return makeResponse(req.id, result)
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err)
    console.error(`[sidecar] Error in ${req.method}: ${message}`)
    return makeError(req.id, -32603, message)
  }
}
