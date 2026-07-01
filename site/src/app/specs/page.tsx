// [OPUS-4.8] sq-rvgr2.1 — /specs index, mirroring papers/page.tsx. One card per registered
// draft (data-driven off src/data/specs.ts), with a status badge, short name, date, the
// blurb, a link to the per-spec page, and a direct PDF link. An honest framing line up top
// states that every draft is an Unofficial Proposal Draft with no W3C standing.
import type { Metadata } from "next";
import Link from "next/link";
import { ArrowRight, FileText, Download, ScrollText } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { SPECS, STATUS_LABEL, STATUS_VARIANT } from "@/data/specs";
import { withBasePath } from "@/lib/base-path";

export const metadata: Metadata = {
  title: "Specs",
  description:
    "sparq's Proposed Specifications — W3C ReSpec-style Unofficial Proposal Drafts of sparq's novel interfaces. Each is authored once and built into both an in-site render and a downloadable PDF. These are proposals for discussion; none is a W3C standard.",
};

// basePath-aware PDF asset link (mirrors papers). Resolves under both the Pages `/sparq`
// prefix and the Tauri root-relative build.
function pdfHref(slug: string): string {
  return withBasePath(`/specs/${slug}.pdf`);
}

export default function SpecsIndexPage() {
  return (
    <div className="space-y-8">
      <header className="space-y-3">
        <div className="flex items-center gap-3">
          <span className="flex size-11 items-center justify-center rounded-xl bg-primary/10 text-primary">
            <ScrollText className="size-5" aria-hidden />
          </span>
          <div>
            <h1 className="text-2xl font-semibold">Specs</h1>
            <p className="text-sm text-muted-foreground">
              Proposed Specifications — W3C ReSpec-style Unofficial Proposal Drafts of sparq&apos;s
              novel interfaces, each with a downloadable PDF.
            </p>
          </div>
        </div>
      </header>

      <section className="measure space-y-3 text-sm text-muted-foreground">
        <p>
          Each draft is authored once as a Typst source and built into both the in-site render
          and a downloadable PDF, so the two cannot disagree. These are{" "}
          <strong className="text-foreground">Unofficial Proposal Drafts</strong>: proposals
          published for discussion in a familiar specification form. None is a W3C standard,
          none is on the W3C Recommendation track, and none carries any official standing — the
          Status-of-This-Document notice on each page states this plainly.
        </p>
      </section>

      <section className="grid gap-4">
        {SPECS.map((s) => (
          <Card key={s.slug} className="transition-colors hover:ring-foreground/20">
            <CardHeader>
              <div className="flex flex-wrap items-center gap-2">
                <Badge variant={STATUS_VARIANT[s.status]}>
                  {STATUS_LABEL[s.status]}
                </Badge>
                <span className="text-xs text-muted-foreground">
                  <code>{s.shortName}</code> · {s.date}
                </span>
              </div>
              <CardTitle className="text-base leading-snug">{s.title}</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              <CardDescription className="leading-relaxed">{s.blurb}</CardDescription>
              <div className="flex flex-wrap items-center gap-4 pt-1">
                <Link
                  href={`/specs/${s.slug}`}
                  className="inline-flex items-center gap-1 text-sm text-primary"
                >
                  <FileText className="size-3.5" aria-hidden />
                  Read draft
                  <ArrowRight className="size-3.5" aria-hidden />
                </Link>
                <a
                  href={pdfHref(s.slug)}
                  className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
                >
                  <Download className="size-3.5" aria-hidden />
                  PDF
                </a>
              </div>
            </CardContent>
          </Card>
        ))}
      </section>
    </div>
  );
}
