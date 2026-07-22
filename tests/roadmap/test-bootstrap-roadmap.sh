#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_UNDER_TEST="${SCRIPT_UNDER_TEST:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/bootstrap-roadmap.sh}"
TEST_TMP="$(mktemp -d)"
trap 'rm -rf "$TEST_TMP"' EXIT
ROOT="$TEST_TMP/root"
MOCK_BIN="$TEST_TMP/bin"
MOCK_LOG="$TEST_TMP/gh.log"
mkdir -p "$ROOT/roadmap/issues" "$MOCK_BIN"

cat >"$ROOT/roadmap/labels.json" <<'JSON'
{"schemaVersion":1,"labels":[{"name":"type:test","color":"abcdef","description":"test label"}]}
JSON
cat >"$ROOT/roadmap/milestones.json" <<'JSON'
{"schemaVersion":1,"milestones":[{"key":"m1","title":"M1","description":"first milestone","dueOffsetDays":0}]}
JSON
cat >"$ROOT/roadmap/issues.index.json" <<'JSON'
{"schemaVersion":1,"repository":"owner/repo","includes":["issues/test.json"]}
JSON
cat >"$ROOT/roadmap/issues/test.json" <<'JSON'
{"schemaVersion":1,"issues":[
  {"key":"root","existingNumber":1,"title":"Root","phase":"roadmap","milestone":"m1","labels":["type:test"],"outcome":"Root outcome.","scope":[],"nonGoals":[],"uxRequirements":[],"technicalConstraints":[],"acceptanceCriteria":[],"testPlan":[],"evidence":[],"definitionOfDone":[],"parent":null,"blockedBy":[],"assignees":["tester"],"state":"open"},
  {"key":"child","existingNumber":2,"title":"Child new","phase":"roadmap","milestone":"m1","labels":["type:test"],"outcome":"Child outcome.","scope":[],"nonGoals":[],"uxRequirements":[],"technicalConstraints":[],"acceptanceCriteria":[],"testPlan":[],"evidence":[],"definitionOfDone":[],"parent":"root","blockedBy":["root"],"assignees":["tester"],"state":"open"},
  {"key":"new","title":"New issue","phase":"roadmap","milestone":"m1","labels":["type:test"],"outcome":"New outcome.","scope":[],"nonGoals":[],"uxRequirements":[],"technicalConstraints":[],"acceptanceCriteria":[],"testPlan":[],"evidence":[],"definitionOfDone":[],"parent":"root","blockedBy":[],"assignees":["tester"],"state":"open"}
]}
JSON

write_guidance_fixture() {
  cat >"$ROOT/roadmap/issue-execution-guidance.json" <<'JSON'
{"schemaVersion":1,"issues":[
  {"key":"root","researchTasks":["Inspect the canonical roadmap."],"setupSteps":["Install Bash, jq, GNU date, and gh."],"implementationSteps":["Reconcile the managed root issue."],"uiUxTasks":["Confirm the issue is readable."],"dataBackendTasks":["Keep the manifest as the canonical data source."],"soloExecution":["Research, implement, verify, and record evidence in order."],"validationSteps":["Run the roadmap test harness."],"deliverables":["Updated managed issue body."],"risksAndRollback":["Use dry-run before apply and revert the manifest on unexpected drift."]},
  {"key":"child","researchTasks":["Inspect the parent and blocker."],"setupSteps":["Prepare the roadmap test fixture."],"implementationSteps":["Reconcile title, body, parent, and blocker."],"uiUxTasks":["Keep instructions scannable."],"dataBackendTasks":["Preserve stable roadmap keys."],"soloExecution":["Finish the blocker before the child."],"validationSteps":["Verify exact relations."],"deliverables":["Updated child issue."],"risksAndRollback":["Restore the prior relation mapping if reconciliation is wrong."]},
  {"key":"new","researchTasks":["Confirm the issue is not a duplicate."],"setupSteps":["Prepare labels and milestone."],"implementationSteps":["Create the managed issue."],"uiUxTasks":["Use a concise title and structured body."],"dataBackendTasks":["Record the generated issue number."],"soloExecution":["Create only after the parent exists."],"validationSteps":["Verify the created issue and parent."],"deliverables":["New managed issue."],"risksAndRollback":["Close the accidental duplicate and fix the manifest mapping."]}
]}
JSON
}
write_guidance_fixture

ROOT_BODY='<!-- roadmap-key: root -->

> Managed by `roadmap/issues.index.json`, `roadmap/issues/*.json`, `roadmap/issue-execution-guidance.json`, and `scripts/bootstrap-roadmap.sh`.

## Outcome

Root outcome.

## Research and source collection

- Inspect the canonical roadmap.

## Environment and setup

- Install Bash, jq, GNU date, and gh.

## Implementation sequence

- Reconcile the managed root issue.

## UI/UX responsibilities

- Confirm the issue is readable.

## Data/backend responsibilities

- Keep the manifest as the canonical data source.

## Solo execution order

