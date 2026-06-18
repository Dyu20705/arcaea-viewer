# Kế hoạch triển khai dự án Arcaea-Viewer

Master plan hiện tại mô tả một nền tảng rất lớn: metadata explorer, chart renderer, replay engine, analytics, offline/PWA, lore graph, progression và AI-assisted analysis. Nếu triển khai đồng thời, dự án dễ biến thành một monorepo phức tạp nhưng không có tính năng nào hoàn thiện.

Vì vậy, kế hoạch thực hiện nên đi theo nguyên tắc:

> Xây dựng một lát cắt hoàn chỉnh từ dữ liệu thô → Rust parser → API → frontend → chart visualization, sau đó mới mở rộng replay, analytics và offline.

Kế hoạch dưới đây được xây dựng trực tiếp từ kiến trúc, monorepo, các subsystem và phased roadmap trong master plan. 

---

# 1. Mục tiêu phiên bản đầu tiên

## Arcaea-Viewer MVP 0.1

Phiên bản đầu tiên phải cho phép người dùng:

1. Mở danh sách bài hát.
2. Tìm kiếm và lọc bài hát/chart.
3. Xem thông tin chi tiết một chart.
4. Nạp một chart từ dữ liệu mẫu hợp pháp.
5. Parse chart bằng Rust.
6. Render chart trên timeline tương tác.
7. Play, pause, scrub và thay đổi tốc độ.
8. Xem các thông tin phân tích cơ bản:

   * tổng số note;
   * note density;
   * BPM/timing segments;
   * phân bố loại note.
9. Hoạt động với một dataset snapshot cố định.
10. Có kiểm thử parser, timing và rendering cơ bản.

## Không đưa vào MVP 0.1

Tạm thời không triển khai:

* tài khoản người dùng;
* replay upload công khai;
* multiplayer hoặc community;
* AI difficulty estimation;
* full lore graph;
* desktop application;
* Kubernetes;
* microservices;
* GraphQL;
* recommendation cá nhân hóa;
* toàn bộ dữ liệu và asset có bản quyền;
* hệ thống admin hoàn chỉnh.

Đây là các tính năng sau MVP, không phải điều kiện để chứng minh giá trị kỹ thuật cốt lõi.

---

# 2. Product slice đầu tiên

Lát cắt đầu tiên cần hoàn chỉnh theo luồng:

```text
Raw chart fixture
      ↓
Rust chart parser
      ↓
Canonical Chart IR
      ↓
Rust timing/analytics
      ↓
WASM bindings
      ↓
React chart viewer
      ↓
Interactive timeline
      ↓
Basic chart analytics
```

Lát cắt này quan trọng hơn việc có hàng nghìn bài hát nhưng chưa render hoặc phân tích được.

---

# 3. Thứ tự ưu tiên subsystem

| Ưu tiên | Subsystem            | Lý do                                         |
| ------- | -------------------- | --------------------------------------------- |
| P0      | Canonical data model | Mọi hệ thống khác phụ thuộc vào nó            |
| P0      | Chart parser         | Chuyển dữ liệu thô thành dữ liệu tin cậy      |
| P0      | Timing engine        | Renderer, replay và analytics đều sử dụng     |
| P0      | Chart renderer       | Giá trị trực quan và bề mặt kiểm chứng parser |
| P1      | Metadata explorer    | Tạo trải nghiệm sản phẩm hoàn chỉnh           |
| P1      | WASM integration     | Chia sẻ Rust core với trình duyệt             |
| P1      | Basic analytics      | Chứng minh hướng analysis-driven              |
| P2      | Replay engine        | Phụ thuộc parser và timing đã ổn định         |
| P2      | Offline/PWA          | Phụ thuộc contract và cache model ổn định     |
| P3      | Lore/progression     | Có thể phát triển độc lập sau                 |
| P3      | AI-assisted analysis | Chỉ thêm sau khi deterministic baseline tốt   |

---

# 4. Cấu trúc monorepo giai đoạn đầu

Không cần tạo toàn bộ cấu trúc tương lai ngay từ ngày đầu. Khởi tạo phiên bản tinh gọn:

