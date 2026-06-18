import type { Metadata } from "next";
import { Brain } from "lucide-react";

import { SurfaceContent } from "@/components/surface-content";
import { InferencePlayground } from "@/components/inference-playground";

export const metadata: Metadata = {
  title: "Inference",
  description:
    "Forward-chaining RDFS / OWL 2 RL / Notation3 reasoning with the sparq reasoner — base vs entailed triple counts and a why() derivation proof tree, live in your tab via the W-reason wasm bundle.",
};

export default function InferenceSurfacePage() {
  return (
    <SurfaceContent
      icon={Brain}
      title="Inference"
      statement="Forward-chain the RDFS / OWL 2 RL / N3 closure — see what reasoning adds, then ask why."
      tier="live-new-wasm"
      intro={
        <>
          <p>
            <code className="font-mono text-foreground">sparq-reason</code> is a
            forward-chaining reasoner over a{" "}
            <code className="font-mono">sparq_core::Graph</code>: it materializes
            the deductive closure of an ontology + facts under{" "}
            <strong className="text-foreground">RDFS</strong> or{" "}
            <strong className="text-foreground">OWL 2 RL</strong>, runs{" "}
            <strong className="text-foreground">Notation3 rules</strong> (
            <code className="font-mono">{"{ … } => { … }"}</code>), and — uniquely —
            reconstructs a <code className="font-mono">why()</code>{" "}
            <strong className="text-foreground">derivation proof tree</strong> for
            any entailed triple: the chain of inference rules and asserted facts that
            justify it.
          </p>
          <p>
            Because it is pure Rust over <code className="font-mono">sparq-engine</code>
            , it compiles to wasm. The playground below loads the tier-b{" "}
            <strong className="text-foreground">&ldquo;W-reason&rdquo; bundle</strong>{" "}
            on demand — a separate, lazily-loaded bundle from the lean SPARQL REPL — and
            runs the reasoner in this tab: paste an ontology, see the base vs entailed
            triple counts, then click an entailed triple to render its proof. The
            default is the classic Socrates syllogism.
          </p>
        </>
      }
      capabilities={[
        {
          title: "RDFS closure",
          body: "subClassOf / subPropertyOf / domain / range entailments, forward-chained to fixpoint.",
        },
        {
          title: "OWL 2 RL closure",
          body: "Property axioms (inverseOf, symmetric/transitive), class axioms, restrictions — the RL profile (includes RDFS).",
        },
        {
          title: "Notation3 rules",
          body: "{ … } => { … } rule documents over facts — the Socrates example, run in-tab.",
        },
        {
          title: "Base vs entailed counts",
          body: "Distinct asserted triples, the closure size, and exactly how many triples reasoning added.",
        },
        {
          title: "why() proof trees",
          body: "Click an entailed triple for one derivation: a premises-before-conclusion DAG bottoming out in asserted facts.",
        },
        {
          title: "Entailed-only delta",
          body: "Just the triples reasoning ADDED — the closure minus the asserted base — not the whole closure.",
        },
      ]}
      runsNote={
        <>
          <p>
            <strong className="text-foreground">Live in your browser tab.</strong>{" "}
            The reasoner is pure Rust over the engine, compiled to wasm in the
            separate W-reason bundle (
            <code className="font-mono">sparq-reason-wasm</code>, built with the{" "}
            <code className="font-mono">explain</code> feature for{" "}
            <code className="font-mono">why()</code>). The page lazy-loads it on first
            interaction so the landing page never pays for it, then calls the bundle&rsquo;s{" "}
            <code className="font-mono text-foreground">Reasoner.materializeStats</code>,{" "}
            <code className="font-mono">Reasoner.entailed</code> /{" "}
            <code className="font-mono">Reasoner.reasonN3</code>, and{" "}
            <code className="font-mono">Reasoner.why</code> bindings — nothing is sent
            to a server.
          </p>
        </>
      }
      caveat={
        <>
          <p>
            The reasoner is a forward-chaining materializer sized for the
            illustrative documents here (tens of triples); it is not a tableau OWL DL
            reasoner, and the OWL 2 RL profile is the rule-based RL fragment, not full
            OWL. The N3 mode reasons over the rule document directly, so the{" "}
            <code className="font-mono">why?</code> proof-tree button is shown for the
            RDFS / OWL profiles only. Materialize large graphs server-side via{" "}
            <code className="font-mono">sparq-cli reason</code>.
          </p>
        </>
      }
    >
      <InferencePlayground />
    </SurfaceContent>
  );
}
