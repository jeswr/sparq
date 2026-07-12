// [OPUS-4.8] sq-rvgr2.1 — unit tests for the spec factory.
//   1. The registry (src/data/specs.ts) round-trips: unique slugs, every source .typ exists,
//      every status has a human label. This mirrors the guarantee build-specs.mjs relies on.
//   2. The HTML post-processor (injectTocAndIds in scripts/build-specs.mjs) turns bare Typst
//      HTML into ReSpec/TR conventions: numbered headings get stable ids, a linked Table of
//      Contents is built from them, and unnumbered headings (Abstract / SOTD) are excluded.
// Run via `npm run test:unit`.
import { test } from "node:test";
import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

import { SPECS, specBySlug, STATUS_LABEL } from "../src/data/specs.ts";
import { injectTocAndIds } from "../scripts/build-specs.mjs";

const SITE = dirname(fileURLToPath(import.meta.url)).replace(/\/test$/, "");

test("the specs registry round-trips: unique slugs, existing sources, labelled statuses", () => {
  assert.ok(SPECS.length >= 1, "at least one spec must be registered");
  const slugs = new Set();
  for (const s of SPECS) {
    assert.ok(!slugs.has(s.slug), `duplicate slug: ${s.slug}`);
    slugs.add(s.slug);
    assert.ok(
      existsSync(join(SITE, "specs", s.source)),
      `spec source missing: specs/${s.source}`,
    );
    assert.ok(STATUS_LABEL[s.status], `spec ${s.slug} has an unlabelled status: ${s.status}`);
    assert.equal(specBySlug(s.slug), s, "specBySlug must resolve every registered slug");
  }
});

test("specBySlug returns undefined for an unknown slug", () => {
  assert.equal(specBySlug("does-not-exist"), undefined);
});

// [FABLE-5] sq-q4jdu — the trust-graph-authz draft's honesty caveats are LOAD-BEARING
// (the bead's acceptance gate): the source must (a) resolve from the registry, (b) carry
// the explicit no-ZK/privacy/unlinkability disclaimer and the sq-qhy4 external-audit-pending
// caveat, and (c) hard-code no timing/perf figure (the spec factory's perf gate enforces the
// number class; this pins the unit-level regression for the phrases).
test("trust-graph-authz: registered, and its honesty caveats are present in the source", () => {
  const spec = specBySlug("trust-graph-authz");
  assert.ok(spec, "specBySlug('trust-graph-authz') must resolve");
  const src = readFileSync(join(SITE, "specs", spec.source), "utf8");
  assert.match(
    src,
    /no ZK, privacy, or unlinkability claim/,
    "the explicit 'no ZK, privacy, or unlinkability claim' disclaimer must be present",
  );
  assert.match(
    src,
    /sq-qhy4/,
    "the sq-qhy4 external-audit-pending caveat must be present",
  );
  assert.match(src, /clear-path/i, "the clear-path-only framing must be present");
  // No hard-coded timing figures (ms/µs/ns latencies or throughput-per-second numbers).
  assert.doesNotMatch(
    src,
    /\d+(?:\.\d+)?\s*(?:ns|µs|us|ms)\b|\d+\s*(?:ops|queries|req|triples)\/s/i,
    "a spec is a design surface, not a benchmark — no hard-coded timing figures",
  );
});

test("injectTocAndIds slugs numbered headings and builds a linked ToC", () => {
  const body =
    '<section class="introductory" id="abstract"><h2>Abstract</h2><p>a</p></section>' +
    "<h2>1. Introduction</h2><p>x</p>" +
    "<h2>2. Terminology</h2><p>y</p>" +
    "<h3>2.1. Sub term</h3><p>z</p>" +
    "<h2>3. References</h2><p>r</p>";
  const out = injectTocAndIds(body);

  // Numbered headings get slug ids derived from their text (number stripped).
  assert.match(out, /<h2 id="introduction">1\. Introduction<\/h2>/);
  assert.match(out, /<h2 id="terminology">2\. Terminology<\/h2>/);
  assert.match(out, /<h3 id="sub-term">2\.1\. Sub term<\/h3>/);
  assert.match(out, /<h2 id="references">3\. References<\/h2>/);

  // A ToC nav is emitted, with anchors to those ids and the section numbers.
  assert.match(out, /<nav id="toc"[^>]*>/);
  assert.match(out, /<a href="#introduction">/);
  assert.match(out, /<a href="#terminology">/);
  assert.match(out, /<a href="#sub-term">/);
  assert.match(out, /<a href="#references">/);

  // The Abstract heading is unnumbered → it must NOT get an id and NOT appear in the ToC.
  assert.match(out, /<section class="introductory" id="abstract"><h2>Abstract<\/h2>/);
  const toc = out.match(/<nav id="toc"[\s\S]*?<\/nav>/)[0];
  assert.doesNotMatch(toc, /Abstract/);

  // The ToC is inserted BEFORE the first numbered heading (after the Abstract section).
  assert.ok(out.indexOf('<nav id="toc"') < out.indexOf('id="introduction"'));
  assert.ok(out.indexOf("</section>") < out.indexOf('<nav id="toc"'));

  // The nested h3 is nested under an <ol> inside its parent h2 list item (a sub-list opened
  // right after the parent's anchor, before the parent <li> closes).
  assert.match(toc, /Terminology<\/a><ol><li><a href="#sub-term">/);
});

test("injectTocAndIds leaves an all-unnumbered fragment untouched", () => {
  const body = "<h2>Abstract</h2><p>only intro material, nothing numbered</p>";
  assert.equal(injectTocAndIds(body), body);
});

test("injectTocAndIds preserves a pre-existing id on a numbered heading", () => {
  const body = '<h2 id="custom">1. Kept</h2>';
  const out = injectTocAndIds(body);
  assert.match(out, /<h2 id="custom">1\. Kept<\/h2>/);
  assert.match(out, /<a href="#custom">/);
});

test("injectTocAndIds never lets an auto-slug collide with a pre-existing custom id", () => {
  // First heading claims the id "intro" explicitly; a LATER heading titled "Intro" would slugify
  // to the same "intro" unless the pre-existing id is recorded. The auto-slug must be disambiguated.
  const body = '<h2 id="intro">1. Preface</h2>' + "<h2>2. Intro</h2>";
  const out = injectTocAndIds(body);
  assert.match(out, /<h2 id="intro">1\. Preface<\/h2>/);
  assert.match(out, /<h2 id="intro-2">2\. Intro<\/h2>/);
  // Both anchors are distinct in the ToC.
  assert.match(out, /<a href="#intro">/);
  assert.match(out, /<a href="#intro-2">/);
});
