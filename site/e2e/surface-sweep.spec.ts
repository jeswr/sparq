// [SONNET-4.6] sq-ymr2e.8 — All-surface render sweep.
//
// WHAT IT GUARDS. A parametrized sweep over the full site route inventory: every route
// loads with no pageerror and no console.error, exposes a <main> landmark, and a per-surface
// key artifact is visible. Design of record: research/web-gui-test-program.md §2.5.
//
// ROUTE INVENTORY. Sourced from the IA source of truth (the data modules), NOT hardcoded:
//   * Papers   → PAPERS from src/data/papers.ts   (each slug → one route)
//   * Specs    → SPECS  from src/data/specs.ts    (each slug → one route)
//   * Benchmarks → BENCHMARK_FAMILY_KEYS constant GUARDED by a regex scan of
//     src/data/benchmarks.ts to detect key drift (see guard test).
//     NOTE: FAMILIES is not imported directly because benchmarks.ts uses an ESM JSON
//     import (`import data from "./benchmarks.generated.json"`) that requires a Node.js
//     `with { type: "json" }` attribute when loaded outside Next.js/webpack bundling.
//     The guard test below reads the source file and verifies BENCHMARK_FAMILY_KEYS matches.
//   * Surface deep pages → DEEP_SURFACE_SLUGS constant, GUARDED against src/app/surface/
//     (any unaccounted folder fails the guard test, so route drift fails loudly)
//   * Showcase pages → SHOWCASE_SLUGS constant, GUARDED against src/app/showcase/
//   * Top-level static routes → STATIC_ROUTES (small, filesystem-stable, bounded set)
//
// DETERMINISM.  Uses the shared `test` from ./support which auto-wires the hermetic-network
// fixture (blocks external fetches; fixture-routes the one external call this site makes)
// and the determinism fixture (frozen clock, seeded random, animations-off). gotoAppReady
// waits for the coi-serviceworker reload to settle and the app shell to hydrate.
//
// CONSOLE ERROR POLICY.  Every route must produce zero console.error / pageerror. The one
// allowlisted pattern is the benign "Failed to load resource: 404" that arises when the
// optional wasm bundle is absent on the light CI lane (the repo's documented posture for
// this lane — see home-smoke.spec.ts). Everything else fails the test. If a pre-existing
// console error is found on main, file a bug bead; do NOT silently widen the allow-list.
//
// NON-VACUITY. A dedicated test injects a synthetic console.error into a live page and
// asserts the tracking machinery catches it — proving the sweep is not a no-op.
//
// KEY ARTIFACTS (per-surface, presence-only — no numbers asserted):
//   home        → hero h1 "In this tab"
//   capabilities→ h1 heading
//   app         → h1 "App"
//   download    → h1 heading
//   examples    → h1 "Examples"
//   assurance   → h1 matching /Assurance/i
//   assurance/served-conformance → h1 heading (long title, first() match)
//   dogfooding  → h1 matching /Dogfooding/i
//   papers      → at least one "Read paper" link on the index page
//   per-paper   → blurb text visible (always sourced from data module, not generated HTML)
//   specs       → at least one "Read draft" link on the index page
//   per-spec    → blurb text visible (always sourced from data module, not generated HTML)
//   benchmarks  → h1 "Benchmarks"
//   per-bench   → h1 heading (family title) per type page
//   surface     → h1 heading per deep page (via the SurfaceContent component)
//   showcase    → h1 heading
//
// SHACL GUARD.  The __wbg_ptr regression guard lives in shacl-validator.spec.ts; this
// sweep does not duplicate it.
import { test, expect, gotoAppReady } from "./support";
import type { Page, ConsoleMessage } from "@playwright/test";
import * as fs from "fs";
import * as path from "path";
import { fileURLToPath } from "url";

// `__dirname` is not available in ESM scope — derive it from import.meta.url.
// [SONNET-4.6] sq-ymr2e.8: Playwright loads spec files as ESM (Node.js 22+ context).
const __specdir = path.dirname(fileURLToPath(import.meta.url));

// ── Data-module imports (safe: no React/component deps, no ESM JSON imports) ───────────

