// [FABLE-5] sq-549m0 — /genai-retrieval: the honest GenAI-retrieval capability ledger.
//
// One static page documenting the RDF-native GenAI-retrieval composition — vector +
// text retrieval inside SPARQL, embedding provenance, the NL→SPARQL loop, and
// graph-derived citations — split EXPLICITLY into what is implemented today (every
// row maps to an extant crate + opt-in cargo feature) vs what is designed-only /
// proposed (every row cites its governing bead). The killer artifact is the status
// ledger itself; there is no marketing framing above it.
//
// HONESTY (the sq-549m0 invariant):
//   * every "implemented" row names the real crate/feature that ships it;
//   * every "proposed" row carries its bead id and claims nothing shipped;
//   * NO performance numbers — comparative evidence on this axis is itself open
//     work (see the "what this page does not claim" strip);
//   * no ZK/MPC claims are made here at all — privacy composition is out of scope
//     for this page (see /capabilities for the caveated privacy lane).
//
// Nav: Cmd-K only (a TOP_PAGES entry in command-palette.tsx), like /assurance and
// /dogfooding — the slim top bar stays at 6 destinations.

import type { Metadata } from "next";
import Link from "next/link";
import {
  AlertTriangle,
  ArrowUpRight,
  CheckCircle2,
  CircleDashed,
  Sparkles,
} from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

export const metadata: Metadata = {
  title: "GenAI retrieval — implemented vs proposed",
  description:
    "The honest status ledger for sparq's RDF-native GenAI-retrieval composition: ANN-in-SPARQL, filtered ANN, embedding provenance, NL→SPARQL, and graph-derived citations — what is implemented today (crate + feature) vs designed-only (bead id).",
};

const REPO = "https://github.com/sparq-org/sparq";

/** An external link to a repo artifact, with the small up-right glyph. */
function Artifact({ href, label }: { href: string; label: string }) {
  return (
    <a
      href={href}
      target="_blank"
      rel="noopener noreferrer"
      className="inline-flex items-center gap-1 text-sm text-primary hover:underline"
    >
      {label}
      <ArrowUpRight className="size-3.5 opacity-70" aria-hidden />
    </a>
  );
}

/** A code token used inline in prose and tables. */
function Code({ children }: { children: React.ReactNode }) {
  return (
    <code className="rounded bg-muted px-1 py-0.5 font-mono text-xs text-foreground">
      {children}
    </code>
  );
}

