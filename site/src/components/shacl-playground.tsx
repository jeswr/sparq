"use client";

// [OPUS-4.8] sq-egy6 — the live /surface/shacl playground: two textareas (data +
// shapes) -> conformance flag + per-violation W3C report, run entirely in your tab
// via the shacl-enabled wasm bundle (Store.validate). The default example is the
// `ex:age "thirty"` datatype violation from skills/shacl-validation/SKILL.md.

import * as React from "react";
import {
  Play,
  Loader2,
  ShieldCheck,
  CheckCircle2,
  XCircle,
} from "lucide-react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { cn } from "@/lib/utils";
import {
  prewarmSparq,
  sparqShaclValidate,
  type ShaclReport,
} from "@/lib/sparq-wasm";
import {
  componentName,
  reportSummary,
  reportToTurtle,
  severityName,
  shortenIri,
} from "@/lib/shacl-report";
import { SHACL_EXAMPLES } from "@/data/shacl-examples";

type RunState =
  | { kind: "idle" }
  | { kind: "running" }
  | { kind: "report"; report: ShaclReport; ms: number }
  | { kind: "error"; message: string };

type EngineState = "cold" | "warming" | "ready" | "error";
type View = "results" | "turtle";

const DEFAULT = SHACL_EXAMPLES[0];

