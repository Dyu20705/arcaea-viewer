# Project Workflow Validation

## 1. Purpose and status

This document records the consistency review and tabletop validation for issue #2: project charter, community promise, and production workflow.

| Field | Value |
| --- | --- |
| Repository | `Dyu20705/arcaea-viewer` |
| Pull request | `#81` |
| Phase | Week 1 — discovery and design gate |
| Target branch | `main` |
| Validation owner | Repository maintainer |
| Canonical roadmap start date | `2026-07-14` |
| Status | Automated validation complete; awaiting maintainer approval |

The goal is not to prove that every future contribution will be correct. The goal is to verify that the workflow provides enough information to decide whether work is Ready, implement it without inventing policy, evaluate evidence, and decide whether it is Done.

## 2. Records reviewed

Repository documents:

- `README.md`;
- `CONTRIBUTING.md`;
- `docs/product/PROJECT_CHARTER.md`;
- `docs/product/WEB_MVP_BRIEF.md`;
- `docs/roadmap/WEB_MVP_ROADMAP.md`;
- `roadmap/README.md`;
- `roadmap/issues/week-1.json`;
- `roadmap/issues/week-6.json`.

GitHub records:

- issue #2 — charter and production workflow;
- issue #3 — wiki-first product requirements;
- issue #11 — legal and provenance guardrails;
- issue #12 — canonical roadmap;
- issue #53 — release-ready community tooling.

## 3. Consistency invariants

The reviewed records must agree that:

1. the current product direction is a public, wiki-first fan database for ordinary Arcaea players;
2. the MVP uses static, versioned, validated data and does not require a production backend, database, accounts, or user-generated content;
3. existing Rust/WASM runtime work is preserved but remains outside public MVP navigation and cannot block the wiki release;
4. repository code licensing is separate from permission to publish third-party facts, prose, and media;
5. contributors and AI may propose, implement, test, and review, while the maintainer retains final product, legal, asset, exception, release, rollback, merge, and human-required closure authority;
6. Week 1 establishes the baseline charter and workflow, while Week 6 expands it with release-ready community tooling.

## 4. Contradictions resolved

| ID | Previous problem | Resolution |
| --- | --- | --- |
| C-01 | README identified the project primarily as a chart-viewer prototype. | Separate current product direction from current implementation. |
| C-02 | README recommended more viewer ergonomics despite the accepted wiki-first roadmap. | Replace the viewer-first recommendation with the six-week wiki sequence and explicit runtime deferral. |
| C-03 | Governing documents were not linked together. | Link the charter, brief, roadmap, contribution guide, and this validation record from README. |
| C-04 | Week 6 wording implied that no contribution workflow existed before release. | Define Week 1 as the baseline and Week 6 as release-ready workflow expansion and issue forms. |
| C-05 | Planned wiki features could be mistaken for implemented behavior. | Use explicit “planned MVP” and “current implementation” wording. |
| C-06 | GPL-3.0-only could be misread as permission for all game content. | State that repository licensing does not grant third-party content permission. |
| C-07 | AI assistance lacked one explicit authority boundary. | Centralize human-required decisions in the charter and CONTRIBUTING. |
| C-08 | No dedicated private project contact exists yet. | Direct sensitive matters to an available private owner-profile contact method until a dedicated channel is published. |
| C-09 | The initial PR targeted `dev`, producing an unrelated 112-file diff. | Retarget to `main` and rebuild the branch from the current `main` head. |
| C-10 | The initial documents were too long for the stated low-overhead workflow. | Remove duplication while retaining every issue #2 acceptance category and evidence gate. |

## 5. Tabletop scenario A — factual metadata correction

### Scenario

A contributor reports an incorrect BPM value and provides an official source with an applicability date or game version.

### Ready evaluation

| Requirement | Evidence | Decision |
| --- | --- | --- |
| Observable outcome | One identified metadata value is corrected. | Satisfied |
| Scope/non-goals | One record, source record, and related validation; no schema redesign or asset changes. | Satisfied |
| Dependency | Static catalog schema and source records must exist. | **Blocked in the current repository** |
| Ownership | Contributor implements; maintainer reviews source interpretation. | Satisfied |
| Acceptance/tests | Correct value, valid source, reference integrity, catalog validation, and build evidence. | Defined |
| Required evidence | Before/after diff, source, version/date, checks, maintainer approval. | Defined |

**Current Ready decision:** Not Ready. The workflow correctly blocks implementation until the metadata schema and provenance contract exist.

**Future Ready condition:** The relevant schema, source record format, validation command, and affected catalog path are available and linked from the issue.

### Simulated Done gate

After the dependency exists, the correction is Done only when:

- the scoped record and source are updated;
- source applicability and uncertainty are reviewed;
- relevant catalog and repository checks pass;
- no asset or unrelated schema change is included;
- AI assistance is disclosed;
- the maintainer approves the factual interpretation;
- the PR is merged and final evidence is linked.

### Ambiguity resolved

An official publication does not automatically override current in-game behavior. Record source date, game version, and any conflict rather than silently selecting one value.

## 6. Tabletop scenario B — keyboard focus fix

### Scenario

A contributor reports that the primary navigation focus indicator is difficult to see in one or both themes.

### Ready evaluation

| Requirement | Evidence | Decision |
| --- | --- | --- |
| Observable outcome | Keyboard users can identify the focused navigation control. | Satisfied |
| Scope/non-goals | Focus style and related tests only; no navigation or theme redesign. | Satisfied |
| Dependency | The public navigation component and approved theme tokens must exist. | **Blocked in the current repository** |
| Ownership | Contributor implements; maintainer reviews user-facing evidence. | Satisfied |
| Acceptance/tests | Visible focus in supported themes, unchanged focus order, keyboard walkthrough, and frontend checks. | Defined |
| Required evidence | Before/after visuals, browser/viewport/theme, keyboard result, tests, maintainer approval. | Defined |

