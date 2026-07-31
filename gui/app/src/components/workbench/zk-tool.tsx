"use client";

// [GPT-5] sq-ixc3.17 — operational host for the site's live-bbjs age prover.
import * as React from "react";
import { AlertTriangle, CheckCircle2, LockKeyhole } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { useEngine } from "@/lib/engine-context";
import { workspaceIntegers } from "@/lib/mpc-sim";

export function ZkTool() {
  const { snapshotStore, storeSize } = useEngine();
  const [age, setAge] = React.useState(30);
  const [state, setState] = React.useState<
    | { kind: "idle" }
    | { kind: "running" }
    | {
        kind: "done";
        verified: boolean;
        eligible: boolean;
        bytes: number;
        proveMs: number;
        verifyMs: number;
      }
    | { kind: "error"; message: string }
  >({ kind: "idle" });

  const workspaceAges = React.useMemo(() => {
    const snapshot = snapshotStore() ?? "";
    return workspaceIntegers(snapshot).filter((value) => [24, 25, 30, 42].includes(value));
  }, [snapshotStore, storeSize]);

  async function prove() {
    setState({ kind: "running" });
    try {
      const { proveAge } = await import("@/lib/zk-prover");
      const result = await proveAge(age);
      setState({
        kind: "done",
        verified: result.verified,
        eligible: result.eligible,
        bytes: result.proofBytes,
        proveMs: result.proveMs,
        verifyMs: result.verifyMs,
      });
    } catch (error) {
      setState({
        kind: "error",
        message: error instanceof Error ? error.message : String(error),
      });
    }
  }

  return (
    <section className="h-full overflow-auto p-5" aria-labelledby="zk-tool-title">
      <div className="mx-auto max-w-3xl space-y-5">
        <header className="flex flex-wrap items-start justify-between gap-3">
          <div>
            <h1 id="zk-tool-title" className="text-lg font-semibold">
              ZK age predicate
            </h1>
            <p className="mt-1 text-sm text-muted-foreground">
              Generate and independently verify an UltraHonk proof for a committed integer literal.
            </p>
          </div>
          <Badge variant="warning">Research track · not externally audited</Badge>
        </header>

        <div className="rounded-lg border bg-card p-4">
          <label htmlFor="zk-age" className="text-sm font-medium">Private integer (age)</label>
          <div className="mt-2 flex flex-wrap items-center gap-2">
            <select
              id="zk-age"
              value={age}
              onChange={(event) => { setAge(Number(event.target.value)); setState({ kind: "idle" }); }}
              className="h-8 rounded-md border bg-background px-3 text-sm"
            >
              {[24, 25, 30, 42].map((value) => (
                <option key={value}>{value}</option>
              ))}
            </select>
            <Button onClick={prove} disabled={state.kind === "running"}>
              <LockKeyhole aria-hidden />
              {state.kind === "running" ? "Proving…" : "Generate + verify proof"}
            </Button>
          </div>
          <p className="mt-3 text-xs text-muted-foreground">
            Public predicate: value ≥ 25. The exact value is a private witness.{" "}
            {workspaceAges.length > 0
              ? `Compatible integer literals found in the live workspace: ${workspaceAges.join(", ")}.`
              : "The live workspace has no integer literal matching this circuit’s committed fixtures; choose a fixture above."}
          </p>
        </div>

        {state.kind === "done" && (
          <div className="rounded-lg border bg-card p-4" role="status">
            <div className="flex items-center gap-2 font-medium">
              <CheckCircle2 className="size-4 text-[var(--success)]" aria-hidden />
              Verification: {state.verified ? "accepted" : "rejected"}
            </div>
            <dl className="mt-3 grid gap-2 text-sm sm:grid-cols-2">
              <div><dt className="text-muted-foreground">Disclosed predicate</dt><dd>{String(state.eligible)}</dd></div>
              <div><dt className="text-muted-foreground">Opaque proof size</dt><dd>{state.bytes.toLocaleString()} bytes</dd></div>
              <div><dt className="text-muted-foreground">This run · prove</dt><dd>{state.proveMs.toFixed(1)} ms</dd></div>
              <div><dt className="text-muted-foreground">This run · verify</dt><dd>{state.verifyMs.toFixed(1)} ms</dd></div>
            </dl>
          </div>
        )}
        {state.kind === "error" && (
          <p className="flex items-start gap-2 rounded-lg bg-destructive/10 p-3 text-sm text-destructive" role="alert">
            <AlertTriangle className="mt-0.5 size-4 shrink-0" aria-hidden />{state.message}
          </p>
        )}
      </div>
    </section>
  );
}
