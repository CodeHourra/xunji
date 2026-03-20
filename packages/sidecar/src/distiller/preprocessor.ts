/**
 * 对话内容前处理器 —— 清理 AI 内部标签 + 超长截断。
 *
 * 在送入 LLM 提炼之前，需要：
 * 1. 剥离 <thinking>、<tool_use> 等对知识提炼无价值的 XML 标签
 * 2. 将超长对话截断到安全长度，保留头部（问题背景）和尾部（最终结论）
 */

/** 需要从对话中剥离的 XML 标签模式 */
const STRIP_PATTERNS = [
  /<thinking>[\s\S]*?<\/thinking>/g,
  /<antml_function_calls>[\s\S]*?<\/antml_function_calls>/g,
  /<function_calls>[\s\S]*?<\/function_calls>/g,
  /<tool_use>[\s\S]*?<\/tool_use>/g,
  /<tool_call>[\s\S]*?<\/tool_call>/g,
  /<tool_result>[\s\S]*?<\/tool_result>/g,
  /<antml_thinking>[\s\S]*?<\/antml_thinking>/g,
]

/**
 * 清理对话内容中的 AI 内部数据标签。
 * 这些内容（推理过程、工具调用 XML）对知识提炼无价值，且浪费 token。
 */
export function clean(content: string): string {
  let result = content
  for (const pattern of STRIP_PATTERNS) {
    result = result.replace(pattern, '')
  }
  // 清除连续多个空行（清理后留下的空白）
  result = result.replace(/\n{3,}/g, '\n\n')
  return result.trim()
}

/**
 * 截断超长内容，保留头尾以保持上下文完整性。
 *
 * 策略：保留开头 headChars（问题背景）+ 结尾 tailChars（最终结论），
 * 中间以省略标记替代。
 */
export function truncate(
  content: string,
  maxChars: number = 12000,
  headChars: number = 8000,
  tailChars: number = 4000,
): string {
  if (content.length <= maxChars) {
    return content
  }

  const head = content.slice(0, headChars)
  const tail = content.slice(-tailChars)
  return `${head}\n\n[... 中间内容已省略（原文 ${content.length} 字符）...]\n\n${tail}`
}
