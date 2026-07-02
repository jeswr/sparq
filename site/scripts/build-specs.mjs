// [OPUS-4.8] sq-rvgr2.1 — the spec-factory build step, the /specs analogue of build-papers.mjs.
//
// For every spec registered in src/data/specs.ts, this:
//   1. runs a build-boundary HONESTY SCAN (the shared no-perf-numbers gate) over the exact
//      spec .typ sources, so a spec can never smuggle a hard-coded performance number, then
//   2. compiles the spec's .typ to BOTH a PDF (public/specs/<slug>.pdf — the download) and a
//      semantic HTML fragment (src/generated/specs/<slug>.html — the in-site render), injecting
//      the spec's metadata JSON via `--input meta=...` so the doc header cannot drift from the
//      /specs index card, then
//   3. POST-PROCESSES the exported HTML into ReSpec/TR conventions: it slugs every NUMBERED
//      heading into a stable `id` (so cross-references + the ToC resolve, and lychee's fragment
//      check passes) and builds a linked Table of Contents from them.
//
// It is wired into `prebuild` (after build-papers) so `next build` always regenerates the
// specs, and into `dev`. The typst binary is resolved from PATH / ~/.local/bin / ~/.cargo/bin;
// CI installs it (same pinned Typst the paper factory uses). When typst is absent (a local dev
// box) we emit an honest placeholder fragment so `next build` never hard-fails — exactly the
// graceful degradation the paper factory uses.
//
// The page route imports the generated HTML fragment at BUILD TIME (readFileSync), so the
// in-site render is a static asset — no ReSpec/puppeteer/Chromium and no client JS shipped.
// (This is why we bake with Typst rather than the ReSpec CLI: Typst is already a pinned CI
// dependency for the paper factory, so /specs adds ZERO new CI surface and the GitHub Pages
// build cannot be broken by a missing headless browser.)

import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { homedir } from "node:os";

const __dirname = dirname(fileURLToPath(import.meta.url));
const SITE = resolve(__dirname, "..");
const REPO_ROOT = resolve(SITE, ".."); // to invoke the shared honesty gate.
const SPECS_DIR = join(SITE, "specs");
const PDF_OUT_DIR = join(SITE, "public", "specs");
const HTML_OUT_DIR = join(SITE, "src", "generated", "specs");

// Human status labels — mirrors STATUS_LABEL in src/data/specs.ts. The injected `statusLabel`
// is what the doc header renders; keeping the mapping here lets the .mjs stay TS-toolchain-free.
const STATUS_LABEL = {
  unofficial: "Unofficial Proposal Draft",
  "cg-draft": "Draft Community Group Report",
  draft: "Editor's Draft",
};

// [OPUS-4.8] anti-drift "GENERATED at build time" header for the fragments under
// src/generated/specs/. Git-ignored (site/.gitignore: /src/generated/); never hand-edit.
function generatedHeader(source) {
  return (
    `<!-- GENERATED at build time by site/scripts/build-specs.mjs from specs/${source} -->\n` +
    `<!-- DO NOT EDIT: regenerate with \`npm run build-specs\` (or \`npm run build\`). -->\n`
  );
}

// ---- resolve the typst binary (same policy as build-papers.mjs) ---------------------------
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

