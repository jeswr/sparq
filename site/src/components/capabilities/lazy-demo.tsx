"use client";

// [OPUS-4.8] sq-vw3ax.3 — the LAZY-MOUNT boundary for the /capabilities gallery.
//
// WHY THIS IS THE #1 RISK (research/website-redesign.md §3 COLLAPSE; the adversarial
// review's must_fix). /capabilities consolidates 8 interactive demos that used to live on
// 8 separate /surface/* routes onto ONE route. If those demo components were imported
// statically, the route's first-load bundle would carry ALL of them at once — including the
// in-browser ZK prover (@noir-lang/noir_js + @aztec/bb.js, ~MB) and the geo/vector
// walkthroughs — making the "lighter" gallery HEAVIER than the 14 pages it replaces.
//
// THE FIX. Every demo is wrapped in `next/dynamic(..., { ssr: false })`, and the dynamic
// component is only RENDERED once its row is expanded (`mounted` flips true on first open).
// `next/dynamic` does not start the network fetch for a chunk until the component is actually
// rendered, so each demo chunk downloads ONLY on first expand — never on route entry. This
// preserves exactly the route-scoping `sidebar-nav.tsx` enforced for the ZK prover: the
// bb.js / noir_js chunks stay out of the /capabilities first-load bundle and arrive only if
// a visitor opens the ZK row. The e2e test (e2e/capabilities-lazy.spec.ts) and a build-time
// bundle check (scripts/check-bundle.mjs, run as `npm run check:bundle` / the build's `postbuild`;
// [FABLE-5] sq-vw3ax.9) both assert no demo chunk loads on route entry.

import * as React from "react";
import dynamic from "next/dynamic";
import type { ComponentType } from "react";

import { Skeleton } from "@/components/ui/skeleton";

/** A layout-stable placeholder shown while a demo chunk downloads on first expand. */
function DemoSkeleton({ label }: { label: string }) {
  return (
    <div
      aria-busy="true"
      aria-label={`Loading the ${label} demo`}
      data-demo-skeleton
      className="space-y-3 py-2"
    >
      <Skeleton className="h-6 w-48" />
      <Skeleton className="h-40 w-full rounded-md" />
      <div className="flex items-center gap-2">
        <Skeleton className="h-9 w-28" />
        <Skeleton className="h-9 w-24" />
      </div>
    </div>
  );
}

// The split table. Each entry is `dynamic(() => import(...))` with `ssr: false`, so each demo
// becomes its OWN async chunk that is fetched only when the component first renders. The
// imports are written as static, statically-analysable specifiers (one per demo) so webpack
// can code-split them at build time — a computed `import(variable)` would defeat that.
//
// IMPORTANT: keep these as separate `dynamic()` calls (not a generic factory over a string),
// because Next/webpack needs the literal import specifier to emit one chunk per demo and to
// keep the heavy bb.js / noir_js graph isolated to the ZK chunk alone.
const DEMOS = {
  geosparql: dynamic(
    () => import("@/components/geosparql-walkthrough").then((m) => m.GeosparqlWalkthrough),
    { ssr: false, loading: () => <DemoSkeleton label="GeoSPARQL" /> },
  ),
  "full-text": dynamic(
    () => import("@/components/text-playground").then((m) => m.TextPlayground),
    { ssr: false, loading: () => <DemoSkeleton label="full-text" /> },
  ),
  vector: dynamic(
    () => import("@/components/vector-walkthrough").then((m) => m.VectorWalkthrough),
    { ssr: false, loading: () => <DemoSkeleton label="vector" /> },
  ),
  genai: dynamic(
    () => import("@/components/genai-walkthrough").then((m) => m.GenaiWalkthrough),
    { ssr: false, loading: () => <DemoSkeleton label="GenAI / NLQ" /> },
  ),
  "http-server": dynamic(
    () => import("@/components/http-server-walkthrough").then((m) => m.HttpServerWalkthrough),
    { ssr: false, loading: () => <DemoSkeleton label="HTTP server" /> },
  ),
  "streaming-rsp": dynamic(
    () => import("@/components/rsp-playground").then((m) => m.RspPlayground),
    { ssr: false, loading: () => <DemoSkeleton label="streaming RSP" /> },
  ),
  federation: dynamic(
    () =>
      import("@/components/federation-walkthrough").then((m) => m.FederationWalkthrough),
    { ssr: false, loading: () => <DemoSkeleton label="federation" /> },
  ),
  zk: dynamic(
    () => import("@/components/zk-car-hire").then((m) => m.ZkCarHire),
    { ssr: false, loading: () => <DemoSkeleton label="ZK query proofs" /> },
  ),
  mpc: dynamic(
    () => import("@/components/mpc-demo").then((m) => m.MpcDemo),
    { ssr: false, loading: () => <DemoSkeleton label="MPC federation" /> },
  ),
  policy: dynamic(
    () => import("@/components/policy-walkthrough").then((m) => m.PolicyWalkthrough),
    { ssr: false, loading: () => <DemoSkeleton label="ODRL usage control" /> },
  ),
} satisfies Record<string, ComponentType>;

/** The set of surface slugs that have a lazily-mountable inline demo on /capabilities. */
export type LazyDemoSlug = keyof typeof DEMOS;

export function hasLazyDemo(slug: string): slug is LazyDemoSlug {
  return slug in DEMOS;
}

/**
 * Renders the demo for `slug` ONLY once `mounted` is true. The parent flips `mounted` to true
 * on first expand and keeps it true thereafter (so collapsing/re-expanding does not re-fetch),
 * while the wrapping <details>/CSS handles show/hide. Until first expand this returns null, so
 * the demo's `dynamic()` is never rendered and its chunk is never fetched.
 */
export function LazyDemo({ slug, mounted }: { slug: LazyDemoSlug; mounted: boolean }) {
  if (!mounted) return null;
  const Demo = DEMOS[slug];
  return <Demo />;
}
