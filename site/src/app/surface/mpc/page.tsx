import type { Metadata } from "next";
import Link from "next/link";
import { Network } from "lucide-react";

import { Button } from "@/components/ui/button";
import { SurfaceContent } from "@/components/surface-content";

export const metadata: Metadata = {
  title: "MPC federation",
  description:
    "Run a SPARQL query across mutually-distrusting RDF holders with sparq-mpc — per-holder local evaluation, disclosed-key joins, and honest-majority Shamir secret sharing for secure aggregates and threshold verdicts.",
};

export default function MpcSurfacePage() {
  return (
    <SurfaceContent
      icon={Network}
      title="MPC federation"
      statement="Answer one SPARQL query across distrusting holders, minimising what each reveals."
      tier="live-sim"
      extraBadge="Native-only · research / semi-honest"
      intro={
        <>
          <p>
            <code className="font-mono text-foreground">sparq-mpc</code> lets a set
            of mutually-distrusting <strong className="text-foreground">holders</strong>
            , each owning private RDF named graphs, jointly answer one SPARQL query
            over the <em>union</em> of their data while minimising what each
            reveals. Each holder evaluates its fragment{" "}
            <strong className="text-foreground">locally</strong> over its own graph;
            only projected rows ever leave a holder.
          </p>
          <p>
            On top of that it adds a crypto-free join on{" "}
            <em>disclosed</em> global IRIs and an honest-majority{" "}
            <strong className="text-foreground">Shamir secret-sharing</strong>{" "}
            backend that powers secure cumulative aggregates, a hidden-value
            (private-key) join, and a secure threshold comparison that opens{" "}
            <em>only</em> a boolean verdict.
          </p>
        </>
      }
      capabilities={[
        {
          term: "Per-holder local evaluation",
          body: "Each holder runs its own SELECT fragment over its own graph; the raw graph never leaves.",
        },
        {
          term: "Disclosed-key global join",
          body: "Crypto-free inner join on shared global IRIs, equal to union-store evaluation.",
        },
        {
          term: "Secure cumulative aggregate",
          body: "Sum private integers over Shamir shares (free local addition), reveal only the total.",
        },
        {
          term: "Secure threshold verdict",
          body: "Open only the bit `sum ≥ threshold` — the £100k flatmates example — never the operands.",
        },
        {
          term: "Hidden-value join",
          body: "Join on a private key via secret-shared equality; only matched payload columns disclosed.",
        },
        {
          term: "Fail-closed backend selection",
          body: "A federation states its security floor up front; an over-strong request is truthfully refused.",
        },
      ]}
      runsNote={
        <>
          <p>
            The flagship <Link className="text-primary underline-offset-4 hover:underline" href="/showcase/mpc-100k">£100k secure-threshold demo</Link>{" "}
            runs a <strong className="text-foreground">faithful in-tab illustration</strong>{" "}
            of the additive-secret-sharing flow. The hardened crate is{" "}
            <strong className="text-foreground">native-only</strong> (it uses{" "}
            <code className="font-mono">std::net</code> and runs as an in-process
            simulation) and is deliberately absent from the wasm bundle.
          </p>
        </>
      }
      caveat={
        <>
          <p>
            <strong className="text-foreground">Early / research, semi-honest.</strong>{" "}
            Confidentiality holds against honest-but-curious parties up to the
            threshold; tampering is detected and aborts where the (n, t) redundancy
            allows, but malicious security is not complete and dishonest-majority is
            future work. The collaborative ZK proof of correctness + source
            attestation is a <strong className="text-foreground">stub</strong>{" "}
            (returns <code className="font-mono">NotYetImplemented</code>) — this
            surface makes no correctness or attestation guarantee, and no external
            cryptographer has reviewed it.
          </p>
        </>
      }
      links={[{ href: "/showcase/mpc-100k", label: "★ Open the £100k flagship" }]}
    >
      <section className="space-y-2">
        <Button asChild variant="default" size="sm">
          <Link href="/showcase/mpc-100k">Try the interactive £100k demo →</Link>
        </Button>
      </section>
    </SurfaceContent>
  );
}