```text
arcaea-viewer/
├── apps/
│   └── web/
│       ├── src/
│       │   ├── app/
│       │   ├── features/
│       │   │   ├── explore/
│       │   │   ├── chart-viewer/
│       │   │   └── analytics/
│       │   ├── components/
│       │   ├── lib/
│       │   └── routes/
│       └── tests/
│
├── crates/
│   ├── core/
│   ├── parser/
│   ├── timing/
│   ├── analytics/
│   └── wasm/
│
├── packages/
│   ├── api-client/
│   ├── types/
│   ├── ui/
│   └── config/
│
├── data/
│   ├── raw/
│   ├── normalized/
│   ├── fixtures/
│   └── snapshots/
│
├── tools/
│   ├── validation/
│   └── benchmark/
│
├── docs/
│   ├── architecture/
│   ├── rfc/
│   └── adr/
│
├── infra/
│   └── docker/
│
├── Cargo.toml
├── package.json
├── pnpm-workspace.yaml
└── README.md
```

Chưa tạo:

```text
apps/admin
apps/desktop
infra/k8s
infra/terraform
crates/replay
apps/worker
```

Chỉ thêm khi milestone tương ứng bắt đầu.

---

# 5. Kế hoạch tổng thể

Kế hoạch hợp lý cho một sinh viên hoặc một developer chính là khoảng **24 tuần**, chia thành 8 milestone.

```text
M0  Project Foundation          2 tuần
M1  Data Model & Fixtures       2 tuần
M2  Parser & Timing Core        4 tuần
M3  Metadata Explorer           3 tuần
M4  Chart Renderer              5 tuần
M5  Basic Analytics             3 tuần
M6  Offline & Quality Gate      3 tuần
M7  MVP Release                 2 tuần
```

Replay engine được triển khai sau MVP trong một roadmap riêng.

---

# 6. Milestone 0 — Project Foundation

## Thời lượng

Tuần 1–2.

## Mục tiêu

Tạo nền móng dự án, tiêu chuẩn code, CI và architecture contract trước khi viết tính năng lớn.

## Công việc

### Repository

* Khởi tạo Git repository.
* Thiết lập pnpm workspace.
* Khởi tạo Cargo workspace.
* Tạo `apps/web`.
* Tạo các Rust crate ban đầu.
* Thiết lập alias và dependency boundaries.

### Frontend

Khởi tạo:

* React;
* TypeScript strict mode;
* Vite;
* TanStack Router hoặc React Router;
* TanStack Query;
* Zustand;
* Vitest;
* Playwright;
* ESLint;
* Prettier.

### Rust

Thiết lập:

* stable Rust toolchain;
* `rustfmt`;
* `clippy`;
* `cargo-nextest`;
* `wasm-pack` hoặc `wasm-bindgen`;
* `serde`;
* `thiserror`;
* property-based testing với `proptest`.

### CI

Pipeline tối thiểu:

```text
install
  ↓
format check
  ↓
lint
  ↓
unit test
  ↓
Rust test
  ↓
WASM build
  ↓
frontend build
```

### Documentation

Tạo:

```text
docs/architecture/SYSTEM_CONTEXT.md
docs/architecture/DEPENDENCY_RULES.md
docs/rfc/RFC_TEMPLATE.md
docs/adr/ADR-0001-monorepo.md
docs/adr/ADR-0002-rust-wasm-boundary.md
docs/adr/ADR-0003-rendering-technology.md
```

## Quyết định phải khóa

1. PixiJS hay WebGL abstraction khác.
2. JSON hay MessagePack tại WASM boundary.
3. Đơn vị thời gian chuẩn:

   * microseconds;
   * milliseconds integer;
   * fixed-point ticks.
4. ID strategy:

   * UUID;
   * stable content ID;
   * compound domain key.
5. Format canonical chart representation.

## Kết quả bàn giao

* Monorepo build thành công.
* Frontend gọi được một hàm Rust/WASM thử nghiệm.
* CI chạy trên mọi pull request.
* Có architecture dependency rules.

## Exit criteria

