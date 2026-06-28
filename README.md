# Arcaea Viewer

Arcaea-Viewer is currently a research-grade chart parsing and preview prototype. The implemented vertical slice parses supported AFF fixtures in Rust, builds timing and renderer scene data in Rust, exposes a versioned JSON contract through WebAssembly, and displays a deterministic SVG chart preview in a small React/Vite app.

## Browser Vertical Slice

Checkpoint 10 supports:

- loading the legal fixture `fixtures/checkpoint9_mixed.aff`;
- parsing AFF through `arcaea-viewer-parser` compiled to WASM;
- returning structured diagnostics for malformed AFF;
- selecting playback time in milliseconds;
- building a Rust timing context and renderer scene;
- rendering the scene to SVG through `arcaea-viewer-renderer`;
- showing primitive counts, visible taps, holds, arcs, arc taps, and hidden notes.

This is a proof of integration, not the final product UI.

## Architecture

- `crates/core`: browser-independent chart domain types.
- `crates/parser`: supported AFF subset parser and diagnostics.
- `crates/timing`: timing maps and playback snapshots.
- `crates/renderer`: renderer scene IR and deterministic SVG backend.
- `crates/wasm`: JSON DTO/envelope boundary for browser calls.
- `apps/web`: React/Vite app that calls WASM and mounts Rust-generated SVG.

Parser, timing, and renderer logic are not reimplemented in TypeScript.

## Prerequisites

- Rust toolchain with the `wasm32-unknown-unknown` target.
- Node.js 22 or newer.
- pnpm 10.32.1 or newer.
- `wasm-pack` is installed as a root dev dependency by `pnpm install`.

## Install

```powershell
pnpm install
```

## Run Web App

```powershell
pnpm dev
```

The command builds `crates/wasm/pkg` with `wasm-pack` and starts the Vite dev server for `apps/web`.

## Test And Check

```powershell
cargo test --workspace
pnpm run wasm:build
pnpm test
pnpm check
```

`pnpm check` runs Rust formatting, clippy, Rust tests, WASM build, frontend type check, frontend unit tests, and frontend production build.

## Production Build

```powershell
pnpm build
```

The web build emits the WASM binary and app assets into `apps/web/dist`.

## Fixtures And Data

Current tests and demos use hand-written fixtures under `fixtures/`. The project does not require copyrighted game assets for core tests or the browser vertical slice.

## Known Limitations

- The AFF parser supports only the subset documented in `docs/CHART_VISUAL_MODEL.md`.
- The SVG renderer is a deterministic debug preview, not official Arcaea scroll physics.
- `debugLabels` is accepted at the WASM boundary; the current SVG backend always includes debug labels.
- No replay, audio sync, metadata explorer, offline cache, analytics UI, backend, account system, or copyrighted dataset is included.

## Roadmap

Recommended next checkpoint: improve the browser viewer ergonomics around fixture selection, malformed fixture examples, and render request controls before starting analytics or replay work.

## License

GPL-3.0-only. See `LICENSE`.
