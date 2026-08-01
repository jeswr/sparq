"use client";

// [OPUS-5] sq-ixc3.17 — the MPC tool: the site's secure-threshold demo turned into an
// operational VERB over the live workspace store.
//
// Translation rule (research/gui-design.md §A.4/§A.5, the sq-ixc3.11 precedent): the site's
// /showcase/mpc-100k PAGE runs four hard-coded salary sliders through an additive-sharing
// illustration inside a "£100k loan application" narrative, wrapped in marketing chrome. Here
// that chrome is CUT. The tool takes the PARTIES AND THEIR PRIVATE CONTRIBUTIONS FROM A
// SPARQL SELECT over the ACTIVE workspace's live store, runs the same protocol shape over
// them, and reveals exactly one bit — `Σ contributions ≥ threshold`. The honesty register is
// carried operationally: the tier dot, a per-skipped-row reason, and one compact caveat strip.
//
// HONESTY: this is an in-tab JS ILLUSTRATION of the protocol shape — plain additive
// (n-out-of-n) sharing over a prime field — NOT the native `sparq-mpc` crate, NOT a network,
// NOT live MPC. See lib/mpc-sim.ts for the exact deltas from the crate (which itself is
// honest-majority semi-honest only). Nothing here is a cryptographic guarantee, and the panel
// says so rather than implying it.
//
// Stable hooks: [data-result-kind="mpc"] (the run pane), [data-result-kind="error"] (errors),
// [data-mpc-verdict] on the revealed bit.

