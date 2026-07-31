"use client";

// [GPT-5] sq-ixc3.17 — operational host for the site's live-sim threshold illustration.
import * as React from "react";
import { Eye, EyeOff, Users } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { useEngine } from "@/lib/engine-context";
import { simulateThreshold, workspaceIntegers, type MpcSimulation } from "@/lib/mpc-sim";

const FALLBACK_VALUES = [30_000, 28_000, 26_000, 24_000];

export function MpcTool() {
  const { snapshotStore, storeSize } = useEngine();
  const [threshold, setThreshold] = React.useState(100_000);
  const [values, setValues] = React.useState(FALLBACK_VALUES);
  const [result, setResult] = React.useState<MpcSimulation | null>(null);
  const [showShares, setShowShares] = React.useState(false);

  const workspaceValues = React.useMemo(
    () => workspaceIntegers(snapshotStore() ?? "").slice(0, 8),
    [snapshotStore, storeSize],
  );

  function loadWorkspaceValues() {
    if (workspaceValues.length >= 2) {
      setValues(workspaceValues);
      setResult(null);
    }
  }

  return (
    <section className="h-full overflow-auto p-5" aria-labelledby="mpc-tool-title">
      <div className="mx-auto max-w-4xl space-y-5">
        <header className="flex flex-wrap items-start justify-between gap-3">
          <div>
            <h1 id="mpc-tool-title" className="text-lg font-semibold">Multi-party threshold</h1>
            <p className="mt-1 text-sm text-muted-foreground">
              Secret-share integer inputs and reveal only whether their sum meets a threshold.
            </p>
          </div>
          <Badge variant="warning">Simulation · honest-majority semi-honest</Badge>
        </header>
        <p className="rounded-lg border bg-muted/30 p-3 text-xs text-muted-foreground">
          This is an in-tab additive-sharing illustration, not the native sparq-mpc protocol and not live networked MPC.
        </p>

        <div className="rounded-lg border bg-card p-4">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <h2 className="text-sm font-semibold">Private party inputs</h2>
            <Button variant="outline" size="sm" onClick={loadWorkspaceValues} disabled={workspaceValues.length < 2}>
              Use live workspace integers ({workspaceValues.length})
            </Button>
          </div>
          <div className="mt-3 grid gap-2 sm:grid-cols-2">
            {values.map((value, index) => (
              <label key={index} className="text-xs text-muted-foreground">
                Party {index + 1}
                <input
                  type="number"
                  min={0}
                  value={value}
                  onChange={(event) => {
                    setValues((current) => current.map((item, itemIndex) => itemIndex === index ? Number(event.target.value) : item));
                    setResult(null);
                  }}
                  className="mt-1 h-8 w-full rounded-md border bg-background px-2 text-sm text-foreground"
                />
              </label>
            ))}
          </div>
          <label className="mt-3 block text-xs text-muted-foreground">
            Public threshold
            <input type="number" min={0} value={threshold} onChange={(event) => { setThreshold(Number(event.target.value)); setResult(null); }} className="ml-2 h-8 rounded-md border bg-background px-2 text-sm text-foreground" />
          </label>
          <Button
            className="mt-4"
            onClick={() =>
              setResult(
                simulateThreshold(
                  values.map((value, index) => ({
                    name: `Party ${index + 1}`,
                    value,
                  })),
                  threshold,
                ),
              )
            }
          >
            <Users aria-hidden /> Run threshold simulation
          </Button>
        </div>

        {result && (
          <div className="space-y-3 rounded-lg border bg-card p-4">
            <div className="flex items-center justify-between gap-2">
              <p className="font-medium" role="status">Revealed verdict: {String(result.verdict)}</p>
              <Button variant="outline" size="sm" onClick={() => setShowShares((value) => !value)}>
                {showShares ? <EyeOff aria-hidden /> : <Eye aria-hidden />}{showShares ? "Hide illustration internals" : "Inspect illustration internals"}
              </Button>
            </div>
            {showShares && (
              <div className="overflow-x-auto">
                <table className="w-full text-right font-mono text-xs">
                  <caption className="sr-only">Simulated additive share matrix</caption>
                  <tbody>
                    {result.shares.map((row, index) => (
                      <tr key={index}>
                        {row.map((share, column) => (
                          <td key={column} className="border p-2">{share}</td>
                        ))}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        )}
      </div>
    </section>
  );
}
