// [OPUS-4.8] sq-jp7ry (issue #835) — CRITICAL-FLOW smoke test for the /capabilities SHOWCASE.
//
// WHAT IT GUARDS. /capabilities (src/app/capabilities/page.tsx) is the consolidated, bold
// showcase of the full feature set — a hero, a flagship "start here" band, and five capability
// THEME lanes (Query & data / Reason & validate / Search & GenAI / Privacy (ZK / MPC) / Serve &
// embed), all derived from the single data/surfaces.ts source. This spec asserts the page renders
// ALL of those structural sections on a DIRECT load AND produces ZERO console errors. It is the
// SECTIONS-RENDER smoke (complementing capabilities-lazy.spec.ts, which guards the lazy-mount /
// code-split invariant, and site-nav.spec.ts, which reaches the themes via a nav click): here we
// confirm the showcase itself boots clean with every theme present — the regression net issue
// #835 asks for ("unexpected bugs get introduced").
//
// NO WASM NEEDED. The hero + flagship band + theme-lane HEADINGS are server-rendered shell that
// paint without the wasm engine (a demo's heavy body mounts only on expand — that is the
// lazy-mount spec's concern), so this spec runs on EVERY lane, including the light site-e2e CI
// lane that builds no wasm bundle.
//
// It is a CORRECTNESS smoke test, not a benchmark: it asserts the DOM + console, never a
// wall-clock threshold (timings on a work-box / CI runner are non-canonical).
import { test, expect, type Page, type ConsoleMessage } from "@playwright/test";

// Relative (no leading slash) so it resolves UNDER the baseURL's `/sparq/` basePath.
const ROUTE = "capabilities/";

// The five capability themes (data/surfaces.ts GROUPS labels). Kept here as the expected set so
// a dropped/renamed lane fails this smoke loudly.
const THEMES = [
  "Query & data",
  "Reason & validate",
  "Search & GenAI",
  "Privacy (ZK / MPC)",
  "Serve & embed",
];

// [GPT-5.6] sq-vw3ax.15 — exact native-row contract. Mutating a title, snippet, or link makes
// this smoke fail, so the coverage is non-vacuous and guards the content the bead added.
const NATIVE_ROWS = [
  {
    slug: "graph-analytics",
    title: "Graph analytics",
    snippet: "let ranks = sparq_algos::pagerank(&graph, Default::default());",
    readme: "https://github.com/sparq-org/sparq/tree/main/crates/sparq-algos",
    skill: "https://github.com/sparq-org/sparq/blob/main/skills/graph-analytics/SKILL.md",
  },
  {
    slug: "rdf-canon",
    title: "RDFC-1.0 canonicalization",
    snippet: "let canonical = sparq_canon::canonicalize(&quads)?;",
    readme: "https://github.com/sparq-org/sparq/tree/main/crates/sparq-canon",
    skill: "https://github.com/sparq-org/sparq/blob/main/skills/rdf-canon/SKILL.md",
  },
  {
    slug: "arrow-columnar",
    title: "Arrow export",
    snippet: "let batch = sparq_arrow::to_record_batch(&result)?;",
    readme: "https://github.com/sparq-org/sparq/tree/main/crates/sparq-arrow",
    skill: "https://github.com/sparq-org/sparq/blob/main/skills/arrow-columnar/SKILL.md",
  },
  {
    slug: "prov-lineage",
    title: "PROV-O lineage",
    snippet: "let derivation = sparq_prov::derive_construct(&graph, query, config)?;",
    readme: "https://github.com/sparq-org/sparq/tree/main/crates/sparq-prov",
    skill: "https://github.com/sparq-org/sparq/blob/main/skills/prov-lineage/SKILL.md",
  },
  {
    slug: "mcp-server",
    title: "MCP server",
    snippet: "let mut server = sparq_mcp::McpServer::new(graph);",
    readme: "https://github.com/sparq-org/sparq/tree/main/crates/sparq-mcp",
    skill: "https://github.com/sparq-org/sparq/blob/main/skills/agent-tools/SKILL.md",
  },
] as const;

/** Collect every `console.error` the page emits, so the test can assert there were none. */
function trackConsoleErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on("console", (msg: ConsoleMessage) => {
    if (msg.type() === "error") errors.push(msg.text());
  });
  page.on("pageerror", (err) => errors.push(String(err)));
  return errors;
}

async function gotoSettled(page: Page, route: string): Promise<void> {
  await page.goto(route, { waitUntil: "domcontentloaded" });
  await page.waitForLoadState("networkidle").catch(() => {});
  await page
    .waitForFunction(() => navigator.serviceWorker?.controller != null, undefined, {
      timeout: 10_000,
    })
    .catch(() => {});
}

test("the showcase renders its hero, flagship band and all five theme sections with no console errors", async ({
  page,
}) => {
  const consoleErrors = trackConsoleErrors(page);
  await gotoSettled(page, ROUTE);

  // The hero <h1> (server-rendered display headline). Matched on a stable copy substring.
  await expect(
    page.getByRole("heading", { name: /one Rust engine/i, level: 1 }),
  ).toBeVisible();

  // The flagship "start here" band heading (the <section aria-labelledby="flagships-heading">
  // names its region by this <h2>). Asserting the heading proves the band rendered.
  await expect(
    page.getByRole("heading", { name: /the breadth is real/i, level: 2 }),
  ).toBeVisible();

  // Every one of the five capability-theme lanes renders its heading. A dropped or renamed
  // theme fails this loudly — the showcase's load-bearing structural invariant.
  for (const theme of THEMES) {
    await expect(page.getByRole("heading", { name: theme })).toBeVisible();
  }

  // Every newly surfaced native crate renders as a real row with the native honesty badge,
  // its exact API line, and both working deep-link targets.
  for (const expected of NATIVE_ROWS) {
    const row = page.locator(`[data-capability="${expected.slug}"]`);
    await expect(row).toBeVisible();
    await expect(row.getByText(expected.title, { exact: true })).toBeVisible();
    await expect(row.getByText("Native crate", { exact: true })).toBeVisible();
    await expect(row.locator("code")).toHaveText(expected.snippet);
    await expect(row.getByRole("link", { name: /Crate \/ source/i })).toHaveAttribute(
      "href",
      expected.readme,
    );
    await expect(row.getByRole("link", { name: /SKILL\.md/i })).toHaveAttribute(
      "href",
      expected.skill,
    );
  }

  // No demo BODY is eagerly mounted on entry (the lazy-mount invariant is fully owned by
  // capabilities-lazy.spec.ts; asserted here only to keep this smoke honest about what "renders
  // its sections" means — the headings, not the heavy demo bodies).
  await expect(page.locator("[data-demo-body]")).toHaveCount(0);

  // The whole render produced zero console errors (the regression net for issue #835).
  expect(consoleErrors, `console errors: ${consoleErrors.join("\n")}`).toEqual([]);
});
