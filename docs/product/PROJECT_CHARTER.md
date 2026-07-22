# Arcaea-Viewer Project Charter

## Status and ownership

| Field | Value |
| --- | --- |
| Project | Arcaea-Viewer |
| Status | Active development |
| Current phase | Web MVP 0.1 — wiki-first public preview |
| Repository license | GPL-3.0-only |
| Governance owner | Repository maintainer |

This charter defines the mission, product boundaries, decision authority, and community commitments of Arcaea-Viewer. It applies to maintainers, human contributors, and AI-assisted contributors.

Repository files, GitHub issues, pull requests, reviews, and recorded decisions are the authoritative project record. Decisions required to reproduce, review, publish, release, or maintain the project must not exist only in private chats, local notes, or AI conversations.

## Mission

Arcaea-Viewer exists to provide ordinary Arcaea players with a fast, accessible, reliable, and clearly sourced fan wiki for discovering and understanding game content.

The project should make songs, charts, packs, partners, story content, releases, events, and game systems easier to find and verify than in an unstructured wiki. It must remain sustainable for one primary maintainer and occasional contributors.

Technical complexity is justified only when it creates measurable player value, improves trustworthiness, or reduces long-term maintenance risk.

## Audience and product promise

The primary audience is ordinary Arcaea players who want to:

- discover songs, artists, packs, partners, and story content;
- look up difficulties, chart constants, BPM, note counts, availability, and release information;
- navigate between related game entities;
- understand current game systems and terminology;
- verify where published information came from;
- identify information that is current, uncertain, incomplete, or under review.

Secondary audiences include contributors, maintainers, and software-engineering reviewers.

Portfolio value is a legitimate secondary outcome. It must not override user usefulness, factual accuracy, legal constraints, accessibility, privacy, security, or maintainability.

The Web MVP promises that a visitor can:

- understand the project;
- browse or search a versioned catalog;
- open useful wiki pages;
- navigate between related entities;
- identify source and update status;
- use keyboard-accessible navigation and supported themes;
- receive clear loading, empty, unavailable, error, and not-found states.

## MVP boundaries

The MVP is a static, public, wiki-first product.

It excludes:

- account registration and authentication;
- application-hosted user-generated content;
- a production backend or database server;
- chart upload, editing, playback, analytics, and replay;
- audio synchronization or previews;
- automatic scraping of third-party wikis;
- redistribution of unlicensed game assets;
- infrastructure added only to make the project appear enterprise-scale.

Existing Rust, WebAssembly, parser, timing, renderer, and chart-preview work remains preserved. It does not define the public MVP navigation and must not block delivery of the wiki.

## Project values

### Player usefulness

The project serves ordinary players first. Features and architecture should reduce the effort required to find, understand, and verify useful information.

### Accuracy and provenance

Published facts must be traceable to reviewable sources. Unknown, disputed, outdated, estimated, or incomplete information must be represented honestly.

Source applicability, review status, game version, and asset permission must be designed into the content model and contribution workflow rather than added during release cleanup.

### Original, respectful presentation

Navigation, summaries, editorial structure, and visual design should be original. The project must not copy distinctive wording, layouts, branding, protected assets, or long-form content without compatible permission.

### Accessibility, privacy, and security

Keyboard use, visible focus, readable contrast, semantic structure, reduced-motion preferences, responsive layouts, and assistive-technology compatibility are product requirements.

The project collects no personal data by default. Contributors and users must never be asked to publish credentials, proprietary files, private account data, or unnecessary personal information as evidence.

### Sustainable engineering

The project favors:

- small, reviewable changes;
- explicit boundaries;
- deterministic data and builds;
- minimal dependencies;
- automated verification where practical;
- documented manual review where automation is insufficient;
- infrastructure proportional to demonstrated requirements.

### Human accountability

AI tools may assist with research, planning, implementation, testing, documentation, and review. Human maintainers remain accountable for product scope, factual publication, legal interpretation, asset permission, security exceptions, releases, rollbacks, and merges.

## Decision priority

When project goals conflict, use this order:

1. legal, licensing, privacy, and user-safety constraints;
2. factual accuracy and source provenance;
3. usefulness to ordinary Arcaea players;
4. accessibility and security;
5. maintainability for current maintainer capacity;
6. reliability and performance;
7. contributor experience;
8. visual refinement;
9. portfolio presentation;
10. speculative future extensibility.

A lower-priority goal must not override a higher-priority requirement without an explicit, documented maintainer decision.

## Governance and responsibility

The repository maintainer is the final decision owner.

| Actor | May do | May not do without maintainer approval |
| --- | --- | --- |
| Human contributor | Propose, implement, test, document, review, and provide evidence | Publish disputed facts or protected assets; accept exceptions; merge; release |
| AI collaborator | Inspect context, draft plans/code/docs/tests, analyze failures, and identify risks | Make final legal decisions; approve assets or security exceptions; merge; release; close human-required issues |
| Maintainer | Review evidence and make final product, legal, release, rollback, and merge decisions | Bypass recorded evidence or conceal unresolved high-impact risk |

