# Arcaea-Viewer: First Step Final

## 1. Executive Intent

Phase 0 exists because Arcaea-Viewer does not yet have enough evidence to justify a broad architecture.
The project has a healthy player-first identity, but identity alone is not a sufficient basis for long-term system design. Before any serious scaling decision, the project must observe how real users behave, what they return for, what they ignore, where they get confused, and which parts of the ecosystem create recurring friction.

This document defines the first deployable research foundation only.
It is not a final roadmap.
It is not a long-term platform blueprint.
It is not a promise that future systems already exist conceptually just because they are desirable.

The first deployment must deliberately resist feature explosion, startup-style scaling assumptions, unnecessary distributed systems, premature AI systems, and social-platform ambitions.
Those choices are not signs of weakness. They are the correct response to uncertainty.

Arcaea-Viewer cannot safely design long-term systems before observing:

- actual community behavior
- retention patterns
- user interaction loops
- chart exploration behavior
- replay usage behavior
- metadata navigation patterns
- performance constraints
- content licensing boundaries
- update cadence of the game ecosystem
- emotional engagement drivers
- community fragmentation
- creator participation willingness
- sustainability of maintenance

Later phases depend on observations collected during the first deployment.
Later phases depend on observations collected during the first deployment.
Later phases depend on observations collected during the first deployment.

The purpose of Phase 0 is therefore narrow and practical:

- establish a serious minimum viable research platform
- keep the system small enough to observe clearly
- collect behavior data with minimal distortion from unnecessary complexity
- validate whether the companion platform concept creates real recurring value
- identify which assumptions are false before they harden into architecture

The initial system should be treated as a controlled deployment, not as a declaration of final form.
A technically impressive system that cannot answer the right questions is worse than a smaller system that can.

## 2. What The First Step Actually Is

The first step is a constrained, offline-capable, visualization-focused companion environment for Arcaea content.
Its job is to let people inspect charts, replay material, and structured metadata with enough quality to reveal how they naturally navigate the ecosystem.

In concrete terms, the first version should behave like:

- an offline-capable chart exploration platform
- a metadata viewer with stable navigation and search
- a chart and replay inspection tool
- a structured content index for songs, packs, characters, and related records
- a visualization-focused research platform for observing actual usage

It should not behave like:

- a full ecosystem replacement
- a live service platform with large operational surface area
- a social network
- a creator economy
- a ranking system
- a recommendation-heavy personalization engine
- a competitive platform
- a generalized rhythm-game framework

The first step solves a very specific problem: it gives Arcaea-Viewer enough shape to be used, measured, and criticized.
That is the point.

The first step intentionally does not solve everything else.
It does not attempt to unify the entire Arcaea ecosystem.
It does not attempt to make the project self-sustaining through social loops.
It does not attempt to predict user preferences with complex models.
It does not attempt to pre-build a future that has not been validated.

The exact system boundary of Phase 0 should be understood as follows:

Included:

- chart browsing
- structured metadata navigation
- chart visualization
- replay representation and replay inspection
- local storage for durable browsing state
- minimal offline capability
- ingest-and-validate pipeline for selected data subsets
- logging and observability sufficient for behavior analysis

Excluded:

- large-scale account systems
- social feeds
- cloud sync as a default dependency
- collaborative editing
- mod ecosystems
- creator tooling
- ranking systems
- machine learning systems
- broad recommendation systems
- multiplayer or community interaction systems with operational complexity

What the first step solves immediately:

- it creates a credible place to inspect charts and related metadata
- it gives the project a stable surface for observing navigation behavior
- it makes replay learning and chart exploration visible enough to study
- it allows the team to measure maintenance cost before committing to expansion

What it intentionally does not solve:

- the full long-term architecture
- retention strategy beyond observation
- ecosystem-wide social cohesion
- content completeness across every possible Arcaea artifact
- final decisions about future service boundaries
- any assumption that the project will or should scale in a linear, platform-like way

The first step should be small enough to understand and serious enough to matter.

## 3. Minimum Core Systems

