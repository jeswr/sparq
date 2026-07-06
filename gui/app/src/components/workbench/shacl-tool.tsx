"use client";

// [OPUS-4.8] sq-ixc3.11 — the SHACL tool: a surface turned into an operational VERB.
// (sq-txrui) [SONNET-4.6] — upgraded with Turtle syntax highlighting (WorkbenchTurtleEditor,
// sq-5sjub) + bulk multi-file shapes upload with per-source enable/disable toggle (ShaclSources,
// sq-vnh1v / sq-txrui).
//
// Translation rule (research/gui-design.md §A.4/§A.5): the website's /surface/shacl PAGE
// validates a pasted DATA document against pasted SHAPES, wrapped in marketing chrome — a hero,
// an intro paragraph, an honesty TIER CARD, a 6-card capability grid, and an always-open
// "How this runs" / "Honest caveats" block. Here that chrome is CUT. The tool validates the
// ACTIVE workspace's LIVE store (not a fixture) against shapes the operator pastes, and the
// honesty register is carried OPERATIONALLY: the tier dot in the header + the measured-latency
// status line, not prose. There is no datatype-violation "default example" sales pitch — the
// data input IS the store the operator already imported.
//
// What runs: the live store is serialised to TriG (the serialise-rdf binding in the GUI wasm
// bundle; it preserves every named graph and does not abbreviate) and validated against the
// shapes via the shacl binding (Store.validate), all in-tab. This is the same SHACL Core +
// SHACL-SPARQL engine the native crate ships; the `live` tier is honest because the bundle
// genuinely carries it.
//
// SHAPES INPUT — three inputs concatenated (Turtle docs concatenate safely; prefix redeclaration
// is legal per the Turtle spec):
//   1. The inline editor (WorkbenchTurtleEditor with syntax highlighting — sq-5sjub).
//   2. Each ENABLED source in the ShaclSources strip (bulk .ttl upload — sq-txrui).
//   Disabled sources contribute nothing; with zero sources behaviour is identical to before.
//
// INVARIANT (sq-txrui): with zero uploaded sources the tool behaves exactly as before (editor
// doc only). A disabled source contributes nothing. The highlight layer never desyncs from the
// edited text (WorkbenchTurtleEditor owned contract).
//
// Stable E2E hooks preserved from sq-ixc3.11:
//   id="shacl-shapes"             — the textarea inside WorkbenchTurtleEditor
//   [data-result-kind="shacl"]    — the report pane container
//   [data-result-kind="error"]    — the error <pre> inside the report pane

import * as React from "react";
import { Play, Loader2, CheckCircle2, XCircle } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import { useEngine } from "@/lib/engine-context";
import { sparqShaclValidate, type ShaclReport, type ShaclResult } from "@sparq/client";
import { basePath } from "@/lib/base-path";
import { TIER_META, toolById } from "@/data/tools";
import { WorkbenchTurtleEditor } from "@/components/workbench/turtle-editor";
import { ShaclSources, type ShapeSource } from "@/components/workbench/shacl-sources";

// A starter shapes graph the operator edits — NOT a fixture wrapped in a "try me" pitch; just a
// minimal valid SHACL document so the empty editor is not the first thing the operator faces.
// It targets nothing specific in the store; the operator points it at their own classes.
const STARTER_SHAPES = `@prefix sh:   <http://www.w3.org/ns/shacl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

# Point sh:targetClass at a class in YOUR live store, then add constraints.
# Example: every node of ex:Person must have exactly one xsd:string name.
#
# ex:PersonShape a sh:NodeShape ;
#   sh:targetClass ex:Person ;
#   sh:property [ sh:path ex:name ; sh:minCount 1 ; sh:maxCount 1 ; sh:datatype xsd:string ] .
`;

