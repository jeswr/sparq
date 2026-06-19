// [OPUS-4.8] sq-4r4b — the Solid (user, app)-pair flagship page (3rd flagship).
import type { Metadata } from "next";
import Link from "next/link";
import {
  ArrowLeft,
  Network,
  Layers,
  KeyRound,
  ShieldOff,
  Eye,
  Info,
  TriangleAlert,
} from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { SolidPairsDemo } from "@/components/solid-pairs-demo";
import { TIER_LABEL, TIER_VARIANT } from "@/data/surfaces";

export const metadata: Metadata = {
  title: "Solid (user, app)-pair result sets",
  description:
    "The same SPARQL query over the same Solid Pod returns different result sets per (user, application) pair — because WAC/ACP access control filters which named graphs each requester may read. A faithful, live-in-tab illustration of sparq-solid's graph-level, fail-closed enforcement.",
};

export default function SolidFlagshipPage() {
  return (
    <div className="space-y-10">
      <Button variant="ghost" size="sm" asChild className="-ml-2">
        <Link href="/">
          <ArrowLeft className="size-4" aria-hidden="true" />
          Back to overview
        </Link>
      </Button>

      {/* Hero */}
      <header className="space-y-3">
        <div className="flex items-center gap-3">
          <span className="flex size-11 items-center justify-center rounded-xl bg-primary/10 text-primary">
            <Network className="size-5" aria-hidden="true" />
          </span>
          <div>
            <h1 className="flex items-center gap-2 text-2xl font-semibold">
              <span className="text-[var(--warning)]" aria-hidden="true">
                ★
              </span>
              Solid (user, app)-pair result sets
            </h1>
            <p className="text-sm text-muted-foreground">
              The same query over the same Pod — different answers for different
              requesters.
            </p>
          </div>
        </div>
        <div className="flex flex-wrap gap-2">
          <Badge variant={TIER_VARIANT["live"]}>{TIER_LABEL["live"]}</Badge>
          <Badge variant="muted">
            Access-control decision precomputed · restriction is live
          </Badge>
        </div>
      </header>

      {/* Narrative */}
      <section className="measure space-y-3 text-muted-foreground">
        <p>
          In Solid, your data lives in <strong className="text-foreground">your Pod</strong>,
          and you decide who may read what. The same resource can be public,
          shared with a named friend, or locked to you alone — and the rules can
          depend not just on <em>who</em> is asking but on{" "}
          <strong className="text-foreground">which application</strong> they are
          asking from.
        </p>
        <p>
          So here is the headline insight:{" "}
          <strong className="text-foreground">
            run the SAME SPARQL query over the SAME Pod, and the result set you get
            back depends on the (user, application) pair
          </strong>{" "}
          — because WAC/ACP access control filters which documents each requester
          may see. sparq models a Pod as{" "}
          <strong className="text-foreground">one named graph per document</strong>{" "}
          and reduces enforcement to a dataset restriction: a query only ever runs
          over the named graphs that pair is authorized to read.
        </p>
        <p>
          This page demonstrates that with a small fixed Pod — a public profile,
          private health data, a shared calendar, and private notes — and a
          selector for (user, app) pairs. Pick two pairs, run a query (the demo
          one, or your own), and watch the answers diverge, with the WAC/ACP grant
          that produced each result shown alongside it.
        </p>
      </section>

      {/* The interactive demo */}
      <section className="space-y-3">
        <h2 className="text-xl font-semibold">Try it</h2>
        <p className="text-sm text-muted-foreground">
          The query is the same for both pairs on a given run &mdash; and you can{" "}
          <strong className="text-foreground">edit it and re-run</strong>. What differs
          is the authorized named-graph set the engine restricts to: the materialized
          access-control decision for that (agent, client), injected as a{" "}
          <code className="font-mono">FROM NAMED</code> clause around whatever you type.
        </p>
        <SolidPairsDemo />
      </section>

      {/* How enforcement works */}
      <section className="space-y-4">
        <h2 className="text-xl font-semibold">How the enforcement works</h2>
        <div className="grid gap-4 md:grid-cols-3">
          <FlowStep
            icon={Layers}
            n={1}
            title="One named graph per document"
            body="A Pod loads as a dataset where each document is a named graph whose name is the resource IRI. WAC .acl / ACP .acr documents are themselves queryable graphs. Access control lives at the graph level."
          />
          <FlowStep
            icon={KeyRound}
            n={2}
            title="Materialize the authorized set"
            body="For a Session { agent, client }, sparq-solid runs N3 rules over the WAC/ACP rules to compute exactly the named graphs that pair may read. The session key is the (agent, client) pair — the same agent from a different app can get a different set."
          />
          <FlowStep
            icon={Eye}
            n={3}
            title="Restrict the query (FROM NAMED)"
            body="The same query is evaluated only over that authorized set, via a FROM NAMED dataset restriction (sparq-solid's rewrite_for). Graphs outside the set are simply absent from the query's dataset — so the rows that come back differ per requester."
          />
        </div>
      </section>

      {/* What differs vs what stays */}
      <section className="space-y-4">
        <h2 className="text-xl font-semibold">
          Fail-closed: no grant means no data
        </h2>
        <div className="grid gap-4 md:grid-cols-2">
          <Card>
            <CardHeader className="flex-row items-center gap-2 space-y-0">
              <KeyRound className="size-5 text-primary" aria-hidden="true" />
              <CardTitle className="text-base">What a pair can see</CardTitle>
            </CardHeader>
            <CardContent className="space-y-2 text-sm text-muted-foreground">
              <p>
                <strong className="text-foreground">Only graphs with a matching grant.</strong>{" "}
                The authorized set is an allow-list: a document is visible only if
                some WAC/ACP rule grants this (agent, client) pair read access to it.
              </p>
              <p>
                <strong className="text-foreground">App-scoped grants are honoured.</strong>{" "}
                A grant scoped to one application (WAC <code className="font-mono">acl:origin</code>,
                ACP <code className="font-mono">acp:client</code>) is minted as a
                pair principal — so the owner from her trusted health app sees data
                the owner from a different app does not.
              </p>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="flex-row items-center gap-2 space-y-0">
              <ShieldOff className="size-5 text-[var(--warning)]" aria-hidden="true" />
              <CardTitle className="text-base">What stays invisible</CardTitle>
            </CardHeader>
            <CardContent className="space-y-2 text-sm text-muted-foreground">
              <p>
                <strong className="text-foreground">Anything with no applicable grant.</strong>{" "}
                Absence of a grant is denial — the default is fail-closed (design
                decision D4). A non-authorized graph is{" "}
                <strong className="text-foreground">indistinguishable from absent</strong>:
                the requester cannot even tell it exists.
              </p>
              <p>
                <strong className="text-foreground">An empty authorized set ⇒ zero rows.</strong>{" "}
                An anonymous or ungranted (agent, client) gets the empty set, so the
                query is restricted to a guaranteed-absent graph and returns nothing
                — the Pod looks empty to it.
              </p>
            </CardContent>
          </Card>
        </div>
      </section>

      {/* HONESTY — mandatory framing */}
      <section className="space-y-3">
        <h2 className="text-xl font-semibold">Honesty: what this demo is, and is not</h2>
        <Card className="ring-[var(--warning)]/30">
          <CardHeader className="flex-row items-center gap-2 space-y-0">
            <TriangleAlert
              className="size-5 text-[var(--warning)]"
              aria-hidden="true"
            />
            <CardTitle className="text-base">
              A capability illustration — not a production Solid server
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-3 text-sm text-muted-foreground">
            <p>
              The <strong className="text-foreground">restriction you see is real</strong>:
              the Pod is loaded into the sparq wasm engine in your tab with its named
              graphs preserved, and the same query is rewritten per pair with a{" "}
              <code className="font-mono">FROM NAMED</code> dataset clause — exactly
              sparq-solid&rsquo;s <code className="font-mono">rewrite_for</code> /{" "}
              <code className="font-mono">query_as_rewrite</code> path. The differing
              result sets are the real engine doing the real graph-level restriction.
            </p>
            <p>
              What is <strong className="text-foreground">precomputed</strong> is the
              access-control <em>decision</em> — which named graphs each (agent,
              client) pair is authorized to read. In the real system the WAC/ACP
              materialization (the <code className="font-mono">sparq-solid</code>{" "}
              crate&rsquo;s N3 reasoner running over the <code className="font-mono">.acl</code>/
              <code className="font-mono">.acr</code> rules) runs natively / at build
              time, not in the browser. This page ships those authorized sets as
              fixtures and applies them live; the WAC/ACP rules behind each one are
              shown in the &ldquo;why these graphs?&rdquo; panels so nothing is hidden.
            </p>
            <p>
              Most importantly, <code className="font-mono">sparq-solid</code> is a{" "}
              <strong className="text-foreground">research track, not a Solid-server
              cutover</strong>. Per the project&rsquo;s house rule (issue #55):{" "}
              <em>
                security-critical paths run through vetted TypeScript in the
                production Solid server; sparq-solid is at most an authorization{" "}
                <strong className="text-foreground">oracle</strong>, never the sole
                gate
              </em>
              . It does not authenticate — it takes an already-resolved (WebID,
              client) Session and answers a graph-set question. HTTP status semantics
              (401 vs 403, <code className="font-mono">WAC-Allow</code>), Solid-OIDC /
              DPoP, and the request lifecycle all stay in the vetted server. This demo
              shows sparq&rsquo;s graph-level access-control <em>capability and
              semantics</em>, not a certified production Solid deployment.
            </p>
          </CardContent>
        </Card>
      </section>

      <section className="flex flex-wrap gap-2">
        <Button asChild variant="outline" size="sm">
          <Link href="/try">
            <Info className="size-4" aria-hidden="true" />
            The live SPARQL REPL
          </Link>
        </Button>
        <Button asChild variant="outline" size="sm">
          <a
            href="https://github.com/jeswr/sparq/tree/main/crates/sparq-solid"
            target="_blank"
            rel="noopener noreferrer"
          >
            sparq-solid source on GitHub
          </a>
        </Button>
        <Button asChild variant="outline" size="sm">
          <a
            href="https://github.com/jeswr/sparq/issues/55"
            target="_blank"
            rel="noopener noreferrer"
          >
            Issue #55 — scope &amp; house rule
          </a>
        </Button>
      </section>
    </div>
  );
}

function FlowStep({
  icon: Icon,
  n,
  title,
  body,
}: {
  icon: React.ComponentType<{ className?: string }>;
  n: number;
  title: string;
  body: string;
}) {
  return (
    <Card className="h-full">
      <CardHeader className="gap-2">
        <div className="flex items-center gap-2">
          <span className="flex size-9 items-center justify-center rounded-lg bg-primary/10 text-primary">
            <Icon className="size-4.5" aria-hidden="true" />
          </span>
          <Badge variant="muted">Step {n}</Badge>
        </div>
        <CardTitle className="text-base">{title}</CardTitle>
      </CardHeader>
      <CardContent>
        <CardDescription className="leading-relaxed">{body}</CardDescription>
      </CardContent>
    </Card>
  );
}
