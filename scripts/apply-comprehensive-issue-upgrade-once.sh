#!/usr/bin/env bash
set -Eeuo pipefail

REPO="${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is required}"
START_DATE="${ROADMAP_START_DATE:-2026-07-14}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
cd "$ROOT_DIR"

log() { printf '[issue-upgrade] %s\n' "$*"; }
die() { printf '[issue-upgrade] ERROR: %s\n' "$*" >&2; exit 1; }

unresolved_pattern='dry-run (create|update) (label|milestone|issue)|dry-run (add|remove) (parent|dependency)|deferred (parent|dependency)|WARNING:'

run_plan() {
  local output="$1"
  shift
  ROADMAP_SLEEP_SECONDS=0 bash scripts/bootstrap-roadmap.sh "$@" >"$output" 2>&1 || {
    tail -n 200 "$output" >&2 || true
    return 1
  }
}

log "Running shell and roadmap regression checks"
bash -n scripts/bootstrap-roadmap.sh
bash -n tests/roadmap/test-bootstrap-roadmap.sh
bash -n tests/roadmap/test-existing-number-map.sh
bash tests/roadmap/test-bootstrap-roadmap.sh >/dev/null
bash tests/roadmap/test-existing-number-map.sh >/dev/null

log "Capturing authoritative pre-apply plan"
run_plan "$TMP_DIR/pre.log" \
  --dry-run --phase all --force-update --start-date "$START_DATE" --repo "$REPO"
pre_changes="$(grep -Ec "$unresolved_pattern" "$TMP_DIR/pre.log" || true)"
log "Pre-apply managed drift lines: $pre_changes"
grep -E "$unresolved_pattern" "$TMP_DIR/pre.log" || true

log "Applying reviewed roadmap without closing superseded issues"
ROADMAP_SLEEP_SECONDS="${ROADMAP_SLEEP_SECONDS:-0.2}" bash scripts/bootstrap-roadmap.sh \
  --apply --phase all --force-update --start-date "$START_DATE" --repo "$REPO" \
  >"$TMP_DIR/apply.log" 2>&1 || {
    tail -n 240 "$TMP_DIR/apply.log" >&2 || true
    die "roadmap apply failed"
  }
applied_changes="$(grep -Ec 'apply (create|update) (label|milestone|issue)|apply (add|remove) (parent|dependency)' "$TMP_DIR/apply.log" || true)"
log "Applied managed changes: $applied_changes"

log "Running strict post-apply no-drift audit"
run_plan "$TMP_DIR/post.log" \
  --dry-run --phase all --force-update --start-date "$START_DATE" --repo "$REPO"
if grep -Eq "$unresolved_pattern" "$TMP_DIR/post.log"; then
  grep -E "$unresolved_pattern" "$TMP_DIR/post.log" >&2 || true
  die "post-apply audit found managed drift"
fi

no_op_issues="$(grep -Ec 'dry-run no-op issue' "$TMP_DIR/post.log" || true)"
no_op_milestones="$(grep -Ec 'dry-run no-op milestone' "$TMP_DIR/post.log" || true)"
no_op_parents="$(grep -Ec 'dry-run no-op parent' "$TMP_DIR/post.log" || true)"
no_op_dependencies="$(grep -Ec 'dry-run no-op dependencies' "$TMP_DIR/post.log" || true)"

log "Audit passed: issues=$no_op_issues milestones=$no_op_milestones parents=$no_op_parents dependencies=$no_op_dependencies"

if [[ -n "${GITHUB_STEP_SUMMARY:-}" ]]; then
  {
    printf '## Comprehensive issue guidance apply\n\n'
    printf -- '- Pre-apply drift lines: `%s`\n' "$pre_changes"
    printf -- '- Applied managed changes: `%s`\n' "$applied_changes"
    printf -- '- No-op issues: `%s`\n' "$no_op_issues"
    printf -- '- No-op milestones: `%s`\n' "$no_op_milestones"
    printf -- '- No-op parent checks: `%s`\n' "$no_op_parents"
    printf -- '- No-op dependency checks: `%s`\n' "$no_op_dependencies"
    printf -- '- Superseded issue closure: `not authorized`\n'
  } >>"$GITHUB_STEP_SUMMARY"
fi
