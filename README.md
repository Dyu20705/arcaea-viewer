# Arcaea-Viewer

Arcaea-Viewer is an unofficial, community-oriented Arcaea fan project.

The current product direction is **Web MVP 0.1**: a fast, accessible, source-aware, static wiki for ordinary Arcaea players. The repository also preserves an existing Rust/WebAssembly chart parsing and rendering prototype, but that runtime work is not the public MVP focus.

> Arcaea-Viewer is not affiliated with, sponsored by, approved by, or endorsed by lowiro or the official Arcaea project.

## Product direction

The Web MVP should let a visitor:

- understand the project and current game highlights;
- search and filter a versioned catalog;
- open useful song, pack, partner, story, and game-topic pages;
- navigate between related entities;
- see source, review, uncertainty, and update status;
- use keyboard-friendly navigation and supported light/dark themes;
- receive clear loading, empty, unavailable, error, and not-found states.

The MVP is intentionally static and does **not** include:

- user accounts or application-hosted user-generated content;
- a production backend or database server;
- chart upload, editing, playback, analytics, or replay;
- audio previews;
- automatic scraping of third-party wikis;
- redistribution of unlicensed game assets;
- infrastructure added only to appear production-grade.

The planned public catalog uses static, versioned data with schemas, validation, generated indexes, and explicit provenance.

Governing documents:

- [Project charter](docs/product/PROJECT_CHARTER.md)
- [Web MVP product brief](docs/product/WEB_MVP_BRIEF.md)
- [Six-week Web MVP roadmap](docs/roadmap/WEB_MVP_ROADMAP.md)
- [Contribution workflow](CONTRIBUTING.md)
- [Workflow validation](docs/process/WORKFLOW_VALIDATION.md)

## Current implementation

The repository currently contains a research-grade chart parsing and preview vertical slice. It:

- loads legal, hand-written AFF fixtures;
- parses the supported AFF subset in Rust;
- reports structured diagnostics for malformed input;
- builds timing and renderer scene data in Rust;
- exposes a versioned JSON boundary through WebAssembly;
- renders a deterministic SVG preview in React.

This is proof of technical integration, not the final public wiki interface. The runtime work remains independently testable and available for post-MVP evaluation without blocking the wiki release.

## Repository structure

- `crates/core`: browser-independent chart domain types.
- `crates/parser`: supported AFF parser and diagnostics.
- `crates/timing`: timing maps and playback snapshots.
- `crates/renderer`: renderer scene IR and deterministic SVG backend.
- `crates/wasm`: versioned JSON DTO/envelope boundary.
- `apps/web`: current React/Vite application.
- `roadmap/`: declarative labels, milestones, issues, parents, and dependencies.
- `docs/product/`: product brief and project charter.
- `docs/process/`: reusable workflow evidence.
- `docs/roadmap/`: stage-gated MVP delivery plan.

Parser, timing, and renderer domain logic must not be reimplemented independently in TypeScript without an approved architectural decision.

## Prerequisites

- Git.
- Rust toolchain with the `wasm32-unknown-unknown` target.
- Node.js 22 or newer.
- pnpm 10.32.1.
- Bash, authenticated GitHub CLI, `jq`, and GNU `date` for roadmap automation.

On Windows, the application workflow can run through PowerShell. WSL or another compatible Bash environment is recommended for roadmap automation.

## Install

```bash
git clone https://github.com/Dyu20705/arcaea-viewer.git
cd arcaea-viewer
rustup target add wasm32-unknown-unknown
pnpm install --frozen-lockfile
```

## Run the current web application

```bash
pnpm dev
```

This builds `crates/wasm/pkg` and starts the current frontend development server.

## Build and validation

```bash
pnpm build
pnpm check
```

`pnpm check` runs:

- Rust formatting;
- Clippy with warnings denied;
- Rust workspace tests;
- the WebAssembly build;
- frontend type checking;
- frontend tests;
- the frontend production build.

Roadmap-specific validation:

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

A successful exit code is necessary but not sufficient for the live dry-run. Review the printed plan for unexpected issue, milestone, label, parent, dependency, assignee, or closure changes.

## Fixtures and data

Current runtime tests use hand-written fixtures under `fixtures/`. Core tests do not require copyrighted game assets.

Official chart files, audio, proprietary game resources, copied third-party wiki prose, and unlicensed media must not be committed as ordinary project data.

## Current limitations

- The public wiki product flow is not implemented yet.
- The current web screen remains a chart-viewer debug prototype.
- The AFF parser supports only the documented subset.
- The SVG renderer is a deterministic technical preview, not official Arcaea scroll physics.
- No production backend, user account system, complete public metadata catalog, replay, audio synchronization, or public analytics is included.
- Planned features must not be described as completed functionality.

## Roadmap

The current six-week sequence is:

1. product, data, legal, workflow, and UI discovery;
2. web foundation and validated static catalog;
3. homepage, explore, song, and pack flows;
4. partner, story, and game-topic encyclopedia surfaces;
5. accessibility, performance, SEO, security, and offline quality;
6. public preview, content audit, deployment, rollback, and community readiness.

Runtime viewer, analytics, replay, production backend, and larger platform work remain outside the MVP critical path.

## Contributing

Contributions are welcome when they are scoped, evidence-backed, and consistent with the wiki-first roadmap. Read [CONTRIBUTING.md](CONTRIBUTING.md) before starting significant work.

Do not submit secrets, private account information, proprietary game files, copied third-party prose, unlicensed media, or fabricated facts, permissions, sources, and test results.

## Corrections, security, and takedown requests

Non-sensitive factual corrections may be proposed through GitHub issues.

Do not publish sensitive security, privacy, licensing, or takedown evidence in a public issue. Use an available private contact method listed by the repository owner until a dedicated project contact channel is published.

## License

Repository source code is licensed under GPL-3.0-only. See [LICENSE](LICENSE).

The repository license does not automatically grant permission to redistribute Arcaea artwork, audio, charts, story text, screenshots, trademarks, or other third-party material.
