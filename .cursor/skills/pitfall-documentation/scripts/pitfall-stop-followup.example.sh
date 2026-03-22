#!/usr/bin/env bash
# 示例：stop hook — 在 Agent 正常结束时自动追加一条 follow-up，提醒是否写踩坑文档。
# 使用方式：复制到 .cursor/hooks/pitfall-stop-followup.sh，chmod +x，
# 在 .cursor/hooks.json 的 "stop" 数组中增加 { "command": ".cursor/hooks/pitfall-stop-followup.sh", "loop_limit": 3 }
# 注意：会导致「每次」Agent 结束都可能多一轮对话，请阅读 SKILL.md 中「慎用」说明。
set -euo pipefail

INPUT=$(cat || echo '{}')
STATUS=$(python3 -c "import json,sys; d=json.loads(sys.argv[1]); print(d.get('status','completed'))" "$INPUT" 2>/dev/null || echo "completed")

if [ "$STATUS" = "completed" ]; then
  MSG='若本会话涉及根因定位与可验证修复，请读取项目技能 pitfall-documentation，将要点写入或合并到 docs/踩坑/ 下合适 Markdown；否则请简短说明无需记录的原因。'
  exec python3 -c "import json,sys; print(json.dumps({'followup_message': sys.argv[1]}))" "$MSG"
fi

echo '{}'
