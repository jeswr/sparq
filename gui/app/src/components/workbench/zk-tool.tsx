"use client";

// [OPUS-5] sq-ixc3.17 — the ZK tool: the site's `/showcase/zk-car-hire` live-bbjs prover turned
// into an operational VERB over the live workspace store.
//
// Translation rule (research/gui-design.md §A.4/§A.5): the site PAGE proves a fixed "age ≥ 25"
// about a hard-coded holder, wrapped in a hero, a caption, a composition narrative and a
// disclosure panel. Here that chrome is CUT. The operand is an integer the operator's OWN SPARQL
// SELECT returned from the imported store, and the comparison operator and bound are the
// operator's to choose — both are public inputs of the shipped circuit
// (zk/compose/filter_int_d2/src/main.nr). Only the operand is hidden.
//
// WHAT ACTUALLY RUNS: a genuine UltraHonk proof, generated AND verified in-tab by `@aztec/bb.js`
// over the committed ACIR artifact. The circuit rebuilds the canonical `"<digits>"^^xsd:integer`
// token in-circuit, hashes it, asserts it matches the committed term anchor, then asserts the
// comparison verdict — so a claim that disagrees with the hidden value is unsatisfiable and no
// proof can be produced.
//
// WHAT IS *NOT* PROVED: no dataset commitment, query-result commitment, subject identifier or
// credential root is among the circuit's public inputs, so the proof does NOT attest that the
// operand came from the selected row, from this store, or from an issued credential — the SELECT
// only supplies a local witness and picks which published anchor to claim about. The panel's copy
// says so at the point of claim rather than in a footnote. See `zk-filter.ts`'s
// `buildCircuitInputs` for the exact statement.
//
// HONEST LIMIT, surfaced per row rather than hidden: the term anchor (`operand_enc`) comes from
// the native encoder and cannot be derived in-tab, so only values whose anchor shipped with this
// build are provable — and only when the row's term really is an `xsd:integer` literal, since the
// circuit's token rebuild binds the datatype too. Every other row stays in the list, labelled
// with the specific reason.
//
// HONESTY: the sparq ZK estate is research-grade — the v1 verifier is internally re-audited only
// and external accredited-cryptographer sign-off is pending (sq-qhy4). A proof produced here is
// NOT a production cryptographic guarantee. See SECURITY.md.
//
// E2E hooks: [data-zk-run], [data-zk-query], [data-zk-prove], [data-zk-candidate],
// [data-zk-verdict], [data-zk-verified], [data-zk-error], [data-zk-prover], [data-zk-tier].

import * as React from "react";
import { Lock, Play, Loader2, ShieldCheck } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import { useEngine } from "@/lib/engine-context";
import { TIER_META, toolById } from "@/data/tools";
import {
  CIRCUIT_MEMBER,
  DEFAULT_ZK_QUERY,
  ZK_OPS,
  candidatesFromResults,
  type OpCode,
  type ZkCandidate,
} from "@/lib/zk-filter";
import { prewarmProver, proveFilter, type ProofResult } from "@/lib/zk-prover";

// [FABLE-5] sq-qgkwy.2 — the override lives in the sibling `.meta.ts` (eagerly bundled for the
// rail/tab honesty read path) so THIS panel module can stay behind a lazy dynamic import().
// Re-exported here so the panel file remains the tool's single import surface.
export { ZK_TOOL_OVERRIDE } from "@/components/workbench/zk-tool.meta";

/** The always-visible caveat. One place so it cannot drift per render site. */
const ZK_CAVEAT =
  "In-tab UltraHonk proving is real 3rd-party WASM (@aztec/bb.js), but the sparq ZK estate is " +
  "research-grade: the v1 verifier is internally re-audited only and external accredited-" +
  "cryptographer sign-off is pending (sq-qhy4). Treat a proof produced here as a research " +
  "artifact, not a production guarantee.";

type QueryState =
  | { kind: "idle" }
  | { kind: "running" }
  | { kind: "rows"; candidates: ZkCandidate[]; queryMs: number }
  | { kind: "error"; message: string };

type ProofState =
  | { kind: "idle" }
  | { kind: "proving" }
  | { kind: "done"; result: ProofResult; candidate: ZkCandidate; op: OpCode; bound: number }
  | { kind: "error"; message: string };

type ProverState = "loading" | "ready" | "error";

const DEFAULT_BOUND = "25";
const DEFAULT_OP: OpCode = 3; // OP_GE

