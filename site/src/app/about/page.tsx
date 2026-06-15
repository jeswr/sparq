import type { Metadata } from "next";

import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { ALL_SURFACES, TIER_LABEL, TIER_VARIANT } from "@/data/surfaces";

export const metadata: Metadata = {
  title: "About",
  description:
    "What sparq is, and the honest \"what runs where\" matrix — which surfaces run live in your tab, which are hosted, and which are captured-I/O walkthroughs.",
};

export default function AboutPage() {
  return (
    <div className="space-y-10">
      <header className="space-y-3">
        <h1 className="text-2xl font-semibold">About this showcase</h1>
        <p className="measure text-muted-foreground">
          sparq is a state-of-the-art Rust RDF triplestore and SPARQL 1.1/1.2
          engine, with a browser WASM port. This site exists to demonstrate its
          feature surfaces with <strong>real engine output</strong>, never mocks.
          The design principle is honesty about the seam: every surface declares
          exactly how it runs.
        </p>
      </header>

      <section className="space-y-4">
        <h2 className="text-xl font-semibold">What runs where</h2>
        <p className="measure text-sm text-muted-foreground">
          Only the core parser + triplestore + SPARQL engine + the four text
          formats are in the shipped lean wasm bundle today. &ldquo;Live
          everywhere&rdquo; is achieved by a three-tier strategy — the lean
          bundle, new optional wasm bundles lazy-loaded per page, and a small
          hosted server / 3rd-party prover where a wasm rebuild is uneconomic —
          plus an in-tab simulation for MPC. Every remaining page degrades to a
          guided walkthrough with real captured I/O.
        </p>
        <div className="overflow-x-auto rounded-xl ring-1 ring-foreground/10">
          <table className="w-full text-left text-sm">
            <thead className="bg-muted/50">
              <tr>
                <th className="px-4 py-2.5 font-medium">Surface</th>
                <th className="px-4 py-2.5 font-medium">How it runs</th>
                <th className="px-4 py-2.5 font-medium">What it does</th>
              </tr>
            </thead>
            <tbody>
              {ALL_SURFACES.map((s) => (
                <tr key={s.slug} className="border-t">
                  <td className="px-4 py-2.5 font-medium">{s.title}</td>
                  <td className="px-4 py-2.5">
                    <Badge variant={TIER_VARIANT[s.tier]}>
                      {TIER_LABEL[s.tier]}
                    </Badge>
                  </td>
                  <td className="px-4 py-2.5 text-muted-foreground">{s.blurb}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      <section className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Honesty notes</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 text-sm text-muted-foreground">
            <p>
              The ZK verifier is research-grade (v1), sound as landed under its
              stated threat model, and pending re-review. Do not treat the ZK
              demos as production-certified.
            </p>
            <p>
              The MPC demo is a faithful in-tab illustration of the protocol the
              native <code className="font-mono">sparq-mpc</code> crate runs — not
              the hardened crate executing in your browser, and not a proof of
              correctness (that layer is a stub today).
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Links</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 text-sm">
            <p>
              <a
                className="text-primary underline-offset-4 hover:underline"
                href="https://github.com/jeswr/sparq"
                target="_blank"
                rel="noopener noreferrer"
              >
                Source on GitHub
              </a>
            </p>
            <p>
              <a
                className="text-primary underline-offset-4 hover:underline"
                href="https://jeswr.github.io/sparq/dev/bench/"
                target="_blank"
                rel="noopener noreferrer"
              >
                Benchmark dashboard
              </a>
            </p>
          </CardContent>
        </Card>
      </section>
    </div>
  );
}
