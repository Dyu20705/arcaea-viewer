# Phase 0.3 — Canonical Data Model

## 1. Mục Tiêu

Tài liệu này chốt xương sống dữ liệu cho Phase 0.

Canonical data model phải trả lời rõ: hệ thống đang nói về thực thể nào, quan hệ nào là thật, dữ liệu nào là nguồn, dữ liệu nào là dẫn xuất, và cái gì được phép thay đổi theo thời gian.

Nguyên tắc chính:

- stable hơn complete
- inspectable hơn clever
- versioned hơn ad hoc
- deterministic hơn linh hoạt quá mức

## 2. Scope Của Canonical Model

Canonical model chỉ bao gồm những thực thể cần để:

- ingest chart/song/pack metadata
- render một chart deterministic
- inspect replay tối thiểu
- đo return behavior và friction cơ bản
- giữ provenance rõ ràng

Không đưa vào model gốc những thứ chưa chứng minh giá trị.

## 3. Entities

### 3.1 Song

Song là thực thể mẹ cho chart và metadata.

Fields tối thiểu:

- `songId`
- `title`
- `titleAliases`
- `artist`
- `packIds`
- `versionGroup`
- `tags`
- `availability`
- `provenance`
- `assetRefs`

### 3.2 Chart

Chart là đơn vị quan trọng nhất của Phase 0.

Fields tối thiểu:

- `chartId`
- `songId`
- `difficulty`
- `chartType`
- `bpmTimeline`
- `noteEvents`
- `holdEvents`
- `arcEvents`
- `specialEvents`
- `parserVersion`
- `sourceVersion`
- `validationStatus`
- `provenance`

### 3.3 Pack

Pack là lớp ngữ cảnh và navigation.

Fields tối thiểu:

- `packId`
- `name`
- `releaseOrder`
- `songIds`
- `loreRefs`
- `provenance`

### 3.4 Asset

Asset là tham chiếu cho thumbnail, icon, banner, visual reference.

Fields tối thiểu:

- `assetId`
- `assetType`
- `checksum`
- `storageRef`
- `sourceRef`
- `licenseBoundary`
- `displayEligibility`
- `provenance`

### 3.5 Replay

Replay là representation của một lần chơi hoặc sample playback.

Fields tối thiểu:

- `replayId`
- `chartId`
- `sessionTimestamp`
- `inputEvents`
- `timingOffsets`
- `environmentHints`
- `parseStatus`
- `provenance`

### 3.6 Session

Session là một đơn vị quan sát hành vi.

Fields tối thiểu:

- `sessionId`
- `startedAt`
- `endedAt`
- `routeSequence`
- `entityOpenSequence`
- `deviceClass`
- `privacyMode`

### 3.7 ObservationEvent

ObservationEvent là một log record có cấu trúc.

Fields tối thiểu:

- `eventId`
- `eventType`
- `timestamp`
- `sessionId`
- `entityRef`
- `routeRef`
- `payload`
- `source`

## 4. Relationships

### 4.1 Core Relations

- Song 1 → N Chart
- Song N ↔ N Pack
- Song 1 → N Asset
- Chart 1 → N Replay
- Session 1 → N ObservationEvent
- Chart 1 → N ObservationEvent
- Song 1 → N ObservationEvent

### 4.2 Relation Rules

Quan hệ phải được model hóa rõ ràng, không nhét tùy tiện vào JSON lẫn lộn.

Nếu một quan hệ cần query, filter, hoặc render ổn định, nó phải tồn tại như một relation layer rõ ràng hoặc index dẫn xuất rõ ràng.

## 5. Schemas

### 5.1 Song Schema

```ts
export interface Song {
  songId: string;
  title: string;
  titleAliases: string[];
  artist: string;
  packIds: string[];
  versionGroup?: string;
  tags: string[];
  availability: {
    isAvailable: boolean;
    region?: string;
    sourceState: 'confirmed' | 'inferred' | 'pending';
  };
  assetRefs: string[];
  provenance: Provenance;
}
```

### 5.2 Chart Schema

```ts
export interface Chart {
  chartId: string;
  songId: string;
  difficulty: string;
  chartType?: string;
  bpmTimeline: BpmPoint[];
  noteEvents: NoteEvent[];
  holdEvents: HoldEvent[];
  arcEvents: ArcEvent[];
  specialEvents: SpecialEvent[];
  parserVersion: string;
  sourceVersion: string;
  validationStatus: 'valid' | 'warning' | 'invalid';
  provenance: Provenance;
}
```