// ---- the registry (parsed from specs.ts without importing TS) -----------------------------
// specs.ts is the single source of truth; we read the per-spec fields via parallel-array
// regexes (the same technique build-papers.mjs uses for slug+source). Every SPEC object MUST
// declare these fields, single-line, in this order — asserted below by the length check.
function readRegistry() {
  const src = readFileSync(join(SITE, "src", "data", "specs.ts"), "utf8");
  const field = (name) => [...src.matchAll(new RegExp(`${name}:\\s*"([^"]*)"`, "g"))].map((m) => m[1]);
  const slugs = field("slug");
  const sources = field("source");
  const titles = field("title");
  const shortNames = field("shortName");
  const statuses = field("status");
  const dates = field("date");
  const editors = field("editors");
  const n = slugs.length;
  const lens = { sources, titles, shortNames, statuses, dates, editors };
  for (const [k, v] of Object.entries(lens)) {
    if (v.length !== n) {
      throw new Error(
        `build-specs: could not parse specs.ts — field '${k}' count ${v.length} != slug count ${n}. ` +
          "Every SPEC must declare slug/source/title/shortName/status/date/editors as single-line string fields.",
      );
    }
  }
  if (n === 0) throw new Error("build-specs: specs.ts declares no specs");
  return slugs.map((slug, i) => ({
    slug,
    source: sources[i],
    title: titles[i],
    shortName: shortNames[i],
    status: statuses[i],
    date: dates[i],
    editors: editors[i],
  }));
}

// ---- build-boundary honesty scan (the two shared honesty gates over the spec sources) ------
// [OPUS-4.8] sq-rvgr2.5: FAIL-CLOSED, and now BOTH shared CI honesty gates run here — the same
// pattern as build-papers.mjs's runBuildBoundaryHonestyScan — so the spec factory can never
// serve an un-scanned or violating draft before the ZK/MPC spec CONTENT (zkSPARQL / MPC-SPARQL,
// beads sq-rvgr2.2/.3) lands:
//   - scripts/check-no-perf-numbers.py --enforce <each spec .typ>  (a hard-coded perf figure in
//        a spec source aborts the build — a spec is a DESIGN surface, not a benchmark), and
//   - scripts/check-privacy-claims.sh  (whole-tree; its git-ls-files surface now includes
//        site/specs/**/*.typ — see the sq-rvgr2.5 note in that gate), so an unqualified ZK/MPC
//        soundness/privacy claim typed into spec prose aborts the build too.
// "Shared" means these are the SAME gate scripts CI runs — not a copy — so the build boundary and
// CI stay in lockstep. The two gates DO NOT share one phrase list: the privacy-claims gate reads
// scripts/honesty-phrases.json (its single source of truth, also used by CI), while the perf gate
// carries its own PERF_PATTERNS in check-no-perf-numbers.py. A non-zero gate exit aborts the build
// (no artifact is written). HONEST SCOPE: this is the COARSE phrase/number class only; a subtle
// semantic overclaim remains Stage-5 human review.
function runHonestyScan(specs) {
  const perfGate = join(REPO_ROOT, "scripts", "check-no-perf-numbers.py");
  const privacyGate = join(REPO_ROOT, "scripts", "check-privacy-claims.sh");
  const sharedList = join(REPO_ROOT, "scripts", "honesty-phrases.json");
  for (const p of [perfGate, privacyGate, sharedList]) {
    if (!existsSync(p)) {
      console.error(`\n[spec-factory] HONESTY SCAN: required gate missing: ${p}\n`);
      process.exit(1);
    }
  }
  const typPaths = specs.map((s) => join(SPECS_DIR, s.source)).filter((t) => existsSync(t));

  const run = (label, cmd, cmdArgs) => {
    try {
      // inherit stderr so a finding is visible; capture nothing — the gate self-reports.
      execFileSync(cmd, cmdArgs, { cwd: REPO_ROOT, stdio: ["ignore", "inherit", "inherit"] });
    } catch (e) {
      const code = typeof e.status === "number" ? e.status : 1;
      console.error(
        `\n[spec-factory] BUILD-BOUNDARY HONESTY SCAN FAILED (${label}, exit ${code}).\n` +
          "  A spec .typ source carries a hard-coded performance number or an unqualified ZK/MPC\n" +
          "  soundness/privacy claim. A spec is a design surface, not a benchmark — remove the\n" +
          "  number; and hedge the claim (or add the inline allow marker). The factory will not\n" +
          "  serve an un-scanned spec. (Privacy-gate phrase list: scripts/honesty-phrases.json;\n" +
          "  perf-gate patterns: PERF_PATTERNS in scripts/check-no-perf-numbers.py.)\n",
      );
      process.exit(1);
    }
  };

  // Perf gate over the exact spec sources (explicit paths => --enforce dispatch to the
  // accessor-aware Typst scan). Privacy gate whole-tree (it enumerates its surface via
  // git ls-files, which now includes site/specs/**/*.typ).
  run("no-perf-numbers", "python3", [perfGate, "--enforce", ...typPaths]);
  run("privacy-claims", "bash", [privacyGate]);

  console.log(
    `[spec-factory] honesty scan passed: ${typPaths.length} spec source(s) scanned by both ` +
      "honesty gates (perf-number + privacy-claim).",
  );
}

