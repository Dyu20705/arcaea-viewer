import type { SceneSummary } from "../lib/wasm/contract";

export function SummaryGrid({ summary }: { summary: SceneSummary | null }) {
  const rows = summary
    ? [
        ["Primitive count", summary.primitiveCount],
        ["Visible taps", summary.visibleTaps],
        ["Visible holds", summary.visibleHolds],
        ["Visible arcs", summary.visibleArcs],
        ["Visible arc taps", summary.visibleArcTaps],
        ["Hidden notes", summary.hiddenNotes],
      ]
    : [
        ["Primitive count", "—"],
        ["Visible taps", "—"],
        ["Visible holds", "—"],
        ["Visible arcs", "—"],
        ["Visible arc taps", "—"],
        ["Hidden notes", "—"],
      ];

  return (
    <dl className="summary-grid" aria-label="Render summary">
      {rows.map(([label, value]) => (
        <div key={label}>
          <dt>{label}</dt>
          <dd>{value}</dd>
        </div>
      ))}
    </dl>
  );
}
