# Roadmap automation

This directory is the declarative source for labels, milestones, issue content, sub-issues, and issue dependencies. `issues.index.json` lists reviewable phase manifests under `roadmap/issues/`.

## Safety model

The workflow is manual (`workflow_dispatch`) and defaults to `dry-run`.

- `dry-run` prints intended changes.
- `apply` writes labels, milestones, issues, sub-issue relationships, and dependencies.
- superseded issue closure requires `--close-superseded`.
- issue content is idempotently managed using a stable `roadmap-key` marker.
- the script updates existing issue numbers declared in the phase manifests and discovers created issues by marker.

## Local prerequisites

- Bash
- GitHub CLI authenticated for the target repository
- `jq`
- GNU `date`

## Examples

```bash
bash scripts/bootstrap-roadmap.sh --dry-run --phase all --start-date 2026-07-14
bash scripts/bootstrap-roadmap.sh --apply --phase all --start-date 2026-07-14
bash scripts/bootstrap-roadmap.sh --apply --phase all --close-superseded --force-update
```

Use `--phase` for later updates after the root roadmap and referenced cross-phase issues already exist. The first write should normally use `--phase all` so the full issue map is available before sub-issue and dependency relationships are created.

`--force-update` enforces the manifest's open/closed state for managed issues. Without it, normal open issues are not forcibly reopened or closed; superseded closures still require `--close-superseded`.

The current GitHub REST API version used by the script is `2026-03-10`.
