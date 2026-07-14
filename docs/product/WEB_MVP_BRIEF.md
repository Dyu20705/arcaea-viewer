# Arcaea-Viewer Web MVP 0.1 Product Brief

## Product intent

Arcaea-Viewer is an unofficial, community-oriented public fan database created from genuine enthusiasm for Arcaea. The project must be useful to ordinary players first, credible to contributors, and strong enough to demonstrate production-minded software engineering.

The MVP is a fast, accessible, image-conscious wiki experience. It is not a chart runtime, replay tool, analytics platform, backend platform, or infrastructure showcase.

## Primary audience

Ordinary Arcaea players who want to discover and look up:

- songs, artists, packs, difficulties, chart constants, BPM, and note counts;
- partners and characters;
- story and world information;
- World Mode, Course Mode, achievements, and other game elements;
- current release and event information.

## MVP promise

A visitor can land on the homepage, understand the project and current game highlights, search or browse the catalog, open a high-quality content page, navigate between related entities, switch between light and dark themes, and use the site comfortably on desktop.

The first public preview should feel more focused and efficient than a conventional wiki:

- strong information architecture;
- fast search and filtering;
- clear provenance and update status;
- high-quality responsive images where use is permitted;
- accessible, keyboard-friendly navigation;
- minimal dependencies and strong performance budgets;
- security and legal constraints designed in, not added later.

## MVP routes

- `/`
- `/explore`
- `/songs/:songId`
- `/packs/:packId`
- `/partners/:partnerId`
- `/story`
- `/game/:topic`
- `/information`
- `/wiki`
- `/about`
- `/legal`
- `/settings`
- `*` / 404

Sorting and filters belong in URL query parameters rather than in route path syntax. Example:

```text
/explore?sort=level&pack=absolute-reason&difficulty=future
```

## Explicit MVP exclusions

The existing Rust/WASM parser, timing, renderer, and analytics work stays preserved in the repository but is not exposed in the public MVP navigation.

The MVP excludes:

- chart upload, AFF editing, chart playback, chart analytics, and replay;
- audio previews;
- user accounts and user-generated content;
- a production backend, database server, microservices, or Kubernetes;
- automatic scraping of third-party wikis;
- unlicensed redistribution of game assets;
- community moderation tooling beyond contribution documentation.

## Technology direction

Keep React, TypeScript, Rust/WASM, and the existing monorepo boundaries.

For the public web surface:

- migrate Vite to Rsbuild/Rspack;
- use Tailwind CSS v4;
- keep dependencies minimal;
- use static, versioned JSON with schemas and validation;
- add PWA/offline support only for the public catalog and app shell;
- deploy the first preview to GitHub Pages;
- keep the runtime crates available but outside the MVP bundle and navigation.

## Delivery model

Use a six-week, stage-gated hybrid process:

1. discovery and design decisions;
2. web foundation;
3. core wiki product;
4. game encyclopedia surfaces;
5. production quality pass;
6. public preview release.

Each stage has a visible acceptance gate, but implementation remains incremental inside the stage. This preserves the speed of a waterfall-style plan without postponing validation until the end.

## Production credibility

Production quality for this MVP means:

- reproducible builds and deterministic content generation;
- explicit legal and provenance records;
- schema-validated data;
- least-privilege GitHub Actions;
- accessibility acceptance criteria;
- performance and image budgets;
- responsive and empty/error/loading states;
- security headers and safe content rendering where hosting permits;
- automated checks and release evidence;
- contribution guidance and a maintainable update workflow.

It does not mean unnecessary infrastructure.

## Community principle

The project should invite contributions without pretending one maintainer can make every page perfect. Content, design, accessibility, localization, and data corrections should be easy to propose through documented pull-request workflows.

## Post-MVP path

After the wiki MVP is stable, the project can grow in this order:

1. broader, versioned community-maintained content;
2. richer search, cross-linking, and content history;
3. localization;
4. optional hosted data/search services when static delivery is proven insufficient;
5. chart viewer and deterministic analytics surfaces;
6. local-first replay and personal analysis;
7. progression, recommendation, lore graph, and research features.

Each expansion must pass a separate product, legal, privacy, performance, and maintenance decision.