// ---- HTML post-processing: inject heading ids + build a linked Table of Contents -----------
// Typst's HTML export is bare (h2/h3 with a "N." text prefix, no ids, no ToC). We turn it into
// ReSpec/TR conventions: every NUMBERED heading gets a stable slug `id`, and a `<nav id="toc">`
// is built from those headings and inserted before the first one. Unnumbered headings (Abstract,
// Status-of-This-Document) sit above the ToC and are intentionally NOT listed in it, matching
// the ReSpec layout.
const NUMBERED = /^\s*(\d+(?:\.\d+)*)\.\s+(.*)$/; // "1. Intro" / "2.3. Foo"

function stripTags(html) {
  return html
    .replace(/<[^>]+>/g, "")
    .replace(/&amp;/g, "&")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .trim();
}

function slugify(text, used) {
  let base = text
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
  if (!base) base = "section";
  let slug = base;
  let i = 2;
  while (used.has(slug)) slug = `${base}-${i++}`;
  used.add(slug);
  return slug;
}

export function injectTocAndIds(body) {
  const used = new Set();
  const toc = []; // { level, num, id, text }
  let firstNumberedIndex = -1;

  // Walk headings in document order. Only h2/h3 that carry a "N." numbering prefix are spec
  // sections; give each a slug id (unless it already has one) and record it for the ToC.
  const withIds = body.replace(
    /<(h[23])((?:\s[^>]*)?)>([\s\S]*?)<\/\1>/g,
    (match, tag, attrs, inner, offset) => {
      const text = stripTags(inner);
      const m = NUMBERED.exec(text);
      if (!m) return match; // unnumbered heading (Abstract / SOTD / the ToC's own title)
      if (firstNumberedIndex === -1) firstNumberedIndex = offset;
      const hasId = /\sid\s*=/.test(attrs);
      const label = m[2].trim();
      const slug = hasId
        ? (/\sid\s*=\s*"([^"]*)"/.exec(attrs)?.[1] ?? slugify(label, used))
        : slugify(label, used);
      // Record EVERY resolved id in `used` — including a heading's pre-existing custom id — so a
      // later auto-generated slug can never collide with it (a duplicate id breaks ToC anchors).
      // (In the slugify branch the id is already recorded; this add is then a harmless no-op.)
      used.add(slug);
      toc.push({ level: tag === "h2" ? 2 : 3, num: m[1], id: slug, text: label });
      const newAttrs = hasId ? attrs : `${attrs} id="${slug}"`;
      return `<${tag}${newAttrs}>${inner}</${tag}>`;
    },
  );

  if (toc.length === 0) return withIds; // nothing numbered — leave as-is

  // Build a nested <ol> from the (flat, ordered) heading list by level.
  let html = '<nav id="toc" class="spec-toc" aria-label="Table of Contents">';
  html += "<h2>Table of Contents</h2><ol>";
  let curLevel = 2;
  for (const e of toc) {
    if (e.level > curLevel) {
      html += "<ol>".repeat(e.level - curLevel);
    } else if (e.level < curLevel) {
      html += "</li>" + "</ol></li>".repeat(curLevel - e.level);
    } else if (html.endsWith("</li>") === false && !html.endsWith("<ol>")) {
      html += "</li>";
    }
    html += `<li><a href="#${e.id}"><span class="toc-num">${e.num}.</span> ${e.text}</a>`;
    curLevel = e.level;
  }
  html += "</li>";
  while (curLevel > 2) {
    html += "</ol></li>";
    curLevel--;
  }
  html += "</ol></nav>";

  // Insert the ToC immediately before the first numbered heading, reusing the byte offset already
  // captured during the walk. Everything before that heading is untouched by the id injection, so
  // the offset (taken in `body`) is still valid in `withIds`; and toc.length > 0 guarantees it was
  // set. This avoids a second full-string scan and keeps the insertion point in one place.
  return withIds.slice(0, firstNumberedIndex) + html + withIds.slice(firstNumberedIndex);
}

