"use client";

// [OPUS-4.8] sq-dwdm — the /surface/vector (Vector / ANN) showcase (tier-e). sparq-vectors
// is an OPT-IN native crate (nothing in the workspace or the lean wasm bundle depends on
// it, and the `vec:` magic predicate sits behind the non-default `vec-predicate` feature),
// and the static Pages site has no backend — so, per the feature-showcase design's honest
// tier-e fallback, this REPLAYS captured output rather than building an index live:
//
//   1. EMBED & SEARCH BY TERM — the Usain Bolt example: embed entity labels with the
//      deterministic HashEmbedder, then exact cosine nearest-neighbour by term. REAL
//      captured output (the cosines are the engine's exact f32). Lexical, NOT semantic.
//   2. k-NN INSIDE SPARQL — pick a `vec:nearest` / `vec:search` query and see the REAL
//      executed result table over a tiny unit-circle store (verbatim term serialization).
//
// HONESTY (see src/lib/vector.ts): the result rows / cosines are real captured engine
// output (answer-exact backend, verbatim oxrdf Term serialization). The HashEmbedder is
// TEST-ONLY (lexical n-gram hashing, no semantics — IS_SEMANTIC_EMBEDDER === false), so the
// neighbours demonstrate the PIPELINE + the exact geometry, not embedding quality. ANN
// recall/latency are NOT captured here and are non-canonical. All data is in
// src/lib/vector.ts (framework-free, unit-tested).

import * as React from "react";
import {
  Binary,
  Crosshair,
  Database,
  FileSearch,
  Play,
  Search,
  Tag,
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
  BOLT,
  IS_SEMANTIC_EMBEDDER,
  RECALL_NOTES,
  UNIT_CIRCLE,
  VEC_QUERIES,
  type VecQuery,
} from "@/lib/vector";

