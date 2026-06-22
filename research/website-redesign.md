# sparq website redesign — design record (design-for-review)

> Status: **DESIGN-ONLY, for maintainer review.** Nothing here is built yet. This is the
> synthesis deliverable for the **website-redesign epic** (a sibling of the GUI epic
> `sq-ixc3`); the decomposed implementation beads it steers are tracked in
> `.beads/issues.jsonl`. Authored by Claude Opus 4.8 while Fable was unavailable — flagged
> `[OPUS-4.8]` for later re-review, especially the ZK/MPC honesty framing.
>
> **The reusable method behind this record lives in
> [`.claude/skills/frontend-design/SKILL.md`](../.claude/skills/frontend-design/SKILL.md);
> this record is the concrete website spec.** The distinct operational GUI is
> [`research/gui-design.md`](gui-design.md) — read both together; the contrast is the point.

Non-canonical timing: no measured/benchmark numbers appear here (this work box is
non-canonical). A wasm bundle *size* is a build-output fact, not a benchmark.

---

## 0. The complaint this redesign answers

The maintainer's verdict on the current site: **"there is too much text; it is
overwhelming."** That complaint is **structurally true**, not a matter of taste. Audited
ground truth (`site/src/components/layout/app-shell.tsx`):

- A persistent `w-64` **left sidebar** renders the **full 6-group / 16-surface tree**
  (`sidebar-nav.tsx`), AND
- a **top-tab bar** repeats *Showcase / Benchmarks / About* (`app-shell.tsx` `TopTab`s), AND
- the **landing page** (`page.tsx`) re-renders the surfaces as **~14 cards** below a
  **4-sentence honesty preamble** plus the 3 flagships.

So the *same* 16 surfaces appear in **up to three places at once**, and every inner
`/surface/*` page is forced through one rigid template
(`surface-content.tsx`: intro **+** 6 capability cards **+** an always-open "How this runs"
card **+** an always-open "Honest caveats" card) regardless of how simple the surface is.
The overwhelm is **tripled navigation + a template that cannot shrink + prose-first inner
pages** — not merely verbose copy. **Cutting structure, not words, is the fix.**

This redesign is judged against one question: *does a stranger now grasp what sparq is, see
one proof, and reach any feature in ≤2 clicks — without reading a wall?*

---

## 1. North star

> For the **RDF/SPARQL-curious developer** evaluating *"is this real, and should I care?"*,
> the website is a **scannable marketing-docs site** that answers *what-is-sparq* in one
> line, proves it with a few **live runnable** examples, and **links out** to the repo /
> `skills/*/SKILL.md` / `crates/*/README.md` / papers for depth. It is **not** the
> operational workbench — that is the **GUI** (`research/gui-design.md`). The website
> *persuades and routes*; the docs and the engine *exhaust and operate*.

Distinctness from the GUI (the load-bearing boundary):

| | Website (this doc) | GUI (`gui-design.md`) |
|---|---|---|
| Question it answers | "should I care, is it real?" | "let me do RDF/SPARQL work" |
| Shape | a few **pages** read top-to-bottom | a **workbench shell** (rails + tabs + Cmd-K) |
| Data | sample graphs in a live REPL demo | persistent imported native store |
| Surfaces | persuasive **previews** that link to depth | operational **tools** opened as tabs |
| Depth | linked out to README/SKILL/papers | the engine itself; Help opens the website |

If a website change makes it feel more like a workbench, it is going the wrong way.

---

## 2. The simpler information architecture

**ONE nav.** Replace the *sidebar tree + top-tab bar* with a single slim **top bar** of
**5 content destinations** + a utility cluster. There is no persistent full sidebar.

```text
[sparq logo] · Home · Examples · Capabilities · Benchmarks · Papers   {Cmd-K · GitHub · theme}
```

Routes (the real page count — see the honesty note below):

- **`/`** — Home. One-line what-is-it → live REPL → 3 flagship cards → 5 capability THEME
  cards → a single `#how-it-runs` tier-legend strip. (rewrite `site/src/app/page.tsx`)
- **`/try`** — the full live REPL (kept; the single best proof artifact, `repl.tsx`).
- **`/examples`** — NEW curated "show me it working" gallery: the 3 flagships + `/try`,
  large demo cards. The `/showcase/{zk-car-hire,mpc-100k,solid-pairs}` pages stay as the
  detail targets, rebuilt scan-first.
- **`/capabilities`** — NEW single compact gallery replacing the 14 dense `/surface/*`
  pages. 5 theme sections; each surface = one **row** (title · one-line blurb · tier badge ·
  *Demo ▸* disclosure **or** *Open →* for the 5 deep pages).