- Research, implement, verify, and record evidence in order.

## Validation steps

- Run the roadmap test harness.

## Required deliverables

- Updated managed issue body.

## Risks, rollback, and scope cuts

- Use dry-run before apply and revert the manifest on unexpected drift.

## Planning metadata

- Roadmap key: `root`
- Phase: `roadmap`
- Milestone key: `m1`'
export ROOT_BODY MOCK_LOG

cat >"$MOCK_BIN/gh" <<'MOCK'
#!/usr/bin/env bash
set -Eeuo pipefail
printf '%q ' "$@" >>"$MOCK_LOG"; printf '\n' >>"$MOCK_LOG"
[[ "${1:-}" == "repo" ]] && { printf '{"nameWithOwner":"owner/repo"}\n'; exit 0; }
[[ "${1:-}" == "api" ]] || exit 2
shift
method="GET"
endpoint=""
input=""
while (($#)); do
  case "$1" in
    --method) method="$2"; shift 2 ;;
    --input) input="$(cat)"; shift 2 ;;
    -H) shift 2 ;;
    --paginate) shift ;;
    *) endpoint="$1"; shift ;;
  esac
done
[[ -z "$input" ]] || printf 'stdin=%s\n' "$input" >>"$MOCK_LOG"

if [[ "$method" != "GET" ]]; then
  case "$endpoint" in
    repos/owner/repo/issues/1) printf '{"number":1,"id":101,"state":"open","title":"Root","body":"x","labels":[{"name":"type:test"}],"assignees":[{"login":"tester"}],"milestone":null}\n' ;;
    repos/owner/repo/issues/2) printf '{"number":2,"id":102,"state":"open","title":"Child new","body":"x","labels":[],"assignees":[],"milestone":null}\n' ;;
    repos/owner/repo/issues) printf '{"number":3,"id":103,"state":"open","title":"New issue","body":"x","labels":[],"assignees":[],"milestone":null}\n' ;;
    *) printf '{}\n' ;;
  esac
  exit 0
fi

case "$endpoint" in
  repos/owner/repo/labels\?*) printf '[{"name":"type:test","color":"abcdef","description":"test label"}]\n' ;;
  repos/owner/repo/milestones\?*) printf '[{"number":1,"state":"open","title":"M1","description":"first milestone","due_on":"2026-07-14T00:00:00Z"}]\n' ;;
  repos/owner/repo/issues\?*)
    if [[ "${MOCK_SCENARIO:-normal}" == "duplicate" ]]; then
      jq -n --arg body "$ROOT_BODY" '[
        {number:1,id:101,state:"open",state_reason:null,title:"Root",body:$body,labels:[{name:"type:test"}],assignees:[{login:"tester"}],milestone:{number:1,title:"M1"}},
        {number:4,id:104,state:"open",state_reason:null,title:"Duplicate",body:$body,labels:[],assignees:[],milestone:null}
      ]'
    else
      jq -n --arg body "$ROOT_BODY" '[
        {number:1,id:101,state:"open",state_reason:null,title:"Root",body:$body,labels:[{name:"type:test"}],assignees:[{login:"tester"}],milestone:{number:1,title:"M1"}},
        {number:2,id:102,state:"open",state_reason:null,title:"Child old",body:"legacy",labels:[{name:"type:test"}],assignees:[{login:"tester"}],milestone:{number:1,title:"M1"}},
        {number:9,id:999,state:"open",state_reason:null,title:"Old parent",body:"",labels:[],assignees:[],milestone:null},
        {number:8,id:900,state:"open",state_reason:null,title:"Old blocker",body:"",labels:[],assignees:[],milestone:null}
      ]'
    fi
    ;;
  repos/owner/repo/issues/1/parent|repos/owner/repo/issues/3/parent) exit 1 ;;
  repos/owner/repo/issues/2/parent) printf '{"number":9,"id":999,"title":"Old parent"}\n' ;;
  repos/owner/repo/issues/1/dependencies/blocked_by\?*) printf '[]\n' ;;
  repos/owner/repo/issues/2/dependencies/blocked_by\?*) printf '[{"number":8,"id":900,"title":"Old blocker"}]\n' ;;
  repos/owner/repo/issues/3/dependencies/blocked_by\?*) printf '[]\n' ;;
  *) printf '{}\n' ;;
esac
MOCK
chmod +x "$MOCK_BIN/gh"

run_script() {
  PATH="$MOCK_BIN:$PATH" ROADMAP_ROOT_DIR="$ROOT" ROADMAP_SLEEP_SECONDS=0 \
    bash "$SCRIPT_UNDER_TEST" --repo owner/repo --start-date 2026-07-14 "$@"
}

fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
assert_contains() {
  grep -Fq -- "$2" <<<"$1" || {
    printf '%s\n%s\n%s\n' '--- output ---' "$1" '--- end ---' >&2
    fail "expected output to contain: $2"
  }
}