// ── Implemented today ─────────────────────────────────────────────────────────
// Every row maps to an extant crate + the opt-in cargo feature that gates it (all
// OFF by default — the core engine and the lean wasm bundle carry none of this).
const IMPLEMENTED: {
  capability: string;
  where: React.ReactNode;
  what: React.ReactNode;
}[] = [
  {
    capability: "k-NN inside SPARQL (vec:nearest / vec:search)",
    where: (
      <>
        <Code>sparq-vectors</Code> · <Code>vec-predicate</Code>
      </>
    ),
    what: (
      <>
        Magic-predicate rewrite: <Code>?node vec:nearest ( &lt;query&gt; k )</Code>{" "}
        and the scored <Code>( ?node ?score ) vec:search ( … )</Code> form run
        nearest-neighbour search as part of query evaluation. Exact brute-force and
        on-disk Vamana backends; an HNSW index behind the further opt-in{" "}
        <Code>approx-ann</Code> feature.
      </>
    ),
  },
  {
    capability: "Filtered (predicate-constrained) ANN",
    where: (
      <>
        <Code>sparq-vectors</Code> · <Code>filtered-ann</Code>
      </>
    ),
    what: (
      <>
        Nearest-neighbour search constrained to graph-derived candidate sets
        (<Code>nearest_exact_filtered</Code>, <Code>FilterConfig</Code>,{" "}
        <Code>IdMask</Code>), with a pre-filter vs post-filter / over-fetch cost
        model deciding the strategy.
      </>
    ),
  },
  {
    capability: ".spqv v3 embedding provenance + compatibility rejection",
    where: (
      <>
        <Code>sparq-vectors</Code> · <Code>spqv-provenance</Code>
      </>
    ),
    what: (
      <>
        The <Code>.spqv</Code> v3 store persists an embedding-provenance record
        (model, version, dimension, metric, normalization) and{" "}
        <strong className="text-foreground">rejects incompatible stores</strong>{" "}
        instead of silently mixing embedding spaces; the <Code>spqvp:</Code>{" "}
        vocabulary asserts the same pipeline facts in RDF.
      </>
    ),
  },
  {
    capability: "Natural-language → SPARQL loop",
    where: (
      <>
        <Code>sparq-nlq</Code> · <Code>live</Code> / <Code>nlq-endpoint</Code>
      </>
    ),
    what: (
      <>
        Schema-card / VoID introspection grounds the prompt; generated SPARQL is
        executed by the real engine, so answers come from the graph, not the model.
        CI and the eval harness run entirely on recorded fixtures; a live model is
        opt-in (Anthropic via <Code>live</Code>, any OpenAI-compatible endpoint via{" "}
        <Code>nlq-endpoint</Code>).
      </>
    ),
  },
  {
    capability: "Provenance citations (emitted, never generated)",
    where: (
      <>
        <Code>sparq-nlq</Code> · <Code>citations</Code>
      </>
    ),
    what: (
      <>
        Citations are read from <Code>prov:wasDerivedFrom</Code> edges the graph
        actually asserts — every emitted citation resolves to a real in-graph
        source by construction, and a subject without provenance simply gets no
        citation. No post-hoc retrieval guessing.
      </>
    ),
  },
  {
    capability: "Hybrid-fusion helpers (Rust API only)",
    where: (
      <>
        <Code>sparq-vectors</Code> · <Code>fuse.rs</Code>
      </>
    ),
    what: (
      <>
        Reciprocal-rank fusion (<Code>fuse_rrf</Code>), min-max score blending
        (<Code>fuse_scores</Code>) and a multi-retriever <Code>hybrid_search</Code>{" "}
        driver combine vector, lexical and structural rankings —{" "}
        <strong className="text-foreground">
          as a Rust API only; there is deliberately no in-query SPARQL surface yet
        </strong>{" "}
        (that is the proposed <Code>vec:hybrid</Code> row below).
      </>
    ),
  },
];

// ── Proposed / designed-only ──────────────────────────────────────────────────
// Nothing in this table ships today. Each row cites the governing bead in the
// tracker (.beads/issues.jsonl) that scopes it.
const PROPOSED: {
  capability: string;
  bead: string;
  status: React.ReactNode;
}[] = [
  {
    capability: "vec:hybrid — hybrid retrieval + reranking as a specified SPARQL extension",
    bead: "sq-lhcot.4",
    status: (
      <>
        A narrow algebra rewrite over text / vector / structural rankings with
        deterministic RRF, explicit weights, top-k semantics and per-signal rank
        provenance, plus an out-of-process reranker trait. Design-only: today
        fusion exists solely as the Rust helpers above.
      </>
    ),
  },
  {
    capability: "GNCE-style planner cardinality estimator",
    bead: "sq-lhcot.5",
    status: (
      <>
        KGE-derived cardinality features on the existing characteristic-set
        planner seam — <strong className="text-foreground">planner-only</strong>,
        so a wrong estimate can never change answers. Not prototyped; adoption is
        gated on winning on held-out workloads.
      </>
    ),
  },
  {
    capability: "External-key .spqv interoperability profile",
    bead: "sq-lhcot.2",
    status: (
      <>
        A jointly designed (with the Kern/PSS project,{" "}
        <Artifact href={`${REPO}/issues/1746`} label="#1746" />) stable-key
        profile so a vector store survives re-parsing identical RDF — today the
        store is keyed by dictionary ids tied to the exact persisted graph
        generation. Co-design not yet frozen; no sparq implementation exists.
      </>
    ),
  },
  {
    capability: "Provenance-carrying GraphRAG vertical slice",
    bead: "sq-lhcot.3",
    status: (
      <>
        An end-to-end, evaluated retrieval-augmented pipeline with provenance and
        reasoning ablations. Today RDF-native GraphRAG is a composition{" "}
        <em>vision</em> backed by the pieces above, not a measured artifact.
      </>
    ),
  },
];

