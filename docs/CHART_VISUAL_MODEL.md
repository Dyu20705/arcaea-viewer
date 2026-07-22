# Chart Visual Model

## Purpose

Checkpoint 9 extends the renderer-facing architecture:

```text
core Chart
+ timing TimingMap / TimingContext / PlaybackSnapshot
-> renderer RenderScene visual IR
-> deterministic SVG backend
```

The renderer crate owns the visual IR, scene construction, and SVG backend. It does not parse AFF source text in production code. The parser appears only as a renderer dev-dependency for the demo/example path.

## Dependency Direction

Production dependencies:

- `arcaea-viewer-core`: no renderer dependency.
- `arcaea-viewer-timing`: depends on `core`, no renderer dependency.
- `arcaea-viewer-renderer`: depends on `core` and `timing`.

Dev dependencies:

- `arcaea-viewer-renderer` dev-depends on `parser` for `examples/chart_preview.rs`.
- `arcaea-viewer-timing` dev-depends on `parser` for examples/tests.

There is no production circular dependency. Parser grammar remains outside renderer.

## Scene IR

`RenderScene` contains deterministic primitives, not raw SVG/XML:

- `Lane`
- `TimingMarker`
- `Hold`
- `Arc`
- `ArcTap`
- `Tap`
- `JudgementLine`
- `Label`

Each primitive has:

- a stable `RenderLayer`;
- a stable source/lane order key;
- typed fields such as `Lane`, `NoteId`, `NormalizedPoint`, state, clipping flags, and debug label.

Current layer order:

```text
Lanes = 10
TimingMarkers = 20
Holds = 30
Arcs = 40
ArcTaps = 45
Taps = 50
JudgementLine = 60
DebugLabels = 70
```

The SVG backend consumes this IR and serializes XML only at the backend boundary.

## Coordinate System

`NormalizedPoint.x` is horizontal playfield position:

- `0.0` is the left playfield edge.
- `1.0` is the right playfield edge.
- Ground lanes use four equal bands.
- Lane center formula: `(lane - 0.5) / 4.0`.
- Lane bounds formula for one-based `lane`: `((lane - 1) / 4.0, lane / 4.0)`.

`NormalizedPoint.y` is preview depth:

- `0.0` is the judgement line.
- `1.0` is the far horizon.
- Time increases away from the judgement line for future notes.
- SVG pixel Y decreases as normalized depth increases.

Arc coordinates:

- `ArcX` maps directly into normalized playfield X.
- `ArcY` is retained as `sky_y` metadata on each sampled arc point.
- Current SVG draws arc X against projected time/depth. It does not yet render true sky height perspective.

Viewport:

- Default viewport: `1280x720`.
- SVG converter constants for default viewport:
  - `horizon_y = 120.0`
  - `judgement_y = height - 120.0`
  - `center_x = width / 2.0`
  - `near_width = width - 240.0`
  - `far_width = near_width * 0.58`
- Pixel conversion:
  - `width_at_depth = near_width + ((far_width - near_width) * depth)`
  - `screen_x = center_x + ((x - 0.5) * width_at_depth)`
  - `screen_y = judgement_y - ((judgement_y - horizon_y) * depth)`

## Time Projection

`ProjectionConfig` defines the visible chart-time window around playback:

- Default past window: `500ms`.
- Default future window: `2000ms`.
- Arc sample steps: `16`.

Projection:

```text
delta_ms = note_time_ms - playback_time_ms
```

- If `delta_ms < -past_window_ms`: hidden past.
- If `delta_ms > future_window_ms`: hidden future.
- If `delta_ms <= 0`: visible depth is `0.0`, clipped to judgement line.
- If `delta_ms > 0`: visible depth is `delta_ms / future_window_ms`.

Playback time maps to the judgement line. The future window edge maps to the horizon. Past notes inside the past window are retained at the judgement line. Objects outside the past/future window are hidden.

This projection is a static debug preview. It is not official Arcaea scroll-speed physics.

## Interval Projection And Boundaries

Holds and arcs use interval projection:

```text
start_delta = start_time - playback_time
end_delta = end_time - playback_time
```

