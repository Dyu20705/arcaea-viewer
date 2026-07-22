# Arcaea-Viewer planning index

The previous broad platform master plan is superseded. It assumed accounts, hosted services, replay, analytics, large-scale ingestion, and other infrastructure before the project had validated a smaller public product. Keeping those assumptions as an active plan would conflict with the repository's current wiki-first direction.

## Current authoritative direction

Use these documents as the source of truth:

- [Project charter](product/PROJECT_CHARTER.md): mission, values, responsibility boundaries, and community promise.
- [Web MVP product brief](product/WEB_MVP_BRIEF.md): target users, product boundaries, routes, and explicit non-goals.
- [Six-week Web MVP roadmap](roadmap/WEB_MVP_ROADMAP.md): stage gates from discovery through public preview.
- [Roadmap automation guide](../roadmap/README.md): canonical managed-issue manifests, execution guidance, and reconciliation safety.
- [Contribution workflow](../CONTRIBUTING.md): readiness, implementation, review, evidence, and merge expectations.

The active delivery system is also represented by the managed GitHub roadmap rooted at issue #34.

## Scope boundary

The current public MVP is a static, versioned, source-aware Arcaea fan wiki. It does not include a production backend, user accounts, chart upload or playback, analytics, replay collection, audio previews, automatic third-party scraping, or redistribution of unlicensed game assets.

The existing Rust/WebAssembly parser and renderer prototype remains preserved and independently testable. Runtime, analytics, replay, larger datasets, and hosted platform work require separate post-MVP decisions backed by measured user value, legal inputs, privacy/security review, sustainable operations, a static fallback, and rollback evidence.

## Historical material

Superseded planning drafts remain recoverable through Git history. Do not copy them back into active documentation without reconciling them against the charter, product brief, six-week roadmap, managed issues, and current repository evidence.
