# Roadmap automation

This directory is the declarative source for labels, milestones, issue content, sub-issues, issue dependencies, and solo-execution guidance.

- `issues.index.json` lists reviewable phase manifests under `roadmap/issues/`.
- `issue-execution-guidance.json` adds the research, setup, implementation, UI/UX, data/backend, solo sequencing, validation, deliverable, and rollback sections rendered into managed issue bodies.
- Phase manifests remain authoritative for product scope, labels, milestones, parents, blockers, state, and supersession.

## Execution-guidance coverage

Every open non-epic issue outside the cleanup phase must have a matching record in `issue-execution-guidance.json`. The bootstrap fails closed when:

- a guidance key is duplicated;
- a guidance key does not match a managed roadmap key;
- an active non-epic issue has no guidance record;
- the resolved guidance is missing a non-empty required execution section.

The guidance manifest has shared defaults plus issue-specific additions. Keep repeated professional workflow requirements in `defaults`; keep domain decisions, commands, research targets, UI states, data responsibilities, and rollback details in the keyed issue entry.

Direct edits to a roadmap-managed GitHub issue are temporary unless the same accepted change is represented in the phase manifest or execution-guidance manifest. A later reconciliation restores repository-declared content.

## Safety model

The normal workflow is manual (`workflow_dispatch`) and defaults to `dry-run`.

- `dry-run` authenticates with read-only issue permissions, reads the current GitHub state, and reports `create`, `update`, `no-op`, relation drift, and deferred relations.
- `apply` writes labels, milestones, issues, sub-issue relationships, and dependencies.
- workflow inputs are passed through environment variables and then validated as data; they are never expanded directly into a shell program.
- `start_date` must be a canonical `YYYY-MM-DD` calendar date.
- duplicate `roadmap-key` markers or conflicting `existingNumber` mappings fail closed.
- issue content is idempotently managed using a stable `roadmap-key` marker.
- superseded issue closure requires `--close-superseded`.
- `--force-update` makes parent and `blocked_by` relationships authoritative: changed or removed relations are deleted before desired relations are added.
- dry-run uses `issues: read`; only an explicitly reviewed apply job receives `issues: write`.
- one-time issue-write workflows must be same-repository, exact-branch/title restricted, least-privilege, audited after apply, and deleted after successful use.

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

Do not pass `--close-superseded` during a content/detail upgrade unless closure is an independently reviewed goal. It is intentionally separate from body and relationship reconciliation.

## Validation

```bash
jq -e '.schemaVersion == 1' roadmap/issue-execution-guidance.json
bash -n scripts/bootstrap-roadmap.sh
bash -n tests/roadmap/test-bootstrap-roadmap.sh
bash tests/roadmap/test-bootstrap-roadmap.sh
bash scripts/bootstrap-roadmap.sh \
  --dry-run \
  --phase all \
  --force-update \
  --start-date 2026-07-14 \
  --repo Dyu20705/arcaea-viewer
```

The CI roadmap job runs the shell tests and then performs a live read-only dry-run against the repository. The tests cover shell-input rejection, guidance rendering and coverage, unknown guidance rejection, live-state planning, duplicate-marker rejection, and exact parent/dependency reconciliation.

After an authorized apply, run the same dry-run with `--force-update`. A complete reconciliation must contain only `no-op` issue, parent, and dependency plans, plus intentional skips for closed superseded records.

The current GitHub REST API version used by the script is `2026-03-10`.
