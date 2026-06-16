"use client";

// [OPUS-4.8] sq-13rg / sq-4r4b — the ZK car-hire flagship demo. Real in-browser
// UltraHonk proving of the age-gate sub-proof (see src/lib/zk-prover.ts); honest
// about what is live, what is composed-but-not-yet-wired, and what is research-grade
// and NOT externally audited.

import * as React from "react";
import {
  ShieldCheck,
  Loader2,
  Play,
  Eye,
  EyeOff,
  FileLock2,
  ArrowRight,
  CircleCheck,
  CircleAlert,
  Cpu,
  KeyRound,
} from "lucide-react";
import { toast } from "sonner";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { cn } from "@/lib/utils";
import {
  proveAgeEligibility,
  prewarmProver,
  PROVABLE_AGES,
  AGE_THRESHOLD,
  type ProofResult,
} from "@/lib/zk-prover";
import {
  CREDENTIAL_TURTLE,
  ELIGIBILITY_QUERY,
  DISCLOSURE,
  CIRCUIT_FAMILY,
  PUBLIC_INPUT_LABELS,
} from "@/data/zk-car-hire";

type Phase =
  | { kind: "idle" }
  | { kind: "loading" } // fetching circuit + bb.js/noir wasm
  | { kind: "proving" }
  | { kind: "verifying" }
  | { kind: "done"; result: ProofResult }
  | { kind: "rejected"; message: string } // under-age → unsatisfiable
  | { kind: "error"; message: string };

// [OPUS-4.8] Prover warm-up lifecycle, surfaced as a subtle indicator — mirrors the
// SPARQL REPL's Engine pill. The dynamic-import + circuit fetch + Barretenberg WASM
// instantiate is kicked off on mount (prewarmProver) so the first "Generate ZK proof"
// pays no cold start. The proving path awaits the same shared promise as a safety net.
type ProverState = "cold" | "warming" | "ready" | "error";