### 5.3 Pack Schema

```ts
export interface Pack {
  packId: string;
  name: string;
  releaseOrder?: number;
  songIds: string[];
  loreRefs: string[];
  provenance: Provenance;
}
```

### 5.4 Asset Schema

```ts
export interface Asset {
  assetId: string;
  assetType: 'thumbnail' | 'icon' | 'banner' | 'chart-art' | 'other';
  checksum: string;
  storageRef: string;
  sourceRef: string;
  licenseBoundary: 'public' | 'restricted' | 'unknown';
  displayEligibility: 'allowed' | 'blocked' | 'needs-review';
  provenance: Provenance;
}
```

### 5.5 Replay Schema

```ts
export interface Replay {
  replayId: string;
  chartId: string;
  sessionTimestamp: string;
  inputEvents: ReplayInputEvent[];
  timingOffsets: TimingOffset[];
  environmentHints: EnvironmentHint[];
  parseStatus: 'valid' | 'warning' | 'invalid';
  provenance: Provenance;
}
```

### 5.6 Provenance Schema

```ts
export interface Provenance {
  sourceSystem: string;
  sourceSnapshotId: string;
  sourceRecordId?: string;
  parserVersion?: string;
  importedAt: string;
  confidence: 'confirmed' | 'inferred' | 'derived';
}
```

## 6. Invariants

### 6.1 Identity Invariants

- `songId`, `chartId`, `packId`, `assetId`, `replayId`, `sessionId`, and `eventId` must be stable and unique within their namespace.
- A chart must always belong to exactly one song.
- A replay must always reference exactly one chart.
- An observation event must always belong to exactly one session.

### 6.2 Determinism Invariants

- Same source snapshot + same parser version + same normalization rules must produce same canonical chart output.
- Renderer must not mutate canonical entities.
- Derived fields must be reproducible from canonical sources.

### 6.3 Provenance Invariants

- Every canonical entity must carry provenance.
- Any inferred field must be marked as inferred or derived.
- If provenance is missing, the entity is incomplete and must not pretend to be authoritative.

### 6.4 Validation Invariants

- Invalid charts must remain representable.
- Validation status must be explicit.
- Partial data must not silently become complete data.

### 6.5 Boundary Invariants

- Source data is immutable in runtime UI.
- Canonical data can be read, not casually rewritten.
- Runtime state cannot become the source of truth.

## 7. Derived Fields

Derived fields are allowed, but they must be explicit and reproducible.

### 7.1 Common Derived Fields

- `searchTokens`
- `difficultyRank`
- `chartDensitySummary`
- `noteCountSummary`
- `durationEstimate`
- `laneOccupancyProfile`
- `returnBehaviorFlags`
- `renderHintSet`

### 7.2 Derived Field Rules

- Derived fields must be computed from canonical input or from another documented derived layer.
- Derived fields must not overwrite source fields.
- Derived fields must be invalidated when source version or parser version changes.
- Derived fields should be cheap to recompute or cacheable locally.

## 8. Provenance Rules

### 8.1 Source Priority

Source order should be explicit:

1. immutable source snapshot
2. parser output
3. normalization output
4. derived view-model
5. runtime state

Higher layers may refine lower layers, but not erase their origin.

### 8.2 Confidence Rules

- `confirmed` means directly observed from a known source.
- `inferred` means computed from evidence, not directly stated.
- `derived` means generated from canonical or normalized data.

### 8.3 Audit Rules

- A user should be able to inspect where the value came from.
- A debug view should expose source snapshot, parser version, and transform path.
- If a field is contested, the system should prefer showing uncertainty over hiding it.

## 9. Versioning Rules

- Schema version must be explicit.
- Parser version must be explicit.
- Normalization version must be explicit.
- Breaking changes require a new versioned contract.
- Non-breaking additions should not force old data to lose meaning.

## 10. Minimum Viable Canonical Set

Phase 0 only needs the following to be real:

- Song
- Chart
- Pack
- Asset
- Replay stub
- Session
- ObservationEvent
- Provenance

If any of these are missing, later layers will start inventing truth instead of reading it.
