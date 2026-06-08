# Measured: is FSST/front-coding worth it on real Wikidata?

Measured on a 2M-triple real Wikidata `truthy` sample (M1):

| | value |
|---|--:|
| distinct terms | 955,530 |
| of which IRIs | 147,518 (15%) |
| of which **literals** | **808,012 (85%)** |
| dict footprint (prefix-factored) | **80 B/term** |
| raw distinct-term bytes | 36 B/term |
| distinct IRI prefixes | 8,571 (top: wikidata.org/entity/, schema.org, skos) |
| IRI **suffix** bytes after prefix-factoring | **11 B/suffix** |

**Conclusion — FSST is a modest (~20%), browser-only lever here, not the big win.**
Prefix factoring already captured the IRI redundancy that FSST/front-coding target:
the long shared `http://www.wikidata.org/entity/` etc. are stored once, and the
per-IRI suffix is already down to ~11 bytes. What remains is (a) the fixed `Stored`
enum slot (~40 B, sized for the literal variant) and (b) literal *value* strings
(85% of terms — labels/descriptions). FSST would compress only (b)'s payload, ~2× on
the string bytes → roughly **80 → ~64 B/term (~20%)**, at the cost of compress-on-lookup
and decompress-on-output per query. That trade is worth it ONLY for the memory-bound
browser, and even there it competes with cheaper wins:
- **Typed/sectioned dictionary** (HDT four-section): IRIs in a 20 B slot instead of the
  shared 40 B enum — saves the variant padding on the 15% IRI terms.
- **More tagged ValueIds** (dates/decimals): Wikidata literals are date/quantity-heavy;
  inlining them removes whole dict entries — but the u32 inline space is exhausted, so
  this needs the u64-id decision (which doubles permutation memory — measured-risky).

So the dict is already near the cheap-win frontier on real data; the remaining dict
levers are modest or memory-gated. Recorded so we don't over-invest in FSST.

## Correctness on real Wikidata (sparq vs Oxigraph, 2M triples)
| query | rows | matches Oxigraph |
|---|--:|:--:|
| `?s schema:name ?n` (scan) | 408,836 | ✓ |
| `?s schema:name ?n . ?s schema:description ?d` (join) | 21,470,933 | ✓ |
