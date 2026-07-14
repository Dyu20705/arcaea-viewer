# Six-Week Web MVP Roadmap

## Capacity

The plan assumes approximately three focused hours per day for six weeks. Scope is intentionally limited to a public, desktop-first wiki MVP.

## Process

The project uses a stage-gated hybrid:

- waterfall-like decision gates keep product, legal, data, and design work ordered;
- weekly vertical increments keep the product testable;
- each phase ends with evidence before the next phase becomes the priority.

## Week 1 — Discovery and design gate

Outputs:

- charter, PRD, sitemap, content taxonomy, and non-goals;
- source and asset policy;
- static data architecture decision;
- reference research and UI direction;
- canonical issue/dependency map.

Gate: a reviewer can explain the MVP, its data sources, legal boundaries, routes, user flows, and visual direction without reading the implementation.

## Week 2 — Web foundation

Outputs:

- Rsbuild/Rspack migration plan and implementation;
- Tailwind CSS v4 foundation;
- routing, layout, theme, settings, design tokens;
- schema validation and static catalog loader;
- quality gate and initial security baseline;
- licensed-asset pipeline design.

Gate: a production build renders the app shell, themes, routes, sample data, and error states with no runtime viewer exposed.

## Week 3 — Core wiki product

Outputs:

- homepage with latest release/event content and project/game introduction;
- explore search, sort, filters, and shareable URL state;
- song and pack detail surfaces;
- related-entity navigation and source/status presentation.

Gate: a player can discover and open useful song information through a coherent public flow.

## Week 4 — Game encyclopedia

Outputs:

- partner/character pages;
- story index and story content structure;
- World Mode, Course Mode, achievements, and game-topic pages;
- information/wiki hub;
- per-game-version content update workflow.

Gate: the MVP covers the agreed wiki categories with consistent templates and provenance.

## Week 5 — Production quality

Outputs:

- accessibility and responsive audit;
- performance and image optimization;
- PWA/offline strategy for static catalog;
- SEO, metadata, structured data, and social previews;
- security, dependency, and supply-chain review;
- privacy-respecting error reporting decision.

Gate: automated and manual evidence meets the release budgets.

## Week 6 — Public preview

Outputs:

- content accuracy and legal audit;
- end-to-end acceptance tests;
- GitHub Pages deployment;
- release checklist and rollback notes;
- contribution guide and issue templates;
- public-preview readiness report.

Gate: the release checklist is complete and every known exception is documented.

## Beyond the MVP

The roadmap retains post-MVP issues for runtime, chart rendering, analytics, replay, larger datasets, and optional backend work. They are not allowed to block the wiki release.

Future work should start with community content operations and localization, then add richer search and history. Runtime and personal-analysis features return only after product, legal, privacy, and maintenance reviews.
