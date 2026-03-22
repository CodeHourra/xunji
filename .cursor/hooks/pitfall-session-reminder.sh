#!/usr/bin/env bash
# sessionStart hook：向当前 Composer 会话注入「踩坑文档」维护提醒（stdout 输出 JSON）。
# 说明：hook 无法获知后续对话是否真的会踩坑，因此只做轻量提示，由 Agent 在收尾时自行判断是否落文档。
set -euo pipefail

# 消费 stdin 中的 sessionStart JSON，避免管道阻塞
cat >/dev/null || true

MSG='本仓库在 docs/踩坑/ 维护「踩坑」类文档。若本会话中完成了非平凡问题的根因定位与修复，请在结束前读取项目技能 pitfall-documentation，将条目写入或合并到合适 Markdown；若仅为简单改动或无关排查，可忽略。'

# 使用 Python 做 JSON 字符串转义，避免手写转义错误
exec python3 -c "import json,sys; print(json.dumps({'additional_context': sys.argv[1]}))" "$MSG"
