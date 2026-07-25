// [OPUS-4.8] sq-ymr2e.1 — HERMETIC NETWORK for the site Playwright suite.
//
// Design of record: research/web-gui-test-program.md §1.2 — "Block all non-localhost requests by
// default; external APIs are fixture-routed. A test that touches the real network is a bug." The
// site is static-exported and the home hero runner executes in-browser WASM (sq-vw3ax.11), so a
// correct run makes ZERO external requests during a journey; this layer turns that design promise
// into an enforced invariant AND records what was blocked so the sibling zero-network journeys
// (§2.1) can assert on it.
//
// POLICY (per request):
//   * same-origin / localhost / 127.0.0.1 / [::1], and data:/blob:/about:/chrome-extension: and
//     non-http(s) schemes (ws HMR etc.) → CONTINUE.
//   * api.github.com/repos/*/sparq/releases/latest → FULFILL from a checked-in JSON fixture
//     (the one external call the product makes; test-data/github.ts).
//   * anything else external → ABORT, and record the URL on `externalRequests`.
//
// THE SERVICE-WORKER SEAM (documented, not hidden): the site ships coi-serviceworker, which
// re-issues fetches from INSIDE the worker and so bypasses page/context routing. That matters for
// exactly one thing — sealing the /download github call and OBSERVING every request for the
// zero-network invariant — which is why those specs additionally set
// `test.use({ serviceWorkers: "block" })` (the download-page.spec pattern). This layer intercepts
// every PAGE-originated request unconditionally; specs that need SW-proof sealing add the block.
import type { BrowserContext, Route } from "@playwright/test";
import {
  githubFixtureResponse,
  isGithubReleaseUrl,
  type GithubFixtureName,
} from "./test-data/github";

/** Hosts that are always allowed (in addition to the page's own localhost origin). */
const LOCAL_HOST_RE = /^(localhost|127\.0\.0\.1|\[::1\]|0\.0\.0\.0|::1)$/i;

/** Schemes that are never network egress (allowed without host classification). */
const NON_NETWORK_SCHEMES = new Set([
  "data:",
  "blob:",
  "about:",
  "chrome:",
  "chrome-extension:",
  "filesystem:",
]);

export type HermeticPolicy = {
  /** Which releases/latest response to serve (or `off` to block that endpoint too). */
  github: GithubFixtureName;
  /** Extra hosts to allow. A string matches the host exactly or as a `.suffix`; a RegExp is
   *  tested against the full URL. Keep this SMALL and justified — every entry is a hole. */
  allowHosts: (string | RegExp)[];
};

export const DEFAULT_HERMETIC_POLICY: HermeticPolicy = {
  github: "release",
  allowHosts: [],
};

/** Handle to the live interception state a spec can assert on. */
export type HermeticController = {
  /** External URLs that were BLOCKED (aborted). The zero-network invariant asserts this empty. */
  readonly externalRequests: readonly string[];
  /** External URLs FULFILLED from a fixture (the github releases call). */
  readonly routedRequests: readonly string[];
  /** Clear both logs — useful to scope an assertion to a single interaction window. */
  reset(): void;
};

type Verdict = "allow" | "github" | "block";

function classify(rawUrl: string, policy: HermeticPolicy): Verdict {
  let url: URL;
  try {
    url = new URL(rawUrl);
  } catch {
    // Opaque/relative — cannot be external egress; allow.
    return "allow";
  }
  if (NON_NETWORK_SCHEMES.has(url.protocol)) return "allow";
  if (url.protocol !== "http:" && url.protocol !== "https:") return "allow";
  if (LOCAL_HOST_RE.test(url.hostname)) return "allow";
  if (isGithubReleaseUrl(rawUrl)) return policy.github === "off" ? "block" : "github";
  for (const allow of policy.allowHosts) {
    if (typeof allow === "string") {
      if (url.hostname === allow || url.hostname.endsWith(`.${allow}`)) return "allow";
    } else if (allow.test(rawUrl)) {
      return "allow";
    }
  }
  return "block";
}

/**
 * Register the hermetic route on a browser context and return a controller the test can inspect.
 * Uses `context.route` (not `page.route`) so it also covers popups/second pages in the context.
 * Awaits the registration so interception is guaranteed active before the test navigates.
 */
export async function installHermeticNetwork(
  context: BrowserContext,
  policy: HermeticPolicy,
): Promise<HermeticController> {
  const externalRequests: string[] = [];
  const routedRequests: string[] = [];

  const handler = async (route: Route): Promise<void> => {
    const url = route.request().url();
    switch (classify(url, policy)) {
      case "allow":
        await route.continue();
        return;
      case "github": {
        routedRequests.push(url);
        const { status, body } = githubFixtureResponse(policy.github);
        await route.fulfill({
          status,
          contentType: "application/json",
          // CORS + CORP headers so the fetch is allowed under the site's cross-origin isolation.
          headers: {
            "access-control-allow-origin": "*",
            "cross-origin-resource-policy": "cross-origin",
          },
          body,
        });
        return;
      }
      case "block":
        externalRequests.push(url);
        await route.abort();
        return;
    }
  };

  await context.route("**/*", handler);

  return {
    get externalRequests() {
      return externalRequests;
    },
    get routedRequests() {
      return routedRequests;
    },
    reset() {
      externalRequests.length = 0;
      routedRequests.length = 0;
    },
  };
}
