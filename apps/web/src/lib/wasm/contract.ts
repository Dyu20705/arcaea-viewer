export const CONTRACT_VERSION = 1;

export type BoundaryError = {
  code: string;
  message: string;
};

export type Diagnostic = {
  code: string;
  severity: string;
  message: string;
  line: number;
  column: number;
  spanStart: number;
  spanEnd: number;
  note?: string | null;
  help?: string | null;
};

export type Envelope<T> = {
  contractVersion: number;
  ok: boolean;
  data: T | null;
  diagnostics: Diagnostic[];
  error: BoundaryError | null;
};

export type ChartSummary = {
  eventCount: number;
  timingGroupCount: number;
  timingEvents: number;
  taps: number;
  holds: number;
  arcs: number;
  arcTaps: number;
};

export type ParseData = {
  summary: ChartSummary;
};

export type SceneSummary = {
  lanes: number;
  visibleTaps: number;
  visibleHolds: number;
  visibleArcs: number;
  visibleArcTaps: number;
  hiddenNotes: number;
  primitiveCount: number;
};

export type SceneData = {
  fixtureName: string;
  playbackMs: number;
  visibleTimeStartMs: number;
  visibleTimeEndMs: number;
  viewport: {
    width: number;
    height: number;
  };
  summary: SceneSummary;
  primitives: Array<{
    index: number;
    kind: string;
    layer: number;
    layerName: string;
    visible: boolean;
    noteId?: number | null;
    parentArcId?: number | null;
    timingGroupId?: number | null;
    lane?: number | null;
    state?: string | null;
    pointCount?: number | null;
  }>;
};

export type RenderRequest = {
  playbackMs: number;
  fixtureName: string;
  pastWindowMs: number;
  futureWindowMs: number;
  viewportWidth: number;
  viewportHeight: number;
  arcSampleSteps: number;
  debugLabels: boolean;
};

export type RenderSvgData = {
  request: RenderRequest;
  scene: SceneData;
  svg: string;
};

export function parseEnvelope<T>(json: string): Envelope<T> {
  const parsed = JSON.parse(json) as Partial<Envelope<T>>;

  if (
    typeof parsed.contractVersion !== "number" ||
    typeof parsed.ok !== "boolean" ||
    !Array.isArray(parsed.diagnostics)
  ) {
    throw new Error("Invalid WASM envelope shape");
  }

  return {
    contractVersion: parsed.contractVersion,
    ok: parsed.ok,
    data: parsed.data ?? null,
    diagnostics: parsed.diagnostics,
    error: parsed.error ?? null,
  };
}

export function buildRenderRequest(playbackMs: number): RenderRequest {
  return {
    playbackMs,
    fixtureName: "checkpoint9_mixed.aff",
    pastWindowMs: 500,
    futureWindowMs: 2_000,
    viewportWidth: 1_280,
    viewportHeight: 720,
    arcSampleSteps: 16,
    debugLabels: true,
  };
}
