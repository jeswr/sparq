"use client";

// [OPUS-5] sq-ixc3.24 — the visual query builder's INSPECTOR column: the selected node's / edge's
// editor, the shape-aware predicate picker, and the result-shape controls (DISTINCT / GROUP BY /
// aggregates / ORDER BY / LIMIT). Split out of query-builder-tool.tsx to keep the panel reviewable.
//
// Pure presentation over the `BuilderModel` — it edits the diagram through the callbacks it is
// given and never generates SPARQL itself (that is `@/lib/query-builder`).

import * as React from "react";
import { Plus, Trash2, X } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  iriLabel,
  negatedNodeIds,
  suggestionsFor,
  type AggregateFn,
  type BuilderAggregate,
  type BuilderAttribute,
  type BuilderEdge,
  type BuilderModel,
  type BuilderNode,
  type EdgeMode,
  type FilterOp,
  type PredicateSuggestion,
  type SchemaIndex,
  type ValueKind,
} from "@/lib/query-builder";

const FIELD = "w-full rounded-sm border bg-background px-1.5 py-1 text-xs";
const LABEL = "block text-[11px] font-medium text-muted-foreground";

/**
 * The predicate picker. Suggestions carry their PROVENANCE — `shape` (declared by a SHACL shape
 * in the store) or `data` (observed on instances of this class) — because they are different
 * kinds of claim. Free text is always allowed: the picker narrows, it never restricts.
 */
function PredicatePicker({
  suggestions,
  onPick,
  label,
  testId,
}: {
  suggestions: PredicateSuggestion[];
  onPick: (predicate: string) => void;
  label: string;
  testId?: string;
}) {
  const [custom, setCustom] = React.useState("");
  return (
    <div className="space-y-1">
      <span className={LABEL}>{label}</span>
      {suggestions.length === 0 ? (
        <p className="text-[11px] text-muted-foreground">
          No suggestions — introspect a non-empty store, or type a predicate IRI below.
        </p>
      ) : (
        <ul className="max-h-40 space-y-0.5 overflow-y-auto rounded-sm border p-1" data-builder-suggestions={testId}>
          {suggestions.map((s) => (
            <li key={`${s.source}-${s.predicate}`}>
              <button
                type="button"
                onClick={() => onPick(s.predicate)}
                className="flex w-full items-center gap-1 rounded-sm px-1 py-0.5 text-left hover:bg-muted"
                title={s.predicate}
              >
                <span className="min-w-0 flex-1 truncate font-mono text-[11px]">
                  {iriLabel(s.predicate)}
                </span>
                <Badge
                  variant="outline"
                  className="h-4 shrink-0 px-1 text-[9px]"
                  title={
                    s.source === "shape"
                      ? "Declared by a SHACL shape in the store"
                      : "Observed on instances in the loaded store"
                  }
                >
                  {s.source}
                </Badge>
                {s.uses !== null && (
                  <span className="shrink-0 text-[9px] text-muted-foreground">{s.uses}</span>
                )}
                <span className="w-8 shrink-0 text-right text-[9px] text-muted-foreground">
                  {s.objectKind === "unknown" ? "" : s.objectKind}
                </span>
              </button>
            </li>
          ))}
        </ul>
      )}
      <div className="flex gap-1">
        <input
          className={FIELD}
          value={custom}
          placeholder="or a predicate IRI / prefixed name"
          onChange={(e) => setCustom(e.target.value)}
          aria-label={`${label} — custom IRI`}
          data-builder-custom-predicate={testId}
        />
        <Button
          size="sm"
          variant="outline"
          disabled={custom.trim() === ""}
          onClick={() => {
            onPick(custom.trim());
            setCustom("");
          }}
        >
          Add
        </Button>
      </div>
    </div>
  );
}

