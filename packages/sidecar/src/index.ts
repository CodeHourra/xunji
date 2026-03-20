import { createInterface } from 'readline'

interface JsonRpcRequest {
  jsonrpc: '2.0'
  method: string
  params?: Record<string, unknown>
  id: number | string
}

interface JsonRpcResponse {
  jsonrpc: '2.0'
  result?: unknown
  error?: { code: number; message: string; data?: unknown }
  id: number | string
}

type Handler = (params: Record<string, unknown>) => Promise<unknown>

const handlers: Record<string, Handler> = {
  ping: async () => ({ status: 'ok' }),
}

function makeResponse(id: number | string, result: unknown): JsonRpcResponse {
  return { jsonrpc: '2.0', result, id }
}

function makeError(id: number | string, code: number, message: string): JsonRpcResponse {
  return { jsonrpc: '2.0', error: { code, message }, id }
}

async function handleRequest(raw: string): Promise<JsonRpcResponse> {
  let req: JsonRpcRequest
  try {
    req = JSON.parse(raw)
  } catch {
    return makeError(0, -32700, 'Parse error')
  }

  const handler = handlers[req.method]
  if (!handler) {
    return makeError(req.id, -32601, `Method not found: ${req.method}`)
  }

  try {
    const result = await handler(req.params ?? {})
    return makeResponse(req.id, result)
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err)
    return makeError(req.id, -32603, message)
  }
}

const rl = createInterface({ input: process.stdin })

rl.on('line', async (line) => {
  const trimmed = line.trim()
  if (!trimmed) return

  const response = await handleRequest(trimmed)
  process.stdout.write(JSON.stringify(response) + '\n')
})

rl.on('close', () => {
  process.exit(0)
})
