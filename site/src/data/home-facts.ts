// [OPUS-4.8] sq-vw3ax.5 — single source for the homepage's STRUCTURAL facts so the hero
// stat strip never hardcodes a count inline. These are structural facts about the engine,
// NOT performance claims and NOT fabricated: the four RDF *text* formats the lean wasm
// bundle parses/serialises (mirrors the `data-formats` surface blurb in data/surfaces.ts:
// "Turtle / N-Triples / N-Quads / TriG"). The hero derives `4` from this list's length, so
// the displayed number and the listed formats can never drift apart.

export const RDF_TEXT_FORMATS = [
  "Turtle",
  "N-Triples",
  "N-Quads",
  "TriG",
] as const;
