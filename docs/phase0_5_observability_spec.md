# Phase 0.5 — Observability Spec

## 1. Mục Tiêu

Tài liệu này định nghĩa observability như một schema chính thức, không phải một ý tưởng mơ hồ.

Phase 0 được tạo ra để học từ hành vi thật, nên mọi log, metric, session, và annotation phải phục vụ câu hỏi nghiên cứu cụ thể:

- người dùng mở gì
- họ hiểu gì
- họ bỏ ở đâu
- họ quay lại gì
- friction nào đang xuất hiện

## 2. Principles

- collect only what is needed
- prefer behavioral signals over personal data
- keep logs inspectable
- keep privacy boundaries explicit
- make every event useful for a research question
- avoid telemetry that exists only because it is easy to add

## 3. Sessions

### 3.1 Session Definition

A session is a bounded local observation window.

Session fields:

- `sessionId`
- `startedAt`
- `endedAt`
- `routeSequence`
- `entityOpenSequence`
- `deviceClass`
- `privacyMode`
- `storageScope`

### 3.2 Session Rules

- sessions may be anonymous or local-only
- sessions do not require user accounts
- sessions must not depend on websocket presence
- sessions should remain analyzable offline
- session identity must not be used as a covert profile

## 4. Events

### 4.1 Event Taxonomy

Tối thiểu phải có các nhóm event sau:

- `session_start`
- `session_end`
- `route_open`
- `route_close`
- `entity_open`
- `entity_close`
- `search_submit`
- `search_result_click`
- `chart_view_open`
- `replay_view_open`
- `metadata_interaction`
- `timeline_scrub`
- `cache_hit`
- `cache_miss`
- `offline_fallback`
- `parse_warning`
- `parse_error`
- `render_warning`
- `render_frame_drop`
- `return_visit`
- `qualitative_note`

### 4.2 Event Shape

```ts
export interface ObservationEvent {
  eventId: string;
  eventType: string;
  timestamp: string;
  sessionId: string;
  entityRef?: {
    kind: 'song' | 'chart' | 'pack' | 'asset' | 'replay';
    id: string;
  };
  routeRef?: string;
  payload?: Record<string, unknown>;
  source: 'ui' | 'parser' | 'renderer' | 'cache' | 'system';
}
```

### 4.3 Event Rules

- events must be small and structured
- events must be versioned if shape changes
- events should not contain secrets
- events must be useful even when aggregated locally

## 5. Metrics

### 5.1 Core Metrics

- chart open latency
- replay open latency
- search latency
- cache hit rate
- cache miss rate
- scrub latency
- render frame drop count
- parse warning count
- parse error count
- revisit rate
- return route frequency

### 5.2 Metric Rules

- metrics should summarize behavior, not replace events
- metrics must be derivable from logged events where possible
- metrics should support trend comparison across sessions
- metrics should be local-first in Phase 0

## 6. Qualitative Annotations

### 6.1 Purpose

Qualitative annotations capture why behavior happened, not just what happened.

Examples:

- user felt chart was hard to parse
- metadata was useful but buried
- replay overlay clarified timing
- user abandoned due to missing asset
- user returned because cache made revisit easy

### 6.2 Annotation Rules

- annotations must be concise
- annotations should be tied to a session or entity where possible
- annotations should stay optional
- annotations must not be treated as authoritative truth unless confirmed

## 7. Logging Boundaries

### 7.1 What To Log

Log:

- routes opened and closed
- entity open / close actions
- search and navigation transitions
- replay and chart interactions
- cache hits and misses
- parse/render warnings
- return behavior signals

### 7.2 What Not To Log

Do not log by default:

- raw personal identity
- passwords or secrets
- unnecessary keystroke streams
- sensitive device fingerprints
- full content payloads when a summary is enough
- anything that cannot justify itself by research value

### 7.3 Boundary Rule

If a piece of data is not needed to understand behavior, do not collect it.

## 8. Privacy Rules

### 8.1 Default Privacy Posture

- local-first by default
- anonymous when possible
- explicit opt-in for anything beyond local diagnostics
- no hidden profile construction
- no surprise transmission of user behavior

### 8.2 Minimization Rules

- store the smallest payload that still answers the research question
- prefer categorical values over raw free text when possible
- truncate or bucket noisy detail if it is not needed
- separate debugging data from user-facing data where practical

### 8.3 Retention Rules

- keep raw event data only as long as needed for Phase 0 analysis
- aggregate or summarize when detailed traces are no longer required
- define deletion policy before expanding collection scope

## 9. Storage Policy

### 9.1 Storage Tiers

1. ephemeral runtime memory
2. local persistent store
3. summarized analytic snapshots
4. exported diagnostic bundles when intentionally produced

### 9.2 Storage Rules

- local persistent storage should be readable and inspectable
- stored telemetry should be versioned
- schemas should survive app upgrades when possible
- stale telemetry should be discardable without breaking the app

### 9.3 Recommended Phase 0 Storage

Prefer local-first storage such as IndexedDB or a small embedded store if needed.

Do not introduce cloud telemetry as the default path in Phase 0.

## 10. Qualifying Return Behavior

Return behavior should be defined by repeated meaningful interaction, not by raw app opens.

Signals include:

- reopening the same chart detail
- revisiting replay inspection
- moving between song, pack, and chart contexts
- using cached content on a later session
- search patterns that repeat across sessions

## 11. Logging Format

### 11.1 Suggested Record Shape

```ts
export interface SessionRecord {
  sessionId: string;
  startedAt: string;
  endedAt?: string;
  events: ObservationEvent[];
  metrics: Record<string, number>;
  notes?: string[];
}
```

### 11.2 Record Rules

- records should be append-friendly
- records should be easy to inspect in debug mode
- records should not require a backend to be useful

## 12. Observability Review

Observability is successful only if it can answer:

- where users hesitate
- what users revisit
- which screens are empty of value
- which render paths are too slow
- which parser or cache paths are unstable

If the logs cannot answer those questions, the spec is too vague.