- **`/capabilities/{sparql,shacl,inference,data-formats,javascript-wasm}`** — the **5
  retained deep pages** (the live-in-your-tab proof surfaces), rebuilt scan-first via a
  disclosure-based `surface-content.tsx`.
- **`/benchmarks`** (+`/benchmarks/[type]`) — kept structurally; already card-per-type with
  honest per-commit/same-box provenance. Light touch: read from the new top nav.
- **`/papers`** (+`/papers/[slug]`) — kept; already well-structured.

**Removed as nav destinations:** `/about` (its "what runs where" tier table folds into the
Home `#how-it-runs` strip + a "How tiers work" `<details>`); the **11 walkthrough
`/surface/*` routes** collapse into `/capabilities` rows.

**Hierarchy depth:** Home → theme → surface row → (optional) demo disclosure. **Max 2 clicks
to any demo; Cmd-K reaches any surface by name in 0 clicks.**

> **Honesty about the page count (reviewer must_fix).** "5 destinations" is the *nav*, not
> the maintenance surface. The site still has ~11 content surfaces a maintainer keeps
> scan-first: Home, `/try`, `/examples`, `/capabilities`, **5** deep pages, **3** showcase
> details, `/benchmarks`, `/papers`. The win is real (16 routes → ~10–11, one nav not three,
> one gallery not 14 dense pages) but it is **not** literally "just 5 pages." Do not sell it
> as such.

The 5 capability THEMES (re-grouped **once** at the single source
`site/src/data/surfaces.ts` `GROUPS`, so the Home grid, `/capabilities`, and Cmd-K all
update together):

1. **Query & data** — SPARQL 1.1/1.2 · data-formats · JavaScript/WASM · live REPL
2. **Reason & validate** — Inference (RDFS/OWL-RL/N3) · SHACL
3. **Search & GenAI** — Full-text (BM25) · Vector · GenAI/NLQ · structural-similarity
4. **Privacy (ZK / MPC)** — ZK query proofs · MPC threshold · Solid (user,app) pairs
5. **Serve & embed** — HTTP server · CLI · Python · streaming RSP-QL · federation

---

## 3. The content-reduction plan (what to CUT / COLLAPSE / MOVE)

This is the commitment, not a vibe. Each item names the concrete artifact and applies the
reviewers' `must_fix`.

### CUT (delete from the site)

- **The whole "Every surface" 14-card grid on Home** (`page.tsx` surface-card loop). Moves
  to `/capabilities`. The landing stops being a 19-thing directory.
- **The 4-sentence honesty preamble in the hero** (`page.tsx`). Becomes ONE success badge +
  the one-line tier-legend strip in `#how-it-runs`.
- **The persistent full-tree sidebar** (`sidebar-nav.tsx` as a shell fixture) and the
  **duplicate top tabs**. One slim top bar remains. *(Gated on Cmd-K shipping first — see
  Risks.)*
- **The per-page "How this runs" card and "Honest caveats" card as always-open co-equal
  blocks** on every surface page. The runsNote shrinks to one sentence; the caveat moves
  behind a closed `<details>`.
- **The forced 6 capability ring-cards** per surface. Become a tight bulleted list (≤5,
  bolded lead term).

### COLLAPSE (keep the content, change its form)

- **14 dense `/surface/*` pages → 1 `/capabilities` gallery** of expand-in-place rows. The
  11 walkthrough surfaces become rows whose *Demo ▸* disclosure mounts the EXISTING
  interactive component inline (`geosparql-walkthrough.tsx`, `vector-walkthrough.tsx`,
  `mpc-demo.tsx`, `text-playground.tsx`, …). **`must_fix` (hard requirement): each
  disclosure MUST `next/dynamic` / `React.lazy` its component so the chunk loads ONLY on
  expand**, explicitly preserving the route-scoping `sidebar-nav.tsx` already enforces for
  the ZK prover (`bb.js`/`noir_js` is ~MB). Without lazy-mount, `/capabilities` is heavier
  than the 14 pages it replaces — that would defeat the redesign.
- **Multi-paragraph `intro` blocks → one 1–2 sentence lead.** Concretely: the GeoSPARQL
  intro (~3 paragraphs / ~30 inline `<code>` spans) → one sentence; the DE-9IM family
  enumeration, topology-rewrite explanation, and feature-gating detail move to the
  `<details>` or out to the SKILL.
- **The 5-clause ZK caveat and the MPC caveat → a one-line flagged summary + disclosure.**
  Surface "Research-grade — not externally audited" as the badge/headline (already present);
  the soundness/linkability paragraph and the MPC "faithful simulation, correctness layer is
  a stub" detail move behind a "Full security caveat" `<details>`. (This stays compliant
  with `scripts/check-privacy-claims.sh` — the qualifier is preserved, just relocated.)
