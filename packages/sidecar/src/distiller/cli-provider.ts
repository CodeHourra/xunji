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

import { execSync, spawn } from 'node:child_process'
import { existsSync } from 'node:fs'
import { delimiter, dirname, join } from 'node:path'
import { homedir, tmpdir } from 'os'
import type { DistillResult } from './api-provider'
import { logCliMergedPrompt } from './payload-log'
import { CONTENT_HINT_CLI } from './prompts'
import { distillLog } from './trace'

/**
 * Unix 下「交互 + 登录」shell 的 PATH（仅 env 拉取失败时回退使用），仅解析一次。
 */
let unixInteractivePath: string | null | undefined

/**
 * 交互 shell 的完整环境（`zsh -il -c env`）：含 NVM_DIR、NVM_BIN、ServBay_* 等。
 * 许多内网/魔改 CLI 会自己解析 node，仅合并 PATH 不够，需整份 env。
 */
let unixInteractiveEnv: Record<string, string> | null | undefined

/**
 * `command -v node` 所在目录（解决 `#!/usr/bin/env node` shebang），仅解析一次。
 */
let nodeBinDirFromShell: string | null | undefined

/** 解析 `/bin/zsh -il -c env` 的逐行输出为键值表（首行首个 `=` 分割） */
function parseEnvLines(out: string): Record<string, string> {
  const parsed: Record<string, string> = {}
  for (const line of out.split('\n')) {
    if (!line) continue
    const eq = line.indexOf('=')
    if (eq <= 0) continue
    const key = line.slice(0, eq)
    if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(key)) continue
    parsed[key] = line.slice(eq + 1)
  }
  return parsed
}

function loadUnixInteractiveEnvFromShellOnce(): void {
  if (unixInteractiveEnv !== undefined) return
  unixInteractiveEnv = null
  if (process.platform === 'win32') return
  try {
    // 使用 command env 避免 zsh alias 覆盖；输出与终端 `env` 一致
    const cmd =
      process.platform === 'darwin'
        ? '/bin/zsh -il -c \'command env\''
        : '/bin/bash -il -c \'command env\''
    const out = execSync(cmd, {
      encoding: 'utf8',
      timeout: 25_000,
      maxBuffer: 10 * 1024 * 1024,
    })
    const parsed = parseEnvLines(out)
    if (Object.keys(parsed).length === 0) {
      console.error('[cli-provider] 交互 shell env 解析为空，将回退为仅 PATH')
    } else {
      unixInteractiveEnv = parsed
      console.error(
        `[cli-provider] 已合并交互 shell 完整环境（${Object.keys(parsed).length} 项，含 NVM/ServBay 等）`,
      )
    }
  } catch (e) {
    console.error('[cli-provider] 读取交互 shell env 失败，将回退为仅 PATH:', e)
    unixInteractiveEnv = null
  }
}

function loadUnixInteractivePathOnce(): void {
  if (unixInteractivePath !== undefined) return
  unixInteractivePath = null
  if (process.platform === 'win32') return
  try {
    const cmd =
      process.platform === 'darwin'
        ? '/bin/zsh -il -c \'printf %s "$PATH"\''
        : '/bin/bash -il -c \'printf %s "$PATH"\''
    unixInteractivePath =
      execSync(cmd, {
        encoding: 'utf8',
        timeout: 20_000,
        maxBuffer: 512 * 1024,
      }).trim() || null
    if (unixInteractivePath) {
      console.error(
        '[cli-provider] 已合并交互式登录 shell 的 PATH（含 nvm/.zshrc 等）',
      )
    }
  } catch (e) {
    console.error('[cli-provider] 无法读取交互 shell PATH:', e)
  }
}

function loadNodeBinDirFromShellOnce(): void {
  if (nodeBinDirFromShell !== undefined) return
  nodeBinDirFromShell = null
  if (process.platform === 'win32') return
  try {
    const cmd =
      process.platform === 'darwin'
        ? '/bin/zsh -il -c \'command -v node\''
        : '/bin/bash -il -c \'command -v node\''
    const line = execSync(cmd, { encoding: 'utf8', timeout: 20_000 })
      .trim()
      .split(/\r?\n/)[0]
    if (
      line &&
      !/not found/i.test(line) &&
      existsSync(line)
    ) {
      nodeBinDirFromShell = dirname(line)
      console.error('[cli-provider] 已将 node 所在目录并入 PATH:', nodeBinDirFromShell)
    }
  } catch (e) {
    console.error(
      '[cli-provider] command -v node 失败（shebang 仍可能报 env: node）:',
      e,
    )
  }
}