The minimum core systems should exist only because they help answer the research questions of Phase 0.
Every subsystem must justify its own operational burden.
If a subsystem cannot produce useful observations, it is probably too early.

### 3.1 Local-First Architecture

A local-first architecture is the correct default for Phase 0 because it reduces operational load, lowers failure modes, and makes the first deployment usable even when connectivity is poor or absent.

Why it exists:

- Arcaea-Viewer should remain usable in a limited but meaningful offline state.
- Local persistence helps reveal what users actually come back to.
- A local-first design avoids premature dependence on distributed infrastructure.

Why it is minimal:

- It avoids the complexity of real-time multi-client synchronization.
- It avoids forcing the first version to solve account, session, and replication concerns before they are understood.
- It keeps the observability boundary close to the user.

Risks:

- local caches can become stale or misleading if update semantics are unclear
- offline storage can create duplicate logic if not carefully bounded
- users may assume completeness where only partial data exists

Scaling implications:

- if local-first behavior proves useful, future sync can be added around an already validated offline model
- if it proves weak, the project avoids having built a complex sync layer for no reason

Future unknowns:

- how often users need offline browsing versus online freshness
- whether replay and chart data are primarily read-many or session-bounded
- how much local storage is acceptable before maintenance becomes painful

### 3.2 Data Ingestion Pipeline

A data ingestion pipeline is necessary, but only as a reproducible and inspectable path from source to normalized records.

Why it exists:

- the platform needs a canonical way to turn raw or semi-structured Arcaea-related data into usable internal records
- ingestion is where licensing boundaries, provenance, and correctness must be visible

Why it is minimal:

- the pipeline should support a limited subset of sources first
- the goal is to observe data quality and update frequency, not to ingest everything available
- complex multi-source ETL should wait until real source behavior is measured

Risks:

- source instability can break assumptions silently
- community-maintained data may be inconsistent or outdated
- overgeneralized ingestion logic tends to become hard to maintain quickly

Scaling implications:

- if the first pipeline is reproducible and understandable, later sources can be added without guessing
- if the first pipeline becomes fragile, expansion will magnify its defects

Future unknowns:

- which sources remain stable enough to depend on
- how often the game ecosystem changes in ways that force re-ingestion
- what level of normalization is actually useful to users versus useful only to engineers

### 3.3 Chart Parser Layer

The chart parser is one of the few places where deterministic behavior is essential from day one.

Why it exists:

- chart structure is the basis of visualization, navigation, and replay inspection
- users cannot be asked to trust chart views if the parser is ambiguous or inconsistent

Why it is minimal:

- the parser should only cover the chart subset necessary for observation and inspection
- it should validate rather than over-interpret
- it should preserve source provenance instead of hiding it

Risks:

- false certainty from incomplete parsing rules
- drift between raw chart source and normalized representation
- overfitting parser behavior to a narrow known dataset

Scaling implications:

- a deterministic parser gives the project a stable core for future visualization and replay logic
- parser complexity should only grow in response to observed real content patterns

Future unknowns:

- what chart variants are actually encountered in the ecosystem
- which edge cases are common enough to justify parser expansion
- whether users primarily need fidelity, debugging, or comparison

### 3.4 Metadata Normalization

Metadata normalization is required because users cannot explore fragmented records efficiently.

Why it exists:

- Arcaea-related knowledge is scattered across community sources and informal conventions
- the project needs a stable internal model for songs, packs, charts, characters, and related entities

Why it is minimal:

- the normalized schema should cover only the records needed for Phase 0 usage
- it should not attempt to solve every future relationship upfront
- it should support inspection and comparison, not encyclopedic completeness

Risks:

- overly rigid schemas can lock in incomplete assumptions
- overly flexible schemas can erase structure and make querying unreliable
- metadata change rate may be higher than expected

Scaling implications:

- if the first normalized model is good, later entity types can be added with confidence
- if the first model is bloated, future migrations will become expensive and risky

Future unknowns:

- which metadata fields are revisited most by users
- whether players care more about search, context, or comparison
- which relationships are worth canonicalizing and which should remain derived

### 3.5 Replay Representation

Replay representation should exist in Phase 0, but only as a disciplined model for observation and inspection.

