# ArcCreate Chart Visual Research

## 1. ArcCreate State Inspected

- Location: `C:\Users\Admin\Src\Cod\arcaea-viewer\ArcCreate`
- Search paths checked: workspace root and parent directory.
- Revision inspected: `34657dc4b4f22894e396cedf55f476743029714b`
- Describe output: `34657dc4`
- License file: `ArcCreate/LICENSE`
- License: GNU GPL version 3.
- Audit date: 2026-06-19, local read-only source audit.
- Scope: chart representation, timing/playback, note visibility, coordinate conversion, arc interpolation, render ordering, and editor preview concepts.

No files under `ArcCreate/` were edited. The folder was used as a read-only conceptual reference.

## 2. Relevant Files

Chart representation:

- `Assets/Scripts/Gameplay/Data/ArcChart.cs`: converts raw chart events into gameplay chart data and distributes events into timing groups.
- `Assets/Scripts/Gameplay/Data/ChartTimingGroup.cs`: stores per-group lists for taps, holds, arcs, arc taps, timings, and reference events.
- `Assets/Scripts/Gameplay/Data/GroupProperties.cs`: stores timing-group flags such as visibility, no-input, no-clip, arc resolution, drop rate, judgement transforms, and per-group render transforms.
- `Assets/Scripts/Gameplay/Data/Events/Note.cs`: base note lifecycle; rebuilds floor position from timing.
- `Assets/Scripts/Gameplay/Data/Events/LongNote.cs`: interval note base with end timing and end floor position.
- `Assets/Scripts/Gameplay/Data/Events/Tap.cs`: ground tap data, lane mapping, render transform, judgement request.
- `Assets/Scripts/Gameplay/Data/Events/Hold.cs`: ground hold interval, clipping at judgement, hold geometry transform, long-note judgement cadence.
- `Assets/Scripts/Gameplay/Data/Events/Arc.Rendering.cs`: arc endpoints, line type, color, trace flag, segmented rendering, sampled world positions.
- `Assets/Scripts/Gameplay/Data/Events/ArcTap.cs`: arc tap as a separate note linked to a parent arc and positioned by sampling that arc.
- `Assets/Scripts/Gameplay/Data/Events/ArcSegmentData.cs`: per-arc segment timing/floor-position and transform data.
- `Assets/Scripts/Gameplay/Data/Events/TimingEvent.cs`: timing event with BPM/divisor and computed floor position.
- `Assets/Scripts/Gameplay/Data/Events/CameraEvent.cs`: camera event representation, out of scope for this checkpoint.
- `Assets/Scripts/Gameplay/Data/Events/ScenecontrolEvent.cs`: scene-control event representation, out of scope for this checkpoint.
- `Assets/Scripts/Gameplay/Data/ArcLineType.cs` and `ArcLineTypeConvert.cs`: AFF curve-token representation including `s`, `b`, `si`, `so`, `sisi`, `siso`, `sosi`, and `soso`.

Timing/playback:

- `Assets/Scripts/Gameplay/Chart/ChartService.cs`: owns runtime timing groups and drives judgement/render updates for all groups.
- `Assets/Scripts/Gameplay/Chart/TimingGroup.Dispatch.cs`: owns note groups, group visibility, update dispatch, add/remove/update events.
- `Assets/Scripts/Gameplay/Chart/TimingGroup.Timing.cs`: maps chart timing to floor position and back.
- `Assets/Scripts/Gameplay/Chart/NoteGroup/ShortNoteGroup.cs`: indexed visibility and judgement for taps and arc taps.
- `Assets/Scripts/Gameplay/Chart/NoteGroup/LongNoteGroup.cs`: interval range-tree visibility for holds and arcs.

Coordinate/projection/rendering:

- `Assets/Scripts/Gameplay/Utility/Values.cs`: playfield constants such as lane width, track forward/backward lengths, arc Y range, and fade length.
- `Assets/Scripts/Gameplay/Utility/ArcFormula.cs`: lane, arc X/Y, floor-position/Z, fade, size, and curve interpolation formulas.
- `Assets/Scripts/Gameplay/Utility/Projection.cs`: editor ray projection onto track and vertical input plane.
- `Assets/Scripts/Gameplay/Render/IRenderService.cs`: concrete draw-call boundary for gameplay rendering.
- `Assets/Scripts/Gameplay/Render/RenderService.cs`: Unity instanced renderer pools and queued arc/trace draw calls.
- `Assets/Scripts/Gameplay/Render/Properties/ArcDrawCall.cs`: queued arc draw-call record with depth.
- `Assets/Scripts/Gameplay/Render/Properties/ArcDrawCallComparer.cs`: arc draw ordering by depth.

Editor preview:

- `Assets/Scripts/Compose/Components/GameplayViewport.cs`: maps editor UI viewport to gameplay camera viewport.
- `Assets/Scripts/Compose/Grid/Timing/EditorBeatlineGenerator.cs`: editor timing-grid generation from timing group floor positions.
- `Assets/Scripts/Compose/Grid/Timing/TimingGrid.cs`: editor beatline display and snapping.
- `Assets/Scripts/Compose/Grid/Vertical/VerticalGrid.cs`: sky/input-plane grid and snapping.
- `Assets/Scripts/Compose/Editing/NoteCreation.cs`: editor note creation and live preview for tap, hold, arc, trace, and arc tap.
- `Assets/Scripts/Compose/Selection/NoteRaycaster.cs`: editor selection raycast over currently rendering notes.

