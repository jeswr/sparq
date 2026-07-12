// [OPUS-4.8] sq-vjn4 — cross-machine external-reference baselines for a suite. These are
// cited competitor numbers at their OWN scale/machine (e.g. QLever on a 2020 M1, EYE
// DeepTaxonomy on an M1) — shown SEPARATELY and CLEARLY LABELLED, never divided into a
// same-box speedup. A null value renders "n/a (not gathered)".
import { fmtNum } from "@/lib/fmt-num";
import type { ReferenceBaseline } from "@/data/benchmarks";

export function ReferencesNote({ references }: { references: ReferenceBaseline[] }) {
  if (references.length === 0) return null;
  return (
    <div className="space-y-2 rounded-lg bg-muted/40 p-3 text-sm">
      <p className="font-medium">External reference baselines (cross-machine — not a same-box comparison)</p>
      <ul className="space-y-1.5 text-muted-foreground">
        {references.map((r, i) => (
          <li key={i} className="leading-relaxed">
            <span className="text-foreground">
              {r.engine} {r.version}
            </span>{" "}
            — {r.metric}:{" "}
            <span className="font-mono">
              {r.value == null ? "n/a (not gathered)" : `${fmtNum(r.value)} ${r.unit}`}
            </span>{" "}
            <span className="text-xs">
              ({r.scale}, {r.env}; {r.caveat})
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
}
