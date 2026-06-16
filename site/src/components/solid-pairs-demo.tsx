"use client";

// [OPUS-4.8] sq-4r4b — the interactive Solid (user, app)-pair demo.
//
// Pick an (agent, client) pair → run the SAME SPARQL query over the SAME Pod → see the
// access-controlled result set that pair gets. The headline insight: identical query,
// identical Pod, DIFFERENT rows — because WAC/ACP restricts the named graphs each
// requester may read. The restriction runs LIVE in your tab: the wasm engine evaluates
// the query rewritten with the pair's `FROM NAMED <authorized-graphs>` set (sparq-solid's
// `rewrite_for`). The access-control DECISION (which graphs) is the materialized
// sparq-solid output, precomputed at build time — see the honesty section on the page.

import * as React from "react";
import {
  Play,
  Loader2,
  User,
  AppWindow,
  FileLock2,
  FileText,
  Globe,
  Users,
  ShieldOff,
  KeyRound,
  ArrowRight,
} from "lucide-react";
import { toast } from "sonner";

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
import {
  loadSparq,
  formatTerm,
  type SparqlResults,
  type WasmStore,
} from "@/lib/sparq-wasm";
import { rewriteForGraphs } from "@/lib/solid-acl";
import {
  SESSIONS,
  SHARED_QUERY,
  POD_NQUADS,
  POD_DOCS,
  ACL_GRANTS,
  DOC_BY_IRI,
  type PodSession,
  type PodDoc,
} from "@/data/solid-pod";

interface RunResult {
  rows: SparqlResults["results"]["bindings"];
  ms: number;
  rewritten: string;
}

type RunState =
  | { kind: "idle" }
  | { kind: "loading" }
  | { kind: "running" }
  | { kind: "done"; result: RunResult }
  | { kind: "error"; message: string };

function shortGraph(iri: string): string {
  return iri.replace("https://alice.pod.example/", "…/");
}