- Hidden when `end_delta < -past_window_ms`.
- Hidden when `start_delta > future_window_ms`.
- Otherwise start/end deltas are clamped into the projection window.
- Deltas at or before playback project to depth `0.0`.
- Positive deltas project linearly toward `1.0`.
- `clipped_at_judgement` is true when any endpoint is in the past.
- `clipped_at_horizon` is true when any endpoint is beyond the future window.

Timestamp behavior:

- Taps are `Upcoming` before their timestamp and `Passed` at their timestamp.
- Holds and arcs are `Active` for inclusive start/end boundaries.
- Zero-duration hold/arc intervals are active exactly at the boundary.
- Zero-duration visible holds render with equal start/end depth; SVG gives them a minimum visual thickness through the fixed polygon conversion.

## Arc Sampling

Arc sampling is deterministic and finite:

- sample count is `arc_sample_steps + 1` before clipping;
- each step uses `progress = step / arc_sample_steps`;
- sample time is rounded from linear progress between arc start and end time;
- samples outside the visible time window are skipped;
- empty sampled arcs are hidden;
- all sample coordinates are checked for finite values.

Supported renderer curve variants today:

- `Straight`
- `Bezier`
- `SineIn`
- `SineOut`
- `SineInOut`
- `SineOutIn`

The core crate now owns `arc_position_at` and `arc_axis_progress`. The renderer samples arcs through that shared function, and arc taps use the same function to derive their visual position from the parent arc. The current curve implementation is a simplified semantic preview. It supports the core variants available today; it does not claim full AFF/ArcCreate curve parity.

## Timing Groups

Timing groups are explicit core semantics as of Checkpoint 9:

- `TimingGroupId::ROOT` is the default/root group.
- `Chart` stores declared timing groups with root first.
- Tap, hold, arc, arc tap, and timing events carry a `TimingGroupId`.
- `TimingGroupProperties` preserves supported `noinput` and `noclip` flags.
- Unknown timing-group properties are parser errors, not ignored metadata.

Timing evaluation uses:

```text
TimingContext {
  root: TimingMap,
  groups: Vec<(TimingGroupId, TimingMap)>
}
```

`TimingMap::from_chart` remains root-only for compatibility. `TimingContext::from_chart` builds one local map per declared timing group. Duplicate timing timestamps are scoped by timing group: root `0ms` and group `0ms` can coexist, but duplicates inside one group are rejected. A declared non-root group without local timing returns an explicit timing error.

## ArcTap Model

Arc taps are separate `ChartEvent::ArcTap` values:

- each arc tap has its own stable `NoteId`;
- each arc tap stores `parent_arc_id`;
- arc taps inherit the timing group of their parent arc;
- parser validation requires the arc tap timestamp to be inside the parent arc interval, including boundaries;
- duplicate arc tap timestamps on the same parent arc are rejected deliberately;
- `PlaybackSnapshot` preserves `NoteId`, `TimingGroupId`, event kind, parent arc ID, and arc-derived position.

Renderer output includes `RenderPrimitive::ArcTap` with SVG metadata:

- `data-event-kind="arctap"`
- `data-note-id`
- `data-parent-arc-id`
- `data-timing-group-id`

Arc taps render above parent arcs through the stable `ArcTaps = 45` layer.

## Playback Visualization Semantics

`PlaybackSnapshot` from the timing crate maps core notes into:

- `Upcoming`
- `Active`
- `Passed`

Renderer mapping:

- `Upcoming`: visible when inside the future window.
- `Active`: visible interval contains playback time, clipped at judgement as needed.
- `Passed`: visible only if inside the past window; otherwise hidden.
- Hidden/outside viewport: no note primitive is emitted and `hidden_notes` increments.

The judgement line has `data-playback-cursor="true"` in SVG because it is the static preview cursor for the current playback timestamp.

## SVG Backend

The SVG backend renders:

- playfield surface;
- horizon;
- four lane polygons;
- timing marker lines;
- hold polygons;
- arc polylines;
- arc tap circles;
- tap rectangles;
- judgement line/playback cursor;
- debug labels and summary.

Required debug metadata:

- `data-note-id` attributes on note primitives;
- `data-event-kind` on note primitives;
- `data-timing-group-id` on note primitives;
- `data-parent-arc-id` on arc tap primitives;
- visible labels containing `NoteId=...`;
- `data-layer`;
- state and clipping flags where applicable.

## Supported AFF Elements End-To-End

Supported from fixture through parser, core, timing, renderer scene, and SVG preview:

- `timing(time,bpm,beats_per_measure);`
- `(time,lane);`
- `hold(start,end,lane);`
- `arc(start,end,x1,x2,curve,y1,y2,color,fx,is_trace);`
- `arc(...)[arctap(time),...];`
- `timinggroup(noinput,noclip){ ... };`

Supported arc curve tokens in the parser:

- `s`
- `b`
- `si`
- `so`
- `sisi`
- `soso`

Supported arc colors:

- `0` blue;
- `1` red;
- `2` green.

Trace arcs:

- `is_trace` is preserved in core and renderer metadata.
- The SVG preview does not implement distinct trace gameplay behavior beyond metadata/styling hooks.

## Unsupported Semantics

Not parsed:

- includes/fragments;
- cameras;
- scene controls;
- special effects;
- no-input/void notes;
- unsupported arc curve tokens outside the current subset.

Parsed but not fully rendered:

- arc `fx` field is accepted positionally by the current AFF subset but is not represented in core semantics.
- trace arcs preserve `is_trace`; dedicated trace rendering is deferred.
- timing-group `noinput` and `noclip` are preserved as metadata but do not change gameplay judgement or clipping behavior in this static preview.

Handled as validation/error cases:

- malformed syntax returns parser diagnostics;
- invalid lanes return domain diagnostics;
- invalid hold/arc intervals return domain diagnostics;
- invalid arc coordinates return domain diagnostics;
- duplicate timing timestamps are rejected by `TimingMap`;
- renderer invalid viewport/window/sampling config returns structured `RenderError`.

Out of scope for Checkpoint 9:

- WebGL;
- WASM UI;
- audio synchronization;
- animation;
- official scroll-speed physics;
- camera and scenecontrol;
- gameplay judgement.

## Test Coverage Map

- deterministic coordinate conversion: renderer tests for projection determinism, lane centers, lane bounds.
- tap placement: `tap_primitive_includes_note_id_and_lane_position`, fixture-style mixed scene SVG test.
- hold interval geometry: hold projection/clipping tests, zero-duration hold test.
- arc endpoint projection: straight arc endpoint test.
- arc interpolation: all supported curve variants generate finite samples.
- playback-state-to-visual-state: timing tests for tap/hold/arc states and renderer state tests.
- stable scene/SVG output: deterministic scene ordering, SVG structure, generated SVG non-empty.
- boundary timestamps: projection playback/future/past boundary tests and timing state boundary tests.
- duplicate timestamps: timing crate rejects duplicate timing events.
- zero-duration interval: timing and renderer tests cover zero-duration hold; arc constructor allows equal endpoints and renderer sampling handles finite points.
- exact playback timestamp: projection maps playback to judgement line; hold/arc boundaries are inclusive.
- viewport clipping: projection hides outside windows and clips intervals at judgement/horizon.
- malformed/unsupported visual input: parser tests malformed and unsupported AFF; renderer tests invalid config/output path structured errors.
- stable `NoteId`: parser note-id tests and SVG `NoteId=...` labels.

## Technique Overlay Decision

Decision for Checkpoint 8: design-only deferred.

Future model:

```text
Chart
+ TimingMap
-> optional TechniqueAnalyzer
-> TechniqueOverlay / annotations keyed by NoteId
-> ChartVisualScene / RenderScene
-> SVG or future web renderer
```

Suggested annotation fields:

- `SuggestedHand`: `Left`, `Right`, `Either`, `Ambiguous`, `Unknown`.
- `confidence`: numeric confidence, not correctness.
- `reason`: concise evidence.
- `simultaneous_action_group`: group identifier for near-simultaneous notes.
- `movement_segment`: sequence/segment identifier.
- `crossover_warning`: optional heuristic flag.
- overlay enable/disable flag at renderer/backend options.

Inferred data must not be stored in `core::Chart`. The renderer can consume overlays by `NoteId` and produce additional label/overlay primitives later.
