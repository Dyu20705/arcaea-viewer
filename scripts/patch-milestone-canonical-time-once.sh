#!/usr/bin/env bash
set -Eeuo pipefail

REPO="${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is required}"
BRANCH="${PATCH_BRANCH:-ops/comprehensive-issue-upgrade}"
BOOTSTRAP="scripts/bootstrap-roadmap.sh"
TEST_FILE="tests/roadmap/test-bootstrap-roadmap.sh"

log() { printf '[milestone-patch] %s\n' "$*"; }
die() { printf '[milestone-patch] ERROR: %s\n' "$*" >&2; exit 1; }

replace_once() {
  local file="$1" old="$2" new="$3" count
  count="$(grep -Foc -- "$old" "$file" || true)"
  [[ "$count" == "1" ]] || die "expected exactly one match in $file, found $count: $old"
  sed -i "s|$old|$new|" "$file"
  grep -Fq -- "$new" "$file" || die "replacement missing in $file"
}

insert_after_once() {
  local file="$1" anchor="$2" line="$3" count tmp
  count="$(grep -Foc -- "$anchor" "$file" || true)"
  [[ "$count" == "1" ]] || die "expected exactly one anchor in $file, found $count"
  grep -Fq -- "$line" "$file" && return 0
  tmp="$(mktemp)"
  awk -v anchor="$anchor" -v addition="$line" '{print; if ($0 == anchor) print addition}' "$file" >"$tmp"
  mv "$tmp" "$file"
  grep -Fq -- "$line" "$file" || die "failed to insert regression assertion"
}

publish_file() {
  local path="$1" message="$2" sha payload
  sha="$(gh api "repos/$REPO/contents/$path?ref=$BRANCH" --jq '.sha')"
  jq -n \
    --arg message "$message" \
    --arg content "$(base64 -w0 "$path")" \
    --arg sha "$sha" \
    --arg branch "$BRANCH" \
    '{message:$message,content:$content,sha:$sha,branch:$branch}' >"$path.payload.json"
  gh api --method PUT "repos/$REPO/contents/$path" --input "$path.payload.json" >/dev/null
  rm -f "$path.payload.json"
  log "published $path"
}

log "Applying asserted canonical timestamp patch"
replace_once \
  "$BOOTSTRAP" \
  '+%Y-%m-%dT23:59:59Z' \
  '+%Y-%m-%dT00:00:00Z'

replace_once \
  "$TEST_FILE" \
  '2026-07-14T23:59:59Z' \
  '2026-07-14T00:00:00Z'

insert_after_once \
  "$TEST_FILE" \
  'assert_contains "$out" "dry-run no-op issue #1 [root]"' \
  'assert_contains "$out" "dry-run no-op milestone: m1 -> M1 due 2026-07-14T00:00:00Z"'

log "Running focused regression checks"
bash -n "$BOOTSTRAP"
bash -n "$TEST_FILE"
bash "$TEST_FILE"
bash tests/roadmap/test-existing-number-map.sh

publish_file "$BOOTSTRAP" "fix(roadmap): canonicalize milestone due timestamps"
publish_file "$TEST_FILE" "test(roadmap): cover GitHub milestone normalization"

log "Patch completed"
