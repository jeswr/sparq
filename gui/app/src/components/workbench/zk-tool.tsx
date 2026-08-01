"use client";

// [OPUS-5] sq-ixc3.17 — the ZK proofs tool: the site's live-bbjs prover turned into an
// operational VERB over the live workspace store.
//
// Translation rule (research/gui-design.md §A.4/§A.5, the sq-ixc3.11 precedent): the site's
// /showcase/zk-car-hire PAGE proves a hard-coded car-hire age credential, wrapped in
// marketing chrome — a hero flow strip, a "what the desk receives" narrative, a disclosure
// table, a circuit-family table and a long honesty essay. Here that chrome is CUT. The tool
// runs a SPARQL SELECT over the ACTIVE workspace's LIVE store, lists the integer literals in
// the result as candidate PRIVATE WITNESSES, and proves a FILTER over the one the operator
// picks — in-tab, with the value never leaving the witness. The honesty register is carried
// OPERATIONALLY (the tier dot, the per-row decline reason, one compact caveat strip) rather
// than as prose.
//
// What actually runs: `@aztec/bb.js` UltraHonk over the compiled sparq circuit member
// `filter_int_d2` (see lib/zk-prover.ts). That is REAL proving in the tab, not a replay.
//
// The ceiling is shown, not hidden. Binding an arbitrary store value to the circuit needs
// the NATIVE blake3 term encoder, which is not in the in-tab wasm bundle, so only values
// with a shipped `operand_enc` commitment can be proven here; every other row is listed with
// the exact reason it was declined (lib/zk-witness.ts). And the whole sparq ZK estate is
// research-track: external accredited-cryptographer sign-off is pending (bead sq-qhy4), so
// nothing in this panel presents a proof as a settled cryptographic guarantee.
//
// Stable hooks (driven by gui/e2e-playwright/specs/zk-tool.spec.ts, the browser lane that
// exercises this panel's REAL prove path end to end): [data-result-kind="zk"] (the candidates
// + proof pane), [data-result-kind="error"] (the error <pre>), [data-zk-prover] (the
// cold-start pill), [data-zk-candidate]/[data-zk-provable] (a candidate row + whether it can
// be proven), [data-zk-proof-verified] on a finished proof, [data-zk-public-inputs] (the
// public-input list the verifier sees), [data-zk-refused] (the circuit's refusal of a false
// claim).

import * as React from "react";
import {
  CircleAlert,
  CircleCheck,
  CircleDot,
  Eye,
  EyeOff,
  Loader2,
  Lock,
  Play,
  ShieldQuestion,
} from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { useEngine } from "@/lib/engine-context";
import { TIER_META, toolById } from "@/data/tools";
import { WorkbenchSparqlEditor } from "@/components/workbench/sparql-editor";
import {
  COMMITTED_VALUES,
  FILTER_BOUND,
  prewarmProver,
  proveFilterGe,
  type ProofResult,
} from "@/lib/zk-prover";
import {
  DEFAULT_WITNESS_QUERY,
  isUnsatisfiable,
  scanWitnesses,
  type WitnessCandidate,
  type WitnessScan,
} from "@/lib/zk-witness";

/** The witness-query lifecycle over the live store. */
type ScanState =
  | { kind: "idle" }
  | { kind: "running" }
  | { kind: "scanned"; scan: WitnessScan; ms: number; rowCount: number }
  | { kind: "error"; message: string };

/** The in-tab proving lifecycle. `refused` is the circuit declining a false claim — the
 *  success case of the "try to forge it" affordance, kept distinct from a real failure. */
type ProveState =
  | { kind: "idle" }
  | { kind: "proving"; claim: boolean }
  | { kind: "done"; result: ProofResult; value: number }
  | { kind: "refused"; claim: boolean }
  | { kind: "error"; message: string };

/** The prover cold-start lifecycle, surfaced as a subtle pill (never blocks the UI). */
type ProverState = "warming" | "ready" | "error";

