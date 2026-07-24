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
{"schemaVersion":1,"repository":"owner/repo","existingNumbers":{"root":1},"includes":["issues/test.json"]}
JSON
cat >"$ROOT/roadmap/issues/test.json" <<'JSON'
{"schemaVersion":1,"issues":[
  {"key":"root","title":"Root","phase":"roadmap","milestone":"m1","labels":["type:test"],"outcome":"Root outcome.","scope":[],"nonGoals":[],"uxRequirements":[],"technicalConstraints":[],"acceptanceCriteria":[],"testPlan":[],"evidence":[],"definitionOfDone":[],"parent":null,"blockedBy":[],"assignees":["tester"],"state":"open"}
]}
JSON
cat >"$ROOT/roadmap/issue-execution-guidance.json" <<'JSON'
{"schemaVersion":1,"defaults":{
  "researchTasks":["Research."],"setupSteps":["Set up."],"implementationSteps":["Implement."],"uiUxTasks":["Review UI."],"dataBackendTasks":["Review data."],"soloExecution":["Execute in order."],"validationSteps":["Validate."],"deliverables":["Deliver."],"risksAndRollback":["Rollback."]
},"issues":[{"key":"root"}]}
JSON

export MOCK_LOG
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
[[ "$method" == "GET" ]] || { printf '{}\n'; exit 0; }
case "$endpoint" in
  repos/owner/repo/labels\?*) printf '[{"name":"type:test","color":"abcdef","description":"test label"}]\n' ;;
  repos/owner/repo/milestones\?*) printf '[{"number":1,"state":"open","title":"M1","description":"first milestone","due_on":"2026-07-14T23:59:59Z"}]\n' ;;
  repos/owner/repo/issues\?*) printf '[{"number":1,"id":101,"state":"open","state_reason":null,"title":"Legacy root","body":"legacy body without marker","labels":[{"name":"type:test"}],"assignees":[{"login":"tester"}],"milestone":{"number":1,"title":"M1"}}]\n' ;;
  repos/owner/repo/issues/1/parent) exit 1 ;;
  repos/owner/repo/issues/1/dependencies/blocked_by\?*) printf '[]\n' ;;
  *) printf '{}\n' ;;
esac
MOCK
chmod +x "$MOCK_BIN/gh"

fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
assert_contains() { grep -Fq -- "$2" <<<"$1" || fail "expected output to contain: $2"; }
assert_not_contains() { grep -Fq -- "$2" <<<"$1" && fail "expected output not to contain: $2" || true; }

printf 'test: central existingNumbers maps a markerless legacy issue\n'
out="$(PATH="$MOCK_BIN:$PATH" ROADMAP_ROOT_DIR="$ROOT" ROADMAP_SLEEP_SECONDS=0 \
  bash "$SCRIPT_UNDER_TEST" --dry-run --phase roadmap --repo owner/repo --start-date 2026-07-14 2>&1)"
assert_contains "$out" "dry-run update issue #1 [root]"
assert_not_contains "$out" "dry-run create issue [root]"

printf 'test: unknown existingNumbers key fails closed\n'
jq '.existingNumbers.ghost = 2' "$ROOT/roadmap/issues.index.json" >"$ROOT/roadmap/issues.index.json.tmp"
mv "$ROOT/roadmap/issues.index.json.tmp" "$ROOT/roadmap/issues.index.json"
if PATH="$MOCK_BIN:$PATH" ROADMAP_ROOT_DIR="$ROOT" ROADMAP_SLEEP_SECONDS=0 \
  bash "$SCRIPT_UNDER_TEST" --dry-run --phase roadmap --repo owner/repo --start-date 2026-07-14 >"$TEST_TMP/unknown.out" 2>&1; then
  fail "unknown existingNumbers key was accepted"
fi
assert_contains "$(cat "$TEST_TMP/unknown.out")" "existingNumbers references unknown issue key: ghost"

printf 'test: duplicate existingNumbers value fails closed\n'
jq '.existingNumbers = {root:1,other:1} | .includes=["issues/test-with-other.json"]' "$ROOT/roadmap/issues.index.json" >"$ROOT/roadmap/issues.index.json.tmp"
mv "$ROOT/roadmap/issues.index.json.tmp" "$ROOT/roadmap/issues.index.json"
jq '.issues += [{"key":"other","title":"Other","phase":"roadmap","milestone":"m1","labels":["type:test"],"outcome":"Other.","scope":[],"nonGoals":[],"uxRequirements":[],"technicalConstraints":[],"acceptanceCriteria":[],"testPlan":[],"evidence":[],"definitionOfDone":[],"parent":null,"blockedBy":[],"assignees":["tester"],"state":"closed"}]' \
  "$ROOT/roadmap/issues/test.json" >"$ROOT/roadmap/issues/test-with-other.json"
if PATH="$MOCK_BIN:$PATH" ROADMAP_ROOT_DIR="$ROOT" ROADMAP_SLEEP_SECONDS=0 \
  bash "$SCRIPT_UNDER_TEST" --dry-run --phase roadmap --repo owner/repo --start-date 2026-07-14 >"$TEST_TMP/duplicate.out" 2>&1; then
  fail "duplicate existingNumbers value was accepted"
fi
assert_contains "$(cat "$TEST_TMP/duplicate.out")" "duplicate existingNumbers issue number: 1"

printf 'All existing-number mapping tests passed.\n'
