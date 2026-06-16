// [OPUS-4.8] sq-gum8 — the paper-factory build step.
//
// For every paper registered in src/data/papers.ts, this:
//   1. runs the build-time HONESTY GATE on src/data/paper-evidence.json (schema + the
//      canonical/indicative invariant), then
//   2. compiles the paper's .typ to BOTH a PDF (public/papers/<slug>.pdf — the download)
//      and a semantic HTML fragment (src/generated/papers/<slug>.html — the in-site render),
//      injecting the SAME evidence JSON via `--input data=...` so the two artifacts cannot
//      show different numbers.
//
// It is wired into `prebuild` (after sync-benchmarks) so `next build` always regenerates the
// papers against fresh data, and into `dev`. The typst binary is resolved from PATH or
// ~/.local/bin or ~/.cargo/bin; CI installs it (see the workflow note at the bottom).
//
// The page route imports the generated HTML fragment at build time, so the in-site render is
// a static asset (no WASM compiler shipped to the browser). See the route + papers.ts.

import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { homedir } from "node:os";

const __dirname = dirname(fileURLToPath(import.meta.url));
const SITE = resolve(__dirname, "..");
const EVIDENCE_PATH = join(SITE, "src", "data", "paper-evidence.json");
const PAPERS_DIR = join(SITE, "papers");
const PDF_OUT_DIR = join(SITE, "public", "papers");
const HTML_OUT_DIR = join(SITE, "src", "generated", "papers");

// ---- resolve the typst binary -------------------------------------------------------------
function resolveTypst() {
  const candidates = [
    process.env.TYPST_BIN,
    "typst",
    join(homedir(), ".local", "bin", "typst"),
    join(homedir(), ".cargo", "bin", "typst"),
  ].filter(Boolean);
  for (const c of candidates) {
    try {
      execFileSync(c, ["--version"], { stdio: "ignore" });
      return c;
    } catch {
      /* try next */
    }
  }
  return null;
}

// ---- the registry (parsed from papers.ts without importing TS) ----------------------------
// papers.ts is the single source of truth; we read the slug + source list from it via a tiny
// regex so this plain .mjs build step needs no TS toolchain. The shape is asserted below.
function readRegistry() {
  const src = readFileSync(join(SITE, "src", "data", "papers.ts"), "utf8");
  const slugs = [...src.matchAll(/slug:\s*"([^"]+)"/g)].map((m) => m[1]);
  const sources = [...src.matchAll(/source:\s*"([^"]+)"/g)].map((m) => m[1]);
  if (slugs.length === 0 || slugs.length !== sources.length) {
    throw new Error(
      `build-papers: could not parse papers.ts (slugs=${slugs.length}, sources=${sources.length})`,
    );
  }
  return slugs.map((slug, i) => ({ slug, source: sources[i] }));
}

// ---- the build-time honesty gate (schema + canonical/indicative invariant) ----------------
// The .typ-level gate (headline() in papers/_lib/bench.typ) already panics the compile if a
// headline cites a non-canonical record. This is the data-layer guard that runs FIRST so the
// failure is a clear, early, build-level message rather than a Typst stack trace, and so the
// evidence file itself is validated even before any paper references a given key.
function runHonestyGate() {
  const raw = JSON.parse(readFileSync(EVIDENCE_PATH, "utf8"));
  const records = raw.records || {};
  const VALID_ENV = new Set(["canonical", "indicative"]);
  const problems = [];
  for (const [key, r] of Object.entries(records)) {
    if (!VALID_ENV.has(r.environment)) {
      problems.push(
        `record '${key}' has environment='${r.environment}' — must be 'canonical' or 'indicative'`,
      );
    }
    if (!r.source || typeof r.source !== "string") {
      problems.push(`record '${key}' is missing a 'source' (every number must trace to a real test/dataset)`);
    }
    if (r.value === undefined) {
      problems.push(`record '${key}' is missing a 'value'`);
    }
  }
  if (problems.length) {
    console.error("\n[paper-factory] HONESTY GATE FAILED:\n  - " + problems.join("\n  - ") + "\n");
    process.exit(1);
  }
  const nCanonical = Object.values(records).filter((r) => r.environment === "canonical").length;
  const nIndicative = Object.values(records).length - nCanonical;
  console.log(
    `[paper-factory] honesty gate passed: ${Object.keys(records).length} evidence records ` +
      `(${nCanonical} canonical, ${nIndicative} indicative).`,
  );
}

// ---- compile one paper to PDF + HTML ------------------------------------------------------
function compilePaper(typst, paper, evidenceJson) {
  const typPath = join(PAPERS_DIR, paper.source);
  if (!existsSync(typPath)) {
    throw new Error(`build-papers: paper source not found: ${typPath}`);
  }
  const pdfOut = join(PDF_OUT_DIR, `${paper.slug}.pdf`);
  const htmlOut = join(HTML_OUT_DIR, `${paper.slug}.html`);
  const common = ["--root", SITE, "--input", `data=${evidenceJson}`];

  // PDF (the download). A headline() gate violation panics here and aborts the build.
  execFileSync(typst, ["compile", typPath, pdfOut, ...common], { stdio: ["ignore", "ignore", "inherit"] });

  // HTML (the in-site render) — Typst native HTML export. `--features html` is required; the
  // experimental-feature + page-rule warnings on stderr are expected and harmless.
  execFileSync(
    typst,
    ["compile", typPath, htmlOut, "--format", "html", "--features", "html", ...common],
    { stdio: ["ignore", "ignore", "inherit"] },
  );

  // Extract just the <body> inner HTML so the React route can inject it as a fragment.
  const full = readFileSync(htmlOut, "utf8");
  const body = full.match(/<body[^>]*>([\s\S]*)<\/body>/i);
  const fragment = body ? body[1] : full;
  writeFileSync(htmlOut, fragment, "utf8");

  console.log(`[paper-factory] built ${paper.slug}: ${pdfOut} + ${htmlOut}`);
}

// ---- main ---------------------------------------------------------------------------------
function main() {
  runHonestyGate();

  const typst = resolveTypst();
  const papers = readRegistry();

  mkdirSync(PDF_OUT_DIR, { recursive: true });
  mkdirSync(HTML_OUT_DIR, { recursive: true });

  if (!typst) {
    // Graceful degradation: a contributor without Typst installed can still run the site.
    // CI MUST have Typst (the workflow installs it) so the real PDFs/HTML are produced; here
    // we emit honest placeholders so `next build` does not hard-fail on a missing binary, and
    // we surface a loud warning.
    console.warn(
      "\n[paper-factory] WARNING: `typst` not found — emitting placeholder paper artifacts.\n" +
        "  Install Typst 0.15+ (https://github.com/typst/typst/releases) so real PDFs/HTML build.\n" +
        "  CI installs it; this fallback is for local dev without Typst only.\n",
    );
    for (const p of papers) {
      const placeholder = `<p class="paper-placeholder">This paper renders from <code>papers/${p.source}</code>. ` +
        `Install the Typst CLI to build it locally; CI builds it automatically.</p>`;
      writeFileSync(join(HTML_OUT_DIR, `${p.slug}.html`), placeholder, "utf8");
      // a 1-line text PDF stand-in is not produced; the route guards a missing PDF asset.
    }
    return;
  }

  const evidenceJson = readFileSync(EVIDENCE_PATH, "utf8");
  for (const p of papers) compilePaper(typst, p, evidenceJson);

  console.log(`[paper-factory] done: ${papers.length} paper(s).`);
}

main();
