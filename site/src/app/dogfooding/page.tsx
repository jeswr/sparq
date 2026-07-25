// (sq-gk9lq) [SONNET-4.6] — /dogfooding: how sparq uses itself as its own knowledge
// management system.
//
// Content grounded in the real estate:
//   crates/sparq-kb              — ontology + SHACL + ingest + sparq-pkg-nl
//   .github/workflows/pkg-ingest.yml — per-merge CI ingest
//   bench/pkg-dogfood/RESULTS.md — the sanctioned measured numbers
//   research/dogfooding-sparq-knowledge-graph.md — design record
//   research/research-kb-program.md             — literature pipeline design
//
// Honest treatment: the ~30× cost ratio is cited from the one measured run
// (N=30, directional, non-canonical work-box). Machine-tier limits, uncalibrated
// confidence, and the not-yet-run live pilot are all stated up front.

import type { Metadata } from "next";
import Link from "next/link";
import {
  ArrowUpRight,
  BookOpen,
  Check,
  Database,
  FlaskConical,
  GitBranch,
  Network,
  Search,
  ShieldCheck,
} from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

export const metadata: Metadata = {
  title: "Dogfooding sparq",
  description:
    "How sparq stores its own project knowledge as RDF and queries it with sparq itself — the provenance-stamped PKG loop, the ingest workflow, the NL query tool, and the measured cost benefit.",
};

const REPO = "https://github.com/sparq-org/sparq";

