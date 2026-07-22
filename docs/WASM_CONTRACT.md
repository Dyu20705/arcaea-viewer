# WASM Contract

## Version

Current contract version: `1`.

All exported functions return a JSON envelope:

```json
{
  "contractVersion": 1,
  "ok": true,
  "data": {},
  "diagnostics": [],
  "error": null
}
```

Failures use either structured parser diagnostics or a boundary error:

```json
{
  "contractVersion": 1,
  "ok": false,
  "data": null,
  "diagnostics": [],
  "error": {
    "code": "MALFORMED_JSON",
    "message": "render request is not valid JSON"
  }
}
```

## Exports

- `parse_chart_json(source: string): string`
- `build_playback_snapshot_json(source: string, playbackMs: number): string`
- `build_render_scene_json(source: string, requestJson: string): string`
- `render_chart_svg(source: string, requestJson: string): string`

`source` is AFF text. TypeScript does not parse AFF.

## Render Request

```json
{
  "playbackMs": 2500,
  "fixtureName": "checkpoint9_mixed.aff",
  "pastWindowMs": 500,
  "futureWindowMs": 2000,
  "viewportWidth": 1280,
  "viewportHeight": 720,
  "arcSampleSteps": 16,
  "debugLabels": true
}
```

All chart and playback times are integer milliseconds. Projection windows are milliseconds. Viewport dimensions are pixels.

Defaults match the renderer defaults: `500ms` past window, `2000ms` future window, `1280x720` viewport, and `16` arc sample steps.

## Diagnostics

Parser diagnostics include:

- `code`
- `category`
- `severity`
- `message`
- `line`
- `column`
- `spanStart`
- `spanEnd`
- optional `note`
- optional `help`

`category` is a stable lowercase grouping for UI filtering and reporting:
`lexical`, `syntax`, `unsupported`, or `domain`.

Parser failures put diagnostics in `diagnostics` and leave `error` as `null`.

Boundary, timing, and renderer failures put a stable `error.code` and human-readable `error.message` in `error`.

## Determinism

For identical AFF source and identical request JSON, the native pipeline and WASM wrapper return equivalent scene summaries and primitive order. Renderer primitive order is represented by the `primitives[index]` order in the DTO.

The boundary rejects malformed JSON and invalid projection settings before calling renderer logic. Output is tested to avoid `NaN`, `Infinity`, and `-Infinity`.

## SVG Trust Boundary

SVG strings are produced by the Rust renderer from parsed chart data. The web app does not insert raw AFF as HTML. Before mounting SVG, the app parses it with `DOMParser`, requires an `<svg>` root, and rejects blocked elements such as `script` and event-handler or `javascript:` attributes.

## Compatibility

Contract changes that remove fields, rename fields, or change units must increment `contractVersion`. Additive fields may keep the same version if existing consumers remain valid.
