#!/usr/bin/env bash
set -Eeuo pipefail

REPO="${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is required}"
START_DATE="${ROADMAP_START_DATE:-2026-07-14}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

log() { printf '[issue-upgrade] %s\n' "$*"; }
die() { printf '[issue-upgrade] ERROR: %s\n' "$*" >&2; exit 1; }

command -v jq >/dev/null 2>&1 || die "jq is required"
command -v gh >/dev/null 2>&1 || die "gh is required"

cd "$ROOT_DIR"

log "Validating shell and JSON inputs"
bash -n scripts/bootstrap-roadmap.sh
bash -n tests/roadmap/test-bootstrap-roadmap.sh
bash -n scripts/apply-comprehensive-issue-upgrade-once.sh
jq -e '.schemaVersion == 1 and (.issues | type == "array")' roadmap/issue-execution-guidance.json >/dev/null

log "Running roadmap unit and security tests"
bash tests/roadmap/test-bootstrap-roadmap.sh

log "Reading current GitHub state and recording the pre-apply plan"
ROADMAP_SLEEP_SECONDS=0 bash scripts/bootstrap-roadmap.sh \
  --dry-run \
  --phase all \
  --force-update \
  --start-date "$START_DATE" \
  --repo "$REPO" | tee "$TMP_DIR/pre-apply.log"

planned_updates="$(grep -Ec 'dry-run (create|update) (label|milestone|issue)' "$TMP_DIR/pre-apply.log" || true)"
planned_relation_changes="$(grep -Ec 'dry-run (add|remove) (parent|dependency)' "$TMP_DIR/pre-apply.log" || true)"
log "Pre-apply plan: $planned_updates managed field updates; $planned_relation_changes relation changes"

log "Applying reviewed manifests without closing superseded issues"
ROADMAP_SLEEP_SECONDS="${ROADMAP_SLEEP_SECONDS:-0.2}" bash scripts/bootstrap-roadmap.sh \
  --apply \
  --phase all \
  --force-update \
  --start-date "$START_DATE" \
  --repo "$REPO" | tee "$TMP_DIR/apply.log"

log "Auditing post-apply GitHub state"
ROADMAP_SLEEP_SECONDS=0 bash scripts/bootstrap-roadmap.sh \
  --dry-run \
  --phase all \
  --force-update \
  --start-date "$START_DATE" \
  --repo "$REPO" | tee "$TMP_DIR/post-apply.log"

if grep -Eq 'dry-run (create|update) (label|milestone|issue)|dry-run (add|remove) (parent|dependency)|deferred (parent|dependency)|WARNING:' "$TMP_DIR/post-apply.log"; then
  printf '%s\n' '--- unresolved post-apply plan ---' >&2
  grep -E 'dry-run (create|update) (label|milestone|issue)|dry-run (add|remove) (parent|dependency)|deferred (parent|dependency)|WARNING:' "$TMP_DIR/post-apply.log" >&2 || true
  printf '%s\n' '--- end unresolved plan ---' >&2
  die "post-apply audit found managed drift"
fi

no_op_issues="$(grep -Ec 'dry-run no-op issue' "$TMP_DIR/post-apply.log" || true)"
no_op_parents="$(grep -Ec 'dry-run no-op parent' "$TMP_DIR/post-apply.log" || true)"
no_op_dependencies="$(grep -Ec 'dry-run no-op dependencies' "$TMP_DIR/post-apply.log" || true)"

log "Post-apply audit passed: $no_op_issues issues, $no_op_parents parent checks, $no_op_dependencies dependency checks are no-op"

if [[ -n "${GITHUB_STEP_SUMMARY:-}" ]]; then
  {
    printf '## Comprehensive issue upgrade\n\n'
    printf -- '- Pre-apply managed updates: `%s`\n' "$planned_updates"
    printf -- '- Pre-apply relation changes: `%s`\n' "$planned_relation_changes"
    printf -- '- Post-apply no-op issues: `%s`\n' "$no_op_issues"
    printf -- '- Post-apply no-op parent checks: `%s`\n' "$no_op_parents"
    printf -- '- Post-apply no-op dependency checks: `%s`\n' "$no_op_dependencies"
    printf -- '- Superseded issue closure: `not authorized`\n'
  } >>"$GITHUB_STEP_SUMMARY"
fi