/**
 * 为 **仅本次** `spawn(用户 CLI)` 构造 `env` 选项：优先注入交互 shell 的完整变量，再前置 PATH。
 *
 * ```text
 * 边界（不影响寻迹主程序）：
 * - 返回值只传给 child_process.spawn({ env })，不执行 Object.assign(process.env, …)，
 *   不修改 sidecar 进程自身的 process.env，更不会影响 Tauri（Rust）主进程。
 * - 寻迹本体：桌面壳 = Rust；sidecar = 已编译的 Bun 单文件，运行时与用户机「node -v」无关。
 * - 用户 NVM/ServBay 的 node 版本只作用于被拉起的 **外部 CLI 子进程**，这正是提炼所需的隔离。
 * ```
 *
 * 第三方 CLI（含 ServBay / 自研 node 解析）常读取 NVM_BIN、SERVBAY_* 等，仅改 PATH 不够。
 */
function envForCliSpawn(): NodeJS.ProcessEnv {
  loadUnixInteractiveEnvFromShellOnce()
  if (!unixInteractiveEnv) {
    loadUnixInteractivePathOnce()
  }
  loadNodeBinDirFromShellOnce()

  const home = homedir()
  const fromShell = unixInteractiveEnv ?? {}
  const base: NodeJS.ProcessEnv = { ...process.env, ...fromShell }

  const pathPrefix = [
    nodeBinDirFromShell,
    '/opt/homebrew/opt/node/bin',
    '/usr/local/opt/node/bin',
    '/opt/homebrew/bin',
    '/opt/homebrew/sbin',
    '/usr/local/bin',
    join(home, '.local', 'bin'),
    join(home, '.bun', 'bin'),
    join(home, 'bin'),
    !unixInteractiveEnv ? unixInteractivePath : undefined,
  ].filter((s): s is string => typeof s === 'string' && s.length > 0)

  const existingPath = base.PATH ?? process.env.PATH ?? ''
  const mergedPath = [...pathPrefix, existingPath].filter(Boolean).join(delimiter)
  return { ...base, PATH: mergedPath }
}

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
   * @param callLabel - judge_value / distill_full，用于日志
   */
  async distill(
    systemPrompt: string,
    content: string,
    callLabel: string = 'distill',
    traceId: string = 'unknown',
  ): Promise<DistillResult> {
    // CLI 不支持多角色消息，将 system prompt + 尾部提示 + 对话内容拼为一条。
    // 使用醒目的分隔线，与 CONTENT_HINT_CLI 中的描述一致。
    const systemPart = `${systemPrompt}${CONTENT_HINT_CLI}`
    const fullPrompt = `${systemPart}\n\n════════════════════════════════════════\n\n${content}`

    logCliMergedPrompt({
      traceId,
      callLabel,
      command: this.command,
      systemPartLen: systemPart.length,
      userLen: content.length,
      mergedLen: fullPrompt.length,
      mergedFull: fullPrompt,
    })

    const responseText = await this.runCli(fullPrompt)

    console.error(
      distillLog(traceId, `[cli-provider] Response: ${responseText.length} chars`),
    )

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
        env: envForCliSpawn(),
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
          let errorMsg = stderr.trim() || `CLI 进程退出码: ${code}`
          // shebang #!/usr/bin/env node：PATH 缺 node
          if (/env:\s*node:/i.test(errorMsg)) {
            errorMsg += ` [提示] 已合并交互 shell 的 PATH 与 node 目录；若仍失败请确认本机已安装 node。`
          } else if (/Node\.js executable for command/i.test(errorMsg)) {
            // 常见：内网 CLI / ServBay 自研逻辑读 NVM_BIN 等，仅 PATH 不够
            errorMsg += ` [提示] 已注入「command env」完整环境；若仍失败请在终端对比「node -v」与同一 CLI 是否正常。`
          }
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
          // 报错中的引号内容即本次 spawn 的第一个参数（短名或绝对路径），便于与设置项对照
          const looksLikePath =
            this.command.includes('/') || this.command.includes('\\')
          const pathHint = looksLikePath
            ? existsSync(this.command)
              ? '路径存在但无法启动（常见于脚本 shebang 依赖的 node 不在 PATH；可改用「命令名」并保证 PATH，或确保 node 在固定路径）'
              : '该路径下没有此文件，请核对设置是否已保存，或路径是否属于本机'
            : '若从桌面应用启动，请确认已安装该 CLI；仍失败可在设置中填写「可执行文件绝对路径」，或将其放入 /opt/homebrew/bin、~/.local/bin 等目录'
          reject(
            new Error(
              `CLI 命令 "${this.command}" 未找到。${pathHint}`,
            ),
          )
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
        env: envForCliSpawn(),
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