The following always require explicit maintainer approval:

- changes to mission or MVP scope;
- major public routes or feature categories;
- legal or licensing interpretation;
- publication of third-party assets;
- acceptance of disputed high-impact facts;
- security, privacy, accessibility, or performance exceptions;
- public release readiness and rollback;
- repository licensing changes;
- merge authority;
- closure of issues marked `status:human-required`.

AI-assisted work must remain reviewable. Pull requests must disclose what AI produced and what a human actually verified.

## Community promise

Arcaea-Viewer will:

- welcome good-faith corrections;
- evaluate corrections by evidence rather than contributor status;
- record source conflicts and visible uncertainty;
- credit accepted contributions through Git and GitHub;
- reject fabricated evidence, unsafe links, secrets, unnecessary personal data, proprietary files, copied prose, and unlicensed assets;
- keep reviews focused on work, evidence, scope, and risk;
- avoid misleading claims of official authority.

A factual correction should identify the affected content, current value, proposed value, source, applicable game version or date, and remaining uncertainty.

Preferred source order:

1. official Arcaea or lowiro publications;
2. official platform or store information;
3. directly verifiable in-game facts documented without redistributing protected files;
4. compatible open or permission-granted sources;
5. third-party references used only to locate facts that are independently verified.

## Sources, assets, and repository licensing

Repository source code is licensed under GPL-3.0-only unless a file states otherwise.

That license does not automatically grant permission to use or redistribute Arcaea names, trademarks, official artwork, character images, song jackets, audio, chart data, story text, screenshots, promotional media, third-party wiki content, or contributor-supplied assets.

Every published media asset requires a reviewable source and permission basis. Factual availability does not grant permission to copy protected expression, presentation, prose, media, or databases.

This charter is an operating policy, not a legal opinion. Unclear cases must be escalated to the maintainer and may require removal, replacement, or non-publication.

## Unofficial fan-project position

Arcaea-Viewer is an unofficial, community-oriented fan project. It is not affiliated with, sponsored by, approved by, or endorsed by lowiro or the official Arcaea project.

Arcaea, related names, trademarks, game content, and third-party assets remain the property of their respective owners.

The project must not imply official status, describe community-maintained information as official, or publish protected material merely because it is available elsewhere.

A concise version of this statement must be visible on the public site before release.

## Corrections and takedown requests

Non-sensitive factual corrections should use the repository issue process and include the affected content, proposed correction, source, relevant version/date, and uncertainty.

Sensitive security, privacy, licensing, or takedown evidence must not be posted publicly. Before the first public release, the project must publish a private contact method on the legal or project-information page. Until then, use an available private repository-owner profile contact method.

The maintainer may temporarily remove disputed content while reviewing a request. The project does not promise an enterprise support SLA, but prioritizes exposed secrets or personal data, security risks, credible licensing claims, materially misleading facts, and then ordinary editorial corrections.

## Production workflow principles

Implementation begins from an issue or documented task with:

- an observable outcome;
- scope and non-goals;
- dependencies and ownership;
- affected files or user surfaces;
- legal, data, source, or asset constraints;
- acceptance criteria;
- a test plan;
- required evidence.

Before merge, a change must demonstrate accepted scope, acceptance evidence, relevant automated and manual checks, updated documentation, applicable provenance and risk review, recorded exceptions, and required maintainer approval.

Scope changes, blockers, rejected alternatives, risk acceptance, and known exceptions must be recorded in repository documentation, an issue, or a pull request.

## Definition of project success

Success is demonstrated by:

- useful player-facing flows;
- accurate, version-aware information;
- explicit source and asset records;
- accessible public surfaces;
- deterministic builds and validation;
- reviewable contribution workflows;
- transparent limitations;
- evidence-backed release decisions;
- sustainable maintenance cost.

Success is not the number of technologies used, infrastructure complexity, similarity to an official product, volume of redistributed content, or unsupported claims of production readiness.

## Charter changes

Changes to the mission, decision priority, governance authority, fan-project position, licensing boundaries, or public commitments require:

1. a GitHub issue or pull request;
2. rationale and affected areas;
3. identified risks and rejected alternatives;
4. consistency review against the product brief, roadmap, contribution guide, and public legal wording;
5. explicit maintainer approval.

## Related documents

- [README](../../README.md)
- [Contribution workflow](../../CONTRIBUTING.md)
- [Web MVP product brief](WEB_MVP_BRIEF.md)
- [Six-week roadmap](../roadmap/WEB_MVP_ROADMAP.md)

When documents conflict, record and resolve the conflict explicitly. The maintainer holds final authority over the accepted resolution.
