import type { Metadata } from "next";
import Link from "next/link";
import { Lock } from "lucide-react";

import { SurfaceContent } from "@/components/surface-content";

export const metadata: Metadata = {
  title: "ZK query proofs",
  description:
    "Prove and verify a SPARQL query result over committed RDF Verifiable Credentials in zero knowledge with sparq-zk — Poseidon2 commitments, BGP + FILTER Noir proofs, issuer attestation, and revocation.",
};

export default function ZkSurfacePage() {
  return (
    <SurfaceContent
      icon={Lock}
      title="ZK query proofs"
      statement="Prove a SPARQL answer over committed RDF credentials in zero knowledge."
      tier="live-bbjs"
      extraBadge="Research-grade · pending external review"
      intro={
        <>
          <p>
            <code className="font-mono text-foreground">sparq-zk</code> +{" "}
            <code className="font-mono text-foreground">sparq-zk-compose</code> let
            a holder prove that a SPARQL query result is faithfully derived from
            issuer-attested RDF Verifiable Credentials —{" "}
            <strong className="text-foreground">without revealing the underlying
            graph</strong>. Graphs are committed with Poseidon2; the query logic
            (BGP scan, integer FILTER, equality join) is proven with Noir circuits
            compiled to Barretenberg UltraHonk.
          </p>
          <p>
            Issuer attestation uses Schnorr signatures (including hidden-key set
            membership), revocation is checked against a status list at a clear or
            hidden index, and verifier nonces give single-use replay defence. The
            whole proof is bundled as a{" "}
            <code className="font-mono text-foreground">ProofManifest</code>.
          </p>
        </>
      }
      capabilities={[
        {
          term: "Per-graph Poseidon2 commitments",
          body: "Commit a credential graph to a single field element the proof binds against.",
        },
        {
          term: "BGP scan + integer FILTER",
          body: "Prove a basic graph pattern matched and a numeric predicate (e.g. age ≥ 25) held.",
        },
        {
          term: "Issuer attestation",
          body: "Schnorr signatures, incl. hidden-key set membership (prove a trusted issuer without naming it).",
        },
        {
          term: "Revocation",
          body: "Status-list non-revocation at a clear or hidden index.",
        },
        {
          term: "Equality join",
          body: "Prove two credentials share a holder term without disclosing it.",
        },
        {
          term: "Replay defence",
          body: "Verifier-supplied nonces make each presentation single-use.",
        },
      ]}
      runsNote={
        <>
          <p>
            Planned <strong className="text-foreground">live in your tab via bb.js</strong>{" "}
            (UltraHonk WASM): the circuit family is 6k–35k gates — roughly 15× under
            the browser ceiling — so proving in-tab is feasible (single-threaded on
            GitHub Pages; a COEP shim unlocks threads). See the{" "}
            <Link className="text-primary underline-offset-4 hover:underline" href="/showcase/zk-car-hire">ZK car-hire flagship</Link>.
          </p>
        </>
      }
      caveat={
        <>
          <p>
            <strong className="text-foreground">Research-grade (v1), pending external
            review.</strong> An internal re-audit found the verifier sound{" "}
            <em>as landed under its stated threat model</em> (the relying party
            supplies its own trust anchors), but no external accredited cryptographer
            has reviewed it, and the project&rsquo;s published{" "}
            <code className="font-mono">SECURITY.md</code> posture is the
            conservative one. Some privacy deferrals remain (e.g. status-list
            IRI/version disclosure → linkability). Do not treat these proofs as
            production-certified.
          </p>
        </>
      }
      links={[{ href: "/showcase/zk-car-hire", label: "★ ZK car-hire flagship" }]}
    />
  );
}
