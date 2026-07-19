import type { Metadata } from "next";
import Link from "next/link";
import { ArrowLeft, ExternalLink, Info } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { MpcDemo } from "@/components/mpc-demo";
import { TIER_LABEL, TIER_VARIANT } from "@/data/surfaces";

// [OPUS-4.8] sq-13rg / sq-4r4b — the MPC £100k secure-threshold flagship.
// [OPUS-4.8] sq-vw3ax.6 — rebuilt DEMO-FIRST (research/website-redesign.md §4): minimal header
// → the demo immediately → one-line caption → the protocol explanation + the FULL security
// caveat behind a closed <details>. The privacy qualifiers (faithful SIMULATION, not the
// hardened crate, NOT a proof of correctness — the correctness layer is a stub, NOT externally
// audited, semi-honest / honest-majority only) are PRESERVED verbatim inside the disclosure and
// summarised in the visible header badge (privacy-claims gate).
export const metadata: Metadata = {
  title: "MPC £100k secure threshold",
  description:
    "Four flatmates learn only whether their combined income clears a £100k threshold — no salary, and not even the exact total, is revealed. A faithful in-tab illustration of the sparq-mpc secure-threshold protocol — not the hardened crate, and not a proof of correctness.",
};

const REPO_URL = "https://github.com/sparq-org/sparq";

export default function MpcFlagshipPage() {
  return (
    <div className="space-y-6">
      <Button variant="ghost" size="sm" asChild className="-ml-2">
        <Link href="/examples">
          <ArrowLeft className="size-4" aria-hidden="true" />
          Back to examples
        </Link>
      </Button>

      {/* Minimal header: title + tier + the "faithful illustration" qualifier badge. */}
      <header className="space-y-2">
        <h1 className="flex items-center gap-2 text-2xl font-semibold tracking-tight">
          <span className="text-[var(--warning)]" aria-hidden="true">
            ★
          </span>
          MPC £100k secure threshold
        </h1>
        <div className="flex flex-wrap gap-2">
          <Badge variant={TIER_VARIANT["live-sim"]}>{TIER_LABEL["live-sim"]}</Badge>
          <Badge variant="warning">
            Faithful illustration · not live MPC · not externally audited
          </Badge>
        </div>
      </header>

      {/* The demo, immediately. */}
      <MpcDemo />

      {/* One-line caption. */}
      <p className="measure text-sm text-muted-foreground">
        Set each party&rsquo;s private income and run the secure threshold: the page reveals
        only the yes/no verdict bit — not any salary, and not even the exact total — and the
        share matrix shows exactly what crosses the wire.
      </p>

      {/* The protocol explanation + the FULL security caveat — behind a closed disclosure. */}
      <details className="rounded-xl border bg-card px-4 py-3 text-sm">
        <summary className="cursor-pointer select-none font-medium">
          How it works &amp; full security caveat
        </summary>
        <div className="mt-3 space-y-3 text-muted-foreground">
          <p>
            Four flatmates want a £100k mortgage together. The lender needs only one bit:{" "}
            <strong className="text-foreground">
              does their combined income clear £100,000?
            </strong>{" "}
            Secure multi-party computation lets them jointly compute that bit without anyone
            disclosing a salary. Each party splits its value into{" "}
            <strong className="text-foreground">random shares</strong> (any single share is a
            uniform random number that leaks nothing), addition is free over shares, and a
            secure comparison opens only the boolean total ≥ £100k — never the total itself.
            This mirrors the native <code className="font-mono">sparq-mpc</code> crate
            (30k + 28k + 26k + 24k = 108k →{" "}
            <strong className="text-foreground">true</strong>).
          </p>
          <p>
            <strong className="text-foreground">What this demo is, and is not.</strong> This
            page is a{" "}
            <strong className="text-foreground">
              client-side illustration of the protocol shape
            </strong>
            , not the hardened crate running in your browser — the{" "}
            <code className="font-mono">sparq-mpc</code> crate is deliberately native-only and
            absent from the wasm bundle, so nothing cryptographic runs here. The in-tab code
            uses simple additive (n-of-n) secret sharing with{" "}
            <code className="font-mono">Math.random</code> to make the &ldquo;shares reveal
            nothing&rdquo; / &ldquo;addition is free&rdquo; properties visible; the real crate
            uses honest-majority{" "}
            <strong className="text-foreground">Shamir t-of-n</strong> sharing and a
            bit-decomposition secure comparison.
          </p>
          <p>
            Even the native crate makes{" "}
            <strong className="text-foreground">no production security claim</strong>. It is
            honest-majority <strong className="text-foreground">semi-honest</strong> only —
            confidentiality holds against parties that follow the protocol but try to learn
            more, <strong className="text-foreground">not</strong> against actively-cheating
            (malicious) parties, and assumes an honest majority. The collaborative
            zero-knowledge <em>proof of correctness and source-attestation</em> layer is a{" "}
            <strong className="text-foreground">stub</strong> that returns{" "}
            <code className="font-mono">NotYetImplemented</code> (beads{" "}
            <code className="font-mono">sq-9hrn</code> / <code className="font-mono">sq-qhy4</code>),
            so this demo does <strong className="text-foreground">not</strong> prove the result
            is correct — there is no soundness guarantee, only that the disclosure pattern is
            minimal under the semi-honest, honest-majority assumption. No external
            cryptographer has reviewed sparq&rsquo;s bespoke MPC; treat it as a research /
            explainer artifact, not a certified secure-computation system.
          </p>
          <p className="flex flex-wrap gap-4">
            <Link
              className="inline-flex items-center gap-1 text-primary underline-offset-4 hover:underline"
              href="/capabilities#privacy"
            >
              <Info className="size-3.5" aria-hidden="true" />
              The MPC surface in depth
            </Link>
            <a
              className="inline-flex items-center gap-1 text-primary underline-offset-4 hover:underline"
              href={`${REPO_URL}/tree/main/crates/sparq-mpc`}
              target="_blank"
              rel="noopener noreferrer"
            >
              sparq-mpc on GitHub <ExternalLink className="size-3.5" />
            </a>
          </p>
        </div>
      </details>
    </div>
  );
}
