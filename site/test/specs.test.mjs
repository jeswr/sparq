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

// [GPT-5.6] sq-tag1q.4 — pin the acceptance-critical SPARQL-CRDT design boundaries. These
// assertions are deliberately exact enough that weakening the convergence claim, omitting
// skolemisation, or moving WHERE evaluation to receivers makes the test fail.
test("sparql-crdt: registered with the required convergence and origin-evaluation boundaries", () => {
  const spec = specBySlug("sparql-crdt");
  assert.ok(spec, "specBySlug('sparql-crdt') must resolve");
  const src = readFileSync(join(SITE, "specs", spec.source), "utf8");
  const compact = src.replace(/\s+/g, " ");

  assert.match(compact, /SU-Set and Live Linked Data/, "the SU-Set prior-art section must remain");
  assert.match(compact, /== m-ld/, "the m-ld prior-art section must remain");
  assert.match(compact, /== NextGraph/, "the NextGraph prior-art section must remain");
  assert.match(
    compact,
    /Every blank node entering the replicated dataset.*MUST.*replaced at the origin boundary.*Skolem IRI/,
    "blank-node identity must be resolved through mandatory origin skolemisation",
  );
  assert.match(
    compact,
    /WHERE` group graph pattern.*MUST.*evaluated exactly once at the origin/,
    "pattern updates must use evaluate-at-origin semantics",
  );
  assert.match(
    compact,
    /strong eventual consistency:.*same set of valid deltas.*same materialised quad set/,
    "the precise dataset-level SEC claim must remain",
  );
  assert.match(
    compact,
    /MUST NOT.*preservation of the source.*pattern-based update/,
    "the draft must not overclaim intention preservation",
  );
  assert.match(
    compact,
    /#dfn\[replica\].*#dfn\[delta-relay\].*#dfn\[origin-evaluator\]/,
    "all three conformance classes must remain",
  );
  assert.match(
    compact,
    /Version-1 CRDT metadata.*MUST.*stored and exchanged out of band/,
    "the out-of-band journal/sidecar decision must remain explicit",
  );
});

// [OPUS-5] sq-tag1q.5 / issue #2548 — pin the E2EE-SPARQL draft's LOAD-BEARING honesty
// clauses. Each assertion below is a claim a future edit could quietly soften, and softening
// any of them would be an honesty defect rather than a wording change: the impossibility
// statement (there is no leakage-free server-side SPARQL over E2EE data), the fact that only
// Profile SE evaluates server-side and that it discloses the whole graph structure, the
// no-forward-secrecy disclosure, the rejection of deterministic/order-revealing value
// encryption, the prohibition on server-side decryption, and the open external-audit gate.
test("e2ee-sparql: registered, and its load-bearing honesty clauses are present", () => {
  const spec = specBySlug("e2ee-sparql");
  assert.ok(spec, "specBySlug('e2ee-sparql') must resolve");
  const src = readFileSync(join(SITE, "specs", spec.source), "utf8");
  const compact = src.replace(/\s+/g, " ");

  // The negative result is the spine of the document and must stay in the BODY.
  assert.match(
    compact,
    /General server-side SPARQL evaluation over end-to-end-encrypted data, without\s*leakage, does not exist/,
    "the impossibility statement must remain in the document body",
  );
  // Leakage must remain a declarable, normative vocabulary.
  assert.match(
    compact,
    /MUST\] declare exactly one leakage tier from `T0`–`T4`/,
    "every profile must be required to declare a T0-T4 leakage tier",
  );
  // Exactly three profiles, with CS mandatory.
  assert.match(
    compact,
    /specifies exactly three profiles: #strong\[CS\], #strong\[BR\], and #strong\[SE\]/,
    "the three-profile scope statement must remain",
  );
  // BR is the NextGraph-shaped profile and MUST NOT be described as server-side query.
  assert.match(
    compact,
    /Query evaluation is #strong\[ALWAYS\] local; the relay #strong\[MUST NOT\] evaluate queries/,
    "Profile BR must keep query evaluation local — the relay never evaluates",
  );
  // SE is the server-side-query profile, and its cost is stated, not buried.
  assert.match(
    compact,
    /Profile SE declares tier\s*#strong\[`T1`\]/,
    "Profile SE must declare the T1 (full-structure) leakage tier",
  );
  assert.match(
    compact,
    /full graph structure and predicate vocabulary are visible to the server/,
    "Profile SE's mandatory leakage statement must name the structural disclosure",
  );
  assert.match(
    compact,
    /protects the #emph\[values\], not\s*the #emph\[shape of the user's life\]/,
    "the plain-language Profile SE warning must remain",
  );
  // Equality tags stay a separate, disclosed opt-in — never a default.
  assert.match(
    compact,
    /Equality tags #strong\[MUST NOT\] be enabled by default/,
    "equality tags must remain a separately-disclosed opt-in, not a default",
  );
  // No forward secrecy / no post-compromise security, in any profile.
  assert.match(
    compact,
    /#strong\[no forward secrecy\] and #strong\[no post-compromise\s*security\] in any profile/,
    "the no-FS / no-PCS disclosure must remain",
  );
  // The rejections must stay rejections, on the attack record.
  assert.match(
    compact,
    /Deterministic and order-revealing value encryption are REJECTED\], not deferred/,
    "DET/ORE must remain rejected rather than deferred",
  );
  assert.match(
    compact,
    /No server-side decryption, ever/,
    "the prohibition on server-side decryption must remain",
  );
  // The audit gate must remain open and named, and no proven property may be claimed.
  assert.match(src, /sq-qhy4/, "the sq-qhy4 external-audit-pending gate must be named");
  assert.match(
    compact,
    /MUST NOT\]\s*assert a proven cryptographic-confidentiality, integrity, forward-secrecy, or\s*post-compromise-security property/,
    "the no-proven-property clause must remain",
  );
  // A spec is a design surface, not a benchmark.
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
