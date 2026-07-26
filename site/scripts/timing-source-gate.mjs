// [OPUS-5] sq-gum8.16 (paper factory F4) — the PUBLICATION-BOUNDARY source gate for measured
// timings.
//
// WHY THIS EXISTS: `site/papers/_lib/timing.typ` promises that a measured wall-clock number can
// only reach a page WITH its provenance. Typst cannot enforce that on its own — it has no
// private module bindings, so every top-level `#let` in that lib (including the parsed dataset
// and the internal `_trec` lookup) is importable, and any paper can also just `json()` the
// generated file itself. Both are bare-value escape hatches around `headline_timing()`.
//
// So the invariant is enforced HERE, mechanically, over the paper sources the factory is about
// to compile: a paper may import ONLY the provenance-rendering entry points from the timing
// lib, may not import it as a whole module, may not touch its internals, and may not read the
// generated timing JSON directly. Anything else FAILS THE BUILD (build-papers.mjs) — the
// fail-closed direction, because the alternative is publishing a number stripped of the
// machine, corpus, aggregate and commit that make it meaningful.
//
// HONEST SCOPE: this is a SOURCE-TEXT check over `site/papers/**/*.typ`. It bounds the paper
// surface the factory compiles; it is not a Typst-level capability system, and it says nothing
// about whether a rendered number is FRAMED honestly (that stays the Stage-5 claims↔evidence
// review in `skills/academic-paper/SKILL.md`).

import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";

// The only bindings a paper may import from the timing lib. `timing_keys` yields record KEYS
// (never values), so it cannot leak a bare number; the other two always render provenance.
export const ALLOWED_TIMING_IMPORTS = new Set([
  "headline_timing",
  "timing_provenance",
  "timing_keys",
]);

// The lib itself and its compile self-check are the implementation of the boundary, not
// consumers of it, so they are the only files exempt from the scan.
const EXEMPT = new Set(["_lib/timing.typ", "_lib/timing-selfcheck.typ"]);

const GENERATED_TIMING_FILE = /paper-timing\.generated/;
// `#import "…/timing.typ"` / `#include "…"`, capturing the optional `: a, b` binding list.
const TIMING_IMPORT = /#(import|include)\s+"([^"]*\btiming\.typ)"\s*(?::([^\n]*))?/g;
// Internal (convention-private) bindings of the lib. Importing or naming any of them in a paper
// is an attempt to read a record without rendering its provenance.
const INTERNALS = /\b(_timing_data|_trec|_envelope|_fmt_us|_host_short)\b/;

function walkTyp(dir, root = dir, out = []) {
  if (!existsSync(dir)) return out;
  for (const entry of readdirSync(dir).sort()) {
    const p = join(dir, entry);
    if (statSync(p).isDirectory()) walkTyp(p, root, out);
    else if (entry.endsWith(".typ")) out.push(relative(root, p));
  }
  return out;
}

// Typst allows `#import "x": a, b as c, *`. Take the name actually bound FROM the module (the
// part before `as`), and treat a wildcard as importing everything, internals included.
function importedNames(list) {
  return list
    .split(",")
    .map((s) => s.trim().split(/\s+as\s+/)[0].trim())
    .filter(Boolean);
}

/**
 * Audit every paper source under `papersDir` for a bypass of the forced-provenance boundary.
 * Returns `{ scanned, problems }`; `problems` is empty iff the boundary holds.
 */
export function auditTimingSources(papersDir) {
  const problems = [];
  const files = walkTyp(papersDir).filter((f) => !EXEMPT.has(f.split(/[\\/]/).join("/")));
  for (const rel of files) {
    const src = readFileSync(join(papersDir, rel), "utf8");
    if (GENERATED_TIMING_FILE.test(src)) {
      problems.push(
        `${rel}: reads the derived timing file (paper-timing.generated.json) directly. A ` +
          "measured number may only render through headline_timing()/timing_provenance(), " +
          "which print the provenance beside it.",
      );
    }
    if (INTERNALS.test(src)) {
      problems.push(
        `${rel}: names an internal binding of _lib/timing.typ (_timing_data / _trec / ...). ` +
          "Those return raw records; use headline_timing(key) or timing_provenance(key).",
      );
    }
    for (const m of src.matchAll(TIMING_IMPORT)) {
      const [, form, path, list] = m;
      if (list === undefined) {
        problems.push(
          `${rel}: \`#${form} "${path}"\` pulls in the whole timing module, which exposes the ` +
            "raw dataset. Import only: " + [...ALLOWED_TIMING_IMPORTS].join(", ") + ".",
        );
        continue;
      }
      for (const name of importedNames(list)) {
        if (!ALLOWED_TIMING_IMPORTS.has(name)) {
          problems.push(
            `${rel}: imports '${name}' from ${path}. Only ` +
              [...ALLOWED_TIMING_IMPORTS].join(", ") +
              " may be imported — every other binding can yield a timing without its provenance.",
          );
        }
      }
    }
  }
  return { scanned: files, problems };
}