import * as React from "react";
import {
  ArrowRight,
  CircleAlert,
  Eye,
  EyeOff,
  Inbox,
  Lock,
  Play,
  ShieldAlert,
  Unlock,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { useEngine } from "@/lib/engine-context";
import { TIER_META, toolById } from "@/data/tools";
import { WorkbenchSparqlEditor } from "@/components/workbench/sparql-editor";
import {
  DEFAULT_PARTIES_QUERY,
  DEFAULT_THRESHOLD,
  partiesFromResults,
  runSecureThreshold,
  type MpcResult,
  type PartyScan,
} from "@/lib/mpc-sim";

type RunState =
  | { kind: "idle" }
  | { kind: "running" }
  | { kind: "done"; scan: PartyScan; result: MpcResult; ms: number }
  | { kind: "error"; message: string };

const num = (n: number) => n.toLocaleString();

export function MpcTool() {
  const { status, run } = useEngine();
  const [query, setQuery] = React.useState(DEFAULT_PARTIES_QUERY);
  const [threshold, setThreshold] = React.useState(DEFAULT_THRESHOLD);
  const [state, setState] = React.useState<RunState>({ kind: "idle" });
  const [revealShares, setRevealShares] = React.useState(false);

  const ready = status.kind === "ready";
  const tool = toolById("mpc");
  const tier = tool ? TIER_META[tool.tier] : null;

  const onRun = React.useCallback(async () => {
    setState({ kind: "running" });
    setRevealShares(false);
    try {
      const { outcome, latencyMs } = await run(query);
      if (outcome.kind === "error") {
        setState({ kind: "error", message: outcome.message });
        return;
      }
      if (outcome.kind !== "select") {
        setState({
          kind: "error",
          message:
            "The contributions query must be a SELECT — each result row is one party's private input.",
        });
        return;
      }
      const scan = partiesFromResults(outcome.results);
      if (scan.parties.length < 2) {
        setState({
          kind: "error",
          message: `The protocol needs at least two parties; the query yielded ${scan.parties.length}. Point the query at a column of numeric contributions in your live store.`,
        });
        return;
      }
      setState({
        kind: "done",
        scan,
        result: runSecureThreshold(scan.parties, threshold),
        ms: latencyMs,
      });
    } catch (err) {
      setState({ kind: "error", message: err instanceof Error ? err.message : String(err) });
    }
  }, [query, run, threshold]);

  const onKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      e.preventDefault();
      void onRun();
    }
  };

  return (
    <div className="flex h-full flex-col">
      {/* ── The contributions query over the live store ── */}
      <div className="flex min-h-0 flex-[2] flex-col border-b">
        <div className="flex shrink-0 flex-wrap items-center gap-2 border-b bg-card px-3 py-1.5">
          <span className="text-xs font-medium text-muted-foreground">Contributions query</span>
          {tier ? (
            <span className="flex items-center gap-1.5 text-[11px] text-muted-foreground">
              <span className={cn("size-2 rounded-full", tier.dot)} aria-hidden />
              {tier.label}
            </span>
          ) : null}
          <label htmlFor="mpc-threshold" className="ml-auto text-[11px] text-muted-foreground">
            Public threshold
          </label>
          <input
            id="mpc-threshold"
            type="number"
            min={0}
            step={1}
            value={threshold}
            onChange={(e) => {
              setThreshold(Number(e.target.value));
              setState({ kind: "idle" }); // a changed threshold invalidates a stale verdict
            }}
            className="tabular h-7 w-28 rounded-md border bg-background px-2 text-xs"
          />
          <Button
            size="sm"
            onClick={() => void onRun()}
            disabled={!ready || state.kind === "running"}
          >
            <Play />
            Run secure threshold
          </Button>
          <span className="text-[11px] text-muted-foreground">⌘↵</span>
        </div>
        <WorkbenchSparqlEditor
          id="mpc-parties-query"
          value={query}
          onChange={setQuery}
          onKeyDown={onKeyDown}
          ariaLabel="MPC contributions query editor"
        />
      </div>

      {/* ── The run: shares → local sums → the one revealed bit ── */}
      <div className="flex min-h-0 flex-[3] flex-col" data-result-kind="mpc">
        <div className="flex items-center gap-2 border-b bg-card px-3 py-1 text-xs">
          <span className="text-muted-foreground">Secure threshold over the live store</span>
          {state.kind === "done" ? (
            <>
              <Button
                size="sm"
                variant="outline"
                className="ml-auto h-6 gap-1 px-2 text-[11px]"
                onClick={() => setRevealShares((r) => !r)}
              >
                {revealShares ? <EyeOff /> : <Eye />}
                {revealShares ? "Hide share values" : "Reveal share values"}
              </Button>
              <span
                className="tabular text-[11px] text-muted-foreground"
                title="Wall-clock latency of the contributions query (performance.now) — non-canonical"
              >
                {state.result.parties.length} parties · {state.ms.toFixed(1)} ms
              </span>
            </>
          ) : null}
        </div>

        <div className="min-h-0 flex-1 space-y-3 overflow-auto p-3">
          {state.kind === "idle" ? (
            <p className="text-sm text-muted-foreground">
              {ready
                ? "Run the query to take each party's private contribution from the live store and reveal only whether their sum meets the threshold."
                : "Waiting for the engine to warm…"}
            </p>
          ) : state.kind === "running" ? (
            <p className="text-sm text-muted-foreground">Querying the live store…</p>
          ) : state.kind === "error" ? (
            <pre
              className="overflow-auto whitespace-pre-wrap font-mono text-xs text-destructive"
              data-result-kind="error"
            >
              {state.message}
            </pre>
          ) : (
            <>
              <SkippedRows scan={state.scan} />
              <ShareMatrix result={state.result} reveal={revealShares} />
              <ReceivedViews result={state.result} reveal={revealShares} />
              <LocalSums result={state.result} />
              <Verdict result={state.result} />
            </>
          )}
          <HonestLimits />
        </div>
      </div>
    </div>
  );
}

/** Rows the store yielded that could NOT become parties — shown, never silently dropped,
 *  because a quietly-shortened party list changes the verdict without saying so. */
function SkippedRows({ scan }: { scan: PartyScan }) {
  if (scan.skipped.length === 0) return null;
  return (
    <div className="rounded-md border border-[var(--warning)]/30 bg-[var(--warning)]/5 p-2 text-[11px] text-muted-foreground">
      <p className="font-medium text-foreground">
        {scan.skipped.length} {scan.skipped.length === 1 ? "row" : "rows"} excluded from the run —
        the verdict below is over the remaining {scan.parties.length} parties only:
      </p>
      <ul className="mt-1 space-y-0.5">
        {scan.skipped.map((s, i) => (
          <li key={i}>
            <span className="font-mono">{s.label}</span> — {s.reason}
          </li>
        ))}
      </ul>
    </div>
  );
}

