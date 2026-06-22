"use client";

// [OPUS-4.8] sq-vw3ax.3 — one ~64px gallery row in /capabilities.
//
// Three shapes (data/capabilities.ts `classify`):
//   * deep  → a plain link "Open →" to the surface's retained /surface/<slug> deep page.
//   * demo  → an expand-in-place disclosure "Demo ▸" that LAZILY mounts the surface's existing
//             interactive component (LazyDemo) only on first expand, plus a "Details & caveats"
//             <details> holding ONE caveat sentence + README + SKILL links.
//   * soon  → a non-interactive "Coming soon" row that links out to GitHub.
//
// The lazy-mount is the load-bearing requirement: `mounted` flips true on first expand and the
// demo chunk is fetched only then (see lazy-demo.tsx). Collapsing keeps `mounted` true so a
// re-expand never re-fetches; we hide the body with CSS rather than unmounting.

import * as React from "react";
import Link from "next/link";
import { ArrowRight, ChevronRight, ExternalLink } from "lucide-react";

import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import { TIER_LABEL, TIER_VARIANT, type Surface } from "@/data/surfaces";
import { surfaceBySlug } from "@/data/capabilities";
import {
  LazyDemo,
  hasLazyDemo,
  type LazyDemoSlug,
} from "@/components/capabilities/lazy-demo";

const REPO_URL = "https://github.com/jeswr/sparq";

interface RowMeta {
  /** ONE caveat sentence (honesty preserved; the full hedge lives in the SKILL/README). */
  caveat: string;
  /** Crate / source link (GitHub). */
  readme: string;
  /** SKILL.md link (GitHub). */
  skill?: string;
}

/** Per-demo one-line caveat + depth links — the honesty kept terse (research §3 COLLAPSE). */
const META: Record<LazyDemoSlug, RowMeta> = {
  geosparql: {
    caveat:
      "Captured-output walkthrough: sparq-geo is an opt-in native crate, so the static site replays real, verbatim engine output rather than running it live; planar DE-9IM, locally-exact metric distance.",
    readme: `${REPO_URL}/tree/main/crates/sparq-geo`,
    skill: `${REPO_URL}/blob/main/skills/geosparql/SKILL.md`,
  },
  "full-text": {
    caveat:
      "Runs live via the optional W-text wasm bundle, lazy-loaded on expand; BM25 over the sample graph, not a tuned production index.",
    readme: `${REPO_URL}/tree/main/crates/sparq-text`,
    skill: `${REPO_URL}/blob/main/skills/full-text/SKILL.md`,
  },
  vector: {
    caveat:
      "Captured-output walkthrough: sparq-vectors is an opt-in native crate, so the real label-embedding run and vec: result tables are replayed verbatim, not executed in the tab.",
    readme: `${REPO_URL}/tree/main/crates/sparq-vectors`,
    skill: `${REPO_URL}/blob/main/skills/vector/SKILL.md`,
  },
  genai: {
    caveat:
      "Captured-output walkthrough: the schema card and executed result tables are real engine output; the model step is a scripted fixture (sparq-nlq is an opt-in native crate).",
    readme: `${REPO_URL}/tree/main/crates/sparq-nlq`,
    skill: `${REPO_URL}/blob/main/skills/genai/SKILL.md`,
  },
  "http-server": {
    caveat:
      "Captured curl + SSE-frame walkthrough: the static site has no backend, so the endpoint's real I/O (incl. a subscription firing on a committed UPDATE) is replayed, not served live.",
    readme: `${REPO_URL}/tree/main/crates/sparq-server`,
    skill: `${REPO_URL}/blob/main/skills/http-server/SKILL.md`,
  },
  "streaming-rsp": {
    caveat:
      "Runs live via the optional RSP wasm bundle, lazy-loaded on expand; RSP-QL windows over a scripted sample stream.",
    readme: `${REPO_URL}/tree/main/crates/sparq-rsp`,
    skill: `${REPO_URL}/blob/main/skills/streaming-rsp/SKILL.md`,
  },
  zk: {
    caveat:
      "Research-grade: the v1 ZK verifier is NOT externally audited — sound as landed under its stated threat model, pending re-review. Indicative engineering, not an audited cryptographic guarantee. Proving runs in-tab via 3rd-party bb.js UltraHonk (the ~MB prover chunk loads only on expand).",
    readme: `${REPO_URL}/tree/main/crates/sparq-zk`,
    skill: `${REPO_URL}/blob/main/skills/zk/SKILL.md`,
  },
  mpc: {
    caveat:
      "Research-grade in-tab SIMULATION of the protocol the native sparq-mpc crate runs — a faithful illustration, not the hardened crate in your browser and not a proof of correctness (that layer is a stub). Not externally audited.",
    readme: `${REPO_URL}/tree/main/crates/sparq-mpc`,
    skill: `${REPO_URL}/blob/main/skills/mpc/SKILL.md`,
  },
};

function TierBadge({ surface }: { surface: Surface }) {
  return (
    <Badge
      variant={TIER_VARIANT[surface.tier]}
      className="h-5 shrink-0 px-2 text-[11px]"
      title={TIER_LABEL[surface.tier]}
    >
      {TIER_LABEL[surface.tier]}
    </Badge>
  );
}