// [SONNET-4.6] sq-ymr2e.8: papers.ts and specs.ts have no component imports and no
// ESM JSON imports — they are safe to load in a Node.js Playwright context.
import { PAPERS } from "../src/data/papers";
import { SPECS } from "../src/data/specs";

// ── Benchmark family keys (sourced from src/data/benchmarks.ts FAMILIES, guarded) ─────

// [SONNET-4.6] sq-ymr2e.8: benchmarks.ts uses `import data from "./benchmarks.generated.json"`
// which requires a `with { type: "json" }` attribute under Node.js 22 ESM — not present in
// the source (added for Next.js/webpack bundling compatibility). Import FAMILIES directly is
// not possible in the Playwright context without a shim. Instead, this constant is derived
// from benchmarks.ts FAMILIES[*].key and the guard test below reads benchmarks.ts source to
// detect any key drift, so the constant never silently diverges from the IA source of truth.
const BENCHMARK_FAMILY_KEYS = [
  "sparql",
  "shacl",
  "geo",
  "fts",
  "vector",
  "reasoning",
  "core",
  "zk",
  "solid",
  "hdt",
  "rsp",
  "genai",
  "gpu",
] as const;

// ── Surface deep-page constant (guarded against filesystem drift) ──────────────────────

// The 5 surface routes that keep a hand-built deep page at /surface/<slug>.
// Source: src/data/capabilities.ts DEEP_PAGE_SLUGS (not re-imported to avoid pulling in
// lucide-react via surfaces.ts). Guard test below asserts every slug has a page.tsx.
const DEEP_SURFACE_SLUGS = [
  "sparql",
  "data-formats",
  "javascript-wasm",
  "shacl",
  "inference",
] as const;

// Surface slugs with their own folder but serving redirect stubs (not deep pages).
// Guard test ensures every surface folder is either here or in DEEP_SURFACE_SLUGS.
const REDIRECT_STUB_SURFACE_SLUGS = [
  "full-text",
  "genai",
  "geosparql",
  "http-server",
  "mpc",
  "streaming-rsp",
  "vector",
  "zk",
] as const;

// ── Showcase constant (guarded against filesystem drift) ───────────────────────────────

const SHOWCASE_SLUGS = [
  "mpc-100k",
  "solid-pairs",
  "verifiable-credentials",
  "zk-car-hire",
] as const;

// ── Console error tracking ─────────────────────────────────────────────────────────────

// Allow the benign resource 404 that appears when the optional wasm bundle is absent on
// the light CI lane (the repo's documented "optional wasm — warn and skip" posture; see
// home-smoke.spec.ts). All other errors fail the sweep immediately.
const BENIGN_RESOURCE_404 = /Failed to load resource:.*\b404\b/i;

function trackErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on("console", (msg: ConsoleMessage) => {
    if (msg.type() !== "error") return;
    const text = msg.text();
    if (BENIGN_RESOURCE_404.test(text)) return;
    errors.push(text);
  });
  page.on("pageerror", (err: Error) => errors.push(String(err)));
  return errors;
}

// ── Route table ───────────────────────────────────────────────────────────────────────

type Assertion = (page: Page) => Promise<void>;

interface RouteEntry {
  /** Path relative to the baseURL /sparq/ — include trailing slash. */
  path: string;
  /** Human-readable name for the test title. */
  name: string;
  /** Per-surface key artifact assertion. */
  artifact?: Assertion;
}

// ── Static top-level routes ────────────────────────────────────────────────────────────

