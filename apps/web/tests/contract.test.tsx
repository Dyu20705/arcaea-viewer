import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { StatusPanel } from "../src/app/App";
import { SummaryGrid } from "../src/components/SummaryGrid";
import { DiagnosticsList } from "../src/features/chart-viewer/DiagnosticsList";
import { buildRenderRequest, parseEnvelope, type Diagnostic } from "../src/lib/wasm/contract";
import { mountTrustedSvg } from "../src/lib/svg";

describe("WASM contract helpers", () => {
  it("parses a valid boundary envelope", () => {
    const envelope = parseEnvelope<{ value: number }>(
      JSON.stringify({
        contractVersion: 1,
        ok: true,
        data: { value: 7 },
        diagnostics: [],
        error: null,
      }),
    );

    expect(envelope.ok).toBe(true);
    expect(envelope.data?.value).toBe(7);
  });

  it("builds playback render requests with Rust-owned projection defaults", () => {
    expect(buildRenderRequest(2500)).toEqual({
      playbackMs: 2500,
      fixtureName: "checkpoint9_mixed.aff",
      pastWindowMs: 500,
      futureWindowMs: 2000,
      viewportWidth: 1280,
      viewportHeight: 720,
      arcSampleSteps: 16,
      debugLabels: true,
    });
  });
});

describe("viewer UI fragments", () => {
  const diagnostic: Diagnostic = {
    code: "DOMAIN_VALIDATION_ERROR",
    severity: "error",
    message: "invalid ground lane",
    line: 2,
    column: 8,
    spanStart: 20,
    spanEnd: 21,
    note: "lane 9 is outside the supported range",
    help: "expected a ground lane from 1 to 4",
  };

  it("renders diagnostics from the boundary", () => {
    render(<DiagnosticsList diagnostics={[diagnostic]} />);

    expect(screen.getByText("DOMAIN_VALIDATION_ERROR")).toBeInTheDocument();
    expect(screen.getByText("invalid ground lane")).toBeInTheDocument();
    expect(screen.getByText("error · 2:8")).toBeInTheDocument();
  });

  it("renders success summary values", () => {
    render(
      <SummaryGrid
        summary={{
          lanes: 4,
          visibleTaps: 1,
          visibleHolds: 2,
          visibleArcs: 1,
          visibleArcTaps: 2,
          hiddenNotes: 3,
          primitiveCount: 18,
        }}
      />,
    );

    expect(screen.getByText("Primitive count")).toBeInTheDocument();
    expect(screen.getByText("18")).toBeInTheDocument();
    expect(screen.getByText("Visible arc taps")).toBeInTheDocument();
  });

  it("renders error state without crashing", () => {
    render(
      <StatusPanel
        wasmStatus="ready"
        runStatus="failure"
        contractVersion={1}
        summary={null}
        error={{ code: "MALFORMED_JSON", message: "render request is not valid JSON" }}
      />,
    );

    expect(screen.getByRole("alert")).toHaveTextContent("MALFORMED_JSON");
  });
});

describe("SVG trust boundary", () => {
  it("mounts renderer SVG documents", () => {
    const container = document.createElement("div");

    mountTrustedSvg(container, '<svg xmlns="http://www.w3.org/2000/svg"><rect /></svg>');

    expect(container.querySelector("svg")).not.toBeNull();
  });

  it("rejects blocked SVG content", () => {
    const container = document.createElement("div");

    expect(() =>
      mountTrustedSvg(
        container,
        '<svg xmlns="http://www.w3.org/2000/svg"><script>alert(1)</script></svg>',
      ),
    ).toThrow(/blocked element/);
  });
});
