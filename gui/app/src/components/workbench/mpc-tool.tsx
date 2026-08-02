"use client";

// [OPUS-5] sq-ixc3.17 — the MPC tool: the site's `/showcase/mpc-100k` demo turned into an
// operational VERB over the live workspace store.
//
// Translation rule (research/gui-design.md §A.4/§A.5): the site PAGE runs the additive-sharing
// illustration over four hard-coded salaries, wrapped in a hero, a caption and a narrative
// disclosure panel. Here that chrome is CUT. The parties come from a SPARQL SELECT the operator
// runs against the ACTUAL imported store, so every share, local sum and disclosed bit derives
// from the operator's own data. The honesty register is carried OPERATIONALLY — the tier dot in
// the header plus one always-visible caveat strip — not as marketing prose.
//
// HONESTY (load-bearing, and the whole reason this tool is `live-sim` not `live`):
//   * This is an in-tab JS ILLUSTRATION of the protocol shape. There is no network, no peer, no
//     cryptographic guarantee, and it is NOT the native `sparq-mpc` crate.
//   * It uses additive n-out-of-n sharing; the native crate uses honest-majority Shamir t-of-n
//     sharing with a bit-decomposition secure comparison.
//   * The native crate is honest-majority semi-honest only, makes no production security claim,
//     and its collaborative zero-knowledge proof-of-correctness layer is a stub returning
//     `NotYetImplemented`. External accredited-cryptographer sign-off is pending (sq-qhy4).
//
// E2E hooks: [data-mpc-run], [data-mpc-query], [data-mpc-threshold], [data-mpc-verdict],
// [data-mpc-total-redacted], [data-mpc-error], [data-mpc-tier].