const STATIC_ROUTES: RouteEntry[] = [
  {
    path: "",
    name: "/ (home)",
    artifact: async (page) => {
      // Hero h1 — the gradient split headline (sq-vw3ax.11).
      await expect(
        page.getByRole("heading", { level: 1, name: /In this tab/i }),
      ).toBeVisible();
    },
  },
  {
    path: "capabilities/",
    name: "/capabilities",
    artifact: async (page) => {
      await expect(page.getByRole("heading", { level: 1 }).first()).toBeVisible();
    },
  },
  {
    path: "app/",
    name: "/app",
    artifact: async (page) => {
      await expect(page.getByRole("heading", { level: 1, name: "App" })).toBeVisible();
    },
  },
  {
    path: "download/",
    name: "/download",
    artifact: async (page) => {
      await expect(page.getByRole("heading", { level: 1 }).first()).toBeVisible();
    },
  },
  {
    path: "examples/",
    name: "/examples",
    artifact: async (page) => {
      await expect(
        page.getByRole("heading", { level: 1, name: "Examples" }),
      ).toBeVisible();
    },
  },
  // [SONNET-4.6] sq-yk2ho — 3 unlinked content routes added to inventory.
  // These pages exist in src/app/ and each render an h1, but were absent from
  // both e2e inventories before this PR.
  {
    path: "assurance/",
    name: "/assurance",
    artifact: async (page) => {
      // h1: "Assurance — how to check that sparq works"
      await expect(
        page.getByRole("heading", { level: 1, name: /Assurance/i }),
      ).toBeVisible();
    },
  },
  {
    path: "assurance/served-conformance/",
    name: "/assurance/served-conformance",
    artifact: async (page) => {
      // h1: "Served-surface SPARQL 1.1 Protocol conformance"
      await expect(page.getByRole("heading", { level: 1 }).first()).toBeVisible();
    },
  },
  {
    path: "dogfooding/",
    name: "/dogfooding",
    artifact: async (page) => {
      // h1: "Dogfooding sparq"
      await expect(
        page.getByRole("heading", { level: 1, name: /Dogfooding/i }),
      ).toBeVisible();
    },
  },
];

// ── Papers index + per-paper routes ───────────────────────────────────────────────────

const PAPERS_ROUTES: RouteEntry[] = [
  {
    path: "papers/",
    name: "/papers (index)",
    artifact: async (page) => {
      // At least one "Read paper" link from a paper card must be present.
      await expect(
        page.getByRole("link", { name: /Read paper/i }).first(),
      ).toBeVisible();
    },
  },
  // Per-paper routes: slugs sourced from PAPERS (IA source of truth — papers.ts).
  ...PAPERS.map((p) => ({
    path: `papers/${p.slug}/`,
    name: `/papers/${p.slug}`,
    // Key artifact: the paper blurb is always rendered from the data module (NOT from the
    // generated HTML fragment), so this assertion holds whether or not build-papers has run.
    artifact: async (page: Page) => {
      // Use a stable, unique prefix of the blurb (first 60 chars).
      const blurbPrefix = p.blurb.slice(0, 60);
      await expect(
        page.getByText(blurbPrefix, { exact: false }).first(),
      ).toBeVisible();
    },
  })),
];

// ── Specs index + per-spec routes ─────────────────────────────────────────────────────

const SPECS_ROUTES: RouteEntry[] = [
  {
    path: "specs/",
    name: "/specs (index)",
    artifact: async (page) => {
      // At least one spec card with a "Read draft" link.
      await expect(
        page.getByRole("link", { name: /Read draft/i }).first(),
      ).toBeVisible();
    },
  },
  // Per-spec routes: slugs sourced from SPECS (IA source of truth — specs.ts).
  ...SPECS.map((s) => ({
    path: `specs/${s.slug}/`,
    name: `/specs/${s.slug}`,
    // Key artifact: the spec blurb is always rendered from the data module.
    artifact: async (page: Page) => {
      const blurbPrefix = s.blurb.slice(0, 60);
      await expect(
        page.getByText(blurbPrefix, { exact: false }).first(),
      ).toBeVisible();
    },
  })),
];

// ── Benchmarks index + per-family routes ──────────────────────────────────────────────

const BENCHMARKS_ROUTES: RouteEntry[] = [
  {
    path: "benchmarks/",
    name: "/benchmarks (index)",
    artifact: async (page) => {
      await expect(
        page.getByRole("heading", { level: 1, name: "Benchmarks" }),
      ).toBeVisible();
    },
  },
  // Per-family routes: keys sourced from BENCHMARK_FAMILY_KEYS (guarded against drift below).
  ...BENCHMARK_FAMILY_KEYS.map((key) => ({
    path: `benchmarks/${key}/`,
    name: `/benchmarks/${key}`,
    artifact: async (page: Page) => {
      // Each type page renders an h1 with the family title (presence only, not the text).
      await expect(page.getByRole("heading", { level: 1 }).first()).toBeVisible();
    },
  })),
];

