#!/usr/bin/env bash
set -Eeuo pipefail

API_VERSION="${GITHUB_API_VERSION:-2026-03-10}"
MODE="dry-run"
PHASE="all"
START_DATE="$(date -u +%F)"
FORCE_UPDATE="false"
CLOSE_SUPERSEDED="false"
REPO="${GITHUB_REPOSITORY:-}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LABELS_FILE="$ROOT_DIR/roadmap/labels.json"
MILESTONES_FILE="$ROOT_DIR/roadmap/milestones.json"
ISSUES_FILE="$ROOT_DIR/roadmap/issues.json"
TMP_DIR="$(mktemp -d)"
ISSUE_MAP="$TMP_DIR/issues.json"
MILESTONE_MAP="$TMP_DIR/milestones.json"

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

usage() {
  cat <<'EOF'
Usage:
  bootstrap-roadmap.sh [options]

Options:
  --dry-run                 Print changes without writing (default).
  --apply                   Apply changes to GitHub.
  --phase <name>            all, roadmap, week-1 ... week-6, post-mvp, cleanup.
  --start-date YYYY-MM-DD   Date used to calculate milestone due dates.
  --force-update            Enforce managed issue state and repair all relations.
  --close-superseded        Allow issues marked closed/not_planned to be closed.
  --repo OWNER/REPO         Override repository detection.
  -h, --help                Show help.
EOF
}

log() {
  printf '[roadmap] %s\n' "$*"
}

die() {
  printf '[roadmap] ERROR: %s\n' "$*" >&2
  exit 1
}

while (($#)); do
  case "$1" in
    --dry-run) MODE="dry-run"; shift ;;
    --apply) MODE="apply"; shift ;;
    --phase) PHASE="${2:?missing phase}"; shift 2 ;;
    --start-date) START_DATE="${2:?missing start date}"; shift 2 ;;
    --force-update) FORCE_UPDATE="true"; shift ;;
    --close-superseded) CLOSE_SUPERSEDED="true"; shift ;;
    --repo) REPO="${2:?missing owner/repo}"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) die "unknown argument: $1" ;;
  esac
done

command -v jq >/dev/null 2>&1 || die "'jq' is required"
command -v date >/dev/null 2>&1 || die "GNU 'date' is required"

if [[ "$MODE" == "apply" || -z "$REPO" ]]; then
  command -v gh >/dev/null 2>&1 || die "GitHub CLI 'gh' is required"
fi

jq -e '.schemaVersion == 1 and (.labels | type == "array")' "$LABELS_FILE" >/dev/null ||
  die "invalid labels manifest"
jq -e '.schemaVersion == 1 and (.milestones | type == "array")' "$MILESTONES_FILE" >/dev/null ||
  die "invalid milestones manifest"
jq -e '.schemaVersion == 1 and (.issues | type == "array")' "$ISSUES_FILE" >/dev/null ||
  die "invalid issues manifest"

date -u -d "$START_DATE" +%F >/dev/null 2>&1 ||
  die "--start-date must be understood by GNU date as YYYY-MM-DD"

if [[ -z "$REPO" ]]; then
  REPO="$(gh repo view --json nameWithOwner --jq '.nameWithOwner')"
