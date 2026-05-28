# Phase 0.4 — Rendering Architecture

## 1. Mục Tiêu

Tài liệu này chốt renderer, timing model, interaction model, coordinate system, replay clock, render layers, performance constraints, và deterministic rendering guarantees cho Phase 0.

Mục tiêu không phải là xây engine hoàn chỉnh.
Mục tiêu là có một rendering contract đủ rõ để chart có thể được load, normalize, render, inspect, và replay overlay sau này mà không phải đập đi làm lại.

## 2. Renderer Choice

### 2.1 Chọn Gì

Phase 0 nên dùng một renderer 2D nhẹ, ưu tiên `Canvas2D` hoặc một abstraction 2D mỏng trên nền `PixiJS` nếu team cần batching tốt hơn.

Lý do:

- đủ cho lane/timeline visualization
- nhanh để iterate
- dễ debug
- ít khởi động hơn custom GPU engine
- phù hợp với vertical slice research-grade

### 2.2 Không Chọn Gì

Phase 0 không nên tự xây:

- custom WebGL engine
- custom WebGPU pipeline
- complex scene graph runtime
- multi-pass shader ecosystem
- animation architecture phụ thuộc vào frame luck

Nếu sau này chart visualization chứng minh cần GPU-heavy rendering, lúc đó mới mở rộng.

## 3. Timing Model

### 3.1 Authoritative Clock

Chart render và replay overlay phải đọc từ một authoritative clock duy nhất.

Clock này là nguồn sự thật cho:

- lane position
- note visibility window
- hold progress
- marker alignment
- replay timing offset
- debug scrubber state

### 3.2 Time Domains

Phải phân biệt rõ:

- source time: thời gian từ chart/replay nguồn
- normalized time: thời gian sau khi parser và normalization đã chuẩn hóa
- render time: thời gian dùng để vẽ frame hiện tại
- interaction time: thời gian do scrubber hoặc user action điều khiển

### 3.3 Derivation Rule

Rendering không được mutate state theo kiểu incremental làm méo thời gian.
Frame state phải được derive từ clock + canonical chart representation.

## 4. Interaction Model

### 4.1 Interaction Primitives

Tối thiểu phải có:

- hover
- select
- scrub
- zoom
- pan
- seek
- focus chart lane
- inspect note or marker

### 4.2 Interaction Boundaries

Interaction chỉ được thay đổi rendering-only state và runtime view-model.

Nó không được:

- mutate canonical chart data
- rewrite replay provenance
- alter parser output
- re-encode normalized entities

### 4.3 Debug-First Interaction

Phase 0 nên ưu tiên interaction giúp quan sát:

- scrubbing timeline
- highlighting event groups
- inspecting note density
- showing timing offsets
- showing provenance and validation warnings

## 5. Coordinate System

### 5.1 Coordinate Axes

Coordinate system phải ổn định và dễ debug.

Đề xuất:

- `x` = lane axis
- `y` = time axis or chart progression axis
- optional z/layer order = render depth only

### 5.2 Space Separation

Phải tách rõ:

- chart space
- viewport space
- device space
- overlay space

### 5.3 Mapping Rule

All note positions should be derivable from canonical chart time + lane metadata.
Không cho phép vị trí note được quyết định bởi arbitrary UI mutation.

## 6. Replay Clock

### 6.1 Deterministic Replay Clock

Replay clock phải deterministic trong phạm vi đầu vào đã biết.

Nếu cùng:

- chart
- replay
- offset
- parser version
- normalization version

thì output must remain identical.

### 6.2 Replay Clock Inputs

Replay clock nên nhận:

- chart timeline
- replay event stream
- timing offsets
- offset calibration mode nếu có
- paused/scrubbed state

### 6.3 Replay Clock Outputs

Replay clock nên xuất:

- current time
- active hit window
- event alignment state
- miss/timing markers
- overlay annotations

## 7. Render Layers

### 7.1 Recommended Layer Order

1. background / atmosphere
2. lane geometry
3. note bodies
4. timing markers / hold fills
5. replay overlay
6. debug annotations
7. interaction highlights
8. UI chrome / metadata sidebar linkage

### 7.2 Layer Rules

- background must not encode gameplay truth
- gameplay truth lives in chart and replay layers
- debug layers must be toggleable
- overlay layers must not mutate base chart layer

## 8. Performance Constraints

### 8.1 Phase 0 Performance Budget

The renderer should be constrained enough to stay observable.

Targets:

- chart open should feel immediate on local fixture data
- scrubbing should remain responsive
- repeated re-render should not create visible drift
- debug overlays should not destroy interactivity

### 8.2 Cost Boundaries

Avoid:

- expensive per-frame data reconstruction
- deep cloning of large chart objects on every render
- layout thrash from DOM-driven animation
- unnecessary recomputation of derived lanes or note groups

### 8.3 Measure What Matters

Measure:

- frame latency
- scrub latency
- open latency
- replay overlay cost
- cache hit/miss impact

## 9. Deterministic Rendering Guarantees

### 9.1 Guarantees

The renderer must guarantee that:

- same input produces same visual state within declared tolerances
- same chart/replay/clock state produces same overlay alignment
- render output is reproducible enough for inspection and snapshot testing
- debug mode does not alter canonical semantics

### 9.2 What Determinism Does Not Mean

Determinism does not mean:

- byte-for-byte identical pixels across every device
- identical font rasterization everywhere
- identical browser compositor behavior everywhere

It means canonical semantics and visual alignment remain stable within defined limits.

### 9.3 Determinism Rule

If the renderer needs to guess, the guess must be explicit, versioned, and observable.

## 10. Implementation Shape

### 10.1 Renderer Pipeline

1. ingest canonical chart data
2. derive render model
3. map render model into layer state
4. draw by clock
5. overlay replay/inspection data
6. emit instrumentation

### 10.2 State Ownership

- canonical state lives outside renderer
- runtime view-model feeds renderer
- render-only state stays local to the view
- replay clock is shared, not duplicated

### 10.3 Debug Surface

Phase 0 renderer should expose a debug surface for:

- note density
- timing offsets
- layer toggles
- validation warnings
- provenance hints

## 11. Minimum Viable Rendering Contract

Phase 0 rendering is successful only if it can answer:

- can one chart render deterministically?
- can the same timeline be inspected repeatedly?
- can replay overlay share the same clock?
- can debug layers reveal why the renderer drew what it drew?

If any answer is no, the renderer is not yet a foundation layer.
