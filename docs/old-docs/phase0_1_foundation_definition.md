# Arcaea-Viewer Foundation Definition

## 1. Project Identity

Arcaea-Viewer is a fanmade companion platform for Arcaea players.
It is built to help people explore the game, enjoy its visual language, understand charts and replays, follow progression goals, and spend time inside a polished Arcaea-themed experience.

It is not a research lab, not a generalized rhythm-game analysis framework, not a wiki clone, and not a copy of sekai.best.

The product should feel like a place Arcaea players actually want to return to, not a database they visit once and leave.

## 2. Experience Identity

Arcaea-Viewer should feel atmospheric rather than noisy.

The interface should reflect the emotional language of Arcaea without copying the game literally. The goal is not imitation, but resonance.

The product should emphasize:

- clarity over clutter
- motion with restraint
- emotional weight through composition
- quiet immersion instead of constant stimulation
- contrast, distance, light, silence, and fragmented memory as design cues

Arcaea has a specific mood: lonely, luminous, fragmented, and emotionally ambiguous.
That mood should be visible in layout, spacing, typography, motion, and visual hierarchy.

## 3. Core Experience Goals

Arcaea-Viewer should consistently deliver the following experiences:

- useful daily tools for browsing songs, charts, and progression targets
- smooth and beautiful chart viewing
- replay viewing that feels clear, responsive, and emotionally satisfying
- lore exploration that connects story, characters, packs, and gameplay context
- progression support that helps players choose practical next goals
- community-oriented surfaces that feel like a fandom companion, not a social feed
- strong atmosphere through layout, typography, motion, and visual composition
- offline-friendly behavior where it improves real player usefulness

The experience should be practical first, but never plain.

## 4. Product Philosophy

The product philosophy is simple: player experience first.

That means:

- prioritize clarity, delight, and usefulness over abstract completeness
- make chart and replay interactions feel trustworthy and pleasant
- keep interfaces discoverable without making them shallow
- use data to support the player, not to turn the project into a research display
- keep the tone passionate, grounded, and respectful to the Arcaea community

If a feature does not make the player experience meaningfully better, it should not be core.

## 5. Signature Experience

The heart of Arcaea-Viewer is the interactive chart and replay experience.

Chart viewing should feel expressive, readable, and alive.
Replay interaction should feel immersive rather than purely analytical.

This is the feature area that most strongly differentiates the platform from traditional wiki or database sites.

## 6. Core System Pillars

The foundation of the product is built around a small set of durable pillars.

### 4.1 Chart Viewer

A polished chart viewer is one of the central experiences.
It should support clear note reading, timeline control, layered visual feedback, and a presentation style that makes Arcaea charts feel alive.

### 4.2 Replay Viewer

Replay viewing should allow players to inspect performance, compare attempts, and understand timing behavior without feeling like they are opening a technical debugger.

### 4.3 Song Explorer

Song browsing should be fast, rich, and enjoyable.
Users should be able to search and filter by practical fields such as title, difficulty, version, pack, artist, chart type, and mechanical feel.

### 4.4 Progress Tracking

The platform should help players make sense of their progression.
That includes practical planning tools, goal setting, and chart-to-chart comparison that supports improvement.

### 4.5 Lore Explorer

Lore should be treated as a meaningful part of the experience, not an afterthought.
The product should connect story content, characters, packs, and gameplay context in a way that feels coherent and exploratory.

### 4.6 Community Features

Community-oriented features should reinforce the feeling of a living fan platform without turning the product into a generic social network.
This can include curated chart collections, favorite memories, score showcases, challenge lists, progression journals, annotations, and other fandom-friendly layers when they fit the project scope.

### 4.7 Recommendation Utilities

Recommendation logic should stay practical and explainable.
Its job is to help players discover useful next steps, not to become an opaque scoring system.

## 7. Technical Direction

The technical direction should stay practical, maintainable, and easy to evolve.

The core engineering philosophy is to keep system boundaries clear:

- presentation should stay separate from chart and replay semantics
- data loading should stay separate from render state
- deterministic logic should remain easy to test and version
- offline persistence should be explicit, not accidental
- optional advanced subsystems should remain optional

Recommended principles:

- React and TypeScript for the frontend experience
- clear feature boundaries instead of one oversized application layer
- TanStack Query or a similar cache layer for server state and synchronization
- localized state only where it clearly improves UX
- Rust for deterministic chart, timing, parsing, and replay logic where it provides real value
- WASM only for compute-heavy or determinism-sensitive hot paths
- offline-first support through persistent client storage where it improves player value
- versioned data contracts so charts, replays, and derived assets remain stable over time
- clear module boundaries so the project does not collapse into a single oversized application layer

The project should be engineered like a serious product, but not like an enterprise platform.

## 8. Scope Discipline

This section defines the hard boundary of the project.

### 8.1 What Arcaea-Viewer Is

- a fanmade companion ecosystem for Arcaea players
- an immersive interactive platform
- a polished chart, replay, and lore experience
- a player-centric utility platform
- a modern community-oriented Arcaea web experience

### 8.2 What Arcaea-Viewer Is Not

- a research lab
- a generalized rhythm-game analysis framework
- an academic visualization system
- a wiki clone
- a sekai.best copy
- an enterprise-scale analytics platform by default
- a product whose identity depends on large-scale ML or data science systems

### 8.3 How Scope Should Be Managed

When a feature is proposed, it should be judged by one question first:

Does this make the Arcaea player experience better in a direct and visible way?

If the answer is no, the feature should be treated as optional, experimental, or out of scope.

## 9. Future Extensions

Advanced systems are welcome only when they serve the core experience.
They should be framed as future or experimental layers, not as the product identity.

Possible future extensions include:

- optional advanced analytics for players who want deeper chart insight
- experimental recommendation models with explainable outputs
- richer replay comparison tools
- community annotation and sharing systems
- deeper progression planning helpers
- future desktop or mobile companion surfaces if they add real value

These ideas are valid only if they remain subordinate to the player-first product.
