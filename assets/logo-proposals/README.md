# sparq logo — proposals

> Proposals only. The shipping logo at [`assets/logo.svg`](../logo.svg) is **unchanged**. These are
> candidate redesigns for the maintainer to pick from — nothing here replaces the current mark yet.

Produced with the `logo-designer` skill (Phase 2 explore → Phase 3 refine → Phase 4 export). Open
[`preview.html`](./preview.html) in a browser to see every concept at multiple sizes on both light and
dark backgrounds (the toggle previews dark mode, which the raster export tool cannot).

## Brief

sparq is a SOTA Rust RDF triplestore + SPARQL 1.1/1.2 engine. Differentiators: very fast + out-of-core;
**privacy** (zero-knowledge query proofs + MPC over federated SPARQL); production/security grade.

- **Type:** combination mark (icon + `sparq` wordmark) plus a derived standalone square icon.
- **Style:** minimal / geometric — modern dev-tool / infra feel.
- **Colour:** privacy-first **teal** to match `solid-pod-manager` + the feature site —
  primary `#1c7f86` ≈ `oklch(0.52 0.094 205)` (light) / `#4cc4cc` ≈ `oklch(0.7 0.1 200)` (dark).
  A single small **amber** energy accent (`#f5a623`) carries the original &ldquo;spark&rdquo;.
  Wordmark ink: `#11343a` (light) / `#e6f4f4` (dark).
- **Light/dark aware** via `@media (prefers-color-scheme:dark)`, the same mechanism as the current logo.
- **Wordmark:** lowercase `sparq` in a monospace stack (`ui-monospace,'JetBrains Mono',…`), keeping the
  code/dev feel of the original.

## Concepts

All combination marks are `viewBox="0 0 1024 512"`; both square icons are `viewBox="0 0 512 512"`.

| # | File | Idea | One-line rationale |
|---|------|------|--------------------|
| 1 | [`concepts/concept-1.svg`](./concepts/concept-1.svg) | **Refined graph + spark** | The current logo, tightened: even node radii, balanced shallow triangle, and the apex node *is* the amber spark instead of a bolt floating above a separate circle. Safest, most familiar. |
| 2 | [`concepts/concept-2.svg`](./concepts/concept-2.svg) | **Graph forms a &ldquo;q&rdquo;** | A hexagonal ring of nodes is the bowl of a lowercase `q`; a single edge descends to an amber terminus node as the tail. q = query → SPAR**Q**. Distinctive, ownable, still literally an RDF graph. |
| 3 | [`concepts/concept-3.svg`](./concepts/concept-3.svg) | **Privacy / proof shield** | An RDF triple knocked out as true negative space inside an escutcheon shield; the top node is amber = the *verified* node. Directly signals the ZK / MPC privacy + verifiability differentiator. Strongest small-size silhouette. |
| 4 | [`concepts/concept-4.svg`](./concepts/concept-4.svg) | **Spark is the edge (speed)** | The amber lightning bolt becomes the connecting *edge* zig-zagging through three nodes — a fast query racing through the triplestore. Most energetic. |
| 5 | [`concepts/concept-5.svg`](./concepts/concept-5.svg) | **Abstract triple** | Subject (amber circle) → predicate (edges) → object (rounded diamond): the atomic RDF triple as a confident, mark-like symbol. Most abstract / brandable. |

### Standalone square icons

- [`icons/icon-q.svg`](./icons/icon-q.svg) — square icon for Concept 2.
- [`icons/icon-shield.svg`](./icons/icon-shield.svg) — square icon for Concept 3.

## Design rationale

The current mark is clean but generic — a plain 3-node graph with a small floating bolt, in neutral
ink. The redesign keeps sparq's two true brand assets (the **RDF graph** and the **spark**) but pushes
each concept toward something more *distinctive* and better optically balanced, and re-skins everything
onto the privacy-teal palette so the engine, `solid-pod-manager`, and the feature site read as one family.

Robustness choices that matter for a real logo:

- **Direct class fills + a `prefers-color-scheme` override**, not CSS `var(--…)` custom properties.
  Custom properties are silently dropped by several SVG renderers (favicons, some raster tools, embeds),
  which would make the mark fall back to black. The direct-fill pattern matches the existing
  `assets/logo.svg` and renders everywhere.
- **Solid fills and thick strokes** (≥14–30 px in the working coordinate space) so nothing thins out at
  favicon sizes. The shield graph is *negative space* via an SVG `<mask>`, so it stays transparent on any
  background.
- **Wordmark sized to fit** the 1024-wide canvas with right padding (the first drafts overflowed the `q`).

## Recommendation

**Primary: Concept 2 (graph-as-&ldquo;q&rdquo;).** It is the most *ownable* of the set — it is unmistakably
an RDF graph (true to what sparq is) yet resolves into a `q`, tying the icon to SPAR**Q** / query in a way
that feels discovered rather than forced. It carries the amber spark forward as the query-terminus node, so
the brand&rsquo;s energy cue survives. It is modern, geometric, and distinct from the generic
&ldquo;dots-and-lines&rdquo; that every graph-DB uses.

**Pair it with Concept 3 (shield) as the favicon / app-icon.** At 16–32 px the q-graph reads as a node
cluster before the `q` resolves (~24 px+), whereas the shield keeps a bold, instantly-recognisable
silhouette at the smallest sizes and doubles as a banner for the privacy/ZK story. Using the q-lockup for
wordmark contexts and the shield for tiny square contexts is a coherent, common pattern.

If the maintainer prefers minimal risk, **Concept 1** is the safe evolution of today&rsquo;s logo and would
ship without surprising anyone.

## Exports

PNGs of the lead concept are in [`export/`](./export/): the square icon `icon-q-{16,32,48,192,512,1024}.png`
and the lockup `lockup-q-{512,1024}.png`. These are **light-mode only** — the raster tool used (resvg/sharp)
does not evaluate `prefers-color-scheme`, so it always renders the light palette; dark mode is correct in the
browser preview and in any `prefers-color-scheme`-aware context. The skill&rsquo;s export script is vendored at
[`scripts/export.sh`](./scripts/export.sh) for regeneration.

## Files

```
assets/logo-proposals/
├── concepts/        concept-1..5.svg   (combination marks, 1024×512)
├── icons/           icon-q.svg, icon-shield.svg   (square, 512×512)
├── export/          PNGs of the lead concept (light mode)
├── scripts/         export.sh (from the logo-designer skill)
├── preview.html     side-by-side, all sizes, light + dark
└── README.md        this file
```
