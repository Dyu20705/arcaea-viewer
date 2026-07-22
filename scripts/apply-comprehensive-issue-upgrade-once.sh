#!/usr/bin/env bash
set -Eeuo pipefail

REPO="${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is required}"
PR_NUMBER="${PR_NUMBER:?PR_NUMBER is required}"
START_DATE="${ROADMAP_START_DATE:-2026-07-14}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

log() { printf '[issue-upgrade] %s\n' "$*"; }
die() { printf '[issue-upgrade] ERROR: %s\n' "$*" >&2; exit 1; }

command -v jq >/dev/null 2>&1 || die "jq is required"
command -v gh >/dev/null 2>&1 || die "gh is required"
cd "$ROOT_DIR"

unresolved_pattern='dry-run (create|update) (label|milestone|issue)|dry-run (add|remove) (parent|dependency)|deferred (parent|dependency)|WARNING:'

log "Validating temporary diagnostic"
bash -n scripts/bootstrap-roadmap.sh
bash -n tests/roadmap/test-bootstrap-roadmap.sh
bash -n tests/roadmap/test-existing-number-map.sh
jq -e '.schemaVersion == 1 and (.issues | type == "array")' roadmap/issue-execution-guidance.json >/dev/null
bash tests/roadmap/test-bootstrap-roadmap.sh >/dev/null
bash tests/roadmap/test-existing-number-map.sh >/dev/null

log "Running authoritative read-only drift plan"
if ! ROADMAP_SLEEP_SECONDS=0 bash scripts/bootstrap-roadmap.sh \
  --dry-run \
  --phase all \
  --force-update \
  --start-date "$START_DATE" \
  --repo "$REPO" >"$TMP_DIR/plan.log" 2>&1; then
  tail -n 200 "$TMP_DIR/plan.log" >&2 || true
  die "read-only roadmap plan failed"
fi

grep -E "$unresolved_pattern" "$TMP_DIR/plan.log" >"$TMP_DIR/drift.log" || true
if [[ -s "$TMP_DIR/drift.log" ]]; then
  diagnostic="$(head -n 160 "$TMP_DIR/drift.log")"
  body="<!-- roadmap-audit-diagnostic -->
## Temporary roadmap audit diagnostic

The authoritative read-only plan reports:

\`\`\`text
$diagnostic
\`\`\`

No issue write was performed by this diagnostic run."
  printf '%s' "$body" >"$TMP_DIR/comment.md"
  gh api --method POST "repos/$REPO/issues/$PR_NUMBER/comments" --raw-field body="$(cat "$TMP_DIR/comment.md")" >/dev/null
  printf '%s\n' '--- exact managed drift ---' >&2
  cat "$TMP_DIR/drift.log" >&2
  printf '%s\n' '--- end managed drift ---' >&2
  die "read-only plan still contains managed drift"
fi

body="<!-- roadmap-audit-diagnostic -->
## Temporary roadmap audit diagnostic

The authoritative read-only plan is clean: no managed field, parent, or dependency drift remains. No issue write was performed by this diagnostic run."
gh api --method POST "repos/$REPO/issues/$PR_NUMBER/comments" --raw-field body="$body" >/dev/null
log "Read-only no-drift diagnostic passed"