Why it exists:

- replay usage is one of the key unknowns of the project
- the platform needs to know whether replay tooling creates meaningful return visits

Why it is minimal:

- the replay model should support inspection, not a full competitive replay ecosystem
- the first version should focus on representation, visualization, and comparison primitives
- replay processing should be bounded to manageable cases

Risks:

- replay formats may be inconsistent, incomplete, or harder to standardize than expected
- users may interpret replay views as authoritative even when data quality is partial
- replay tooling can become maintenance-heavy if it expands too early

Scaling implications:

- if replay inspection proves valuable, the model can later support richer analysis
- if it proves low-value, the project avoids committing to a costly subsystem with weak retention

Future unknowns:

- whether users use replay views for learning, curiosity, comparison, or simple entertainment
- how often replays are revisited relative to charts and metadata
- whether replay interaction increases retention or merely satisfies a niche interest

### 3.6 Asset Indexing

Asset indexing is necessary because visual content is part of the user experience, but it must remain restrained.

Why it exists:

- images, chart assets, icons, and related resources need stable lookup and caching behavior
- users should not feel that the platform breaks when visual assets are unavailable

Why it is minimal:

- the asset index should only cover what the first deployment actually displays
- broad media management is unnecessary at this stage
- the system should prefer integrity over breadth

Risks:

- asset caches can grow quickly
- licensing boundaries may restrict what can be stored or distributed
- asset mismatch can silently damage trust in the platform

Scaling implications:

- a disciplined index makes later asset expansion possible without chaos
- a sloppy one tends to become an expensive storage and invalidation problem

Future unknowns:

- which assets are most essential to recognition and atmosphere
- how much caching is actually needed to support smooth browsing
- which asset types are safest and most sustainable to retain locally

### 3.7 Rendering Pipeline

The rendering pipeline should be treated as a research instrument, not only as presentation code.

Why it exists:

- visualization is the primary way the project can make chart and replay behavior legible
- the rendering surface is where correctness and usability intersect

Why it is minimal:

- the first version should focus on reliable chart display, not an elaborate cinematic engine
- the pipeline should make behavior visible without obscuring it with excessive effects
- the system should support a limited number of high-value views rather than many speculative scenes

Risks:

- rendering complexity can explode if every visual idea becomes a core requirement
- performance regressions can hide inside visually attractive changes
- animation systems often create maintenance burden disproportionate to their value

Scaling implications:

- if the first rendering model is stable, later visual polish can be added incrementally
- if the first model is unstable, all future visualization work becomes risky

Future unknowns:

- what visualizations are actually useful versus merely impressive
- how much motion helps versus distracts
- whether the platform’s value is better expressed through clarity or atmosphere, and in which proportions

### 3.8 Storage Strategy

Storage strategy must be simple enough to understand and constrained enough to be safe.

Why it exists:

- the platform needs durable local state for browsing, inspection, and limited offline use
- without storage, the first deployment cannot reveal return behavior reliably

Why it is minimal:

- local persistence should only hold data necessary for Phase 0 use cases
- storage should not be designed around speculative massive datasets
- the first system should prioritize recoverability and inspectability over abstraction

Risks:

- stale local state can mislead users
- storage growth can become operationally messy
- migration complexity can appear quickly if the schema is not disciplined

Scaling implications:

- a bounded storage model makes future sync and retention analysis possible
- a chaotic storage model turns every later improvement into a migration problem

Future unknowns:

- what portion of usage is truly session-bound versus persistent
- whether local assets or metadata matter more for retention
- how much storage is acceptable before users notice burden

### 3.9 Update Synchronization Strategy

Update synchronization should be treated as a low-frequency, controlled process in Phase 0, not as a real-time platform feature.

Why it exists:

- the content ecosystem changes over time
- the project must know how to refresh records without breaking trust

Why it is minimal:

- the first step should support explicit refresh or versioned ingestion cycles
- automatic continuous sync is premature without usage evidence
- the platform should observe update cadence before building elaborate invalidation logic

Risks:

- stale data can undermine confidence
- overcomplicated sync can create hidden failure modes
- update assumptions may differ significantly from actual ecosystem behavior