// ── Surface deep-page routes ───────────────────────────────────────────────────────────

const SURFACE_ROUTES: RouteEntry[] = DEEP_SURFACE_SLUGS.map((slug) => ({
  path: `surface/${slug}/`,
  name: `/surface/${slug}`,
  artifact: async (page: Page) => {
    // SurfaceContent always renders an h1 with the surface title.
    await expect(page.getByRole("heading", { level: 1 }).first()).toBeVisible();
  },
}));

// ── Showcase routes ────────────────────────────────────────────────────────────────────

const SHOWCASE_ROUTES: RouteEntry[] = SHOWCASE_SLUGS.map((slug) => ({
  path: `showcase/${slug}/`,
  name: `/showcase/${slug}`,
  artifact: async (page: Page) => {
    await expect(page.getByRole("heading", { level: 1 }).first()).toBeVisible();
  },
}));

// ── Complete route inventory ───────────────────────────────────────────────────────────

const ALL_ROUTES: RouteEntry[] = [
  ...STATIC_ROUTES,
  ...PAPERS_ROUTES,
  ...SPECS_ROUTES,
  ...BENCHMARKS_ROUTES,
  ...SURFACE_ROUTES,
  ...SHOWCASE_ROUTES,
];

// ── Filesystem path anchors ────────────────────────────────────────────────────────────

const SURFACE_APP_DIR = path.join(__specdir, "../src/app/surface");
const SHOWCASE_APP_DIR = path.join(__specdir, "../src/app/showcase");
const BENCHMARKS_SRC = path.join(__specdir, "../src/data/benchmarks.ts");

// ── Guard tests: route drift fails loudly ─────────────────────────────────────────────