function Artifact({
  href,
  label,
}: {
  href: string;
  label: string;
}) {
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

export default function DogfoodingPage() {
  return (
    <div className="space-y-10">
      {/* ── Header ──────────────────────────────────────────────── */}
      <header className="space-y-3">
        <div className="flex items-center gap-3">
          <span className="flex size-11 items-center justify-center rounded-xl bg-primary/10 text-primary">
            <Database className="size-5" aria-hidden />
          </span>
          <div>
            <h1 className="text-2xl font-semibold">Dogfooding sparq</h1>
            <p className="text-sm text-muted-foreground">
              sparq stores its own project knowledge as RDF and queries it with sparq.
            </p>
          </div>
        </div>
        <div className="flex flex-wrap gap-2">
          <Badge variant="outline">knowledge management</Badge>
          <Badge variant="outline">research-grade</Badge>
        </div>
      </header>

      {/* ── Lead ────────────────────────────────────────────────── */}
      <section className="measure space-y-3 text-sm text-muted-foreground">
        <p>
          sparq&rsquo;s own working knowledge &mdash; research findings, task backlog,
          skill surfaces, and literature &mdash; lives as an RDF graph queried by sparq
          itself. Every committed fact carries a{" "}
          <code className="rounded bg-muted px-1 py-0.5 font-mono text-xs text-foreground">
            prov:wasDerivedFrom
          </code>{" "}
          lineage and passes a SHACL gate before it enters the store. A cheap Haiku
          sub-agent answers project-knowledge questions via SPARQL so the expensive
          orchestrator only reads the answer, not the corpus.
        </p>
        <p>
          This page explains the loop concretely: what goes in, how it is stored, how it
          is queried, what has been measured, and what remains open.
        </p>
      </section>

      {/* ── Knowledge model ─────────────────────────────────────── */}
      <section className="space-y-4">
        <div className="flex items-center gap-2">
          <Network className="size-4 text-primary" aria-hidden />
          <h2 className="text-lg font-semibold">The knowledge model</h2>
        </div>
        <p className="text-sm text-muted-foreground">
          The{" "}
          <Artifact
            href={`${REPO}/blob/main/crates/sparq-kb/ontology/pkg/pkg.ttl`}
            label="pkg.ttl ontology"
          />{" "}
          in{" "}
          <code className="rounded bg-muted px-1 py-0.5 font-mono text-xs text-foreground">
            crates/sparq-kb
          </code>{" "}
          follows a reuse-first design: almost no net-new terms are minted. Standard
          vocabularies cover each concern.
        </p>
        <div className="grid gap-3 sm:grid-cols-2">
          {[
            {
              vocab: "PROV-O",
              role: "who derived what from which source, and when (generatedAtTime)",
            },
            {
              vocab: "SKOS",
              role: "concepts, algorithms, and techniques with typed relations (extends / usesMethodIn / supersedes)",
            },
            {
              vocab: "DCAT + FaBiO + DC",
              role: "the source catalog — papers, design records, and the literature corpus",
            },
            {
              vocab: "DQV",
              role: "quality axis: confidence and assurance as named, distinct quality dimensions",
            },
            {
              vocab: "CiTO",
              role: "typed citation edges (citesAsEvidence, usesMethodIn)",
            },
            {
              vocab: "schema.org",
              role: "fills remaining gaps; the top-level type for agent knowledge-management",
            },
          ].map(({ vocab, role }) => (
            <div
              key={vocab}
              className="flex gap-2.5 rounded-lg border bg-card px-3 py-2.5 text-sm"
            >
              <Check
                className="mt-0.5 size-4 shrink-0 text-[var(--success)]"
                aria-hidden
              />
              <span className="text-muted-foreground">
                <span className="font-semibold text-foreground">{vocab}</span>
                {" — "}
                {role}
              </span>
            </div>
          ))}
        </div>
        <p className="text-xs text-muted-foreground">
          Only four terms are genuinely net-new:{" "}
          <code className="font-mono">pkg:exploredStatus</code>,{" "}
          <code className="font-mono">pkg:followUpPriority</code>,{" "}
          <code className="font-mono">pkg:confidence</code>, and{" "}
          <code className="font-mono">pkg:couldBeMergedWith</code>
          {" — "}plus the{" "}
          <code className="font-mono">pkg:dependsOn</code> /{" "}
          <code className="font-mono">pkg:blockedBy</code> inverse pair. See the{" "}
          <Artifact
            href={`${REPO}/blob/main/crates/sparq-kb/ontology/pkg/pkg.ttl`}
            label="ontology file"
          />{" "}
          and the{" "}
          <Artifact
            href={`${REPO}/blob/main/research/dogfooding-sparq-knowledge-graph.md`}
            label="design record"
          />{" "}
          for the full reuse table.
        </p>
      </section>

      {/* ── What gets captured ──────────────────────────────────── */}
      <section className="space-y-4">
        <div className="flex items-center gap-2">
          <BookOpen className="size-4 text-primary" aria-hidden />
          <h2 className="text-lg font-semibold">What gets captured</h2>
        </div>
        <div className="grid gap-4 sm:grid-cols-3">
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-base">Tasks &amp; backlog</CardTitle>
            </CardHeader>
            <CardContent className="text-sm text-muted-foreground">
              Every bead (bd task) is projected as a{" "}
              <code className="font-mono text-xs text-foreground">pkg:Task</code> with
              its status, priority, dependency edges, and links to design records. The
              bd tracker stays the source of record; the PKG mirrors it as a read model.
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-base">Skills &amp; techniques</CardTitle>
            </CardHeader>
            <CardContent className="text-sm text-muted-foreground">
              Each{" "}
              <code className="font-mono text-xs text-foreground">skills/*/SKILL.md</code>{" "}
              YAML front-matter is deterministically parsed into{" "}
              <code className="font-mono text-xs text-foreground">pkg:Source</code> +{" "}
              <code className="font-mono text-xs text-foreground">pkg:Technique</code>{" "}
              triples, capturing name, description, and the CiTO relations to related
              techniques.
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-base">Research findings</CardTitle>
            </CardHeader>
            <CardContent className="text-sm text-muted-foreground">
              Hand-authored findings from design records are kept in{" "}
              <code className="font-mono text-xs text-foreground">
                agents-findings.yaml.ld
              </code>
              , compiled to Turtle. Each finding carries{" "}
              <code className="font-mono text-xs text-foreground">prov:wasDerivedFrom</code>
              , a{" "}
              <code className="font-mono text-xs text-foreground">pkg:confidence</code>{" "}
              score, and a{" "}
              <code className="font-mono text-xs text-foreground">secx:assurance</code>{" "}
              level from the zkp-sparql vocabulary.
            </CardContent>
          </Card>
        </div>
      </section>

      {/* ── The ingest loop ─────────────────────────────────────── */}
      <section className="space-y-4">
        <div className="flex items-center gap-2">
          <GitBranch className="size-4 text-primary" aria-hidden />
          <h2 className="text-lg font-semibold">The ingest loop</h2>
        </div>
        <ol className="space-y-3 text-sm text-muted-foreground">
          <li className="flex gap-3">
            <span className="flex size-6 shrink-0 items-center justify-center rounded-full bg-primary/10 text-xs font-semibold text-primary">
              1
            </span>
            <span>
              <strong className="text-foreground">Structured-parse projection.</strong>{" "}
              <Artifact
                href={`${REPO}/blob/main/crates/sparq-kb/ingest/ingest_pkg.py`}
                label="ingest_pkg.py"
              />{" "}
              deterministically projects the bead backlog and skill front-matter into
              Turtle. No LLM call, no guesswork &mdash; the output is a function of the
              committed JSON + YAML.
            </span>
          </li>
          <li className="flex gap-3">
            <span className="flex size-6 shrink-0 items-center justify-center rounded-full bg-primary/10 text-xs font-semibold text-primary">
              2
            </span>
            <span>
              <strong className="text-foreground">SHACL validation, fail-closed.</strong>{" "}
              The{" "}
              <Artifact
                href={`${REPO}/blob/main/crates/sparq-kb/shapes/pkg.shapes.ttl`}
                label="SHACL shapes"
              />{" "}
              gate runs against the output. Any conformance failure quarantines the
              offending edges to a sidecar{" "}
              <code className="font-mono text-xs text-foreground">.stale-edges.tsv</code>{" "}
              &mdash; they are reported, never silently dropped, and never admitted to the
              store.
            </span>
          </li>
          <li className="flex gap-3">
            <span className="flex size-6 shrink-0 items-center justify-center rounded-full bg-primary/10 text-xs font-semibold text-primary">
              3
            </span>
            <span>
              <strong className="text-foreground">Per-merge CI trigger.</strong>{" "}
              The{" "}
              <Artifact
                href={`${REPO}/blob/main/.github/workflows/pkg-ingest.yml`}
                label="pkg-ingest.yml"
              />{" "}
              workflow runs on every push to{" "}
              <code className="font-mono text-xs text-foreground">main</code> that touches
              the ingest scripts, ontology, shapes, beads, or skills. The ingested Turtle
              and any quarantine sidecar are uploaded as workflow artifacts so every run is
              auditable.
            </span>
          </li>
        </ol>
      </section>

      {/* ── How it is queried ───────────────────────────────────── */}
      <section className="space-y-4">
        <div className="flex items-center gap-2">
          <Search className="size-4 text-primary" aria-hidden />
          <h2 className="text-lg font-semibold">How it is queried</h2>
        </div>
        <div className="grid gap-4 sm:grid-cols-2">
          <div className="space-y-2 rounded-xl border bg-card p-4 text-sm">
            <p className="font-semibold text-foreground">sparq-pkg-nl (the NL tool)</p>
            <p className="text-muted-foreground">
              A Haiku sub-agent does the full natural-language &rarr; SPARQL &rarr; run
              &rarr; NL-answer round-trip. The orchestrator emits a plain-English question
              and reads back the answer plus the executed SPARQL and provenance IRI — the
              verbose middle runs on the cheap model. The result is computed by the engine,
              not generated by the LLM.
            </p>
            <p className="text-muted-foreground">
              Invoked via the{" "}
              <code className="font-mono text-xs text-foreground">query-pkg</code> skill or
              as a direct sparq-pkg-nl agent call.
            </p>
          </div>
          <div className="space-y-2 rounded-xl border bg-card p-4 text-sm">
            <p className="font-semibold text-foreground">pkg_query binary</p>
            <p className="text-muted-foreground">
              The{" "}
              <code className="font-mono text-xs text-foreground">pkg_query</code> CLI
              loads the PKG Turtle, introspects the schema, and runs a SPARQL SELECT. It
              is also used in the A/B benchmark harness to measure the cost of having the
              orchestrator run queries itself, versus delegating to the cheap-model NL
              tool.
            </p>
          </div>
        </div>
      </section>

      {/* ── Measured cost benefit ───────────────────────────────── */}
      <section className="space-y-4">
        <div className="flex items-center gap-2">
          <FlaskConical className="size-4 text-primary" aria-hidden />
          <h2 className="text-lg font-semibold">Measured cost benefit</h2>
        </div>
        <Card className="border-primary/20 bg-primary/5">
          <CardContent className="pt-4 text-sm">
            <p className="text-muted-foreground">
              A single N=30, counterbalanced, non-canonical work-box run (list-price
              approximations; not a significance study) compared three arms on
              PKG-answerable project-knowledge questions:
            </p>
            <div className="my-4 overflow-x-auto">
              <table className="w-full text-left text-xs">
                <thead>
                  <tr className="border-b text-muted-foreground">
                    <th className="pb-1.5 pr-4 font-medium">Arm</th>
                    <th className="pb-1.5 pr-4 font-medium">Who answers</th>
                    <th className="pb-1.5 pr-4 font-medium">Approx. cost ratio</th>
                    <th className="pb-1.5 font-medium">Quality</th>
                  </tr>
                </thead>
                <tbody className="text-muted-foreground">
                  <tr className="border-b">
                    <td className="py-1.5 pr-4 font-mono">A — read-docs</td>
                    <td className="py-1.5 pr-4">Opus reads source docs</td>
                    <td className="py-1.5 pr-4">~30×</td>
                    <td className="py-1.5">0.92</td>
                  </tr>
                  <tr className="border-b">
                    <td className="py-1.5 pr-4 font-mono">B — Opus pkg-query</td>
                    <td className="py-1.5 pr-4">Opus runs pkg_query</td>
                    <td className="py-1.5 pr-4">~16×</td>
                    <td className="py-1.5">1.00</td>
                  </tr>
                  <tr>
                    <td className="py-1.5 pr-4 font-mono font-semibold text-foreground">
                      C — Haiku NL-tool
                    </td>
                    <td className="py-1.5 pr-4 text-foreground">
                      Haiku sub-agent (NL tool)
                    </td>
                    <td className="py-1.5 pr-4 font-semibold text-foreground">1×</td>
                    <td className="py-1.5 text-foreground">1.00</td>
                  </tr>
                </tbody>
              </table>
            </div>
            <p className="text-xs text-muted-foreground">
              Cost ratios are model-price-weighted (list-price Opus / Haiku rates). All
              figures are NON-CANONICAL &mdash; work-box transcripts, list-price
              approximations, single directional run. See the full methodology and
              caveats in{" "}
              <Artifact
                href={`${REPO}/blob/main/bench/pkg-dogfood/RESULTS.md`}
                label="bench/pkg-dogfood/RESULTS.md"
              />
              .
            </p>
          </CardContent>
        </Card>
        <p className="text-xs text-muted-foreground">
          The win is scoped to the PKG-answerable question class (questions the store can
          answer; questions outside the PKG head-slice still require a doc-read on every
          arm). The quality gap between arms A and B/C is a modest over-statement because
          the gold keys reward the structured row-shaped answer pkg-query returns; the
          cost gap is the load-bearing result.
        </p>
      </section>

      {/* ── Literature pipeline ─────────────────────────────────── */}
      <section className="space-y-4">
        <div className="flex items-center gap-2">
          <ShieldCheck className="size-4 text-primary" aria-hidden />
          <h2 className="text-lg font-semibold">Literature pipeline</h2>
        </div>
        <p className="text-sm text-muted-foreground">
          Beyond the hand-authored tier, a machine-extraction pipeline (behind the
          optional{" "}
          <code className="font-mono text-xs text-foreground">literature</code> cargo
          feature) ingests external papers. Each step is fail-closed.
        </p>
        <ol className="space-y-2 text-sm text-muted-foreground">
          <li className="flex gap-2.5">
            <span className="font-semibold text-foreground">CORE connector.</span>{" "}
            Fetches papers via the CORE v3 API, capturing DOI, title, abstract, and
            license. Unknown license is treated as non-redistributable.
          </li>
          <li className="flex gap-2.5">
            <span className="font-semibold text-foreground">Capped extractor.</span>{" "}
            A Haiku sub-agent extracts findings from each abstract. Machine-tier caps are
            enforced at the extractor boundary: confidence ≤ 0.7, assurance never{" "}
            <code className="font-mono text-xs text-foreground">secx:Proven</code>.
          </li>
          <li className="flex gap-2.5">
            <span className="font-semibold text-foreground">Grounding gate.</span>{" "}
            Every extracted claim must be entailed by a span in the abstract, and cited
            DOIs must resolve within the batch. Un-groundable findings go to quarantine,
            never the store.
          </li>
          <li className="flex gap-2.5">
            <span className="font-semibold text-foreground">Tiered emission.</span>{" "}
            Findings are stamped with{" "}
            <code className="font-mono text-xs text-foreground">prov:wasDerivedFrom</code>
            ,{" "}
            <code className="font-mono text-xs text-foreground">
              prov:wasGeneratedBy
            </code>
            , and{" "}
            <code className="font-mono text-xs text-foreground">
              prov:generatedAtTime
            </code>{" "}
            before entering the machine tier. License-restricted findings are kept in a
            separate tier (metadata-only in any public export; no abstract-derived text).
          </li>
        </ol>
        <p className="text-sm text-muted-foreground">
          A private dump repo (
          <code className="font-mono text-xs text-foreground">
            sparq-org/research-kb
          </code>
          ) holds per-merge, per-tier Turtle exports plus a manifest and a
          dump-provenance record. Flipping the repo public is a maintainer decision, gated
          on mechanically-verified license-tier enforcement.
        </p>
      </section>

      {/* ── Honest limits ───────────────────────────────────────── */}
      <details className="group rounded-xl border bg-muted/30 px-4 py-3 text-sm">
        <summary className="cursor-pointer select-none font-medium text-foreground/90">
          Honest limits
        </summary>
        <ul className="mt-3 space-y-2 text-muted-foreground">
          <li>
            <strong className="text-foreground">Machine tier is capped, not trusted.</strong>{" "}
            Confidence is bounded at ≤ 0.7 and assurance is never{" "}
            <code className="font-mono text-xs">secx:Proven</code> for machine-extracted
            findings. The SHACL shapes enforce this; a finding that violates the cap is
            quarantined, never admitted.
          </li>
          <li>
            <strong className="text-foreground">
              Confidence scores are uncalibrated.
            </strong>{" "}
            The{" "}
            <code className="font-mono text-xs">pkg:confidence</code> literal is a
            meaningful ordering signal but it is not a calibrated probability. How to
            propagate confidence through SPARQL joins (annotated-RDF semirings) remains
            an open research question.
          </li>
          <li>
            <strong className="text-foreground">The live pilot has not yet run.</strong>{" "}
            The literature pipeline is implemented and the CORE API key is provisioned, but
            the first end-to-end pilot (connector &rarr; extractor &rarr; grounding
            &rarr; conformance audit) is maintainer-armed and has not been executed. No
            literature-derived finding has been admitted to the store yet.
          </li>
          <li>
            <strong className="text-foreground">
              The cost result is scoped and non-canonical.
            </strong>{" "}
            The ~30× ratio is from one N=30 run on work-box transcripts at list-price
            approximations. It holds for the PKG-answerable question class; questions
            outside the head-slice still require a doc-read. See{" "}
            <Artifact
              href={`${REPO}/blob/main/bench/pkg-dogfood/RESULTS.md`}
              label="RESULTS.md"
            />{" "}
            for the full caveat list.
          </li>
          <li>
            <strong className="text-foreground">
              SPARQL can return wrong answers from correct data.
            </strong>{" "}
            The NL &rarr; SPARQL conversion is LLM-generated. Bounded-exact answers
            eliminate in-store answer fabrication, but the LLM can still generate a
            wrong-but-syntactically-valid query that returns an incorrect result. The
            executed SPARQL is always returned alongside the answer so callers can verify.
          </li>
        </ul>
      </details>

      {/* ── Artifacts ───────────────────────────────────────────── */}
      <section className="flex flex-wrap gap-2">
        <Button asChild variant="outline" size="sm">
          <a
            href={`${REPO}/blob/main/crates/sparq-kb/ontology/pkg/pkg.ttl`}
            target="_blank"
            rel="noopener noreferrer"
          >
            pkg.ttl ontology
            <ArrowUpRight className="size-3.5 opacity-60" aria-hidden />
          </a>
        </Button>
        <Button asChild variant="outline" size="sm">
          <a
            href={`${REPO}/blob/main/.github/workflows/pkg-ingest.yml`}
            target="_blank"
            rel="noopener noreferrer"
          >
            pkg-ingest.yml workflow
            <ArrowUpRight className="size-3.5 opacity-60" aria-hidden />
          </a>
        </Button>
        <Button asChild variant="outline" size="sm">
          <a
            href={`${REPO}/blob/main/bench/pkg-dogfood/RESULTS.md`}
            target="_blank"
            rel="noopener noreferrer"
          >
            A/B benchmark results
            <ArrowUpRight className="size-3.5 opacity-60" aria-hidden />
          </a>
        </Button>
        <Button asChild variant="outline" size="sm">
          <a
            href={`${REPO}/blob/main/research/dogfooding-sparq-knowledge-graph.md`}
            target="_blank"
            rel="noopener noreferrer"
          >
            Design record
            <ArrowUpRight className="size-3.5 opacity-60" aria-hidden />
          </a>
        </Button>
        <Button asChild variant="outline" size="sm">
          <a
            href={`${REPO}/tree/main/crates/sparq-kb`}
            target="_blank"
            rel="noopener noreferrer"
          >
            crates/sparq-kb
            <ArrowUpRight className="size-3.5 opacity-60" aria-hidden />
          </a>
        </Button>
        <Button asChild variant="outline" size="sm">
          <Link href="/capabilities">Back to Capabilities</Link>
        </Button>
      </section>
    </div>
  );
}