export function ZkTool() {
  const { status, run } = useEngine();
  const [query, setQuery] = React.useState(DEFAULT_WITNESS_QUERY);
  const [scanState, setScanState] = React.useState<ScanState>({ kind: "idle" });
  const [selectedRow, setSelectedRow] = React.useState<number | null>(null);
  const [proveState, setProveState] = React.useState<ProveState>({ kind: "idle" });
  const [prover, setProver] = React.useState<ProverState>("warming");

  const ready = status.kind === "ready";
  const tool = toolById("zk");
  const tier = tool ? TIER_META[tool.tier] : null;

  // Pre-warm the in-tab prover in the background on mount, off the render path. A failure
  // only marks the pill: proveFilterGe awaits the same shared init, so the button still
  // retries the cold start rather than silently no-opping.
  React.useEffect(() => {
    let cancelled = false;
    prewarmProver()
      .then(() => {
        if (!cancelled) setProver("ready");
      })
      .catch(() => {
        if (!cancelled) setProver("error");
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const onScan = React.useCallback(async () => {
    setScanState({ kind: "running" });
    setSelectedRow(null);
    setProveState({ kind: "idle" });
    try {
      const { outcome, latencyMs } = await run(query);
      if (outcome.kind === "error") {
        setScanState({ kind: "error", message: outcome.message });
        return;
      }
      if (outcome.kind !== "select") {
        setScanState({
          kind: "error",
          message:
            "The witness query must be a SELECT — the tool reads candidate witnesses out of its result rows.",
        });
        return;
      }
      const scan = scanWitnesses(outcome.results, COMMITTED_VALUES);
      setScanState({ kind: "scanned", scan, ms: latencyMs, rowCount: outcome.rowCount });
      // Pre-select the first row a proof can actually be produced for.
      setSelectedRow(scan.candidates.find((c) => c.provable)?.row ?? null);
    } catch (err) {
      setScanState({ kind: "error", message: err instanceof Error ? err.message : String(err) });
    }
  }, [query, run]);

  const candidates = scanState.kind === "scanned" ? scanState.scan.candidates : [];
  const selected = candidates.find((c) => c.row === selectedRow) ?? null;

  const onProve = React.useCallback(
    async (candidate: WitnessCandidate, claim: boolean) => {
      setProveState({ kind: "proving", claim });
      try {
        const result = await proveFilterGe(candidate.value, claim);
        setProver("ready");
        setProveState({ kind: "done", result, value: candidate.value });
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        setProveState(
          isUnsatisfiable(message) ? { kind: "refused", claim } : { kind: "error", message },
        );
      }
    },
    [],
  );

  const onKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      e.preventDefault();
      void onScan();
    }
  };

  return (
    <div className="flex h-full flex-col">
      {/* ── Witness query over the live store ── */}
      <div className="flex min-h-0 flex-[2] flex-col border-b">
        <div className="flex shrink-0 flex-wrap items-center gap-2 border-b bg-card px-3 py-1.5">
          <span className="text-xs font-medium text-muted-foreground">Witness query</span>
          {tier ? (
            <span className="flex items-center gap-1.5 text-[11px] text-muted-foreground">
              <span className={cn("size-2 rounded-full", tier.dot)} aria-hidden />
              {tier.label}
            </span>
          ) : null}
          <ProverPill prover={prover} />
          <Button
            size="sm"
            onClick={() => void onScan()}
            disabled={!ready || scanState.kind === "running"}
            className="ml-auto"
          >
            {scanState.kind === "running" ? <Loader2 className="animate-spin" /> : <Play />}
            Find witnesses
          </Button>
          <span className="text-[11px] text-muted-foreground">⌘↵</span>
        </div>
        <WorkbenchSparqlEditor
          id="zk-witness-query"
          value={query}
          onChange={setQuery}
          onKeyDown={onKeyDown}
          ariaLabel="ZK witness query editor"
        />
      </div>

      {/* ── Candidates + prove + honest limits ── */}
      <div className="flex min-h-0 flex-[3] flex-col" data-result-kind="zk">
        <div className="flex items-center gap-2 border-b bg-card px-3 py-1 text-xs">
          <span className="text-muted-foreground">Candidate private witnesses</span>
          {scanState.kind === "scanned" ? (
            <span
              className="ml-auto tabular text-[11px] text-muted-foreground"
              title="Wall-clock latency of this query (performance.now) — non-canonical"
            >
              {candidates.filter((c) => c.provable).length} of {scanState.rowCount} rows provable
              in-tab · {scanState.ms.toFixed(1)} ms
            </span>
          ) : null}
        </div>

        <div className="min-h-0 flex-1 space-y-3 overflow-auto p-3">
          {scanState.kind === "idle" ? (
            <p className="text-sm text-muted-foreground">
              {ready
                ? "Run the query to list the integer literals in your live store that can be used as a hidden witness."
                : "Waiting for the engine to warm…"}
            </p>
          ) : scanState.kind === "running" ? (
            <p className="text-sm text-muted-foreground">Querying the live store…</p>
          ) : scanState.kind === "error" ? (
            <pre
              className="overflow-auto whitespace-pre-wrap font-mono text-xs text-destructive"
              data-result-kind="error"
            >
              {scanState.message}
            </pre>
          ) : (
            <>
              <CandidateTable
                scan={scanState.scan}
                selectedRow={selectedRow}
                onSelect={(row) => {
                  setSelectedRow(row);
                  setProveState({ kind: "idle" });
                }}
              />
              <ProvePanel
                candidate={selected}
                state={proveState}
                onProve={(claim) => selected && void onProve(selected, claim)}
              />
            </>
          )}
          <HonestLimits />
        </div>
      </div>
    </div>
  );
}

/** The prover cold-start pill — reports load progress only; it alters no claim. */
function ProverPill({ prover }: { prover: ProverState }) {
  if (prover === "ready") {
    return (
      <Badge variant="muted" aria-live="polite" data-zk-prover="ready">
        <CircleCheck className="text-[var(--success)]" /> Prover ready
      </Badge>
    );
  }
  if (prover === "error") {
    return (
      <Badge variant="warning" aria-live="polite" data-zk-prover="error">
        <CircleAlert /> Prover unavailable — retries on prove
      </Badge>
    );
  }
  return (
    <Badge variant="muted" aria-live="polite" data-zk-prover="warming">
      <Loader2 className="animate-spin" /> Loading prover…
    </Badge>
  );
}

/**
 * The candidate rows. Every row from the query is accounted for: provable rows are
 * selectable, declined rows show WHY they cannot be proven in-tab, and rows whose witness
 * column was unbound or wrongly typed are counted in the footer. Nothing is silently
 * dropped, because a quietly-shortened list would read as "your store has nothing to prove".
 */
function CandidateTable({
  scan,
  selectedRow,
  onSelect,
}: {
  scan: WitnessScan;
  selectedRow: number | null;
  onSelect: (row: number) => void;
}) {
  if (!scan.valueVar) {
    return (
      <p className="rounded-md border bg-muted/30 p-3 text-sm text-muted-foreground">
        No <code className="font-mono">xsd:integer</code> column in the result, so there is no
        witness to hide. The circuit binds the canonical{" "}
        <code className="font-mono">&quot;…&quot;^^xsd:integer</code> token, so a value typed{" "}
        <code className="font-mono">xsd:int</code> or <code className="font-mono">xsd:decimal</code>{" "}
        cannot stand in for it.
      </p>
    );
  }
  return (
    <div className="overflow-hidden rounded-md ring-1 ring-foreground/10">
      <table className="w-full text-left text-xs">
        <thead className="bg-muted/50 text-muted-foreground">
          <tr>
            <th scope="col" className="px-3 py-1.5 font-medium">
              {scan.labelVar ? `?${scan.labelVar}` : "row"}
            </th>
            <th scope="col" className="px-3 py-1.5 font-medium">
              ?{scan.valueVar} (becomes the private witness)
            </th>
            <th scope="col" className="px-3 py-1.5 font-medium">
              In-tab proof
            </th>
          </tr>
        </thead>
        <tbody>
          {scan.candidates.map((c) => (
            <tr
              key={c.row}
              data-zk-candidate={c.value}
              data-zk-provable={c.provable ? "true" : "false"}
              className={cn(
                "border-t align-top",
                c.row === selectedRow && "bg-primary/5 ring-1 ring-inset ring-primary/30",
              )}
            >
              <td className="px-3 py-1.5">
                {c.provable ? (
                  <button
                    type="button"
                    aria-pressed={c.row === selectedRow}
                    onClick={() => onSelect(c.row)}
                    className="break-all text-left font-mono text-primary underline-offset-2 hover:underline"
                  >
                    {c.label}
                  </button>
                ) : (
                  <span className="break-all font-mono text-muted-foreground">{c.label}</span>
                )}
              </td>
              <td className="tabular px-3 py-1.5 font-mono">{c.lexical}</td>
              <td className="px-3 py-1.5">
                {c.provable ? (
                  <Badge variant="success">
                    <CircleCheck /> provable
                  </Badge>
                ) : (
                  <span className="flex flex-col gap-1">
                    <Badge variant="muted" className="w-fit">
                      <CircleDot /> declined
                    </Badge>
                    <span className="text-muted-foreground">{c.decline}</span>
                  </span>
                )}
              </td>
            </tr>
          ))}
          {scan.candidates.length === 0 ? (
            <tr className="border-t">
              <td colSpan={3} className="px-3 py-2 text-muted-foreground">
                No row bound an <code className="font-mono">xsd:integer</code> witness.
              </td>
            </tr>
          ) : null}
        </tbody>
      </table>
      <p className="border-t bg-muted/30 px-3 py-1.5 text-[11px] text-muted-foreground">
        The values are in the clear here because this is your own store in your own tab — what a
        proof hides is the value from the <em>verifier</em>, not from you.
        {scan.skipped > 0 ? (
          <>
            {" "}
            {scan.skipped} further {scan.skipped === 1 ? "row was" : "rows were"} skipped: the
            witness column was unbound or not an <code className="font-mono">xsd:integer</code>{" "}
            literal.
          </>
        ) : null}
      </p>
    </div>
  );
}

/**
 * The prove controls + verdict for the selected witness. Two buttons, because the honest
 * claim and the false one are BOTH informative: proving the true verdict yields a real
 * in-tab proof, and asking for the opposite makes the constraint system refuse — that
 * refusal is the property the tool demonstrates rather than asserts.
 */
function ProvePanel({
  candidate,
  state,
  onProve,
}: {
  candidate: WitnessCandidate | null;
  state: ProveState;
  onProve: (claim: boolean) => void;
}) {
  if (!candidate) {
    return (
      <p className="text-xs text-muted-foreground">
        Select a provable row above to prove a <code className="font-mono">FILTER</code> over its
        hidden value.
      </p>
    );
  }
  const trueVerdict = candidate.value >= FILTER_BOUND;
  const busy = state.kind === "proving";
  return (
    <div className="space-y-3 rounded-md border bg-card p-3">
      <div className="flex flex-wrap items-center gap-2">
        <Lock className="size-4 text-primary" aria-hidden />
        <span className="text-xs">
          Prove{" "}
          <code className="font-mono">
            ?{"{"}witness{"}"} &gt;= {FILTER_BOUND}
          </code>{" "}
          for <span className="font-mono">{candidate.label}</span> — the value stays the private
          witness.
        </span>
      </div>
      <div className="flex flex-wrap items-center gap-2">
        <Button size="sm" disabled={busy} onClick={() => onProve(trueVerdict)}>
          {state.kind === "proving" && state.claim === trueVerdict ? (
            <Loader2 className="animate-spin" />
          ) : (
            <Play />
          )}
          Prove verdict {trueVerdict ? "true" : "false"}
        </Button>
        <Button size="sm" variant="outline" disabled={busy} onClick={() => onProve(!trueVerdict)}>
          {state.kind === "proving" && state.claim !== trueVerdict ? (
            <Loader2 className="animate-spin" />
          ) : (
            <ShieldQuestion />
          )}
          Try to prove {trueVerdict ? "false" : "true"}
        </Button>
        <span aria-live="polite" role="status" className="text-[11px] text-muted-foreground">
          {busy
            ? "Proving in your tab (UltraHonk)…"
            : state.kind === "done"
              ? state.result.verified
                ? "Proof generated and verified — in your tab."
                : "Proof generated but did NOT verify."
              : state.kind === "refused"
                ? "No proof produced — the claim is unsatisfiable."
                : state.kind === "error"
                  ? "Proving failed."
                  : "Idle."}
        </span>
      </div>

      {state.kind === "refused" ? (
        <p
          className="rounded-md border border-[var(--warning)]/30 bg-[var(--warning)]/5 p-2 text-xs text-muted-foreground"
          data-zk-refused={state.claim ? "true" : "false"}
        >
          <CircleAlert className="mr-1 inline size-3.5 text-[var(--warning)]" aria-hidden />
          The witness solve failed, so <strong>no proof exists</strong> for that claim: the
          constraint system will not certify a verdict the hidden value does not satisfy. That
          refusal happened in your tab, on the real circuit.
        </p>
      ) : null}

      {state.kind === "error" ? (
        <pre
          className="overflow-auto whitespace-pre-wrap font-mono text-xs text-destructive"
          data-result-kind="error"
        >
          {state.message}
        </pre>
      ) : null}

      {state.kind === "done" ? (
        <ProofSummary result={state.result} value={state.value} />
      ) : null}
    </div>
  );
}

/** What the verifier receives — and, explicitly, what it does not. */
function ProofSummary({ result, value }: { result: ProofResult; value: number }) {
  return (
    <div
      className="space-y-2 rounded-md bg-muted/40 p-3"
      data-zk-proof-verified={result.verified ? "true" : "false"}
    >
      <div className="flex flex-wrap items-center gap-2">
        {result.verified ? (
          <CircleCheck className="size-4 text-[var(--success)]" aria-hidden />
        ) : (
          <CircleAlert className="size-4 text-destructive" aria-hidden />
        )}
        <span className="text-xs font-medium">
          Verdict {result.verdict ? "true" : "false"} · verified in your tab:{" "}
          {result.verified ? "yes" : "no"}
        </span>
        <Badge variant="muted" className="ml-auto tabular">
          {result.threads === 1 ? "single-thread" : `${result.threads} threads`}
        </Badge>
      </div>
      <dl className="grid grid-cols-[max-content_1fr] gap-x-3 gap-y-0.5 text-[11px] text-muted-foreground">
        <dt>proof size</dt>
        <dd className="tabular font-mono">
          {(result.proofByteLength / 1024).toFixed(1)} KB (
          {Math.round(result.proofByteLength / 32)} fields)
        </dd>
        <dt>prove · verify</dt>
        <dd className="tabular font-mono">
          {result.proveMs.toFixed(0)} ms · {result.verifyMs.toFixed(0)} ms (measured here,
          non-canonical)
        </dd>
      </dl>
      <div className="rounded-md border bg-background p-2">
        <p className="mb-1 flex items-center gap-1.5 text-[11px] font-medium">
          <Eye className="size-3.5" aria-hidden /> Public inputs the verifier sees
        </p>
        <ul
          className="space-y-0.5 font-mono text-[11px] text-muted-foreground"
          data-zk-public-inputs={result.publicInputs.length}
        >
          {result.publicInputs.map((p, i) => (
            <li key={i} className="break-all" data-zk-public-input={p}>
              · {p}
            </li>
          ))}
        </ul>
        <p className="mt-1.5 flex items-start gap-1.5 text-[11px] text-[var(--success)]">
          <EyeOff className="mt-0.5 size-3.5 shrink-0" aria-hidden />
          The store value {value} is the private witness — it is not among the public inputs.
        </p>
      </div>
    </div>
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
        The proving and verifying here are real: a genuine UltraHonk run over the compiled sparq
        circuit member <code className="font-mono">filter_int_d2</code>, in your tab, with the
        store value as the private witness. But the sparq ZK estate is{" "}
        <strong className="text-foreground">research-track and NOT externally audited</strong> —
        no external accredited cryptographer has reviewed any part of sparq&rsquo;s bespoke
        cryptography, and that review is required before any ZK security or attestation claim may
        be relied upon (bead <code className="font-mono">sq-qhy4</code>; gap{" "}
        <code className="font-mono">CR-G1</code> in{" "}
        <code className="font-mono">compliance/cryptoreview/</code>). Treat a “verified” result
        here as a research demonstration, not a production guarantee.
      </p>
      <p className="mt-1.5">
        Scope: exactly one relation is checked — the{" "}
        <code className="font-mono">FILTER</code> comparison against the bound. Nothing here
        verifies a signature, a scan, a join, revocation or a holder key, and only values with a
        shipped term commitment can be proven in-tab (the encoder is native-only).
      </p>
    </div>
  );
}