```text
pnpm test       PASS
pnpm build      PASS
cargo test      PASS
cargo clippy    PASS
WASM demo       PASS
```

---

# 7. Milestone 1 — Canonical Data Model và Fixtures

## Thời lượng

Tuần 3–4.

## Mục tiêu

Xây dựng một mô hình chart độc lập với UI và nguồn dữ liệu.

## Domain model đề xuất

```rust
pub struct Chart {
    pub id: ChartId,
    pub metadata: ChartMetadata,
    pub timing_groups: Vec<TimingGroup>,
    pub events: Vec<ChartEvent>,
    pub source: SourceProvenance,
    pub schema_version: SchemaVersion,
}
```

```rust
pub enum ChartEvent {
    Tap(TapNote),
    Hold(HoldNote),
    Arc(ArcNote),
    Timing(TimingEvent),
    Camera(CameraEvent),
    SceneControl(SceneControlEvent),
}
```

## Các entity chính

* `Song`
* `Chart`
* `Difficulty`
* `Pack`
* `Artist`
* `TimingEvent`
* `TapNote`
* `HoldNote`
* `ArcNote`
* `ChartMetadata`
* `SourceProvenance`
* `DatasetVersion`

## Quy tắc dữ liệu

Mọi chart canonical phải đảm bảo:

* event có thứ tự xác định;
* timestamp không âm, trừ khi format cho phép offset rõ ràng;
* ID ổn định;
* timing segment không mơ hồ;
* arc control point hợp lệ;
* mọi warning và error có mã;
* có schema version;
* có source checksum.

## Fixtures

Tạo ít nhất 8 chart fixture:

1. Chart chỉ có tap.
2. Chart có hold.
3. Chart có arc.
4. Chart có BPM change.
5. Chart có offset.
6. Chart có event cùng timestamp.
7. Chart malformed.
8. Chart stress test nhiều event.

## Kết quả bàn giao

```text
data/fixtures/
├── simple-tap/
├── hold-basic/
├── arc-basic/
├── bpm-change/
├── offset-edge/
├── same-timestamp/
├── malformed/
└── stress-large/
```

## Exit criteria

* Schema canonical đã được viết thành tài liệu.
* Fixture hợp lệ parse được.
* Fixture lỗi trả về diagnostics có cấu trúc.
* Snapshot canonical ổn định qua nhiều lần chạy.

---

# 8. Milestone 2 — Parser và Timing Engine

## Thời lượng

Tuần 5–8.

Đây là milestone kỹ thuật quan trọng nhất.

---

## 8.1 Parser

### Pipeline

```text
Raw bytes
   ↓
Lexing/tokenization
   ↓
Syntax parsing
   ↓
Semantic validation
   ↓
Normalization
   ↓
Canonical Chart IR
   ↓
Diagnostics report
```

### Parser output

```rust
pub struct ParseResult {
    pub chart: Option<Chart>,
    pub diagnostics: Vec<Diagnostic>,
}
```

```rust
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: Severity,
    pub message: String,
    pub location: Option<SourceLocation>,
}
```

### Yêu cầu

* Không panic với input người dùng.
* Không bỏ qua dữ liệu lỗi âm thầm.
* Phân biệt warning và fatal error.
* Có provenance.
* Có checksum.
* Parser output deterministic.

### Testing

* unit tests;
* snapshot tests;
* property tests;
* fuzz tests;
* malformed input tests;
* large chart benchmark.

---

## 8.2 Timing engine

### Trách nhiệm

* chuyển beat sang chart time;
* chuyển chart time sang beat;
* xử lý BPM segment;
* xử lý offset;
* trả về active event tại thời điểm bất kỳ;
* xác định vùng visible theo timeline;
* cung cấp frame-independent time calculation.

### Quy tắc quan trọng

Không sử dụng `f32` làm nguồn sự thật cho timing.

Ưu tiên:

```rust
pub struct ChartTime(i64); // microseconds
```

Hoặc fixed-point integer tương đương.

`f32` chỉ dùng ở lớp rendering cuối cùng.

### API cơ bản

