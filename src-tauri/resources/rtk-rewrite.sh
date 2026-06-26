#!/usr/bin/env bash
# CC Switch RTK hook — thin delegate to `rtk rewrite`.
# Requires: rtk on PATH, jq
set -euo pipefail

INPUT=$(cat)
CMD=$(jq -r '.tool_input.command // empty' <<<"$INPUT")

if [ -z "$CMD" ]; then
  exit 0
fi

REWRITTEN=$(rtk rewrite "$CMD" 2>/dev/null) || exit 0
EXIT_CODE=$?

case $EXIT_CODE in
  0)
    [ "$CMD" = "$REWRITTEN" ] && exit 0
    ;;
  1|2)
    exit 0
    ;;
  3)
    ;;
  *)
    exit 0
    ;;
esac

if [ "$EXIT_CODE" -eq 3 ]; then
  jq -c --arg cmd "$REWRITTEN" \
    '.tool_input.command = $cmd | {
      "hookSpecificOutput": {
        "hookEventName": "PreToolUse",
        "updatedInput": .tool_input
      }
    }' <<<"$INPUT"
else
  jq -c --arg cmd "$REWRITTEN" \
    '.tool_input.command = $cmd | {
      "hookSpecificOutput": {
        "hookEventName": "PreToolUse",
        "permissionDecision": "allow",
        "permissionDecisionReason": "CC Switch RTK auto-rewrite",
        "updatedInput": .tool_input
      }
    }' <<<"$INPUT"
fi
