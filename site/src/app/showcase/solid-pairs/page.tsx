// [OPUS-4.8] sq-4r4b — the Solid (user, app)-pair flagship page (3rd flagship).
// [OPUS-4.8] sq-vw3ax.6 — rebuilt DEMO-FIRST (research/website-redesign.md §4): minimal header
// → the demo immediately → one-line caption → the enforcement explanation + the full caveat
// behind a closed <details>. The honesty (the restriction is REAL but the access-control
// DECISION is precomputed; sparq-solid is a research track / authorization oracle, never the
// sole gate) is preserved inside the disclosure.
import type { Metadata } from "next";
import Link from "next/link";
import { ArrowLeft, ExternalLink, Info } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { SolidPairsDemo } from "@/components/solid-pairs-demo";
import { TIER_LABEL, TIER_VARIANT } from "@/data/surfaces";
import { withBasePath } from "@/lib/base-path";

export const metadata: Metadata = {
  title: "Solid (user, app)-pair result sets",
  description:
    "The same SPARQL query over the same Solid Pod returns different result sets per (user, application) pair — because WAC/ACP access control filters which named graphs each requester may read. A faithful, live-in-tab illustration of sparq-solid's graph-level, fail-closed enforcement.",
};

const REPO_URL = "https://github.com/sparq-org/sparq";

export default function SolidFlagshipPage() {
  return (
    <div className="space-y-6">
      <Button variant="ghost" size="sm" asChild className="-ml-2">
        <Link href="/examples">
          <ArrowLeft className="size-4" aria-hidden="true" />
          Back to examples
        </Link>
      </Button>

      {/* Minimal header: title + tier + the "decision precomputed, restriction live" qualifier. */}
      <header className="space-y-2">
        <h1 className="flex items-center gap-2 text-2xl font-semibold tracking-tight">
          <span className="text-[var(--warning)]" aria-hidden="true">
            ★
          </span>
          Solid (user, app)-pair result sets
        </h1>
        <div className="flex flex-wrap gap-2">
          <Badge variant={TIER_VARIANT["live"]}>{TIER_LABEL["live"]}</Badge>
          <Badge variant="muted">
            Access-control decision precomputed · restriction is live
          </Badge>
        </div>
      </header>

      {/* The demo, immediately. */}
      <SolidPairsDemo />

      {/* One-line caption. */}
      <p className="measure text-sm text-muted-foreground">
        Run the same SPARQL over the same Pod for two different (user, app) pairs and watch the
        answers diverge — the engine restricts each query to exactly the named graphs that pair
        is authorized to read, with the WAC/ACP grant that produced each result shown alongside.
      </p>

      {/* The enforcement explanation + the full caveat — behind a closed disclosure. */}
      <details className="rounded-xl border bg-card px-4 py-3 text-sm">
        <summary className="cursor-pointer select-none font-medium">
          How enforcement works &amp; full caveat
        </summary>
        <div className="mt-3 space-y-3 text-muted-foreground">
          <p>
            In Solid your data lives in <strong className="text-foreground">your Pod</strong>,
            and the rules for who may read what can depend not just on <em>who</em> is asking
            but on <strong className="text-foreground">which application</strong> they ask
            from. sparq models a Pod as{" "}
            <strong className="text-foreground">one named graph per document</strong> and
            reduces enforcement to a dataset restriction: for a Session {`{ agent, client }`},
            sparq-solid materializes exactly the named graphs that pair may read, then evaluates
            the query only over that authorized set via a{" "}
            <code className="font-mono">FROM NAMED</code> clause (its{" "}
            <code className="font-mono">rewrite_for</code> path). The default is fail-closed:
            absence of a grant is denial, and a non-authorized graph is indistinguishable from
            absent — an ungranted requester gets the empty set, so the Pod looks empty to it.
          </p>
          <p>
            <strong className="text-foreground">What is real, and what is precomputed.</strong>{" "}
            The <strong className="text-foreground">restriction you see is real</strong>: the
            Pod is loaded into the sparq wasm engine in your tab with its named graphs
            preserved, and the same query is rewritten per pair — the differing result sets are
            the real engine doing the real graph-level restriction. What is{" "}
            <strong className="text-foreground">precomputed</strong> is the access-control{" "}
            <em>decision</em> (which named graphs each pair may read): the WAC/ACP
            materialization runs natively / at build time, not in the browser, and ships here as
            fixtures (the rules behind each are shown in the &ldquo;why these graphs?&rdquo;
            panels, so nothing is hidden).
          </p>
          <p>
            Most importantly, <code className="font-mono">sparq-solid</code> is a{" "}
            <strong className="text-foreground">research track, not a Solid-server
            cutover</strong>. Per the project house rule (issue #55), security-critical paths
            run through vetted TypeScript in the production Solid server; sparq-solid is at most
            an authorization <strong className="text-foreground">oracle</strong>, never the sole
            gate. It does not authenticate — it answers a graph-set question for an
            already-resolved (WebID, client) Session. This demo shows the graph-level
            access-control <em>capability and semantics</em>, not a certified production Solid
            deployment.
          </p>
          <p className="flex flex-wrap gap-4">
            <a
              className="inline-flex items-center gap-1 text-primary underline-offset-4 hover:underline"
              href={withBasePath("/app/")}
            >
              <Info className="size-3.5" aria-hidden="true" />
              The SPARQL workbench
            </a>
            <a
              className="inline-flex items-center gap-1 text-primary underline-offset-4 hover:underline"
              href={`${REPO_URL}/tree/main/crates/sparq-solid`}
              target="_blank"
              rel="noopener noreferrer"
            >
              sparq-solid on GitHub <ExternalLink className="size-3.5" />
            </a>
            <a
              className="inline-flex items-center gap-1 text-primary underline-offset-4 hover:underline"
              href={`${REPO_URL}/issues/55`}
              target="_blank"
              rel="noopener noreferrer"
            >
              Issue #55 — scope &amp; house rule <ExternalLink className="size-3.5" />
            </a>
          </p>
        </div>
      </details>
    </div>
  );
}
