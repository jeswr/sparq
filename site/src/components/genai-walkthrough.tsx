"use client";

// [OPUS-4.8] sq-3was — the /surface/genai (GenAI / NLQ) walkthrough (tier-e).
// sparq-nlq is an opt-in NATIVE crate (it can pull a model behind a trait) and is NOT
// in the lean wasm bundle, and the static Pages site has no backend — so, per the
// feature-showcase design's honest tier-e fallback, this REPLAYS captured output
// rather than calling a model live:
//
//   1. SCHEMA CARD — the verbatim `to_text_summary(4000)` introspection deck the loop
//      grounds the model with (pure index introspection — no model). GENUINELY REAL.
//   2. NL → SPARQL LOOP — pick a question, step the loop (ground → generate → validate
//      → execute), and see the generated SPARQL plus the REAL executed result table.
//      One question takes the REPAIR path (a malformed first query, the parser error
//      fed back, a fixed second query).
//
// HONESTY (see src/lib/genai.ts): the RESULT ROWS, row counts, repair counts and the
// schema card are real captured engine output (verbatim oxrdf Term serialization). The
// natural-language → SPARQL *generation step* is a SCRIPTED fixture, NOT a live model
// call (IS_LIVE_LLM === false), so the query text is "a realistic, schema-grounded
// query a competent model would write", not "what an LLM said here". The page labels
// both halves. All data is in src/lib/genai.ts (framework-free, unit-tested).

import * as React from "react";
import {
  Sparkles,
  FileSearch,
  Wand2,
  CheckCircle2,
  Play,
  Database,
  AlertTriangle,
} from "lucide-react";

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
  DATASET,
  IS_LIVE_LLM,
  QUESTIONS,
  SCHEMA_SUMMARY,
  type Question,
} from "@/lib/genai";

const fmt = new Intl.NumberFormat("en-US");

/** A result cell: render `null` (unbound) explicitly, else verbatim. */
function Cell({ value }: { value: string | null }) {
  if (value === null) {
    return <span className="text-muted-foreground/60 italic">unbound</span>;
  }
  return <span className="break-all">{value}</span>;
}