```rust
fn beat_to_time(beat: BeatPosition) -> ChartTime;
fn time_to_beat(time: ChartTime) -> BeatPosition;
fn events_in_range(start: ChartTime, end: ChartTime) -> &[ChartEvent];
fn resolve_timing_segment(time: ChartTime) -> &TimingSegment;
```

## Benchmark mục tiêu ban đầu

* parse chart trung bình dưới 50 ms;
* query visible events dưới 2 ms;
* không allocation lớn trong vòng lặp render;
* cùng input luôn cho cùng canonical output.

## Kết quả bàn giao

* Rust parser production-ready ở mức MVP.
* Timing engine có edge-case tests.
* WASM có thể nhận chart fixture và trả canonical summary.
* Có benchmark report.

## Exit criteria

* Không panic với fuzz corpus.
* Determinism test đạt 100%.
* BPM change và offset tests đều pass.
* Parser diagnostics dễ đọc.
* Benchmark không có regression rõ ràng.

---

# 9. Milestone 3 — Metadata Explorer

## Thời lượng

Tuần 9–11.

## Mục tiêu

Tạo bề mặt sản phẩm đầu tiên có thể sử dụng.

## Các route

```text
/
├── /explore
├── /charts/:chartId
├── /analytics/:chartId
└── /settings
```

## Trang Explore

Cần có:

* search box;
* difficulty filter;
* song/artist filter;
* BPM range;
* note count range;
* sorting;
* result virtualization;
* URL-backed filter state.

Ví dụ URL:

```text
/explore?difficulty=9,9plus&sort=density-desc&q=memory
```

## Chart detail page

Hiển thị:

* title;
* artist;
* chart designer;
* difficulty;
* BPM;
* note count;
* version;
* pack;
* source provenance;
* link tới chart viewer;
* link tới analytics.

## Data strategy cho MVP

Không cần PostgreSQL ngay lập tức.

Có thể dùng:

```text
Normalized JSON snapshot
        ↓
Static data adapter
        ↓
TanStack Query
        ↓
UI
```

Nhưng phải đặt adapter interface giống API tương lai:

```ts
interface ChartRepository {
  listCharts(query: ChartSearchQuery): Promise<Page<ChartSummary>>;
  getChart(id: string): Promise<ChartDetail>;
}
```

Nhờ vậy có thể thay static adapter bằng REST API mà không viết lại UI.

## Kết quả bàn giao

* Search/filter hoạt động.
* Deep link hoạt động.
* Loading, empty và error states rõ ràng.
* Query key được chuẩn hóa.
* Không lưu server data vào Zustand.

## Exit criteria

* Có thể tìm chart trong dataset mẫu.
* URL có thể chia sẻ và khôi phục đúng filter.
* Danh sách lớn không gây lag rõ rệt.
* Chart detail có provenance.

---

# 10. Milestone 4 — Chart Visualization Engine

## Thời lượng

Tuần 12–16.

Đây là phần tạo ra bản sắc chính của dự án.

## Kiến trúc renderer

```text
React Controls
      ↓
Timeline Controller
      ↓
Rust/WASM Chart Evaluation
      ↓
Frame State
      ↓
Render Command Buffer
      ↓
PixiJS/WebGL Scene
```

## Không để React render từng note

React chỉ quản lý:

* page;
* controls;
* panels;
* accessibility layer;
* overlay text;
* settings.

PixiJS/WebGL quản lý:

* note objects;
* arcs;
* lane;
* animation;
* culling;
* render loop.

## Frame state

```ts
interface ChartFrameState {
  chartTimeUs: number;
  visibleStartUs: number;
  visibleEndUs: number;
  notes: RenderNote[];
  arcs: RenderArc[];
  timingLines: RenderTimingLine[];
}
```

## Tính năng cần hoàn thành

### Timeline

* play;
* pause;
* seek;
* scrub;
* speed 0.25×–2×;
* restart;
* step frame/debug;
* calibration offset.

### Visual objects

* lane geometry;
* timing lines;
* tap notes;
* hold notes;
* basic arcs;
* active/pending/passed states;
* debug bounding boxes.