- **The tier explanation told four times → told once** in `#how-it-runs`; the badge is the
  per-page pointer; each per-page "How this runs" becomes one sentence + a repro link.

### MOVE (off the site, to the asset that already holds it)

- **Reproduction commands, the full capability matrix, edge-case boundaries, feature-gating
  minutiae** → `crates/<x>/README.md` + `skills/<x>/SKILL.md` (the ZK and GeoSPARQL pages
  already link out — make it **universal**: every surface row links its README + SKILL).
- **Threat-model nuance, prior art, the rationale itself** → `research/*.md`.
- **The `/about` "what runs where" prose** → the Home `#how-it-runs` "How tiers work"
  `<details>`.

### One-line summary of the cut

> Home goes from *{4-sentence preamble + 3 flagships + 14-card grid}* to *{1 sentence + 1
> badge + live REPL + 3 flagships + 5 theme cards + 1 tier strip}*; the **14 dense surface
> pages collapse into ONE `/capabilities` gallery** of lazy-expand rows plus **5** rebuilt
> deep pages; the **tripled nav becomes one slim top bar + Cmd-K**; and all reference
> material (repro commands, full matrices, threat-model hedging) **moves to the README /
> SKILL / research / papers that already exist.**

---

## 4. Page specs

### HOME (`/`) — route + prove in under one screen