## 3. ArcCreate Pipeline

Observed source pipeline:

```text
AFF/raw chart source
-> RawEvent / RawTimingGroup records
-> ArcChart conversion
-> ChartTimingGroup lists
-> runtime TimingGroup note groups
-> timing events recalculate floor positions
-> note groups select visible notes by floor-position/timing windows
-> note UpdateRender builds Unity transforms
-> RenderService queues/batches draw calls
-> Unity renderer submits mesh/material instances
```

Checkpoints 8 and 9 adopted the pipeline shape, not ArcCreate's Unity implementation:

```text
AFF fixture
-> parser parse_chart
-> core Chart
-> timing TimingContext / PlaybackSnapshot
-> renderer RenderScene visual IR
-> deterministic SVG backend
```

## 4. Coordinate And Timing Concepts

ArcCreate timing:

- `TimingGroup.GetEventAt(timing)` chooses the active timing event by sorted timing list.
- `GetFloorPosition(timing)` returns `event.floor_position + event.bpm * (timing - event.timing)`.
- `RecalculateFloorPosition` accumulates floor position between adjacent timing events.
- `GetFloorPositionFromCurrent(timing)` subtracts current playback floor position from target floor position.
- Note visibility windows are selected in floor-position space, not directly in milliseconds.

ArcCreate depth and visibility:

- `ArcFormula.FloorPositionToZ(fp, drop_rate)` computes `fp / BaseBpm * drop_rate / -1000`.
- `Values.TrackLengthForward = 100` and `Values.TrackLengthBackward = 53.5`.
- `WithinRenderRange(z)` checks `-TrackLengthForward <= z <= TrackLengthBackward`.
- `ShortNoteGroup.UpdateRender` converts forward/backward track length into floor-position windows and selects taps/arctaps from indexes.
- `LongNoteGroup.UpdateRender` does the same for interval notes using range trees.
- `GroupProperties.Visible` can suppress rendering for a whole group.
- `NoInput` and `NoClip` alter clipping and judgement behavior.

ArcCreate coordinate mapping:

- Arc X: `ArcXToWorld(x) = (-LaneWidth * 2 * x) + LaneWidth`.
- Arc Y: `ArcYToWorld(y) = ArcY0 + ((ArcY1 - ArcY0) * y)`.
- Lane X: `LaneToWorldX(lane) = (-LaneWidth * lane) + (LaneWidth * 2.5)`.
- Lane-to-arc helper: `LaneToArcX(lane) = (0.5 * lane) - 0.75`.
- Editor ray projection casts camera rays both onto the vertical input plane (`z = 0`) and the ground track plane (`y = 0`).

ArcCreate arc interpolation:

- `S` is linear interpolation.
- `I` uses sine-in style interpolation.
- `O` uses sine-out style interpolation.
- `B` is a cubic Bezier-like formula using endpoint values as both controls.
- `X(start,end,t,type)` and `Y(start,end,t,type)` select different interpolation functions depending on `ArcLineType`.
- `Arc.Rendering.cs` exposes `WorldXAt`, `WorldYAt`, `ArcXAt`, and `ArcYAt`, clamping progress to `0..=1` and handling zero-duration arcs by selecting start or end based on timestamp boundary.

ArcCreate render ordering:

- `RenderService` batches tap, hold, connection-line, arc, trace, arc-tap, shadow, height-indicator, head, and cap draw calls.
- Arc and trace segment calls are queued, sorted with `ArcDrawCallComparer`, and then flushed.
- The ordering is backend/lifecycle-specific and was not ported, but it confirms that visual draw order should be explicit.

## 5. Ideas Adopted

- Keep parser/domain data separate from renderer-facing visual data.
- Let timing/playback state be computed before scene construction.
- Treat coordinate conversion as named, testable functions.
- Separate short-note visibility from interval-note visibility.
- Preserve stable note identifiers through visual labels and backend output.
- Use stable explicit layers instead of relying on source order alone.
- Treat timing groups as chart semantics with local timing maps, not renderer metadata.
- Treat arc taps as real related notes linked to parent arcs, not as decorations.
- Keep editor/analysis annotations as optional overlays.

## 6. Ideas Rejected

- Unity `MonoBehaviour` lifecycle, scene graph, instanced renderer pools, mesh transforms, materials, skins, particles, input, score, and judgement services.
- Direct floor-position/drop-rate implementation in Checkpoint 8, because current timing crate does not yet model timing groups, scroll speed, or scene controls.
- ArcCreate camera and scene-control systems.
- ArcCreate mutable global service architecture.
- ArcCreate render batching and material pooling for the SVG backend.
- Direct porting of editor raycasting, selection, macros, and note creation state.
- Any claim that hand assignments are correct or optimal; future hand guidance must remain heuristic and derived.

