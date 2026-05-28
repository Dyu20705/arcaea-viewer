# Arcaea-Viewer Master Plan

## 1. Tổng Quan

Arcaea-Viewer là một fanmade companion platform cho người chơi Arcaea.
Mục tiêu của dự án là tạo ra một không gian mà người chơi muốn quay lại thường xuyên để xem chart, mở replay, theo dõi tiến độ, khám phá lore, và tận hưởng một trải nghiệm Arcaea được chăm chút kỹ.

Dự án này không được định vị như một research lab, không phải một generalized rhythm-game analysis framework, và không được viết theo tinh thần của một wiki clone.

Foundation-Definition.md là nguồn sự thật về bản sắc sản phẩm. Master plan này chuyển bản sắc đó thành hướng triển khai, kiến trúc và roadmap.

## 2. Mục Tiêu Sản Phẩm

Arcaea-Viewer nên tối ưu cho các trải nghiệm sau:

- xem chart đẹp, rõ, và có chiều sâu thị giác
- mở replay một cách mượt mà, dễ hiểu, và đáng tin cậy
- tìm song, chart, pack, artist, difficulty, và tag một cách nhanh
- hỗ trợ progression planning theo nhu cầu thật của người chơi
- khám phá lore và mối liên kết giữa story, character, pack, và gameplay
- cảm thấy có atmosphere, không khô cứng như một bảng dữ liệu
- có utility thật trong đời sống chơi game hằng ngày
- vẫn hữu ích khi offline hoặc kết nối yếu
- tạo cảm giác đây là một fan platform sống động, không chỉ là công cụ tra cứu

## 3. Experience Identity

Arcaea-Viewer nên mang cảm giác atmospheric hơn là noisy.

The platform should reflect Arcaea's emotional language without copying the game literally. The goal is resonance, not imitation.

The interface should emphasize:

- clarity over clutter
- motion with restraint
- emotional weight through composition
- quiet immersion instead of constant stimulation
- contrast, distance, light, silence, and fragmented memory as design cues

If a screen feels generic, crowded, or engagement-driven, it is probably drifting away from the identity.

## 4. Nguyên Tắc Sản Phẩm

### 3.1 Player Experience First

Mọi quyết định nên trả lời câu hỏi: tính năng này có làm trải nghiệm của người chơi Arcaea tốt hơn không?

Nếu câu trả lời không rõ ràng, tính năng đó nên được xem là optional, experimental, hoặc out of scope.

### 3.2 Practical Over Abstract

Dự án nên hữu ích trước khi nó phức tạp.
Trực giác sản phẩm phải thắng over-engineering.
Các hệ thống phân tích chỉ nên tồn tại khi chúng phục vụ chart viewing, replay understanding, progression, hoặc discovery.

### 3.3 Beautiful But Maintainable

UI và visualization cần có cảm xúc, nhưng không được hy sinh tính bền vững.
Tách rõ domain logic, rendering, state, và data access để project lớn lên mà không vỡ cấu trúc.

### 3.4 Deterministic Where It Matters

Chart parsing, timing, replay evaluation, và derived computations cần tính nhất quán cao.
Kết quả phải có thể tái lập, debug được, và versioned rõ ràng.

### 3.5 Respect the Community

Tone của sản phẩm phải gần gũi với cộng đồng Arcaea.
Không cần startup rhetoric, không cần academic posturing, và không cần pretension.

## 5. Signature Experience

The heart of Arcaea-Viewer is the interactive chart and replay experience.

Chart viewing should feel expressive, readable, and alive.
Replay interaction should feel immersive rather than purely analytical.

This is the feature area that most strongly differentiates the platform from traditional wiki or database sites.

## 6. Core Pillars

### 4.1 Chart Viewer

Đây là một trụ cột trung tâm.
Chart viewer phải đẹp, mượt, dễ đọc, và giúp người dùng hiểu nhịp điệu, arc, hold, flick, và các pattern chính của chart.

