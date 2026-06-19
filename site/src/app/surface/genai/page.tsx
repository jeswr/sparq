import type { Metadata } from "next";
import { Sparkles } from "lucide-react";

import { SurfaceContent } from "@/components/surface-content";
import { GenaiWalkthrough } from "@/components/genai-walkthrough";

export const metadata: Metadata = {
  title: "GenAI / NLQ",
  description:
    "sparq-nlq — natural-language questions → SPARQL, grounded in a real schema card, validated by spargebra, executed under a query budget, with a bounded repair loop. This page replays REAL captured engine output over the 1.78M-triple Olympics dataset; the model step is a recorded fixture, not a live LLM.",
};

export default function GenaiSurfacePage() {
  return (
    <SurfaceContent
      icon={Sparkles}
      title="GenAI / NLQ"
      statement="Natural-language → SPARQL: schema-card grounding, spargebra validation, budgeted execution, and a bounded repair loop."
      tier="walkthrough"
      extraBadge="Opt-in crate · captured output"
      intro={
        <>
          <p>
            <code className="font-mono text-foreground">sparq-nlq</code> turns a
            natural-language question into a SPARQL query over a{" "}
            <code className="font-mono">sparq_core::Graph</code>. It is a deliberately{" "}
            <strong className="text-foreground">lean loop</strong> (retrieve + repair, not
            a sprawling agent):{" "}
            <strong className="text-foreground">ground</strong> the prompt with{" "}
            <code className="font-mono">sparq-introspect</code>&rsquo;s token-budgeted
            schema card → <strong className="text-foreground">generate</strong> a query →{" "}
            <strong className="text-foreground">validate</strong> it with the real{" "}
            <code className="font-mono">spargebra</code> parser → on a parse/exec error{" "}
            <strong className="text-foreground">repair</strong> (the error fed back, a few
            rounds) → <strong className="text-foreground">execute</strong> under a{" "}
            <code className="font-mono">QueryBudget</code> (LLM output is untrusted input).
          </p>
          <p>
            It is an <strong className="text-foreground">opt-in crate</strong>: nothing in
            the workspace depends on it, the default build does not compile it, and the
            engine carries zero GenAI code. The model sits behind a small{" "}
            <code className="font-mono">Llm</code> trait with{" "}
            <strong className="text-foreground">record / replay</strong> fixtures, so CI is
            fully offline; a thin Anthropic Messages-API client lives behind a non-default{" "}
            <code className="font-mono">live</code> feature.
          </p>
          <p>
            This static Pages site has no backend and the crate is native-only — so,
            honestly, this is a <strong className="text-foreground">captured-output
            walkthrough</strong>. Two halves, labelled distinctly:{" "}
            <strong className="text-foreground">(1)</strong> the schema card and{" "}
            <strong className="text-foreground">every result table</strong> are{" "}
            <em>real, verbatim engine output</em> — produced by running the real loop with
            the committed replay fixture against the real 1.78M-triple Olympics dataset
            (term serialization intact, datatypes and language tags preserved).{" "}
            <strong className="text-foreground">(2)</strong> the{" "}
            <em>NL→SPARQL generation step itself is a recorded fixture, not a live model
            call here</em> — the queries are realistic, schema-grounded answers a competent
            model would write, with one deliberately malformed to exercise the repair path.
            We do not claim the query text is &ldquo;what an LLM said&rdquo;; we claim the
            loop&rsquo;s plumbing and the executed results are real.
          </p>
        </>
      }
      capabilities={[
        {
          title: "Schema-card grounding (real introspection)",
          body: "sparq-introspect's to_text_summary builds a token-budgeted deck — exact class/predicate counts, datatype ranges, characteristic sets — from the store's indexes, not guesses.",
        },
        {
          title: "Validate before execute",
          body: "The extracted query is parsed with the real spargebra parser before the engine sees it, so a malformed query becomes a concrete repair signal rather than a silent failure.",
        },
        {
          title: "Bounded repair loop",
          body: "Parse and execution errors (unsupported forms, budget trips) are fed back with the failed query; a capped number of rounds, then the loop gives up honestly.",
        },
        {
          title: "Budgeted execution",
          body: "LLM-generated queries are untrusted input — every query runs under a QueryBudget (default 10 s wall clock, 1M materialised rows); opting out is explicit.",
        },
        {
          title: "Record / replay — offline CI",
          body: "The Llm trait records (prompt, completion) pairs to a fixture and replays them by exact-prompt match, so the whole loop is deterministic and network-free in tests.",
        },
        {
          title: "Index-grounded entity linking (opt-in)",
          body: "Optionally resolve a question's proper nouns to concrete IRIs in this store (label index + sparq-sim structural siblings) and surface them in the prompt — off by default.",
        },
      ]}
      runsNote={
        <>
          <p>
            <strong className="text-foreground">Captured-output replay.</strong> The schema
            card is the verbatim <code className="font-mono">to_text_summary(4000)</code>{" "}
            deck; every result table is the verbatim output of the real loop (replay
            fixture) executed by the sparq engine over the real Olympics dataset. The
            row counts and the one repair round are cross-checked, offline and
            deterministically, by the crate&rsquo;s CI test (
            <code className="font-mono">olympics_eval.rs</code>). To run it live:
          </p>
          <pre className="overflow-x-auto rounded-lg border bg-muted/40 p-3 font-mono text-[12px] leading-relaxed">
            {`cargo add sparq-nlq --features live
export ANTHROPIC_API_KEY=sk-ant-...
# then: Nlq::new(&graph, Box::new(AnthropicLlm::from_env()?)).ask("...")`}
          </pre>
        </>
      }
      caveat={
        <>
          <p>
            <strong className="text-foreground">The model step here is not live.</strong>{" "}
            The generation step on this page is a <em>recorded fixture</em>, not a real LLM
            call, so the SPARQL text is &ldquo;what a competent model would write&rdquo;,
            not a measured model output.{" "}
            <strong className="text-foreground">
              Live-model exec-accuracy numbers are not measured here
            </strong>{" "}
            — that needs an API key and a canonical host (open work in the crate).
          </p>
          <p>
            <strong className="text-foreground">Exec-accuracy is not correctness.</strong>{" "}
            A query can parse, execute and return rows while answering the{" "}
            <em>wrong</em> question — passing the loop is not a guarantee the SPARQL
            faithfully captures the intent. Strict logit-level grammar-constrained{" "}
            <em>decoding</em> is not implementable against the Anthropic Messages API (no
            logit/grammar parameter); the dictionary-grounded check on this surface is the
            design&rsquo;s API-backend fallback, and it is opt-in.
          </p>
        </>
      }
      links={[
        {
          href: "https://github.com/jeswr/sparq/tree/main/crates/sparq-nlq",
          label: "sparq-nlq crate",
          external: true,
        },
        {
          href: "https://github.com/jeswr/sparq/tree/main/crates/sparq-introspect",
          label: "sparq-introspect crate",
          external: true,
        },
      ]}
    >
      <GenaiWalkthrough />
    </SurfaceContent>
  );
}
