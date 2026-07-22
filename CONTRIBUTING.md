# Contributing to Arcaea-Viewer

Thank you for contributing to Arcaea-Viewer.

The current priority is Web MVP 0.1: a fast, accessible, source-aware, static fan wiki for ordinary Arcaea players. Read the [project charter](docs/product/PROJECT_CHARTER.md), [product brief](docs/product/WEB_MVP_BRIEF.md), and [roadmap](docs/roadmap/WEB_MVP_ROADMAP.md) before significant work.

This baseline workflow is designed for one primary maintainer, occasional contributors, and reviewable AI-assisted work. Week 6 may add issue forms, a Code of Conduct, and other release-ready community tooling.

## 1. Core rules

- One issue represents one clear outcome.
- Repository files, GitHub issues, pull requests, reviews, and recorded evidence are canonical.
- Private chats, AI conversations, Discord messages, or local notes are not sufficient as the only record of a decision.
- Do not describe planned functionality as implemented.
- Do not add a production backend, accounts, public runtime features, automatic scraping, or unlicensed assets unless an approved issue changes the current MVP boundary.
- Do not commit secrets, private personal data, proprietary game files, copied third-party prose, or fabricated evidence.
- AI assistance never replaces human verification or maintainer authority.

Roadmap-managed issue bodies are generated from:

```text
roadmap/issues.index.json
roadmap/issues/*.json
scripts/bootstrap-roadmap.sh
```

When an accepted issue change must survive reconciliation, update the relevant manifest rather than only editing the GitHub issue body.

## 2. Supported contribution types

### Documentation and decisions

Examples include setup corrections, architecture explanations, product decisions, policy changes, and broken links.

Keep documentation consistent with the current implementation, charter, product brief, and roadmap.

### Data and factual corrections

A correction must identify:

- affected record or page;
- current and proposed values;
- supporting source;
- applicable game version or date;
- remaining uncertainty.

Do not copy third-party wiki prose, table layouts, screenshots, databases, or media merely because the underlying facts are public.

### UI and accessibility

User-facing changes should include applicable screenshots or recordings plus keyboard, theme, viewport, reduced-motion, and semantic verification.

Do not copy distinctive official or third-party layouts, branding, icons, or protected assets.

### Code, tests, and tooling

Code changes must stay inside approved scope, follow existing boundaries, avoid unnecessary dependencies, and add or update tests when behavior changes.

New tooling must solve a demonstrated problem. Do not add infrastructure solely to make the project appear more sophisticated.

## 3. Issue-first planning

Open or use an issue for:

- product behavior or route changes;
- schemas and data contracts;
- legal, provenance, or asset decisions;
- accessibility or security changes;
- dependency or build-tool changes;
- substantial refactoring;
- roadmap changes;
- work requiring multiple commits or review decisions.

A separate issue is normally unnecessary for an isolated typo, broken link, or mechanical formatting correction. The pull request must still explain the problem and scope.

## 4. Definition of Ready

Work is **Ready** when all applicable items are satisfied:

- [ ] Outcome is observable.
- [ ] Scope and non-goals are explicit.
- [ ] Dependencies and blockers are recorded.
- [ ] Owner or assignee is identified.
- [ ] Affected files, packages, routes, records, or user surfaces are known.
- [ ] Required product decisions are resolved.
- [ ] Legal, licensing, source, provenance, privacy, and asset constraints are identified.
- [ ] Acceptance criteria are specific and verifiable.
- [ ] Automated and manual checks are defined.
- [ ] Required evidence is defined.
- [ ] Risks and rollback needs are understood.
- [ ] The issue is small enough for a focused implementation cycle.

If a required dependency or decision is missing, mark the issue blocked instead of inventing an answer during implementation.

Investigations may start with incomplete implementation detail only when they have a clear question, boundaries, evidence expectations, decision owner, and stopping condition.

## 5. Local setup

Requirements:

- Git;
- Rust with `wasm32-unknown-unknown`;
- Node.js 22 or newer;
- pnpm 10.32.1;
- Bash, authenticated `gh`, `jq`, and GNU `date` for roadmap automation.

```bash
git clone https://github.com/Dyu20705/arcaea-viewer.git
cd arcaea-viewer
rustup target add wasm32-unknown-unknown
pnpm install --frozen-lockfile
pnpm dev
```

Run the complete repository check with:

```bash
pnpm check
```

Do not report a command as passing unless it was actually executed and its output inspected.

## 6. Branches and commits

Start from current `main`:

```bash
git switch main
git pull --ff-only
git switch -c <type>/issue-<number>-<short-slug>
```

Examples:

```text
docs/issue-2-project-charter-workflow
data/issue-14-seed-catalog
feat/issue-43-explore-filters
fix/issue-22-navigation-focus
```

Use focused, imperative commit messages:

```text
docs(product): add project charter
docs(process): record workflow validation
fix(accessibility): restore visible navigation focus
data(catalog): correct sourced song metadata
```

Keep unrelated cleanup out of the branch.

## 7. Implementation workflow

1. Read the issue and governing documents.
2. Confirm the Definition of Ready.
3. Inspect affected files and existing conventions.
4. Create a focused branch.
5. Reproduce the problem or define expected behavior.
6. Add or update tests when behavior changes.
7. Implement the smallest accepted change.
8. Run focused checks.
9. Review the diff for scope creep, secrets, generated artifacts, and accidental files.
10. Run final required checks.
11. Open or update a draft pull request.
12. Map acceptance criteria to evidence.
13. Resolve blocking review findings.
14. Obtain required maintainer approvals.
15. Merge, record final evidence, and close the issue.

