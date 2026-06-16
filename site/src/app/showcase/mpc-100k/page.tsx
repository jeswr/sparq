import type { Metadata } from "next";
import Link from "next/link";
import {
  ArrowLeft,
  Lock,
  Network,
  Split,
  Sigma,
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
import { MpcDemo } from "@/components/mpc-demo";
import { TIER_LABEL, TIER_VARIANT } from "@/data/surfaces";

export const metadata: Metadata = {
  title: "MPC £100k secure threshold",
  description:
    "Four flatmates learn only whether their combined income clears a £100k threshold — no salary, and not even the exact total, is revealed. A faithful in-tab illustration of the sparq-mpc secure-threshold protocol.",
};

export default function MpcFlagshipPage() {
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
            <Lock className="size-5" aria-hidden="true" />
          </span>
          <div>
            <h1 className="flex items-center gap-2 text-2xl font-semibold">
              <span className="text-[var(--warning)]" aria-hidden="true">
                ★
              </span>
              MPC £100k secure threshold
            </h1>
            <p className="text-sm text-muted-foreground">
              Can a group collectively afford it — without anyone revealing their
              salary?
            </p>
          </div>
        </div>
        <div className="flex flex-wrap gap-2">
          <Badge variant={TIER_VARIANT["live-sim"]}>{TIER_LABEL["live-sim"]}</Badge>
          <Badge variant="muted">Faithful illustration — not live MPC</Badge>
        </div>
      </header>

      {/* Narrative */}
      <section className="measure space-y-3 text-muted-foreground">
        <p>
          Four flatmates want to apply for a £100k mortgage together. The lender
          only needs to know one thing:{" "}
          <strong className="text-foreground">
            does their combined income clear £100,000?
          </strong>{" "}
          Nobody wants to disclose their salary to the others, and the group does
          not even want to reveal the exact total — only the yes/no answer.
        </p>
        <p>
          Secure multi-party computation (MPC) lets the parties jointly compute
          that single bit. Each party&rsquo;s income stays on their own device;
          what crosses the wire is a set of <em>random shares</em> that, on their
          own, reveal nothing. This mirrors the path tested in the native{" "}
          <code className="font-mono text-foreground">sparq-mpc</code> crate
          (30k + 28k + 26k + 24k = 108k → <strong className="text-foreground">true</strong>).
        </p>
      </section>

      {/* The interactive demo */}
      <section className="space-y-3">
        <h2 className="text-xl font-semibold">Try it</h2>
        <p className="text-sm text-muted-foreground">
          Set each party&rsquo;s private income and run the secure threshold. The
          page reveals only the verdict bit — watch the share matrix to see exactly
          what is exchanged.
        </p>
        <MpcDemo />
      </section>

      {/* How the protocol works */}
      <section className="space-y-4">
        <h2 className="text-xl font-semibold">How the protocol works</h2>
        <div className="grid gap-4 md:grid-cols-3">
          <FlowStep
            icon={Split}
            n={1}
            title="Secret-share"
            body="Each party splits its private value into N random shares whose sum equals the value. Any single share is a uniform random number — it leaks nothing. Each party keeps one share and sends the rest to its peers."
          />
          <FlowStep
            icon={Sigma}
            n={2}
            title="Add over shares"
            body="Addition is free on secret shares: every party locally sums the shares it holds. Combining those local sums yields a sharing of the grand total, without anyone ever seeing another party's value."
          />
          <FlowStep
            icon={Eye}
            n={3}
            title="Reveal one bit"
            body="A secure comparison opens only the boolean total ≥ £100k. The exact total is never reconstructed as an output — the lender learns one bit, and nothing else."
          />
        </div>
      </section>

      {/* What is shared vs private */}
      <section className="space-y-4">
        <h2 className="text-xl font-semibold">What is shared vs. what stays private</h2>
        <div className="grid gap-4 md:grid-cols-2">
          <Card>
            <CardHeader className="flex-row items-center gap-2 space-y-0">
              <Lock className="size-5 text-primary" aria-hidden="true" />
              <CardTitle className="text-base">Stays private</CardTitle>
            </CardHeader>
            <CardContent className="space-y-2 text-sm text-muted-foreground">
              <p>
                <strong className="text-foreground">Each party&rsquo;s income.</strong>{" "}
                It never leaves the device in the clear; only random shares are
                sent, and no party can see another&rsquo;s value.
              </p>
              <p>
                <strong className="text-foreground">The exact total.</strong> It is
                only ever held as a secret sharing and is never opened — only the
                threshold comparison&rsquo;s result is.
              </p>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="flex-row items-center gap-2 space-y-0">
              <Network className="size-5 text-[var(--success)]" aria-hidden="true" />
              <CardTitle className="text-base">Shared / revealed</CardTitle>
            </CardHeader>
            <CardContent className="space-y-2 text-sm text-muted-foreground">
              <p>
                <strong className="text-foreground">Random shares between peers.</strong>{" "}
                These cross the wire but are uniformly random — fewer than the
                reconstruction threshold of them reveal nothing.
              </p>
              <p>
                <strong className="text-foreground">The verdict bit.</strong> The
                single output: whether the combined income meets £100k. The public
                threshold itself is, of course, public.
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
              A faithful illustration — not live MPC, and not a security claim
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-3 text-sm text-muted-foreground">
            <p>
              This page is a{" "}
              <strong className="text-foreground">
                client-side illustration of the protocol shape
              </strong>
              , not the hardened crate running in your browser. The{" "}
              <code className="font-mono">sparq-mpc</code> crate is deliberately
              native-only and is absent from the wasm bundle, so nothing
              cryptographic runs here. The in-tab code uses simple additive
              (n-of-n) secret sharing with <code className="font-mono">Math.random</code>{" "}
              to make the &ldquo;shares reveal nothing&rdquo; and &ldquo;addition
              is free&rdquo; properties visible; the real crate uses honest-majority{" "}
              <strong className="text-foreground">Shamir t-of-n</strong> sharing and
              a bit-decomposition secure comparison.
            </p>
            <p>
              Even the native crate makes{" "}
              <strong className="text-foreground">no production security claim</strong>.
              It is honest-majority <strong className="text-foreground">semi-honest</strong>{" "}
              only — confidentiality holds against parties that follow the protocol
              but try to learn more, not against actively-cheating parties in every
              configuration. The collaborative zero-knowledge{" "}
              <em>proof of correctness and source-attestation</em> is a stub that
              returns <code className="font-mono">NotYetImplemented</code>. So this
              demo does <strong className="text-foreground">not</strong> prove the
              result is correct, only that the disclosure pattern is minimal.
            </p>
            <p>
              No external cryptographer has reviewed sparq&rsquo;s bespoke MPC. Per
              the project&rsquo;s published <code className="font-mono">SECURITY.md</code>{" "}
              posture, treat this as a research / explainer artifact, not a
              certified secure-computation system. The crypto-review readiness doc
              states it plainly: <code className="font-mono">sparq-mpc</code> carries
              no confidentiality, correctness, attestation, or malicious-security
              guarantee today.
            </p>
          </CardContent>
        </Card>
      </section>

      <section className="flex flex-wrap gap-2">
        <Button asChild variant="outline" size="sm">
          <Link href="/surface/mpc">
            <Info className="size-4" aria-hidden="true" />
            The MPC surface in depth
          </Link>
        </Button>
        <Button asChild variant="outline" size="sm">
          <a
            href="https://github.com/jeswr/sparq/tree/main/crates/sparq-mpc"
            target="_blank"
            rel="noopener noreferrer"
          >
            sparq-mpc source on GitHub
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