Scaling implications:

- if update cadence is understood early, later automation can be chosen deliberately
- if it is guessed, the project may build the wrong maintenance model entirely

Future unknowns:

- how frequently important data changes
- whether users care more about freshness or consistency
- what parts of the ecosystem require fast updates versus stable archival treatment

### 3.10 Observability and Logging

Observability is not a support feature. It is a core research requirement.

Why it exists:

- the project must capture actual navigation and usage patterns
- without observability, the team will only have opinions

Why it is minimal:

- logs and metrics should be focused on the research questions of Phase 0
- the system does not need a complex telemetry platform to start
- basic event capture, diagnostics, and failure tracing are enough to begin with

Risks:

- too much logging can create privacy and maintenance issues
- too little logging will make the deployment non-informative
- noisy metrics can conceal the signals that matter

Scaling implications:

- if observability is designed early, future decisions can be evidence-based
- if observability is neglected, later scaling decisions will be made blind

Future unknowns:

- what behaviors actually deserve instrumentation
- how much qualitative context is necessary alongside metrics
- which signals predict usefulness versus superficial novelty

## 4. Research-Driven Product Philosophy

Phase 0 must be explicit about what is known, what is assumed, what is hypothesized, and what remains unknown.
This separation is essential because future architecture decisions depend on these answers.
Later phases depend on observations collected during the first deployment.

### 4.1 Known Facts

These are the facts that can be treated as starting constraints:

- Arcaea has a distinct emotional and visual identity.
- The ecosystem around the game is fragmented.
- Players often rely on scattered community sources for information.
- Chart exploration and replay understanding are meaningful activities.
- Lore matters emotionally, but is often not easy to navigate.
- The project should remain fanmade and practical rather than trying to replace the game.

These facts are enough to justify a first deployment.
They are not enough to justify a final architecture.

### 4.2 Assumptions

These are reasonable but still unverified:

- users will want a companion space rather than a pure database
- chart and replay browsing can support return visits
- atmospheric presentation will increase perceived value without harming clarity
- local-first behavior will improve usability enough to justify the storage cost
- curated metadata navigation is more useful than broad platform abstraction
- a small research-oriented foundation can reveal what the community actually wants

Every assumption must be treated as provisional.
If observed behavior contradicts an assumption, the architecture must adapt.

### 4.3 Hypotheses

These are the specific propositions that Phase 0 should test:

- users revisit chart pages for different reasons than they revisit lore pages
- replay tools create stronger learning behavior than simple chart browsing alone
- metadata clarity matters more than feature count for early trust
- offline capability improves perceived usefulness even when most sessions are online
- atmosphere improves retention only if it does not reduce navigation speed
- community warmth is more sustainable than high-frequency social interaction
- users value explainable structure more than broad automation

The first deployment exists to make these hypotheses observable.

### 4.4 Unknowns

These are the uncertainties that must remain open until real usage exists:

- whether the platform becomes a chart reference tool, a replay learning tool, a lore navigation tool, or a mixture of the three
- what content types create the highest return-visit probability
- how much maintenance the ecosystem will require
- whether the community is willing to contribute curated material
- which visualizations are actually used versus merely admired
- how much operational complexity the project can tolerate before becoming brittle
- whether long-term retention depends more on utility, atmosphere, or the combination of both

### 4.5 Metrics That Matter

The correct metrics in Phase 0 are not vanity metrics.
They should describe behavior, friction, and sustainability.

Useful metrics include:

- repeat visits to specific entity types
- navigation path depth and route revisitation
- chart detail dwell time
- replay open rate and replay revisit rate
- metadata search success rate
- abandonment points in key workflows
- offline usage rate
- stale-data complaints or refresh requests
- error rates by subsystem
- maintenance hours per content update
- user-reported usefulness by feature class

These metrics matter because they tell the team whether the platform is becoming a meaningful companion or merely a visually polished index.

### 4.6 Qualitative Observations

Quantitative metrics alone are insufficient.
The project must also watch for qualitative signals such as:

- confusion around chart labels or navigation hierarchy
- users asking for the same context repeatedly
- users treating the platform as a one-time lookup versus a return space
- community members describing the platform in unexpectedly narrow or broad terms
- repeated requests for features that would radically change the product model
- mismatch between the intended atmosphere and the actual user reaction

Future architecture decisions depend on these observations.

## 5. Deliberately Deferred Systems

The following systems are intentionally postponed.
They are not rejected permanently.
They are delayed because building them before observation would create unnecessary risk, complexity, and false certainty.
For each deferred system, later phases depend on observations collected during the first deployment.

### 5.1 Competitive Infrastructure

Deferred because the project does not yet know whether users want competition-oriented behavior outside the game itself.
Building competitive systems too early introduces ranking pressure, fairness expectations, and moderation burden before the core companion model has been proven.

Observations required before building:

- whether users are asking for competition-related workflows
- whether replay comparison naturally evolves into ranking behavior
- whether the community wants performance ladders or just learning tools

Premature risks:

- incentive distortion
- social comparison pressure
- moderation complexity
- design drift away from the companion-space identity

### 5.2 Social Systems

Deferred because generic social systems tend to consume the product identity.
Feeds, follows, comments, and engagement loops are expensive and frequently produce more noise than value.

Observations required before building:

- whether users actually want persistent social interaction inside the platform
- whether curated community surfaces are sufficient
- whether people return for content or for other people

Premature risks:

- platform bloat
- moderation burden
- engagement mechanics that distort the product tone
- infrastructure cost without proven retention payoff

### 5.3 Recommendation Systems

Deferred because recommendation systems are easy to build badly and expensive to maintain well.
The project should first learn what users search for, revisit, and ignore.

Observations required before building:

- which chart attributes matter to players during discovery
- which content leads to successful follow-through
- whether users trust algorithmic suggestions at all

Premature risks:

- opaque outputs
- misleading suggestions
- maintenance cost for weak or unstable value
- false confidence in inferred preference models

### 5.4 Account Systems

Deferred because account systems add privacy, authentication, profile, retention, and data governance complexity immediately.
Phase 0 should not force identity management before user value is proven.

Observations required before building:

- whether users need persistent identity or just stable local persistence
- whether any cross-device behavior is actually important
- whether saved progress requires server authority

Premature risks:

- authentication overhead
- account lifecycle burden
- recovery and privacy concerns
- operational complexity without validated benefit

### 5.5 Cloud Sync

Deferred because sync only makes sense after the first deployment reveals genuine multi-device or collaboration needs.

Observations required before building:

- whether local-first usage is enough
- whether users care about cross-device continuity
- whether content freshness matters enough to justify replication logic

Premature risks:

- conflict resolution complexity
- synchronization bugs
- stale or duplicated state
- maintenance burden that exceeds early value

### 5.6 Large-Scale APIs

Deferred because large public API surfaces invite versioning obligations, abuse concerns, and long-term compatibility commitments.
The first deployment should stay close to the core interface that is actually used.

Observations required before building:

- which data surfaces people actually need programmatic access to
- whether external integrations are requested at all
- whether the internal data model is stable enough to expose

Premature risks:

- unnecessary support burden
- contract rigidity
- security exposure
- architectural overcommitment

### 5.7 Mod Ecosystems

Deferred because modding support introduces compatibility, distribution, and community governance challenges that are unrelated to the first research objective.

Observations required before building:

- whether users want customization or just access
- whether third-party extensibility would actually improve utility
- whether the base product is stable enough to be extended safely

Premature risks:

- fragmentation
- support complexity
- compatibility failures
- scope dilution

### 5.8 Creator Tooling

Deferred because creator tools assume a mature participation model that has not yet been validated.
The project must first determine whether there is sustained willingness to contribute curated content.

Observations required before building:

- whether community members want to annotate, curate, or publish content
- what content types are actually contributed voluntarily
- how much moderation and review would be required

Premature risks:

- editorial burden
- content quality inconsistency
- workflow complexity
- false assumption of creator participation

### 5.9 Ranking Systems

Deferred because ranking systems strongly alter user behavior and create fairness, measurement, and motivational effects that cannot be undone easily.

