# Comprehensive Roadmap Issue Upgrade Design

## Goal

Upgrade every roadmap-managed GitHub issue so one maintainer can execute the work professionally from research and environment setup through implementation, validation, evidence, and rollback, while preserving the wiki-first MVP boundaries and existing dependency graph.

## Context

The current roadmap correctly separates the static fan-wiki MVP from deferred runtime, analytics, replay, and hosted-platform work. Several issues, however, contain only outcome, scope, acceptance criteria, and a generic Definition of Done. That is adequate for portfolio-level planning but insufficient as an execution guide for one person covering product, UI/UX, frontend, static-data tooling, release operations, and later backend evaluation.

Sekai Viewer is used as a technical reference for strong patterns—static versioned master data, asset indirection, route-level code splitting, resilient states, lazy media loading, list/detail navigation, and separated dynamic services—but its heavy client-side joins, legacy frontend stack, token storage, broad feature surface, and production asset extraction complexity are not copied into the MVP.

## Architecture

Add `roadmap/issue-execution-guidance.json` as a declarative companion to the existing phase manifests. Each guidance record is keyed by the stable `roadmap-key` and supplies execution-only sections:

- research and source collection;
- environment and setup;
- implementation sequence;
- UI/UX responsibilities;
- data/backend responsibilities;
- solo execution order;
- validation and test plan;
- required deliverables and evidence;
- risks, rollback, and scope cuts.

`scripts/bootstrap-roadmap.sh` merges this guidance into the phase issue records before validation and body rendering. Existing issue fields remain authoritative for product scope, labels, milestones, parents, blockers, and lifecycle state.

## Coverage policy

Every open non-epic roadmap issue must have a guidance record. Epics may have lighter coordination guidance. Closed completed or superseded issues may omit guidance because they are historical records.

Guidance keys that do not correspond to a managed issue fail validation. Duplicate keys fail validation. Required guidance arrays for active non-epic issues must be non-empty.

## Rendering order

Issue bodies render the following sections when present:

1. Outcome
2. Scope
3. Non-goals
4. Research and source collection
5. Environment and setup
6. Implementation sequence
7. UI/UX responsibilities
8. Data/backend responsibilities
9. Solo execution order
10. UX requirements
11. Technical constraints
12. Acceptance criteria
13. Test plan
14. Required deliverables
15. Required evidence
16. Risks, rollback, and scope cuts
17. Definition of Done
18. Planning metadata

The managed marker and existing planning metadata remain unchanged.

## One-shot application workflow

After CI validates the durable changes, a temporary same-repository pull-request workflow applies the reviewed PR-head manifests to GitHub issues with `issues: write` and `contents: read` only. It is restricted by exact repository, branch, and PR title. A temporary shell wrapper performs:

1. shell and JSON preflight checks;
2. roadmap unit tests;
3. live dry-run;
4. authoritative issue/relationship apply without closing superseded issues;
5. post-apply dry-run audit requiring no remaining create/update/relation drift.

The temporary workflow and wrapper are removed from the PR branch after a successful run. Durable manifests, tests, documentation, and bootstrap logic remain.

## Safety constraints

- Do not merge, close issues, or close superseded issues automatically.
- Do not add a production backend to the MVP.
- Do not publish protected chart files, audio, or unlicensed game assets.
- Do not make Sekai Viewer a visual or code template; use it only as one research reference.
- Keep issue bodies below GitHub limits and avoid repetitive enterprise process.
- Preserve `existingNumber`, labels, milestone, parent, blocker, and state mappings.
- Treat repository manifests as canonical; any direct connector correction must also be represented in the pending PR before merge.

## Verification

- Roadmap tests demonstrate RED before implementation and GREEN afterward.
- CI roadmap automation and live read-only dry-run pass.
- Temporary apply workflow completes successfully.
- Post-apply audit reports no managed issue drift.
- Connector review samples every phase and verifies detailed execution sections are present.
- Temporary automation files are absent from the final PR diff.
