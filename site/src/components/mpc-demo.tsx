"use client";

// [OPUS-4.8] sq-3hrc — the interactive MPC £100k secure-threshold demo.
// Each of N parties enters a PRIVATE contribution; the page runs a faithful
// in-tab illustration of additive secret sharing + a secure threshold and
// reveals ONLY the boolean `total ≥ £100k`, never the individual values or the
// exact sum. See src/lib/mpc-sim.ts for the protocol shape + the honesty notes.

import * as React from "react";
import {
  Lock,
  Unlock,
  ArrowRight,
  RotateCcw,
  Eye,
  EyeOff,
  ShieldCheck,
  ShieldAlert,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { cn } from "@/lib/utils";
import { runSecureThreshold, type MpcResult, type Party } from "@/lib/mpc-sim";

const THRESHOLD = 100_000;
const DEFAULT_VALUES = [30_000, 28_000, 26_000, 24_000]; // crate's tested 108k → true
const NAMES = ["Alice", "Bob", "Carol", "Dave"];

const gbp = (n: number) =>
  new Intl.NumberFormat("en-GB", {
    style: "currency",
    currency: "GBP",
    maximumFractionDigits: 0,
  }).format(n);

export function MpcDemo() {
  const [values, setValues] = React.useState<number[]>(DEFAULT_VALUES);
  const [result, setResult] = React.useState<MpcResult | null>(null);
  const [revealMatrix, setRevealMatrix] = React.useState(false);

  const total = values.reduce((a, b) => a + b, 0);

  const run = React.useCallback(() => {
    const parties: Party[] = values.map((value, i) => ({
      name: NAMES[i] ?? `Party ${i + 1}`,
      value,
    }));
    setResult(runSecureThreshold(parties, THRESHOLD));
  }, [values]);

  const reset = React.useCallback(() => {
    setValues(DEFAULT_VALUES);
    setResult(null);
    setRevealMatrix(false);
  }, []);

  const setValue = (i: number, v: number) => {
    setValues((prev) => {
      const next = [...prev];
      next[i] = v;
      return next;
    });
    // changing an input invalidates a stale verdict
    setResult(null);
  };

  return (
    <div className="space-y-6">
      {/* Inputs: each party's PRIVATE contribution */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">
            1 · Each party&rsquo;s private contribution
          </CardTitle>
          <CardDescription>
            These values never leave their owner in the protocol. Adjust them and
            run the secure threshold — only the verdict bit is revealed.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {values.map((v, i) => (
            <div key={i} className="space-y-1.5">
              <div className="flex items-center justify-between">
                <label
                  htmlFor={`party-${i}`}
                  className="flex items-center gap-1.5 text-sm font-medium"
                >
                  <Lock className="size-3.5 text-primary" aria-hidden="true" />
                  {NAMES[i]}&rsquo;s income
                  <Badge variant="default" className="ml-1">
                    private
                  </Badge>
                </label>
                <span className="tabular text-sm font-semibold">{gbp(v)}</span>
              </div>
              <input
                id={`party-${i}`}
                type="range"
                min={0}
                max={60_000}
                step={1_000}
                value={v}
                onChange={(e) => setValue(i, Number(e.target.value))}
                aria-label={`${NAMES[i]}'s private income contribution in pounds`}
                aria-valuetext={gbp(v)}
                className="w-full accent-[var(--primary)]"
              />
            </div>
          ))}
          <div className="flex flex-wrap items-center gap-2 pt-1">
            <Button onClick={run}>
              <ShieldCheck className="size-4" aria-hidden="true" />
              Run secure threshold
            </Button>
            <Button variant="outline" onClick={reset}>
              <RotateCcw className="size-4" aria-hidden="true" />
              Reset
            </Button>
            <span className="text-xs text-muted-foreground">
              Public threshold: <strong className="tabular">{gbp(THRESHOLD)}</strong>
            </span>
          </div>
        </CardContent>
      </Card>

      {/* Result — only renders after running */}
      {result && <MpcResultPanels result={result} revealMatrix={revealMatrix} onToggleMatrix={() => setRevealMatrix((r) => !r)} actualTotal={total} />}
    </div>
  );
}

function MpcResultPanels({
  result,
  revealMatrix,
  onToggleMatrix,
  actualTotal,
}: {
  result: MpcResult;
  revealMatrix: boolean;
  onToggleMatrix: () => void;
  actualTotal: number;
}) {
  const n = result.parties.length;
  return (
    <div className="space-y-6">
      {/* 2 · the share matrix — what crosses the wire */}
      <Card>
        <CardHeader className="flex-row items-start justify-between gap-2 space-y-0">
          <div className="space-y-1.5">
            <CardTitle className="text-base">
              2 · Secret-share &amp; distribute
            </CardTitle>
            <CardDescription>
              Each party splits its value into {n} random shares. Off-diagonal
              cells are <strong>sent to a peer</strong> — this is all that crosses
              the wire. Any single share is a uniform random number that reveals
              nothing about the original value.
            </CardDescription>
          </div>
          <Button variant="outline" size="sm" onClick={onToggleMatrix}>
            {revealMatrix ? (
              <>
                <EyeOff className="size-3.5" aria-hidden="true" />
                Hide share values
              </>
            ) : (
              <>
                <Eye className="size-3.5" aria-hidden="true" />
                Reveal share values
              </>
            )}
          </Button>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto rounded-xl ring-1 ring-foreground/10">
            <table className="w-full text-left text-sm">
              <caption className="sr-only">
                Share distribution matrix: row = the party producing shares,
                column = the party receiving them.
              </caption>
              <thead className="bg-muted/50">
                <tr>
                  <th scope="col" className="px-3 py-2 font-medium">
                    From \ To
                  </th>
                  {result.parties.map((p) => (
                    <th key={p.name} scope="col" className="px-3 py-2 font-medium">
                      {p.name}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {result.matrix.map((row, i) => (
                  <tr key={i} className="border-t">
                    <th scope="row" className="px-3 py-2 font-medium">
                      {result.parties[i].name}
                    </th>
                    {row.map((cell, j) => (
                      <td
                        key={j}
                        className={cn(
                          "tabular px-3 py-2",
                          cell.kept
                            ? "bg-[color-mix(in_oklch,var(--success)_10%,transparent)] text-[var(--success)]"
                            : "text-muted-foreground",
                        )}
                      >
                        <span className="block font-mono text-xs">
                          {revealMatrix ? cell.value.toLocaleString() : "••••••"}
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
          <p className="mt-3 text-xs text-muted-foreground">
            Read a <strong>column</strong> to see what one party receives: a
            mixture of one share from every party. No column reveals any single
            input — addition over shares is what produces the answer.
          </p>
        </CardContent>
      </Card>

      {/* 3 · local sums → combine */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">
            3 · Add over shares (free, zero-round)
          </CardTitle>
          <CardDescription>
            Each party sums the column of shares it holds. Combining these local
            sums reconstructs the secret-shared total — without anyone ever
            holding another party&rsquo;s value in the clear.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap items-center gap-3">
            {result.localSums.map((s, j) => (
              <div
                key={j}
                className="rounded-lg bg-muted px-3 py-2 text-center"
              >
                <div className="text-[10px] uppercase tracking-wide text-muted-foreground">
                  {result.parties[j].name}&rsquo;s local sum
                </div>
                <div className="tabular font-mono text-xs">
                  {s.toLocaleString()}
                </div>
              </div>
            ))}
            <ArrowRight className="size-4 text-muted-foreground" aria-hidden="true" />
            <div className="rounded-lg bg-primary/10 px-3 py-2 text-center text-primary">
              <div className="text-[10px] uppercase tracking-wide">
                secret-shared total
              </div>
              <div className="font-mono text-xs">never opened</div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* 4 · reveal ONLY the verdict bit — the money shot */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">4 · Reveal only the verdict</CardTitle>
          <CardDescription>
            The secure comparison opens exactly one bit:{" "}
            <code className="font-mono">total ≥ {gbp(THRESHOLD)}</code>. The exact
            total is never an output of the computation.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div
            className={cn(
              "flex items-center gap-3 rounded-xl px-4 py-4 ring-1",
              result.verdict
                ? "bg-[color-mix(in_oklch,var(--success)_10%,transparent)] ring-[var(--success)]/30"
                : "bg-destructive/10 ring-destructive/30",
            )}
            role="status"
            aria-live="polite"
          >
            {result.verdict ? (
              <Unlock className="size-6 text-[var(--success)]" aria-hidden="true" />
            ) : (
              <ShieldAlert className="size-6 text-destructive" aria-hidden="true" />
            )}
            <div>
              <div
                className={cn(
                  "text-lg font-semibold",
                  result.verdict ? "text-[var(--success)]" : "text-destructive",
                )}
              >
                {result.verdict
                  ? `Combined income clears ${gbp(THRESHOLD)}`
                  : `Combined income is below ${gbp(THRESHOLD)}`}
              </div>
              <div className="text-sm text-muted-foreground">
                The bank learns this one bit —{" "}
                <code className="font-mono">
                  {result.verdict ? "true" : "false"}
                </code>{" "}
                — and nothing else.
              </div>
            </div>
          </div>

          {/* what's shared vs what stays private */}
          <div className="grid gap-3 md:grid-cols-2">
            <div className="space-y-2 rounded-xl bg-muted/50 p-4 ring-1 ring-foreground/10">
              <div className="flex items-center gap-1.5 text-sm font-semibold">
                <Unlock className="size-4 text-[var(--success)]" aria-hidden="true" />
                Revealed (one bit)
              </div>
              <ul className="space-y-1 text-sm text-muted-foreground">
                <li>
                  ✓ Whether the sum meets the £100k threshold:{" "}
                  <strong className={result.verdict ? "text-[var(--success)]" : "text-destructive"}>
                    {result.verdict ? "true" : "false"}
                  </strong>
                </li>
                <li>✓ The public threshold itself ({gbp(THRESHOLD)})</li>
              </ul>
            </div>
            <div className="space-y-2 rounded-xl bg-muted/50 p-4 ring-1 ring-foreground/10">
              <div className="flex items-center gap-1.5 text-sm font-semibold">
                <Lock className="size-4 text-primary" aria-hidden="true" />
                Stays private (never revealed)
              </div>
              <ul className="space-y-1 text-sm text-muted-foreground">
                {result.parties.map((p) => (
                  <li key={p.name}>
                    ✗ {p.name}&rsquo;s income{" "}
                    <span className="font-mono text-xs line-through opacity-60">
                      {gbp(p.value)}
                    </span>
                  </li>
                ))}
                <li>
                  ✗ The exact total{" "}
                  <span className="font-mono text-xs line-through opacity-60">
                    {gbp(actualTotal)}
                  </span>{" "}
                  (redacted — not an output)
                </li>
              </ul>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
