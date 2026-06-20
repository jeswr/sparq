"use client";

// [OPUS-4.8] sq-4296 (#935 / #981) — the CODE-SPLIT boundary for the live REPL.
//
// The `Repl` is by far the heaviest client component on the site: it pulls in the SPARQL
// editor, the results / dataset / workspace panels, the endpoint + subscriptions + health
// views, and the `@sparq/client` query surface — and it is rendered on BOTH the home page
// (`/`) and `/try`. Statically importing it put all of that JS into those routes' FIRST-LOAD
// bundle, so the page could not become interactive until the whole REPL chunk had downloaded
// and hydrated. That is the "don't bundle it into the main bundle" half of the maintainer's
// website-load-time ask (#935): the heavy code should NOT sit on the critical path.
//
// `next/dynamic` with `ssr: false` moves the `Repl` into its OWN async chunk that the browser
// fetches AFTER the route shell has rendered, so the page paints + becomes interactive first,
// then the REPL streams in. A server component cannot call `dynamic(..., { ssr: false })`
// directly in the Next.js App Router, so this thin `"use client"` wrapper owns the split; the
// server pages render `<ReplLazy />` instead of `<Repl />` with no other change.
//
// This is the COMPONENT-level lazy load; the wasm ENGINE itself is loaded separately and even
// later — `prewarmSparqWhenIdle` (in `@sparq/client`) defers the ~300 kB+ engine wasm fetch to
// a browser-idle slot once the REPL has mounted. The two together keep the page interactive
// before either the REPL chunk or the engine wasm has finished loading.

import dynamic from "next/dynamic";

import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";

/**
 * A layout-stable placeholder shown while the `Repl` chunk downloads. It mirrors the REPL's
 * card shape (header row + editor block + a run-control row) so there is no layout shift when
 * the real component swaps in. Purely presentational — no wasm, no client state.
 */
function ReplSkeleton() {
  return (
    <Card aria-busy="true" aria-label="Loading the live SPARQL REPL" data-repl-skeleton>
      <CardHeader className="gap-3">
        <div className="flex items-center justify-between gap-2">
          <Skeleton className="h-5 w-40" />
          <Skeleton className="h-6 w-28 rounded-full" />
        </div>
        <Skeleton className="h-9 w-full" />
      </CardHeader>
      <CardContent className="space-y-3">
        <Skeleton className="h-40 w-full rounded-md" />
        <div className="flex items-center gap-2">
          <Skeleton className="h-9 w-28" />
          <Skeleton className="h-9 w-24" />
          <Skeleton className="ml-auto h-9 w-32" />
        </div>
        <Skeleton className="h-24 w-full rounded-md" />
      </CardContent>
    </Card>
  );
}

// The split point. `ssr: false` keeps the REPL out of the static-export prerender (it is a
// browser-only, wasm-backed component anyway), so it ships as a separate client chunk fetched
// after the route shell renders. The skeleton is the loading state in the meantime.
const Repl = dynamic(() => import("@/components/repl").then((m) => m.Repl), {
  ssr: false,
  loading: () => <ReplSkeleton />,
});

/** The code-split entry point the server pages render in place of the heavy `Repl`. */
export function ReplLazy() {
  return <Repl />;
}