function CandidateRow({
  candidate,
  selected,
  onSelect,
}: {
  candidate: ZkCandidate;
  selected: boolean;
  onSelect: () => void;
}) {
  return (
    <label
      data-zk-candidate={candidate.provable ? "provable" : "unprovable"}
      className={cn(
        "flex cursor-pointer items-start gap-2 border-b px-2 py-1.5 text-xs last:border-b-0",
        selected && "bg-muted",
        !candidate.provable && "cursor-not-allowed opacity-70",
      )}
    >
      <input
        type="radio"
        name="zk-candidate"
        className="mt-0.5"
        checked={selected}
        disabled={!candidate.provable}
        onChange={onSelect}
        aria-label={`row ${candidate.row + 1}`}
      />
      <span className="min-w-0 flex-1">
        <span className="flex flex-wrap items-center gap-2">
          <span className="truncate font-mono text-[11px]" title={candidate.subject}>
            {candidate.subject || `(row ${candidate.row + 1})`}
          </span>
          <span className="font-mono tabular-nums">{candidate.literal || "—"}</span>
          <Badge variant={candidate.provable ? "success" : "muted"}>
            {candidate.provable ? "provable in-tab" : "not provable"}
          </Badge>
        </span>
        {!candidate.provable && (
          <span className="block pt-0.5 text-[11px] text-muted-foreground">
            {candidate.reason}
          </span>
        )}
      </span>
    </label>
  );
}

