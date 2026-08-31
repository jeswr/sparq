"use client";

// [OPUS-5] sq-ixc3.21 — the dataset OVERVIEW tool: the read-side "what IS this dataset" tab
// (research/competitive-feature-analysis-2026-07.md names the GraphDB Explore views as a cheap,
// high-value parity gap). Three views over ONE snapshot of the live store:
//
//   1. class-hierarchy bubble — a class per bubble, area ∝ its direct instance count, nested by
//      the asserted `rdfs:subClassOf` axioms. Click a bubble to drill into its instances.
//   2. class-relationship chord — a ribbon per ordered class pair, width ∝ the statements
//      between their instances. Click a ribbon for the per-predicate breakdown.
//   3. domain–range panel — the OBSERVED predicate signatures (domain class → range class or
//      datatype) with the statement count behind each one.
//
// HONESTY. Everything on this panel comes from four aggregate SPARQL queries the user can read
// (the "Queries" disclosure at the bottom) run over the live in-tab store — no canned figure, no
// sampling, no estimate. The counts are of ASSERTED statements: no entailment is applied here, so
// a class's bubble shows DIRECT `rdf:type` instances, and the domain–range rows are what the data
// contains rather than what an `rdfs:domain`/`rdfs:range` axiom declares. The panel says so.
// A query that fails is reported by name instead of being silently dropped, and the view caps
// (top-N chord classes, per-query row limits, the drill-down instance limit) are all stated.
// The snapshot does not follow the store: importing data marks it stale and asks for a Refresh
// rather than quietly showing figures for a store that no longer exists.
//
// The two SVGs are dependency-free (same reasoning as graph-view.tsx: no d3 / cytoscape for a
// view this size); the layout maths lives in lib/dataset-overview.ts and is unit-tested there.

import * as React from "react";
import { Loader2, PieChart, RefreshCw } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { useEngine } from "@/lib/engine-context";
import {
  CHORD_MAX_CLASSES,
  CLASS_COUNT_QUERY,
  CLASS_RELATION_QUERY,
  LITERAL_RANGE_QUERY,
  SUBCLASS_QUERY,
  abbreviateIri,
  buildChordModel,
  buildClassHierarchy,
  buildDomainRangeRows,
  instanceListQuery,
  packHierarchy,
  parseClassRows,
  parseInstanceRows,
  parseLiteralRangeRows,
  parseRelationRows,
  parseSubClassEdges,
  predicatesBetween,
  type BubblePack,
  type ChordModel,
  type ChordRibbon,
  type ClassRow,
  type DomainRangeRow,
  type RelationRow,
} from "@/lib/dataset-overview";

/** Rows the bubble drill-down asks for. A view bound, stated in the drill-down header. */
const INSTANCE_LIMIT = 50;
/** Domain–range rows rendered before the table says how many more there are. */
const DOMAIN_RANGE_ROWS = 60;

/** One computed snapshot of the store — every view on this panel reads from the same one. */
interface Snapshot {
  classes: ClassRow[];
  pack: BubblePack;
  relations: RelationRow[];
  chord: ChordModel;
  domainRange: DomainRangeRow[];
  /** Queries that did not return a result table, named so a partial view is never silent. */
  failures: string[];
  /** The store size the snapshot was taken at — drives the "stale, Refresh" notice. */
  storeSize: number;
}

type LoadState =
  | { kind: "idle" }
  | { kind: "loading" }
  | { kind: "ready"; snapshot: Snapshot }
  | { kind: "error"; message: string };

/** What the user drilled into, if anything. */
type Selection =
  | { kind: "class"; iri: string }
  | { kind: "pair"; source: string; target: string }
  | null;

type InstanceState =
  | { kind: "loading" }
  | { kind: "ready"; rows: Array<{ kind: string; value: string }> }
  | { kind: "error"; message: string };

