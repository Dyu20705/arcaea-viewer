#!/usr/bin/env bash
set -Eeuo pipefail

REPO="${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is required}"
START_DATE="${ROADMAP_START_DATE:-2026-07-14}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_FILE="$ROOT_DIR/roadmap-diagnostic.txt"
cd "$ROOT_DIR"

command -v jq >/dev/null 2>&1
command -v gh >/dev/null 2>&1

current_file="$(mktemp)"
desired_file="$(mktemp)"
trap 'rm -f "$current_file" "$desired_file"' EXIT

gh api --paginate "repos/$REPO/milestones?state=all&per_page=100" |
  jq -s 'add // [] | map({number,state,title,description:(.description // ""),due_on:(.due_on // null),updated_at})' >"$current_file"

jq --arg start "$START_DATE" '
  [.milestones[] |
    . as $m |
    {
      key: .key,
      title: .title,
      description: (.description // ""),
      due_on: (if .dueOffsetDays == null then null else "__COMPUTE__" end),
      dueOffsetDays: (.dueOffsetDays // null)
    }
  ]
' roadmap/milestones.json >"$desired_file.raw"

jq -c '.[]' "$desired_file.raw" | while IFS= read -r row; do
  offset="$(jq -r '.dueOffsetDays // empty' <<<"$row")"
  due="null"
  if [[ -n "$offset" ]]; then
    due="$(date -u -d "$START_DATE + $offset days" +%Y-%m-%dT23:59:59Z)"
    jq --arg due "$due" '.due_on=$due | del(.dueOffsetDays)' <<<"$row"
  else
    jq '.due_on=null | del(.dueOffsetDays)' <<<"$row"
  fi
done | jq -s '.' >"$desired_file"

jq -n --slurpfile current "$current_file" --slurpfile desired "$desired_file" '
  {
    current: $current[0],
    desired: $desired[0],
    comparison: [
      $desired[0][] as $d |
      ($current[0] | map(select(.title == $d.title)) | .[0] // null) as $c |
      {
        key: $d.key,
        title: $d.title,
        current: $c,
        desired: $d,
        description_equal: (($c.description // "") == $d.description),
        state_open: (($c.state // "") == "open"),
        due_equal: (($c.due_on // null) == $d.due_on)
      }
    ]
  }
' >"$OUTPUT_FILE"

printf '[issue-upgrade] wrote milestone comparison to %s\n' "$OUTPUT_FILE"