**Current Ready decision:** Not Ready. The workflow correctly blocks a production accessibility fix against temporary viewer UI.

**Future Ready condition:** The target navigation component, design tokens, supported themes, and expected keyboard behavior exist and are linked from the issue.

### Simulated Done gate

After the dependency exists, the fix is Done only when:

- focused tests and full frontend checks pass;
- keyboard reachability and order are verified manually or through reliable interaction tests;
- focus is visible in supported themes and viewports;
- screenshots or recordings identify the tested environment;
- accessibility, privacy, security, and performance impacts are reviewed;
- AI assistance is disclosed;
- the maintainer approves the user-facing result;
- the PR is merged and final evidence is linked.

### Ambiguity resolved

A screenshot alone is insufficient accessibility evidence. It cannot prove keyboard reachability, focus order, or persistence.

## 7. Cross-scenario findings

The workflow demonstrated that:

- one issue maps to one observable outcome;
- unresolved dependencies produce a clear Not Ready decision instead of speculative implementation;
- facts require source and version applicability;
- user-facing accessibility changes require manual or reliable interaction evidence;
- CI passing alone does not prove Done;
- N/A review categories require a reason;
- AI assistance requires human verification;
- issue closure follows merge and final evidence.

## 8. Exact validation plan

### Review surface

```bash
git diff --name-only main...HEAD
git diff --check main...HEAD
```

Expected changed files:

```text
CONTRIBUTING.md
README.md
docs/process/WORKFLOW_VALIDATION.md
docs/product/PROJECT_CHARTER.md
docs/roadmap/WEB_MVP_ROADMAP.md
```

No application code, dependencies, workflows, generated artifacts, assets, roadmap manifests, or issue automation should change.

### Documentation checks

Verify manually that:

- all linked repository paths exist;
- documented commands match `package.json`, CI, and `roadmap/README.md`;
- README distinguishes planned product from current implementation;
- the unofficial statement is consistent;
- licensing does not imply third-party content permission;
- the charter and CONTRIBUTING define maintainer-only decisions;
- Week 1 and Week 6 responsibilities do not overlap;
- no document claims that pending checks or approvals passed.

### Repository quality gate

```bash
pnpm install --frozen-lockfile
pnpm check
```

Expected: Rust format, Clippy, Rust tests, WASM build, frontend typecheck, frontend tests, and frontend production build complete with exit status `0`.

### Roadmap validation

```bash
bash -n scripts/bootstrap-roadmap.sh
bash -n tests/roadmap/test-bootstrap-roadmap.sh
bash tests/roadmap/test-bootstrap-roadmap.sh
bash scripts/bootstrap-roadmap.sh \
  --dry-run \
  --phase all \
  --start-date 2026-07-14 \
  --repo Dyu20705/arcaea-viewer
```

Expected:

- roadmap shell checks and tests pass;
- the live plan reports no unexplained managed-state drift;
- issue #2 remains consistent with `roadmap/issues/week-1.json`;
- no unexpected create, update, close, parent, dependency, label, milestone, or assignee operation is planned.

A successful exit code is not enough; review the printed plan.

## 9. Evidence record

| Evidence | Status | Location or notes |
| --- | --- | --- |
| Documentation consistency review | Complete | Sections 3–4 |
| Metadata-correction tabletop | Complete | Section 5; current decision is Not Ready |
| Accessibility-fix tabletop | Complete | Section 6; current decision is Not Ready |
| Focused five-file diff | Complete | GitHub comparison against `main` |
| Whitespace and link review | Complete | Manual review of the final five-file diff and repository-relative links |
| `pnpm check` | Complete | CI run `29920882369`, job `check` |
| Roadmap tests | Complete | CI run `29920882369`, job `roadmap-automation` |
| Live roadmap dry-run | Complete | CI run `29920882369`, job `roadmap-automation` |
| Maintainer policy approval | Pending | Maintainer comment on PR or issue #2 |

The evidence-only update that records these results must also retain green checks on the final pull-request head.

## 10. Maintainer approval gate

Before merge, the maintainer must explicitly approve:

- unofficial fan-project wording;
- community promise;
- correction and takedown path;
- repository-license versus third-party-content boundary;
- AI collaborator responsibility boundary;
- Definition of Ready and Definition of Done;
- production workflow and merge policy;
- Week 1 baseline versus Week 6 expansion.

Suggested approval comment:

```markdown
## Maintainer approval for issue #2

I reviewed and approve:

- [ ] unofficial fan-project wording;
- [ ] community promise and correction/takedown path;
- [ ] repository-license and third-party-content boundary;
- [ ] AI collaborator responsibility boundaries;
- [ ] Definition of Ready and Definition of Done;
- [ ] production workflow and merge policy;
- [ ] Week 1 baseline and Week 6 community-expansion boundary.

Final product, legal, licensing, asset-publication, exception, release,
rollback, merge, and human-required issue-closure decisions remain
human-owned.
```

An AI-generated approval statement is not maintainer approval.

## 11. Closure gate

Issue #2 may close only when:

- the five-file diff is reviewed and merged;
- applicable CI and roadmap checks pass;
- the live dry-run has no unexplained drift;
- the maintainer approval gate is complete;
- every exception has an owner and follow-up issue;
- final evidence is linked from the issue or pull request.

Until then, the PR should reference issue #2 rather than auto-close it.