export function OverviewTool() {
  const { run, status, storeSize } = useEngine();
  const [state, setState] = React.useState<LoadState>({ kind: "idle" });
  const [selection, setSelection] = React.useState<Selection>(null);
  const [instances, setInstances] = React.useState<InstanceState | null>(null);
  const requestRef = React.useRef(0);
  const instanceRef = React.useRef(0);

  const ready = status.kind === "ready";

  const load = React.useCallback(async () => {
    const ticket = requestRef.current + 1;
    requestRef.current = ticket;
    setState({ kind: "loading" });
    setSelection(null);
    setInstances(null);

    const failures: string[] = [];
    const ask = async (label: string, query: string) => {
      const { outcome } = await run(query);
      if (outcome.kind === "select") return outcome.results;
      failures.push(
        `${label} — ${outcome.kind === "error" ? outcome.message : `no result table (${outcome.kind})`}`,
      );
      return null;
    };

    try {
      // Sequential on purpose: one in-tab store, one query at a time.
      const classResults = await ask("class counts", CLASS_COUNT_QUERY);
      const subClassResults = await ask("subclass axioms", SUBCLASS_QUERY);
      const relationResults = await ask("class relationships", CLASS_RELATION_QUERY);
      const literalResults = await ask("literal ranges", LITERAL_RANGE_QUERY);
      if (requestRef.current !== ticket) return; // superseded by a later Refresh

      if (classResults === null && relationResults === null) {
        setState({
          kind: "error",
          message: `The overview queries did not run: ${failures.join("; ")}`,
        });
        return;
      }

      const classes = classResults ? parseClassRows(classResults) : [];
      const edges = subClassResults ? parseSubClassEdges(subClassResults) : [];
      const relations = relationResults ? parseRelationRows(relationResults) : [];
      const literals = literalResults ? parseLiteralRangeRows(literalResults) : [];
      setState({
        kind: "ready",
        snapshot: {
          classes,
          pack: packHierarchy(buildClassHierarchy(classes, edges)),
          relations,
          chord: buildChordModel(relations),
          domainRange: buildDomainRangeRows(relations, literals),
          failures,
          storeSize,
        },
      });
    } catch (err) {
      if (requestRef.current !== ticket) return;
      setState({ kind: "error", message: err instanceof Error ? err.message : String(err) });
    }
  }, [run, storeSize]);

  // Compute once the engine is warm; after that the user drives it with Refresh (the notice
  // below says when the store has moved on from the snapshot).
  React.useEffect(() => {
    if (ready && state.kind === "idle") void load();
  }, [ready, state.kind, load]);

  const onSelectClass = React.useCallback(
    async (iri: string) => {
      setSelection({ kind: "class", iri });
      const query = instanceListQuery(iri, INSTANCE_LIMIT);
      if (query === null) {
        setInstances({
          kind: "error",
          message: "This class IRI cannot be written into a SPARQL query safely.",
        });
        return;
      }
      const ticket = instanceRef.current + 1;
      instanceRef.current = ticket;
      setInstances({ kind: "loading" });
      try {
        const { outcome } = await run(query);
        if (instanceRef.current !== ticket) return;
        if (outcome.kind === "select") {
          setInstances({ kind: "ready", rows: parseInstanceRows(outcome.results) });
        } else {
          setInstances({
            kind: "error",
            message: outcome.kind === "error" ? outcome.message : `no result table (${outcome.kind})`,
          });
        }
      } catch (err) {
        if (instanceRef.current !== ticket) return;
        setInstances({ kind: "error", message: err instanceof Error ? err.message : String(err) });
      }
    },
    [run],
  );

  const snapshot = state.kind === "ready" ? state.snapshot : null;
  const stale = snapshot !== null && snapshot.storeSize !== storeSize;

  return (
    <div className="flex h-full flex-col" data-tool-panel="overview">
      <div className="flex items-center gap-2 border-b bg-card px-3 py-1.5">
        <PieChart className="size-3.5 shrink-0 text-muted-foreground" />
        <span className="text-xs font-medium text-muted-foreground">Overview</span>
        <Badge variant="outline" className="h-5 gap-1 text-[10px]" title="Where these queries run">
          LOCAL · in-tab WASM
        </Badge>
        <div className="ml-auto flex items-center gap-1.5">
          <Button
            size="sm"
            onClick={() => void load()}
            disabled={!ready || state.kind === "loading"}
            title="Re-run the four aggregate queries over the live store"
            data-overview-refresh
          >
            {state.kind === "loading" ? (
              <Loader2 className="size-3.5 animate-spin" />
            ) : (
              <RefreshCw className="size-3.5" />
            )}
            Refresh
          </Button>
        </div>
      </div>

      <div className="min-h-0 flex-1 overflow-auto" data-overview-body>
        {!ready ? (
          <p className="p-4 text-sm text-muted-foreground">Waiting for the engine to warm…</p>
        ) : state.kind === "error" ? (
          <pre
            className="overflow-auto whitespace-pre-wrap p-3 font-mono text-xs text-destructive"
            data-overview-error
          >
            {state.message}
          </pre>
        ) : snapshot === null ? (
          <p className="p-4 text-sm text-muted-foreground">
            Summarising the live store with four aggregate queries…
          </p>
        ) : (
          <div className="flex flex-col gap-4 p-3">
            {stale && (
              <p
                className="rounded-md border border-[var(--warning)]/40 bg-[var(--warning)]/5 p-2 text-xs text-muted-foreground"
                data-overview-stale
              >
                The store has changed since this snapshot was taken (
                {snapshot.storeSize.toLocaleString()} → {storeSize.toLocaleString()} quads).
                Refresh to recompute.
              </p>
            )}
            {snapshot.failures.length > 0 && (
              <ul
                className="rounded-md border border-destructive/40 bg-destructive/5 p-2 text-xs text-muted-foreground"
                data-overview-failures
              >
                {snapshot.failures.map((f) => (
                  <li key={f}>Query failed: {f}</li>
                ))}
              </ul>
            )}

            <ClassBubbles
              snapshot={snapshot}
              selection={selection}
              instances={instances}
              onSelectClass={(iri) => void onSelectClass(iri)}
            />
            <ClassChord
              snapshot={snapshot}
              selection={selection}
              onSelectPair={(source, target) => {
                setSelection({ kind: "pair", source, target });
                setInstances(null);
              }}
            />
            <DomainRangePanel rows={snapshot.domainRange} />
            <QueryDisclosure />
          </div>
        )}
      </div>
    </div>
  );
}

