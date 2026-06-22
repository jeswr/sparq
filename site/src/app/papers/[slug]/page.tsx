// [OPUS-4.8] sq-gum8 — per-paper page. Static-exported via generateStaticParams over
// src/data/papers.ts. Renders: the in-site HTML (built from the same .typ + evidence as the
// PDF, read at build time from src/generated/papers/<slug>.html), a "Download PDF" button
// (basePath-prefixed static asset), and a provenance stamp. No client JS / WASM compiler.
import { readFileSync } from "node:fs";
import { join } from "node:path";

import type { Metadata } from "next";
import { notFound } from "next/navigation";
import Link from "next/link";
import { ArrowLeft, Download } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { PaperHtml } from "@/components/papers/paper-html";
import {
  PaperProvenance,
  type Provenance,
} from "@/components/papers/paper-provenance";
import {
  PAPERS,
  paperBySlug,
  STATUS_LABEL,
  STATUS_VARIANT,
  FAMILY_LABEL,
} from "@/data/papers";
import { LATEST, GENERATED_AT } from "@/data/benchmarks";
import evidence from "@/data/paper-evidence.json";
import { withBasePath } from "@/lib/base-path";

export function generateStaticParams() {
  return PAPERS.map((p) => ({ slug: p.slug }));
}

export const dynamicParams = false;

export async function generateMetadata({
  params,
}: {
  params: Promise<{ slug: string }>;
}): Promise<Metadata> {
  const { slug } = await params;
  const paper = paperBySlug(slug);
  if (!paper) return {};
  return { title: paper.title, description: paper.blurb };
}

// Read the build-time-generated HTML fragment. If it is missing (e.g. a local dev run with
// no Typst installed produced only a placeholder), fall back to an honest notice.
function readPaperHtml(slug: string): string {
  try {
    return readFileSync(
      join(process.cwd(), "src", "generated", "papers", `${slug}.html`),
      "utf8",
    );
  } catch {
    return `<p>This paper has not been built yet. Run <code>npm run build-papers</code> with the Typst CLI installed.</p>`;
  }
}

function provenanceFor(): Provenance {
  const records = Object.values(
    (evidence as { records: Record<string, { environment: string }> }).records,
  );
  const canonical = records.filter((r) => r.environment === "canonical").length;
  return {
    commit: LATEST.commit ? LATEST.commit.slice(0, 8) : "unknown",
    generatedAt: GENERATED_AT ?? "",
    canonical,
    indicative: records.length - canonical,
  };
}

// basePath-aware PDF asset link. [OPUS-4.8] sq-9vw5 — env-switched (was hardcoded `/sparq`)
// so the PDF resolves under both the Pages `/sparq` prefix and the Tauri root-relative build.
function pdfHref(slug: string): string {
  return withBasePath(`/papers/${slug}.pdf`);
}

export default async function PaperPage({
  params,
}: {
  params: Promise<{ slug: string }>;
}) {
  const { slug } = await params;
  const paper = paperBySlug(slug);
  if (!paper) notFound();

  const html = readPaperHtml(slug);
  const prov = provenanceFor();

  return (
    <div className="space-y-6">
      <Button variant="ghost" size="sm" asChild className="-ml-2">
        <Link href="/papers">
          <ArrowLeft className="size-4" aria-hidden />
          All papers
        </Link>
      </Button>

      <header className="space-y-3">
        <div className="flex flex-wrap items-center gap-2">
          <Badge variant={STATUS_VARIANT[paper.status]}>
            {STATUS_LABEL[paper.status]}
          </Badge>
          <Badge variant="muted">{FAMILY_LABEL[paper.family]}</Badge>
          <span className="text-xs text-muted-foreground">{paper.venue}</span>
        </div>
        <div className="flex flex-wrap items-start justify-between gap-3">
          <p className="measure text-sm text-muted-foreground">{paper.blurb}</p>
          <Button asChild size="sm" className="shrink-0">
            <a href={pdfHref(paper.slug)} download>
              <Download className="size-4" aria-hidden />
              Download PDF
            </a>
          </Button>
        </div>
      </header>

      <PaperHtml html={html} />

      <PaperProvenance prov={prov} />
    </div>
  );
}