export function ZkCarHire() {
  const [age, setAge] = React.useState(30);
  const [phase, setPhase] = React.useState<Phase>({ kind: "idle" });
  const [prover, setProver] = React.useState<ProverState>("cold");

  // Pre-warm the in-browser ZK prover in the background on mount, off the render path.
  // A failure resets the indicator; proveAgeEligibility still awaits the shared init,
  // so the button never silently no-ops or throws on a cold prover. Route-scoped (this
  // component only renders on the ZK pages) so the ~MB bb.js WASM is never fetched on
  // other routes — the prover assets stay off /try, /papers, /benchmarks, etc.
  React.useEffect(() => {
    let cancelled = false;
    setProver("warming");
    prewarmProver()
      .then(() => {
        if (!cancelled) setProver("ready");
      })
      .catch(() => {
        // Transient load failure (fetch/instantiate) — the cache self-resets, so the
        // next "Generate ZK proof" retries the cold start. Surface it on the pill only.
        if (!cancelled) setProver("error");
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const run = React.useCallback(async () => {
    try {
      setPhase({ kind: "loading" });
      // small yield so the "loading" status paints before the heavy wasm fetch.
      await new Promise((r) => setTimeout(r, 0));
      setPhase({ kind: "proving" });
      const result = await proveAgeEligibility(age);
      // The shared prover init resolved (proveAgeEligibility awaits it): reflect ready.
      setProver("ready");
      setPhase({ kind: "verifying" });
      // verifyProof already ran inside proveAgeEligibility; reflect it.
      setPhase({ kind: "done", result });
      if (!result.verified) {
        toast.error("Proof did not verify", {
          description: "The in-tab verifier rejected the generated proof.",
        });
      }
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      // The circuit is unsatisfiable for an under-age claim: witness solve fails.
      if (/verdict mismatch|execution failed|unsatisfiable|assert/i.test(message)) {
        setPhase({
          kind: "rejected",
          message:
            "The circuit is unsatisfiable for this age — you cannot produce a proof of eligibility you do not have. (This is the soundness of the age-gate, live.)",
        });
      } else {
        setPhase({ kind: "error", message });
        toast.error("Proving failed", { description: message });
      }
    }
  }, [age]);

  const busy =
    phase.kind === "loading" ||
    phase.kind === "proving" ||
    phase.kind === "verifying";

  const statusText =
    phase.kind === "loading"
      ? "Loading the Noir + Barretenberg prover (WASM)…"
      : phase.kind === "proving"
        ? "Proving age ≥ 25 in your tab (UltraHonk)…"
        : phase.kind === "verifying"
          ? "Verifying the proof…"
          : phase.kind === "done"
            ? phase.result.verified
              ? "Proof generated and verified — in your tab."
              : "Proof generated but did NOT verify."
            : phase.kind === "rejected"
              ? "No proof produced — claim is unsatisfiable."
              : phase.kind === "error"
                ? "Proving failed."
                : "Idle — choose an age and generate a proof.";

  const underAge = age < AGE_THRESHOLD;

  return (
    <div className="space-y-8">
      {/* ── The flow: private credentials → query → proof → verdict ── */}
      <FlowStrip />

      <div className="grid gap-6 lg:grid-cols-2">
        {/* Private inputs */}
        <Card>
          <CardHeader className="flex-row items-center justify-between gap-2 space-y-0">
            <CardTitle className="flex items-center gap-2 text-base">
              <FileLock2 className="size-4 text-primary" />
              Your credentials (private)
            </CardTitle>
            <Badge variant="muted">
              <EyeOff className="size-3" /> never leaves your device
            </Badge>
          </CardHeader>
          <CardContent className="space-y-3">
            <pre className="max-h-64 overflow-auto rounded-lg border bg-muted/40 p-3 font-mono text-[11.5px] leading-relaxed">
              {CREDENTIAL_TURTLE}
            </pre>
            <p className="text-xs text-muted-foreground">
              Two W3C Verifiable Credentials — a gov-ID and a DVLA driving
              licence. The car-hire desk never sees this Turtle; only commitments
              to it and the eligibility verdict.
            </p>
          </CardContent>
        </Card>

        {/* The query + the prove control */}
        <Card>
          <CardHeader className="flex-row items-center justify-between gap-2 space-y-0">
            <CardTitle className="flex items-center gap-2 text-base">
              <ShieldCheck className="size-4 text-primary" />
              The desk’s eligibility query (public)
            </CardTitle>
            <div className="flex flex-wrap items-center justify-end gap-2">
              <ProverIndicator prover={prover} />
              <Badge variant="success">
                <Cpu className="size-3" /> Live via bb.js
              </Badge>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            <pre className="overflow-auto rounded-lg border bg-muted/40 p-3 font-mono text-[11.5px] leading-relaxed">
              {ELIGIBILITY_QUERY}
            </pre>

            <div className="space-y-2">
              <label
                htmlFor="zk-age"
                className="flex items-center justify-between text-sm font-medium"
              >
                <span>Your real age (the private witness)</span>
                <span className="tabular text-muted-foreground">{age}</span>
              </label>
              <div className="flex flex-wrap gap-1.5" role="group" aria-label="Choose age">
                {PROVABLE_AGES.map((a) => (
                  <Button
                    key={a}
                    id={a === age ? "zk-age" : undefined}
                    variant={a === age ? "default" : "outline"}
                    size="sm"
                    aria-pressed={a === age}
                    onClick={() => {
                      setAge(a);
                      setPhase({ kind: "idle" });
                    }}
                  >
                    {a}
                  </Button>
                ))}
              </div>
              <p className="text-xs text-muted-foreground">
                {underAge ? (
                  <>
                    {age} &lt; {AGE_THRESHOLD} — the circuit will be{" "}
                    <strong>unsatisfiable</strong>: you cannot prove eligibility
                    you do not have.
                  </>
                ) : (
                  <>
                    {age} ≥ {AGE_THRESHOLD} — provable. The age value itself is the
                    private witness and is never disclosed.
                  </>
                )}
              </p>
            </div>

            <div className="flex items-center gap-3">
              <Button onClick={run} disabled={busy}>
                {busy ? (
                  <Loader2 className="size-4 animate-spin" />
                ) : (
                  <Play className="size-4" />
                )}
                Generate ZK proof
              </Button>
              <p
                aria-live="polite"
                role="status"
                className="text-xs text-muted-foreground"
              >
                {statusText}
              </p>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* ── Verdict ── */}
      <VerdictPanel phase={phase} age={age} />

      {/* ── Reveal vs hide ── */}
      <DisclosurePanel />

      {/* ── The composed circuit family ── */}
      <CircuitFamilyPanel />

      {/* ── Honesty ── */}
      <HonestyPanel />
    </div>
  );
}

// [OPUS-4.8] Subtle prover-readiness pill. Reuses the badge tokens; never blocks the
// UI and never alters any security claim — it only reports cold-start progress.
function ProverIndicator({ prover }: { prover: ProverState }) {
  if (prover === "ready") {
    return (
      <Badge variant="muted" aria-live="polite">
        <CircleCheck className="size-3 text-[var(--success)]" /> Prover ready
      </Badge>
    );
  }
  if (prover === "error") {
    return (
      <Badge variant="warning" aria-live="polite">
        <CircleAlert className="size-3" /> Prover load failed — retries on proof
      </Badge>
    );
  }
  return (
    <Badge variant="muted" aria-live="polite">
      <Loader2 className="size-3 animate-spin" /> Initializing ZK prover…
    </Badge>
  );
}

function FlowStrip() {
  const steps = [
    { icon: FileLock2, label: "RDF credentials", note: "private, on-device" },
    { icon: ShieldCheck, label: "SPARQL eligibility query", note: "the public relation" },
    { icon: KeyRound, label: "ZK proof (UltraHonk)", note: "generated in your tab" },
    { icon: CircleCheck, label: "Desk sees “Eligible”", note: "+ the proof, nothing else" },
  ];
  return (
    <ol className="grid gap-3 rounded-xl bg-muted/30 p-4 ring-1 ring-foreground/10 sm:grid-cols-7 sm:items-center">
      {steps.map((s, i) => (
        <React.Fragment key={s.label}>
          <li className="flex items-center gap-3 sm:col-span-1 sm:flex-col sm:gap-1.5 sm:text-center">
            <span className="flex size-9 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary">
              <s.icon className="size-4" />
            </span>
            <span className="min-w-0">
              <span className="block text-xs font-medium leading-tight">{s.label}</span>
              <span className="block text-[11px] text-muted-foreground">{s.note}</span>
            </span>
          </li>
          {i < steps.length - 1 && (
            <li aria-hidden className="hidden justify-center text-muted-foreground/50 sm:flex">
              <ArrowRight className="size-4" />
            </li>
          )}
        </React.Fragment>
      ))}
    </ol>
  );
}

function VerdictPanel({ phase, age }: { phase: Phase; age: number }) {
  if (phase.kind === "rejected") {
    return (
      <Card>
        <CardContent className="flex flex-col gap-2 pt-5 sm:flex-row sm:items-center">
          <span className="flex size-10 shrink-0 items-center justify-center rounded-lg bg-[color-mix(in_oklch,var(--warning)_18%,transparent)] text-[var(--warning)]">
            <CircleAlert className="size-5" />
          </span>
          <div>
            <p className="font-semibold">Not eligible — and no proof exists</p>
            <p className="text-sm text-muted-foreground">{phase.message}</p>
          </div>
        </CardContent>
      </Card>
    );
  }
  if (phase.kind === "error") {
    return (
      <pre className="overflow-x-auto rounded-xl border border-destructive/30 bg-destructive/5 p-4 text-xs text-destructive">
        {phase.message}
      </pre>
    );
  }
  if (phase.kind !== "done") return null;
  const r = phase.result;
  return (
    <Card>
      <CardHeader className="flex-row items-center justify-between gap-2 space-y-0">
        <CardTitle className="flex items-center gap-2 text-base">
          {r.verified ? (
            <CircleCheck className="size-5 text-[var(--success)]" />
          ) : (
            <CircleAlert className="size-5 text-destructive" />
          )}
          What the car-hire desk receives
        </CardTitle>
        <Badge variant={r.verified ? "success" : "muted"} className="tabular">
          {r.threads === 1 ? "single-thread" : `${r.threads} threads`}
        </Badge>
      </CardHeader>
      <CardContent className="space-y-4">
        <div
          className={cn(
            "rounded-lg p-4 text-center",
            r.eligible
              ? "bg-[color-mix(in_oklch,var(--success)_14%,transparent)] text-[var(--success)]"
              : "bg-muted text-muted-foreground",
          )}
        >
          <p className="text-xs uppercase tracking-wide opacity-80">
            Eligibility verdict (the only age-derived bit revealed)
          </p>
          <p className="text-2xl font-semibold">
            ELIGIBLE: {r.eligible ? "true" : "false"}
          </p>
        </div>

        <div className="grid gap-3 sm:grid-cols-2">
          <Stat label="Proof verified in your tab" value={r.verified ? "yes" : "no"} />
          <Stat label="Proof size (fields)" value={r.proofByteLength.toString()} />
          <Stat label="Prove time" value={`${r.proveMs.toFixed(0)} ms`} sub="non-canonical" />
          <Stat label="Verify time" value={`${r.verifyMs.toFixed(0)} ms`} sub="non-canonical" />
        </div>

        <div className="rounded-lg border bg-muted/30 p-3">
          <p className="mb-1.5 flex items-center gap-1.5 text-xs font-medium">
            <Eye className="size-3.5" /> Public inputs the desk verifies against
          </p>
          <ul className="space-y-0.5 text-xs text-muted-foreground">
            {PUBLIC_INPUT_LABELS.map((l) => (
              <li key={l} className="font-mono">
                · {l}
              </li>
            ))}
          </ul>
          <p className="mt-2 flex items-center gap-1.5 text-xs font-medium text-[var(--success)]">
            <EyeOff className="size-3.5" /> The age {age} is the private witness — it
            is <span className="font-mono">not</span> among the public inputs.
          </p>
        </div>
      </CardContent>
    </Card>
  );
}

function Stat({ label, value, sub }: { label: string; value: string; sub?: string }) {
  return (
    <div className="rounded-lg border bg-card p-3">
      <p className="text-xs text-muted-foreground">{label}</p>
      <p className="tabular text-lg font-semibold">
        {value}
        {sub && <span className="ml-1 text-[10px] font-normal text-muted-foreground">{sub}</span>}
      </p>
    </div>
  );
}

function DisclosurePanel() {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">What is revealed vs hidden</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="overflow-x-auto rounded-lg ring-1 ring-foreground/10">
          <table className="w-full text-left text-sm">
            <thead className="bg-muted/50">
              <tr>
                <th className="px-4 py-2.5 font-medium text-[var(--success)]">
                  <span className="inline-flex items-center gap-1.5">
                    <Eye className="size-3.5" /> The desk sees (public)
                  </span>
                </th>
                <th className="px-4 py-2.5 font-medium">
                  <span className="inline-flex items-center gap-1.5">
                    <EyeOff className="size-3.5" /> Stays private
                  </span>
                </th>
              </tr>
            </thead>
            <tbody>
              {DISCLOSURE.map((row) => (
                <tr key={row.reveals} className="border-t align-top">
                  <td className="px-4 py-2.5">{row.reveals}</td>
                  <td className="px-4 py-2.5 text-muted-foreground">{row.hides}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </CardContent>
    </Card>
  );
}

function CircuitFamilyPanel() {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">The composed circuit family</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <p className="measure text-sm text-muted-foreground">
          A full car-hire presentation composes one sub-proof per relation. This
          page proves the <strong>age-gate</strong> member live in your tab; the
          others are real sparq circuits (compiled + gate-counted today) that the
          native crate proves over <code className="font-mono">bb</code> and that
          compose into one <code className="font-mono">ProofManifest</code>. Wiring
          the remaining members into the browser is tracked as a follow-up.
        </p>
        <div className="overflow-x-auto rounded-lg ring-1 ring-foreground/10">
          <table className="w-full text-left text-sm">
            <thead className="bg-muted/50">
              <tr>
                <th className="px-4 py-2.5 font-medium">Circuit</th>
                <th className="px-4 py-2.5 font-medium">Proves</th>
                <th className="px-4 py-2.5 font-medium">Gates</th>
                <th className="px-4 py-2.5 font-medium">In-tab</th>
              </tr>
            </thead>
            <tbody>
              {CIRCUIT_FAMILY.map((c) => (
                <tr key={c.name} className="border-t align-top">
                  <td className="px-4 py-2.5 font-mono text-[12.5px]">{c.name}</td>
                  <td className="px-4 py-2.5 text-muted-foreground">{c.role}</td>
                  <td className="tabular px-4 py-2.5">{c.gates.toLocaleString()}</td>
                  <td className="px-4 py-2.5">
                    {c.live ? (
                      <Badge variant="success">live</Badge>
                    ) : (
                      <Badge variant="muted">composed</Badge>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <p className="text-xs text-muted-foreground">
          Gate counts are the measured <code className="font-mono">bb gates</code>{" "}
          snapshot (non-canonical, for scale). All members sit roughly 15× under the
          bb.js single-thread browser ceiling.
        </p>
      </CardContent>
    </Card>
  );
}

function HonestyPanel() {
  return (
    <Card className="ring-[var(--warning)]/30">
      <CardHeader className="flex-row items-center gap-2 space-y-0">
        <CircleAlert className="size-5 text-[var(--warning)]" />
        <CardTitle className="text-base">
          Honest limits — read before you trust this
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-3 text-sm text-muted-foreground">
        <p>
          The age-gate proof on this page is <strong>real</strong>: a genuine
          UltraHonk proof of the sparq{" "}
          <code className="font-mono">filter_int</code> circuit, generated and
          verified by <code className="font-mono">@aztec/bb.js</code> in your
          browser. The exact age is the private witness and never appears in the
          public inputs.
        </p>
        <p>
          <strong className="text-foreground">
            But the sparq ZK estate is research-grade and has NOT been externally
            audited.
          </strong>{" "}
          No external accredited cryptographer has reviewed any part of sparq’s
          bespoke cryptography. The verifier-soundness claim rests entirely on
          sparq’s own internal, single-model self-audits. An external-cryptographer
          audit is <strong>required</strong> before any ZK security, privacy or
          attestation claim may be relied upon in production (tracked as bead{" "}
          <code className="font-mono">sq-qhy4</code>, and as gap{" "}
          <code className="font-mono">CR-G1</code> in{" "}
          <code className="font-mono">compliance/cryptoreview/</code>).
        </p>
        <p>
          The published security posture (<code className="font-mono">SECURITY.md</code>)
          treats the v1 verifier as <strong>NOT sound</strong> for a relying party.
          An internal re-audit finds it &ldquo;sound as landed&rdquo; under a stated
          threat model — but that is remediation progress, <em>not</em> a production
          guarantee, and it is single-model (pending re-review). Known privacy
          deferrals remain: holder possession is not yet bound to the specific
          credential, and the status-list IRI/version are disclosed (linkability).
        </p>
        <p className="rounded-lg bg-muted/50 p-3 text-foreground">
          <strong>Bottom line:</strong> this page demonstrates the{" "}
          <em>flow and the mechanics</em> of a sparq ZK query-proof, with a real
          proof in your tab. It does <strong>not</strong> assert that the proof is
          cryptographically trustworthy for production use. Treat any
          &ldquo;verified&rdquo; result here as a research demonstration.
        </p>
        <p className="flex flex-wrap gap-3 pt-1">
          <a
            className="text-primary underline-offset-4 hover:underline"
            href="https://github.com/jeswr/sparq/blob/main/SECURITY.md"
            target="_blank"
            rel="noopener noreferrer"
          >
            SECURITY.md
          </a>
          <a
            className="text-primary underline-offset-4 hover:underline"
            href="https://github.com/jeswr/sparq/tree/main/compliance/cryptoreview"
            target="_blank"
            rel="noopener noreferrer"
          >
            Cryptographic review readiness
          </a>
          <a
            className="text-primary underline-offset-4 hover:underline"
            href="https://github.com/jeswr/sparq/tree/main/zk/compose"
            target="_blank"
            rel="noopener noreferrer"
          >
            The Noir circuits
          </a>
        </p>
      </CardContent>
    </Card>
  );
}