### 4.2 Replay Viewer

Replay viewer phải cho phép người chơi xem lại performance, so sánh attempt, hiểu timing, và nhìn rõ các khoảnh khắc quyết định mà không biến trải nghiệm thành một debug console.

### 4.3 Song Explorer

Song explorer nên là nơi duyệt nội dung theo cách tự nhiên: search, filter, cross-link, và khám phá theo pack, version, difficulty, artist, hoặc mechanic feel.

### 4.4 Progression Tools

Công cụ progression phải giúp người chơi chọn mục tiêu hợp lý, hiểu độ khó tương đối, và lên lộ trình luyện tập có ý nghĩa.

### 4.5 Lore Explorer

Lore explorer cần nối được story, character, chapter, và gameplay context thành một trải nghiệm liền mạch.

### 4.6 Community Surfaces

Project nên có các bề mặt cộng đồng vừa phải nhưng có giá trị: curated chart collections, favorite memories, score showcases, challenge lists, progression journals, annotations, hoặc những hình thức tương tác có ích khác.

### 4.7 Recommendation Utilities

Recommendation utilities là lớp hỗ trợ, không phải lõi bản sắc.
Chúng nên giải thích được và phục vụ quyết định của người chơi, không phải thay người chơi quyết định thay.

## 7. Technical Direction

Hướng kỹ thuật nên thực dụng và dễ duy trì:

- clear system boundaries between presentation, data loading, and deterministic logic
- React và TypeScript cho frontend
- feature-oriented architecture thay vì một đống global state
- TanStack Query hoặc một lớp cache tương đương cho server state
- local state chỉ dùng khi thật sự cần cho UX
- Rust cho parsing, timing, replay, và các đường nóng cần tính xác định cao
- WASM chỉ dùng khi nó thật sự tạo giá trị về compute hoặc determinism
- IndexedDB hoặc persistent client storage cho offline-first behavior
- API rõ ràng, versioned, và dễ debug
- module boundaries chặt để chart, replay, lore, và progression không trộn lẫn
- observability và test coverage cho các đường logic quan trọng

Điểm mấu chốt là: dùng engineering nghiêm túc, nhưng không biến sản phẩm thành một enterprise platform.

## 8. Scope Discipline

### 8.1 What the Project Is

- một fanmade companion ecosystem cho Arcaea players
- một immersive interactive platform
- một polished chart, replay, and lore experience
- một player-centric utility platform
- một modern community-oriented Arcaea web experience

### 8.2 What the Project Is Not

- một research lab
- một generalized rhythm-game analysis framework
- một academic visualization system
- một wiki clone
- một copy của sekai.best
- một enterprise data platform mặc định
- một hệ thống mà ML hoặc data science định nghĩa bản sắc chính

### 8.3 Non-Core Areas

Những mảng sau chỉ nên là optional hoặc future extensions nếu chúng thật sự nâng chất lượng trải nghiệm:

- advanced analytics
- AI-assisted insights
- experimental recommendation models
- advanced simulation tools
- deep community moderation systems
- desktop or mobile shells nếu sau này có lý do rõ ràng

## 9. Implementation Shape

### 9.1 Frontend

Frontend nên được chia theo domain surface:

- explore
- chart viewer
- replay viewer
- progression
- lore
- settings
- community surfaces

Mỗi surface nên tự quản lý route, query hooks, local view state, và rendering concerns của nó.

### 9.2 Data Layer

Dữ liệu canonical nên đi qua API và cache layer rõ ràng.
Local persistence chỉ giữ những gì giúp trải nghiệm tốt hơn: recent views, cached entities, offline pages, and selected replay or chart artifacts.

### 9.3 Core Logic

Core logic nên tập trung vào:

- chart parsing
- timing math
- replay evaluation
- note state derivation
- versioned asset and metadata handling

### 9.4 Visualization