/** A titled panel section — the three views share the chrome. `dataKey` is the e2e handle. */
function Section({
  title,
  note,
  dataKey,
  children,
}: {
  title: string;
  note: React.ReactNode;
  dataKey: string;
  children: React.ReactNode;
}) {
  return (
    <section className="rounded-md border bg-card" data-overview-section={dataKey}>
      <header className="border-b px-3 py-1.5">
        <h2 className="text-xs font-medium">{title}</h2>
        <p className="mt-0.5 text-[11px] leading-snug text-muted-foreground">{note}</p>
      </header>
      {children}
    </section>
  );
}

// ── 1. Class-hierarchy bubble ──────────────────────────────────────────────────────────────────

function ClassBubbles({
  snapshot,
  selection,
  instances,
  onSelectClass,
}: {
  snapshot: Snapshot;
  selection: Selection;
  instances: InstanceState | null;
  onSelectClass: (iri: string) => void;
}) {
  const { pack, classes } = snapshot;
  const selected = selection?.kind === "class" ? selection.iri : null;

  return (
    <Section
      title="Classes"
      note={
        <>
          One bubble per class, area proportional to its <strong>direct</strong> instance count (
          no entailment applied), nested by asserted <code>rdfs:subClassOf</code>. Click a bubble
          for its instances.
        </>
      }
      dataKey="bubbles"
    >
      {pack.bubbles.length === 0 ? (
        <p className="p-3 text-sm text-muted-foreground">
          No classes: the live store has no <code>rdf:type</code> statements.
        </p>
      ) : (
        <>
          <div className="p-3">
            <svg
              viewBox={`0 0 ${pack.width} ${pack.height}`}
              className="mx-auto h-auto w-full max-w-[620px]"
              role="img"
              aria-label="Class hierarchy bubble chart, sized by instance count"
            >
              {pack.bubbles.map((b) => {
                const isSelected = b.iri === selected;
                const parents =
                  b.otherParents.length > 0
                    ? ` — also a subclass of ${b.otherParents.map(abbreviateIri).join(", ")}`
                    : "";
                return (
                  <g
                    key={b.iri}
                    role="button"
                    tabIndex={0}
                    className="cursor-pointer focus:outline-1 focus:outline-primary"
                    onClick={() => onSelectClass(b.iri)}
                    onKeyDown={(ev) => {
                      if (ev.key === "Enter" || ev.key === " ") {
                        ev.preventDefault();
                        onSelectClass(b.iri);
                      }
                    }}
                    aria-label={`${b.label}: ${b.instances} direct instances`}
                    data-overview-bubble={b.label}
                  >
                    <title>
                      {`${b.label} — ${b.instances.toLocaleString()} direct instances`}
                      {b.totalInstances !== b.instances
                        ? ` (${b.totalInstances.toLocaleString()} including subclasses)`
                        : ""}
                      {parents}
                    </title>
                    <circle
                      cx={b.x}
                      cy={b.y}
                      r={b.r}
                      fill="var(--primary)"
                      fillOpacity={b.depth === 0 ? 0.1 : 0.16 + 0.08 * b.depth}
                      stroke={isSelected ? "var(--primary)" : "var(--border)"}
                      strokeWidth={isSelected ? 2 : 1}
                    />
                    {/* A parent's own label sits at the top of its ring so it does not collide
                        with the child bubbles packed in the middle. */}
                    {b.r >= 18 && (
                      <text
                        x={b.x}
                        y={b.y}
                        className="fill-foreground"
                        fontSize={9}
                        textAnchor="middle"
                        dy={b.depth === 0 && b.totalInstances !== b.instances ? -b.r + 12 : 3}
                      >
                        {b.label}
                      </text>
                    )}
                  </g>
                );
              })}
            </svg>
          </div>
          <p className="border-t px-3 py-1.5 text-[11px] text-muted-foreground">
            {classes.length.toLocaleString()} classes ·{" "}
            {classes.reduce((sum, c) => sum + c.instances, 0).toLocaleString()} typed instances
            (summed over classes; a resource with several types counts once per type).
          </p>
          {selected !== null && (
            <InstancePanel iri={selected} instances={instances} />
          )}
        </>
      )}
    </Section>
  );
}