Observations required before building:

- whether users need rankings or simply comparative context
- whether chart difficulty visibility already solves enough of the problem
- whether ranking would help or harm the companion-space tone

Premature risks:

- unhealthy comparison pressure
- maintenance of fairness rules
- game-like escalation that distracts from companionship

### 5.10 Machine Learning Systems

Deferred because ML systems are often used to simulate insight before actual observations exist.
That is exactly the wrong order.

Observations required before building:

- which behaviors are stable enough to model
- whether deterministic heuristics already answer the user need
- whether there is sufficient data quality to justify learning-based inference

Premature risks:

- opaque decision-making
- data hunger
- overfitting to incomplete signals
- maintenance cost without operational clarity

### 5.11 Collaborative Systems

Deferred because collaboration requires identity, conflict handling, permissions, moderation, and versioning logic that should not be invented before the first deployment has evidence of shared usage.

Observations required before building:

- whether users actually want shared collections or joint annotation
- how much coordination the community can sustain
- whether collaboration improves return visits

Premature risks:

- permissions complexity
- conflict resolution
- moderation burden
- unnecessary coupling to account systems

### 5.12 Why Deferral Is Strategic, Not Conservative

Deferral is not a refusal to innovate.
It is a method for preventing architecture from outrunning evidence.

Each deferred system becomes justified only if the first deployment produces the observations needed to support it.
Later phases depend on observations collected during the first deployment.

## 6. Technical Architecture Principles

The first deployment must obey strict engineering principles because the whole purpose of Phase 0 is to learn without self-sabotage.
The architecture should be research-friendly, deterministic, and cheap to reason about.
It should not be a miniature enterprise system.

### 6.1 Deterministic Pipelines

Parsing, normalization, replay representation, and derived chart views should behave deterministically.
Repeated inputs should produce repeated outputs.
This is necessary for reproducibility and trust.

Determinism matters because if a result changes unexpectedly, the team cannot tell whether the ecosystem changed or the system did.

### 6.2 Reproducibility

Ingestion and derived artifact generation must be replayable from known sources.
The system should be able to explain how a record was produced.
This is not an optional luxury.
It is the only way to make the first deployment auditable.

### 6.3 Inspectable Data

All meaningful data should remain inspectable in some human-readable or at least human-auditable form.
A black-box pipeline will hide the assumptions Phase 0 is trying to test.

### 6.4 Modular Ingestion

The ingestion path should be modular enough to add or remove sources without rewriting the entire system.
However, modular does not mean broad.
The system should be narrow first, then modular only where the first deployment requires it.

### 6.5 Isolation Boundaries

Presentation, ingestion, parsing, storage, and observability should remain distinct.
If the UI starts deciding chart semantics, or the parser starts depending on presentation state, the architecture will become difficult to evolve.

### 6.6 Offline-First Behavior

Offline capability is valuable because it proves the platform can remain useful outside ideal conditions.
It also lowers dependency on operational infrastructure.
But offline-first should be bounded to the features that truly need it.

### 6.7 Graceful Degradation

When a record is missing, stale, or unavailable, the system should fail visibly and informatively.
Silent failure is unacceptable.
Overconfident failure is also unacceptable.
The user should understand what is known, what is missing, and what may return later.

### 6.8 Long-Term Maintainability

The first deployment must be maintainable by a small team.
That means avoiding unnecessary abstractions, minimizing cross-cutting state, and preferring explicitness over cleverness.

A system that requires a large team to keep alive before it has proven value is the wrong system.

### 6.9 Low Operational Complexity

Phase 0 should minimize:

- background services
- distributed dependencies
- runtime coordination surfaces
- administrative dashboards
- ambiguous operational states

Operational simplicity is not a weakness; it is what allows real observation.

### 6.10 Observability-First Design

Logging, metrics, and traces should be built in from the start, but only for the questions that matter.
The first deployment must let the team see:

- what users opened
- what they revisited
- where they dropped off
- what was stale
- what failed
- what was actually useful

Without observability, every later expansion is guesswork.

### 6.11 Explicit Rejection of Infrastructure Hype