/** Step 1–2: each party splits its value into n shares; off-diagonal cells cross the wire. */
function ShareMatrix({ result, reveal }: { result: MpcResult; reveal: boolean }) {
  return (
    <section className="space-y-1.5">
      <h3 className="text-xs font-medium">1 · Secret-share and distribute</h3>
      <p className="text-[11px] text-muted-foreground">
        Each party splits its private contribution into {result.parties.length} random shares. Row
        = the party producing shares, column = the party receiving them. Only the off-diagonal
        cells are sent; any single share is a uniform random field element.
      </p>
      <div className="overflow-x-auto rounded-md ring-1 ring-foreground/10">
        <table className="w-full text-left text-[11px]">
          <caption className="sr-only">
            Share distribution matrix: row = the party producing shares, column = the party
            receiving them.
          </caption>
          <thead className="bg-muted/50 text-muted-foreground">
            <tr>
              <th scope="col" className="px-3 py-1.5 font-medium">
                from \ to
              </th>
              {result.parties.map((p) => (
                <th key={p.name} scope="col" className="px-3 py-1.5 font-medium">
                  {p.name}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {result.matrix.map((row, i) => (
              <tr key={i} className="border-t">
                <th scope="row" className="px-3 py-1.5 font-medium">
                  {result.parties[i].name}
                </th>
                {row.map((cell, j) => (
                  <td
                    key={j}
                    className={cn(
                      "tabular px-3 py-1.5",
                      cell.kept ? "text-[var(--success-on-tint)]" : "text-muted-foreground",
                    )}
                  >
                    <span className="block font-mono">
                      {reveal ? num(cell.value) : "••••••"}
                    </span>
                    <span className="text-[10px] uppercase tracking-wide">
                      {cell.kept ? "kept" : "→ sent"}
                    </span>
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}

/** Step 3: each party's whole received view — one column of the matrix. */
function ReceivedViews({ result, reveal }: { result: MpcResult; reveal: boolean }) {
  return (
    <section className="space-y-1.5">
      <h3 className="text-xs font-medium">2 · What each party actually holds</h3>
      <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-4">
        {result.received.map((column, j) => (
          <div key={j} className="space-y-1 rounded-md border bg-card p-2">
            <p className="flex items-center gap-1.5 text-[11px] font-medium">
              <Inbox className="size-3 text-primary" aria-hidden />
              {result.parties[j].name}&rsquo;s view
            </p>
            <ul className="space-y-0.5">
              {column.map((share, i) => (
                <li key={i} className="flex items-center justify-between gap-2 text-[11px]">
                  <span className="truncate text-muted-foreground">
                    {i === j ? "own share (kept)" : `from ${result.parties[i].name}`}
                  </span>
                  <span className="tabular shrink-0 font-mono">
                    {reveal ? num(share) : "••••"}
                  </span>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>
      <p className="text-[11px] text-muted-foreground">
        Every share but the closing one is sampled uniformly at random, so a party&rsquo;s entire
        received view is statistically independent of every other party&rsquo;s contribution. That
        independence is the confidentiality property this shape relies on under its
        honest-majority, semi-honest assumption — it is not a guarantee against a participant that
        deviates from the protocol.
      </p>
    </section>
  );
}

/** Step 4: the free, zero-round local addition over shares. */
function LocalSums({ result }: { result: MpcResult }) {
  return (
    <section className="space-y-1.5">
      <h3 className="text-xs font-medium">3 · Add over shares (free, zero-round)</h3>
      <div className="flex flex-wrap items-center gap-2">
        {result.localSums.map((s, j) => (
          <div key={j} className="rounded-md bg-muted px-2 py-1 text-center">
            <div className="text-[10px] uppercase tracking-wide text-muted-foreground">
              {result.parties[j].name}
            </div>
            <div className="tabular font-mono text-[11px]">{num(s)}</div>
          </div>
        ))}
        <ArrowRight className="size-3.5 text-muted-foreground" aria-hidden />
        <div className="rounded-md bg-primary/10 px-2 py-1 text-center text-primary">
          <div className="text-[10px] uppercase tracking-wide">secret-shared total</div>
          <div className="font-mono text-[11px]">never opened</div>
        </div>
      </div>
    </section>
  );
}

/** Step 5: the one revealed bit, next to an explicit list of what stays withheld. */
function Verdict({ result }: { result: MpcResult }) {
  return (
    <section className="space-y-2">
      <h3 className="text-xs font-medium">4 · Reveal only the verdict</h3>
      <div
        role="status"
        aria-live="polite"
        data-mpc-verdict={result.verdict ? "true" : "false"}
        className={cn(
          "flex items-center gap-3 rounded-md px-3 py-2 ring-1",
          result.verdict
            ? "bg-[color-mix(in_oklch,var(--success)_10%,transparent)] ring-[var(--success)]/30"
            : "bg-destructive/10 ring-destructive/30",
        )}
      >
        {result.verdict ? (
          <Unlock className="size-5 text-[var(--success)]" aria-hidden />
        ) : (
          <ShieldAlert className="size-5 text-destructive" aria-hidden />
        )}
        <div>
          <p className="text-sm font-semibold">
            Σ contributions {result.verdict ? "≥" : "<"} {num(result.threshold)}
          </p>
          <p className="text-[11px] text-muted-foreground">
            The verifier learns this one bit —{" "}
            <code className="font-mono">{result.verdict ? "true" : "false"}</code> — and nothing
            else.
          </p>
        </div>
      </div>
      <div className="rounded-md border bg-card p-2">
        <p className="flex items-center gap-1.5 text-[11px] font-medium">
          <Lock className="size-3 text-primary" aria-hidden /> Withheld — never an output of the
          computation
        </p>
        <ul className="mt-1 space-y-0.5 text-[11px] text-muted-foreground">
          {result.parties.map((p) => (
            <li key={p.name}>
              <span className="font-mono">{p.name}</span>&rsquo;s contribution{" "}
              <span className="font-mono line-through opacity-60">{num(p.value)}</span>
            </li>
          ))}
          <li>
            the exact total{" "}
            <span className="font-mono line-through opacity-60">{num(result.totalRedacted)}</span>
          </li>
        </ul>
        <p className="mt-1 text-[11px] text-muted-foreground">
          Struck-through values are shown ONLY because this whole run happens inside your one tab —
          the workbench already holds the store the contributions came from. In a real deployment
          each party holds its own value and none of these would be visible here.
        </p>
      </div>
    </section>
  );
}

/** The compact operational caveat strip — the honesty register this tool must carry. */
function HonestLimits() {
  return (
    <div className="rounded-md border border-[var(--warning)]/30 bg-[var(--warning)]/5 p-3 text-[11px] text-muted-foreground">
      <p className="mb-1 flex items-center gap-1.5 font-medium text-foreground">
        <CircleAlert className="size-3.5 text-[var(--warning)]" aria-hidden />
        Honest limits
      </p>
      <p>
        This is an <strong className="text-foreground">in-tab JS illustration</strong> of the
        protocol shape — plain additive (n-out-of-n) secret sharing over a prime field, with the
        contributions read from your live store. It is <strong>not</strong> the native{" "}
        <code className="font-mono">sparq-mpc</code> crate, <strong>not</strong> a network, and{" "}
        <strong>not</strong> live MPC: no message crosses a machine boundary and the randomness is{" "}
        <code className="font-mono">Math.random</code>, not a CSPRNG. The crate itself uses
        honest-majority Shamir <em>t</em>-of-<em>n</em> sharing with a bit-decomposition secure
        comparison, and is <strong className="text-foreground">semi-honest only</strong> — its
        collaborative proof-of-correctness layer is a stub, and it has not been externally audited
        (bead <code className="font-mono">sq-qhy4</code>). Treat the verdict here as a
        demonstration of the flow, not a cryptographic guarantee.
      </p>
    </div>
  );
}