export function NodeInspector({
  node,
  index,
  negated,
  onChange,
  onChangeAttribute,
  onAddAttribute,
  onDropAttribute,
}: {
  node: BuilderNode;
  index: SchemaIndex | null;
  negated: boolean;
  onChange: (fn: (n: BuilderNode) => BuilderNode) => void;
  onChangeAttribute: (attrId: string, fn: (a: BuilderAttribute) => BuilderAttribute) => void;
  onAddAttribute: (predicate: string) => void;
  onDropAttribute: (attrId: string) => void;
}) {
  const suggestions = index ? suggestionsFor(index, node.classIri) : [];
  return (
    <div className="space-y-3 border-b p-3" data-builder-node-inspector={node.variable}>
      <div className="space-y-1">
        <label className={LABEL} htmlFor="builder-node-var">
          Variable
        </label>
        <input
          id="builder-node-var"
          className={`${FIELD} font-mono`}
          value={node.variable}
          onChange={(e) => onChange((n) => ({ ...n, variable: e.target.value }))}
        />
      </div>

      <div className="space-y-1">
        <label className={LABEL} htmlFor="builder-node-class">
          Class (rdf:type)
        </label>
        <select
          id="builder-node-class"
          className={FIELD}
          value={node.classIri ?? ""}
          onChange={(e) => onChange((n) => ({ ...n, classIri: e.target.value || null }))}
        >
          <option value="">any type</option>
          {(index?.classes ?? []).map((c) => (
            <option key={c.classIri} value={c.classIri}>
              {iriLabel(c.classIri)}
              {c.instances !== null ? ` (${c.instances})` : " (shape-declared)"}
            </option>
          ))}
          {node.classIri && !(index?.classes ?? []).some((c) => c.classIri === node.classIri) && (
            <option value={node.classIri}>{iriLabel(node.classIri)}</option>
          )}
        </select>
        <input
          className={FIELD}
          placeholder="or a class IRI"
          defaultValue=""
          onBlur={(e) => {
            const v = e.target.value.trim();
            if (v) {
              onChange((n) => ({ ...n, classIri: v }));
              e.target.value = "";
            }
          }}
          aria-label="Class IRI"
        />
      </div>

      <label className="flex items-center gap-1.5 text-xs">
        <input
          type="checkbox"
          checked={node.projected}
          disabled={negated}
          onChange={(e) => onChange((n) => ({ ...n, projected: e.target.checked }))}
          data-builder-node-projected
        />
        Select <code className="font-mono">?{node.variable}</code>
        {negated && <span className="text-[10px] text-[var(--warning)]">(not visible outside NOT EXISTS)</span>}
      </label>

      <div className="space-y-2">
        <span className={LABEL}>Attributes</span>
        {node.attributes.length === 0 && (
          <p className="text-[11px] text-muted-foreground">None yet.</p>
        )}
        {node.attributes.map((a) => (
          <AttributeEditor
            key={a.id}
            attribute={a}
            negated={negated}
            onChange={(fn) => onChangeAttribute(a.id, fn)}
            onDrop={() => onDropAttribute(a.id)}
          />
        ))}
        <PredicatePicker
          suggestions={suggestions}
          onPick={onAddAttribute}
          label="Add an attribute"
          testId="attribute"
        />
      </div>
    </div>
  );
}

const FILTER_OPS: { value: FilterOp; label: string }[] = [
  { value: "=", label: "=" },
  { value: "!=", label: "≠" },
  { value: "<", label: "<" },
  { value: "<=", label: "≤" },
  { value: ">", label: ">" },
  { value: ">=", label: "≥" },
  { value: "contains", label: "contains" },
  { value: "starts", label: "starts with" },
  { value: "ends", label: "ends with" },
  { value: "regex", label: "matches regex" },
];

const VALUE_KINDS: ValueKind[] = ["string", "number", "boolean", "date", "iri"];