The first step should explicitly reject:

- unnecessary microservices
- premature distributed architecture
- “platform” decomposition for its own sake
- infrastructure that exists only to look mature
- operational complexity before user value is proven

If a simpler architecture can answer the same research question, the simpler architecture wins.

## 7. Initial User Experience Philosophy

The first user experience should feel calm, legible, and intentional.
It should not feel like a feature-complete dashboard.
It should feel like a careful environment for exploration, inspection, and return visits.

### 7.1 What the First Experience Should Feel Like

The first experience should communicate:

- this space understands Arcaea’s atmosphere
- this space helps me orient myself
- this space does not waste my attention
- this space gives me useful context without overwhelming me
- this space is stable enough to trust

### 7.2 Acceptable Friction

Some friction is acceptable in Phase 0 if it protects clarity and reduces scope:

- limited data coverage
- some manual navigation between records
- constrained feature breadth
- partial offline availability
- conservative design choices
- simple interactions that are not yet heavily personalized

This friction is acceptable because the first deployment is a learning system, not a mature platform.

### 7.3 Unacceptable Friction

Some friction would be a failure:

- confusing navigation hierarchy
- unclear chart or replay meaning
- unstable data freshness
- broken return paths
- opaque loading states
- poor performance in basic browsing
- excessive reliance on speculative future systems
- any UX that feels like a generic database with a skin on top

### 7.4 Primary Workflows

The most important workflows in Phase 0 are:

- find a song or chart quickly
- inspect structured metadata
- open a chart view and understand what is being shown
- open a replay view and understand what is being compared
- move from one piece of information to a related piece without losing context
- return to previously viewed content and recognize what changed

### 7.5 UX as an Observation Instrument

The user experience is also the measurement instrument.
If the UI is too complex, users will fail for reasons unrelated to the ecosystem.
If the UI is too shallow, the team will not learn enough.
The experience must therefore be balanced: simple enough to use, rich enough to observe.

## 8. Success Criteria For Phase 0

Phase 0 success must not be measured by vanity metrics.
The goal is not growth theater.
The goal is evidence.

### 8.1 Technical Success Criteria

Phase 0 is successful if:

- ingestion is stable and reproducible
- the parser is deterministic on the supported dataset
- metadata normalization remains inspectable
- chart and replay views are reliable enough for repeated use
- offline or local persistence behaves predictably
- the system can be maintained without constant emergency intervention

### 8.2 Behavioral Success Criteria

Phase 0 is successful if:

- users revisit specific content types repeatedly
- navigation patterns show genuine exploration, not one-time curiosity
- chart and replay views produce repeat use rather than novelty-only use
- users ask for context that was not initially obvious
- the platform helps users make choices they could not make easily elsewhere

### 8.3 Research Success Criteria

Phase 0 is successful if it answers the following with evidence rather than speculation:

- how users navigate chart information
- which metadata users revisit most
- whether replay systems create retention or only occasional interest
- whether the platform is used as a wiki, analysis tool, or practice tool
- what features naturally create return visits
- which systems become maintenance-heavy fastest
- what content becomes outdated most frequently
- what visualizations are genuinely useful versus novelty

### 8.4 Sustainability Success Criteria

Phase 0 is successful if the maintenance burden remains realistic for the team size.
That means the platform can survive normal updates, data refreshes, bug fixes, and modest feature corrections without becoming a permanent operational drain.

### 8.5 Feedback Quality Criteria

Useful feedback matters more than positive feedback.
The first deployment succeeds if user feedback becomes more specific over time.
That indicates the platform is legible enough for people to critique productively.

### 8.6 Architecture Survivability Criteria

The architecture is successful if future decisions can be made without tearing down the entire foundation.
If every new observation requires a rewrite, the first step failed.

## 9. Failure Conditions

This section is mandatory because realistic planning requires explicit failure modes.
The first deployment can fail in several distinct ways.

### 9.1 Technical Failure

The project can fail technically if:

- parsers become brittle
- data normalization becomes inconsistent
- rendering becomes slow or unstable
- local storage is unreliable
- the codebase grows faster than understanding
- observability is insufficient to diagnose issues
- the architecture becomes too clever for its own maintenance budget

