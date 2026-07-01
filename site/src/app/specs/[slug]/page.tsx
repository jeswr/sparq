// [OPUS-4.8] sq-rvgr2.1 — per-spec page. Static-exported via generateStaticParams over
// src/data/specs.ts, mirroring papers/[slug]/page.tsx. Renders: the in-site ReSpec-style HTML
// (built from the same .typ as the PDF, read at build time from
// src/generated/specs/<slug>.html), a "Download PDF" button (basePath-prefixed static asset),
// and a short provenance footer. No client JS / ReSpec / Chromium.
import { readFileSync } from "node:fs";
import { join } from "node:path";

import type { Metadata } from "next";
import { notFound } from "next/navigation";
import Link from "next/link";
import { ArrowLeft, Download } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { SpecHtml } from "@/components/specs/spec-html";
import { SPECS, specBySlug, STATUS_LABEL, STATUS_VARIANT } from "@/data/specs";
import { withBasePath } from "@/lib/base-path";

export function generateStaticParams() {
  return SPECS.map((s) => ({ slug: s.slug }));
}

export const dynamicParams = false;

export async function generateMetadata({
  params,
}: {
  params: Promise<{ slug: string }>;
}): Promise<Metadata> {
  const { slug } = await params;
  const spec = specBySlug(slug);
  if (!spec) return {};
  return { title: spec.title, description: spec.blurb };
}

// Read the build-time-generated HTML fragment. If it is missing (e.g. a local dev run with no
// Typst installed produced only a placeholder), fall back to an honest notice.
function readSpecHtml(slug: string): string {
  try {
    return readFileSync(
      join(process.cwd(), "src", "generated", "specs", `${slug}.html`),
      "utf8",
    );
  } catch {
    return `<p class="spec-placeholder">This draft has not been built yet. Run <code>npm run build-specs</code> with the Typst CLI installed.</p>`;
  }
}

// basePath-aware PDF asset link (mirrors papers).
function pdfHref(slug: string): string {
  return withBasePath(`/specs/${slug}.pdf`);
}

export default async function SpecPage({
  params,
}: {
  params: Promise<{ slug: string }>;
}) {
  const { slug } = await params;
  const spec = specBySlug(slug);
  if (!spec) notFound();

  const html = readSpecHtml(slug);

  return (
    <div className="space-y-6">
      <Button variant="ghost" size="sm" asChild className="-ml-2">
        <Link href="/specs">
          <ArrowLeft className="size-4" aria-hidden />
          All specs
        </Link>
      </Button>

      <header className="space-y-3">
        <div className="flex flex-wrap items-center gap-2">
          <Badge variant={STATUS_VARIANT[spec.status]}>
            {STATUS_LABEL[spec.status]}
          </Badge>
          <span className="text-xs text-muted-foreground">
            <code>{spec.shortName}</code> · {spec.date}
          </span>
        </div>
        <div className="flex flex-wrap items-start justify-between gap-3">
          <p className="measure text-sm text-muted-foreground">{spec.blurb}</p>
          <Button asChild size="sm" className="shrink-0">
            <a href={pdfHref(spec.slug)} download>
              <Download className="size-4" aria-hidden />
              Download PDF
            </a>
          </Button>
        </div>
      </header>

      <SpecHtml html={html} />

      <footer className="mt-10 rounded-lg bg-muted/40 p-4 text-xs text-muted-foreground">
        This is an <strong className="text-foreground">Unofficial Proposal Draft</strong>{" "}
        published by the sparq project. It is not a W3C standard, is not on the W3C
        Recommendation track, and carries no official standing. It is a proposal for
        discussion. The in-site render above and the downloadable PDF are built from a single
        Typst source, so they cannot disagree.
      </footer>
    </div>
  );
}
