import { useEffect, useMemo, useRef, useState } from "react";
import checkpointFixture from "../../../../fixtures/checkpoint9_mixed.aff?raw";
import { SummaryGrid } from "../components/SummaryGrid";
import { DiagnosticsList } from "../features/chart-viewer/DiagnosticsList";
import { parseChart, renderChartSvg } from "../lib/wasm/client";
import type {
  BoundaryError,
  Diagnostic,
  ParseData,
  RenderSvgData,
  SceneSummary,
} from "../lib/wasm/contract";
import {
  buildRenderRequest,
  CONTRACT_VERSION,
  type Envelope,
} from "../lib/wasm/contract";
import { mountTrustedSvg } from "../lib/svg";

type WasmStatus = "loading" | "ready" | "error";
type RunStatus = "idle" | "running" | "success" | "failure";

type ViewerState = {
  parseEnvelope: Envelope<ParseData> | null;
  renderEnvelope: Envelope<RenderSvgData> | null;
  diagnostics: Diagnostic[];
  error: BoundaryError | null;
  svg: string | null;
};

const INITIAL_VIEWER: ViewerState = {
  parseEnvelope: null,
  renderEnvelope: null,
  diagnostics: [],
  error: null,
  svg: null,
};

export function App() {
  const [source, setSource] = useState(checkpointFixture);
  const [playbackMs, setPlaybackMs] = useState(2_500);
  const [wasmStatus, setWasmStatus] = useState<WasmStatus>("loading");
  const [runStatus, setRunStatus] = useState<RunStatus>("idle");
  const [viewer, setViewer] = useState<ViewerState>(INITIAL_VIEWER);
  const [svgError, setSvgError] = useState<string | null>(null);
  const svgContainerRef = useRef<HTMLDivElement | null>(null);

  const renderRequest = useMemo(
    () => buildRenderRequest(playbackMs),
    [playbackMs],
  );
  const summary = viewer.renderEnvelope?.data?.scene.summary ?? null;

  useEffect(() => {
    parseChart(checkpointFixture)
      .then(() => setWasmStatus("ready"))
      .catch(() => setWasmStatus("error"));
  }, []);

  useEffect(() => {
    if (!viewer.svg || !svgContainerRef.current) {
      return;
    }
    try {
      mountTrustedSvg(svgContainerRef.current, viewer.svg);
      setSvgError(null);
    } catch (error) {
      setSvgError(
        error instanceof Error ? error.message : "Unable to mount SVG",
      );
      svgContainerRef.current.replaceChildren();
    }
  }, [viewer.svg]);

  async function runRender() {
    setRunStatus("running");
    setSvgError(null);
    try {
      const parseEnvelope = await parseChart(source);
      if (!parseEnvelope.ok) {
        setViewer({
          parseEnvelope,
          renderEnvelope: null,
          diagnostics: parseEnvelope.diagnostics,
          error: parseEnvelope.error,
          svg: null,
        });
        setRunStatus("failure");
        return;
      }

      const renderEnvelope = await renderChartSvg(source, renderRequest);
      setViewer({
        parseEnvelope,
        renderEnvelope,
        diagnostics: renderEnvelope.diagnostics,
        error: renderEnvelope.error,
        svg: renderEnvelope.data?.svg ?? null,
      });
      setRunStatus(renderEnvelope.ok ? "success" : "failure");
    } catch (error) {
      setViewer({
        ...INITIAL_VIEWER,
        error: {
          code: "WEB_RUNTIME_ERROR",
          message:
            error instanceof Error
              ? error.message
              : "Unexpected web runtime error",
        },
      });
      setRunStatus("failure");
    }
  }

  return (
    <main className="app-shell">
      <section className="toolbar" aria-label="Chart input">
        <div>
          <h1>Arcaea Viewer</h1>
          <p>Checkpoint 10 · AFF → Rust → WASM → React → SVG</p>
        </div>
        <button
          type="button"
          onClick={() => void runRender()}
          disabled={runStatus === "running"}
        >
          {runStatus === "running" ? "Rendering" : "Parse & Render"}
        </button>
      </section>

      <section className="workspace">
        <aside className="input-panel">
          <label className="field">
            <span>Fixture</span>
            <input value="checkpoint9_mixed.aff" readOnly />
          </label>

          <label className="field">
            <span>Playback time</span>
            <input
              aria-label="Playback time"
              type="number"
              min="-1000"
              max="8000"
              step="100"
              value={playbackMs}
              onChange={(event) => setPlaybackMs(Number(event.target.value))}
            />
          </label>

          <input
            aria-label="Playback range"
            className="range"
            type="range"
            min="-1000"
            max="8000"
            step="100"
            value={playbackMs}
            onChange={(event) => setPlaybackMs(Number(event.target.value))}
          />

          <details>
            <summary>Projection</summary>
            <dl className="projection-grid">
              <dt>Past</dt>
              <dd>{renderRequest.pastWindowMs}ms</dd>
              <dt>Future</dt>
              <dd>{renderRequest.futureWindowMs}ms</dd>
              <dt>Viewport</dt>
              <dd>
                {renderRequest.viewportWidth}×{renderRequest.viewportHeight}
              </dd>
              <dt>Arc steps</dt>
              <dd>{renderRequest.arcSampleSteps}</dd>
            </dl>
          </details>

          <label className="field source-field">
            <span>Source AFF</span>
            <textarea
              value={source}
              onChange={(event) => setSource(event.target.value)}
            />
          </label>
        </aside>

        <section className="main-panel">
          <StatusPanel
            wasmStatus={wasmStatus}
            runStatus={runStatus}
            error={viewer.error}
            summary={summary}
            contractVersion={
              viewer.renderEnvelope?.contractVersion ??
              viewer.parseEnvelope?.contractVersion ??
              CONTRACT_VERSION
            }
          />

          <div className="viewer-panel" aria-label="SVG chart preview">
            {!viewer.svg && runStatus !== "running" ? (
              <div className="empty-state">No scene rendered</div>
            ) : null}
            {runStatus === "running" ? (
              <div className="empty-state">Rendering scene</div>
            ) : null}
            {svgError ? <div className="error-state">{svgError}</div> : null}
            <div
              data-testid="svg-viewer"
              ref={svgContainerRef}
              className="svg-stage"
            />
          </div>

          <DiagnosticsList diagnostics={viewer.diagnostics} />
        </section>
      </section>
    </main>
  );
}

export function StatusPanel({
  wasmStatus,
  runStatus,
  error,
  summary,
  contractVersion,
}: {
  wasmStatus: WasmStatus;
  runStatus: RunStatus;
  error: BoundaryError | null;
  summary: SceneSummary | null;
  contractVersion: number;
}) {
  return (
    <section className="status-panel" aria-label="Status">
      <div className="status-row">
        <StatusBadge label="WASM" value={wasmStatus} />
        <StatusBadge label="Render" value={runStatus} />
        <StatusBadge label="Contract" value={`v${contractVersion}`} />
      </div>
      {error ? (
        <div className="error-state" role="alert">
          <strong>{error.code}</strong>
          <span>{error.message}</span>
        </div>
      ) : null}
      <SummaryGrid summary={summary} />
    </section>
  );
}

function StatusBadge({ label, value }: { label: string; value: string }) {
  return (
    <div className="status-badge">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
