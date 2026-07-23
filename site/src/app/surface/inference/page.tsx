import type { Metadata } from "next";
import { Brain } from "lucide-react";

import { SurfaceContent } from "@/components/surface-content";
import { InferencePlayground } from "@/components/inference-playground";

export const metadata: Metadata = {
  title: "Inference",
  description:
    "Forward-chaining RDFS / OWL 2 RL / Notation3 reasoning with the sparq reasoner — base vs entailed triple counts and a why() derivation proof tree, live in your tab via the W-reason wasm bundle.",
};

// [OPUS-4.8] sq-vw3ax.8 — condensed to the scan-first template: demo first, one-line lead,
// ≤5 capabilities, single-sentence runs note + reproduce, caveats behind <details>, plus the
// universal README + SKILL.md links. The bundle-mechanics detail moves to the SKILL.
export default function InferenceSurfacePage() {
  return (
    <SurfaceContent
      icon={Brain}
      title="Inference"
      statement="Forward-chain the RDFS / OWL 2 RL / N3 closure — see what reasoning adds, then ask why."
      tier="live-new-wasm"
      intro={
        <p>
          <code className="font-mono text-foreground">sparq-reason</code> materializes
          the deductive closure under RDFS, OWL 2 RL or Notation3 rules and — uniquely —
          reconstructs a <code className="font-mono">why()</code> derivation proof tree
          for any entailed triple; the playground above runs it in your tab on the
          lazily-loaded W-reason wasm bundle.
        </p>
      }
      capabilities={[
        {
          title: "RDFS & OWL 2 RL closure",
          body: "subClassOf / subPropertyOf / domain / range plus the RL property & class axioms, forward-chained to fixpoint.",
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
      runsNote="Live in your browser tab — the reasoner is pure Rust over the engine, compiled to the separate W-reason wasm bundle that the page lazy-loads on first interaction, so nothing is sent to a server."
      reproduce="cargo test -p sparq-reason"
      caveat={
        <p>
          This is a forward-chaining materializer sized for the illustrative documents
          here (tens of triples), not a tableau OWL DL reasoner; the OWL 2 RL profile is
          the rule-based RL fragment, not full OWL. The N3 mode reasons over the rule
          document directly, so the <code className="font-mono">why?</code> proof-tree
          button shows for the RDFS / OWL profiles only. Materialize large graphs
          server-side via <code className="font-mono">sparq-cli reason</code>.
        </p>
      }
      readmeHref="https://github.com/sparq-org/sparq/tree/main/crates/sparq-reason"
      skillHref="https://github.com/sparq-org/sparq/blob/main/skills/inference/SKILL.md"
    >
      <InferencePlayground />
    </SurfaceContent>
  );
}