### Debug overlays

* chart timestamp;
* beat position;
* visible event count;
* FPS;
* event ID;
* timing segment;
* note type.

### Rendering optimization

* object pooling;
* off-screen culling;
* immutable precomputed arc geometry;
* reusable buffers;
* no per-frame parsing;
* no React state update mỗi frame.

## Testing

### Snapshot cases

* tap placement;
* hold duration;
* arc geometry;
* BPM transition;
* seek state;
* speed changes.

### Performance cases

* 500 notes;
* 2.000 notes;
* chart có nhiều arc;
* liên tục scrub;
* resize viewport.

## Mục tiêu hiệu năng

Trên thiết bị desktop phổ thông:

* 60 FPS với chart thông thường;
* frame budget trung bình dưới 16,6 ms;
* không memory leak khi replay liên tục;
* scrub phản hồi gần như tức thời;
* không reparse chart khi seek.

## Exit criteria

* Chart fixture được render đúng thứ tự và vị trí.
* Play/pause/seek không làm lệch timeline.
* Không phụ thuộc vào frame rate để tính chart time.
* Visual snapshot tests bắt được regression.
* Stress fixture vẫn sử dụng được.

---

# 11. Milestone 5 — Analytics Baseline

## Thời lượng

Tuần 17–19.

## Mục tiêu

Chứng minh Arcaea-Viewer là công cụ phân tích, không chỉ là chart viewer.

## Analytics phiên bản đầu

### Tổng quan chart

* tổng số event;
* số tap;
* số hold;
* số arc;
* chart duration;
* average notes per second;
* peak notes per second.

### Density curve

Tính theo nhiều cửa sổ:

* 250 ms;
* 500 ms;
* 1 giây;
* 2 giây.

```rust
pub struct DensityPoint {
    pub time: ChartTime,
    pub event_count: u32,
    pub normalized_rate: f32,
}
```

### Burst segmentation

Xác định:

* burst start;
* burst end;
* peak;
* recovery window;
* sustained density region.

### Arc baseline

* arc count;
* average duration;
* average path variation;
* concurrent arc count;
* arcs overlapping tap/hold sections.

### Timing complexity

* số BPM segment;
* số BPM transition;
* offset complexity;
* event clustering tại cùng timestamp.

## UI Analytics

Hiển thị:

* density timeline;
* note type breakdown;
* chart section markers;
* peak segments;
* metric explanation;
* link từ chart viewer tới thời điểm tương ứng.

Ví dụ:

```text
Peak density: 11.8 notes/s
Time range: 01:34.200–01:35.200
Evidence: 8 taps, 2 holds, 3 arc interactions
```

## Nguyên tắc

Không tạo một con số “difficulty AI score” trong giai đoạn này.

Mỗi metric phải:

* xác định được cách tính;
* có unit;
* có giải thích;
* reproducible;
* có version.

## Exit criteria

* Metrics giống nhau với cùng chart/version.
* Density chart liên kết được với visualization timeline.
* Có tests cho từng feature.
* Analytics schema có `analytics_version`.

---

# 12. Milestone 6 — Offline và Quality Gate

## Thời lượng

Tuần 20–22.

## Offline scope của MVP

Người dùng offline có thể:

* mở app shell;
* xem metadata đã tải;
* mở chart đã cache;
* render chart đã cache;
* xem analytics đã cache;
* biết dữ liệu nào stale hoặc missing.

## Thành phần

* service worker;
* persisted TanStack Query cache;
* IndexedDB manifest;
* chart cache;
* dataset version;
* cache integrity checksum;
* manual cache clear;
* storage usage display.

## Cache manifest

```ts
interface CacheManifest {
  manifestVersion: number;
  datasetVersion: string;
  schemaVersion: string;
  cachedCharts: CachedChartEntry[];
  generatedAt: string;
}
```

## Trạng thái UI bắt buộc

```text
Available offline
Cached but stale
Online only
Sync required
Corrupted cache
Unsupported cache version
```

## Quality gate

Chạy đầy đủ:

* frontend unit tests;
* Rust tests;
* parser fuzz tests;
* WASM integration tests;
* rendering snapshots;
* Playwright E2E;
* Lighthouse;
* bundle analysis;
* memory profiling;
* dependency audit.

## E2E flows

### Flow 1

```text
Explore
→ Search
→ Open chart
→ Play chart
→ Open analytics
→ Click peak segment
→ Return to same chart time
```

### Flow 2

```text
Open chart online
→ Cache chart
→ Disable network
→ Reload
→ Open cached chart
→ Render and inspect analytics
```

### Flow 3

```text
Load malformed fixture
→ Parser rejects safely
→ UI displays structured diagnostic
```

## Exit criteria

* Các critical flow chạy tự động.
* Offline reload hoạt động.
* Corrupted cache không crash app.
* Parser không panic.
* Không có lỗi accessibility nghiêm trọng.
* Không có high-severity dependency vulnerability chưa được xử lý.

---

# 13. Milestone 7 — MVP Release

## Thời lượng

Tuần 23–24.

## Công việc

### Product polish

* responsive layout;
* keyboard controls;
* onboarding ngắn;
* empty states;
* error boundaries;
* loading skeletons;
* provenance display;
* legal notice;
* cache management page.

### Documentation

Tạo:

```text
README.md
CONTRIBUTING.md
SECURITY.md
LEGAL_AND_DATA_POLICY.md
docs/architecture/OVERVIEW.md
docs/api/CHART_IR.md
docs/api/WASM_CONTRACT.md
docs/runbooks/LOCAL_DEVELOPMENT.md
docs/runbooks/RELEASE.md
docs/runbooks/DATA_UPDATE.md
```

### Demo dataset

Dataset demo phải:

* nhỏ;
* có provenance;
* không yêu cầu phân phối asset không rõ quyền;
* có nhiều loại chart edge case;
* versioned;
* reproducible.

### Release artifacts

* web deployment;
* tagged Git release;
* changelog;
* benchmark report;
* demo video;
* known limitations;
* roadmap 0.2.

## Định nghĩa MVP hoàn thành

Arcaea-Viewer 0.1 được xem là hoàn thành khi:

> Một người dùng có thể tìm một chart trong dataset mẫu, mở chart, xem metadata, phát và scrub visualization, quan sát note/arc theo timeline, xem density analytics và mở lại chart đó khi offline.

---

# 14. Roadmap sau MVP

## Version 0.2 — Replay Foundation

Thời lượng dự kiến: 6–8 tuần.

### Công việc

* replay format RFC;
* replay parser;
* input normalization;
* hit window model;
* deterministic evaluation;
* per-note result;
* ghost rendering;
* replay comparison UI;
* export diagnostics.

### Điều kiện bắt đầu

* parser ổn định;
* timing engine đã được version hóa;
* renderer dùng canonical timeline;
* chart ID và version ID ổn định.

---

## Version 0.3 — Backend và canonical database

Thời lượng dự kiến: 5–7 tuần.

### Thành phần

* Rust REST API;
* PostgreSQL;
* migration system;
* search indexing;
* Meilisearch;
* object storage;
* ingestion jobs;
* API client generation.

### Chỉ triển khai khi

* static snapshot không còn đáp ứng update;
* dataset đã đủ lớn;
* cần synchronization;
* cần server-side search;
* cần replay persistence.

---

## Version 0.4 — Progression và Lore

### Progression

* skill dimensions;
* prerequisite graph;
* target chart analysis;
* rule-based recommendation;
* explainable path generation.

### Lore

* entity graph;
* story chronology;
* character relationships;
* pack/chapter links;
* gameplay unlock context.

---

## Version 0.5 — AI-Assisted Analysis

Chỉ bắt đầu sau khi có:

* analytics baseline;
* versioned feature vectors;
* đủ chart samples;
* evaluation dataset;
* human-labelled comparisons;
* explainability contract.

AI chỉ được phép bổ trợ:

```text
Deterministic analytics = source of truth
AI output              = optional interpretation
```

---

# 15. Backlog Epic

## EPIC-01 Foundation

