// [OPUS-4.8] sq-gum8 — the per-paper provenance stamp. Numbers in a paper are injected at
// build time from the paper-bound evidence file; this footer states WHERE they came from so
// a reader can trace any figure. Honest by construction: it names the commit the benchmark
// snapshot was generated at and the evidence-tier breakdown, and flags that the canonical
// performance runner is not yet operational (so no wall-clock number backs any headline).
import { GitCommitHorizontal, ShieldCheck } from "lucide-react";

import { Badge } from "@/components/ui/badge";

export interface Provenance {
  /** short commit the benchmark snapshot was generated at (or "unknown"). */
  commit: string;
  /** ISO date the benchmark snapshot was generated. */
  generatedAt: string;
  /** number of canonical evidence records bound into the factory. */
  canonical: number;
  /** number of indicative (work-box) records (never a headline). */
  indicative: number;
}

export function PaperProvenance({ prov }: { prov: Provenance }) {
  const date = prov.generatedAt
    ? new Date(prov.generatedAt).toLocaleString()
    : "—";
  return (
    <footer className="mt-10 space-y-2 rounded-lg bg-muted/40 p-4 text-xs text-muted-foreground">
      <div className="flex flex-wrap items-center gap-2">
        <Badge variant="muted" className="gap-1">
          <ShieldCheck aria-hidden />
          honesty gate enforced
        </Badge>
        <Badge variant="muted" className="gap-1">
          <GitCommitHorizontal aria-hidden />
          snapshot {prov.commit}
        </Badge>
        <span>·</span>
        <span>data generated {date}</span>
      </div>
      <p>
        Numbers in this paper are injected at build time from the paper-bound evidence file
        and trace to named tests and conformance-ratchet constants in the sparq crates (each
        record names its <code>source</code>). Of{" "}
        {prov.canonical + prov.indicative} evidence records,{" "}
        <strong className="text-foreground">{prov.canonical}</strong> are{" "}
        <strong className="text-foreground">canonical</strong> (deterministic,
        machine-independent) and {prov.indicative} are indicative. Only canonical numbers may
        back a headline result — a build-time gate fails the build otherwise. The canonical
        wall-clock performance runner is not yet operational, so no latency figure backs any
        claim here.
      </p>
    </footer>
  );
}
