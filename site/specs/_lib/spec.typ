// [OPUS-4.8] sq-rvgr2.1 — shared spec-factory helpers, the /specs analogue of papers/_lib.
//
// A spec is authored ONCE as a Typst source under site/specs/ and built into BOTH a
// downloadable PDF and an in-site HTML render (scripts/build-specs.mjs), exactly like the
// paper factory. The single-source discipline means the PDF and the in-site render cannot
// disagree.
//
// W3C ReSpec / TR CONVENTIONS. These helpers emit the ReSpec-style structure the bead asks
// for. Typst HTML export is bare (h2/h3 + text numbering, no ids, no ToC), so we use Typst's
// `#context target()` switch to emit RICH, class-annotated HTML (via `html.elem`) in the HTML
// target and nicely-styled blocks in the PDF target from ONE source. The heading ids + the
// linked Table of Contents are injected by scripts/build-specs.mjs as a post-processing pass
// over the exported HTML (it slugs every NUMBERED heading and builds the ToC from them).
//
// HONESTY. The Status-of-This-Document boilerplate (`sotd()`) states plainly that the
// document is an Unofficial Proposal Draft with NO W3C standing — it must never claim to be a
// standard. Editors / title / status / date / shortName come from the injected `meta` JSON
// (site/src/data/specs.ts is the single source), so the doc header cannot drift from the
// /specs index card.

// ---- injected metadata (from build-specs.mjs `--input meta=<json>`) -----------------------
#let _meta = json(bytes(sys.inputs.at("meta", default: "{}")))
#let _title = _meta.at("title", default: "Untitled Specification")
#let _status-label = _meta.at("statusLabel", default: "Unofficial Proposal Draft")
#let _short-name = _meta.at("shortName", default: "")
#let _editors = _meta.at("editors", default: "")
#let _date = _meta.at("date", default: "")

// The ONE canonical Status-of-This-Document statement. Empirical-honesty: an Unofficial
// Proposal Draft carries NO W3C standing — this text must say so, unhedged.
#let _sotd-body = [
  This document is published by the sparq project as an *Unofficial Proposal Draft*. It is
  #strong[not] a W3C standard and it is #strong[not] on the W3C Recommendation track. It has
  #strong[no official standing], is not endorsed by the W3C or any Working or Community Group,
  and confers no rights or obligations. It may be updated, replaced, or withdrawn at any time.

  It is published to invite discussion and to make sparq's design proposals reviewable in a
  familiar specification form; it is a proposal, not a ratified specification.
]

// ---- the document header (title + status + editors + date) --------------------------------
#let spec-head() = context {
  if target() == "html" {
    html.elem("header", attrs: (class: "spec-head"))[
      #html.elem("p", attrs: (class: "spec-status-line"))[#_status-label]
      #html.elem("h1", attrs: (class: "spec-title"))[#_title]
      #html.elem("dl", attrs: (class: "spec-meta"))[
        #if _short-name != "" [
          #html.elem("dt")[Short name]
          #html.elem("dd")[#html.elem("code")[#_short-name]]
        ]
        #if _date != "" [
          #html.elem("dt")[Date]
          #html.elem("dd")[#_date]
        ]
        #if _editors != "" [
          #html.elem("dt")[Editors]
          #html.elem("dd")[#_editors]
        ]
      ]
    ]
  } else {
    align(center)[
      #text(size: 10pt, weight: "bold", fill: rgb("#b00"))[#upper(_status-label)]
      #v(4pt)
      #text(size: 17pt, weight: "bold")[#_title]
      #v(4pt)
      #text(size: 9pt, fill: gray)[
        #if _short-name != "" [#_short-name · ]
        #if _date != "" [#_date · ]
        #if _editors != "" [Editors: #_editors]
      ]
    ]
    v(6pt)
  }
}

// ---- an introductory (unnumbered) section: Abstract, etc. ---------------------------------
// Unnumbered so build-specs.mjs's "numbered-heading" ToC discriminator excludes it (Abstract
// and the SOTD sit ABOVE the ToC in ReSpec, and are not listed in it).
#let intro-section(id, title, body) = context {
  if target() == "html" {
    html.elem("section", attrs: (class: "introductory", id: id))[
      #html.elem("h2")[#title]
      #body
    ]
  } else {
    heading(level: 2, numbering: none, outlined: false)[#title]
    body
  }
}

// ---- the Status-of-This-Document box (the honest "no W3C standing" banner) -----------------
#let sotd() = context {
  if target() == "html" {
    html.elem("section", attrs: (class: "introductory sotd", id: "sotd"))[
      #html.elem("h2")[Status of This Document]
      #_sotd-body
    ]
  } else {
    block(width: 100%, inset: 10pt, radius: 4pt, stroke: 0.75pt + rgb("#b00"), fill: rgb("#fff5f5"))[
      #heading(level: 2, numbering: none, outlined: false)[Status of This Document]
      #_sotd-body
    ]
    v(6pt)
  }
}

// ---- inline term definition (ReSpec <dfn>) ------------------------------------------------
#let dfn(term) = context {
  if target() == "html" { html.elem("dfn", attrs: (class: "respec-dfn"))[#term] } else { strong(emph(term)) }
}

// ---- a citation to a references entry (ReSpec [[KEY]] cross-reference) ---------------------
// HTML links to the `#ref-<key>` anchor `references()` emits; PDF renders plain bracketed text.
#let cite(key) = context {
  if target() == "html" {
    html.elem("a", attrs: (class: "spec-ref", href: "#ref-" + key))[[#key]]
  } else {
    strong[[#key]]
  }
}

// ---- an informative note aside (ReSpec .note) ---------------------------------------------
#let note(body) = context {
  if target() == "html" {
    html.elem("aside", attrs: (class: "note"))[
      #html.elem("span", attrs: (class: "note-title"))[Note]
      #body
    ]
  } else {
    block(width: 100%, inset: 8pt, radius: 3pt, stroke: (left: 3pt + rgb("#3a7")), fill: rgb("#f2fbf6"))[
      #strong[Note] — #body
    ]
  }
}

// ---- a bibliography reference row (ReSpec .ref) -------------------------------------------
// `entries` is an array of (key, text) pairs. Renders as a definition list in both targets.
#let references(entries) = context {
  if target() == "html" {
    html.elem("dl", attrs: (class: "spec-refs"))[
      #for (key, body) in entries [
        #html.elem("dt", attrs: (id: "ref-" + key))[[#key]]
        #html.elem("dd")[#body]
      ]
    ]
  } else {
    for (key, body) in entries {
      block(above: 6pt)[#strong[[#key]] #body]
    }
  }
}