/** Compact a full IRI (optionally `<>`-wrapped) to a CURIE for the well-known SHACL vocab. */
const PREFIXES: [string, string][] = [
  ["http://www.w3.org/ns/shacl#", "sh:"],
  ["http://www.w3.org/2001/XMLSchema#", "xsd:"],
  ["http://www.w3.org/1999/02/22-rdf-syntax-ns#", "rdf:"],
  ["http://www.w3.org/2000/01/rdf-schema#", "rdfs:"],
];
function shortenIri(s: string): string {
  const inner = s.startsWith("<") && s.endsWith(">") ? s.slice(1, -1) : s;
  for (const [ns, prefix] of PREFIXES) {
    if (inner.startsWith(ns)) return prefix + inner.slice(ns.length);
  }
  return s;
}

/** The bare local name of a constraint-component / severity IRI (`…#Foo` → `Foo`). */
function localName(iri: string): string {
  const hash = iri.lastIndexOf("#");
  return hash >= 0 ? iri.slice(hash + 1) : iri;
}

type RunState =
  | { kind: "idle" }
  | { kind: "running" }
  | { kind: "report"; report: ShaclReport; ms: number; storeQuads: number }
  | { kind: "error"; message: string };

function Violation({ r }: { r: ShaclResult }) {
  return (
    <div className="border-b px-3 py-2 text-xs">
      <div className="flex flex-wrap items-center gap-1.5">
        <Badge variant="warning">{localName(r.sourceConstraintComponent)}</Badge>
        <span className="text-muted-foreground">{localName(r.severity)}</span>
      </div>
      <dl className="mt-1 grid grid-cols-[max-content_1fr] gap-x-3 gap-y-0.5 font-mono">
        <dt className="text-muted-foreground">focus</dt>
        <dd className="break-all">{shortenIri(r.focusNode)}</dd>
        {r.path ? (
          <>
            <dt className="text-muted-foreground">path</dt>
            <dd className="break-all">{shortenIri(r.path)}</dd>
          </>
        ) : null}
        {r.value ? (
          <>
            <dt className="text-muted-foreground">value</dt>
            <dd className="break-all">{shortenIri(r.value)}</dd>
          </>
        ) : null}
      </dl>
      {r.message ? <p className="mt-1 text-muted-foreground">{r.message}</p> : null}
    </div>
  );
}