export default function GenaiRetrievalPage() {
  return (
    <div className="space-y-10">
      {/* ── Header ──────────────────────────────────────────────── */}
      <header className="space-y-3">
        <div className="flex items-center gap-3">
          <span className="flex size-11 items-center justify-center rounded-xl bg-primary/10 text-primary">
            <Sparkles className="size-5" aria-hidden />
          </span>
          <div>
            <h1 className="text-2xl font-semibold">
              GenAI retrieval — implemented vs proposed
            </h1>
            <p className="text-sm text-muted-foreground">
              The honest status ledger for sparq&rsquo;s RDF-native retrieval
              composition.
            </p>
          </div>
        </div>
        <div className="flex flex-wrap gap-2">
          <Badge variant="outline">vector search</Badge>
          <Badge variant="outline">NL → SPARQL</Badge>
          <Badge variant="outline">provenance</Badge>
        </div>
      </header>

      {/* ── Lead ────────────────────────────────────────────────── */}
      <section className="measure space-y-3 text-sm text-muted-foreground">
        <p>
          sparq&rsquo;s GenAI thesis is <em>composition</em>: vector and text
          retrieval, exact SPARQL, schema introspection and provenance run in{" "}
          <strong className="text-foreground">one engine</strong>, where
          approximate, model-derived signals only ever <em>propose</em> — exact
          query evaluation decides what is answered, and provenance decides what is
          cited. This page is the status ledger for that composition: the first
          table is what ships today (every row names its crate and opt-in cargo
          feature), the second is what is designed-only (every row cites its
          tracker bead). All of it is opt-in — the core engine and the lean wasm
          bundle carry none of this by default.
        </p>
      </section>

      {/* ── Implemented today ───────────────────────────────────── */}
      <section className="space-y-4">
        <div className="flex items-center gap-2">
          <CheckCircle2 className="size-4 text-[var(--success)]" aria-hidden />
          <h2 className="text-lg font-semibold">Implemented today</h2>
        </div>
        <p className="text-sm text-muted-foreground">
          Each capability is in the repo now, gated behind the named opt-in cargo
          feature (all OFF by default), with tests in both feature states.
        </p>
        <div className="overflow-x-auto rounded-xl border">
          <table className="w-full border-collapse text-left text-sm">
            <thead>
              <tr className="border-b bg-muted/40 text-xs uppercase tracking-wide text-muted-foreground">
                <th scope="col" className="px-3 py-2.5 font-medium">
                  Capability
                </th>
                <th scope="col" className="px-3 py-2.5 font-medium">
                  Crate · feature
                </th>
                <th scope="col" className="px-3 py-2.5 font-medium">
                  What ships
                </th>
              </tr>
            </thead>
            <tbody>
              {IMPLEMENTED.map((r) => (
                <tr
                  key={r.capability}
                  className="border-b last:border-0 align-top hover:bg-muted/30"
                >
                  <td className="px-3 py-2.5 font-medium text-foreground">
                    {r.capability}
                  </td>
                  <td className="whitespace-nowrap px-3 py-2.5">{r.where}</td>
                  <td className="px-3 py-2.5 text-muted-foreground">{r.what}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <p className="text-xs text-muted-foreground">
          Source of truth:{" "}
          <Artifact
            href={`${REPO}/tree/main/crates/sparq-vectors`}
            label="crates/sparq-vectors"
          />{" "}
          and{" "}
          <Artifact
            href={`${REPO}/tree/main/crates/sparq-nlq`}
            label="crates/sparq-nlq"
          />{" "}
          — each Cargo.toml documents its feature gates.
        </p>
      </section>

      {/* ── Proposed / designed-only ────────────────────────────── */}
      <section className="space-y-4">
        <div className="flex items-center gap-2">
          <CircleDashed className="size-4 text-muted-foreground" aria-hidden />
          <h2 className="text-lg font-semibold">Proposed — designed, not built</h2>
        </div>
        <p className="text-sm text-muted-foreground">
          Nothing in this table ships today. Each row cites the governing bead in
          the repo&rsquo;s task tracker; a row moves to the table above only when
          its implementation lands behind an opt-in feature with tests.
        </p>
        <div className="overflow-x-auto rounded-xl border">
          <table className="w-full border-collapse text-left text-sm">
            <thead>
              <tr className="border-b bg-muted/40 text-xs uppercase tracking-wide text-muted-foreground">
                <th scope="col" className="px-3 py-2.5 font-medium">
                  Capability
                </th>
                <th scope="col" className="px-3 py-2.5 font-medium">
                  Bead
                </th>
                <th scope="col" className="px-3 py-2.5 font-medium">
                  Status
                </th>
              </tr>
            </thead>
            <tbody>
              {PROPOSED.map((r) => (
                <tr
                  key={r.bead}
                  className="border-b last:border-0 align-top hover:bg-muted/30"
                >
                  <td className="px-3 py-2.5 font-medium text-foreground">
                    {r.capability}
                  </td>
                  <td className="whitespace-nowrap px-3 py-2.5">
                    <Code>{r.bead}</Code>
                  </td>
                  <td className="px-3 py-2.5 text-muted-foreground">{r.status}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {/* ── What this page does NOT claim ───────────────────────── */}
      <section className="space-y-4">
        <div className="flex items-center gap-2">
          <AlertTriangle className="size-4 text-amber-500" aria-hidden />
          <h2 className="text-lg font-semibold">What this page does not claim</h2>
        </div>
        <ul className="space-y-3 text-sm text-muted-foreground">
          <li className="rounded-lg border bg-card px-3 py-2.5">
            <strong className="text-foreground">No performance claims.</strong>{" "}
            This page carries no numbers by design. Comparative retrieval evidence
            (matched-recall ANN comparisons against purpose-built vector systems,
            real-model NL→SPARQL accuracy) is itself open work — tracked as beads{" "}
            <Code>sq-hmd7l.19</Code> and <Code>sq-2m6zm.7</Code>. Measured results
            live only on the <Link href="/benchmarks" className="text-primary underline underline-offset-2">benchmarks dashboard</Link>.
          </li>
          <li className="rounded-lg border bg-card px-3 py-2.5">
            <strong className="text-foreground">
              &ldquo;Implemented&rdquo; means implemented and gated, not best-in-class.
            </strong>{" "}
            Purpose-built vector databases are today more polished on the pure-ANN
            axis; sparq&rsquo;s differentiation claim is the composition with exact
            SPARQL, validation and provenance — not that any single retrieval
            primitive is fastest.
          </li>
          <li className="rounded-lg border bg-card px-3 py-2.5">
            <strong className="text-foreground">No privacy claims here.</strong>{" "}
            This page makes no zero-knowledge or MPC claim; the privacy estate is
            research-grade and documented separately, with its caveats, under the{" "}
            <Link
              href="/capabilities/#privacy"
              className="text-primary underline underline-offset-2"
            >
              privacy lane
            </Link>{" "}
            on Capabilities.
          </li>
        </ul>
      </section>

      {/* ── Front doors / CTAs ──────────────────────────────────── */}
      <section className="flex flex-wrap gap-3 border-t pt-6">
        <Button asChild variant="outline" size="sm">
          <Link href="/capabilities/#search-genai">
            Search &amp; GenAI demos — Capabilities
          </Link>
        </Button>
        <Button asChild variant="outline" size="sm">
          <Link href="/specs">Specs — the vector + GenAI proposal draft</Link>
        </Button>
        <Button asChild variant="outline" size="sm">
          <a
            href={`${REPO}/tree/main/crates/sparq-vectors`}
            target="_blank"
            rel="noopener noreferrer"
          >
            crates/sparq-vectors
            <ArrowUpRight className="size-3.5 opacity-60" aria-hidden />
          </a>
        </Button>
        <Button asChild variant="outline" size="sm">
          <a
            href={`${REPO}/tree/main/crates/sparq-nlq`}
            target="_blank"
            rel="noopener noreferrer"
          >
            crates/sparq-nlq
            <ArrowUpRight className="size-3.5 opacity-60" aria-hidden />
          </a>
        </Button>
      </section>
    </div>
  );
}
