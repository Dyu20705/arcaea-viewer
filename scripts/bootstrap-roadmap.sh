#!/usr/bin/env bash
set -Eeuo pipefail

API_VERSION="${GITHUB_API_VERSION:-2026-03-10}"
MODE="dry-run"
PHASE="all"
START_DATE="$(date -u +%F)"
FORCE_UPDATE="false"
CLOSE_SUPERSEDED="false"
REPO="${GITHUB_REPOSITORY:-}"
ROOT_DIR="${ROADMAP_ROOT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
LABELS_FILE="$ROOT_DIR/roadmap/labels.json"
MILESTONES_FILE="$ROOT_DIR/roadmap/milestones.json"
ISSUES_INDEX_FILE="$ROOT_DIR/roadmap/issues.index.json"
GUIDANCE_FILE="$ROOT_DIR/roadmap/issue-execution-guidance.json"
TMP_DIR="$(mktemp -d)"
ISSUES_BASE_FILE="$TMP_DIR/issues-base.json"
GUIDANCE_RESOLVED_FILE="$TMP_DIR/guidance-resolved.json"
ISSUES_FILE="$TMP_DIR/issues.json"
REMOTE_ISSUES="$TMP_DIR/remote-issues.json"
ISSUE_MAP="$TMP_DIR/issues-map.json"
LABEL_MAP="$TMP_DIR/labels-map.json"
MILESTONE_MAP="$TMP_DIR/milestones-map.json"
SLEEP_SECONDS="${ROADMAP_SLEEP_SECONDS:-1}"

cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT

usage() {
  cat <<'USAGE'
Usage: bootstrap-roadmap.sh [options]

Options:
  --dry-run                 Read GitHub and print create/update/no-op plans (default).
  --apply                   Apply changes to GitHub.
  --phase <name>            all, roadmap, week-1 ... week-6, post-mvp, cleanup.
  --start-date YYYY-MM-DD   Date used to calculate milestone due dates.
  --force-update            Reconcile managed state, parents, and dependencies exactly.
  --close-superseded        Allow issues marked closed/not_planned to be closed.
  --repo OWNER/REPO         Override repository detection.
  -h, --help                Show help.
USAGE
}

log() { printf '[roadmap] %s\n' "$*"; }
warn() { printf '[roadmap] WARNING: %s\n' "$*" >&2; }
die() { printf '[roadmap] ERROR: %s\n' "$*" >&2; exit 1; }