/** A short label for an `<iri>` cell — strip the namespace for legibility. */
function shortIri(cell: string): string {
  const m = cell.match(/^<.*[/#]([^/#>]+)>$/);
  return m ? m[1] : cell;
}

function ResultTable({ q }: { q: VecQuery }) {
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
          {q.rows.map((row, i) => (
            <tr key={i} className="border-b last:border-0">
              {row.map((cell, j) => (
                <td key={j} className="px-3 py-1.5 align-top break-all">
                  {cell}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export function VectorWalkthrough() {
  const [selected, setSelected] = React.useState(0);
  const [ran, setRan] = React.useState(false);
  const q = VEC_QUERIES[selected];

  function pick(i: number) {
    setSelected(i);
    setRan(false);
  }

  return (
    <div className="space-y-10">
      {/* ── 1. Embed labels + exact nearest-neighbour by term (the Bolt example) ──── */}
      <section className="space-y-3">
        <div className="flex items-center gap-2">
          <Tag className="size-5 text-primary" aria-hidden="true" />
          <h2 className="text-xl font-semibold">
            Embed labels, then search by term
          </h2>
          <Badge variant="success" className="text-[10px] uppercase">
            real captured output
          </Badge>
        </div>
        <p className="measure text-sm text-muted-foreground">
          <code className="font-mono">embed_labels</code> stores one f32 vector per labeled
          entity (keyed by its dictionary id), then{" "}
          <code className="font-mono">nearest_term_exact</code> returns the cosine-closest
          neighbours of a term — the seed excluded, best first. This run embeds five athlete
          labels and queries the neighbours of{" "}
          <strong className="text-foreground">Usain Bolt</strong>. The vectors come from the{" "}
          <strong className="text-foreground">deterministic, test-only</strong>{" "}
          <code className="font-mono">HashEmbedder</code> (lexical n-gram hashing), so the
          nearest neighbour is{" "}
          <em>&ldquo;Usain Bolt Junior&rdquo;</em> because the strings overlap — this shows
          the pipeline and the exact geometry, <strong className="text-foreground">not</strong>{" "}
          semantic quality (a real deployment supplies its own embedder).
        </p>

        <div className="grid gap-4 md:grid-cols-2">
          {/* The fixture. */}
          <Card>
            <CardHeader className="flex-row items-center gap-2 space-y-0">
              <FileSearch className="size-4 text-muted-foreground" aria-hidden="true" />
              <CardTitle className="text-sm">
                Fixture — {BOLT.embedded} labeled entities embedded
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-1.5 font-mono text-[12px]">
              {BOLT.fixture.map((e) => (
                <div key={e.iri} className="flex items-center gap-2">
                  <Binary className="size-3 shrink-0 text-primary/60" aria-hidden="true" />
                  <span className="text-muted-foreground">{shortIri(`<${e.iri}>`)}</span>
                  <span className="break-all">&ldquo;{e.label}&rdquo;</span>
                </div>
              ))}
            </CardContent>
          </Card>

          {/* The captured neighbours. */}
          <Card>
            <CardHeader className="flex-row items-center gap-2 space-y-0">
              <Crosshair className="size-4 text-primary" aria-hidden="true" />
              <CardTitle className="text-sm">
                nearest_term_exact( {shortIri(BOLT.seed)}, 4 )
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="overflow-x-auto rounded-lg border">
                <table className="w-full border-collapse font-mono text-[12px]">
                  <thead>
                    <tr className="border-b bg-muted/60">
                      <th className="px-3 py-1.5 text-left font-semibold">neighbour</th>
                      <th className="px-3 py-1.5 text-right font-semibold">cosine</th>
                    </tr>
                  </thead>
                  <tbody>
                    {BOLT.neighbours.map((n) => (
                      <tr key={n.term} className="border-b last:border-0">
                        <td className="px-3 py-1.5">{shortIri(n.term)}</td>
                        <td
                          className={cn(
                            "px-3 py-1.5 text-right tabular-nums",
                            n.cosine >= 0
                              ? "text-[var(--success)]"
                              : "text-muted-foreground",
                          )}
                        >
                          {n.cosine.toFixed(4)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
              <p className="mt-2 text-xs text-muted-foreground">
                The seed {shortIri(BOLT.seed)} is excluded. Cosines are the engine&rsquo;s
                exact f32, captured verbatim (shown to 4 dp).
              </p>
            </CardContent>
          </Card>
        </div>
      </section>

      {/* ── 2. k-NN inside SPARQL — the vec: magic predicate ──────────────────────── */}
      <section className="space-y-4">
        <div className="flex items-center gap-2">
          <Search className="size-5 text-primary" aria-hidden="true" />
          <h2 className="text-xl font-semibold">k-NN inside SPARQL — the vec: predicate</h2>
          <Badge variant="success" className="text-[10px] uppercase">
            real captured output
          </Badge>
        </div>
        <p className="measure text-sm text-muted-foreground">
          With the opt-in <code className="font-mono">vec-predicate</code> feature, vector
          k-NN is expressed in <strong className="text-foreground">plain SPARQL</strong> via
          the <code className="font-mono">vec:nearest</code> /{" "}
          <code className="font-mono">vec:search</code> magic predicates — a
          spargebra-algebra rewrite that inlines the hits as a{" "}
          <code className="font-mono">VALUES</code> table, so the engine, planner and wasm
          bundle are unchanged. These run over a tiny{" "}
          <strong className="text-foreground">unit-circle store</strong> (5 entities, dim 2)
          so the geometry is legible:
        </p>

        {/* The unit-circle store. */}
        <div className="flex flex-wrap gap-2 font-mono text-[12px]">
          {UNIT_CIRCLE.map((e) => (
            <span
              key={e.iri}
              className="inline-flex items-center gap-1.5 rounded-full bg-muted/40 px-2.5 py-1 ring-1 ring-foreground/10"
            >
              <span className="text-primary">{shortIri(`<${e.iri}>`)}</span>
              <span className="text-muted-foreground">
                [{e.vec[0]}, {e.vec[1]}]
              </span>
            </span>
          ))}
        </div>

        {/* Query chips. */}
        <div className="flex flex-wrap gap-2">
          {VEC_QUERIES.map((item, i) => (
            <button
              key={item.id}
              type="button"
              onClick={() => pick(i)}
              className={cn(
                "rounded-full px-3 py-1.5 text-left text-[12.5px] ring-1 transition-colors",
                i === selected
                  ? "bg-primary/10 text-primary ring-primary/30"
                  : "bg-muted/40 text-muted-foreground ring-foreground/10 hover:bg-muted",
              )}
            >
              {item.caption}
            </button>
          ))}
        </div>

        <Card>
          <CardHeader className="space-y-3">
            <div className="flex flex-wrap items-center justify-between gap-3">
              <CardTitle className="text-base">{q.caption}</CardTitle>
              <Button size="sm" onClick={() => setRan(true)} disabled={ran}>
                <Play className="size-4" aria-hidden="true" />
                {ran ? "Executed" : "Run"}
              </Button>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            <pre className="overflow-x-auto rounded-lg border bg-muted/40 p-3 font-mono text-[12.5px] leading-relaxed">
              {q.sparql}
            </pre>
            {ran && (
              <div className="space-y-1.5">
                <div className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
                  <Database className="size-3.5" aria-hidden="true" />
                  result
                  <Badge variant="success" className="text-[10px] uppercase">
                    real engine output
                  </Badge>
                </div>
                <ResultTable q={q} />
                <p className="text-xs text-muted-foreground">
                  {q.rows.length} row{q.rows.length === 1 ? "" : "s"}, executed by the sparq
                  engine over the unit-circle store with the answer-exact scan — term
                  serialization verbatim (datatypes intact).
                </p>
              </div>
            )}
          </CardContent>
        </Card>
      </section>

      {/* ── Exact vs approximate — honest, non-canonical ──────────────────────────── */}
      <section className="space-y-3">
        <div className="flex items-center gap-2">
          <Binary className="size-5 text-primary" aria-hidden="true" />
          <h2 className="text-xl font-semibold">Exact vs approximate — the trade-off</h2>
          <Badge variant="muted" className="text-[10px] uppercase">
            non-canonical
          </Badge>
        </div>
        <p className="measure text-sm text-muted-foreground">
          The captures above use the <strong className="text-foreground">exact</strong>{" "}
          brute-force scan (answer-exact, the right default below ~10
          <sup>5</sup> vectors). At scale you trade some recall for speed with an approximate
          index. Approximate search is <strong className="text-foreground">approximate</strong>{" "}
          &mdash; recall <code className="font-mono">&lt; 1.0</code>. The figures below are{" "}
          <strong className="text-foreground">the crate&rsquo;s own test gates, not
          canonical</strong> and not measured here &mdash; run the tests for the numbers; we
          show no latency (hardware-dependent).
        </p>
        <div className="grid gap-3 sm:grid-cols-3">
          {RECALL_NOTES.map((r) => (
            <div
              key={r.backend}
              className="rounded-xl bg-muted/40 p-4 ring-1 ring-foreground/10"
            >
              <div className="text-sm font-semibold">{r.backend}</div>
              <div className="mt-1 text-xs text-muted-foreground">{r.note}</div>
              <div className="mt-2 font-mono text-[11px] text-muted-foreground/80">
                {r.test}
              </div>
            </div>
          ))}
        </div>
        {!IS_SEMANTIC_EMBEDDER && (
          <p className="text-xs text-muted-foreground">
            Note: the captured embeddings come from the test-only{" "}
            <code className="font-mono">HashEmbedder</code> (lexical, no semantics). They
            demonstrate the store / search / SPARQL-rewrite pipeline and the exact geometry,
            not the retrieval quality of a real embedding model.
          </p>
        )}
      </section>
    </div>
  );
}