Technical failure is especially dangerous because it often looks like progress until the system starts resisting change.

### 9.2 Operational Failure

The project can fail operationally if:

- updates require too much manual intervention
- content refreshes become fragile
- bug resolution takes longer than expected
- platform upkeep outweighs user value
- maintenance tasks depend on knowledge held by too few people

A small project can survive a lot of complexity if it is coherent.
It cannot survive operational confusion for long.

### 9.3 Scope Creep Failure

Scope creep is one of the fastest ways to destroy Phase 0.
The project can fail if it tries to become:

- a social platform too early
- a recommendation engine too early
- a creator ecosystem too early
- a competitive system too early
- a cloud-synced identity platform too early

Each of those additions would demand its own assumptions, support burden, and maintenance model.
If the first deployment has not yet observed enough behavior, those additions are premature.

### 9.4 Maintenance Burden Failure

The project can fail if the upkeep cost becomes larger than the value it creates.
This can happen even when the product is well-designed if the data ecosystem is too fragmented, the update cadence is too high, or the source material is too unstable.

The team must watch for the point where the platform starts requiring work simply to stay coherent.
That is a serious warning sign.

### 9.5 Ecosystem Dependence Failure

The project is vulnerable to the health of the Arcaea ecosystem around it.
If community activity, source availability, content licensing, or update patterns shift in ways that make curation harder, the platform may become less sustainable than expected.

This is why Phase 0 must observe ecosystem dependence before long-term commitments are made.

### 9.6 Misread Retention Failure

The project can also fail if it misreads curiosity as retention.
A user may explore the platform once, enjoy it, and never return.
That is not the same as proving lasting value.

Only repeated behavior can justify later expansion.

### 9.7 Atmospheric Drift Failure

Because Arcaea has a specific emotional language, the product can fail by becoming generic.
If the UI becomes noisy, over-animated, or cluttered, the project will lose part of the identity it is trying to support.

Atmosphere is not decoration.
It is part of the product thesis.
If the atmosphere drifts, the platform may still function technically while losing its reason to exist.

## 10. Long-Term Transition Logic

Phase 0 must be designed so that future phases are conditional, not assumed.
The first deployment should generate evidence that determines whether expansion is justified.
Later phases depend on observations collected during the first deployment.

### 10.1 What Can Become Phase 1 and Beyond

Only after enough evidence exists should the project consider expansion into areas such as:

- broader personalization
- more advanced replay tools
- more complete community surfaces
- more robust progression support
- optional recommendation logic
- stronger update workflows
- selective cloud-backed features

These are not guaranteed outcomes.
They are conditional responses to observed need.

### 10.2 Conditions for Expansion

A future phase is justified only if:

- the first deployment has stable usage evidence
- the maintenance burden remains acceptable
- the architecture survives real usage without repeated structural damage
- the ecosystem demand is real, not imagined
- the feature would clearly reduce friction or increase understanding
- the new subsystem can be added without distorting the companion-space identity

### 10.3 Conditions for Non-Expansion

A future phase should not happen if:

- the existing platform already answers the core use cases well enough
- the proposed subsystem would introduce more burden than value
- the ecosystem does not demonstrate need
- the feature would pull the project toward platform ambition instead of companion usefulness
- the observations are still too weak to support it

Refusing to expand is a valid outcome.
That is what disciplined research planning looks like.

### 10.4 Transition as Evidence-Based Design

Future architecture decisions depend on the first deployment’s observations.
This should be treated as a strict rule.
No major expansion should be justified by desire alone.
It must be justified by observed behavior, maintenance reality, and ecosystem demand.

### 10.5 Final Position

The first step is not a smaller version of the final product.
It is the only responsible way to discover what the final product should become.

If the project learns that users mainly want chart inspection, it should lean there.
If it learns that replay learning matters most, it should lean there.
If it learns that lore navigation is the primary return driver, it should lean there.
If it learns that community warmth matters more than breadth, it should lean there.

That is the purpose of Phase 0: to force the future to earn its existence.