function AttributeEditor({
  attribute,
  negated,
  onChange,
  onDrop,
}: {
  attribute: BuilderAttribute;
  /** The owning node is bound only inside a NOT EXISTS branch — so this variable is too. */
  negated: boolean;
  onChange: (fn: (a: BuilderAttribute) => BuilderAttribute) => void;
  onDrop: () => void;
}) {
  const f = attribute.filter;
  return (
    <div className="space-y-1 rounded-sm border p-1.5" data-builder-attribute={attribute.variable}>
      <div className="flex items-center gap-1">
        <span className="min-w-0 flex-1 truncate font-mono text-[11px]" title={attribute.predicate}>
          {iriLabel(attribute.predicate)}
        </span>
        <button
          type="button"
          onClick={onDrop}
          aria-label={`Remove attribute ?${attribute.variable}`}
          className="rounded-sm p-0.5 text-muted-foreground hover:text-destructive"
        >
          <X className="size-3" />
        </button>
      </div>
      <input
        className={`${FIELD} font-mono`}
        value={attribute.variable}
        onChange={(e) => onChange((a) => ({ ...a, variable: e.target.value }))}
        aria-label="Attribute variable"
      />
      <div className="flex flex-wrap gap-2 text-[11px]">
        <label className="flex items-center gap-1">
          <input
            type="checkbox"
            checked={attribute.projected}
            disabled={negated}
            onChange={(e) => onChange((a) => ({ ...a, projected: e.target.checked }))}
          />
          select
          {negated && (
            <span className="text-[10px] text-[var(--warning)]">(not visible outside NOT EXISTS)</span>
          )}
        </label>
        <label className="flex items-center gap-1">
          <input
            type="checkbox"
            checked={attribute.optional}
            onChange={(e) => onChange((a) => ({ ...a, optional: e.target.checked }))}
          />
          optional
        </label>
        <label className="flex items-center gap-1">
          <input
            type="checkbox"
            checked={f !== null}
            onChange={(e) =>
              onChange((a) => ({
                ...a,
                filter: e.target.checked ? { op: "=", value: "", kind: "string" } : null,
              }))
            }
            data-builder-attribute-filter
          />
          filter
        </label>
      </div>
      {f && (
        <div className="flex flex-wrap gap-1">
          <select
            className={`${FIELD} w-24`}
            value={f.op}
            onChange={(e) => onChange((a) => ({ ...a, filter: { ...f, op: e.target.value as FilterOp } }))}
            aria-label="Filter operator"
          >
            {FILTER_OPS.map((o) => (
              <option key={o.value} value={o.value}>
                {o.label}
              </option>
            ))}
          </select>
          <select
            className={`${FIELD} w-20`}
            value={f.kind}
            onChange={(e) => onChange((a) => ({ ...a, filter: { ...f, kind: e.target.value as ValueKind } }))}
            aria-label="Filter value kind"
          >
            {VALUE_KINDS.map((k) => (
              <option key={k} value={k}>
                {k}
              </option>
            ))}
          </select>
          <input
            className={FIELD}
            value={f.value}
            placeholder="value"
            onChange={(e) => onChange((a) => ({ ...a, filter: { ...f, value: e.target.value } }))}
            aria-label="Filter value"
            data-builder-filter-value
          />
          {f.op === "regex" && (
            <label className="flex items-center gap-1 text-[11px]">
              <input
                type="checkbox"
                checked={f.caseInsensitive ?? false}
                onChange={(e) =>
                  onChange((a) => ({ ...a, filter: { ...f, caseInsensitive: e.target.checked } }))
                }
              />
              ignore case
            </label>
          )}
        </div>
      )}
    </div>
  );
}

const EDGE_MODES: { value: EdgeMode; label: string; hint: string }[] = [
  { value: "required", label: "required", hint: "A plain triple pattern." },
  {
    value: "optional",
    label: "optional",
    hint: "OPTIONAL { … } — the match may be absent. The target node's class and attributes go inside the OPTIONAL too, so while it carries any it must be a leaf.",
  },
  {
    value: "not-exists",
    label: "not exists",
    hint: "FILTER NOT EXISTS { … } — the AND-NOT branch. Its target node must be a leaf, and its variables are not projectable.",
  },
];

