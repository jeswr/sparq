// [OPUS-4.8] sq-gum8 — shared paper-factory helpers. ONE source of numbers.
//
// The build (site/scripts/build-papers.mjs / the typst CLI) injects the paper-bound
// evidence JSON via `--input data=...`. Every paper reads it through `ev(...)` below and
// NEVER hard-codes a number — so the PDF and the in-site HTML cannot disagree, and a paper
// auto-updates when the evidence refreshes.
//
// HONESTY GATE (enforced in build-papers.mjs, mirrored here as a hard fail):
// `headline(key)` is the ONLY accessor allowed inside a headline result table/figure. It
// REFUSES to render a record whose `environment` is not "canonical" — so a non-canonical
// (work-box / indicative) number can never appear as a paper's headline evidence. Use the
// plain `ev(key)` accessor (no gate) only for clearly-labelled "indicative" callouts.

#let evidence = json(bytes(sys.inputs.data))
#let anon = sys.inputs.at("anon", default: "false") == "true"

// Raw record lookup (panics loudly if the key is missing — a typo must fail the build,
// never silently render an empty string).
#let _rec(key) = {
  let r = evidence.records.at(key, default: none)
  if r == none {
    panic("paper-factory: unknown evidence key '" + key + "' — add it to site/src/data/paper-evidence.json")
  }
  r
}

// Plain value accessor — NOT gated. Only for explicitly-"indicative"-labelled callouts.
#let ev(key) = _rec(key).value

// HEADLINE accessor — the honesty gate. Returns the value ONLY if the record is canonical.
#let headline(key) = {
  let r = _rec(key)
  if r.environment != "canonical" {
    panic(
      "paper-factory honesty gate: headline('" + key + "') references an environment='" +
      r.environment + "' record. Headline tables may cite ONLY environment='canonical' " +
      "numbers (deterministic, machine-independent). Move it to an 'indicative' callout " +
      "or supply a canonical record.",
    )
  }
  r.value
}

// A provenance line for a headline number (source test + environment).
#let provenance(key) = {
  let r = _rec(key)
  text(size: 0.85em, fill: gray)[evidence: `#r.source` (environment: #r.environment)]
}

// Author block honours the double-blind anon toggle.
#let authors() = if anon {
  align(center)[#text(style: "italic")[Author(s) omitted for double-blind review]]
} else {
  align(center)[Jesse Wright · the sparq project]
}