function RowShell({
  surface,
  trailing,
}: {
  surface: Surface;
  trailing: React.ReactNode;
}) {
  const Icon = surface.icon;
  return (
    <div className="flex items-center gap-3 px-3 py-3">
      <span className="flex size-9 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary">
        <Icon className="size-4.5" aria-hidden />
      </span>
      <div className="flex min-w-0 flex-1 flex-col">
        <span className="truncate text-sm font-medium">{surface.title}</span>
        <span className="truncate text-sm text-muted-foreground">{surface.blurb}</span>
      </div>
      <TierBadge surface={surface} />
      {trailing}
    </div>
  );
}

/** "Open →" deep-page row — a plain link to the surface's retained deep page. */
function DeepRow({ surface }: { surface: Surface }) {
  return (
    <Link
      href={surface.href}
      className="group block rounded-xl transition-colors hover:bg-muted/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50"
    >
      <RowShell
        surface={surface}
        trailing={
          <span className="inline-flex shrink-0 items-center gap-1 text-sm font-medium text-primary">
            Open
            <ArrowRight className="size-3.5 transition-transform group-hover:translate-x-0.5" />
          </span>
        }
      />
    </Link>
  );
}

/** "Coming soon" row — no demo yet; links out to source. */
function SoonRow({ surface }: { surface: Surface }) {
  return (
    <a
      href={REPO_URL}
      target="_blank"
      rel="noopener noreferrer"
      className="group block rounded-xl transition-colors hover:bg-muted/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50"
    >
      <RowShell
        surface={surface}
        trailing={
          <span className="inline-flex shrink-0 items-center gap-1 text-sm font-medium text-muted-foreground">
            Source
            <ExternalLink className="size-3.5" />
          </span>
        }
      />
    </a>
  );
}

/** "Demo ▸" expand-in-place row — lazily mounts the demo on first expand. */
function DemoRow({ surface, slug }: { surface: Surface; slug: LazyDemoSlug }) {
  const [open, setOpen] = React.useState(false);
  // Once mounted, stay mounted across collapse/re-expand so the chunk is fetched at most once.
  const [mounted, setMounted] = React.useState(false);
  const meta = META[slug];

  const toggle = () => {
    setOpen((v) => {
      const next = !v;
      if (next) setMounted(true);
      return next;
    });
  };

  return (
    <div className="rounded-xl">
      <button
        type="button"
        onClick={toggle}
        aria-expanded={open}
        className="group flex w-full items-center rounded-xl text-left transition-colors hover:bg-muted/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50"
      >
        <RowShell
          surface={surface}
          trailing={
            <span className="inline-flex shrink-0 items-center gap-1 text-sm font-medium text-primary">
              Demo
              <ChevronRight
                className={cn(
                  "size-3.5 transition-transform",
                  open && "rotate-90",
                )}
                aria-hidden
              />
            </span>
          }
        />
      </button>

      {/* The body is only present in the DOM once first expanded; CSS hides it on collapse so a
          re-expand does not re-fetch the chunk (mounted stays true). */}
      {mounted && (
        <div className={cn("px-3 pb-4", !open && "hidden")} data-demo-body={slug}>
          <div className="rounded-lg border bg-muted/20 p-3">
            <LazyDemo slug={slug} mounted={mounted} />
          </div>

          <details className="mt-3 rounded-lg border bg-card px-3 py-2 text-sm">
            <summary className="cursor-pointer select-none font-medium text-foreground/90">
              Details &amp; caveats
            </summary>
            <p className="mt-2 text-muted-foreground">{meta.caveat}</p>
            <div className="mt-2 flex flex-wrap gap-3 text-sm">
              <a
                className="inline-flex items-center gap-1 text-primary underline-offset-4 hover:underline"
                href={meta.readme}
                target="_blank"
                rel="noopener noreferrer"
              >
                Crate / source <ExternalLink className="size-3.5" />
              </a>
              {meta.skill && (
                <a
                  className="inline-flex items-center gap-1 text-primary underline-offset-4 hover:underline"
                  href={meta.skill}
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  SKILL.md <ExternalLink className="size-3.5" />
                </a>
              )}
            </div>
          </details>
        </div>
      )}
    </div>
  );
}

// [OPUS-4.8] sq-vw3ax.3 — the boundary is `slug` (a string) + `kind`, NOT the Surface object:
// a Surface carries a lucide `icon` function, which cannot cross the server→client RSC boundary
// (it is not serializable). The server /capabilities page passes the serializable slug; this
// client component resolves the full surface (icon included) from the GROUPS source by slug.
export function CapabilityRowItem({
  slug,
  kind,
}: {
  slug: string;
  kind: "deep" | "demo" | "soon";
}) {
  const surface = surfaceBySlug(slug);
  if (!surface) return null; // data drift — a row for an unknown surface; render nothing.
  if (kind === "deep") return <DeepRow surface={surface} />;
  if (kind === "soon") return <SoonRow surface={surface} />;
  // demo — guard against a data drift where a "demo" surface lacks a registered demo.
  if (hasLazyDemo(surface.slug)) {
    return <DemoRow surface={surface} slug={surface.slug} />;
  }
  return <SoonRow surface={surface} />;
}
