#!/usr/bin/env bash
# session-start.sh — surfaces active architectural decisions at session start
# Event: SessionStart
set -euo pipefail

DECISIONS_FILE=".ai/context/decisions.md"

# Skip if no decisions file or it's empty/template
[ -f "$DECISIONS_FILE" ] || exit 0
grep -q "^## " "$DECISIONS_FILE" 2>/dev/null || exit 0

ACTIVE_DECISIONS=$(grep -B1 -A3 "active" "$DECISIONS_FILE" 2>/dev/null | grep -v "^--$" || true)
if [ -n "$ACTIVE_DECISIONS" ]; then
  echo "=== Active Decisions ==="
  echo "$ACTIVE_DECISIONS"
  echo ""
fi
