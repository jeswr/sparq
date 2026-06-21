"use client";

// [OPUS-4.8] sq-vw3ax.3 / .7 — a CLIENT-SIDE redirect stub for a removed route.
//
// The site is `output: "export"` (static, no server), so it CANNOT issue a real 301
// (research/website-redesign.md §7 must_fix). When the redesign removes a route — the 8+
// /surface/<slug> walkthrough pages folded into /capabilities, and /about folded into the
// Home #how-it-runs strip — inbound and cross-page links to the old path would otherwise hit
// the catch-all's `dynamicParams = false` and hard-404. This stub is the static-export
// stand-in: a tiny prerendered page at the old path that, on mount, client-redirects to the
// new destination (with a visible fallback link for no-JS / crawlers).
//
// router.replace (not push) keeps the dead route out of history, and respects basePath so the
// same export serves Pages (/sparq prefix) and the Tauri webview (root) unchanged.

import * as React from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";

export function RedirectStub({
  to,
  label,
}: {
  /** The destination path (basePath-relative, e.g. "/capabilities#privacy"). */
  to: string;
  /** Human label for the destination, shown in the no-JS fallback link. */
  label: string;
}) {
  const router = useRouter();

  React.useEffect(() => {
    router.replace(to);
  }, [router, to]);

  return (
    <div className="mx-auto max-w-md py-16 text-center text-sm text-muted-foreground">
      <p>
        This page has moved. Redirecting to{" "}
        <Link href={to} className="text-primary underline-offset-4 hover:underline">
          {label}
        </Link>
        …
      </p>
      <noscript>
        <p className="mt-2">
          JavaScript is disabled — follow the link above to continue.
        </p>
      </noscript>
    </div>
  );
}