function ResultTable({ q }: { q: Question }) {
  if (q.isAsk) {
    // ASK: zero variables, one unit row iff true.
    const isTrue = q.headRows.length > 0;
    return (
      <div className="rounded-lg border bg-muted/40 p-3 font-mono text-[12.5px]">
        ASK →{" "}
        <span className={cn("font-semibold", isTrue ? "text-[var(--success)]" : "text-[var(--warning)]")}>
          {isTrue ? "true" : "false"}
        </span>
        <span className="ml-1 text-muted-foreground">
          (unit-row encoding: {q.vars.length} variables, one empty row iff satisfiable)
        </span>
      </div>
    );
  }
  return (
    <div className="overflow-x-auto rounded-lg border">
      <table className="w-full border-collapse font-mono text-[12px]">
        <thead>
          <tr className="border-b bg-muted/60">
            {q.vars.map((v) => (
              <th key={v} className="px-3 py-1.5 text-left font-semibold">
                ?{v}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {q.headRows.map((row, i) => (
            <tr key={i} className="border-b last:border-0">
              {row.map((cell, j) => (
                <td key={j} className="px-3 py-1.5 align-top">
                  <Cell value={cell} />
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function StepBadge({
  icon: Icon,
  label,
  done,
}: {
  icon: typeof FileSearch;
  label: string;
  done: boolean;
}) {
  return (
    <div
      className={cn(
        "flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-medium ring-1 transition-colors",
        done
          ? "bg-[color-mix(in_oklch,var(--success)_12%,transparent)] text-[var(--success)] ring-[var(--success)]/30"
          : "bg-muted/50 text-muted-foreground ring-foreground/10",
      )}
    >
      <Icon className="size-3.5" aria-hidden="true" />
      {label}
    </div>
  );
}

export function GenaiWalkthrough() {
  const [selected, setSelected] = React.useState(0);
  // step = how far through the loop the user has stepped (0 … STEPS.length).
  const [step, setStep] = React.useState(0);

  const q = QUESTIONS[selected];
  const hasRepair = q.repairs > 0;
  // Loop stages revealed in order. The repair stage only appears for the repair case.
  const stages = React.useMemo(
    () => [
      { key: "ground", label: "Ground", icon: FileSearch },
      { key: "generate", label: "Generate", icon: Wand2 },
      { key: "validate", label: "Validate", icon: CheckCircle2 },
      ...(hasRepair ? [{ key: "repair", label: "Repair", icon: AlertTriangle }] : []),
      { key: "execute", label: "Execute", icon: Database },
    ],
    [hasRepair],
  );

  function pick(i: number) {
    setSelected(i);
    setStep(0);
  }
  const maxStep = stages.length;
  const done = step >= maxStep;

  return (
    <div className="space-y-10">
      {/* ── The schema card — genuinely real introspection grounding ──────────── */}
      <section className="space-y-3">
        <div className="flex items-center gap-2">
          <FileSearch className="size-5 text-primary" aria-hidden="true" />
          <h2 className="text-xl font-semibold">Schema card — what grounds the model</h2>
          <Badge variant="success" className="text-[10px] uppercase">
            real introspection
          </Badge>
        </div>
        <p className="measure text-sm text-muted-foreground">
          Before any model sees the question, sparq builds a token-budgeted{" "}
          <strong className="text-foreground">schema summary</strong> straight from the
          store&rsquo;s permutation indexes — classes &amp; predicates with{" "}
          <em>exact</em> counts, datatype ranges, characteristic predicate sets and a
          prefix glossary. This is the verbatim{" "}
          <code className="font-mono">to_text_summary(4000)</code> output over the{" "}
          {fmt.format(DATASET.triples)}-triple{" "}
          <a
            href={DATASET.source}
            target="_blank"
            rel="noopener noreferrer"
            className="underline underline-offset-2"
          >
            {DATASET.name}
          </a>{" "}
          dataset — no model, no guessing. The trailing{" "}
          <code className="font-mono">…</code> markers are its own budget truncation.
        </p>
        <pre className="max-h-80 overflow-auto rounded-lg border bg-muted/40 p-3 font-mono text-[11.5px] leading-relaxed">
          {SCHEMA_SUMMARY}
        </pre>
      </section>

      {/* ── The NL → SPARQL loop ───────────────────────────────────────────────── */}
      <section className="space-y-4">
        <div className="flex items-center gap-2">
          <Sparkles className="size-5 text-primary" aria-hidden="true" />
          <h2 className="text-xl font-semibold">Ask in English — replay the loop</h2>
        </div>
        <p className="measure text-sm text-muted-foreground">
          Pick a question, then <strong className="text-foreground">Step</strong> through
          the loop: <em>ground</em> with the schema card, <em>generate</em> a query,{" "}
          <em>validate</em> it with the real <code className="font-mono">spargebra</code>{" "}
          parser, and <em>execute</em> it under a query budget. The last question takes
          the <strong className="text-foreground">repair</strong> path — a malformed first
          query, the parser error fed back, a fixed second query.
        </p>

        {/* Question chips. */}
        <div className="flex flex-wrap gap-2">
          {QUESTIONS.map((item, i) => (
            <button
              key={item.question}
              type="button"
              onClick={() => pick(i)}
              className={cn(
                "rounded-full px-3 py-1.5 text-left text-[12.5px] ring-1 transition-colors",
                i === selected
                  ? "bg-primary/10 text-primary ring-primary/30"
                  : "bg-muted/40 text-muted-foreground ring-foreground/10 hover:bg-muted",
              )}
            >
              {item.question}
              {item.repairs > 0 && (
                <Badge variant="warning" className="ml-2 text-[10px] uppercase">
                  repair
                </Badge>
              )}
            </button>
          ))}
        </div>

        <Card>
          <CardHeader className="space-y-3">
            <div className="flex flex-wrap items-center justify-between gap-3">
              <CardTitle className="text-base">
                <span className="text-muted-foreground">Q:</span> {q.question}
              </CardTitle>
              <div className="flex gap-2">
                <Button size="sm" onClick={() => setStep((s) => Math.min(s + 1, maxStep))} disabled={done}>
                  <Play className="size-4" aria-hidden="true" />
                  {step === 0 ? "Step" : done ? "Done" : "Next step"}
                </Button>
                <Button size="sm" variant="outline" onClick={() => setStep(0)} disabled={step === 0}>
                  Reset
                </Button>
              </div>
            </div>
            <div className="flex flex-wrap gap-1.5">
              {stages.map((s, i) => (
                <StepBadge key={s.key} icon={s.icon} label={s.label} done={i < step} />
              ))}
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            {/* Ground */}
            {step >= 1 && (
              <div className="space-y-1.5">
                <div className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
                  <FileSearch className="size-3.5" aria-hidden="true" />
                  ground — the schema card above is prepended to the prompt
                </div>
              </div>
            )}

            {/* Generate */}
            {step >= 2 && (
              <div className="space-y-1.5">
                <div className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
                  <Wand2 className="size-3.5" aria-hidden="true" />
                  generate
                  <Badge variant="muted" className="text-[10px] uppercase">
                    scripted fixture · not a live model
                  </Badge>
                </div>
                <pre className="overflow-x-auto rounded-lg border bg-muted/40 p-3 font-mono text-[12.5px] leading-relaxed">
                  {hasRepair ? firstAttempt(q) : q.sparql}
                </pre>
              </div>
            )}

            {/* Validate (and repair) */}
            {step >= 3 && (
              <div className="space-y-1.5">
                <div className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
                  <CheckCircle2 className="size-3.5" aria-hidden="true" />
                  validate — spargebra parse
                </div>
                {hasRepair ? (
                  <div className="rounded-lg border border-[var(--warning)]/30 bg-[var(--warning)]/5 p-3 font-mono text-[12px] text-[color-mix(in_oklch,var(--warning)_80%,var(--foreground))]">
                    parse error → the failed query + the parser message are sent back to
                    the model (repair round 1)
                  </div>
                ) : (
                  <div className="rounded-lg border bg-muted/40 p-3 font-mono text-[12px] text-[var(--success)]">
                    parsed OK
                  </div>
                )}
              </div>
            )}

            {/* Repair (repair case only) — shows the fixed query. */}
            {hasRepair && step >= 4 && (
              <div className="space-y-1.5">
                <div className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
                  <AlertTriangle className="size-3.5" aria-hidden="true" />
                  repair — second completion (parses)
                  <Badge variant="muted" className="text-[10px] uppercase">
                    scripted fixture
                  </Badge>
                </div>
                <pre className="overflow-x-auto rounded-lg border bg-muted/40 p-3 font-mono text-[12.5px] leading-relaxed">
                  {q.sparql}
                </pre>
              </div>
            )}

            {/* Execute — REAL captured rows. */}
            {done && (
              <div className="space-y-1.5">
                <div className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
                  <Database className="size-3.5" aria-hidden="true" />
                  execute
                  <Badge variant="success" className="text-[10px] uppercase">
                    real engine output
                  </Badge>
                </div>
                <ResultTable q={q} />
                <p className="text-xs text-muted-foreground">
                  {q.isAsk ? (
                    <>Real result, executed by the sparq engine over the full dataset.</>
                  ) : (
                    <>
                      Showing the first {q.headRows.length} of{" "}
                      <strong className="text-foreground">{fmt.format(q.totalRows)}</strong>{" "}
                      row{q.totalRows === 1 ? "" : "s"} — real, executed by the sparq engine
                      over the full {fmt.format(DATASET.triples)}-triple dataset (term
                      serialization verbatim, datatypes &amp; language tags intact).
                    </>
                  )}{" "}
                  {!IS_LIVE_LLM && (
                    <em>
                      The query came from the recorded fixture, not a live model call.
                    </em>
                  )}
                </p>
              </div>
            )}
          </CardContent>
        </Card>
      </section>
    </div>
  );
}

// The repair case's deliberately-malformed first attempt (mirrors the fixture's first
// completion: the aggregate alias loses its closing paren). Shown only for the repair
// question so the "Generate → Validate → Repair" arc is legible; the fixed query is
// `q.sparql`. Kept here (not in the data module) because it is the loop's INPUT, not a
// captured result, and exists for exactly one question.
function firstAttempt(q: Question): string {
  return q.sparql
    .replace("SELECT ?sport (COUNT(?event) AS ?count)", "SELECT ?sport (COUNT(?event) AS ?count")
    .replace("\nORDER BY DESC(?count)", "");
}
