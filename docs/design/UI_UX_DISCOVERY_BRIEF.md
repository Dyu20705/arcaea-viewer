# UI/UX Discovery Brief

## Design objective

Create a character-led editorial database that feels connected to Arcaea without copying the official site, Sekai Viewer, or a community wiki.

The interface must balance two equally important goals:

- high information density and fast retrieval;
- visual comfort, atmosphere, and high-quality imagery.

## Reference principle

References are used to study information architecture, navigation, content density, search behavior, accessibility, and visual rhythm. They are not templates to reproduce.

Research should include:

- the official Arcaea website;
- Arcaea Wiki;
- Sekai Viewer;
- high-quality music databases;
- modern game encyclopedias;
- editorial archives;
- accessible data-heavy web applications.

The design issue must document what is learned, what is rejected, and why.

## Theme concept

Use a dual-character theme:

### Light — Hikari-inspired

- background: `#F7F3F2`
- surface: `#EBE3E4`
- elevated surface: `#DBCED4`
- primary ink: `#273557`
- muted ink: `#7C728B`
- rose accent: `#B12551`
- soft rose: `#DDB5C1`

### Dark — Tairitsu-inspired

- background: `#131414`
- surface: `#1F2233`
- elevated surface: `#313346`
- border/muted: `#5D5C70`
- primary text: `#D6D1D2`
- blue accent: `#6E97B2`
- bright blue accent: `#93C8DA`

These are starting tokens derived from the provided visual reference, not final brand colors. Contrast testing decides the final values.

## Visual language

Explore:

- asymmetric polygonal cuts used sparingly for navigation and hero framing;
- partner or jacket artwork as contextual accents, not unreadable backgrounds;
- calm surfaces with clear hierarchy;
- subtle depth and motion that respects `prefers-reduced-motion`;
- generous focus states and readable body typography;
- dark and light themes with equal quality, not a color inversion.

Avoid:

- glassmorphism that harms contrast;
- excessive gradients and animation;
- character art behind dense text;
- tiny wiki-like typography;
- dependency-heavy component systems;
- visual imitation of copyrighted UI assets.

## Required design deliverables before page implementation

- product principles and user stories;
- sitemap and navigation model;
- content taxonomy;
- desktop-first wireframes;
- responsive behavior notes;
- component inventory;
- state matrix: loading, empty, partial, error, stale, offline;
- light/dark token proposal;
- typography and spacing scale;
- image treatment rules;
- accessibility criteria;
- performance implications;
- clickable or high-fidelity prototype for the home, explore, and song detail flows;
- design decision record with rejected alternatives.

## Core UX flows

1. Latest event or release → related songs, packs, partners, and story.
2. Search or filter → scan results → open song → navigate to pack or related partner.
3. Browse game topics → find concise explanation → follow related entities.
4. Change theme and accessibility preferences.
5. Identify source, game version, and last-reviewed status for important facts.

## Accessibility gate

Every implementation issue must include:

- semantic landmarks and heading order;
- keyboard path;
- visible focus;
- contrast verification;
- text resizing behavior;
- reduced-motion behavior;
- image alt/decorative decision;
- screen-reader labels for controls;
- desktop and narrow viewport evidence.

## Performance gate

Design choices must fit these initial budgets:

- no large UI framework unless an ADR proves it necessary;
- route-level code splitting;
- responsive image variants;
- no autoplay media;
- predictable layout dimensions to limit CLS;
- static content first;
- measured bundle and Core Web Vitals before release.

Final budgets are set in the performance issue after a baseline build.