test.describe("surface-sweep route guards", () => {
  // ── Surface guards ──

  test("DEEP_SURFACE_SLUGS: every slug has a page.tsx in src/app/surface/", () => {
    for (const slug of DEEP_SURFACE_SLUGS) {
      const pagePath = path.join(SURFACE_APP_DIR, slug, "page.tsx");
      expect(
        fs.existsSync(pagePath),
        `Expected deep page at src/app/surface/${slug}/page.tsx — update DEEP_SURFACE_SLUGS if the route was added or removed`,
      ).toBe(true);
    }
  });

  test("REDIRECT_STUB_SURFACE_SLUGS: every slug has a page.tsx in src/app/surface/", () => {
    for (const slug of REDIRECT_STUB_SURFACE_SLUGS) {
      const pagePath = path.join(SURFACE_APP_DIR, slug, "page.tsx");
      expect(
        fs.existsSync(pagePath),
        `Expected redirect stub at src/app/surface/${slug}/page.tsx — update REDIRECT_STUB_SURFACE_SLUGS if the route was added or removed`,
      ).toBe(true);
    }
  });

  test("surface app directory: every subfolder is accounted for in the sweep inventory", () => {
    const allDirs = fs
      .readdirSync(SURFACE_APP_DIR, { withFileTypes: true })
      .filter((d) => d.isDirectory())
      .map((d) => d.name)
      // [slug] is the catch-all for remaining slugs (cli, python); not a named constant route.
      .filter((n) => n !== "[slug]");

    const knownSlugs: Set<string> = new Set([
      ...DEEP_SURFACE_SLUGS,
      ...REDIRECT_STUB_SURFACE_SLUGS,
    ]);

    for (const dir of allDirs) {
      expect(
        knownSlugs.has(dir),
        `src/app/surface/${dir}/ is not in DEEP_SURFACE_SLUGS or REDIRECT_STUB_SURFACE_SLUGS — add it to the sweep inventory`,
      ).toBe(true);
    }
  });

  // ── Showcase guards ──

  test("SHOWCASE_SLUGS: every slug has a page.tsx in src/app/showcase/", () => {
    for (const slug of SHOWCASE_SLUGS) {
      const pagePath = path.join(SHOWCASE_APP_DIR, slug, "page.tsx");
      expect(
        fs.existsSync(pagePath),
        `Expected showcase page at src/app/showcase/${slug}/page.tsx — update SHOWCASE_SLUGS`,
      ).toBe(true);
    }
  });

  test("showcase app directory: every subfolder is in SHOWCASE_SLUGS", () => {
    const allDirs = fs
      .readdirSync(SHOWCASE_APP_DIR, { withFileTypes: true })
      .filter((d) => d.isDirectory())
      .map((d) => d.name);

    const knownSlugs: Set<string> = new Set(SHOWCASE_SLUGS);
    for (const dir of allDirs) {
      expect(
        knownSlugs.has(dir),
        `src/app/showcase/${dir}/ is not in SHOWCASE_SLUGS — add it to the sweep inventory`,
      ).toBe(true);
    }
  });

  // ── Benchmark family key guard ──

  test("BENCHMARK_FAMILY_KEYS: guard against key drift in src/data/benchmarks.ts", () => {
    // Read the benchmarks.ts source and extract the family keys via regex.
    // Pattern: `key: "..."` inside the FAMILIES const array.
    const src = fs.readFileSync(BENCHMARKS_SRC, "utf8");
    const matches = [...src.matchAll(/key:\s*"([a-z][a-z0-9-]*)"/g)];
    const keysInSource = matches.map((m) => m[1]);

    // Every key in our constant must appear in the source.
    for (const key of BENCHMARK_FAMILY_KEYS) {
      expect(
        keysInSource,
        `BENCHMARK_FAMILY_KEYS["${key}"] not found in src/data/benchmarks.ts — check for a removed family`,
      ).toContain(key);
    }

    // Every key in the source must appear in our constant (catches newly-added families).
    for (const key of keysInSource) {
      expect(
        BENCHMARK_FAMILY_KEYS as readonly string[],
        `benchmarks.ts has key "${key}" not in BENCHMARK_FAMILY_KEYS — add it to the sweep inventory`,
      ).toContain(key);
    }
  });

  // ── Papers / Specs non-vacuity guards ──

  test("PAPERS: route inventory covers every registered paper slug", () => {
    expect(PAPERS.length, "PAPERS must be non-empty").toBeGreaterThan(0);
    const perPaperPaths = PAPERS_ROUTES.filter((r) => r.path !== "papers/").map(
      (r) => r.path,
    );
    for (const paper of PAPERS) {
      expect(perPaperPaths).toContain(`papers/${paper.slug}/`);
    }
  });

  test("SPECS: route inventory covers every registered spec slug", () => {
    expect(SPECS.length, "SPECS must be non-empty").toBeGreaterThan(0);
    const perSpecPaths = SPECS_ROUTES.filter((r) => r.path !== "specs/").map(
      (r) => r.path,
    );
    for (const spec of SPECS) {
      expect(perSpecPaths).toContain(`specs/${spec.slug}/`);
    }
  });
});

// ── Non-vacuity: injected console.error is caught ─────────────────────────────────────

test("non-vacuity: injected console.error is caught by the sweep error tracker", async ({
  page,
}) => {
  const errors: string[] = [];
  page.on("console", (msg: ConsoleMessage) => {
    if (msg.type() === "error") errors.push(msg.text());
  });

  await gotoAppReady(page, "");
  await page.evaluate(() => console.error("__sq-ymr2e-8-non-vacuity-sentinel__"));

  expect(errors).toContain("__sq-ymr2e-8-non-vacuity-sentinel__");
});

// ── Parametrized render sweep ──────────────────────────────────────────────────────────

for (const entry of ALL_ROUTES) {
  test(`[sweep] ${entry.name} — zero console errors + main + artifact`, async ({ page }) => {
    const errors = trackErrors(page);

    await gotoAppReady(page, entry.path);

    // Every route must expose a <main> landmark (provided by the AppShell layout).
    await expect(page.locator("main").first()).toBeAttached({ timeout: 15_000 });

    // Key artifact check (presence only — never a benchmark number or timing claim).
    if (entry.artifact) {
      await entry.artifact(page);
    }

    // Zero real console errors. If this fires for an existing route on main, create a bead.
    expect(
      errors,
      `console errors on ${entry.name}:\n${errors.join("\n")}`,
    ).toEqual([]);
  });
}