printf 'test: rejects injected start_date\n'
marker="$TEST_TMP/injected"
if PATH="$MOCK_BIN:$PATH" ROADMAP_ROOT_DIR="$ROOT" bash "$SCRIPT_UNDER_TEST" --dry-run --repo owner/repo --start-date "2026-07-14;touch $marker" >/dev/null 2>&1; then
  fail "malicious start_date was accepted"
fi
[[ ! -e "$marker" ]] || fail "injection marker was created"

printf 'test: execution guidance renders and preserves no-op classification\n'
: >"$MOCK_LOG"
out="$(run_script --dry-run --phase roadmap 2>&1)"
assert_contains "$out" "dry-run no-op issue #1 [root]"
assert_contains "$out" "dry-run no-op milestone: m1 -> M1 due 2026-07-14T00:00:00Z"
assert_contains "$out" "dry-run update issue #2 [child]"
assert_contains "$out" "dry-run create issue [new]"
assert_contains "$out" "parent drift for [child]"
assert_contains "$(cat "$MOCK_LOG")" "repos/owner/repo/issues\?state=all\&per_page=100"

printf 'test: unknown execution guidance key fails closed\n'
jq '.issues += [{"key":"ghost","researchTasks":["x"],"setupSteps":["x"],"implementationSteps":["x"],"uiUxTasks":["x"],"dataBackendTasks":["x"],"soloExecution":["x"],"validationSteps":["x"],"deliverables":["x"],"risksAndRollback":["x"]}]' \
  "$ROOT/roadmap/issue-execution-guidance.json" >"$ROOT/roadmap/issue-execution-guidance.json.tmp"
mv "$ROOT/roadmap/issue-execution-guidance.json.tmp" "$ROOT/roadmap/issue-execution-guidance.json"
if run_script --dry-run --phase roadmap >"$TEST_TMP/unknown-guidance.out" 2>&1; then
  fail "unknown guidance key was accepted"
fi
assert_contains "$(cat "$TEST_TMP/unknown-guidance.out")" "execution guidance references unknown issue key"
write_guidance_fixture

printf 'test: missing active issue execution guidance fails closed\n'
jq '.issues |= map(select(.key != "child"))' \
  "$ROOT/roadmap/issue-execution-guidance.json" >"$ROOT/roadmap/issue-execution-guidance.json.tmp"
mv "$ROOT/roadmap/issue-execution-guidance.json.tmp" "$ROOT/roadmap/issue-execution-guidance.json"
if run_script --dry-run --phase roadmap >"$TEST_TMP/missing-guidance.out" 2>&1; then
  fail "missing active issue guidance was accepted"
fi
assert_contains "$(cat "$TEST_TMP/missing-guidance.out")" "missing execution guidance for active issue: child"
write_guidance_fixture

printf 'test: duplicate roadmap markers fail closed\n'
if MOCK_SCENARIO=duplicate run_script --dry-run --phase roadmap >"$TEST_TMP/duplicate.out" 2>&1; then
  fail "duplicate markers were accepted"
fi
assert_contains "$(cat "$TEST_TMP/duplicate.out")" "duplicate roadmap-key markers"

printf 'test: force-update reconciles changed parent and dependencies\n'
: >"$MOCK_LOG"
out="$(run_script --apply --phase roadmap --force-update 2>&1)"
assert_contains "$out" "apply remove parent #9 from [child]"
assert_contains "$out" "apply add parent [root] to [child]"
assert_contains "$out" "apply remove dependency [child] <- issue-id 900"
assert_contains "$out" "apply add dependency [child] <- issue-id 101"
log="$(cat "$MOCK_LOG")"
assert_contains "$log" "--method DELETE repos/owner/repo/issues/9/sub_issue"
assert_contains "$log" "--method POST repos/owner/repo/issues/1/sub_issues"
assert_contains "$log" "--method DELETE repos/owner/repo/issues/2/dependencies/blocked_by/900"
assert_contains "$log" "--method POST repos/owner/repo/issues/2/dependencies/blocked_by"

printf 'test: apply clears an existing issue milestone when manifest omits one\n'
jq '.issues |= map(select(.key == "root")) | .issues[0].milestone = null' \
  "$ROOT/roadmap/issues/test.json" >"$ROOT/roadmap/issues/test.json.tmp"
mv "$ROOT/roadmap/issues/test.json.tmp" "$ROOT/roadmap/issues/test.json"
jq '.issues |= map(select(.key == "root"))' \
  "$ROOT/roadmap/issue-execution-guidance.json" >"$ROOT/roadmap/issue-execution-guidance.json.tmp"
mv "$ROOT/roadmap/issue-execution-guidance.json.tmp" "$ROOT/roadmap/issue-execution-guidance.json"
: >"$MOCK_LOG"
out="$(run_script --apply --phase roadmap 2>&1)"
assert_contains "$out" "apply update issue #1 [root]"
assert_contains "$(cat "$MOCK_LOG")" '"milestone":null'

printf 'All roadmap automation tests passed.\n'