export function SolidPairsDemo() {
  const [sessionId, setSessionId] = React.useState<string>(SESSIONS[0].id);
  const [compareId, setCompareId] = React.useState<string>(SESSIONS[3].id);
  const [state, setState] = React.useState<RunState>({ kind: "idle" });
  const [compareState, setCompareState] = React.useState<RunState>({ kind: "idle" });
  const storeRef = React.useRef<WasmStore | null>(null);

  const session = SESSIONS.find((s) => s.id === sessionId)!;
  const compare = SESSIONS.find((s) => s.id === compareId)!;

  const ensureStore = React.useCallback(async (): Promise<WasmStore> => {
    if (storeRef.current) return storeRef.current;
    const Store = await loadSparq();
    // loadDataset preserves the named graphs (one per document) — essential, because
    // the whole access-control model is graph-level.
    const store = Store.loadDataset(POD_NQUADS, "nquads");
    storeRef.current = store;
    return store;
  }, []);

  const runFor = React.useCallback(
    async (s: PodSession): Promise<RunResult> => {
      const store = await ensureStore();
      const rewritten = rewriteForGraphs(SHARED_QUERY, s.authorizedGraphs);
      const t0 = performance.now();
      const json = store.query(rewritten);
      const ms = performance.now() - t0;
      const parsed = JSON.parse(json) as SparqlResults;
      return { rows: parsed.results?.bindings ?? [], ms, rewritten };
    },
    [ensureStore],
  );

  const run = React.useCallback(async () => {
    try {
      setState({ kind: "loading" });
      setCompareState({ kind: "loading" });
      const [a, b] = await Promise.all([runFor(session), runFor(compare)]);
      setState({ kind: "done", result: a });
      setCompareState({ kind: "done", result: b });
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setState({ kind: "error", message });
      setCompareState({ kind: "idle" });
      toast.error("Query failed", { description: message });
    }
  }, [runFor, session, compare]);

  // Re-run automatically when a selector changes, if we've already run once.
  const hasRun = state.kind === "done" || state.kind === "error";
  React.useEffect(() => {
    if (hasRun) void run();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [sessionId, compareId]);

  const busy = state.kind === "loading" || state.kind === "running";

  return (
    <div className="space-y-6">
      {/* The shared query — identical for everyone. */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">1 · One query, one Pod</CardTitle>
          <CardDescription>
            This exact query runs for <em>every</em> (user, app) pair, against the
            same four-document Pod. Nothing about the query changes — only{" "}
            <strong>who is asking, from which app</strong>.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <pre className="overflow-x-auto rounded-lg border bg-muted/40 p-3 font-mono text-[12.5px] leading-relaxed">
            {SHARED_QUERY}
          </pre>
          <PodMap />
        </CardContent>
      </Card>

      {/* The (user, app) selectors. */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">
            2 · Pick two (user, app) pairs to compare
          </CardTitle>
          <CardDescription>
            The session key is exactly sparq-solid&rsquo;s{" "}
            <code className="font-mono">Session {`{ agent, client }`}</code>. Pick a
            pair on each side, run, and watch the result sets diverge.
          </CardDescription>
        </CardHeader>
        <CardContent className="grid gap-4 md:grid-cols-2">
          <SessionPicker
            legend="Pair A"
            value={sessionId}
            onChange={setSessionId}
            accent="primary"
          />
          <SessionPicker
            legend="Pair B"
            value={compareId}
            onChange={setCompareId}
            accent="muted"
          />
          <div className="md:col-span-2">
            <Button onClick={run} disabled={busy}>
              {busy ? (
                <Loader2 className="size-4 animate-spin" aria-hidden="true" />
              ) : (
                <Play className="size-4" aria-hidden="true" />
              )}
              {state.kind === "loading"
                ? "Loading engine…"
                : "Run the same query for both pairs"}
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Side-by-side results. */}
      <div className="grid gap-4 lg:grid-cols-2">
        <SessionResult
          session={session}
          state={state}
          accent="primary"
          otherRows={
            compareState.kind === "done" ? compareState.result.rows : undefined
          }
        />
        <SessionResult
          session={compare}
          state={compareState}
          accent="muted"
          otherRows={state.kind === "done" ? state.result.rows : undefined}
        />
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------

function SessionPicker({
  legend,
  value,
  onChange,
  accent,
}: {
  legend: string;
  value: string;
  onChange: (id: string) => void;
  accent: "primary" | "muted";
}) {
  return (
    <fieldset className="space-y-2 rounded-xl border p-3">
      <legend
        className={cn(
          "px-1 text-xs font-semibold uppercase tracking-wide",
          accent === "primary" ? "text-primary" : "text-muted-foreground",
        )}
      >
        {legend}
      </legend>
      {SESSIONS.map((s) => (
        <label
          key={s.id}
          className={cn(
            "flex cursor-pointer items-start gap-2 rounded-lg p-2 text-sm transition-colors",
            value === s.id ? "bg-muted ring-1 ring-foreground/15" : "hover:bg-muted/50",
          )}
        >
          <input
            type="radio"
            name={`session-${legend}`}
            value={s.id}
            checked={value === s.id}
            onChange={() => onChange(s.id)}
            className="mt-1 accent-[var(--primary)]"
          />
          <span className="space-y-0.5">
            <span className="flex flex-wrap items-center gap-1.5 font-medium">
              <User className="size-3.5 text-muted-foreground" aria-hidden="true" />
              {s.user}
              <ArrowRight className="size-3 text-muted-foreground" aria-hidden="true" />
              <AppWindow className="size-3.5 text-muted-foreground" aria-hidden="true" />
              {s.app}
            </span>
            <span className="block text-xs text-muted-foreground">{s.scenario}</span>
          </span>
        </label>
      ))}
    </fieldset>
  );
}

const SENS_ICON: Record<PodDoc["sensitivity"], React.ComponentType<{ className?: string }>> = {
  public: Globe,
  shared: Users,
  private: FileLock2,
};

function PodMap() {
  return (
    <div className="space-y-2">
      <p className="text-xs font-medium text-muted-foreground">
        The Pod — one named graph per document:
      </p>
      <div className="grid gap-2 sm:grid-cols-2">
        {POD_DOCS.map((d) => {
          const Icon = SENS_ICON[d.sensitivity];
          return (
            <div
              key={d.iri}
              className="flex items-start gap-2 rounded-lg border bg-muted/30 p-2.5 text-sm"
            >
              <Icon className="mt-0.5 size-4 text-muted-foreground" aria-hidden="true" />
              <div className="min-w-0">
                <div className="flex items-center gap-1.5">
                  <span className="font-medium">{d.label}</span>
                  <Badge
                    variant={
                      d.sensitivity === "public"
                        ? "success"
                        : d.sensitivity === "shared"
                          ? "warning"
                          : "muted"
                    }
                    className="h-4 px-1.5 text-[10px]"
                  >
                    {d.sensitivity}
                  </Badge>
                </div>
                <div className="truncate font-mono text-[11px] text-muted-foreground">
                  {shortGraph(d.iri)}
                </div>
                <div className="text-xs text-muted-foreground">{d.about}</div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------

function SessionResult({
  session,
  state,
  accent,
  otherRows,
}: {
  session: PodSession;
  state: RunState;
  accent: "primary" | "muted";
  otherRows?: SparqlResults["results"]["bindings"];
}) {
  const result = state.kind === "done" ? state.result : null;
  const visibleGraphs = session.authorizedGraphs;
  const empty = visibleGraphs.length === 0;

  // Which graphs the OTHER pair sees but this one does not (and vice versa) → the diff.
  const otherGraphs = otherRows ? graphSet(otherRows) : null;
  const thisGraphs = result ? graphSet(result.rows) : new Set<string>();

  return (
    <Card
      className={cn(
        accent === "primary" ? "ring-1 ring-primary/30" : "ring-1 ring-foreground/10",
      )}
    >
      <CardHeader className="space-y-1.5">
        <CardTitle className="flex flex-wrap items-center gap-1.5 text-base">
          <User className="size-4 text-muted-foreground" aria-hidden="true" />
          {session.user}
          <span className="text-muted-foreground">·</span>
          <AppWindow className="size-4 text-muted-foreground" aria-hidden="true" />
          {session.app}
        </CardTitle>
        <CardDescription className="font-mono text-[11px]">
          agent={session.userWebId ?? "(anonymous)"} · client={session.appId ?? "(any)"}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Authorized graphs (the FROM NAMED set) + the fail-closed property. */}
        <div className="space-y-1.5">
          <div className="flex items-center gap-1.5 text-xs font-semibold">
            <KeyRound className="size-3.5 text-primary" aria-hidden="true" />
            Authorized graphs (FROM NAMED)
          </div>
          {empty ? (
            <p className="flex items-center gap-1.5 rounded-lg bg-muted/50 p-2 text-xs text-muted-foreground">
              <ShieldOff className="size-3.5" aria-hidden="true" />
              None — fail-closed. The query is restricted to a guaranteed-absent
              graph, so it returns nothing.
            </p>
          ) : (
            <ul className="space-y-1">
              {visibleGraphs.map((g) => {
                const doc = DOC_BY_IRI[g];
                return (
                  <li key={g} className="flex items-center gap-1.5 text-xs">
                    <span className="inline-block size-1.5 rounded-full bg-[var(--success)]" />
                    <span className="font-mono">{shortGraph(g)}</span>
                    {doc && <span className="text-muted-foreground">— {doc.label}</span>}
                  </li>
                );
              })}
            </ul>
          )}
          {/* The hidden graphs — what this pair CANNOT see, and that it can't tell they exist. */}
          <HiddenGraphs visible={visibleGraphs} />
        </div>

        {/* The "why" — the grants that produced this set. */}
        <WhyGrants session={session} />

        {/* The actual result rows from the live engine. */}
        <ResultRows state={state} session={session} />

        {/* The diff line vs the other pair. */}
        {result && otherGraphs && (
          <DiffSummary
            thisCount={result.rows.length}
            otherCount={otherRows!.length}
            extra={[...thisGraphs].filter((g) => !otherGraphs.has(g))}
          />
        )}
      </CardContent>
    </Card>
  );
}

function graphSet(rows: SparqlResults["results"]["bindings"]): Set<string> {
  const s = new Set<string>();
  for (const r of rows) {
    const g = r["graph"]?.value;
    if (g) s.add(g);
  }
  return s;
}

function HiddenGraphs({ visible }: { visible: string[] }) {
  const hidden = POD_DOCS.filter((d) => !visible.includes(d.iri));
  if (hidden.length === 0) return null;
  return (
    <details className="text-xs text-muted-foreground">
      <summary className="cursor-pointer select-none">
        {hidden.length} graph{hidden.length === 1 ? "" : "s"} invisible to this pair
      </summary>
      <ul className="mt-1 space-y-0.5 pl-3">
        {hidden.map((d) => (
          <li key={d.iri} className="flex items-center gap-1.5">
            <ShieldOff className="size-3" aria-hidden="true" />
            <span className="line-through opacity-70">{d.label}</span>
            <span className="opacity-60">(indistinguishable from absent)</span>
          </li>
        ))}
      </ul>
    </details>
  );
}

function WhyGrants({ session }: { session: PodSession }) {
  const grants = ACL_GRANTS.filter((g) => session.authorizedGraphs.includes(g.docIri));
  return (
    <details className="rounded-lg border bg-muted/20 p-2 text-xs">
      <summary className="cursor-pointer select-none font-semibold">
        Why these graphs? {grants.length} WAC/ACP grant{grants.length === 1 ? "" : "s"} matched
      </summary>
      <div className="mt-2 space-y-2">
        {grants.length === 0 && (
          <p className="text-muted-foreground">
            No grant matched this (agent, client) pair on any document beyond the
            public profile — fail-closed default.
          </p>
        )}
        {grants.map((g) => {
          const doc = DOC_BY_IRI[g.docIri];
          return (
            <div key={g.docIri} className="space-y-1">
              <div className="flex flex-wrap items-center gap-1.5">
                <Badge variant={g.system === "ACP" ? "default" : "muted"} className="h-4 px-1.5 text-[10px]">
                  {g.system}
                </Badge>
                <span className="font-medium">{doc?.label}</span>
                <span className="text-muted-foreground">
                  → {g.subject}, {g.client}
                </span>
              </div>
              <pre className="overflow-x-auto rounded bg-background/60 p-2 font-mono text-[10.5px] leading-snug text-muted-foreground">
                {g.rule}
              </pre>
            </div>
          );
        })}
      </div>
    </details>
  );
}

function ResultRows({ state, session }: { state: RunState; session: PodSession }) {
  if (state.kind === "error") {
    return (
      <pre className="overflow-x-auto rounded-lg border border-destructive/30 bg-destructive/5 p-3 text-xs text-destructive">
        {state.message}
      </pre>
    );
  }
  if (state.kind === "idle") {
    return (
      <p className="rounded-lg border bg-muted/20 p-3 text-xs text-muted-foreground">
        Run the query to see what this pair gets back.
      </p>
    );
  }
  if (state.kind === "loading" || state.kind === "running") {
    return (
      <p className="flex items-center gap-2 rounded-lg border bg-muted/20 p-3 text-xs text-muted-foreground">
        <Loader2 className="size-3.5 animate-spin" aria-hidden="true" />
        Running on the wasm engine…
      </p>
    );
  }

  const rows = state.result.rows;
  return (
    <div className="space-y-1.5">
      <div
        className="flex items-center justify-between text-xs font-semibold"
        aria-live="polite"
      >
        <span className="flex items-center gap-1.5">
          <FileText className="size-3.5 text-primary" aria-hidden="true" />
          Result set
        </span>
        <span className="tabular text-muted-foreground">
          {rows.length} row{rows.length === 1 ? "" : "s"} · {state.result.ms.toFixed(1)} ms · live
        </span>
      </div>
      {rows.length === 0 ? (
        <p className="rounded-lg border bg-muted/30 p-3 text-xs text-muted-foreground">
          No solutions. {session.authorizedGraphs.length === 0
            ? "Fail-closed: this pair has no authorized graphs, so the Pod looks empty to it."
            : "This pair's authorized graphs held no matching triples."}
        </p>
      ) : (
        <div className="max-h-72 overflow-auto rounded-lg border">
          <table className="w-full text-left text-[12px]">
            <thead className="sticky top-0 bg-muted/80 backdrop-blur">
              <tr>
                <th className="px-2 py-1.5 font-medium">graph</th>
                <th className="px-2 py-1.5 font-medium">s</th>
                <th className="px-2 py-1.5 font-medium">p</th>
                <th className="px-2 py-1.5 font-medium">o</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((row, i) => (
                <tr key={i} className="border-t">
                  <td className="px-2 py-1 font-mono text-[10.5px] text-muted-foreground">
                    {shortGraph(row["graph"]?.value ?? "")}
                  </td>
                  <td className="px-2 py-1 font-mono text-[11px]">{compact(formatTerm(row["s"]))}</td>
                  <td className="px-2 py-1 font-mono text-[11px]">{compact(formatTerm(row["p"]))}</td>
                  <td className="px-2 py-1 font-mono text-[11px]">{formatTerm(row["o"])}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

function compact(s: string): string {
  return s
    .replace("https://alice.pod.example/", "…/")
    .replace("http://xmlns.com/foaf/0.1/", "foaf:")
    .replace("http://schema.org/", "schema:")
    .replace("http://www.w3.org/2001/XMLSchema#", "xsd:");
}

function DiffSummary({
  thisCount,
  otherCount,
  extra,
}: {
  thisCount: number;
  otherCount: number;
  extra: string[];
}) {
  const delta = thisCount - otherCount;
  return (
    <div className="rounded-lg bg-primary/5 p-2.5 text-xs ring-1 ring-primary/20">
      <span className="font-semibold">vs. the other pair: </span>
      {delta === 0 && extra.length === 0 ? (
        <span className="text-muted-foreground">same {thisCount} rows.</span>
      ) : (
        <span className="text-muted-foreground">
          {delta > 0 ? `+${delta}` : delta} rows
          {extra.length > 0 && (
            <>
              {" "}
              — this pair additionally sees{" "}
              <strong className="text-foreground">
                {extra.map((g) => DOC_BY_IRI[g]?.label ?? shortGraph(g)).join(", ")}
              </strong>
            </>
          )}
          .
        </span>
      )}
    </div>
  );
}
