import type { Metadata } from "next";
import Link from "next/link";
import { ArrowLeft } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Logo } from "@/components/logo";
import { LogoGallery } from "@/components/brand/logo-gallery";

// [OPUS-4.8] sq-8pbx — shield + lightning logo CONCEPTS gallery for issue #207.
// ROUND 2: the maintainer rejected the entire first round (too generic). This is a
// fresh, higher-quality set with tighter silhouettes and chunkier favicon-legible
// bolts. Additive: it does NOT replace the live favicon/header Logo; it is a chooser
// the maintainer can eyeball in the exported site before picking a direction.
export const metadata: Metadata = {
  title: "Logo concepts",
  description:
    "Shield + lightning-bolt logo concepts for sparq (issue #207) — security + speed, in the teal brand palette. A fresh, higher-quality round; concepts to choose from, they do not replace the current mark yet.",
};

export default function BrandPage() {
  return (
    <div className="space-y-10">
      <Button variant="ghost" size="sm" asChild className="-ml-2">
        <Link href="/">
          <ArrowLeft className="size-4" />
          Back to overview
        </Link>
      </Button>

      <header className="space-y-3">
        <div className="flex flex-wrap items-center gap-2">
          <Badge variant="default">Logo concepts</Badge>
          <Badge variant="outline">issue #207</Badge>
        </div>
        <h1 className="text-2xl font-semibold tracking-tight">
          Shield + lightning logo concepts
        </h1>
        <p className="measure text-muted-foreground">
          A <strong>second, higher-quality round</strong> on the sparq mark around a{" "}
          <strong>shield fused with a lightning bolt</strong> — the two pillars of
          sparq, <strong>security</strong> (verifiable / ZK query proofs) and{" "}
          <strong>speed</strong> (a fast SPARQL engine). The first round was rejected
          for reading too generically; this set rebuilds each mark with a{" "}
          <strong>tighter, more modern silhouette</strong> (flatter shoulders /
          confident point, or a real app-tile squircle — not a floppy heraldic
          pentagon) and a <strong>chunky negative-space bolt</strong> that stays
          legible at 16px. Every concept uses the real site teal (the same{" "}
          <code>--primary</code> token, with an sRGB hex fallback for standalone
          export) and leans far less on the warm accent that cluttered the first
          round. Each is previewed below on both a light and a dark field and at
          favicon sizes.
        </p>
        <p className="measure text-sm text-muted-foreground">
          These are <strong>concepts to choose from</strong> — they do not replace
          the current favicon or header mark. They supersede the earlier options
          rejected on{" "}
          <a
            href="https://github.com/jeswr/sparq/issues/207"
            target="_blank"
            rel="noreferrer"
            className="underline underline-offset-2 hover:text-foreground"
          >
            issue #207
          </a>
          . Each mark was rasterised at 16 / 32 / 64 / 256px on light and dark fields
          during design and the geometry tuned against what the rendered pixels
          actually showed (no external image-generation API was available, so these
          are hand-authored SVGs). Pick a direction (or mix and match a mark with a
          wordmark treatment) and it can be wired into the favicon + header in a
          follow-up.
        </p>
      </header>

      <section className="space-y-4">
        <h2 className="text-lg font-semibold">For comparison — the current mark</h2>
        <p className="measure text-sm text-muted-foreground">
          The mark in the header today (a graph-node motif with a small spark). The
          concepts below take the shield + lightning direction instead.
        </p>
        <div className="flex w-fit items-center rounded-lg bg-white p-4 ring-1 ring-foreground/10">
          <Logo className="h-10 w-auto text-[#0f1c20]" />
        </div>
      </section>

      <section className="space-y-4">
        <h2 className="text-lg font-semibold">Concepts</h2>
        <LogoGallery />
      </section>
    </div>
  );
}
