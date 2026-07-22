/// <reference types="vite/client" />

declare module "@arcaea-viewer/wasm" {
  export default function init(input?: unknown): Promise<unknown>;
  export function parse_chart_json(source: string): string;
  export function build_playback_snapshot_json(
    source: string,
    playbackMs: number,
  ): string;
  export function build_render_scene_json(
    source: string,
    requestJson: string,
  ): string;
  export function render_chart_svg(source: string, requestJson: string): string;
}