while (($#)); do
  case "$1" in
    --dry-run) MODE="dry-run"; shift ;;
    --apply) MODE="apply"; shift ;;
    --phase) [[ $# -ge 2 ]] || die "missing --phase value"; PHASE="$2"; shift 2 ;;
    --start-date) [[ $# -ge 2 ]] || die "missing --start-date value"; START_DATE="$2"; shift 2 ;;
    --force-update) FORCE_UPDATE="true"; shift ;;
    --close-superseded) CLOSE_SUPERSEDED="true"; shift ;;
    --repo) [[ $# -ge 2 ]] || die "missing --repo value"; REPO="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) die "unknown argument: $1" ;;
  esac
done

command -v jq >/dev/null 2>&1 || die "'jq' is required"
command -v date >/dev/null 2>&1 || die "GNU 'date' is required"
command -v gh >/dev/null 2>&1 || die "GitHub CLI 'gh' is required, including for dry-run"

[[ "$MODE" == "dry-run" || "$MODE" == "apply" ]] || die "invalid mode"
[[ "$PHASE" =~ ^(all|roadmap|week-[1-6]|post-mvp|cleanup)$ ]] || die "invalid phase: $PHASE"
[[ "$START_DATE" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] || die "--start-date must match YYYY-MM-DD"
canonical_date="$(date -u -d "$START_DATE" +%F 2>/dev/null)" || die "--start-date is not a valid calendar date"
[[ "$canonical_date" == "$START_DATE" ]] || die "--start-date must be a canonical calendar date"
[[ "$SLEEP_SECONDS" =~ ^[0-9]+([.][0-9]+)?$ ]] || die "ROADMAP_SLEEP_SECONDS must be numeric"

jq -e '.schemaVersion == 1 and (.labels | type == "array")' "$LABELS_FILE" >/dev/null || die "invalid labels manifest"
jq -e '.schemaVersion == 1 and (.milestones | type == "array")' "$MILESTONES_FILE" >/dev/null || die "invalid milestones manifest"
jq -e '.schemaVersion == 1 and (.includes | type == "array") and ((.existingNumbers // {}) | type == "object")' "$ISSUES_INDEX_FILE" >/dev/null || die "invalid issues index"

validate_existing_numbers() {
  local unknown_key duplicate_number invalid_entry conflict_key

  invalid_entry="$(jq -r '
    (.existingNumbers // {}) |
    to_entries |
    map(select((.value | type) != "number" or (.value | floor) != .value or .value < 1)) |
    .[0].key // empty
  ' "$ISSUES_INDEX_FILE")"
  [[ -z "$invalid_entry" ]] || die "existingNumbers contains invalid issue number for key: $invalid_entry"

  unknown_key="$(jq -nr --slurpfile base "$ISSUES_BASE_FILE" --slurpfile index "$ISSUES_INDEX_FILE" '
    ($base[0].issues | map(.key)) as $known |
    (($index[0].existingNumbers // {}) | keys | map(select(. as $key | ($known | index($key)) == null))) |
    .[0] // empty
  ')"
  [[ -z "$unknown_key" ]] || die "existingNumbers references unknown issue key: $unknown_key"

  duplicate_number="$(jq -r '
    [(.existingNumbers // {}) | to_entries[] | .value] |
    group_by(.) |
    map(select(length > 1) | .[0]) |
    .[0] // empty
  ' "$ISSUES_INDEX_FILE")"
  [[ -z "$duplicate_number" ]] || die "duplicate existingNumbers issue number: $duplicate_number"

  conflict_key="$(jq -nr --slurpfile base "$ISSUES_BASE_FILE" --slurpfile index "$ISSUES_INDEX_FILE" '
    ($index[0].existingNumbers // {}) as $mapped |
    [
      $base[0].issues[] |
      select(.existingNumber != null) |
      select(($mapped[.key] // .existingNumber) != .existingNumber) |
      .key
    ] |
    .[0] // empty
  ')"
  [[ -z "$conflict_key" ]] || die "existingNumbers conflicts with phase manifest for key: $conflict_key"
}

apply_existing_numbers() {
  jq -n --slurpfile base "$ISSUES_BASE_FILE" --slurpfile index "$ISSUES_INDEX_FILE" '
    ($index[0].existingNumbers // {}) as $mapped |
    $base[0] |
    .issues |= map(
      . as $issue |
      ($mapped[$issue.key] // null) as $number |
      if $number == null then . else .existingNumber = $number end
    )
  ' >"$ISSUES_BASE_FILE.mapped"
  mv "$ISSUES_BASE_FILE.mapped" "$ISSUES_BASE_FILE"
}

validate_execution_guidance() {
  [[ -f "$GUIDANCE_FILE" ]] || die "missing execution guidance manifest"
  jq -e '
    .schemaVersion == 1 and
    ((.defaults // {}) | type == "object") and
    (.issues | type == "array") and
    all(.issues[]; ((.key | type) == "string" and (.key | length) > 0))
  ' "$GUIDANCE_FILE" >/dev/null || die "invalid execution guidance manifest"

  local duplicate_key unknown_key missing_key
  duplicate_key="$(jq -r '[.issues[].key] | group_by(.) | map(select(length > 1) | .[0]) | .[0] // empty' "$GUIDANCE_FILE")"
  [[ -z "$duplicate_key" ]] || die "duplicate execution guidance key: $duplicate_key"

  unknown_key="$(jq -nr --slurpfile base "$ISSUES_BASE_FILE" --slurpfile guidance "$GUIDANCE_FILE" '
    ($base[0].issues | map(.key)) as $known |
    ($guidance[0].issues | map(.key) | map(select(. as $key | ($known | index($key)) == null))) |
    .[0] // empty
  ')"
  [[ -z "$unknown_key" ]] || die "execution guidance references unknown issue key: $unknown_key"

  missing_key="$(jq -nr --slurpfile base "$ISSUES_BASE_FILE" --slurpfile guidance "$GUIDANCE_FILE" '
    ($guidance[0].issues | map(.key)) as $provided |
    [
      $base[0].issues[] |
      select((.state // "open") == "open") |
      select(.phase != "cleanup") |
      select(((.labels // []) | index("type:epic")) == null) |
      select(.key as $key | ($provided | index($key)) == null) |
      .key
    ] |
    .[0] // empty
  ')"
  [[ -z "$missing_key" ]] || die "missing execution guidance for active issue: $missing_key"
}

resolve_execution_guidance() {
  jq -n --slurpfile guidance "$GUIDANCE_FILE" '
    def fields: [
      "researchTasks",
      "setupSteps",
      "implementationSteps",
      "uiUxTasks",
      "dataBackendTasks",
      "soloExecution",
      "validationSteps",
      "deliverables",
      "risksAndRollback"
    ];
    def resolve($defaults; $entry):
      reduce fields[] as $field
        ($entry; .[$field] = (($defaults[$field] // []) + ($entry[$field] // [])));
    ($guidance[0].defaults // {}) as $defaults |
    {
      schemaVersion: $guidance[0].schemaVersion,
      issues: [$guidance[0].issues[] | resolve($defaults; .)]
    }
  ' >"$GUIDANCE_RESOLVED_FILE"

  jq -e '
    def fields: [
      "researchTasks",
      "setupSteps",
      "implementationSteps",
      "uiUxTasks",
      "dataBackendTasks",
      "soloExecution",
      "validationSteps",
      "deliverables",
      "risksAndRollback"
    ];
    def nonempty_array($value): (($value | type) == "array" and ($value | length) > 0);
    all(.issues[]; . as $issue | all(fields[]; . as $field | nonempty_array($issue[$field])))
  ' "$GUIDANCE_RESOLVED_FILE" >/dev/null || die "invalid resolved execution guidance entry"
}

build_issues_manifest() {
  local -a files=()
  local relative_path full_path
  while IFS= read -r relative_path; do
    [[ "$relative_path" =~ ^issues/[A-Za-z0-9._-]+\.json$ ]] || die "unsafe issue include path: $relative_path"
    full_path="$ROOT_DIR/roadmap/$relative_path"
    [[ -f "$full_path" ]] || die "missing issue manifest: $relative_path"
    jq -e '.schemaVersion == 1 and (.issues | type == "array")' "$full_path" >/dev/null || die "invalid issue manifest: $relative_path"
    files+=("$full_path")
  done < <(jq -r '.includes[]' "$ISSUES_INDEX_FILE")
  ((${#files[@]} > 0)) || die "issues index contains no manifests"

  jq -s --slurpfile index "$ISSUES_INDEX_FILE" '
    {
      schemaVersion: $index[0].schemaVersion,
      repository: $index[0].repository,
      defaults: ($index[0].defaults // {}),
      issues: (map(.issues) | add)
    }
  ' "${files[@]}" >"$ISSUES_BASE_FILE"

  validate_existing_numbers
  apply_existing_numbers
  validate_execution_guidance
  resolve_execution_guidance

  jq -n --slurpfile base "$ISSUES_BASE_FILE" --slurpfile guidance "$GUIDANCE_RESOLVED_FILE" '
    ($guidance[0].issues | map({key: .key, value: (del(.key))}) | from_entries) as $by_key |
    $base[0] |
    .issues |= map(. + ($by_key[.key] // {}))
  ' >"$ISSUES_FILE"
}
build_issues_manifest

if [[ -z "$REPO" ]]; then REPO="$(gh repo view --json nameWithOwner --jq '.nameWithOwner')"; fi
[[ "$REPO" =~ ^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$ ]] || die "repository must be OWNER/REPO"
MANIFEST_REPO="$(jq -r '.repository' "$ISSUES_FILE")"
[[ "$REPO" == "$MANIFEST_REPO" ]] || die "manifest targets $MANIFEST_REPO but current repository is $REPO"

api() {
  gh api -H "Accept: application/vnd.github+json" -H "X-GitHub-Api-Version: $API_VERSION" "$@"
}

urlencode() { jq -rn --arg value "$1" '$value | @uri'; }
phase_selected() { [[ "$PHASE" == "all" || "$PHASE" == "$1" ]]; }
sleep_after_write() { [[ "$MODE" != "apply" || "$SLEEP_SECONDS" == "0" ]] || sleep "$SLEEP_SECONDS"; }

render_issue_body() {
  jq -r '
    def bullets($values): if (($values // []) | length) == 0 then "" else (($values // []) | map("- " + .) | join("\n")) + "\n" end;
    def section($title; $values): if (($values // []) | length) == 0 then "" else "## " + $title + "\n\n" + bullets($values) + "\n" end;
    "<!-- roadmap-key: " + .key + " -->\n\n" +
    "> Managed by `roadmap/issues.index.json`, `roadmap/issues/*.json`, `roadmap/issue-execution-guidance.json`, and `scripts/bootstrap-roadmap.sh`.\n\n" +
    "## Outcome\n\n" + .outcome + "\n\n" +
    section("Scope"; .scope) +
    section("Non-goals"; .nonGoals) +
    section("Research and source collection"; .researchTasks) +
    section("Environment and setup"; .setupSteps) +
    section("Implementation sequence"; .implementationSteps) +
    section("UI/UX responsibilities"; .uiUxTasks) +
    section("Data/backend responsibilities"; .dataBackendTasks) +
    section("Solo execution order"; .soloExecution) +
    section("UX requirements"; .uxRequirements) +
    section("Technical constraints"; .technicalConstraints) +
    section("Acceptance criteria"; .acceptanceCriteria) +
    section("Test plan"; .testPlan) +
    section("Validation steps"; .validationSteps) +
    section("Required deliverables"; .deliverables) +
    section("Required evidence"; .evidence) +
    section("Risks, rollback, and scope cuts"; .risksAndRollback) +
    section("Definition of Done"; .definitionOfDone) +
    (if .supersededBy then "## Superseded by\n\n- Roadmap key: `" + .supersededBy + "`\n\n" else "" end) +
    "## Planning metadata\n\n" +
    "- Roadmap key: `" + .key + "`\n" +
    "- Phase: `" + .phase + "`\n" +
    "- Milestone key: `" + (.milestone // "none") + "`\n" +
    (if ((.blockedBy // []) | length) > 0 then "- Blocked by roadmap keys: " + ((.blockedBy // []) | map("`" + . + "`") | join(", ")) + "\n" else "" end)
  ' <<<"$1"
}

validate_references() {
  log "Validating manifest references"
  jq -e '
    (.issues | map(.key)) as $keys |
    ((.issues | map(.key) | length) == (.issues | map(.key) | unique | length)) and
    all(.issues[]; . as $issue |
      (($issue.parent == null) or ($keys | index($issue.parent) != null)) and
      all(($issue.blockedBy // [])[]; . as $dependency | $keys | index($dependency) != null) and
      (($issue.supersededBy == null) or ($keys | index($issue.supersededBy) != null)))
  ' "$ISSUES_FILE" >/dev/null || die "duplicate key or unresolved parent/dependency/superseded reference"
  jq -e --slurpfile labels "$LABELS_FILE" '($labels[0].labels | map(.name)) as $known | all(.issues[].labels[]; . as $label | $known | index($label) != null)' "$ISSUES_FILE" >/dev/null || die "issue references undefined label"
  jq -e --slurpfile milestones "$MILESTONES_FILE" '($milestones[0].milestones | map(.key)) as $known | all(.issues[]; . as $issue | ($issue.milestone == null) or ($known | index($issue.milestone) != null))' "$ISSUES_FILE" >/dev/null || die "issue references undefined milestone"
  jq -e '
    [ .issues[] | select(.existingNumber != null) | .existingNumber ] as $numbers |
    ($numbers | length) == ($numbers | unique | length)
  ' "$ISSUES_FILE" >/dev/null || die "duplicate existingNumber across merged issue manifests"
}

load_remote_state() {
  log "Loading current labels, milestones, and issues from GitHub ($MODE performs reads)"
  api --paginate "repos/$REPO/labels?per_page=100" | jq -s 'add // []' >"$TMP_DIR/remote-labels.json"
  jq 'reduce .[] as $label ({}; .[$label.name] = {name: $label.name, color: ($label.color | ascii_downcase), description: ($label.description // "")})' "$TMP_DIR/remote-labels.json" >"$LABEL_MAP"

  api --paginate "repos/$REPO/milestones?state=all&per_page=100" | jq -s 'add // []' >"$TMP_DIR/remote-milestones.json"
  jq 'reduce .[] as $milestone ({}; .[$milestone.title] = {number: $milestone.number, state: $milestone.state, title: $milestone.title, description: ($milestone.description // ""), due_on: ($milestone.due_on // null)})' "$TMP_DIR/remote-milestones.json" >"$MILESTONE_MAP"

  api --paginate "repos/$REPO/issues?state=all&per_page=100" | jq -s 'add // [] | map(select(has("pull_request") | not))' >"$REMOTE_ISSUES"
  duplicate_markers="$(jq -r '[.[] | ((.body // "") | (try capture("roadmap-key: (?<key>[A-Za-z0-9._-]+)").key catch null)) as $key | select($key != null) | {key: $key, number: .number}] | group_by(.key) | map(select(length > 1)) | .[] | "\(.[0].key): " + (map("#\(.number)") | join(", "))' "$REMOTE_ISSUES")"
  [[ -z "$duplicate_markers" ]] || die "duplicate roadmap-key markers detected: $duplicate_markers"
  jq 'reduce .[] as $issue ({};
      (($issue.body // "") | (try capture("roadmap-key: (?<key>[A-Za-z0-9._-]+)").key catch null)) as $key |
      if $key then .[$key] = {number: $issue.number, id: $issue.id, state: $issue.state} else . end)' "$REMOTE_ISSUES" >"$ISSUE_MAP"
}

remote_issue_by_number() { jq -c --argjson number "$1" '.[] | select(.number == $number)' "$REMOTE_ISSUES" | head -n1; }
issue_number_by_key() { jq -r --arg key "$1" '.[$key].number // empty' "$ISSUE_MAP"; }
issue_id_by_key() { jq -r --arg key "$1" '.[$key].id // empty' "$ISSUE_MAP"; }
record_issue() {
  local key="$1" number="$2" id="$3" state="$4"
  jq --arg key "$key" --argjson number "$number" --argjson id "$id" --arg state "$state" '.[$key] = {number: $number, id: $id, state: $state}' "$ISSUE_MAP" >"$ISSUE_MAP.tmp" && mv "$ISSUE_MAP.tmp" "$ISSUE_MAP"
}

resolve_existing_issue() {
  local issue_json="$1" key existing marker_number existing_json existing_marker
  key="$(jq -r '.key' <<<"$issue_json")"
  existing="$(jq -r '.existingNumber // empty' <<<"$issue_json")"
  marker_number="$(issue_number_by_key "$key")"
  if [[ -n "$existing" ]]; then
    existing_json="$(remote_issue_by_number "$existing")"
    [[ -n "$existing_json" ]] || die "manifest maps [$key] to missing issue #$existing"
    existing_marker="$(jq -r '((.body // "") | (try capture("roadmap-key: (?<key>[A-Za-z0-9._-]+)").key catch ""))' <<<"$existing_json")"
    [[ -z "$existing_marker" || "$existing_marker" == "$key" ]] || die "issue #$existing is already managed by roadmap-key '$existing_marker', not '$key'"
    [[ -z "$marker_number" || "$marker_number" == "$existing" ]] || die "roadmap-key '$key' points to #$marker_number but existingNumber maps to #$existing"
    record_issue "$key" "$existing" "$(jq -r '.id' <<<"$existing_json")" "$(jq -r '.state' <<<"$existing_json")"
    printf '%s' "$existing"
  else
    printf '%s' "$marker_number"
  fi
}

milestone_title_for_key() { jq -r --arg key "$1" '.milestones[] | select(.key == $key) | .title' "$MILESTONES_FILE"; }
milestone_number_for_key() {
  local key="$1" title
  [[ -n "$key" && "$key" != "null" ]] || return 0
  title="$(milestone_title_for_key "$key")"; [[ -n "$title" ]] || die "unknown milestone key: $key"
  jq -r --arg title "$title" '.[$title].number // empty' "$MILESTONE_MAP"
}

upsert_labels() {
  log "Planning labels"
  local label name color description current payload encoded action
  while IFS= read -r label; do
    name="$(jq -r '.name' <<<"$label")"; color="$(jq -r '.color | ascii_downcase' <<<"$label")"; description="$(jq -r '.description // ""' <<<"$label")"
    current="$(jq -c --arg name "$name" '.[$name] // null' "$LABEL_MAP")"
    action="create"
    if [[ "$current" != "null" ]]; then
      if jq -e --arg color "$color" --arg description "$description" '.color == $color and .description == $description' <<<"$current" >/dev/null; then action="no-op"; else action="update"; fi
    fi
    log "$MODE $action label: $name"
    [[ "$MODE" == "apply" && "$action" != "no-op" ]] || continue
    payload="$(jq -n --arg name "$name" --arg color "$color" --arg description "$description" '{name: $name, color: $color, description: $description}')"
    if [[ "$action" == "create" ]]; then api --method POST "repos/$REPO/labels" --input - <<<"$payload" >/dev/null; else encoded="$(urlencode "$name")"; api --method PATCH "repos/$REPO/labels/$encoded" --input - <<<"$payload" >/dev/null; fi
    sleep_after_write
  done < <(jq -c '.labels[]' "$LABELS_FILE")
}

upsert_milestones() {
  log "Planning milestones from start date $START_DATE"
  local milestone key title description offset due_on current number action payload response
  while IFS= read -r milestone; do
    key="$(jq -r '.key' <<<"$milestone")"; title="$(jq -r '.title' <<<"$milestone")"; description="$(jq -r '.description // ""' <<<"$milestone")"; offset="$(jq -r '.dueOffsetDays // empty' <<<"$milestone")"
    due_on=""; [[ -z "$offset" ]] || due_on="$(date -u -d "$START_DATE + $offset days" +%Y-%m-%dT23:59:59Z)"
    current="$(jq -c --arg title "$title" '.[$title] // null' "$MILESTONE_MAP")"; action="create"
    if [[ "$current" != "null" ]]; then
      if jq -e --arg description "$description" --arg due "$due_on" '.description == $description and .state == "open" and ((.due_on // "") == $due)' <<<"$current" >/dev/null; then action="no-op"; else action="update"; fi
    fi
    log "$MODE $action milestone: $key -> $title${due_on:+ due $due_on}"
    [[ "$MODE" == "apply" && "$action" != "no-op" ]] || continue
    payload="$(jq -n --arg title "$title" --arg description "$description" --arg due "$due_on" '{title: $title, description: $description, state: "open"} + (if $due == "" then {} else {due_on: $due} end)')"
    if [[ "$action" == "create" ]]; then response="$(api --method POST "repos/$REPO/milestones" --input - <<<"$payload")"; number="$(jq -r '.number' <<<"$response")"; else number="$(jq -r '.number' <<<"$current")"; response="$(api --method PATCH "repos/$REPO/milestones/$number" --input - <<<"$payload")"; fi
    jq --arg title "$title" --argjson number "$number" --arg description "$description" --arg due "$due_on" '.[$title] = {number: $number, state: "open", title: $title, description: $description, due_on: (if $due == "" then null else $due end)}' "$MILESTONE_MAP" >"$MILESTONE_MAP.tmp" && mv "$MILESTONE_MAP.tmp" "$MILESTONE_MAP"
    sleep_after_write
  done < <(jq -c '.milestones[]' "$MILESTONES_FILE")
}

issue_desired_model() {
  local issue_json="$1" body="$2" milestone_title="$3" target_state state_reason
  target_state="$(jq -r '.state // "open"' <<<"$issue_json")"; state_reason="$(jq -r '.stateReason // empty' <<<"$issue_json")"
  jq -n --arg title "$(jq -r '.title' <<<"$issue_json")" --arg body "$body" \
    --argjson labels "$(jq -c '.labels // [] | sort' <<<"$issue_json")" \
    --argjson assignees "$(jq -c '.assignees // [] | sort' <<<"$issue_json")" \
    --arg milestone "$milestone_title" --arg target_state "$target_state" --arg reason "$state_reason" \
    --arg force "$FORCE_UPDATE" --arg close "$CLOSE_SUPERSEDED" \
    '{title: $title, body: $body, labels: $labels, assignees: $assignees, milestone: $milestone} +
     (if $target_state == "closed" and $close == "true" then {state: "closed", state_reason: (if $reason == "" then "not_planned" else $reason end)}
      elif $target_state != "closed" and $force == "true" then {state: "open"} else {} end)'
}

issue_current_model() {
  jq '{title: .title, body: (.body // ""), labels: (.labels | map(.name) | sort), assignees: (.assignees | map(.login) | sort), milestone: (.milestone.title // ""), state: .state, state_reason: (.state_reason // "")}' <<<"$1"
}

issue_needs_update() {
  jq -e --argjson desired "$2" '
    . as $current | ($desired | to_entries) | any(.[]; .key as $key | ($current[$key] // "") != .value)
  ' <<<"$1" >/dev/null
}

upsert_issues() {
  log "Planning managed issues for phase '$PHASE'"
  local issue_json issue_phase key title target_state milestone_key milestone_title milestone_number body number current current_model desired action payload response
  while IFS= read -r issue_json; do
    issue_phase="$(jq -r '.phase' <<<"$issue_json")"; phase_selected "$issue_phase" || continue
    key="$(jq -r '.key' <<<"$issue_json")"; title="$(jq -r '.title' <<<"$issue_json")"; target_state="$(jq -r '.state // "open"' <<<"$issue_json")"
    if [[ "$target_state" == "closed" && "$CLOSE_SUPERSEDED" != "true" ]]; then resolve_existing_issue "$issue_json" >/dev/null || true; log "skip superseded [$key]; use --close-superseded"; continue; fi
    milestone_key="$(jq -r '.milestone // empty' <<<"$issue_json")"; milestone_title=""; milestone_number=""
    if [[ -n "$milestone_key" ]]; then milestone_title="$(milestone_title_for_key "$milestone_key")"; milestone_number="$(milestone_number_for_key "$milestone_key")"; fi
    body="$(render_issue_body "$issue_json")"; number="$(resolve_existing_issue "$issue_json")"; desired="$(issue_desired_model "$issue_json" "$body" "$milestone_title")"
    action="create"; current=""
    if [[ -n "$number" ]]; then
      current="$(remote_issue_by_number "$number")"; [[ -n "$current" ]] || die "mapped issue #$number for [$key] is absent"
      current_model="$(issue_current_model "$current")"
      if issue_needs_update "$current_model" "$desired"; then action="update"; else action="no-op"; fi
    fi
    log "$MODE $action issue${number:+ #$number} [$key]: $title"
    [[ "$MODE" == "apply" && "$action" != "no-op" ]] || continue
    payload="$(jq -n --argjson desired "$desired" --argjson milestone "${milestone_number:-0}" '$desired | del(.milestone) + (if $milestone == 0 then {} else {milestone: $milestone} end)')"
    if [[ "$action" == "create" ]]; then response="$(api --method POST "repos/$REPO/issues" --input - <<<"$payload")"; else response="$(api --method PATCH "repos/$REPO/issues/$number" --input - <<<"$payload")"; fi
    record_issue "$key" "$(jq -r '.number' <<<"$response")" "$(jq -r '.id' <<<"$response")" "$(jq -r '.state' <<<"$response")"
    jq --argjson response "$response" --argjson number "$(jq -r '.number' <<<"$response")" 'map(select(.number != $number)) + [$response]' "$REMOTE_ISSUES" >"$REMOTE_ISSUES.tmp" && mv "$REMOTE_ISSUES.tmp" "$REMOTE_ISSUES"
    sleep_after_write
  done < <(jq -c '.issues[]' "$ISSUES_FILE")
}

get_parent_json() {
  local child_number="$1" response
  if response="$(api "repos/$REPO/issues/$child_number/parent" 2>/dev/null)"; then printf '%s' "$response"; else printf 'null'; fi
}

get_blockers_json() { api --paginate "repos/$REPO/issues/$1/dependencies/blocked_by?per_page=100" | jq -s 'add // []'; }

reconcile_parent() {
  local child_key="$1" desired_parent_key="$2" child_number child_id desired_parent_number desired_parent_id current current_number current_id
  child_number="$(issue_number_by_key "$child_key")"; child_id="$(issue_id_by_key "$child_key")"
  if [[ -z "$child_number" ]]; then log "$MODE deferred parent [$child_key]: issue will be created first"; return; fi
  desired_parent_number=""; desired_parent_id=""
  if [[ -n "$desired_parent_key" && "$desired_parent_key" != "null" ]]; then desired_parent_number="$(issue_number_by_key "$desired_parent_key")"; desired_parent_id="$(issue_id_by_key "$desired_parent_key")"; [[ -n "$desired_parent_number" ]] || { log "$MODE deferred parent [$child_key] -> [$desired_parent_key]: parent is not created"; return; }; fi
  current="$(get_parent_json "$child_number")"; current_number="$(jq -r 'if type == "object" then (.number // "") else "" end' <<<"$current")"; current_id="$(jq -r 'if type == "object" then (.id // "") else "" end' <<<"$current")"
  if [[ "$current_number" == "$desired_parent_number" ]]; then log "$MODE no-op parent: [$child_key]${desired_parent_key:+ -> [$desired_parent_key]}"; return; fi
  if [[ -n "$current_number" && "$FORCE_UPDATE" != "true" ]]; then warn "parent drift for [$child_key]: current #$current_number, desired ${desired_parent_number:-none}; rerun with --force-update"; return; fi
  if [[ -n "$current_number" ]]; then
    log "$MODE remove parent #$current_number from [$child_key]"
    if [[ "$MODE" == "apply" ]]; then jq -n --argjson sub_issue_id "$child_id" '{sub_issue_id: $sub_issue_id}' | api --method DELETE "repos/$REPO/issues/$current_number/sub_issue" --input - >/dev/null; sleep_after_write; fi
  fi
  if [[ -n "$desired_parent_number" ]]; then
    log "$MODE add parent [$desired_parent_key] to [$child_key]"
    if [[ "$MODE" == "apply" ]]; then jq -n --argjson sub_issue_id "$child_id" '{sub_issue_id: $sub_issue_id}' | api --method POST "repos/$REPO/issues/$desired_parent_number/sub_issues" --input - >/dev/null; sleep_after_write; fi
  fi
  : "$desired_parent_id" "$current_id"
}

reconcile_dependencies() {
  local issue_json="$1" key number blockers desired_ids current_ids missing extra blocker_key blocker_id blocker_number
  key="$(jq -r '.key' <<<"$issue_json")"; number="$(issue_number_by_key "$key")"
  if [[ -z "$number" ]]; then log "$MODE deferred dependencies [$key]: issue will be created first"; return; fi
  desired_ids='[]'
  while IFS= read -r blocker_key; do
    [[ -n "$blocker_key" ]] || continue
    blocker_id="$(issue_id_by_key "$blocker_key")"
    if [[ -z "$blocker_id" ]]; then log "$MODE deferred dependency [$key] <- [$blocker_key]: blocker is not created"; continue; fi
    desired_ids="$(jq --argjson id "$blocker_id" '. + [$id] | unique' <<<"$desired_ids")"
  done < <(jq -r '.blockedBy // [] | .[]' <<<"$issue_json")
  blockers="$(get_blockers_json "$number")"; current_ids="$(jq 'map(.id) | unique' <<<"$blockers")"
  missing="$(jq -n --argjson desired "$desired_ids" --argjson current "$current_ids" '$desired - $current')"; extra="$(jq -n --argjson desired "$desired_ids" --argjson current "$current_ids" '$current - $desired')"
  if [[ "$(jq 'length' <<<"$missing")" == "0" && "$(jq 'length' <<<"$extra")" == "0" ]]; then log "$MODE no-op dependencies: [$key]"; return; fi
  if [[ "$(jq 'length' <<<"$extra")" != "0" && "$FORCE_UPDATE" != "true" ]]; then warn "dependency drift for [$key]: extra blocker ids $(jq -c . <<<"$extra"); rerun with --force-update"; extra='[]'; fi
  while IFS= read -r blocker_id; do
    [[ -n "$blocker_id" ]] || continue
    log "$MODE remove dependency [$key] <- issue-id $blocker_id"
    if [[ "$MODE" == "apply" ]]; then api --method DELETE "repos/$REPO/issues/$number/dependencies/blocked_by/$blocker_id" >/dev/null; sleep_after_write; fi
  done < <(jq -r '.[]' <<<"$extra")
  while IFS= read -r blocker_id; do
    [[ -n "$blocker_id" ]] || continue
    blocker_number="$(jq -r --argjson id "$blocker_id" '.[] | select(.id == $id) | .number' "$REMOTE_ISSUES" | head -n1)"
    log "$MODE add dependency [$key] <- issue-id $blocker_id${blocker_number:+ (#$blocker_number)}"
    if [[ "$MODE" == "apply" ]]; then jq -n --argjson issue_id "$blocker_id" '{issue_id: $issue_id}' | api --method POST "repos/$REPO/issues/$number/dependencies/blocked_by" --input - >/dev/null; sleep_after_write; fi
  done < <(jq -r '.[]' <<<"$missing")
}

apply_relations() {
  log "Planning/reconciling parent and dependency relations"
  local issue_json issue_phase target_state parent
  while IFS= read -r issue_json; do
    issue_phase="$(jq -r '.phase' <<<"$issue_json")"
    if [[ "$PHASE" != "all" && "$FORCE_UPDATE" != "true" ]]; then phase_selected "$issue_phase" || continue; fi
    target_state="$(jq -r '.state // "open"' <<<"$issue_json")"; [[ "$target_state" != "closed" || "$CLOSE_SUPERSEDED" == "true" ]] || continue
    parent="$(jq -r '.parent // empty' <<<"$issue_json")"
    reconcile_parent "$(jq -r '.key' <<<"$issue_json")" "$parent"
    reconcile_dependencies "$issue_json"
  done < <(jq -c '.issues[]' "$ISSUES_FILE")
}

main() {
  log "Repository: $REPO"
  log "Mode: $MODE; phase: $PHASE; start date: $START_DATE; force-update: $FORCE_UPDATE"
  validate_references
  load_remote_state
  while IFS= read -r issue_json; do resolve_existing_issue "$issue_json" >/dev/null || true; done < <(jq -c '.issues[]' "$ISSUES_FILE")
  upsert_labels
  upsert_milestones
  upsert_issues
  apply_relations
  log "Roadmap bootstrap complete"
}
main "$@"