export function EdgeInspector({
  edge,
  model,
  index,
  onChange,
  onDelete,
}: {
  edge: BuilderEdge;
  model: BuilderModel;
  index: SchemaIndex | null;
  onChange: (fn: (e: BuilderEdge) => BuilderEdge) => void;
  onDelete: () => void;
}) {
  const from = model.nodes.find((n) => n.id === edge.from);
  const suggestions = index ? suggestionsFor(index, from?.classIri ?? null) : [];
  const mode = EDGE_MODES.find((m) => m.value === edge.mode);
  return (
    <div className="space-y-3 border-b p-3" data-builder-edge-inspector>
      <div className="flex items-center gap-1">
        <span className="min-w-0 flex-1 truncate font-mono text-xs">
          ?{from?.variable ?? "?"} → ?{model.nodes.find((n) => n.id === edge.to)?.variable ?? "?"}
        </span>
        <button
          type="button"
          onClick={onDelete}
          aria-label="Remove this edge"
          className="rounded-sm p-0.5 text-muted-foreground hover:text-destructive"
        >
          <Trash2 className="size-3.5" />
        </button>
      </div>

      <div className="space-y-1">
        <span className={LABEL}>Predicate</span>
        <input
          className={`${FIELD} font-mono`}
          value={edge.predicate}
          placeholder="predicate IRI"
          onChange={(e) => onChange((x) => ({ ...x, predicate: e.target.value }))}
          aria-label="Edge predicate"
          data-builder-edge-predicate
        />
      </div>

      <PredicatePicker
        suggestions={suggestions}
        onPick={(p) => onChange((x) => ({ ...x, predicate: p }))}
        label="Pick from the schema"
        testId="edge"
      />

      <div className="space-y-1">
        <span className={LABEL}>Mode</span>
        <select
          className={FIELD}
          value={edge.mode}
          onChange={(e) => onChange((x) => ({ ...x, mode: e.target.value as EdgeMode }))}
          aria-label="Edge mode"
          data-builder-edge-mode
        >
          {EDGE_MODES.map((m) => (
            <option key={m.value} value={m.value}>
              {m.label}
            </option>
          ))}
        </select>
        <p className="text-[10px] text-muted-foreground">{mode?.hint}</p>
      </div>

      <div className="space-y-1">
        <span className={LABEL}>Alternative predicates (UNION)</span>
        {edge.alternatives.map((alt, i) => (
          <div key={`${edge.id}-alt-${i}`} className="flex gap-1">
            <input
              className={`${FIELD} font-mono`}
              value={alt}
              onChange={(e) =>
                onChange((x) => ({
                  ...x,
                  alternatives: x.alternatives.map((a, j) => (j === i ? e.target.value : a)),
                }))
              }
              aria-label={`Alternative predicate ${i + 1}`}
            />
            <button
              type="button"
              onClick={() =>
                onChange((x) => ({ ...x, alternatives: x.alternatives.filter((_, j) => j !== i) }))
              }
              aria-label={`Remove alternative predicate ${i + 1}`}
              className="rounded-sm p-0.5 text-muted-foreground hover:text-destructive"
            >
              <X className="size-3" />
            </button>
          </div>
        ))}
        <Button
          size="sm"
          variant="outline"
          disabled={edge.mode !== "required"}
          onClick={() => onChange((x) => ({ ...x, alternatives: [...x.alternatives, ""] }))}
          title={
            edge.mode === "required"
              ? "Match any one of these predicates (emits a UNION)"
              : "Alternatives are only available on a required edge"
          }
        >
          <Plus className="size-3" /> alternative
        </Button>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Result shape: DISTINCT / GROUP BY / aggregates / ORDER BY / LIMIT.
// ---------------------------------------------------------------------------

const AGGREGATE_FNS: AggregateFn[] = ["COUNT", "SUM", "AVG", "MIN", "MAX", "SAMPLE", "GROUP_CONCAT"];

export function ResultShapePanel({
  model,
  onChange,
  onAddAggregate,
}: {
  model: BuilderModel;
  onChange: React.Dispatch<React.SetStateAction<BuilderModel>>;
  /** Append a fresh aggregate. The panel owns id minting, not this presentational view. */
  onAddAggregate: () => void;
}) {
  const groupable = React.useMemo(() => {
    const negated = negatedNodeIds(model);
    const out: string[] = [];
    for (const n of model.nodes) {
      if (negated.has(n.id)) continue;
      out.push(n.variable);
      for (const a of n.attributes) out.push(a.variable);
    }
    return out;
  }, [model]);
  const orderable = [...groupable, ...model.aggregates.map((a) => a.alias)];

  return (
    <div className="space-y-3 p-3" data-builder-result-shape>
      <span className={LABEL}>Result shape</span>

      <div className="flex items-center gap-3 text-xs">
        <label className="flex items-center gap-1.5">
          <input
            type="checkbox"
            checked={model.distinct}
            onChange={(e) => onChange((m) => ({ ...m, distinct: e.target.checked }))}
            data-builder-distinct
          />
          DISTINCT
        </label>
        <label className="flex items-center gap-1.5">
          LIMIT
          <input
            className={`${FIELD} w-20`}
            type="number"
            min={1}
            value={model.limit ?? ""}
            placeholder="none"
            onChange={(e) =>
              onChange((m) => ({ ...m, limit: e.target.value === "" ? null : Number(e.target.value) }))
            }
            aria-label="LIMIT"
          />
        </label>
      </div>

      <div className="space-y-1">
        <span className={LABEL}>GROUP BY</span>
        {groupable.length === 0 && <p className="text-[11px] text-muted-foreground">No variables yet.</p>}
        <div className="flex flex-wrap gap-1">
          {groupable.map((v) => {
            const on = model.groupBy.includes(v);
            return (
              <button
                key={v}
                type="button"
                onClick={() =>
                  onChange((m) => ({
                    ...m,
                    groupBy: on ? m.groupBy.filter((g) => g !== v) : [...m.groupBy, v],
                  }))
                }
                className={`rounded-full border px-2 py-0.5 font-mono text-[10px] ${on ? "border-primary text-primary" : "text-muted-foreground"}`}
                data-builder-group-by={v}
              >
                ?{v}
              </button>
            );
          })}
        </div>
      </div>

      <div className="space-y-1">
        <span className={LABEL}>Aggregates</span>
        {model.aggregates.map((agg) => (
          <div key={agg.id} className="flex flex-wrap gap-1" data-builder-aggregate={agg.alias}>
            <select
              className={`${FIELD} w-28`}
              value={agg.fn}
              onChange={(e) =>
                onChange((m) => updateAggregate(m, agg.id, (a) => ({ ...a, fn: e.target.value as AggregateFn })))
              }
              aria-label="Aggregate function"
            >
              {AGGREGATE_FNS.map((fn) => (
                <option key={fn} value={fn}>
                  {fn}
                </option>
              ))}
            </select>
            <select
              className={`${FIELD} w-24`}
              value={agg.target}
              onChange={(e) =>
                onChange((m) => updateAggregate(m, agg.id, (a) => ({ ...a, target: e.target.value })))
              }
              aria-label="Aggregated variable"
            >
              <option value="*">*</option>
              {groupable.map((v) => (
                <option key={v} value={v}>
                  ?{v}
                </option>
              ))}
            </select>
            <input
              className={`${FIELD} w-24 font-mono`}
              value={agg.alias}
              onChange={(e) =>
                onChange((m) => updateAggregate(m, agg.id, (a) => ({ ...a, alias: e.target.value })))
              }
              aria-label="Aggregate result variable"
            />
            <label className="flex items-center gap-1 text-[11px]">
              <input
                type="checkbox"
                checked={agg.distinct}
                onChange={(e) =>
                  onChange((m) => updateAggregate(m, agg.id, (a) => ({ ...a, distinct: e.target.checked })))
                }
              />
              distinct
            </label>
            <button
              type="button"
              onClick={() => onChange((m) => ({ ...m, aggregates: m.aggregates.filter((a) => a.id !== agg.id) }))}
              aria-label={`Remove aggregate ?${agg.alias}`}
              className="rounded-sm p-0.5 text-muted-foreground hover:text-destructive"
            >
              <X className="size-3" />
            </button>
          </div>
        ))}
        <Button
          size="sm"
          variant="outline"
          onClick={onAddAggregate}
          data-builder-add-aggregate
        >
          <Plus className="size-3" /> aggregate
        </Button>
      </div>

      <div className="space-y-1">
        <span className={LABEL}>ORDER BY</span>
        <div className="flex flex-wrap gap-1">
          {orderable.map((v) => {
            const entry = model.orderBy.find((o) => o.variable === v);
            return (
              <button
                key={v}
                type="button"
                onClick={() =>
                  onChange((m) => ({
                    ...m,
                    orderBy: !entry
                      ? [...m.orderBy, { variable: v, desc: true }]
                      : entry.desc
                        ? m.orderBy.map((o) => (o.variable === v ? { ...o, desc: false } : o))
                        : m.orderBy.filter((o) => o.variable !== v),
                  }))
                }
                className={`rounded-full border px-2 py-0.5 font-mono text-[10px] ${entry ? "border-primary text-primary" : "text-muted-foreground"}`}
                title="Click to cycle: descending → ascending → off"
                data-builder-order-by={v}
              >
                ?{v}
                {entry ? (entry.desc ? " ↓" : " ↑") : ""}
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
}

function updateAggregate(
  model: BuilderModel,
  id: string,
  fn: (a: BuilderAggregate) => BuilderAggregate,
): BuilderModel {
  return { ...model, aggregates: model.aggregates.map((a) => (a.id === id ? fn(a) : a)) };
}