export function ShaclTool() {
  const { status, storeSize, serializeStore } = useEngine();
  const [shapes, setShapes] = React.useState(STARTER_SHAPES);
  // (sq-txrui) uploaded shape sources — each has an id, name, text and enabled flag.
  const [sources, setSources] = React.useState<ShapeSource[]>([]);
  const [state, setState] = React.useState<RunState>({ kind: "idle" });

  const ready = status.kind === "ready";
  const tool = toolById("shacl");
  const tier = tool ? TIER_META[tool.tier] : null;

  const onValidate = React.useCallback(async () => {
    setState({ kind: "running" });
    const data = serializeStore();
    if (data === null) {
      setState({
        kind: "error",
        message:
          "The live store is not ready (or this wasm bundle lacks the serialise binding), so it cannot be serialised for validation.",
      });
      return;
    }
    const t0 = performance.now();
    try {
      // (sq-txrui) — concatenate the inline editor doc + every ENABLED source. Turtle docs
      // concatenate safely; prefix redeclaration is legal per the Turtle spec.
      const allShapes = [shapes, ...sources.filter((s) => s.enabled).map((s) => s.text)].join(
        "\n",
      );
      // Validate the LIVE store (serialised to TriG) against the operator's shapes. The shapes
      // are Turtle; the data is TriG — both are accepted by the validate binding, which folds
      // named graphs into the data graph SHACL validates.
      const report = await sparqShaclValidate(data, allShapes, "trig", {
        basePath: basePath(),
      });
      setState({
        kind: "report",
        report,
        ms: performance.now() - t0,
        storeQuads: storeSize,
      });
    } catch (err) {
      setState({
        kind: "error",
        message: err instanceof Error ? err.message : String(err),
      });
    }
  }, [serializeStore, shapes, sources, storeSize]);

  const onKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      e.preventDefault();
      void onValidate();
    }
  };

  return (
    <div className="flex h-full flex-col">
      {/* Shapes editor + sources strip — the shapes input: the DATA is the live store. */}
      <div className="flex min-h-0 flex-[3] flex-col border-b">
        {/* Editor header: label, tier dot, validate button */}
        <div className="flex shrink-0 items-center gap-2 border-b bg-card px-3 py-1.5">
          <span className="text-xs font-medium text-muted-foreground">SHACL shapes</span>
          {tier ? (
            <span className="flex items-center gap-1.5 text-[11px] text-muted-foreground">
              <span className={cn("size-2 rounded-full", tier.dot)} aria-hidden />
              {tier.label}
            </span>
          ) : null}
          <Button
            size="sm"
            onClick={() => void onValidate()}
            disabled={!ready || state.kind === "running"}
            className="ml-auto"
          >
            {state.kind === "running" ? <Loader2 className="animate-spin" /> : <Play />}
            Validate live store
          </Button>
          <span className="text-[11px] text-muted-foreground">⌘↵</span>
        </div>

        {/* (sq-txrui) Turtle overlay editor — replaces the plain <textarea>. The id="shacl-shapes"
            hook is preserved on the inner <textarea> (WorkbenchTurtleEditor passes it through). */}
        <WorkbenchTurtleEditor
          id="shacl-shapes"
          value={shapes}
          onChange={setShapes}
          onKeyDown={onKeyDown}
          ariaLabel="SHACL shapes editor"
          placeholder="Paste a SHACL shapes graph (Turtle)…"
        />

        {/* (sq-txrui) SHAPES SOURCES strip — bulk .ttl upload, per-source toggle. */}
        <ShaclSources sources={sources} onChange={setSources} />
      </div>

      {/* Report pane — conformance flag + per-violation W3C report over the live store. */}
      <div className="flex min-h-0 flex-[2] flex-col" data-result-kind="shacl">
        <div className="flex items-center gap-2 border-b bg-card px-3 py-1 text-xs">
          {state.kind === "report" ? (
            state.report.conforms ? (
              <span className="flex items-center gap-1.5 text-[var(--success)]">
                <CheckCircle2 className="size-3.5" /> Conforms
              </span>
            ) : (
              <span className="flex items-center gap-1.5 text-[var(--warning)]">
                <XCircle className="size-3.5" />
                {state.report.results.length}{" "}
                {state.report.results.length === 1 ? "violation" : "violations"}
              </span>
            )
          ) : (
            <span className="text-muted-foreground">Validation report</span>
          )}
          {state.kind === "report" ? (
            <span
              className="ml-auto tabular text-[11px] text-muted-foreground"
              title="Wall-clock latency of this validation (performance.now)"
            >
              {state.ms.toFixed(1)} ms over {state.storeQuads.toLocaleString()} quads
            </span>
          ) : null}
        </div>
        <div className="min-h-0 flex-1 overflow-auto">
          {state.kind === "idle" ? (
            <p className="p-3 text-sm text-muted-foreground">
              {ready
                ? "Validate the live store against the shapes above."
                : "Waiting for the engine to warm…"}
            </p>
          ) : state.kind === "running" ? (
            <p className="p-3 text-sm text-muted-foreground">Validating…</p>
          ) : state.kind === "error" ? (
            <pre
              className="overflow-auto whitespace-pre-wrap p-3 font-mono text-xs text-destructive"
              data-result-kind="error"
            >
              {state.message}
            </pre>
          ) : state.report.conforms ? (
            <p className="p-3 text-sm text-muted-foreground">
              No violations — the live store conforms to the shapes.
            </p>
          ) : (
            <div>
              {state.report.results.map((r, i) => (
                <Violation key={i} r={r} />
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
