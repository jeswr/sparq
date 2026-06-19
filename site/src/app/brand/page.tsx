import type { Metadata } from "next";
import Link from "next/link";
import { ArrowLeft } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Logo } from "@/components/logo";
import { LogoGallery } from "@/components/brand/logo-gallery";

// [OPUS-4.8] sq-jnh9 — shield + lightning logo CONCEPTS gallery for issue #207.
// Additive: it does NOT replace the live favicon/header Logo; it is a chooser the
// maintainer can eyeball in the exported site before picking a direction.
export const metadata: Metadata = {
  title: "Logo concepts",
  description:
    "Shield + lightning-bolt logo concepts for sparq (issue #207) — security + speed, in the teal brand palette. Concepts to choose from; they do not replace the current mark yet.",
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
          A fresh pass on the sparq mark around a{" "}
          <strong>shield fused with a lightning bolt</strong> — the two pillars of
          sparq, <strong>security</strong> (verifiable / ZK query proofs) and{" "}
          <strong>speed</strong> (a fast SPARQL engine). Every concept reuses the
          real site palette: the privacy-first teal brand token (the same{" "}
          <code>--primary</code> used across the site) and the established warm
          spark accent. Marks use <code>currentColor</code> where it helps, so they
          follow the light / dark theme; each is previewed below on both a light and
          a dark field and at favicon sizes.
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
          . Pick a direction (or mix and match a mark with a wordmark treatment) and
          it can be wired into the favicon + header in a follow-up.
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