function InstancePanel({ iri, instances }: { iri: string; instances: InstanceState | null }) {
  return (
    <div className="border-t px-3 py-2" data-overview-instances>
      <p className="text-[11px] font-medium">
        Instances of <code>{abbreviateIri(iri)}</code>
        <span className="ml-1 font-normal text-muted-foreground">
          (first {INSTANCE_LIMIT} the engine returns)
        </span>
      </p>
      {instances === null || instances.kind === "loading" ? (
        <p className="mt-1 text-xs text-muted-foreground">Running the instance query…</p>
      ) : instances.kind === "error" ? (
        <p className="mt-1 font-mono text-xs text-destructive">{instances.message}</p>
      ) : instances.rows.length === 0 ? (
        <p className="mt-1 text-xs text-muted-foreground">No instances returned.</p>
      ) : (
        <ul className="mt-1 max-h-40 overflow-auto font-mono text-[11px]">
          {instances.rows.map((row) => (
            <li key={`${row.kind}:${row.value}`} className="truncate" title={row.value}>
              {row.kind === "bnode" ? `_:${row.value}` : row.value}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

// ── 2. Class-relationship chord ────────────────────────────────────────────────────────────────

const CHORD_SIZE = 340;
const CHORD_CENTRE = CHORD_SIZE / 2;
const CHORD_OUTER = 132;
const CHORD_INNER = 122;

/** Angle (0 at 12 o'clock, clockwise) + radius → a point in the chord SVG's coordinates. */
function polar(angle: number, radius: number): { x: number; y: number } {
  return {
    x: CHORD_CENTRE + radius * Math.sin(angle),
    y: CHORD_CENTRE - radius * Math.cos(angle),
  };
}

/** The annulus segment for one class arc. */
function arcPath(start: number, end: number): string {
  const large = end - start > Math.PI ? 1 : 0;
  const o0 = polar(start, CHORD_OUTER);
  const o1 = polar(end, CHORD_OUTER);
  const i1 = polar(end, CHORD_INNER);
  const i0 = polar(start, CHORD_INNER);
  return [
    `M ${o0.x} ${o0.y}`,
    `A ${CHORD_OUTER} ${CHORD_OUTER} 0 ${large} 1 ${o1.x} ${o1.y}`,
    `L ${i1.x} ${i1.y}`,
    `A ${CHORD_INNER} ${CHORD_INNER} 0 ${large} 0 ${i0.x} ${i0.y}`,
    "Z",
  ].join(" ");
}

/** The ribbon: the two endpoint chords joined through the centre of the circle. */
function ribbonPath(r: ChordRibbon): string {
  const sourceLarge = r.sourceEndAngle - r.sourceStartAngle > Math.PI ? 1 : 0;
  const targetLarge = r.targetEndAngle - r.targetStartAngle > Math.PI ? 1 : 0;
  const s0 = polar(r.sourceStartAngle, CHORD_INNER);
  const s1 = polar(r.sourceEndAngle, CHORD_INNER);
  const t0 = polar(r.targetStartAngle, CHORD_INNER);
  const t1 = polar(r.targetEndAngle, CHORD_INNER);
  return [
    `M ${s0.x} ${s0.y}`,
    `A ${CHORD_INNER} ${CHORD_INNER} 0 ${sourceLarge} 1 ${s1.x} ${s1.y}`,
    `Q ${CHORD_CENTRE} ${CHORD_CENTRE} ${t0.x} ${t0.y}`,
    `A ${CHORD_INNER} ${CHORD_INNER} 0 ${targetLarge} 1 ${t1.x} ${t1.y}`,
    `Q ${CHORD_CENTRE} ${CHORD_CENTRE} ${s0.x} ${s0.y}`,
    "Z",
  ].join(" ");
}

/** Deterministic per-arc hue so the same dataset always colours the same way. */
function arcColour(index: number): string {
  return `hsl(${(index * 47) % 360} 58% 52%)`;
}

function ClassChord({
  snapshot,
  selection,
  onSelectPair,
}: {
  snapshot: Snapshot;
  selection: Selection;
  onSelectPair: (source: string, target: string) => void;
}) {
  const { chord, relations } = snapshot;
  const hueOf = React.useMemo(
    () => new Map(chord.arcs.map((a, i) => [a.iri, arcColour(i)] as const)),
    [chord.arcs],
  );
  const selectedPair = selection?.kind === "pair" ? selection : null;

  return (
    <Section
      title="Class relationships"
      note={
        <>
          A ribbon per ordered class pair, width proportional to the statements from instances of
          one class to instances of the other (all predicates except <code>rdf:type</code>
          {" "}summed). Click a ribbon for its predicates.
        </>
      }
      dataKey="chord"
    >
      {chord.arcs.length === 0 ? (
        <p className="p-3 text-sm text-muted-foreground">
          No relationships: no statement in the store links an instance of one class to an
          instance of another.
        </p>
      ) : (
        <>
          <div className="p-3">
            <svg
              viewBox={`0 0 ${CHORD_SIZE} ${CHORD_SIZE}`}
              className="mx-auto h-auto w-full max-w-[420px]"
              role="img"
              aria-label="Chord diagram of statement counts between classes"
            >
              {chord.ribbons.map((r) => {
                const isSelected =
                  selectedPair?.source === r.source && selectedPair?.target === r.target;
                return (
                  <g
                    key={`${r.source} ${r.target}`}
                    role="button"
                    tabIndex={0}
                    className="cursor-pointer focus:outline-1 focus:outline-primary"
                    onClick={() => onSelectPair(r.source, r.target)}
                    onKeyDown={(ev) => {
                      if (ev.key === "Enter" || ev.key === " ") {
                        ev.preventDefault();
                        onSelectPair(r.source, r.target);
                      }
                    }}
                    aria-label={`${abbreviateIri(r.source)} to ${abbreviateIri(r.target)}: ${r.count} statements`}
                    data-overview-ribbon={`${abbreviateIri(r.source)}->${abbreviateIri(r.target)}`}
                  >
                    <title>
                      {`${abbreviateIri(r.source)} → ${abbreviateIri(r.target)}: ${r.count.toLocaleString()} statements`}
                    </title>
                    <path
                      d={ribbonPath(r)}
                      fill={hueOf.get(r.source) ?? "var(--primary)"}
                      fillOpacity={isSelected ? 0.75 : 0.4}
                      stroke={isSelected ? "var(--foreground)" : "none"}
                      strokeWidth={0.75}
                    />
                  </g>
                );
              })}
              {chord.arcs.map((a) => {
                const mid = (a.startAngle + a.endAngle) / 2;
                const label = polar(mid, CHORD_OUTER + 8);
                const east = Math.sin(mid) >= 0;
                return (
                  <g key={a.iri} data-overview-arc={a.label}>
                    <title>
                      {`${a.label}: ${a.value.toLocaleString()} statement endpoints`}
                    </title>
                    <path d={arcPath(a.startAngle, a.endAngle)} fill={hueOf.get(a.iri)} />
                    <text
                      x={label.x}
                      y={label.y}
                      className="fill-foreground"
                      fontSize={9}
                      textAnchor={east ? "start" : "end"}
                      dominantBaseline="middle"
                    >
                      {a.label}
                    </text>
                  </g>
                );
              })}
            </svg>
          </div>
          <p className="border-t px-3 py-1.5 text-[11px] text-muted-foreground">
            {chord.shownStatements.toLocaleString()} statements between{" "}
            {chord.arcs.length.toLocaleString()} classes.
            {chord.hiddenClasses > 0 && (
              <>
                {" "}
                {chord.hiddenClasses.toLocaleString()} further class
                {chord.hiddenClasses === 1 ? "" : "es"} (
                {chord.hiddenStatements.toLocaleString()} statements) are outside the top-
                {CHORD_MAX_CLASSES} the chord shows.
              </>
            )}
          </p>
          {selectedPair && (
            <PairPanel
              relations={relations}
              source={selectedPair.source}
              target={selectedPair.target}
            />
          )}
        </>
      )}
    </Section>
  );
}

function PairPanel({
  relations,
  source,
  target,
}: {
  relations: readonly RelationRow[];
  source: string;
  target: string;
}) {
  const rows = predicatesBetween(relations, source, target);
  return (
    <div className="border-t px-3 py-2" data-overview-pair>
      <p className="text-[11px] font-medium">
        <code>{abbreviateIri(source)}</code> → <code>{abbreviateIri(target)}</code> predicates
      </p>
      {rows.length === 0 ? (
        <p className="mt-1 text-xs text-muted-foreground">No predicates for this pair.</p>
      ) : (
        <ul className="mt-1 max-h-40 overflow-auto text-[11px]">
          {rows.map((row) => (
            <li key={row.predicate} className="flex gap-2">
              <code className="min-w-0 flex-1 truncate" title={row.predicate}>
                {abbreviateIri(row.predicate)}
              </code>
              <span className="tabular-nums text-muted-foreground">
                {row.count.toLocaleString()}
              </span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

// ── 3. Domain–range panel ──────────────────────────────────────────────────────────────────────

function DomainRangePanel({ rows }: { rows: readonly DomainRangeRow[] }) {
  const shown = rows.slice(0, DOMAIN_RANGE_ROWS);
  return (
    <Section
      title="Domain–range"
      note={
        <>
          The signatures the data <strong>actually contains</strong> — not the declared{" "}
          <code>rdfs:domain</code> / <code>rdfs:range</code> axioms, which may differ.
        </>
      }
      dataKey="domain-range"
    >
      {shown.length === 0 ? (
        <p className="p-3 text-sm text-muted-foreground">
          No predicate signatures: the store has no statements on typed subjects.
        </p>
      ) : (
        <>
          <table className="w-full text-[11px]">
            <thead className="text-muted-foreground">
              <tr className="border-b">
                <th className="px-3 py-1 text-left font-medium">Predicate</th>
                <th className="px-3 py-1 text-left font-medium">Domain</th>
                <th className="px-3 py-1 text-left font-medium">Range</th>
                <th className="px-3 py-1 text-right font-medium">Statements</th>
              </tr>
            </thead>
            <tbody>
              {shown.map((row) => (
                <tr
                  key={`${row.predicate} ${row.domain} ${row.range ?? ""} ${row.rangeKind}`}
                  className="border-b last:border-b-0"
                >
                  <td className="px-3 py-1 font-mono" title={row.predicate}>
                    {row.predicateLabel}
                  </td>
                  <td className="px-3 py-1 font-mono" title={row.domain}>
                    {row.domainLabel}
                  </td>
                  <td className="px-3 py-1 font-mono" title={row.range ?? "untyped literal"}>
                    <span
                      className={cn(
                        "mr-1 inline-block rounded px-1 text-[9px] uppercase",
                        row.rangeKind === "class"
                          ? "bg-primary/10 text-primary"
                          : "bg-muted text-muted-foreground",
                      )}
                    >
                      {row.rangeKind === "class" ? "class" : "literal"}
                    </span>
                    {row.rangeLabel}
                  </td>
                  <td className="px-3 py-1 text-right tabular-nums">
                    {row.count.toLocaleString()}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          {rows.length > shown.length && (
            <p className="border-t px-3 py-1.5 text-[11px] text-muted-foreground">
              Showing the {shown.length} most frequent of {rows.length.toLocaleString()}{" "}
              signatures.
            </p>
          )}
        </>
      )}
    </Section>
  );
}

// ── The queries, in the open ───────────────────────────────────────────────────────────────────

function QueryDisclosure() {
  return (
    <details className="rounded-md border bg-card" data-overview-queries>
      <summary className="cursor-pointer px-3 py-1.5 text-xs font-medium">
        Queries behind this overview
      </summary>
      <div className="border-t px-3 py-2">
        <p className="mb-2 text-[11px] text-muted-foreground">
          Run in order over the live store. Copy any of them into the Query tool to see the rows
          this panel is drawn from.
        </p>
        {[CLASS_COUNT_QUERY, SUBCLASS_QUERY, CLASS_RELATION_QUERY, LITERAL_RANGE_QUERY].map(
          (query) => (
            <pre
              key={query}
              className="mb-2 overflow-auto rounded bg-muted p-2 font-mono text-[11px] last:mb-0"
            >
              {query}
            </pre>
          ),
        )}
      </div>
    </details>
  );
}