export function ShaclPlayground() {
  const [data, setData] = React.useState(DEFAULT.data);
  const [shapes, setShapes] = React.useState(DEFAULT.shapes);
  const [state, setState] = React.useState<RunState>({ kind: "idle" });
  const [engine, setEngine] = React.useState<EngineState>("cold");
  const [activeExample, setActiveExample] = React.useState<string>(DEFAULT.id);
  const [view, setView] = React.useState<View>("results");

  // Pre-warm the wasm engine on mount (off the render path) so the first "Validate"
  // pays no cold start. A failure resets the indicator; validate() retries the load.
  React.useEffect(() => {
    let cancelled = false;
    setEngine("warming");
    prewarmSparq()
      .then(() => {
        if (!cancelled) setEngine("ready");
      })
      .catch((e) => {
        if (cancelled) return;
        setEngine("error");
        toast.error("Engine failed to load", {
          description: e instanceof Error ? e.message : String(e),
        });
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const run = React.useCallback(async () => {
    setState({ kind: "running" });
    try {
      const t0 = performance.now();
      const report = await sparqShaclValidate(data, shapes, "turtle");
      const ms = performance.now() - t0;
      setEngine("ready");
      setState({ kind: "report", report, ms });
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setState({ kind: "error", message });
      toast.error("Validation failed", { description: message });
    }
  }, [data, shapes]);

  const selectExample = React.useCallback((id: string) => {
    const ex = SHACL_EXAMPLES.find((e) => e.id === id);
    if (!ex) return;
    setData(ex.data);
    setShapes(ex.shapes);
    setActiveExample(id);
    setState({ kind: "idle" });
  }, []);

  const busy = state.kind === "running";

  return (
    <Card>
      <CardHeader className="flex-row items-center justify-between gap-2 space-y-0">
        <CardTitle className="flex items-center gap-2 text-base">
          <ShieldCheck className="size-4 text-primary" />
          Live SHACL validator
        </CardTitle>
        <EngineIndicator engine={engine} />
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="flex flex-wrap gap-1.5">
          {SHACL_EXAMPLES.map((ex) => (
            <Button
              key={ex.id}
              variant={activeExample === ex.id ? "default" : "outline"}
              size="sm"
              onClick={() => selectExample(ex.id)}
              title={ex.description}
            >
              {ex.label}
            </Button>
          ))}
        </div>

        <div className="grid gap-3 md:grid-cols-2">
          <div className="space-y-1.5">
            <label
              htmlFor="shacl-data"
              className="text-xs font-medium text-muted-foreground"
            >
              Data graph (Turtle)
            </label>
            <textarea
              id="shacl-data"
              value={data}
              spellCheck={false}
              onChange={(e) => setData(e.target.value)}
              rows={12}
              className="w-full resize-y rounded-lg border bg-muted/40 p-3 font-mono text-[12.5px] leading-relaxed outline-none focus-visible:ring-3 focus-visible:ring-ring/40"
            />
          </div>
          <div className="space-y-1.5">
            <label
              htmlFor="shacl-shapes"
              className="text-xs font-medium text-muted-foreground"
            >
              Shapes graph (Turtle)
            </label>
            <textarea
              id="shacl-shapes"
              value={shapes}
              spellCheck={false}
              onChange={(e) => setShapes(e.target.value)}
              rows={12}
              className="w-full resize-y rounded-lg border bg-muted/40 p-3 font-mono text-[12.5px] leading-relaxed outline-none focus-visible:ring-3 focus-visible:ring-ring/40"
            />
          </div>
        </div>

        <div className="flex flex-wrap items-center gap-3">
          <Button onClick={run} disabled={busy}>
            {busy ? (
              <Loader2 className="size-4 animate-spin" />
            ) : (
              <Play className="size-4" />
            )}
            Validate
          </Button>
          {state.kind === "report" && (
            <ViewTabs view={view} onChange={setView} />
          )}
          <p aria-live="polite" className="text-xs text-muted-foreground">
            {state.kind === "report" &&
              `${state.report.results.length} result${state.report.results.length === 1 ? "" : "s"} · ${state.ms.toFixed(1)} ms`}
            {state.kind === "running" && "Validating on the wasm engine…"}
            {state.kind === "idle" &&
              engine === "warming" &&
              "Pre-warming the wasm engine…"}
          </p>
        </div>

        <ResultPanel state={state} view={view} />
      </CardContent>
    </Card>
  );
}

function EngineIndicator({ engine }: { engine: EngineState }) {
  if (engine === "ready") {
    return (
      <Badge variant="success" aria-live="polite">
        <CheckCircle2 className="size-3" /> Engine ready
      </Badge>
    );
  }
  if (engine === "error") {
    return (
      <Badge variant="warning" aria-live="polite">
        Engine failed — retries on validate
      </Badge>
    );
  }
  return (
    <Badge variant="muted" aria-live="polite">
      <Loader2 className="size-3 animate-spin" /> Engine loading…
    </Badge>
  );
}

function ViewTabs({
  view,
  onChange,
}: {
  view: View;
  onChange: (v: View) => void;
}) {
  const tabs: { value: View; label: string }[] = [
    { value: "results", label: "Results" },
    { value: "turtle", label: "W3C Turtle" },
  ];
  return (
    <div
      role="tablist"
      aria-label="Report view"
      className="inline-flex rounded-lg border bg-muted/40 p-0.5"
    >
      {tabs.map((t) => (
        <button
          key={t.value}
          type="button"
          role="tab"
          aria-selected={view === t.value}
          onClick={() => onChange(t.value)}
          className={cn(
            "rounded-md px-2.5 py-1 text-xs font-medium transition-colors outline-none focus-visible:ring-3 focus-visible:ring-ring/40",
            view === t.value
              ? "bg-background text-foreground shadow-sm"
              : "text-muted-foreground hover:text-foreground",
          )}
        >
          {t.label}
        </button>
      ))}
    </div>
  );
}

function ResultPanel({ state, view }: { state: RunState; view: View }) {
  if (state.kind === "error") {
    return (
      <pre className="overflow-x-auto rounded-lg border border-destructive/30 bg-destructive/5 p-3 text-xs text-destructive">
        {state.message}
      </pre>
    );
  }
  if (state.kind !== "report") return null;
  const { report } = state;

  return (
    <div className="space-y-3">
      <ConformanceBanner conforms={report.conforms} summary={reportSummary(report)} />
      {view === "turtle" ? (
        <pre className="max-h-96 overflow-auto rounded-lg border bg-muted/40 p-3 font-mono text-[12.5px] leading-relaxed">
          {reportToTurtle(report)}
        </pre>
      ) : (
        <ResultList report={report} />
      )}
    </div>
  );
}

function ConformanceBanner({
  conforms,
  summary,
}: {
  conforms: boolean;
  summary: string;
}) {
  return (
    <div
      className={cn(
        "flex items-center gap-2 rounded-lg p-3 text-sm font-medium",
        conforms
          ? "bg-[color-mix(in_oklch,var(--success)_15%,transparent)] text-[var(--success)]"
          : "bg-destructive/10 text-destructive",
      )}
    >
      {conforms ? (
        <CheckCircle2 className="size-4" aria-hidden="true" />
      ) : (
        <XCircle className="size-4" aria-hidden="true" />
      )}
      <span>
        <span className="font-mono">sh:conforms</span> = {conforms ? "true" : "false"}{" "}
        — {summary}
      </span>
    </div>
  );
}

function ResultList({ report }: { report: ShaclReport }) {
  if (report.results.length === 0) {
    return (
      <p className="rounded-lg border bg-muted/30 p-3 text-sm text-muted-foreground">
        No violations — every targeted node satisfies its shape.
      </p>
    );
  }
  return (
    <ul className="space-y-2">
      {report.results.map((r, i) => (
        <li
          key={i}
          className="rounded-lg border bg-muted/30 p-3 text-sm"
        >
          <div className="mb-1.5 flex flex-wrap items-center gap-2">
            <Badge variant="warning" className="font-mono text-[11px]">
              {componentName(r.sourceConstraintComponent)}
            </Badge>
            <Badge variant="muted" className="text-[11px]">
              {severityName(r.severity)}
            </Badge>
          </div>
          {r.message && (
            <p className="mb-1.5 text-foreground">{r.message}</p>
          )}
          <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-0.5 font-mono text-[12px] text-muted-foreground">
            <dt>focus</dt>
            <dd className="break-all text-foreground">{shortenIri(r.focusNode)}</dd>
            {r.path && (
              <>
                <dt>path</dt>
                <dd className="break-all text-foreground">{shortenIri(r.path)}</dd>
              </>
            )}
            {r.value && (
              <>
                <dt>value</dt>
                <dd className="break-all text-foreground">{shortenIri(r.value)}</dd>
              </>
            )}
          </dl>
        </li>
      ))}
    </ul>
  );
}