import * as React from "react";
import { Users, Play, Loader2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import { useEngine } from "@/lib/engine-context";
import { TIER_META, toolById } from "@/data/tools";
import {
  DEFAULT_MPC_QUERY,
  describeInputProblem,
  partiesFromResults,
  runSecureThreshold,
  type MpcResult,
  type SkippedRow,
} from "@/lib/mpc-sim";

// [FABLE-5] sq-qgkwy.2 — the override lives in the sibling `.meta.ts` (eagerly bundled for the
// rail/tab honesty read path) so THIS panel module can stay behind a lazy dynamic import().
// Re-exported here so the panel file remains the tool's single import surface.
export { MPC_TOOL_OVERRIDE } from "@/components/workbench/mpc-tool.meta";

/** The always-visible caveat. Kept verbatim in one place so it cannot drift per render site. */
const SIM_CAVEAT =
  "In-tab JS illustration of the protocol shape — no network, no peer, no cryptographic " +
  "guarantee, and NOT the native sparq-mpc crate. It uses additive n-out-of-n sharing; the " +
  "native crate uses honest-majority Shamir t-of-n sharing with a bit-decomposition secure " +
  "comparison, is honest-majority semi-honest only, and its collaborative zero-knowledge " +
  "proof-of-correctness layer is a stub. External cryptographer sign-off is pending (sq-qhy4).";

type RunState =
  | { kind: "idle" }
  | { kind: "running" }
  | { kind: "result"; result: MpcResult; skipped: SkippedRow[]; queryMs: number }
  | { kind: "error"; message: string };

const DEFAULT_THRESHOLD = "100";

/** Field elements are wide; show them compactly but in full on hover. */
function share(n: number): string {
  return n.toLocaleString("en-GB");
}

function SkippedRows({ skipped }: { skipped: SkippedRow[] }) {
  if (skipped.length === 0) return null;
  return (
    <details className="rounded-md border px-2 py-1.5 text-[11px] text-muted-foreground">
      <summary className="cursor-pointer">
        {skipped.length} row(s) could not become a party
      </summary>
      <ul className="space-y-0.5 pt-1">
        {skipped.map((s) => (
          <li key={s.row}>
            row {s.row + 1}
            {s.party ? ` (${s.party})` : ""}: {s.reason}
          </li>
        ))}
      </ul>
    </details>
  );
}

/** Step 1 + 2 — the N×N sharing matrix; the diagonal is what each party KEPT. */
function ShareMatrix({ result }: { result: MpcResult }) {
  const { parties, matrix } = result;
  return (
    <div className="overflow-auto rounded-md border" data-mpc-matrix="">
      <table className="w-full text-left text-xs">
        <thead className="border-b bg-card">
          <tr>
            <th className="px-2 py-1 font-medium text-muted-foreground">shares of ↓ held by →</th>
            {parties.map((p) => (
              <th key={p.name} className="px-2 py-1 font-medium text-muted-foreground">
                {p.name}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {matrix.map((row, i) => (
            <tr key={parties[i].name} className="border-b last:border-b-0">
              <td className="px-2 py-1 font-medium">{parties[i].name}</td>
              {row.map((cell, j) => (
                <td
                  key={j}
                  className={cn(
                    "px-2 py-1 font-mono tabular-nums",
                    cell.kept ? "bg-muted font-semibold" : "text-muted-foreground",
                  )}
                  title={cell.kept ? "kept by the originating party" : "sent to this peer"}
                >
                  {share(cell.value)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

/** Step 3 — each party's local sum over the column it received (free, zero-round addition). */
function LocalSums({ result }: { result: MpcResult }) {
  return (
    <div className="overflow-auto rounded-md border" data-mpc-local-sums="">
      <table className="w-full text-left text-xs">
        <thead className="border-b bg-card">
          <tr>
            <th className="px-2 py-1 font-medium text-muted-foreground">party</th>
            <th className="px-2 py-1 font-medium text-muted-foreground">shares received</th>
            <th className="px-2 py-1 font-medium text-muted-foreground">local sum</th>
          </tr>
        </thead>
        <tbody>
          {result.parties.map((p, j) => (
            <tr key={p.name} className="border-b last:border-b-0">
              <td className="px-2 py-1 font-medium">{p.name}</td>
              <td className="px-2 py-1 font-mono text-[11px] text-muted-foreground">
                {result.received[j].map(share).join(" + ")}
              </td>
              <td className="px-2 py-1 font-mono tabular-nums">{share(result.localSums[j])}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export function MpcTool() {
  const { status, run } = useEngine();
  const [query, setQuery] = React.useState(DEFAULT_MPC_QUERY);
  const [threshold, setThreshold] = React.useState(DEFAULT_THRESHOLD);
  const [state, setState] = React.useState<RunState>({ kind: "idle" });

  const ready = status.kind === "ready";
  // toolById (not the registry's resolveTool) to avoid an import cycle; the override does not
  // change the tier, so the base read is the honest one.
  const tool = toolById("mpc");
  const tier = tool ? TIER_META[tool.tier] : null;

  const onRun = React.useCallback(() => {
    const parsedThreshold = Number(threshold);
    setState({ kind: "running" });
    void run(query).then(
      ({ outcome, latencyMs }) => {
        if (outcome.kind === "error") {
          setState({ kind: "error", message: outcome.message });
          return;
        }
        if (outcome.kind !== "select") {
          setState({
            kind: "error",
            message: `The parties come from a SELECT over the live store; this query produced a ${outcome.kind} result.`,
          });
          return;
        }
        const { parties, skipped } = partiesFromResults(outcome.results);
        const problem = describeInputProblem(parties, parsedThreshold);
        if (problem) {
          setState({ kind: "error", message: problem });
          return;
        }
        try {
          setState({
            kind: "result",
            result: runSecureThreshold(parties, parsedThreshold),
            skipped,
            queryMs: latencyMs,
          });
        } catch (err) {
          setState({
            kind: "error",
            message: err instanceof Error ? err.message : String(err),
          });
        }
      },
      (err) =>
        setState({ kind: "error", message: err instanceof Error ? err.message : String(err) }),
    );
  }, [query, threshold, run]);

  return (
    <div className="flex h-full flex-col overflow-auto">
      {/* Operational header: icon + verb + the honesty tier dot. */}
      <div className="flex items-center gap-2 border-b bg-card px-3 py-1.5">
        <Users className="size-4 text-primary" aria-hidden />
        <h2 className="text-sm font-semibold">MPC threshold</h2>
        {tier && (
          <span
            className="flex items-center gap-1.5 text-[11px] text-muted-foreground"
            data-mpc-tier=""
          >
            <span className={cn("size-2 rounded-full", tier.dot)} aria-hidden />
            {tier.label}
          </span>
        )}
        <Button
          size="sm"
          onClick={onRun}
          disabled={!ready || state.kind === "running"}
          data-mpc-run=""
          className="ml-auto"
        >
          {state.kind === "running" ? <Loader2 className="animate-spin" /> : <Play />}
          Share + disclose verdict
        </Button>
      </div>

      {/* The one always-visible caveat — the `live-sim` tier in operational form. */}
      <p
        data-mpc-caveat=""
        className="border-b bg-card px-3 py-2 text-[11px] text-[var(--warning)]"
      >
        {SIM_CAVEAT}
      </p>

      {/* Inputs: the query that binds the parties + the public threshold. */}
      <div className="grid gap-3 p-3 md:grid-cols-[2fr_1fr]">
        <div className="flex flex-col gap-1">
          <label
            htmlFor="mpc-query"
            className="text-xs font-medium text-muted-foreground"
          >
            Parties (SELECT ?party ?value over the live store)
          </label>
          <textarea
            id="mpc-query"
            data-mpc-query=""
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            rows={7}
            spellCheck={false}
            className="w-full resize-y rounded-md border bg-background p-2 font-mono text-xs outline-none focus:ring-2 focus:ring-ring"
          />
        </div>
        <div className="flex flex-col gap-1">
          <label
            htmlFor="mpc-threshold"
            className="text-xs font-medium text-muted-foreground"
          >
            Threshold (public)
          </label>
          <input
            id="mpc-threshold"
            data-mpc-threshold=""
            value={threshold}
            onChange={(e) => setThreshold(e.target.value)}
            inputMode="numeric"
            spellCheck={false}
            className="rounded-md border bg-background px-2 py-1.5 font-mono text-xs outline-none focus:ring-2 focus:ring-ring"
          />
          <p className="pt-1 text-[11px] text-muted-foreground">
            Each party&apos;s value is split into one random share per party. Only the diagonal
            stays home; every other share is what would cross the wire. Summing shares is free, so
            the parties reach a shared total without any of them seeing another&apos;s value — and
            the run discloses one bit: total ≥ threshold.
          </p>
        </div>
      </div>

      {/* Results. */}
      <div className="flex min-h-0 flex-1 flex-col gap-3 border-t p-3">
        {state.kind === "idle" && (
          <p className="text-sm text-muted-foreground">
            Run the query to bind the parties from the live store, then share and disclose.
          </p>
        )}
        {state.kind === "running" && <p className="text-sm text-muted-foreground">Running…</p>}
        {state.kind === "error" && (
          <pre
            data-mpc-error=""
            className="whitespace-pre-wrap font-mono text-xs text-destructive"
          >
            {state.message}
          </pre>
        )}
        {state.kind === "result" && (
          <>
            <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
              <Badge variant="outline">
                {state.result.parties.length} parties from the live store
              </Badge>
              <span>query {state.queryMs.toFixed(1)} ms</span>
            </div>

            <SkippedRows skipped={state.skipped} />

            <div>
              <p className="pb-1 text-xs font-medium text-muted-foreground">
                1 + 2 · secret-share and distribute
              </p>
              <ShareMatrix result={state.result} />
            </div>

            <div>
              <p className="pb-1 text-xs font-medium text-muted-foreground">
                3 · each party sums the shares it holds
              </p>
              <LocalSums result={state.result} />
            </div>

            <div className="rounded-md border p-3">
              <p className="pb-1 text-xs font-medium text-muted-foreground">
                4 + 5 · combine, then disclose one bit
              </p>
              <p className="text-xs text-muted-foreground">
                Combined total{" "}
                <span className="font-mono line-through" data-mpc-total-redacted="">
                  {share(state.result.totalRedacted)}
                </span>{" "}
                — struck through because the real protocol NEVER opens it. It is computed here
                only so this panel can show what is withheld.
              </p>
              <div className="flex items-center gap-2 pt-2">
                <span className="text-xs text-muted-foreground">
                  total ≥ {state.result.threshold}
                </span>
                <Badge
                  variant={state.result.verdict ? "success" : "muted"}
                  data-mpc-verdict={state.result.verdict ? "true" : "false"}
                >
                  {String(state.result.verdict)}
                </Badge>
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
