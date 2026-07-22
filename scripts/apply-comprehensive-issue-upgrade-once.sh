#!/usr/bin/env bash
set -u

REPO="${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is required}"
START_DATE="${ROADMAP_START_DATE:-2026-07-14}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d)"
OUTPUT_FILE="$ROOT_DIR/roadmap-diagnostic.txt"
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
    detail="$(head -n 240 "$TMP_DIR/result.log")"
  else
    heading="No managed drift"
    detail="The authoritative read-only plan contains no create, update, relation, deferred, or warning action."
  fi
else
  heading="Read-only plan failed"
  detail="$(tail -n 240 "$TMP_DIR/plan.log")"
fi

{
  printf 'heading=%s\n' "$heading"
  printf 'exit_code=%s\n' "$plan_rc"
  printf 'mode=read-only --dry-run --force-update\n'
  printf '%s\n' '--- detail ---'
  printf '%s\n' "$detail"
  printf '%s\n' '--- end detail ---'
} >"$OUTPUT_FILE"

printf '[issue-upgrade] wrote %s\n' "$OUTPUT_FILE"
printf '%s\n' "$detail"

# Diagnostic workflow intentionally succeeds so the artifact is always uploaded.
exit 0
