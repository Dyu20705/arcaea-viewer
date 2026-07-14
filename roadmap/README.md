# Roadmap automation

This directory is the declarative source for labels, milestones, issue content, sub-issues, and issue dependencies. `issues.index.json` lists reviewable phase manifests under `roadmap/issues/`.

## Safety model

The workflow is manual (`workflow_dispatch`) and defaults to `dry-run`.

- `dry-run` authenticates with read-only issue permissions, reads the current GitHub state, and reports `create`, `update`, `no-op`, relation drift, and deferred relations.
- `apply` writes labels, milestones, issues, sub-issue relationships, and dependencies.
- workflow inputs are passed through environment variables and then validated as data; they are never expanded directly into a shell program.
- `start_date` must be a canonical `YYYY-MM-DD` calendar date.
- duplicate `roadmap-key` markers or conflicting `existingNumber` mappings fail closed.
- issue content is idempotently managed using a stable `roadmap-key` marker.
- superseded issue closure requires `--close-superseded`.
- `--force-update` makes parent and `blocked_by` relationships authoritative: changed or removed relations are deleted before desired relations are added.
- dry-run uses `issues: read`; only the apply job receives `issues: write`.

## Local prerequisites

- Bash
- GitHub CLI authenticated for the target repository
- `jq`
- GNU `date`

Dry-run also requires GitHub authentication because it validates the live repository instead of assuming every entry will be created.

## Examples

```bash
bash scripts/bootstrap-roadmap.sh --dry-run --phase all --start-date 2026-07-14
bash scripts/bootstrap-roadmap.sh --apply --phase week-1 --start-date 2026-07-14
bash scripts/bootstrap-roadmap.sh --apply --phase all --close-superseded --force-update
```

Use `--phase` for later updates after the root roadmap and referenced cross-phase issues already exist. The first write should normally use `--phase all` so the full issue map is available before sub-issue and dependency relationships are created.

## Validation

```bash
bash -n scripts/bootstrap-roadmap.sh
bash -n tests/roadmap/test-bootstrap-roadmap.sh
tests/roadmap/test-bootstrap-roadmap.sh
```

The CI roadmap job runs these tests and then performs a live read-only dry-run against the repository. The tests cover shell-input rejection, live-state planning, duplicate-marker rejection, and exact parent/dependency reconciliation.

The current GitHub REST API version used by the script is `2026-03-10`.
