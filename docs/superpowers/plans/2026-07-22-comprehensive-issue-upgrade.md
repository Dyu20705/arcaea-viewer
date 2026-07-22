# Comprehensive Roadmap Issue Upgrade Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make every active roadmap issue a detailed, solo-executable guide from research and setup through UI/data implementation, testing, evidence, and rollback, then apply and audit the upgraded issues through a temporary GitHub Action.

**Architecture:** Keep phase manifests authoritative for scope and dependency metadata. Add a keyed execution-guidance manifest that is merged into issue records by `scripts/bootstrap-roadmap.sh`, validate complete coverage with `jq`, render the new sections deterministically, and use a temporary restricted PR workflow for one-time apply and post-apply drift auditing.

**Tech Stack:** Bash, jq, GitHub CLI/API, GitHub Actions, JSON manifests, existing roadmap shell tests.

## Global Constraints

- Web MVP remains a static wiki without a production backend, account system, playback, analytics, replay, or protected chart/audio data.
- Existing labels, milestones, `existingNumber`, parent, blocker, state, and supersession mappings remain intact.
- Every open non-epic issue must have non-empty research, setup, implementation, solo execution, validation, deliverable, and risk guidance.
- Temporary issue-write automation must use only `contents: read` and `issues: write`, be restricted to the same repository and exact branch/title, and be deleted after a successful audited run.
- No merge or automatic issue closure is performed by this plan.

---

### Task 1: Add failing execution-guidance tests

**Files:**
- Modify: `tests/roadmap/test-bootstrap-roadmap.sh`

**Interfaces:**
- Consumes: existing `ROADMAP_ROOT_DIR` test harness and mocked `gh` executable.
- Produces: tests requiring `roadmap/issue-execution-guidance.json`, detailed rendered sections, unknown-key rejection, and missing active-issue guidance rejection.

- [ ] Add a guidance fixture for `root`, `child`, and `new`.
- [ ] Change the expected managed body for `root` to contain the new execution sections.
- [ ] Add a test where a guidance key is unknown and require the script to fail closed.
- [ ] Add a test where an open non-epic issue lacks guidance and require the script to fail closed.
- [ ] Open a draft PR and confirm the roadmap CI fails because the production script does not yet load or validate guidance.

### Task 2: Merge, validate, and render execution guidance

**Files:**
- Modify: `scripts/bootstrap-roadmap.sh`
- Create: `roadmap/issue-execution-guidance.json`
- Modify: `roadmap/README.md`

**Interfaces:**
- Consumes: existing phase issue records keyed by `.key`.
- Produces: merged issue records with `researchTasks`, `setupSteps`, `implementationSteps`, `uiUxTasks`, `dataBackendTasks`, `soloExecution`, `validationSteps`, `deliverables`, and `risksAndRollback` arrays.

- [ ] Load and validate the guidance manifest with `jq`.
- [ ] Merge guidance into each issue before reference validation and rendering.
- [ ] Reject duplicate/unknown guidance keys.
- [ ] Require complete guidance for every open non-epic issue outside `cleanup`.
- [ ] Render each new section only when non-empty.
- [ ] Document ownership, coverage, update procedure, and direct-edit drift rules.
- [ ] Run CI and confirm roadmap tests and live dry-run pass.

### Task 3: Populate phase-specific solo execution guidance

**Files:**
- Create/complete: `roadmap/issue-execution-guidance.json`

**Interfaces:**
- Consumes: all roadmap keys from `roadmap/issues.index.json` and included phase manifests.
- Produces: concise but specific guidance for product, UX, data, frontend, assets, content, security, release, and post-MVP work.

- [ ] Cover the root roadmap and phase epics with dependency, capacity, and gate evidence guidance.
- [ ] Cover Week 1 decisions with primary-source research, ADR/prototype outputs, and human approval points.
- [ ] Cover Week 2 foundation with exact setup, migration, schema, image, and quality workflows.
- [ ] Cover Weeks 3–4 pages with list/detail UI states, URL-state behavior, data view models, provenance, and content review.
- [ ] Cover Weeks 5–6 quality/release with measurable audits, reproducible commands, rollback, and public-preview evidence.
- [ ] Cover post-MVP issues with entry criteria and explicit reasons not to implement speculative infrastructure.
- [ ] Keep closed/superseded cleanup records historical and unchanged.

### Task 4: Add and run temporary audited issue application

**Files:**
- Create temporarily: `scripts/apply-comprehensive-issue-upgrade-once.sh`
- Create temporarily: `.github/workflows/apply-comprehensive-issue-upgrade-once.yml`

**Interfaces:**
- Consumes: reviewed PR-head manifests and bootstrap script.
- Produces: upgraded GitHub issue bodies and a post-apply no-drift audit.

- [ ] Restrict the workflow to same-repository PR branch `ops/comprehensive-issue-upgrade` and exact authorization title.
- [ ] Use pinned checkout and least-privilege permissions.
- [ ] Validate shell syntax, JSON manifests, guidance coverage, and roadmap tests.
- [ ] Execute a live dry-run, then `--apply --phase all --force-update` without `--close-superseded`.
- [ ] Execute a second live dry-run and fail if any managed issue or relationship still requires create/update/add/remove/deferred action.
- [ ] Inspect workflow jobs and logs; rerun only failed jobs if the failure is transient.

### Task 5: Remove temporary automation and audit issues

**Files:**
- Delete from PR branch: `scripts/apply-comprehensive-issue-upgrade-once.sh`
- Delete from PR branch: `.github/workflows/apply-comprehensive-issue-upgrade-once.yml`

**Interfaces:**
- Consumes: successful apply workflow evidence.
- Produces: a durable PR containing only canonical manifests, bootstrap logic, tests, and documentation.

- [ ] Delete both one-shot files using their exact current blob SHAs.
- [ ] Confirm the final PR changed-file list contains no temporary issue-write automation.
- [ ] Search and sample issues from every phase; verify all expected sections and dependency metadata.
- [ ] Run a final connector-based completeness review against all managed open issues.
- [ ] Correct any remaining issue-body detail directly with the GitHub connector and mirror the correction in the guidance manifest before merge.
- [ ] Report PR, workflow evidence, issue audit results, remaining human-required decisions, and merge requirement for restoring manifest/issue canonical consistency.