* monorepo bootstrap;
* shared config;
* CI;
* dependency boundaries;
* WASM hello world;
* release convention.

## EPIC-02 Data Contracts

* chart schema;
* IDs;
* provenance;
* versions;
* fixtures;
* snapshots.

## EPIC-03 Parser

* lexer;
* parser;
* semantic validation;
* diagnostics;
* normalization;
* fuzzing.

## EPIC-04 Timing

* chart time type;
* beat conversion;
* BPM changes;
* offsets;
* range queries;
* deterministic tests.

## EPIC-05 Explorer

* search;
* filters;
* sorting;
* route state;
* chart detail;
* virtualized list.

## EPIC-06 Renderer

* canvas initialization;
* lane;
* tap;
* hold;
* arc;
* timeline;
* controls;
* debug overlays;
* object pooling.

## EPIC-07 Analytics

* count metrics;
* density;
* burst detection;
* arc metrics;
* timing complexity;
* visualization.

## EPIC-08 Offline

* PWA shell;
* IndexedDB;
* query persistence;
* manifest;
* integrity;
* stale state;
* cache controls.

## EPIC-09 Quality

* snapshots;
* integration;
* E2E;
* benchmarks;
* accessibility;
* audit;
* release gate.

---

# 16. Dependency graph

```text
Foundation
   ├── Data Model
   │      ├── Parser
   │      │      ├── Timing Engine
   │      │      │      ├── Renderer
   │      │      │      │      ├── Replay
   │      │      │      │      └── Offline playback
   │      │      │      └── Analytics
   │      │      └── Normalized dataset
   │      │             └── Metadata Explorer
   │      └── API contracts
   └── CI / Quality
```

Replay không được xây dựng trước timing engine.

Analytics không nên đọc dữ liệu thô trực tiếp.

Renderer không được chứa parser semantics.

Frontend không được tự tính logic timing quan trọng.

---

# 17. Quy tắc kiến trúc bắt buộc

## Rule 1

```text
Raw source → Parser → Canonical IR
```

Không subsystem nào được đọc raw chart trực tiếp ngoài parser/ingest.

## Rule 2

```text
Rust timing engine = timing source of truth
```

Frontend không tự tái hiện BPM hoặc offset logic.

## Rule 3

```text
React ≠ render loop
```

Không cập nhật React state 60 lần/giây để di chuyển note.

## Rule 4

```text
Analytics = pure versioned functions
```

Không gắn analytics trực tiếp với UI.

## Rule 5

```text
Every persisted artifact has a version
```

Bao gồm:

* chart schema;
* replay format;
* analytics output;
* cache manifest;
* dataset snapshot;
* API contract.

## Rule 6

```text
No silent parse recovery
```

Mọi recovery phải sinh diagnostic.

## Rule 7

```text
No copyrighted asset dependency for core tests
```

Test và demo phải chạy bằng fixture tự tạo hoặc dữ liệu được phép sử dụng.

---

# 18. Công nghệ đề xuất cho bản đầu

## Frontend

```text
React
TypeScript
Vite
TanStack Query
TanStack Router
Zustand
PixiJS
Dexie
Zod
Vitest
Playwright
```

## Rust

```text
serde
serde_json
thiserror
tracing
wasm-bindgen
tsify hoặc specta
proptest
criterion
cargo-fuzz
```

## Tooling

```text
pnpm
Turborepo hoặc Nx nhẹ
GitHub Actions
Changesets
Docker
```

## Chưa cần

```text
Kubernetes
GraphQL
Kafka
Redis
Terraform đa cloud
Microservices
AI model serving
```

---

# 19. Mô hình issue

Mỗi issue nên có:

```markdown
## Context

## Goal

## Scope

## Non-goals

## Technical approach

## Acceptance criteria

## Tests required

## Performance considerations

## Security/legal considerations

## Dependencies
```

Ví dụ issue:

```markdown
Title: feat(timing): implement BPM segment resolution

Goal:
Resolve the active BPM segment for an arbitrary ChartTime.

Acceptance criteria:
- Correctly resolves before, at, and after segment boundaries.
- Supports multiple events at the same timestamp.
- Does not use floating-point time as source of truth.
- Includes property tests.
- Deterministic across native Rust and WASM.
```

