import type { Metadata } from "next";
import Link from "next/link";
import { ArrowLeft } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ZkCarHire } from "@/components/zk-car-hire";

// [OPUS-4.8] sq-13rg / sq-4r4b — the ZK cross-credential car-hire flagship.
export const metadata: Metadata = {
  title: "ZK car-hire — prove eligibility, reveal nothing",
  description:
    "Prove you may hire a car — age ≥ 25 over hidden credential data — with a real zero-knowledge proof generated in your browser via Noir + Barretenberg UltraHonk. The exact age is never disclosed.",
};

export default function ZkCarHirePage() {
  return (
    <div className="space-y-8">
      <Button variant="ghost" size="sm" asChild className="-ml-2">
        <Link href="/">
          <ArrowLeft className="size-4" />
          Back to overview
        </Link>
      </Button>

      <header className="space-y-3">
        <div className="flex items-center gap-2">
          <Badge variant="success">Live via bb.js — real proof in your tab</Badge>
          <Badge variant="warning">Research-grade · not externally audited</Badge>
        </div>
        <h1 className="flex items-center gap-2 text-2xl font-semibold tracking-tight">
          <span className="text-[var(--warning)]" aria-hidden>
            ★
          </span>
          Prove you may hire a car — reveal nothing
        </h1>
        <p className="measure text-muted-foreground">
          A renter holds two W3C Verifiable Credentials — a gov-ID and a DVLA
          driving licence. The car-hire desk must learn only that the renter is{" "}
          <strong>≥ 25</strong>, holds a <strong>valid licence</strong>, and that
          both credentials belong to the <strong>same person</strong> — and nothing
          else: not the date of birth, not the licence number, not the identity.
          The age-gate below proves <em>age ≥ 25</em> over a hidden age with a real
          UltraHonk zero-knowledge proof, generated and verified entirely in your
          browser.
        </p>
      </header>

      <ZkCarHire />
    </div>
  );
}
