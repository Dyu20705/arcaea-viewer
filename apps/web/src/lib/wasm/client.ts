import type {
  Envelope,
  ParseData,
  RenderRequest,
  RenderSvgData,
} from "./contract";
import { parseEnvelope } from "./contract";

type WasmModule = {
  default: (input?: unknown) => Promise<unknown>;
  parse_chart_json: (source: string) => string;
  render_chart_svg: (source: string, requestJson: string) => string;
};

let wasmPromise: Promise<WasmModule> | null = null;

export async function loadWasm(): Promise<WasmModule> {
  if (!wasmPromise) {
    wasmPromise = import("@arcaea-viewer/wasm").then(async (module) => {
      await module.default();
      return module;
    });
  }
  return wasmPromise;
}

export async function parseChart(source: string): Promise<Envelope<ParseData>> {
  const wasm = await loadWasm();
  return parseEnvelope<ParseData>(wasm.parse_chart_json(source));
}

export async function renderChartSvg(
  source: string,
  request: RenderRequest,
): Promise<Envelope<RenderSvgData>> {
  const wasm = await loadWasm();
  return parseEnvelope<RenderSvgData>(
    wasm.render_chart_svg(source, JSON.stringify(request)),
  );
}