---

# 20. Definition of Done

Một task chỉ được xem là hoàn thành khi:

* code chạy;
* strict type check pass;
* unit tests có mặt;
* error states được xử lý;
* không phá dependency rules;
* public contract có tài liệu;
* benchmark được thêm nếu nằm trong hot path;
* accessibility được kiểm tra nếu có UI;
* provenance/version được xử lý nếu có dữ liệu;
* CI pass.

Một feature không được xem là hoàn thành chỉ vì “đã hiển thị được trên màn hình”.

---

# 21. Rủi ro lớn nhất

## Rủi ro 1: Dữ liệu không rõ nguồn gốc

Biện pháp:

* provenance bắt buộc;
* không phụ thuộc vào asset game gốc để chạy core;
* demo fixtures độc lập;
* tách metadata, derived data và protected media;
* legal/data policy ngay từ đầu.

## Rủi ro 2: Renderer làm sai timing

Biện pháp:

* một canonical time source;
* timing math trong Rust;
* frame state derived từ timestamp;
* visual snapshots;
* debug overlays.

## Rủi ro 3: WASM boundary quá chatty

Biện pháp:

* batch data;
* dùng typed arrays;
* tránh gọi WASM cho từng note mỗi frame;
* generate frame/range result theo lô.

## Rủi ro 4: Overengineering backend

Biện pháp:

* MVP dùng snapshot adapter;
* chỉ thêm PostgreSQL khi có nhu cầu thật;
* giữ repository interface để migration dễ dàng.

## Rủi ro 5: Analytics không đáng tin

Biện pháp:

* bắt đầu bằng descriptive metrics;
* không đưa ra difficulty score quá sớm;
* hiện evidence;
* version algorithm;
* có fixture được tính thủ công.

## Rủi ro 6: Scope quá lớn

Biện pháp:

Mọi feature mới phải trả lời:

> Feature này có cần để hoàn thành luồng chart fixture → parser → renderer → analytics hay không?

Nếu không, chuyển sang backlog sau MVP.

---

# 22. Công việc cần thực hiện ngay

## Tuần đầu tiên

### Ngày 1

* tạo repository;
* khởi tạo pnpm và Cargo workspace;
* thêm README mô tả MVP;
* thêm `.editorconfig`;
* thêm license phù hợp;
* tạo project board.

### Ngày 2

* khởi tạo React/Vite;
* bật TypeScript strict;
* thiết lập lint/format;
* tạo base routing.

### Ngày 3

* khởi tạo Rust crates;
* thiết lập shared domain types;
* tạo Rust unit test đầu tiên.

### Ngày 4

* tạo WASM binding demo;
* React gọi Rust/WASM;
* kiểm tra native và browser output giống nhau.

### Ngày 5

* thiết lập GitHub Actions;
* chạy lint, tests và build;
* bảo vệ main branch.

### Ngày 6

* viết `CHART_IR_DRAFT.md`;
* xác định time unit;
* định nghĩa chart event enums;
* tạo fixture đầu tiên.

### Ngày 7

* review kiến trúc;
* khóa RFC đầu tiên;
* tạo backlog Milestone 1 và 2.

---

# 23. Bản chốt phạm vi

## Arcaea-Viewer 0.1 phải là

> Một interactive chart research viewer có Rust parser, deterministic timing engine, browser visualization, metadata exploration, basic explainable analytics và cached offline access.

## Arcaea-Viewer 0.1 không phải là

* một wiki đầy đủ;
* một clone gameplay;
* một công cụ cheat;
* một asset extraction tool;
* một social platform;
* một sản phẩm AI;
* một hệ thống microservice quy mô doanh nghiệp;
* một replay platform hoàn chỉnh;
* một ứng dụng desktop.

Đây là phạm vi đủ mạnh để dự án có giá trị portfolio, thể hiện Rust, WASM, React, data engineering, graphics, testing và system architecture, nhưng vẫn có khả năng hoàn thành.