Top-to-bottom: (1) **Hero** — ONE sentence ("sparq is a state-of-the-art RDF triplestore +
SPARQL 1.1/1.2 engine that runs in your browser tab") + one success badge + exactly **2
buttons** `[Try the REPL]` `[GitHub]`; the 3 one-line feature pills stay. (2) **The live
REPL** immediately (`repl.tsx`), one-line caption kept — the killer artifact, above the
fold-ish. (3) **"See it work"** — the 3 FLAGSHIP cards at large prominence, linking to
`/examples` detail pages. (4) **"What sparq can do"** — the **5 capability THEME cards**,
each listing its 2–4 surfaces as plain text links to `/capabilities#<theme>`. (5)
**`#how-it-runs`** — ONE compact tier-legend strip (live / walkthrough / hosted / sim) + a
"How tiers work" `<details>` holding the `/about` prose. **CUT:** the 14-card grid, the
4-sentence preamble, the duplicate tier explanation.

### EXAMPLES (`/examples`)

One-line lede; a 2-col grid of large demo cards = the 3 flagships + a "Live REPL" card. Each
card: title, one-line **outcome** ("Prove you may hire a car without revealing your
documents"), tier badge, a still/thumbnail, `[Open demo]`. No prose walls. The
`/showcase/*` pages are the detail targets, rebuilt **demo-first** (demo → one-line caption →
caveat behind `<details>`).

### CAPABILITIES (`/capabilities`) — the big consolidation

Page title + one line; then **5 theme sections**. Each theme = a header + a list of surface
**rows** (~64px): `[icon] Title — one-line blurb · [tier badge] · [Demo ▸ | Open →]`.
*Demo ▸* **lazily** mounts the surface's existing interactive component inline (no
navigation); the 5 live-tier surfaces use *Open →* to their deep page. **CUT/MOVED per
surface:** every multi-paragraph `intro`, the 6-card capability grid, the always-open "How
this runs" + "Honest caveats" cards — replaced by (a) the one-line blurb, (b) a "Details &
caveats" `<details>` per expanded demo holding ONE caveat sentence + README + SKILL links.

### CAPABILITY DEEP PAGE (`/capabilities/{sparql,shacl,inference,data-formats,javascript-wasm}`)

Rebuild `surface-content.tsx` to a **scan-first, disclosure-based** template with **all
content blocks OPTIONAL props**. Order: (1) title + tier badge + ONE-sentence statement;
(2) the interactive demo IMMEDIATELY (`children`); (3) a tight Capabilities list (bolded
lead term + short clause, max ~5 — **not** 6 ring-cards); (4) "How this runs" as a **single
sentence** whose authority is the badge ("Runs live in your tab via the lean wasm bundle —
reproduce: `cargo test -p sparq-engine`"); (5) caveats as a **closed `<details>`**. A simple
surface renders just *statement + demo + one-line note*.

### BENCHMARKS (`/benchmarks`) — keep

No structural change; already card-per-type with honest per-commit / same-box provenance.
Keep that framing verbatim. Light touch only: read from the new top nav.

### PAPERS (`/papers`) — keep

Already well-structured. No change beyond nav.

---

## 5. Visual design system (shared with the GUI; see the skill)

- **Tokens, not hex.** Tailwind v4 + the shadcn/radix tokens already in `site/src`; light/
  dark via `theme-provider.tsx`. Don't hardcode colours.
- **The honesty-tier badge is a first-class component** — `TIER_LABEL` / `TIER_VARIANT`
  (`surfaces.ts`): `success`=live, `warning`=hosted, `muted`=walkthrough, `default`=sim. The
  tier taxonomy is preserved **verbatim**; the badge carries the per-page truth so prose
  doesn't have to.
- **Marketing density:** roomy (`max-w-6xl`, generous padding) — the *opposite* of the GUI's
  dense IDE chrome. This visual contrast is intentional and reinforces "different product."
- **Cmd-K command palette** (new — `cmdk` dep): a `CommandDialog` indexing the single
  `GROUPS` source. It is the discoverability backstop that lets us shrink the nav.
- **One icon per surface** (`lucide-react`, declared in `surfaces.ts`); shared code
  highlighters (`sparql-editor.tsx`, `rdf-highlight.tsx`, `pretty-turtle`).

---

## 6. Accessibility + performance budget

- **A11y:** `<nav aria-label>` + `aria-current` on the active item; focus-visible rings;
  ESC-closes the Cmd-K dialog; `sr-only` titles on icon-only controls (the shell already
  does several — keep them). Colour is never the sole tier signal — keep the text label.
- **Perf:** static Pages bundle — keep the **main bundle wasm-free** (async-load the
  ~2.5 MB engine wasm *after* page load; the open beads `sq-4296` / `sq-55w5a` already track
  this); **lazy-load every heavy demo chunk on-expand** (the `/capabilities` `must_fix`).
  Preserve `images:{unoptimized:true}` + `basePath:"/sparq"` (load-bearing for Pages/Tauri).
- **Honesty gates:** `scripts/check-privacy-claims.sh` (ZK/MPC stay qualified), the
  terminology gate, and the no-hard-coded-perf rule gate all user-facing copy — write copy
  that passes them by construction. The COLLAPSE moves keep every qualifier; they just
  relocate it behind a disclosure.

---

## 7. Risks + sequencing (reviewer must_fix applied)

1. **Cmd-K is a prerequisite, not a nicety.** It does not exist today (no `cmdk` dep, no
   `CommandDialog`). It is the *only* fast path to the 11 surfaces that lose their route.
   **Land Cmd-K (cmdk dep + `CommandDialog` wired to `GROUPS`) BEFORE deleting the
   sidebar.** If it slips, keep the sidebar until it lands.
2. **`/capabilities` must lazy-mount** (§3 COLLAPSE) — stated as a hard requirement, not an
   implementation detail. Verify with a bundle check that no demo chunk loads on route
   entry.
3. **Static export cannot 301.** `output:"export"` has no server. Every removed route
   (`/about`, the 11 `/surface/*`) needs a **client-side redirect stub** at the old path (or
   the `[slug]` catch-all `dynamicParams=false` hard-404s inbound/cross-page links). Ship the
   stubs **with** the deletion.
4. **Sequence, don't big-bang.** This is a large refactor (rewrite `page.tsx`, new
   `/examples`, new `/capabilities` consolidating 14 pages, rebuild `surface-content.tsx`
   with optional props, new Cmd-K, redirect stubs). Land it as a sequence of small PRs
   (Cmd-K → re-group `GROUPS` → `/capabilities` lazy gallery + stubs → Home rewrite → nav
   collapse → deep-page rebuild), each green, not one long-lived branch. The decomposed beads
   encode this order.

---

## 8. Key file citations (ground truth)

- The audited overwhelm: `site/src/components/layout/app-shell.tsx` (sidebar + top tabs),
  `site/src/components/layout/sidebar-nav.tsx` (full tree + ZK route-scoping),
  `site/src/app/page.tsx` (preamble + 14-card grid), `site/src/components/surface-content.tsx`
  (the rigid template).
- Single IA source: `site/src/data/surfaces.ts` (`Tier`, `GROUPS`, `FLAGSHIPS`,
  `TIER_LABEL`, `TIER_VARIANT`).
- The killer artifact + reusable demos: `site/src/components/repl.tsx`,
  `sparql-editor.tsx`, the per-surface `*-walkthrough.tsx` / `*-playground.tsx`,
  `zk-car-hire.tsx`, `mpc-demo.tsx`, `solid-pairs-demo.tsx`.
- Static-export config (don't break): `site/next.config.ts`.
- Honesty gates: `scripts/check-privacy-claims.sh`; the terminology + no-perf gates in
  `.github/workflows/docs-quality.yml`.
- Reusable method: `.claude/skills/frontend-design/SKILL.md`. Distinct GUI: `research/gui-design.md`.
- Prior site design: `research/feature-showcase-site-design.md` (the build this redesigns).
- Open perf beads already filed: `sq-4296`, `sq-55w5a` (async/lazy wasm load).
