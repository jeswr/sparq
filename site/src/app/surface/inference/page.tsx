import type { Metadata } from "next";
import { Brain } from "lucide-react";

import { SurfaceContent } from "@/components/surface-content";
import { InferencePlayground } from "@/components/inference-playground";

export const metadata: Metadata = {
  title: "Inference",
  description:
    "Forward-chaining RDFS / OWL 2 RL / Notation3 reasoning with the sparq reasoner — base vs entailed triple counts and a why() derivation proof tree, live in your tab via the W-reason wasm bundle.",
};

// [OPUS-4.8] sq-vw3ax.4 — scan-first deep page: statement + the inference
// playground first (lazy-loads the W-reason bundle on interaction), then a tight
// capabilities list, a one-sentence "how this runs", caveats closed.
export default function InferenceSurfacePage() {
  return (
    <SurfaceContent
      icon={Brain}
      title="Inference"
      statement="Forward-chain the RDFS / OWL 2 RL / N3 closure — see what reasoning adds, then ask why, in your tab."
      tier="live-new-wasm"
      capabilities={[
        {
          term: "RDFS closure",
          body: "subClassOf / subPropertyOf / domain / range entailments, forward-chained to fixpoint.",
        },
        {
          term: "OWL 2 RL closure",
          body: "Property axioms (inverseOf, symmetric/transitive), class axioms and restrictions — the rule-based RL profile (includes RDFS).",
        },
        {
          term: "Notation3 rules",
          body: "{ … } => { … } rule documents over facts — the Socrates example, run in-tab.",
        },
        {
          term: "Base vs entailed counts",
          body: "Distinct asserted triples, the closure size, and exactly how many triples reasoning added (the entailed-only delta).",
        },
        {
          term: "why() proof trees",
          body: "Click an entailed triple for one derivation: a premises-before-conclusion DAG bottoming out in asserted facts.",
        },
      ]}
      runsNote={
        <>
          Runs live in your tab via the separately lazy-loaded W-reason bundle (
          <code className="font-mono">sparq-reason-wasm</code>, built with the{" "}
          <code className="font-mono">explain</code> feature) — loaded only on first
          interaction so the landing page never pays for it, with no network round-trip.
          Reproduce: <code className="font-mono">cargo test -p sparq-reason</code>.
        </>
      }
      caveat={
        <p>
          The reasoner is a forward-chaining materializer sized for the illustrative
          documents here (tens of triples); it is not a tableau OWL DL reasoner, and the
          OWL 2 RL profile is the rule-based RL fragment, not full OWL. The N3 mode
          reasons over the rule document directly, so the{" "}
          <code className="font-mono">why?</code> proof-tree button shows for the RDFS /
          OWL profiles only. Materialize large graphs server-side via{" "}
          <code className="font-mono">sparq-cli reason</code>.
        </p>
      }
    >
      <InferencePlayground />
    </SurfaceContent>
  );
}