## 8. Validation by change type

### Documentation-only

```bash
git diff --check
```

Verify paths, commands, links, product wording, and consistency with the charter, brief, roadmap, and implementation.

### Frontend

```bash
pnpm run lint
pnpm run test:web
pnpm run build
pnpm check
```

Record applicable keyboard, focus, theme, viewport, loading/empty/error, reduced-motion, and semantic checks.

### Rust or WebAssembly

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
pnpm run wasm:build
pnpm check
```

### Data and catalog

Run every available schema, duplicate-ID, reference-integrity, provenance, asset-record, catalog-test, and production-build check.

Clearly distinguish syntax validation from source or permission review.

### Roadmap manifests or automation

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

Inspect the live plan for unexpected issue, milestone, label, assignee, parent, dependency, or closure changes.

### Security-sensitive changes

Record the threat or failure scenario, regression evidence, secret-exposure review, unsafe-input review, residual risk, and explicit maintainer decision. Do not publish sensitive exploit evidence in a public issue.

## 9. Pull request requirements

A pull request should contain:

```markdown
## Outcome

Describe the observable result.

Refs #<issue-number>.

## Changed

- important changes
- affected files or surfaces
- explicit scope cuts

## Acceptance evidence

- [ ] criterion — file, test, screenshot, review, or other evidence

## Validation

- [ ] exact command or manual check — result

## Sources and assets

- Sources added or changed:
- Assets added or changed:
- Permission evidence:
- N/A reason:

## AI assistance

- AI-assisted: Yes / No
- Scope of assistance:
- Human verification:
- Commands actually executed:
- Facts, sources, or permissions reviewed by:

## Risks and exceptions

- Known risks:
- Accepted exceptions:
- Follow-up issue:
- Rollback approach:
```

Use `N/A` with a reason instead of deleting applicable review categories.

Prefer a pull request small enough to understand without reconstructing the project. Split unrelated subsystems, bulk content, optional refactoring, or generated output when they are independently reviewable.

## 10. Review and human authority

Review evaluates scope, acceptance criteria, correctness, tests, documentation, provenance, asset permission, accessibility, privacy, security, performance, maintainability, and rollback feasibility.

Classify comments as blocking, required, suggestion, question, or follow-up. Do not implement feedback mechanically; verify that it is correct and within scope.

Explicit maintainer approval is required for:

- mission or MVP scope;
- major public routes or feature categories;
- legal or licensing interpretation;
- third-party asset publication;
- disputed high-impact facts;
- security, privacy, accessibility, or performance exceptions;
- public release readiness and rollback;
- repository licensing;
- merge authority;
- closure of `status:human-required` issues.

An AI-generated approval statement is not maintainer approval.

## 11. AI-assisted contributions

AI may assist with orientation, research planning, design alternatives, implementation plans, code, tests, documentation, review, and failure analysis.

The contributor remains responsible for repository state, generated code, commands and test results, factual claims, citations, permissions, security/privacy risks, and scope.

AI tools must not invent tests, sources, permissions, GitHub state, or product decisions; claim final legal authority; publish secrets; merge changes; or close human-required issues.

Disclose what AI produced, what a human reviewed, what commands actually ran, and what still requires human judgment.

## 12. Sources, assets, and sensitive reports

Prefer official sources, official platform information, directly verifiable in-game facts, compatible open or permission-granted sources, and finally independently verified third-party references.

When sources conflict, record the conflict and version applicability instead of silently selecting a convenient value.

Do not publish a media asset without a reviewable source and permission basis. Repository GPL licensing does not automatically cover third-party game content.

For ordinary bugs and non-sensitive corrections, use public issues. For sensitive security, privacy, licensing, or takedown matters, use an available private repository-owner profile contact method until a dedicated channel is published.

## 13. Scope changes and exceptions

When implementation discovers new scope:

1. stop expanding the change;
2. record the new requirement and why it is necessary;
3. document alternatives and effects on acceptance criteria;
4. obtain approval before continuing.

A blocker record should include the blocked outcome, dependency or decision, evidence, owner, and next decision point.

An exception record should include the unmet requirement, reason, impact, risk, compensating controls, owner, review point, and follow-up issue.

## 14. Definition of Done

A contribution is **Done** only when all applicable items are satisfied:

- [ ] Approved scope is implemented with no hidden scope creep.
- [ ] Every acceptance criterion maps to reviewable evidence.
- [ ] Relevant automated and manual checks are complete.
- [ ] Changed behavior has appropriate tests.
- [ ] Documentation matches final behavior.
- [ ] Source, provenance, and asset permissions are reviewed.
- [ ] Accessibility, privacy, security, and performance impacts are reviewed.
- [ ] Generated artifacts and dependency changes are reviewed.
- [ ] Risks, exceptions, and follow-up ownership are recorded.
- [ ] AI assistance and human verification are disclosed.
- [ ] Required maintainer decisions are recorded.
- [ ] CI passes.
- [ ] Pull request is merged.
- [ ] Final evidence is linked from the issue.
- [ ] The issue is closed with the correct reason.

Passing CI alone does not satisfy the Definition of Done.

## 15. Merge and closure

The maintainer owns the final merge decision. Squash merge is the default unless preserving individual commits provides clear review value.

Do not merge unresolved conflicts, unexplained failing tests, fabricated or unverified evidence, unresolved legal/licensing decisions, or AI-generated changes without human verification.

Close an issue only after its outcome is delivered or conclusively rejected/superseded. A completion record should link the merged pull request, final evidence, checks, known exceptions, follow-up issues, and required maintainer approval.