// ---- compile one spec to PDF + HTML -------------------------------------------------------
function compileSpec(typst, spec) {
  const typPath = join(SPECS_DIR, spec.source);
  if (!existsSync(typPath)) throw new Error(`build-specs: spec source not found: ${typPath}`);

  const meta = JSON.stringify({
    title: spec.title,
    statusLabel: STATUS_LABEL[spec.status] ?? "Unofficial Proposal Draft",
    shortName: spec.shortName,
    editors: spec.editors,
    date: spec.date,
  });
  const pdfOut = join(PDF_OUT_DIR, `${spec.slug}.pdf`);
  const htmlOut = join(HTML_OUT_DIR, `${spec.slug}.html`);
  const common = ["--root", SITE, "--input", `meta=${meta}`];

  // PDF (the download).
  execFileSync(typst, ["compile", typPath, pdfOut, ...common], { stdio: ["ignore", "ignore", "inherit"] });

  // HTML (the in-site render) — Typst native HTML export. The experimental-feature warnings on
  // stderr are expected and harmless.
  execFileSync(
    typst,
    ["compile", typPath, htmlOut, "--format", "html", "--features", "html", ...common],
    { stdio: ["ignore", "ignore", "inherit"] },
  );

  const full = readFileSync(htmlOut, "utf8");
  const bodyMatch = full.match(/<body[^>]*>([\s\S]*)<\/body>/i);
  const body = bodyMatch ? bodyMatch[1] : full;
  const fragment = injectTocAndIds(body);
  writeFileSync(htmlOut, generatedHeader(spec.source) + fragment, "utf8");

  console.log(`[spec-factory] built ${spec.slug}: ${pdfOut} + ${htmlOut}`);
}

// ---- main ---------------------------------------------------------------------------------
function main() {
  const specs = readRegistry();
  runHonestyScan(specs);

  mkdirSync(PDF_OUT_DIR, { recursive: true });
  mkdirSync(HTML_OUT_DIR, { recursive: true });

  const typst = resolveTypst();
  if (!typst) {
    console.warn(
      "\n[spec-factory] WARNING: `typst` not found — emitting placeholder spec artifacts.\n" +
        "  Install Typst 0.15+ (https://github.com/typst/typst/releases) so real PDFs/HTML build.\n" +
        "  CI installs it; this fallback is for local dev without Typst only.\n",
    );
    for (const s of specs) {
      const placeholder = `<p class="spec-placeholder">This specification renders from <code>specs/${s.source}</code>. ` +
        `Install the Typst CLI to build it locally; CI builds it automatically.</p>`;
      writeFileSync(join(HTML_OUT_DIR, `${s.slug}.html`), generatedHeader(s.source) + placeholder, "utf8");
    }
    return;
  }

  for (const s of specs) compileSpec(typst, s);
  console.log(`[spec-factory] done: ${specs.length} spec(s).`);
}

// Only run when invoked as a script (so the post-processor can be unit-tested via import).
if (process.argv[1] && resolve(process.argv[1]) === resolve(fileURLToPath(import.meta.url))) {
  main();
}
