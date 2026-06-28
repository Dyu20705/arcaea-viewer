import type { Diagnostic } from "../../lib/wasm/contract";

export function DiagnosticsList({
  diagnostics,
}: {
  diagnostics: Diagnostic[];
}) {
  if (diagnostics.length === 0) {
    return (
      <section className="diagnostics-panel" aria-label="Diagnostics">
        <h2>Diagnostics</h2>
        <p>No diagnostics</p>
      </section>
    );
  }

  return (
    <section className="diagnostics-panel" aria-label="Diagnostics">
      <h2>Diagnostics</h2>
      <ul>
        {diagnostics.map((diagnostic) => (
          <li
            key={`${diagnostic.code}-${diagnostic.spanStart}-${diagnostic.spanEnd}`}
          >
            <strong>{diagnostic.code}</strong>
            <span>{diagnostic.message}</span>
            <small>
              {diagnostic.severity} · {diagnostic.line}:{diagnostic.column}
            </small>
          </li>
        ))}
      </ul>
    </section>
  );
}