## 7. License And Attribution

ArcCreate is GPL-3.0. Arcaea-Viewer is GPL-3.0-only, so the license family is compatible, but this checkpoint still avoids source-code copying.

What was used:

- Conceptual architecture: chart data -> timing/playback -> visual selection -> concrete rendering.
- Conceptual separation of short notes and long notes.
- Conceptual confirmation that arc taps should be modeled explicitly later.
- Conceptual confirmation that render order must be stable.

What was independently implemented:

- The Rust `RenderScene` IR.
- The SVG trapezoid projection.
- `ProjectionConfig` past/future chart-time window.
- Lane center/bounds formulas in normalized `0..=1` space.
- Deterministic SVG serialization.
- The current simplified arc sampler.

No ArcCreate source file was imported into Arcaea-Viewer. No ArcCreate C# code was directly adapted line-by-line. If future checkpoints adopt floor-position/drop-rate formulas more closely, that work should be called out explicitly in the relevant design doc and attribution notes.

## 8. Architecture Mapping

| ArcCreate concept | Arcaea-Viewer equivalent | Decision | Reason |
| --- | --- | --- | --- |
| `RawEvent` / `RawTimingGroup` | parser syntax events and fixture AFF | adapted | Parser owns source-text grammar. |
| `ArcChart` | `parse_chart` -> `core::Chart` | adapted | Core chart is parser-independent. |
| `ChartTimingGroup` | `TimingGroupId`, `TimingGroup`, `TimingContext` | adapted for CP9 | Groups are explicit event namespaces with local timing maps. |
| `TimingGroup.GetFloorPosition` | `TimingMap` + simple projection | partially adapted | Current preview remains time-window based, but local timing maps are group-scoped. |
| `Playback` services | `PlaybackSnapshot` | adapted | Timing crate owns state without audio. |
| `ShortNoteGroup` | tap projection path | adapted | Short notes use timestamp visibility. |
| `LongNoteGroup` | hold/arc interval projection | adapted | Interval geometry is clipped by window. |
| `ArcFormula.LaneToWorldX` | `lane_center_x`, `lane_bounds_x` | adapted | Current renderer uses normalized coordinates. |
| `ArcFormula.ArcXToWorld/YToWorld` | arc X retained as normalized X; sky Y retained as metadata | adapted | SVG preview is 2D debug projection. |
| `ArcFormula.X/Y` | renderer `ease_curve` | adapted | Supports current core curve variants only. |
| `ArcSegmentData` | `ArcSamplePoint` | adapted | SVG uses deterministic finite polyline samples. |
| `RenderService` | `render_scene_to_svg` backend | adapted | Backend consumes prepared scene IR. |
| `ArcDrawCallComparer` | `RenderLayer` + stable order | adapted | Stable ordering is backend-independent. |
| Arc taps | `ArcTapNote`, snapshot parent link, `RenderPrimitive::ArcTap` | adapted for CP9 | Parent arc identity and arc-derived position are modeled explicitly. |
| Camera/scenecontrol | unsupported | rejected for CP8 | Out of scope and needs broader semantics. |
| Editor note creation/raycast | future editor UI | rejected for CP8 | This checkpoint is static preview only. |
| Group properties no-input/no-clip | `TimingGroupProperties` metadata | partially adapted | Flags are preserved and unknown properties reject; gameplay effects are deferred. |

## 9. Open Questions

- Should Checkpoint 10 introduce floor-position/drop-rate projection, or first broaden core AFF semantics further?
- Should timing-group properties beyond `noinput`/`noclip` become typed core semantics or parser diagnostics?
- Should future AFF import preserve raw group property text for unsupported but non-fatal editor workflows, or keep strict parser rejection?
- How many ArcCreate curve tokens should core represent before the renderer claims full AFF arc interpolation?
- Which annotations belong in analytics overlays versus renderer scene primitives?

## 10. Hand-Guidance Decision

Decision for Checkpoint 8: design-only deferred.

The current visual IR can accept future annotations because every visible note primitive carries a stable `NoteId`, and labels/overlays are already independent renderer primitives. No inferred technique data is stored in `core::Chart`.

Suggested future model:

```text
Chart
+ TimingMap
-> optional TechniqueAnalyzer
-> TechniqueOverlay / annotations keyed by NoteId
-> RenderScene or successor ChartVisualScene
-> SVG / future web renderer
```

Vocabulary:

- `SuggestedHand`: `Left`, `Right`, `Either`, `Ambiguous`, `Unknown`.
- `confidence`: bounded numeric score, not a correctness claim.
- `reason`: short evidence such as lane side, simultaneous group, preceding motion, or arc continuity.
- `simultaneous_action_group`: groups notes at or near the same timestamp.
- `movement_segment`: links a sequence of note IDs used for motion reasoning.
- `crossover_warning`: optional heuristic warning.
- overlay enable/disable: renderer option, not core chart mutation.

The words "correct", "optimal", and "best technique" should not be used for this feature. Use "heuristic", "suggested", "inferred", "ambiguous", and "confidence".