Visualization cần được coi là một subsystem nghiêm túc.
Nó phải tách khỏi logic chuẩn hóa dữ liệu để render có thể thay đổi mà không phá domain semantics.

### 9.5 Offline Support

Offline-first là một lợi thế thực tế, không phải khẩu hiệu.
Tối thiểu, app nên hỗ trợ cached browsing, local chart pages, replay sessions đã lưu, và graceful degradation khi mạng kém.

## 10. Roadmap Consolidation

### Phase 1: Core Viewing Foundation

Mục tiêu:

- chốt information architecture
- chốt visual language và design direction
- xác định canonical data model cho songs, charts, packs, story, replays, và user-facing metadata
- dựng responsive UI shell
- làm song database và chart viewer MVP
- thiết lập local storage và cache primitives
- đặt nền cho replay parsing basics

Deliverables:

- responsive UI shell
- canonical entity model
- song database with browse and filter skeleton
- chart viewer MVP
- replay parsing basics
- versioned data contract
- basic offline cache shell

Validation:

- các route cốt lõi mở được ổn định
- data model không trộn lẫn domain
- người dùng có thể duyệt nội dung chính mà không bị vỡ trải nghiệm

### Phase 2: Replay and Progression

Mục tiêu:

- render chart một cách rõ, đẹp, và đáng tin cậy
- xây dựng replay viewer với timeline, scrubber, và comparison mode
- đồng bộ playback, timing overlays, và note state
- thêm progression tracking và practical planning views
- tăng cường search, filtering, và discovery flow

Deliverables:

- replay viewer
- timing overlays
- scrubbing and playback controls
- progression tracking views
- improved search and filtering
- snapshot or image-diff coverage cho các trạng thái quan trọng

Validation:

- chart render nhất quán
- replay playback tái lập được
- UI vẫn mượt khi scrub hoặc đổi mode
- người chơi có thể theo dõi tiến độ và chọn mục tiêu hợp lý

### Phase 3: Lore and Community

Mục tiêu:

- làm lore explorer đủ mạnh để khám phá hằng ngày
- nối lore, pack, character, và gameplay context
- thêm community surfaces có tính fandom hơn là social-media hơn
- giới thiệu recommendation helpers ở mức giải thích được

Deliverables:

- lore explorer with connected entities
- curated chart collections
- favorite memories or score showcase surfaces
- challenge lists or progression journals
- explainable recommendation helpers

Validation:

- người chơi cảm thấy lore và community layers có ích thật
- recommendation helpers hỗ trợ quyết định thực tế
- các liên kết nội dung tạo cảm giác liền mạch

### Phase 4: Polish and Optional Extensions

Mục tiêu:

- hoàn thiện offline experience
- cải thiện accessibility, performance, và polish
- thêm advanced analytics only if it meaningfully helps users
- mở experimental systems ở mức phụ trợ, không ảnh hưởng identity

Deliverables:

- persistent user preferences
- PWA or similar offline behavior nếu hợp lý
- accessibility refinements
- performance tuning cho browsing và playback
- optional advanced analytics
- experimental recommendation or comparison layers

Validation:

- app vẫn hữu ích khi offline hoặc chập chờn mạng
- accessibility và performance không bị bỏ quên
- optional layers không kéo product lệch khỏi player-first identity

## 11. Technical Guardrails

- keep data formats versioned
- keep module boundaries narrow
- prefer deterministic logic for chart, replay, and timing paths
- avoid hidden coupling between UI and core semantics
- treat provenance as a product requirement
- keep legal and community safety in scope
- do not introduce complexity just because it is technically interesting

## 12. Future Direction

Arcaea-Viewer should continue to grow as a place that feels worth spending time in.

The long-term north star is not maximum scale or maximum abstraction.
It is a high-quality fanmade experience that is useful, atmospheric, maintainable, and deeply aligned with how Arcaea players actually explore the game.
