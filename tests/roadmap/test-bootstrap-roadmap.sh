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

ROOT_BODY='<!-- roadmap-key: root -->

> Managed by `roadmap/issues.index.json`, `roadmap/issues/*.json`, and `scripts/bootstrap-roadmap.sh`.

## Outcome

Root outcome.

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
while (($#)); do
  case "$1" in
    --method) method="$2"; shift 2 ;;
    --input) cat >/dev/null; shift 2 ;;
    -H) shift 2 ;;
    --paginate) shift ;;
    *) endpoint="$1"; shift ;;
  esac
done

if [[ "$method" != "GET" ]]; then
  case "$endpoint" in
    repos/owner/repo/issues/2) printf '{"number":2,"id":102,"state":"open","title":"Child new","body":"x","labels":[],"assignees":[],"milestone":null}\n' ;;
    repos/owner/repo/issues) printf '{"number":3,"id":103,"state":"open","title":"New issue","body":"x","labels":[],"assignees":[],"milestone":null}\n' ;;
    *) printf '{}\n' ;;
  esac
  exit 0
fi

case "$endpoint" in
  repos/owner/repo/labels\?*) printf '[{"name":"type:test","color":"abcdef","description":"test label"}]\n' ;;
  repos/owner/repo/milestones\?*) printf '[{"number":1,"state":"open","title":"M1","description":"first milestone","due_on":"2026-07-14T23:59:59Z"}]\n' ;;
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

printf 'test: dry-run reads GitHub and classifies no-op/update/create\n'
: >"$MOCK_LOG"
out="$(run_script --dry-run --phase roadmap 2>&1)"
assert_contains "$out" "dry-run no-op issue #1 [root]"
assert_contains "$out" "dry-run update issue #2 [child]"
assert_contains "$out" "dry-run create issue [new]"
assert_contains "$out" "parent drift for [child]"
assert_contains "$(cat "$MOCK_LOG")" "repos/owner/repo/issues\?state=all\&per_page=100"

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

printf 'All roadmap automation tests passed.\n'