fi
[[ "$REPO" == */* ]] || die "repository must be OWNER/REPO"

MANIFEST_REPO="$(jq -r '.repository' "$ISSUES_FILE")"
[[ "$REPO" == "$MANIFEST_REPO" ]] ||
  die "manifest targets $MANIFEST_REPO but current repository is $REPO"

api() {
  gh api \
    -H "Accept: application/vnd.github+json" \
    -H "X-GitHub-Api-Version: $API_VERSION" \
    "$@"
}

urlencode() {
  jq -rn --arg value "$1" '$value | @uri'
}

phase_selected() {
  local issue_phase="$1"
  [[ "$PHASE" == "all" || "$PHASE" == "$issue_phase" ]]
}

sleep_after_write() {
  if [[ "$MODE" == "apply" ]]; then
    sleep 1
  fi
}

render_issue_body() {
  local issue_json="$1"
  jq -r '
    def bullets($values):
      if (($values // []) | length) == 0 then ""
      else (($values // []) | map("- " + .) | join("\n")) + "\n"
      end;
    def section($title; $values):
      if (($values // []) | length) == 0 then ""
      else "## " + $title + "\n\n" + bullets($values) + "\n"
      end;
    "<!-- roadmap-key: " + .key + " -->\n\n" +
    "> Managed by `roadmap/issues.json` and `scripts/bootstrap-roadmap.sh`.\n\n" +
    "## Outcome\n\n" + .outcome + "\n\n" +
    section("Scope"; .scope) +
    section("Non-goals"; .nonGoals) +
    section("UX requirements"; .uxRequirements) +
    section("Technical constraints"; .technicalConstraints) +
    section("Acceptance criteria"; .acceptanceCriteria) +
    section("Test plan"; .testPlan) +
    section("Required evidence"; .evidence) +
    section("Definition of Done"; .definitionOfDone) +
    (if .supersededBy then
       "## Superseded by\n\n- Roadmap key: `" + .supersededBy + "`\n\n"
     else "" end) +
    "## Planning metadata\n\n" +
    "- Roadmap key: `" + .key + "`\n" +
    "- Phase: `" + .phase + "`\n" +
    "- Milestone key: `" + (.milestone // "none") + "`\n" +
    (if ((.blockedBy // []) | length) > 0 then
       "- Blocked by roadmap keys: " + ((.blockedBy // []) | map("`" + . + "`") | join(", ")) + "\n"
     else "" end)
  ' <<<"$issue_json"
}

load_remote_state() {
  if [[ "$MODE" == "dry-run" ]]; then
    printf '{}\n' >"$ISSUE_MAP"
    printf '{}\n' >"$MILESTONE_MAP"
    return
  fi

  log "Loading current issues and milestones"
  api --paginate "repos/$REPO/issues?state=all&per_page=100" |
    jq -s '
      add
      | map(select(has("pull_request") | not))
      | reduce .[] as $issue ({};
          if (($issue.body // "") | capture("roadmap-key: (?<key>[A-Za-z0-9._-]+)")? | .key) as $key
          then .[$key] = {number: $issue.number, id: $issue.id, state: $issue.state, body: ($issue.body // "")}
          else .
          end
        )
    ' >"$ISSUE_MAP"

  api --paginate "repos/$REPO/milestones?state=all&per_page=100" |
    jq -s 'add | reduce .[] as $m ({}; .[$m.title] = {number: $m.number, state: $m.state})' \
    >"$MILESTONE_MAP"
}

upsert_labels() {
  log "Processing labels"
  jq -c '.labels[]' "$LABELS_FILE" | while IFS= read -r label; do
    local_name="$(jq -r '.name' <<<"$label")"
    color="$(jq -r '.color' <<<"$label")"
    description="$(jq -r '.description // ""' <<<"$label")"
    encoded="$(urlencode "$local_name")"

    if [[ "$MODE" == "dry-run" ]]; then
      log "DRY-RUN upsert label: $local_name"
      continue
    fi

    if api "repos/$REPO/labels/$encoded" >/dev/null 2>&1; then
      jq -n \
        --arg name "$local_name" \
        --arg color "$color" \
        --arg description "$description" \
        '{name:$name,color:$color,description:$description}' |
        api --method PATCH "repos/$REPO/labels/$encoded" --input - >/dev/null
    else
      jq -n \
        --arg name "$local_name" \
        --arg color "$color" \
        --arg description "$description" \
        '{name:$name,color:$color,description:$description}' |
        api --method POST "repos/$REPO/labels" --input - >/dev/null
    fi
    sleep_after_write
  done
}

upsert_milestones() {
  log "Processing milestones from start date $START_DATE"
  jq -c '.milestones[]' "$MILESTONES_FILE" | while IFS= read -r milestone; do
    key="$(jq -r '.key' <<<"$milestone")"
    title="$(jq -r '.title' <<<"$milestone")"
    description="$(jq -r '.description // ""' <<<"$milestone")"
    offset="$(jq -r '.dueOffsetDays // empty' <<<"$milestone")"
    due_on=""
    if [[ -n "$offset" ]]; then
      due_on="$(date -u -d "$START_DATE + $offset days" +%Y-%m-%dT23:59:59Z)"
    fi

    if [[ "$MODE" == "dry-run" ]]; then
      log "DRY-RUN upsert milestone: $key -> $title${due_on:+ due $due_on}"
      continue
    fi

    number="$(jq -r --arg title "$title" '.[$title].number // empty' "$MILESTONE_MAP")"
    payload="$(jq -n \
      --arg title "$title" \
      --arg description "$description" \
      --arg due_on "$due_on" \
      '{title:$title,description:$description,state:"open"}
       + (if $due_on == "" then {} else {due_on:$due_on} end)')"

    if [[ -n "$number" ]]; then
      api --method PATCH "repos/$REPO/milestones/$number" --input - <<<"$payload" >/dev/null
    else
      response="$(api --method POST "repos/$REPO/milestones" --input - <<<"$payload")"
      number="$(jq -r '.number' <<<"$response")"
      jq --arg title "$title" --argjson number "$number" \
        '.[$title] = {number:$number,state:"open"}' "$MILESTONE_MAP" >"$MILESTONE_MAP.tmp"
      mv "$MILESTONE_MAP.tmp" "$MILESTONE_MAP"
    fi
    sleep_after_write
  done
}

milestone_number_for_key() {
  local key="$1"
  [[ -n "$key" && "$key" != "null" ]] || return 0
  local title
  title="$(jq -r --arg key "$key" '.milestones[] | select(.key == $key) | .title' "$MILESTONES_FILE")"
  [[ -n "$title" ]] || die "unknown milestone key: $key"
  if [[ "$MODE" == "dry-run" ]]; then
    printf '0'
  else
    jq -r --arg title "$title" '.[$title].number' "$MILESTONE_MAP"
  fi
}

issue_number_by_key() {
  local key="$1"
  jq -r --arg key "$key" '.[$key].number // empty' "$ISSUE_MAP"
}

issue_id_by_key() {
  local key="$1"
  jq -r --arg key "$key" '.[$key].id // empty' "$ISSUE_MAP"
}

record_issue() {
  local key="$1" number="$2" id="$3" state="$4"
  jq \
    --arg key "$key" \
    --argjson number "$number" \
    --argjson id "$id" \
    --arg state "$state" \
    '.[$key] = {number:$number,id:$id,state:$state}' \
    "$ISSUE_MAP" >"$ISSUE_MAP.tmp"
  mv "$ISSUE_MAP.tmp" "$ISSUE_MAP"
}

resolve_existing_issue() {
  local issue_json="$1"
  local key existing number response
  key="$(jq -r '.key' <<<"$issue_json")"
  existing="$(jq -r '.existingNumber // empty' <<<"$issue_json")"
  number="$(issue_number_by_key "$key")"
  if [[ -n "$number" ]]; then
    printf '%s' "$number"
    return
  fi
  if [[ -n "$existing" && "$MODE" == "apply" ]]; then
    response="$(api "repos/$REPO/issues/$existing")"
    [[ "$(jq -r 'has("pull_request")' <<<"$response")" == "false" ]] ||
      die "#$existing is a pull request, not an issue"
    record_issue "$key" "$(jq -r '.number' <<<"$response")" "$(jq -r '.id' <<<"$response")" "$(jq -r '.state' <<<"$response")"
    printf '%s' "$existing"
  fi
}

upsert_issues() {
  log "Processing managed issues for phase '$PHASE'"
  jq -c '.issues[]' "$ISSUES_FILE" | while IFS= read -r issue_json; do
    issue_phase="$(jq -r '.phase' <<<"$issue_json")"
    phase_selected "$issue_phase" || continue

    key="$(jq -r '.key' <<<"$issue_json")"
    title="$(jq -r '.title' <<<"$issue_json")"
    target_state="$(jq -r '.state // "open"' <<<"$issue_json")"
    state_reason="$(jq -r '.stateReason // empty' <<<"$issue_json")"
    milestone_key="$(jq -r '.milestone // empty' <<<"$issue_json")"
    body="$(render_issue_body "$issue_json")"
    labels_json="$(jq -c '.labels // []' <<<"$issue_json")"
    assignees_json="$(jq -c '.assignees // []' <<<"$issue_json")"
    milestone_number="$(milestone_number_for_key "$milestone_key")"
    number="$(resolve_existing_issue "$issue_json")"

    desired_state=""
    desired_reason=""
    if [[ "$target_state" == "closed" ]]; then
      if [[ "$CLOSE_SUPERSEDED" == "true" ]]; then
        desired_state="closed"
        desired_reason="${state_reason:-not_planned}"
      fi
    elif [[ "$FORCE_UPDATE" == "true" ]]; then
      desired_state="open"
    fi

    payload="$(jq -n \
      --arg title "$title" \
      --arg body "$body" \
      --argjson labels "$labels_json" \
      --argjson assignees "$assignees_json" \
      --argjson milestone "${milestone_number:-0}" \
      --arg state "$desired_state" \
      --arg state_reason "$desired_reason" \
      '{
        title:$title,
        body:$body,
        labels:$labels,
        assignees:$assignees
      }
      + (if $milestone == 0 then {} else {milestone:$milestone} end)
      + (if $state == "" then {} else {state:$state} end)
      + (if $state_reason == "" then {} else {state_reason:$state_reason} end)')"

    if [[ "$MODE" == "dry-run" ]]; then
      if [[ -n "$(jq -r '.existingNumber // empty' <<<"$issue_json")" ]]; then
        log "DRY-RUN update existing issue #$(jq -r '.existingNumber' <<<"$issue_json"): $title"
      else
        log "DRY-RUN upsert issue [$key]: $title"
      fi
      continue
    fi

    if [[ -n "$number" ]]; then
      response="$(api --method PATCH "repos/$REPO/issues/$number" --input - <<<"$payload")"
    else
      response="$(api --method POST "repos/$REPO/issues" --input - <<<"$payload")"
    fi
    record_issue "$key" "$(jq -r '.number' <<<"$response")" "$(jq -r '.id' <<<"$response")" "$(jq -r '.state' <<<"$response")"
    log "Managed #$(jq -r '.number' <<<"$response") [$key]"
    sleep_after_write
  done
}

ensure_subissue() {
  local parent_key="$1" child_key="$2"
  local parent_number child_id
  parent_number="$(issue_number_by_key "$parent_key")"
  child_id="$(issue_id_by_key "$child_key")"
  [[ -n "$parent_number" && -n "$child_id" ]] ||
    die "cannot relate $child_key under $parent_key; issue mapping is missing"

  if api "repos/$REPO/issues/$parent_number/sub_issues?per_page=100" |
      jq -e --argjson child_id "$child_id" 'map(.id) | index($child_id) != null' >/dev/null; then
    return
  fi

  jq -n --argjson sub_issue_id "$child_id" '{sub_issue_id:$sub_issue_id}' |
    api --method POST "repos/$REPO/issues/$parent_number/sub_issues" --input - >/dev/null
  sleep_after_write
}

ensure_dependency() {
  local issue_key="$1" blocker_key="$2"
  local issue_number blocker_id
  issue_number="$(issue_number_by_key "$issue_key")"
  blocker_id="$(issue_id_by_key "$blocker_key")"
  [[ -n "$issue_number" && -n "$blocker_id" ]] ||
    die "cannot add dependency $issue_key <- $blocker_key; issue mapping is missing"

  if api "repos/$REPO/issues/$issue_number/dependencies/blocked_by?per_page=100" |
      jq -e --argjson blocker_id "$blocker_id" 'map(.id) | index($blocker_id) != null' >/dev/null; then
    return
  fi

  jq -n --argjson issue_id "$blocker_id" '{issue_id:$issue_id}' |
    api --method POST "repos/$REPO/issues/$issue_number/dependencies/blocked_by" --input - >/dev/null
  sleep_after_write
}

apply_relations() {
  if [[ "$MODE" == "dry-run" ]]; then
    log "DRY-RUN relation plan"
    jq -r --arg phase "$PHASE" '
      .issues[]
      | select($phase == "all" or .phase == $phase)
      | .key as $key
      | if .parent then "sub-issue: \($key) -> \(.parent)" else empty end,
        ((.blockedBy // [])[] | "blocked-by: \($key) <- \(.)")
    ' "$ISSUES_FILE"
    return
  fi

  log "Applying sub-issue hierarchy"
  jq -c '.issues[] | select(.parent != null)' "$ISSUES_FILE" | while IFS= read -r issue_json; do
    issue_phase="$(jq -r '.phase' <<<"$issue_json")"
    if [[ "$PHASE" != "all" && "$FORCE_UPDATE" != "true" ]]; then
      phase_selected "$issue_phase" || continue
    fi
    ensure_subissue "$(jq -r '.parent' <<<"$issue_json")" "$(jq -r '.key' <<<"$issue_json")"
  done

  log "Applying issue dependencies"
  jq -c '.issues[] | select((.blockedBy // []) | length > 0)' "$ISSUES_FILE" | while IFS= read -r issue_json; do
    issue_phase="$(jq -r '.phase' <<<"$issue_json")"
    if [[ "$PHASE" != "all" && "$FORCE_UPDATE" != "true" ]]; then
      phase_selected "$issue_phase" || continue
    fi
    issue_key="$(jq -r '.key' <<<"$issue_json")"
    jq -r '.blockedBy[]' <<<"$issue_json" | while IFS= read -r blocker_key; do
      ensure_dependency "$issue_key" "$blocker_key"
    done
  done
}

validate_references() {
  log "Validating manifest references"
  jq -e '
    (.issues | map(.key)) as $keys
    | ((.issues | map(.key) | length) == (.issues | map(.key) | unique | length))
      and all(.issues[];
        . as $issue
        | (($issue.parent == null) or ($keys | index($issue.parent) != null))
          and all(($issue.blockedBy // [])[]; . as $dependency | $keys | index($dependency) != null)
          and (($issue.supersededBy == null) or ($keys | index($issue.supersededBy) != null))
      )
  ' "$ISSUES_FILE" >/dev/null || die "duplicate key or unresolved parent/dependency/superseded reference"

  jq -e \
    --slurpfile labels "$LABELS_FILE" \
    '($labels[0].labels | map(.name)) as $known
     | all(.issues[].labels[]; . as $label | $known | index($label) != null)' \
    "$ISSUES_FILE" >/dev/null || die "issues.json references an undefined label"

  jq -e \
    --slurpfile milestones "$MILESTONES_FILE" \
    '($milestones[0].milestones | map(.key)) as $known
     | all(.issues[]; . as $issue | ($issue.milestone == null) or ($known | index($issue.milestone) != null))' \
    "$ISSUES_FILE" >/dev/null || die "issues.json references an undefined milestone"
}

main() {
  log "Repository: $REPO"
  log "Mode: $MODE; phase: $PHASE; start date: $START_DATE"
  validate_references
  load_remote_state
  upsert_labels
  upsert_milestones
  upsert_issues
  apply_relations
  log "Roadmap bootstrap complete"
}

main "$@"
