#!/usr/bin/env bash
set -u

REPO="${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is required}"
PR_NUMBER="${PR_NUMBER:?PR_NUMBER is required}"
START_DATE="${ROADMAP_START_DATE:-2026-07-14}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
cd "$ROOT_DIR"

unresolved_pattern='dry-run (create|update) (label|milestone|issue)|dry-run (add|remove) (parent|dependency)|deferred (parent|dependency)|WARNING:'

ROADMAP_SLEEP_SECONDS=0 bash scripts/bootstrap-roadmap.sh \
  --dry-run \
  --phase all \
  --force-update \
  --start-date "$START_DATE" \
  --repo "$REPO" >"$TMP_DIR/plan.log" 2>&1
plan_rc=$?

if [[ $plan_rc -eq 0 ]]; then
  grep -E "$unresolved_pattern" "$TMP_DIR/plan.log" >"$TMP_DIR/result.log" || true
  if [[ -s "$TMP_DIR/result.log" ]]; then
    heading="Managed drift remains"
    detail="$(head -n 160 "$TMP_DIR/result.log")"
  else
    heading="No managed drift"
    detail="The authoritative read-only plan contains no create, update, relation, deferred, or warning action."
  fi
else
  heading="Read-only plan failed"
  detail="$(tail -n 160 "$TMP_DIR/plan.log")"
fi

body="<!-- roadmap-audit-diagnostic -->
## Temporary roadmap audit diagnostic — $heading

- Exit code: \`$plan_rc\`
- Mode: read-only \`--dry-run --force-update\`

\`\`\`text
$detail
\`\`\`

No roadmap issue write was performed by this diagnostic run."

jq -n --arg body "$body" '{body:$body}' >"$TMP_DIR/comment.json"
post_rc=0
gh api --method POST "repos/$REPO/issues/$PR_NUMBER/comments" --input "$TMP_DIR/comment.json" >/dev/null || post_rc=$?

printf '[issue-upgrade] plan_rc=%s post_rc=%s heading=%s\n' "$plan_rc" "$post_rc" "$heading"
printf '%s\n' "$detail"

# Diagnostic workflow intentionally succeeds so the captured result remains inspectable.
exit 0
