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
// same export serves Pages (/sparq prefix) and the Tauri webview (root) unchanged. When the
// destination is a SEPARATE deployed app (the `hard` prop — see below), a full-page
// window.location redirect is used instead so the browser loads that app's own HTML.

import * as React from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";

import { withBasePath } from "@/lib/base-path";

export function RedirectStub({
  to,
  label,
  hard,
}: {
  /** The destination path (basePath-relative, e.g. "/capabilities#privacy"). */
  to: string;
  /** Human label for the destination, shown in the no-JS fallback link. */
  label: string;
  /**
   * [OPUS-4.8] sq-vw3ax.11 — force a HARD (full-page) redirect via `window.location` instead of a
   * next/router soft `replace`. Required when `to` is served by a SEPARATE Next.js app at the same
   * origin (e.g. /app = the gui/app workbench overlaid at /app/): a soft nav across two
   * distinct Next builds fetches the foreign RSC Flight payload (/app/index.txt) and lands
   * on a raw .txt instead of the destination app. Same-site removed routes (the /surface/* and
   * /about stubs) stay a soft `replace` and pass `hard` falsy.
   */
  hard?: boolean;
}) {
  const router = useRouter();
  // For a hard redirect the hand-written absolute URL must be base-path-prefixed (Next does not
  // rewrite window.location) and carry the trailing slash (`trailingSlash: true`) so the static
  // export's directory index resolves. Deterministic → stable across renders.
  const hardDest = withBasePath(to.endsWith("/") ? to : `${to}/`);

  React.useEffect(() => {
    if (hard) {
      window.location.replace(hardDest); // [OPUS-4.8] replace (not assign) — keeps /gui out of back-stack per the "dead route out of history" intent
    } else {
      router.replace(to);
    }
  }, [router, to, hard, hardDest]);

  return (
    <div className="mx-auto max-w-md py-16 text-center text-sm text-muted-foreground">
      <p>
        This page has moved. Redirecting to{" "}
        {hard ? (
          <a
            href={hardDest}
            className="text-primary underline-offset-4 hover:underline"
          >
            {label}
          </a>
        ) : (
          <Link href={to} className="text-primary underline-offset-4 hover:underline">
            {label}
          </Link>
        )}
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
