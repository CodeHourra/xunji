/**
 * CLI Provider —— 通过本地 AI CLI 工具（claude、gemini 等）做知识提炼。
 *
 * 工作原理：
 * 1. 将 system_prompt + 对话内容合并为一条消息
 * 2. 通过 -p 参数（非交互模式）传入 CLI
 * 3. 从 stdout 读取 LLM 响应文本
 *
 * 支持的 CLI 工具（官方非交互模式参数均为 -p）：
 * - claude (Anthropic)：claude -p "prompt"
 * - gemini (Google)：gemini -p "prompt"
 * - codex (OpenAI)：codex -p "prompt"（如名称有改动可自定义 command）
 *
 * 注意：CLI 模式不支持 token 统计（返回 0），也不支持超时精确控制。
 */

import { spawn } from 'child_process'
import { tmpdir } from 'os'
import type { DistillResult } from './api-provider'
import { CONTENT_HINT_CLI } from './prompts'

export interface CliProviderConfig {
  /** CLI 命令名称（如 "claude"、"gemini"、"codex"，可自定义） */
  command: string
  /** 附加参数，追加在 -p prompt 之前（如 ["--model", "claude-opus-4-5"]） */
  extraArgs?: string[]
}

export class CliProvider {
  private readonly command: string
  private readonly extraArgs: string[]

  constructor(config: CliProviderConfig) {
    this.command = config.command
    this.extraArgs = config.extraArgs ?? []
  }

  /**
   * 调用 CLI 工具执行提炼。
   * @param systemPrompt - 系统提示词（Prompt 模板）
   * @param content - 经过前处理的对话内容
   */
  async distill(systemPrompt: string, content: string): Promise<DistillResult> {
    // CLI 不支持多角色消息，将 system prompt + 尾部提示 + 对话内容拼为一条。
    // 使用醒目的分隔线，与 CONTENT_HINT_CLI 中的描述一致。
    const fullPrompt = `${systemPrompt}${CONTENT_HINT_CLI}\n\n════════════════════════════════════════\n\n${content}`

    console.error(
      `[cli-provider] Calling ${this.command} -p, content length: ${fullPrompt.length}`,
    )

    const responseText = await this.runCli(fullPrompt)

    console.error(`[cli-provider] Response: ${responseText.length} chars`)

    return {
      content: responseText,
      // CLI 模式无法获取 token 用量
      promptTokens: 0,
      completionTokens: 0,
    }
  }

  /**
   * 执行 CLI 命令，通过 -p 参数传入提示词，返回 stdout 文本。
   *
   * 使用 -p 非交互模式：claude -p "..." / gemini -p "..."
   * 若提示词超过 shell 参数长度限制（通常 256KB），自动降级为 stdin 管道模式。
   */
  private runCli(prompt: string): Promise<string> {
    return new Promise((resolve, reject) => {
      const MAX_ARG_SIZE = 200_000 // 200KB 安全阈值，避免超出 ARG_MAX

      let args: string[]
      let useStdin = false

      if (prompt.length > MAX_ARG_SIZE) {
        // 超长提示词：使用 stdin 管道模式（不传 -p 参数）
        args = [...this.extraArgs]
        useStdin = true
        console.error(`[cli-provider] Prompt too long (${prompt.length}), using stdin mode`)
      } else {
        // 标准非交互模式：-p "prompt"（与官方 -p/--print 参数一致）
        args = [...this.extraArgs, '-p', prompt]
      }

      const child = spawn(this.command, args, {
        stdio: ['pipe', 'pipe', 'pipe'],
        // 使用临时目录作为 cwd，避免 CLI 工具（如 claude）自动读取项目目录下的
        // CLAUDE.md / .claude/ 等上下文文件，干扰提炼 prompt 的执行。
        cwd: tmpdir(),
      })

      if (useStdin) {
        child.stdin.write(prompt)
        child.stdin.end()
      } else {
        // -p 模式不需要 stdin 输入
        child.stdin.end()
      }

      let stdout = ''
      let stderr = ''

      child.stdout.on('data', (data: Buffer) => {
        stdout += data.toString()
      })

      child.stderr.on('data', (data: Buffer) => {
        stderr += data.toString()
      })

      child.on('close', (code) => {
        if (code !== 0) {
          const errorMsg = stderr.trim() || `CLI 进程退出码: ${code}`
          console.error(`[cli-provider] Error: ${errorMsg}`)
          reject(new Error(errorMsg))
          return
        }
        const result = stdout.trim()
        if (!result) {
          reject(new Error(`CLI 返回内容为空（stderr: ${stderr.trim()}）`))
          return
        }
        resolve(result)
      })

      child.on('error', (err) => {
        if ((err as NodeJS.ErrnoException).code === 'ENOENT') {
          reject(new Error(`CLI 命令 "${this.command}" 未找到，请确认已安装并在 PATH 中`))
        } else {
          reject(new Error(`CLI 启动失败: ${err.message}`))
        }
      })
    })
  }

  /** 检查 CLI 是否可用（执行 --version 验证） */
  async isAvailable(): Promise<boolean> {
    return new Promise((resolve) => {
      const child = spawn(this.command, ['--version'], {
        stdio: ['ignore', 'ignore', 'ignore'],
        cwd: tmpdir(),
      })
      child.on('close', (code) => resolve(code === 0))
      child.on('error', () => resolve(false))
    })
  }

  getInfo() {
    return { provider: 'cli', command: this.command }
  }
}