function ProofReport({
  result,
  candidate,
  op,
  bound,
}: {
  result: ProofResult;
  candidate: ZkCandidate;
  op: OpCode;
  bound: number;
}) {
  const symbol = ZK_OPS.find((o) => o.code === op)?.symbol ?? "?";
  return (
    <div className="space-y-2 rounded-md border p-3" data-zk-report="">
      <div className="flex flex-wrap items-center gap-2 text-xs">
        <span className="text-muted-foreground">
          claim: the value behind the committed anchor is {symbol} {bound}
        </span>
        <Badge
          variant={result.verdict ? "success" : "muted"}
          data-zk-verdict={result.verdict ? "true" : "false"}
        >
          {String(result.verdict)}
        </Badge>
        <Badge
          variant={result.verified ? "success" : "warning"}
          data-zk-verified={result.verified ? "true" : "false"}
        >
          <ShieldCheck aria-hidden />
          {result.verified ? "verified in-tab" : "verification rejected"}
        </Badge>
      </div>

      <p className="text-[11px] text-muted-foreground" data-zk-scope="">
        Scope: the statement is about the committed term anchor only. You picked it locally from{" "}
        <span className="font-mono">{candidate.subject || `row ${candidate.row + 1}`}</span>, but no
        commitment to that row, to this store or to an issuing credential is among the public
        inputs, so the proof does not attest where the value came from.
      </p>

      <dl className="grid grid-cols-2 gap-x-4 gap-y-1 text-[11px] text-muted-foreground sm:grid-cols-4">
        {(
          [
            ["circuit", CIRCUIT_MEMBER],
            ["proof bytes", String(result.bundle.proof.length)],
            ["prove", `${result.proveMs.toFixed(0)} ms`],
            ["verify", `${result.verifyMs.toFixed(0)} ms`],
          ] as const
        ).map(([k, v]) => (
          <div key={k}>
            <dt className="font-medium">{k}</dt>
            <dd className="font-mono tabular-nums">{v}</dd>
          </div>
        ))}
      </dl>
      <p className="text-[11px] text-muted-foreground">
        {result.threads === 1
          ? "Proved single-threaded — this host is not cross-origin isolated, so bb.js cannot use SharedArrayBuffer. Timings are this machine's, not a benchmark."
          : `Proved with ${result.threads} bb.js threads. Timings are this machine's, not a benchmark.`}
      </p>

      <div>
        <p className="pb-1 text-[11px] font-medium text-muted-foreground">
          Public inputs the verifier sees — the value is not one of them, but{" "}
          <span className="font-mono">operand_enc</span> comes from the small anchor table this
          build publishes, so a verifier holding that table can read off which committed value the
          proof was made about
        </p>
        <ul className="space-y-0.5" data-zk-public-inputs="">
          {result.bundle.publicInputs.map((pi, i) => (
            <li key={i} className="break-all font-mono text-[11px] text-muted-foreground">
              {pi}
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}

export function ZkTool() {
  const { status, run } = useEngine();
  const [query, setQuery] = React.useState(DEFAULT_ZK_QUERY);
  const [queryState, setQueryState] = React.useState<QueryState>({ kind: "idle" });
  const [selectedRow, setSelectedRow] = React.useState<number | null>(null);
  const [op, setOp] = React.useState<OpCode>(DEFAULT_OP);
  const [bound, setBound] = React.useState(DEFAULT_BOUND);
  const [proof, setProof] = React.useState<ProofState>({ kind: "idle" });
  const [prover, setProver] = React.useState<ProverState>("loading");
  const [proverError, setProverError] = React.useState<string | null>(null);

  const ready = status.kind === "ready";
  // toolById (not the registry's resolveTool) to avoid an import cycle; the override does not
  // change the tier, so the base read is the honest one.
  const tool = toolById("zk");
  const tier = tool ? TIER_META[tool.tier] : null;

  // Pay the prover cold start (bb.js + noir_js chunks, the ACIR fetch, the Barretenberg
  // instantiate) in the background so the first Prove click does not. Purely a UX measure.
  React.useEffect(() => {
    let live = true;
    prewarmProver().then(
      () => {
        if (live) setProver("ready");
      },
      (err: unknown) => {
        if (!live) return;
        setProver("error");
        setProverError(err instanceof Error ? err.message : String(err));
      },
    );
    return () => {
      live = false;
    };
  }, []);

  const candidates = queryState.kind === "rows" ? queryState.candidates : [];
  const selected = candidates.find((c) => c.row === selectedRow) ?? null;
  const provableCount = candidates.filter((c) => c.provable).length;

  const onRun = React.useCallback(() => {
    setQueryState({ kind: "running" });
    setSelectedRow(null);
    setProof({ kind: "idle" });
    void run(query).then(
      ({ outcome, latencyMs }) => {
        if (outcome.kind === "error") {
          setQueryState({ kind: "error", message: outcome.message });
          return;
        }
        if (outcome.kind !== "select") {
          setQueryState({
            kind: "error",
            message: `The operand comes from a SELECT over the live store; this query produced a ${outcome.kind} result.`,
          });
          return;
        }
        const { candidates: rows } = candidatesFromResults(outcome.results);
        setQueryState({ kind: "rows", candidates: rows, queryMs: latencyMs });
        setSelectedRow(rows.find((c) => c.provable)?.row ?? null);
      },
      (err) =>
        setQueryState({
          kind: "error",
          message: err instanceof Error ? err.message : String(err),
        }),
    );
  }, [query, run]);

  const onProve = React.useCallback(() => {
    if (!selected?.digits) return;
    const parsedBound = Number(bound);
    setProof({ kind: "proving" });
    void proveFilter({ digits: selected.digits, op, bound: parsedBound }).then(
      (result) => setProof({ kind: "done", result, candidate: selected, op, bound: parsedBound }),
      (err) =>
        setProof({ kind: "error", message: err instanceof Error ? err.message : String(err) }),
    );
  }, [selected, op, bound]);

  return (
    <div className="flex h-full flex-col overflow-auto">
      {/* Operational header: icon + verb + the honesty tier dot + the prover readiness pill. */}
      <div className="flex items-center gap-2 border-b bg-card px-3 py-1.5">
        <Lock className="size-4 text-primary" aria-hidden />
        <h2 className="text-sm font-semibold">ZK proofs</h2>
        {tier && (
          <span
            className="flex items-center gap-1.5 text-[11px] text-muted-foreground"
            data-zk-tier=""
          >
            <span className={cn("size-2 rounded-full", tier.dot)} aria-hidden />
            {tier.label}
          </span>
        )}
        <Badge
          variant={prover === "ready" ? "success" : prover === "error" ? "warning" : "muted"}
          data-zk-prover={prover}
        >
          {prover === "ready"
            ? "prover ready"
            : prover === "error"
              ? "prover unavailable"
              : "prover loading…"}
        </Badge>
        <Button
          size="sm"
          onClick={onRun}
          disabled={!ready || queryState.kind === "running"}
          data-zk-run=""
          className="ml-auto"
        >
          {queryState.kind === "running" ? <Loader2 className="animate-spin" /> : <Play />}
          Find operands
        </Button>
      </div>

      {/* The one always-visible caveat — the `live-bbjs` tier in operational form. */}
      <p data-zk-caveat="" className="border-b bg-card px-3 py-2 text-[11px] text-[var(--warning)]">
        {ZK_CAVEAT}
      </p>

      {prover === "error" && proverError && (
        <pre
          data-zk-prover-error=""
          className="whitespace-pre-wrap border-b px-3 py-2 font-mono text-[11px] text-destructive"
        >
          {proverError}
        </pre>
      )}

      {/* The query that binds the candidate operands. */}
      <div className="flex flex-col gap-1 p-3">
        <label htmlFor="zk-query" className="text-xs font-medium text-muted-foreground">
          Operands (SELECT ?subject ?value over the live store)
        </label>
        <textarea
          id="zk-query"
          data-zk-query=""
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          rows={5}
          spellCheck={false}
          className="w-full resize-y rounded-md border bg-background p-2 font-mono text-xs outline-none focus:ring-2 focus:ring-ring"
        />
      </div>

      {/* Candidates + the claim being made about the selected one. */}
      <div className="flex min-h-0 flex-1 flex-col gap-3 border-t p-3">
        {queryState.kind === "idle" && (
          <p className="text-sm text-muted-foreground">
            Run the query to list the integers in the live store this build can prove about.
          </p>
        )}
        {queryState.kind === "running" && <p className="text-sm text-muted-foreground">Running…</p>}
        {queryState.kind === "error" && (
          <pre
            data-zk-error=""
            className="whitespace-pre-wrap font-mono text-xs text-destructive"
          >
            {queryState.message}
          </pre>
        )}

        {queryState.kind === "rows" && (
          <>
            <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
              <Badge variant="outline">
                {provableCount} of {candidates.length} rows provable with this build
              </Badge>
              <span>query {queryState.queryMs.toFixed(1)} ms</span>
            </div>

            {candidates.length === 0 ? (
              <p className="text-sm text-muted-foreground">
                The query returned no rows, so there is nothing to prove about.
              </p>
            ) : (
              <div className="overflow-auto rounded-md border">
                {candidates.map((c) => (
                  <CandidateRow
                    key={c.row}
                    candidate={c}
                    selected={c.row === selectedRow}
                    onSelect={() => {
                      setSelectedRow(c.row);
                      setProof({ kind: "idle" });
                    }}
                  />
                ))}
              </div>
            )}

            {provableCount === 0 && candidates.length > 0 && (
              <p className="rounded-md border border-[var(--warning)]/30 bg-[var(--warning)]/5 p-2 text-[11px] text-muted-foreground">
                None of these rows can be proven in-tab. The shipped circuit member{" "}
                <span className="font-mono">{CIRCUIT_MEMBER}</span> witnesses a canonical
                xsd:integer of exactly two digits, and the term anchor that binds a value to its
                committed encoding is produced by the native encoder — so a value with no shipped
                anchor is listed and explained rather than proven.
              </p>
            )}

            {/* The claim: operator + bound are PUBLIC circuit inputs, so they are yours to pick. */}
            <div className="flex flex-wrap items-end gap-3">
              <div className="flex flex-col gap-1">
                <label htmlFor="zk-op" className="text-xs font-medium text-muted-foreground">
                  Comparison (public)
                </label>
                <select
                  id="zk-op"
                  data-zk-op=""
                  value={op}
                  onChange={(e) => {
                    setOp(Number(e.target.value) as OpCode);
                    setProof({ kind: "idle" });
                  }}
                  className="rounded-md border bg-background px-2 py-1.5 font-mono text-xs outline-none focus:ring-2 focus:ring-ring"
                >
                  {ZK_OPS.map((o) => (
                    <option key={o.code} value={o.code}>
                      {o.symbol} ({o.circuitName})
                    </option>
                  ))}
                </select>
              </div>
              <div className="flex flex-col gap-1">
                <label htmlFor="zk-bound" className="text-xs font-medium text-muted-foreground">
                  Bound (public)
                </label>
                <input
                  id="zk-bound"
                  data-zk-bound=""
                  value={bound}
                  onChange={(e) => {
                    setBound(e.target.value);
                    setProof({ kind: "idle" });
                  }}
                  inputMode="numeric"
                  spellCheck={false}
                  className="w-28 rounded-md border bg-background px-2 py-1.5 font-mono text-xs outline-none focus:ring-2 focus:ring-ring"
                />
              </div>
              <Button
                size="sm"
                onClick={onProve}
                disabled={!selected?.provable || prover !== "ready" || proof.kind === "proving"}
                data-zk-prove=""
              >
                {proof.kind === "proving" ? <Loader2 className="animate-spin" /> : <Lock />}
                Prove + verify in-tab
              </Button>
            </div>

            <p className="text-[11px] text-muted-foreground">
              The proof publishes a fixed domain-separation constant, the committed term anchor,
              the operator, the bound and the resulting verdict. The store value itself is a
              private witness. The circuit asserts the published verdict equals the one the hidden
              value satisfies, so a claim that disagrees with the data cannot be proven at all —
              the witness solve fails and this panel reports that failure verbatim. What it proves
              is knowledge of an <span className="font-mono">xsd:integer</span> matching the
              committed anchor and the comparison — not that the value came from this store: no
              dataset or credential commitment is among the public inputs, so the query supplies a
              local witness rather than attested provenance. The published constant is not a
              per-presentation verifier nonce, so this panel offers no freshness or replay
              protection either: a bundle stays valid indefinitely and can be replayed verbatim.
            </p>

            {proof.kind === "proving" && (
              <p className="text-sm text-muted-foreground">Proving in-tab…</p>
            )}
            {proof.kind === "error" && (
              <pre
                data-zk-proof-error=""
                className="whitespace-pre-wrap font-mono text-xs text-destructive"
              >
                {proof.message}
              </pre>
            )}
            {proof.kind === "done" && (
              <ProofReport
                result={proof.result}
                candidate={proof.candidate}
                op={proof.op}
                bound={proof.bound}
              />
            )}
          </>
        )}
      </div>
    </div>
  );
}
