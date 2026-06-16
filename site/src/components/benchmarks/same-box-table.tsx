// [OPUS-4.8] sq-vjn4 — the same-box cross-engine comparison table (e.g. SP2Bench: sparq
// vs Oxigraph / QLever on ONE ephemeral box). Competitor cells render "n/a" where a
// timing was not captured — never fabricated. Provenance + the honest caveat are shown.
import { fmtNum, type SameBoxComparison } from "@/data/benchmarks";

export function SameBoxTable({ comparison }: { comparison: SameBoxComparison }) {
  const engines = comparison.engines;
  return (
    <div className="space-y-2">
      <div className="overflow-x-auto rounded-lg ring-1 ring-foreground/10">
        <table className="w-full border-collapse text-sm">
          <thead>
            <tr className="border-b bg-muted/40 text-left">
              <th className="px-3 py-2 font-medium">Query</th>
              <th className="px-3 py-2 text-right font-medium">Rows</th>
              {engines.map((e) => (
                <th key={e.id} className="px-3 py-2 text-right font-medium" title={e.env}>
                  {e.label}
                  {e.version ? (
                    <code className="ml-1 text-[11px] font-normal text-muted-foreground">
                      {e.version}
                    </code>
                  ) : null}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {comparison.rows.map((r) => (
              <tr key={r.query} className="border-b last:border-0 hover:bg-muted/30">
                <td className="px-3 py-2 font-mono">{r.query}</td>
                <td className="px-3 py-2 text-right font-mono tabular-nums text-muted-foreground">
                  {fmtNum(r.rows)}
                </td>
                {engines.map((e) => {
                  const v = r.values[e.id];
                  return (
                    <td
                      key={e.id}
                      className="px-3 py-2 text-right font-mono tabular-nums"
                    >
                      {v == null ? (
                        <span className="text-muted-foreground">n/a</span>
                      ) : (
                        `${fmtNum(v)} ${r.unit}`
                      )}
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <p className="text-xs text-muted-foreground">
        Same-box {comparison.suite} gather · {comparison.scale} · min-of-{comparison.iters} ·{" "}
        {comparison.env.host_class} · NON-CANONICAL (for cross-engine ordering, not an
        absolute reference). {comparison.env.note}
      </p>
    </div>
  );
}
